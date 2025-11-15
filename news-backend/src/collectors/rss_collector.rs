use crate::collectors::web_parser::WebParser;
use crate::models::raw_document::ArticleMetadata;
use anyhow::{Context, Result};
use atom_syndication::Feed as AtomFeed;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rss::Channel;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Cliente para coleta de artigos via RSS feeds
pub struct RssCollector {
    client: Client,
    temp_dir: PathBuf,
}

type FeedItem = (String, Option<DateTime<Utc>>, Option<String>, String);

impl RssCollector {
    /// Cria novo cliente RSS
    pub fn new(temp_dir: PathBuf) -> Self {
        // Configure client to handle SSL errors (unrecognized name, etc.)
        // This is needed for some servers with SSL certificate issues
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true) // Accept invalid SSL certificates
            .danger_accept_invalid_hostnames(true) // Accept invalid hostnames (fixes "unrecognized name" error)
            .build()
            .expect("Failed to create RSS client");
        
        Self {
            client,
            temp_dir,
        }
    }

    /// Busca artigos de um feed RSS
    ///
    /// # Arguments
    /// * `feed_url` - URL do feed RSS/Atom
    /// * `max_results` - Número máximo de artigos a retornar
    /// * `base_url` - URL base para construir URLs relativas (opcional)
    pub async fn fetch_feed(
        &self,
        feed_url: &str,
        max_results: Option<u32>,
        _base_url: Option<&str>,
    ) -> Result<Vec<ArticleMetadata>> {
        let max = max_results.unwrap_or(10);

        info!(url = %feed_url, max_results = max, "Fetching RSS feed");

        let response = self.client.get(feed_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch RSS feed: HTTP {}",
                response.status()
            ));
        }

        let feed_content = response.text().await?;

        // Verificar se é realmente um feed RSS/Atom e não HTML
        let content_start = feed_content.trim_start();
        if !content_start.starts_with("<?xml")
            && !content_start.starts_with("<feed")
            && !content_start.starts_with("<rss")
        {
            warn!(
                url = %feed_url,
                "Feed URL returned HTML instead of RSS/Atom feed"
            );
            return Err(anyhow::anyhow!(
                "Feed URL '{}' returned HTML instead of RSS/Atom feed. This may not be a valid feed URL. Consider using HTML collector instead.",
                feed_url
            ));
        }

        // Salvar XML temporário para debug
        let temp_file = self
            .temp_dir
            .join(format!("rss_feed_{}.xml", chrono::Utc::now().timestamp()));
        tokio::fs::write(&temp_file, &feed_content)
            .await
            .context("Failed to save RSS feed XML")?;

        info!(temp_file = %temp_file.display(), "Saved RSS feed XML");

        // Try to parse as RSS first, then Atom
        let mut articles = Vec::new();
        let items_to_process: Vec<FeedItem>;

        // Try RSS first
        match Channel::read_from(feed_content.as_bytes()) {
            Ok(channel) => {
                info!(
                    feed_title = %channel.title(),
                    item_count = channel.items().len(),
                    "Parsed RSS feed"
                );

                items_to_process = channel
                    .items()
                    .iter()
                    .take(max as usize)
                    .map(|item| {
                        let url = item
                            .link()
                            .map(|s| s.trim().to_string())
                            .or_else(|| item.guid().map(|g| g.value().trim().to_string()))
                            .unwrap_or_default();

                        let title = item.title().unwrap_or("Untitled").trim().to_string();
                        let date = item.pub_date().and_then(|date_str| {
                            DateTime::parse_from_rfc2822(date_str)
                                .or_else(|_| DateTime::parse_from_rfc3339(date_str))
                                .map(|dt| dt.with_timezone(&Utc))
                                .ok()
                        });
                        let author = item.author().map(|s| s.trim().to_string());

                        (url, date, author, title)
                    })
                    .collect();
            }
            Err(_) => {
                // Try Atom
                match feed_content.parse::<AtomFeed>() {
                    Ok(feed) => {
                        info!(
                            feed_title = %feed.title.value,
                            entry_count = feed.entries.len(),
                            "Parsed Atom feed"
                        );

                        items_to_process = feed
                            .entries
                            .iter()
                            .take(max as usize)
                            .map(|entry| {
                                // Buscar link com rel="alternate" primeiro (link do artigo)
                                // Se não encontrar, usar o primeiro link disponível
                                let url = entry
                                    .links
                                    .iter()
                                    .find(|link| link.rel == "alternate")
                                    .map(|link| link.href.clone())
                                    .or_else(|| entry.links.first().map(|link| link.href.clone()))
                                    .unwrap_or_default();

                                let title = entry.title.value.clone();
                                let date = Some(entry.updated.with_timezone(&Utc));
                                let author = entry.authors.first().map(|a| a.name.clone());

                                (url, date, author, title)
                            })
                            .collect();
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to parse RSS or Atom feed: {}", e));
                    }
                }
            }
        }

        // Process items (from RSS or Atom)
        for (idx, (url, published_date, author, title)) in items_to_process.into_iter().enumerate()
        {
            if url.is_empty() {
                warn!(index = idx + 1, title = %title, "Skipping item with no URL");
                continue;
            }

            // Create ArticleMetadata from parsed data
            let article_id = WebParser::generate_id_from_url(&url)
                .unwrap_or_else(|| WebParser::generate_id_from_title(&title));

            let title_clone = title.clone();
            let mut article = ArticleMetadata {
                id: article_id,
                title: title_clone.clone(), // Mantido para compatibilidade
                original_title: Some(title_clone), // Título original da fonte
                generated_title: None,      // Será preenchido quando o artigo for publicado
                url: url.clone(),
                published_date,
                author,
                summary: None,
                image_url: None,
                source_type: Some("rss".to_string()),
                content_html: None,
                content_text: None,
                category: None,
                slug: None,
            };

            // Fetch full article content from URL - REQUIRED
            match self.fetch_article_content(&article.url).await {
                Ok(full_content) => {
                    article.content_html = Some(full_content.0.clone());
                    article.content_text = Some(full_content.1.clone());

                    // Verificar se o conteúdo é válido (não vazio e tem tamanho mínimo suficiente para escrever notícia)
                    // Mínimo de 1000 caracteres para garantir conteúdo completo
                    const MIN_CONTENT_LENGTH: usize = 1000;
                    if full_content.1.trim().is_empty() || full_content.1.len() < MIN_CONTENT_LENGTH
                    {
                        warn!(
                            index = idx + 1,
                            title = %article.title,
                            url = %article.url,
                            content_length = full_content.1.len(),
                            min_required = MIN_CONTENT_LENGTH,
                            "Article content too short or empty, skipping"
                        );
                        continue; // Ignorar artigo sem conteúdo válido
                    }

                    info!(
                        index = idx + 1,
                        title = %article.title,
                        url = %article.url,
                        content_length = full_content.1.len(),
                        "Fetched full article content"
                    );
                    articles.push(article);
                }
                Err(e) => {
                    warn!(
                        index = idx + 1,
                        title = %article.title,
                        url = %article.url,
                        error = %e,
                        "Failed to fetch full article content, skipping article"
                    );
                    // Ignorar artigo sem conteúdo completo
                    continue;
                }
            }

            // Rate limiting: delay entre requisições
            if idx < max as usize - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        // Old code removed - items processed above
        /*
        for (idx, item) in channel.items().iter().take(max as usize).enumerate() {
            match WebParser::parse_rss_item(item, base_url) {
                Ok(mut article) => {
                    // Fetch full article content from URL - REQUIRED
                    match self.fetch_article_content(&article.url).await {
                        Ok(full_content) => {
                            article.content_html = Some(full_content.0.clone());
                            article.content_text = Some(full_content.1.clone());

                            // Verificar se o conteúdo é válido (não vazio e tem tamanho mínimo suficiente para escrever notícia)
                            // Mínimo de 1000 caracteres para garantir conteúdo completo
                            const MIN_CONTENT_LENGTH: usize = 1000;
                            if full_content.1.trim().is_empty() || full_content.1.len() < MIN_CONTENT_LENGTH {
                                warn!(
                                    index = idx + 1,
                                    title = %article.title,
                                    url = %article.url,
                                    content_length = full_content.1.len(),
                                    min_required = MIN_CONTENT_LENGTH,
                                    "Article content too short or empty, skipping"
                                );
                                continue; // Ignorar artigo sem conteúdo válido
                            }

                            info!(
                                index = idx + 1,
                                title = %article.title,
                                url = %article.url,
                                content_length = full_content.1.len(),
                                "Fetched full article content"
                            );
                            articles.push(article);
                        }
                        Err(e) => {
                            warn!(
                                index = idx + 1,
                                title = %article.title,
                                url = %article.url,
                                error = %e,
                                "Failed to fetch full article content, skipping article"
                            );
                            // Ignorar artigo sem conteúdo completo
                            continue;
        */

        info!(count = articles.len(), "Extracted articles from feed");

        Ok(articles)
    }

    /// Busca artigos de múltiplos feeds RSS
    ///
    /// # Arguments
    /// * `feeds` - Vetor de tuplas (feed_url, max_results, base_url)
    #[allow(dead_code)]
    pub async fn fetch_multiple_feeds(
        &self,
        feeds: Vec<(String, Option<u32>, Option<String>)>,
    ) -> Result<Vec<ArticleMetadata>> {
        let mut all_articles = Vec::new();

        let feeds_count = feeds.len();
        for (feed_url, max_results, base_url) in feeds {
            match self
                .fetch_feed(&feed_url, max_results, base_url.as_deref())
                .await
            {
                Ok(articles) => {
                    all_articles.extend(articles);
                }
                Err(e) => {
                    error!(
                        feed_url = %feed_url,
                        error = %e,
                        "Failed to fetch RSS feed"
                    );
                }
            }

            // Rate limiting: delay entre feeds
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        let total_feeds = feeds_count;
        info!(
            total_feeds = total_feeds,
            total_articles = all_articles.len(),
            "Fetched all RSS feeds"
        );

        Ok(all_articles)
    }

    /// Busca conteúdo completo do artigo pela URL
    /// Retorna (HTML, texto extraído)
    async fn fetch_article_content(&self, url: &str) -> Result<(String, String)> {
        use crate::collectors::web_parser::WebParser;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {}", response.status()));
        }

        let html_content = response.text().await?;

        // Parse HTML para extrair conteúdo principal
        let document = scraper::Html::parse_document(&html_content);

        // Tentar seletores comuns para conteúdo de artigo (em ordem de preferência)
        // Priorizar seletores que geralmente contêm o conteúdo completo do artigo
        let content_selectors = vec![
            "article main",          // Artigo com main content
            "main article",          // Main com article
            "article .article-body", // Body do artigo
            "article .content",      // Conteúdo do artigo
            "article",               // Tag article completa
            ".article-content",      // Conteúdo do artigo
            ".post-content",         // Conteúdo do post
            ".entry-content",        // Conteúdo da entrada
            "main .content",         // Main com content
            ".content",              // Classe content genérica
            "main",                  // Tag main
        ];

        let mut content_html = String::new();
        let mut content_text = String::new();
        const MIN_CONTENT_LENGTH: usize = 1000; // Mínimo necessário para conteúdo completo

        // Tentar cada seletor até encontrar conteúdo suficiente
        for selector_str in content_selectors {
            if let Ok(selector) = scraper::Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let candidate_html = element.html();
                    let candidate_text = WebParser::extract_text_from_html(&candidate_html);

                    // Se encontrou conteúdo significativo, usar este
                    if !candidate_text.trim().is_empty()
                        && candidate_text.len() >= MIN_CONTENT_LENGTH
                    {
                        content_html = candidate_html;
                        content_text = candidate_text;
                        info!(
                            selector = selector_str,
                            length = content_text.len(),
                            "Found article content"
                        );
                        break;
                    }
                    // Se ainda não temos nada ou este é maior, usar este (mas continuar procurando melhor)
                    else if candidate_text.len() > content_text.len()
                        && candidate_text.len() >= 500
                    {
                        content_html = candidate_html;
                        content_text = candidate_text;
                    }
                }

                // Se já encontrou conteúdo suficiente, parar
                if content_text.len() >= MIN_CONTENT_LENGTH {
                    break;
                }
            }
        }

        // Fallback: usar body se não encontrou conteúdo específico suficiente
        // Extrair texto do body, o extract_text_from_html já remove scripts/styles automaticamente
        if content_text.len() < MIN_CONTENT_LENGTH
            && let Ok(body_selector) = scraper::Selector::parse("body")
            && let Some(body) = document.select(&body_selector).next()
        {
            content_html = body.html();
            content_text = WebParser::extract_text_from_html(&content_html);

            info!(
                length = content_text.len(),
                "Using body content as fallback"
            );
        }

        // Apply content cleaning to remove noise (buttons, navigation, etc.)
        use crate::collectors::content_cleaner::ContentCleaner;
        let (cleaned_html, cleaned_text) = ContentCleaner::clean_content_pipeline(&content_html);

        info!(
            original_html_len = content_html.len(),
            cleaned_html_len = cleaned_html.len(),
            original_text_len = content_text.len(),
            cleaned_text_len = cleaned_text.len(),
            "Applied content cleaning"
        );

        Ok((cleaned_html, cleaned_text))
    }
}
