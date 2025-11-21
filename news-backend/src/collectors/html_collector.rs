use crate::collectors::web_parser::WebParser;
use crate::models::raw_document::ArticleMetadata;
use anyhow::{Context, Result};
use reqwest::Client;
use scraper::Html;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, warn};
use url;

type PageConfig = (String, Option<HashMap<String, String>>, Option<u32>);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum HtmlRenderingMode {
    Auto,
    NoJs,
    ForceJs,
}

/// Cliente para coleta de artigos via HTML scraping
pub struct HtmlCollector {
    client: Client,
    temp_dir: PathBuf,
}

impl HtmlCollector {
    /// Cria novo cliente HTML
    pub fn new(temp_dir: PathBuf) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();

        // User Agent - simula navegador Chrome real
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
            ),
        );

        // Accept headers - tipos de conteúdo aceitos
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
        );
        headers.insert(
            reqwest::header::ACCEPT_ENCODING,
            reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
        );

        // Security headers - para contornar bot protection (Cloudflare, etc)
        // HeaderName::from_static pode lançar panic se o nome for inválido, mas para
        // nomes conhecidos como sec-fetch-*, isso é seguro
        // Usar try_parse ou unwrap apenas para nomes conhecidos
        headers.insert(
            reqwest::header::HeaderName::from_static("sec-fetch-site"),
            reqwest::header::HeaderValue::from_static("none"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("sec-fetch-mode"),
            reqwest::header::HeaderValue::from_static("navigate"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("sec-fetch-user"),
            reqwest::header::HeaderValue::from_static("?1"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("sec-fetch-dest"),
            reqwest::header::HeaderValue::from_static("document"),
        );

        // Referer - simula navegação vinda do Google
        headers.insert(
            reqwest::header::REFERER,
            reqwest::header::HeaderValue::from_static("https://www.google.com/"),
        );

        // Upgrade-Insecure-Requests
        headers.insert(
            reqwest::header::HeaderName::from_static("upgrade-insecure-requests"),
            reqwest::header::HeaderValue::from_static("1"),
        );

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .default_headers(headers)
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .expect("Failed to create HTML client"),
            temp_dir,
        }
    }

    /// Busca HTML usando Playwright para renderizar JavaScript
    ///
    /// # Arguments
    /// * `url` - URL da página a renderizar
    ///
    /// # Returns
    /// HTML renderizado ou None se falhar
    fn fetch_with_js(url: &str) -> Option<String> {
        debug!(url = %url, "Fetching HTML with Playwright (JavaScript rendering)");

        // Obter caminho do diretório atual (news-backend)
        let current_dir = std::env::current_dir().ok()?;
        let scraper_js = current_dir.join("js").join("scraper.js");

        if !scraper_js.exists() {
            error!(
                scraper_path = %scraper_js.display(),
                "Scraper.js not found"
            );
            return None;
        }

        let output = Command::new("node")
            .arg(scraper_js.as_os_str())
            .arg(url)
            .current_dir(&current_dir)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let html = String::from_utf8_lossy(&output.stdout).to_string();
                    debug!(
                        html_length = html.len(),
                        "Successfully fetched HTML with Playwright"
                    );
                    Some(html)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!(
                        url = %url,
                        exit_code = ?output.status.code(),
                        error = %stderr,
                        "Playwright failed to fetch HTML"
                    );
                    None
                }
            }
            Err(e) => {
                error!(
                    url = %url,
                    error = %e,
                    "Failed to execute Playwright scraper"
                );
                None
            }
        }
    }

    /// Verifica se um collector precisa de JavaScript rendering
    ///
    /// # Arguments
    /// * `collector_id` - ID do collector (ex: "html_meta_ai", "html_anthropic")
    ///
    /// # Returns
    /// true se o collector precisa de JavaScript rendering
    fn needs_js_rendering(collector_id: Option<&str>) -> bool {
        // Lista de collectors que precisam de JavaScript rendering
        // Inclui sites com popups de cookies, JavaScript pesado, ou que precisam de interações
        const JS_COLLECTORS: &[&str] = &[
            "html_meta_ai",
            "html_anthropic",
            "html_alibaba_damo",
            "html_xai",
            "html_deepseek",
            "html_mistral_ai",   // 308 redirect, precisa JS
            "html_character_ai", // 308 redirect, precisa JS
            "html_intel_ai",     // 403, precisa JS
            "html_reuters_ai",   // Reuters AI - precisa JS para carregar artigos
            // Robótica - sites com popups de cookies
            "html_robot_report", // Tem popups de cookies antes de acessar notícias
            "html_boston_dynamics",
            "html_yaskawa",
            "html_agility",
            "html_unitree",
            "html_robohub",
            // Computação Quântica - sites com JavaScript pesado
            "html_quantum_computing_report",
            "html_ibm_quantum",
            "html_rigetti",
            "html_dwave",
            "html_quantinuum",
            "html_pasqal",
            "html_xanadu",
            "html_infleqtion",
            "html_qci",
            "html_ieee",           // IEEE Advancing Technology
            "html_quanta_quantum", // Quanta Magazine
            // IA - sites com interações necessárias
            "html_langchain",
            "html_pinecone",
            "html_anyscale",
            "html_modal",
            "html_fastai",
            "html_eleuther",
        ];

        if let Some(id) = collector_id {
            JS_COLLECTORS.contains(&id)
        } else {
            false
        }
    }

    /// Retorna headers customizados para sites específicos que costumam bloquear bots
    fn get_site_specific_headers(url: &str) -> Option<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Ok(parsed_url) = url::Url::parse(url)
            && let Some(host) = parsed_url.host_str()
        {
            // Sites com Cloudflare ou proteção anti-bot forte
            if host.contains("anthropic.com")
                || host.contains("stability.ai")
                || host.contains("character.ai")
            {
                // Headers extras para contornar Cloudflare
                headers.insert(
                    reqwest::header::HeaderName::from_static("cf-ray"),
                    reqwest::header::HeaderValue::from_static(""),
                );
                headers.insert(
                    reqwest::header::HeaderName::from_static("cf-ipcountry"),
                    reqwest::header::HeaderValue::from_static("US"),
                );
                return Some(headers);
            }

            // Sites que checam Referer específico
            if host.contains("perplexity.ai") {
                headers.insert(
                    reqwest::header::REFERER,
                    reqwest::header::HeaderValue::from_str(&format!("https://{}/", host))
                        .unwrap_or_else(|_| {
                            reqwest::header::HeaderValue::from_static("https://www.perplexity.ai/")
                        }),
                );
                return Some(headers);
            }

            // Sites acadêmicos (Stanford, Berkeley)
            if host.contains("stanford.edu") || host.contains("berkeley.edu") {
                headers.insert(
                    reqwest::header::REFERER,
                    reqwest::header::HeaderValue::from_str(&format!("https://{}/", host))
                        .unwrap_or_else(|_| {
                            reqwest::header::HeaderValue::from_static("https://google.com/")
                        }),
                );
                return Some(headers);
            }

            // Sites que frequentemente bloqueiam bots (403 Forbidden)
            if host.contains("eleuther.ai") || host.contains("x.ai") {
                // Headers mais robustos para contornar bloqueios
                headers.insert(
                    reqwest::header::USER_AGENT,
                    reqwest::header::HeaderValue::from_static(
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                    ),
                );
                headers.insert(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static(
                        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8"
                    ),
                );
                headers.insert(
                    reqwest::header::ACCEPT_LANGUAGE,
                    reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
                );
                headers.insert(
                    reqwest::header::ACCEPT_ENCODING,
                    reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
                );
                headers.insert(
                    reqwest::header::DNT,
                    reqwest::header::HeaderValue::from_static("1"),
                );
                headers.insert(
                    reqwest::header::CONNECTION,
                    reqwest::header::HeaderValue::from_static("keep-alive"),
                );
                headers.insert(
                    reqwest::header::UPGRADE_INSECURE_REQUESTS,
                    reqwest::header::HeaderValue::from_static("1"),
                );
                headers.insert(
                    reqwest::header::REFERER,
                    reqwest::header::HeaderValue::from_str(&format!("https://{}/", host))
                        .unwrap_or_else(|_| {
                            reqwest::header::HeaderValue::from_static("https://google.com/")
                        }),
                );
                return Some(headers);
            }
        }

        None
    }

    /// Verifica se uma URL precisa de JavaScript rendering baseado no domínio
    ///
    /// # Arguments
    /// * `url` - URL a verificar
    ///
    /// # Returns
    /// true se a URL precisa de JavaScript rendering
    fn needs_js_rendering_by_url(url: &str) -> bool {
        // Domínios que precisam de JavaScript rendering
        // Inclui sites com popups de cookies, JavaScript pesado, ou que precisam de interações
        const JS_DOMAINS: &[&str] = &[
            // IA - Empresas principais
            "mistral.ai",
            "character.ai",
            "intel.com",
            "ai.meta.com",
            "about.fb.com",
            "anthropic.com",
            "alizila.com",
            "x.ai",
            "deepseek.ai",
            "deepseek.com",
            "blog.perplexity.ai",
            "perplexity.ai",
            "venturebeat.com",
            "time.com",
            "reuters.com",        // Reuters - precisa JS para carregar artigos
            // Robótica
            "therobotreport.com", // Tem popups de cookies
            "bostondynamics.com",
            "yaskawa.com",
            "agilityrobotics.com",
            "unitree.com",
            "robohub.org",
            "automate.org",
            "universal-robots.com",
            "omron.com",
            "global.abb",
            // Computação Quântica
            "quantumcomputingreport.com",
            "research.ibm.com",
            "quantamagazine.org",
            "globenewswire.com",
            "dwavequantum.com",
            "quantinuum.com",
            "pasqal.com",
            "xanadu.ai",
            "infleqtion.com",
            "quantumcomputinginc.com",
            "ieee.org",
            // IA - Startups e ferramentas
            "langchain.com",
            "pinecone.io",
            "anyscale.com",
            "modal.com",
            "fast.ai",
            "eleuther.ai",
        ];

        if let Ok(parsed_url) = url::Url::parse(url)
            && let Some(host) = parsed_url.host_str()
        {
            return JS_DOMAINS.iter().any(|domain| host.contains(domain));
        }
        false
    }

    /// Busca artigos de uma página HTML
    ///
    /// # Arguments
    /// * `base_url` - URL base da página (ex: https://ai.meta.com/blog/)
    /// * `selectors` - Seletores CSS configuráveis (opcional)
    ///   - "title": Seletor para título do artigo
    ///   - "content": Seletor para conteúdo principal
    ///   - "author": Seletor para autor
    ///   - "article": Seletor para container do artigo (para múltiplos artigos)
    /// * `max_results` - Número máximo de artigos a retornar
    /// * `collector_id` - ID do collector (opcional, usado para detectar se precisa JS rendering)
    pub async fn fetch_page(
        &self,
        base_url: &str,
        selectors: Option<&HashMap<String, String>>,
        max_results: Option<u32>,
        collector_id: Option<&str>,
    ) -> Result<Vec<ArticleMetadata>> {
        self.fetch_page_with_mode(
            base_url,
            selectors,
            max_results,
            collector_id,
            HtmlRenderingMode::Auto,
        )
        .await
    }

    pub async fn fetch_page_with_mode(
        &self,
        base_url: &str,
        selectors: Option<&HashMap<String, String>>,
        max_results: Option<u32>,
        collector_id: Option<&str>,
        rendering_mode: HtmlRenderingMode,
    ) -> Result<Vec<ArticleMetadata>> {
        let max = max_results.unwrap_or(10);

        debug!(
            url = %base_url,
            max_results = max,
            collector_id = ?collector_id,
            mode = ?rendering_mode,
            "Fetching HTML page"
        );

        // Verificar se precisa de JavaScript rendering
        let needs_js = match rendering_mode {
            HtmlRenderingMode::ForceJs => {
                debug!(
                    url = %base_url,
                    collector_id = ?collector_id,
                    "Forcing JavaScript rendering via Playwright"
                );
                true
            }
            HtmlRenderingMode::NoJs => {
                debug!(
                    url = %base_url,
                    collector_id = ?collector_id,
                    "Forcing static HTML fetch (no JavaScript)"
                );
                false
            }
            HtmlRenderingMode::Auto => {
                let needs_js_by_collector = Self::needs_js_rendering(collector_id);
                let needs_js_by_url = Self::needs_js_rendering_by_url(base_url);

                debug!(
                    url = %base_url,
                    collector_id = ?collector_id,
                    needs_js_by_collector = needs_js_by_collector,
                    needs_js_by_url = needs_js_by_url,
                    needs_js_rendering = needs_js_by_collector || needs_js_by_url,
                    "Checking if JavaScript rendering is needed"
                );

                needs_js_by_collector || needs_js_by_url
            }
        };

        let html_content = if needs_js {
            debug!(url = %base_url, "Using Playwright for JavaScript rendering");
            // Usar Playwright para renderizar JavaScript
            match Self::fetch_with_js(base_url) {
                Some(html) => {
                    debug!(
                        url = %base_url,
                        html_length = html.len(),
                        "Successfully fetched HTML with Playwright"
                    );
                    html
                }
                None => {
                    warn!(
                        url = %base_url,
                        "Playwright failed, falling back to regular HTTP request"
                    );
                    // Fallback para requisição HTTP normal com headers customizados
                    let mut request = self.client.get(base_url);
                    if let Some(custom_headers) = Self::get_site_specific_headers(base_url) {
                        debug!(url = %base_url, "Applying site-specific headers");
                        for (name, value) in custom_headers.iter() {
                            request = request.header(name.clone(), value.clone());
                        }
                    }
                    let response = request.send().await?;

                    if !response.status().is_success() {
                        return Err(anyhow::anyhow!(
                            "Failed to fetch HTML page: HTTP {}",
                            response.status()
                        ));
                    }

                    response.text().await?
                }
            }
        } else {
            debug!(url = %base_url, "Using regular HTTP request (no JavaScript rendering needed)");
            // Requisição HTTP normal com headers customizados para contornar bloqueios
            let mut request = self.client.get(base_url);
            if let Some(custom_headers) = Self::get_site_specific_headers(base_url) {
                debug!(url = %base_url, "Applying site-specific headers");
                for (name, value) in custom_headers.iter() {
                    request = request.header(name.clone(), value.clone());
                }
            }
            let response = request.send().await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to fetch HTML page: HTTP {}",
                    response.status()
                ));
            }

            response.text().await?
        };

        // Salvar HTML temporário para debug
        let temp_file = self
            .temp_dir
            .join(format!("html_page_{}.html", chrono::Utc::now().timestamp()));
        tokio::fs::write(&temp_file, &html_content)
            .await
            .context("Failed to save HTML page")?;

        debug!(
            temp_file = %temp_file.display(),
            html_length = html_content.len(),
            "Saved HTML page"
        );

        // Se o HTML é muito pequeno ou vazio, pode ser que precise JavaScript
        if html_content.len() < 1000 {
            warn!(
                url = %base_url,
                html_length = html_content.len(),
                "HTML response is very small, may require JavaScript rendering"
            );
        }

        // Parse HTML
        let document = Html::parse_document(&html_content);

        // Verificar se há algum conteúdo útil no HTML
        let body_text: String = document
            .select(
                &scraper::Selector::parse("body")
                    .unwrap_or_else(|_| scraper::Selector::parse("html").unwrap()),
            )
            .next()
            .map(|el| el.text().collect())
            .unwrap_or_default();

        debug!(
            body_text_length = body_text.len(),
            has_script_tags = html_content.contains("<script"),
            has_react = html_content.contains("react") || html_content.contains("React"),
            "HTML content analysis"
        );

        // Se há seletor "link" configurado, usar para encontrar links diretamente
        // Isso é útil quando os links estão fora de elementos article
        let articles = if let Some(link_selector_str) = selectors.and_then(|s| s.get("link")) {
            // Se o seletor contém padrões como [href*='...'], buscar todos os links e filtrar manualmente
            let link_selector = if link_selector_str.contains("[href") {
                // Usar seletor simples "a" e filtrar depois
                scraper::Selector::parse("a").unwrap_or_else(|_| {
                    // Fallback se falhar
                    scraper::Selector::parse("a").expect("Failed to parse 'a' selector")
                })
            } else {
                scraper::Selector::parse(link_selector_str).unwrap_or_else(|_| {
                    scraper::Selector::parse("a").expect("Failed to parse link selector")
                })
            };

            let mut link_urls = Vec::new();

            // Extrair padrões do seletor para filtrar
            let href_patterns: Vec<String> = if link_selector_str.contains("[href*=") {
                // Extrair padrões de [href*='pattern'] ou [href*="pattern"]
                let pattern_re = regex::Regex::new(r#"\[href\*=['"]([^'"]+)['"]"#).unwrap();
                pattern_re
                    .captures_iter(link_selector_str)
                    .map(|cap| {
                        // Pegar o primeiro grupo que contém o padrão
                        cap.get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default()
                    })
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                vec![]
            };

            // Verificar quantos elementos <a> existem no documento total
            let all_a_elements: Vec<_> = document
                .select(&scraper::Selector::parse("a").unwrap())
                .collect();
            debug!(
                total_a_elements = all_a_elements.len(),
                link_selector_used = %link_selector_str,
                "Checking HTML structure"
            );

            // Se não há elementos <a>, tentar buscar URLs diretamente no HTML usando regex
            let use_regex_fallback = all_a_elements.is_empty() && !href_patterns.is_empty();

            let mut total_links_found = 0;
            let mut total_links_found_regex = 0;

            if use_regex_fallback {
                // Buscar URLs diretamente no HTML usando regex
                warn!(
                    url = %base_url,
                    "No <a> elements found, using regex fallback to extract URLs"
                );

                // Buscar URLs que correspondem aos padrões de forma mais genérica
                // Primeiro, tentar extrair URLs de JSON embutido no HTML
                let json_url_pattern = regex::Regex::new(r#"(?s)(\{[^}]*"url"[^}]*\})"#).unwrap();
                let mut json_urls = Vec::new();
                for cap in json_url_pattern.captures_iter(&html_content) {
                    if let Some(json_match) = cap.get(1)
                        && let Ok(json_val) =
                            serde_json::from_str::<serde_json::Value>(json_match.as_str())
                        && let Some(url_str) = json_val.get("url").and_then(|v| v.as_str())
                        && !url_str.is_empty()
                    {
                        json_urls.push(url_str.to_string());
                    }
                }

                // Buscar URLs diretamente no HTML (também em strings JSON não parseadas)
                let url_re = regex::Regex::new(r#"(https?://[^\s"'<>\)]+)"#).unwrap();

                let mut all_urls_found = 0;
                for cap in url_re.captures_iter(&html_content) {
                    all_urls_found += 1;
                    if let Some(url_match) = cap.get(1) {
                        let mut url = url_match.as_str().to_string();

                        // Limpar caracteres finais indesejados
                        url = url
                            .trim_end_matches('"')
                            .trim_end_matches('\'')
                            .trim_end_matches('>')
                            .trim_end_matches(')')
                            .trim_end_matches(',')
                            .to_string();

                        // Verificar se a URL corresponde a algum padrão
                        let matches_pattern = href_patterns.iter().any(|p| {
                            if p.starts_with("http") {
                                url.starts_with(p)
                            } else {
                                url.contains(p)
                            }
                        });

                        if !matches_pattern {
                            continue;
                        }

                        total_links_found_regex += 1;

                        // Filtrar URLs que não são artigos
                        if url.contains("/category/")
                            || url.contains("/tag/")
                            || url.contains("/author/")
                            || url.contains("/page/")
                            || url.contains("/feed/")
                            || url.contains("wp-json")
                            || url.contains("#")
                        {
                            continue;
                        }

                        // Verificar se parece ser um artigo
                        let base_url_normalized = base_url.trim_end_matches('/');
                        let url_normalized = url.trim_end_matches('/');
                        if url_normalized != base_url_normalized {
                            let path_segments: Vec<&str> = url
                                .split('/')
                                .filter(|s| !s.is_empty() && !s.contains(':'))
                                .collect();
                            // Para x.ai/news, aceitar qualquer URL que contenha /news/ (genérico para qualquer notícia)
                            let is_xai = url.contains("x.ai/news") && (
                                path_segments.len() >= 3 ||
                                url.contains("/news/")
                            );
                            let is_article = url.contains("/20")
                                || path_segments.len() >= 4
                                || (path_segments.len() >= 3
                                    && path_segments
                                        .last()
                                        .map(|s| s.matches('-').count() >= 2)
                                        .unwrap_or(false))
                                || is_xai; // Permitir URLs do x.ai/news com qualquer path válido

                            if is_article && !link_urls.contains(&url) {
                                link_urls.push(url.clone());
                            }
                        }
                    }
                }

                // Também buscar URLs relativas e convertê-las para absolutas
                let fallback_rel_regex = regex::Regex::new(r#"["']?(/[^"'\s<>\)]+)["']?"#).unwrap();

                for pattern in &href_patterns {
                    if !pattern.starts_with("http") && pattern.contains('/') {
                        // Buscar URLs relativas que começam com / e contêm o padrão
                        let rel_pattern =
                            format!(r#"["\']?({}[^"'\s<>\)]*)["\']?"#, regex::escape(pattern));
                        let re = regex::Regex::new(&rel_pattern)
                            .unwrap_or_else(|_| fallback_rel_regex.clone());

                        for cap in re.captures_iter(&html_content) {
                            if let Some(rel_match) = cap.get(1) {
                                let mut rel_path = rel_match.as_str().to_string();

                                // Limpar caracteres indesejados
                                rel_path = rel_path
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .trim_matches(')')
                                    .to_string();

                                // Se não começa com /, pode ser uma URL completa
                                if !rel_path.starts_with('/')
                                    && (rel_path.starts_with("http://")
                                        || rel_path.starts_with("https://"))
                                {
                                    let url = rel_path;
                                    if link_urls.contains(&url) {
                                        continue;
                                    }

                                    total_links_found_regex += 1;
                                    if Self::should_skip_url(&url) {
                                        continue;
                                    }

                                    Self::push_if_article(&mut link_urls, base_url, &url);
                                    continue;
                                }

                                // Resolver URL relativa
                                let resolved_url = Self::resolve_relative_url(base_url, &rel_path);

                                if link_urls.contains(&resolved_url) {
                                    continue;
                                }

                                total_links_found_regex += 1;

                                if Self::should_skip_url(&resolved_url) {
                                    continue;
                                }

                                let base_url_normalized = base_url.trim_end_matches('/');
                                let url_normalized = resolved_url.trim_end_matches('/');
                                if url_normalized == base_url_normalized {
                                    continue;
                                }

                                let path_segments: Vec<&str> = resolved_url
                                    .split('/')
                                    .filter(|s| !s.is_empty() && !s.contains(':'))
                                    .collect();
                                let is_article = resolved_url.contains("/20")
                                    || path_segments.len() >= 4
                                    || (path_segments.len() >= 3
                                        && path_segments
                                            .last()
                                            .map(|s| s.matches('-').count() >= 2)
                                            .unwrap_or(false));

                                if is_article {
                                    link_urls.push(resolved_url);
                                }
                            }
                        }
                    }
                }

                debug!(
                    all_urls_in_html = all_urls_found,
                    urls_matching_patterns = total_links_found_regex,
                    valid_article_urls = link_urls.len(),
                    href_patterns = ?href_patterns,
                    "Extracted URLs using regex fallback"
                );

                // Adicionar URLs encontradas em JSON
                for json_url in json_urls {
                    // Verificar se corresponde a algum padrão
                    let matches_pattern = href_patterns.iter().any(|p| {
                        if p.starts_with("http") {
                            json_url.starts_with(p)
                        } else {
                            json_url.contains(p)
                        }
                    });

                    if matches_pattern {
                        let base_url_normalized = base_url.trim_end_matches('/');
                        let url_normalized = json_url.trim_end_matches('/');
                        if url_normalized != base_url_normalized {
                            let path_segments: Vec<&str> = json_url
                                .split('/')
                                .filter(|s| !s.is_empty() && !s.contains(':'))
                                .collect();
                            let is_article = json_url.contains("/20")
                                || path_segments.len() >= 4
                                || (path_segments.len() >= 3
                                    && path_segments
                                        .last()
                                        .map(|s| s.matches('-').count() >= 2)
                                        .unwrap_or(false));

                            if is_article && !link_urls.contains(&json_url) {
                                link_urls.push(json_url);
                            }
                        }
                    }
                }

                // Se ainda não encontrou URLs, tentar buscar diretamente por padrões comuns
                if link_urls.is_empty() {
                    // Extrair domínio base uma vez
                    let base_domain = base_url
                        .replace("https://", "")
                        .replace("http://", "")
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .to_string();

                    warn!(
                        url = %base_url,
                        base_domain = %base_domain,
                        total_urls_found = all_urls_found,
                        "No URLs found, trying aggressive extraction methods"
                    );

                    // Método 1: Buscar URLs relativas que seguem padrão /news/YYYY/...
                    for pattern in &href_patterns {
                        if !pattern.starts_with("http") && pattern.contains('/') {
                            // Buscar padrões como /news/2025/algum-artigo
                            let rel_pattern = format!(
                                r#"["\']?({}[0-9]{{4}}/[^"'\s<>\)]+)["\']?"#,
                                regex::escape(pattern)
                            );
                            if let Ok(rel_re) = regex::Regex::new(&rel_pattern) {
                                for cap in rel_re.captures_iter(&html_content) {
                                    if let Some(rel_match) = cap.get(1) {
                                        let rel_path = rel_match
                                            .as_str()
                                            .trim_matches('"')
                                            .trim_matches('\'')
                                            .trim_matches(')')
                                            .trim_matches(',');

                                        // Resolver URL relativa
                                        use url::Url;
                                        let resolved_url = if let Ok(base) = Url::parse(base_url) {
                                            if let Ok(resolved) = base.join(rel_path) {
                                                resolved.to_string()
                                            } else {
                                                format!(
                                                    "{}{}",
                                                    base_url.trim_end_matches('/'),
                                                    rel_path
                                                )
                                            }
                                        } else {
                                            format!(
                                                "{}{}",
                                                base_url.trim_end_matches('/'),
                                                rel_path
                                            )
                                        };

                                        if !link_urls.contains(&resolved_url) {
                                            link_urls.push(resolved_url.clone());
                                            debug!(found_url = %resolved_url, "Found relative URL via pattern extraction");
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Método 2: Buscar qualquer URL que contenha 'news' e ano do mesmo domínio
                    if link_urls.is_empty() {
                        let news_url_re = regex::Regex::new(
                            r#"(https?://[^\s"'<>\)]*news[^\s"'<>\)]*20[0-9]{2}[^\s"'<>\)]*)"#,
                        )
                        .unwrap();
                        for cap in news_url_re.captures_iter(&html_content) {
                            if let Some(url_match) = cap.get(1) {
                                let mut url = url_match.as_str().to_string();
                                url = url
                                    .trim_end_matches('"')
                                    .trim_end_matches('\'')
                                    .trim_end_matches('>')
                                    .trim_end_matches(')')
                                    .trim_end_matches(',')
                                    .to_string();

                                if url.contains(&base_domain) && !link_urls.contains(&url) {
                                    let url_clone = url.clone();
                                    link_urls.push(url);
                                    debug!(found_url = %url_clone, "Found URL using aggressive extraction");
                                }
                            }
                        }
                    }

                    // Método 3: Buscar qualquer path relativo que pareça ser um artigo
                    if link_urls.is_empty() {
                        let article_path_re = regex::Regex::new(
                            r#"["\']?(/[^"'\s<>\)]*news[^"'\s<>\)]*20[0-9]{2}[^"'\s<>\)]*)["\']?"#,
                        )
                        .unwrap();
                        for cap in article_path_re.captures_iter(&html_content) {
                            if let Some(path_match) = cap.get(1) {
                                let rel_path = path_match
                                    .as_str()
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .trim_matches(')')
                                    .trim_matches(',');

                                // Resolver URL relativa
                                use url::Url;
                                let resolved_url = if let Ok(base) = Url::parse(base_url) {
                                    if let Ok(resolved) = base.join(rel_path) {
                                        resolved.to_string()
                                    } else {
                                        format!("{}{}", base_url.trim_end_matches('/'), rel_path)
                                    }
                                } else {
                                    format!("{}{}", base_url.trim_end_matches('/'), rel_path)
                                };

                                if !link_urls.contains(&resolved_url) {
                                    link_urls.push(resolved_url.clone());
                                    debug!(found_url = %resolved_url, "Found relative article path");
                                }
                            }
                        }
                    }

                    if !link_urls.is_empty() {
                        debug!(
                            urls_found_aggressive = link_urls.len(),
                            "Successfully extracted URLs using aggressive methods"
                        );
                    }
                }
            }

            for link_element in document.select(&link_selector) {
                total_links_found += 1;
                if let Some(href) = link_element.value().attr("href") {
                    // Se há padrões específicos no seletor, verificar se o href corresponde
                    // Para x.ai/news, aceitar qualquer link que contenha /news/ (genérico para todas as notícias)
                    let is_xai_base = base_url.contains("x.ai/news");
                    if !href_patterns.is_empty() {
                        let matches_pattern = href_patterns
                            .iter()
                            .any(|pattern| href.contains(pattern.as_str()));
                        // Para x.ai, aceitar qualquer link que vá para /news/ (genérico para qualquer notícia)
                        let matches_xai_generic = is_xai_base && (
                            href.contains("/news/") ||
                            href.contains("x.ai/news")
                        );
                        if !matches_pattern && !matches_xai_generic {
                            continue;
                        }
                    }

                    // Filtrar links que não são artigos
                    if href.contains("/category/")
                        || href.contains("/tag/")
                        || href.contains("/author/")
                        || href.contains("/page/")
                        || href.contains("/feed/")
                        || href.contains("wp-json")
                        || href == "#"
                        || href.starts_with("#")
                    {
                        continue;
                    }

                    // Resolver URL relativa
                    let resolved_url =
                        if href.starts_with("http://") || href.starts_with("https://") {
                            href.to_string()
                        } else {
                            use url::Url;
                            if let Ok(base) = Url::parse(base_url) {
                                if let Ok(resolved) = base.join(href) {
                                    resolved.to_string()
                                } else {
                                    format!(
                                        "{}/{}",
                                        base_url.trim_end_matches('/'),
                                        href.trim_start_matches('/')
                                    )
                                }
                            } else {
                                format!(
                                    "{}/{}",
                                    base_url.trim_end_matches('/'),
                                    href.trim_start_matches('/')
                                )
                            }
                        };

                    // Verificar se parece ser um artigo (não é apenas a base)
                    let base_url_normalized = base_url.trim_end_matches('/');
                    let resolved_url_normalized = resolved_url.trim_end_matches('/');
                    if resolved_url_normalized != base_url_normalized {
                        // Verificar se é um artigo válido:
                        // - Contém ano (2024, 2025)
                        // - Ou tem 4+ segmentos no path
                        // - Ou termina com padrão de slug (múltiplas palavras com hífen)
                        // - Ou é de x.ai/news (mais permissivo para este domínio)
                        let path_segments: Vec<&str> = resolved_url
                            .split('/')
                            .filter(|s| !s.is_empty() && !s.contains(':'))
                            .collect();
                        // Para x.ai/news, aceitar qualquer URL que contenha /news/ e não seja apenas a base
                        // Solução genérica: qualquer path após /news/ é considerado artigo
                        let is_xai = resolved_url.contains("x.ai/news") && (
                            path_segments.len() >= 3 ||
                            resolved_url.contains("/news/")
                        );
                        let is_article = resolved_url.contains("/20")
                            || path_segments.len() >= 4
                            || (path_segments.len() >= 3
                                && path_segments
                                    .last()
                                    .map(|s| s.matches('-').count() >= 2)
                                    .unwrap_or(false))
                            || is_xai; // Permitir URLs do x.ai/news com qualquer path válido

                        if is_article {
                            link_urls.push(resolved_url);
                        }
                    }
                }
            }

            // Remover duplicatas (preservar ordem original para todas as fontes)
            // A ordem original garante que notícias mais recentes sejam processadas primeiro
            link_urls.dedup();

            debug!(
                link_selector = %link_selector_str,
                total_links_processed = total_links_found,
                valid_article_links = link_urls.len(),
                href_patterns = ?href_patterns,
                "Found article links directly (order preserved)"
            );

            // Debug: mostrar alguns links encontrados
            if !link_urls.is_empty() {
                debug!(
                    sample_links = ?link_urls.iter().take(5).collect::<Vec<_>>(),
                    "Sample links found"
                );
            }

            // Buscar conteúdo de cada link
            let mut articles = Vec::new();
            for (idx, link_url) in link_urls.iter().take(max as usize * 2).enumerate() {
                if articles.len() >= max as usize {
                    break;
                }

                debug!(
                    index = idx + 1,
                    url = %link_url,
                    "Fetching article from link"
                );

                match self.fetch_full_article(link_url, selectors).await {
                    Ok(metadata) => {
                        // Limite padrão para todos (x.ai já confirmado ter conteúdo suficiente)
                        const MIN_CONTENT_LENGTH: usize = 1500;
                        let min_length = MIN_CONTENT_LENGTH;
                        let has_valid_content = metadata
                            .content_text
                            .as_ref()
                            .map(|text| !text.trim().is_empty() && text.len() >= min_length)
                            .unwrap_or(false);

                        if has_valid_content {
                            debug!(
                                index = idx + 1,
                                title = %metadata.title,
                                url = %metadata.url,
                                content_length = metadata.content_text.as_ref().map(|t| t.len()).unwrap_or(0),
                                "Fetched article from link"
                            );
                            articles.push(metadata);
                        } else {
                            warn!(
                                index = idx + 1,
                                url = %link_url,
                                content_length = metadata.content_text.as_ref().map(|t| t.len()).unwrap_or(0),
                                "Article content too short, skipping"
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            index = idx + 1,
                            url = %link_url,
                            error = %e,
                            "Failed to fetch article from link"
                        );
                    }
                }

                // Rate limiting
                if idx < link_urls.len().saturating_sub(1) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }

            articles
        } else if let Some(article_selector_str) = selectors.and_then(|s| s.get("article")) {
            // Multiple articles on one page (e.g., blog listing)
            self.extract_multiple_articles(
                &document,
                article_selector_str,
                base_url,
                selectors,
                max,
            )
            .await?
        } else {
            // Single article page
            match WebParser::parse_html_page(&html_content, base_url, selectors).await {
                Ok(web_article) => {
                    let metadata = WebParser::web_article_to_metadata(web_article, None);

                    // Verificar se o conteúdo é válido (não vazio e tem tamanho mínimo suficiente para escrever notícia)
                    const MIN_CONTENT_LENGTH: usize = 1500; // Mínimo de 1000 caracteres para garantir conteúdo completo
                    let has_valid_content = metadata
                        .content_text
                        .as_ref()
                        .map(|text| !text.trim().is_empty() && text.len() >= MIN_CONTENT_LENGTH)
                        .unwrap_or(false);

                    if has_valid_content {
                        vec![metadata]
                    } else {
                        warn!(
                            title = %metadata.title,
                            url = %metadata.url,
                            "Article content too short or empty, skipping"
                        );
                        vec![] // Retornar lista vazia se conteúdo inválido
                    }
                }
                Err(e) => {
                    warn!(
                        url = %base_url,
                        error = %e,
                        "Failed to parse HTML article, skipping"
                    );
                    vec![] // Retornar lista vazia se falhar
                }
            }
        };

        debug!(count = articles.len(), "Extracted articles from HTML page");

        Ok(articles)
    }

    /// Extrai múltiplos artigos de uma página de listagem
    async fn extract_multiple_articles(
        &self,
        document: &Html,
        article_selector_str: &str,
        base_url: &str,
        selectors: Option<&HashMap<String, String>>,
        max: u32,
    ) -> Result<Vec<ArticleMetadata>> {
        use scraper::Selector;

        let article_selector = Selector::parse(article_selector_str).map_err(|e| {
            anyhow::anyhow!("Invalid article selector '{}': {}", article_selector_str, e)
        })?;

        let mut articles = Vec::new();
        let mut processed_urls = std::collections::HashSet::new();

        // Coletar todos os elementos primeiro, depois filtrar
        let all_elements: Vec<_> = document.select(&article_selector).collect();

        debug!(
            selector = %article_selector_str,
            elements_found = all_elements.len(),
            "Found article elements in page"
        );

        for (idx, article_element) in all_elements.iter().take((max * 3) as usize).enumerate() {
            // Try to extract link to full article
            let article_url = self
                .extract_article_link(*article_element, base_url, selectors)
                .unwrap_or_else(|| {
                    // If no link found, use base URL with index
                    format!("{}#article-{}", base_url, idx)
                });

            // Filtrar URLs que não são artigos (categorias, tags, páginas, etc)
            if article_url.contains("/category/") 
                || article_url.contains("/tag/") 
                || article_url.contains("/author/") 
                || article_url.contains("/page/")
                || article_url.contains("/feed/")
                || article_url.contains("wp-json")
                || article_url.contains("#article-")  // URLs com âncoras genéricas
                || processed_urls.contains(&article_url)
            {
                continue;
            }

            // Se for o próprio elemento base_url, pular
            let base_url_normalized = base_url.trim_end_matches('/');
            let article_url_normalized = article_url.trim_end_matches('/');
            if article_url_normalized == base_url_normalized {
                continue;
            }

            // Verificar se a URL parece ser de um artigo (não é apenas a base)
            // URLs de artigos geralmente têm mais de um segmento após o domínio
            if let Ok(parsed_url) = url::Url::parse(&article_url) {
                let path_segments: Vec<&str> = parsed_url
                    .path_segments()
                    .map(|s| s.collect::<Vec<_>>())
                    .unwrap_or_default();

                // Se tiver menos de 2 segmentos, provavelmente não é um artigo
                if path_segments.len() < 2 && !article_url.contains("/20") {
                    // Exceto se tiver ano (2024, 2025)
                    continue;
                }
            }

            processed_urls.insert(article_url.clone());

            debug!(
                index = idx + 1,
                url = %article_url,
                total_found = all_elements.len(),
                "Processing article URL"
            );

            // Se já temos artigos suficientes, parar
            if articles.len() >= max as usize {
                break;
            }

            // Fetch full article content from URL if it's different from base URL
            if article_url.starts_with("http://") || article_url.starts_with("https://") {
                // Require full article content - skip if fetch fails
                match self.fetch_full_article(&article_url, selectors).await {
                    Ok(metadata) => {
                        // Verificar se o conteúdo é válido (não vazio e tem tamanho mínimo suficiente para escrever notícia)
                        const MIN_CONTENT_LENGTH: usize = 1500; // Mínimo de 1000 caracteres para garantir conteúdo completo
                        let has_valid_content = metadata
                            .content_text
                            .as_ref()
                            .map(|text| !text.trim().is_empty() && text.len() >= MIN_CONTENT_LENGTH)
                            .unwrap_or(false);

                        if !has_valid_content {
                            warn!(
                                index = idx + 1,
                                title = %metadata.title,
                                url = %metadata.url,
                                "Article content too short or empty, skipping"
                            );
                            continue; // Ignorar artigo sem conteúdo válido
                        }

                        debug!(
                            index = idx + 1,
                            title = %metadata.title,
                            url = %metadata.url,
                            content_length = metadata.content_text.as_ref().map(|t| t.len()).unwrap_or(0),
                            "Fetched full article content"
                        );
                        articles.push(metadata);
                    }
                    Err(e) => {
                        warn!(
                            index = idx + 1,
                            url = %article_url,
                            error = %e,
                            "Failed to fetch full article content, skipping article"
                        );
                        // Ignorar artigo sem conteúdo completo
                        continue;
                    }
                }
            } else {
                // No separate article URL, parse from article element (full content on same page)
                let article_html = article_element.html();
                match WebParser::parse_html_page(&article_html, &article_url, selectors).await {
                    Ok(web_article) => {
                        let metadata = WebParser::web_article_to_metadata(web_article, None);

                        // Verificar se o conteúdo é válido (não vazio e tem tamanho mínimo suficiente)
                        const MIN_CONTENT_LENGTH: usize = 1500; // Mínimo de 1000 caracteres para garantir conteúdo completo
                        let has_valid_content = metadata
                            .content_text
                            .as_ref()
                            .map(|text| !text.trim().is_empty() && text.len() >= MIN_CONTENT_LENGTH)
                            .unwrap_or(false);

                        if !has_valid_content {
                            warn!(
                                index = idx + 1,
                                title = %metadata.title,
                                url = %metadata.url,
                                "Article content too short or empty, skipping"
                            );
                            continue; // Ignorar artigo sem conteúdo válido
                        }

                        debug!(
                            index = idx + 1,
                            title = %metadata.title,
                            url = %metadata.url,
                            content_length = metadata.content_text.as_ref().map(|t| t.len()).unwrap_or(0),
                            "Parsed HTML article"
                        );
                        articles.push(metadata);
                    }
                    Err(e) => {
                        warn!(
                            index = idx + 1,
                            error = %e,
                            "Failed to parse HTML article, skipping"
                        );
                        // Ignorar artigo que não pode ser parseado
                        continue;
                    }
                }
            }

            // Rate limiting: delay entre requisições
            if idx < max.saturating_sub(1) as usize {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        Ok(articles)
    }

    /// Extrai link para o artigo completo
    fn extract_article_link(
        &self,
        article_element: scraper::ElementRef,
        base_url: &str,
        _selectors: Option<&HashMap<String, String>>,
    ) -> Option<String> {
        use scraper::Selector;

        // Se o próprio elemento é um link <a>, usar diretamente
        if article_element.value().name() == "a"
            && let Some(href) = article_element.value().attr("href")
        {
            return Self::resolve_href(base_url, href);
        }

        // Try common link selectors dentro do elemento
        let link_selectors = vec![
            "a",
            "h1 a",
            "h2 a",
            "h3 a",
            "h4 a",
            ".title a",
            ".article-title a",
            ".post-title a",
            "a.read-more",
        ];

        for selector_str in link_selectors {
            if let Ok(link_selector) = Selector::parse(selector_str)
                && let Some(link) = article_element.select(&link_selector).next()
                && let Some(href) = link.value().attr("href")
            {
                if href.contains("/category/")
                    || href.contains("/tag/")
                    || href.contains("/author/")
                    || href.contains("/page/")
                {
                    continue;
                }

                if let Some(resolved) = Self::resolve_href(base_url, href) {
                    return Some(resolved);
                }
            }
        }

        None
    }

    fn resolve_href(base_url: &str, href: &str) -> Option<String> {
        if let Ok(base) = url::Url::parse(base_url)
            && let Ok(resolved) = base.join(href)
        {
            return Some(resolved.to_string());
        }

        if href.starts_with("http://") || href.starts_with("https://") {
            return Some(href.to_string());
        }

        Some(Self::resolve_relative_url(base_url, href))
    }

    fn should_skip_url(url: &str) -> bool {
        url.contains("/category/")
            || url.contains("/tag/")
            || url.contains("/author/")
            || url.contains("/page/")
            || url.contains("/feed/")
            || url.contains("wp-json")
            || url.contains('#')
    }

    fn push_if_article(link_urls: &mut Vec<String>, base_url: &str, url: &str) {
        let base_url_normalized = base_url.trim_end_matches('/');
        let url_normalized = url.trim_end_matches('/');
        if url_normalized == base_url_normalized {
            return;
        }

        let path_segments: Vec<&str> = url
            .split('/')
            .filter(|segment| !segment.is_empty() && !segment.contains(':'))
            .collect();
        // Para x.ai/news, aceitar qualquer URL que contenha /news/ (genérico para qualquer notícia)
        let is_xai = url.contains("x.ai/news") && (
            path_segments.len() >= 3 ||
            url.contains("/news/")
        );
        let is_article = url.contains("/20")
            || path_segments.len() >= 4
            || (path_segments.len() >= 3
                && path_segments
                    .last()
                    .map(|segment| segment.matches('-').count() >= 2)
                    .unwrap_or(false))
            || is_xai; // Permitir URLs do x.ai/news com qualquer path válido

        if is_article {
            link_urls.push(url.to_string());
        }
    }

    fn resolve_relative_url(base_url: &str, rel_path: &str) -> String {
        if rel_path.starts_with("http://") || rel_path.starts_with("https://") {
            return rel_path.to_string();
        }

        if let Ok(base) = url::Url::parse(base_url)
            && let Ok(resolved) = base.join(rel_path)
        {
            return resolved.to_string();
        }

        let trimmed_base = base_url.trim_end_matches('/');
        if rel_path.starts_with('/') {
            format!("{trimmed_base}{rel_path}")
        } else {
            format!("{trimmed_base}/{rel_path}")
        }
    }

    /// Busca artigos de múltiplas páginas HTML
    ///
    /// # Arguments
    /// * `pages` - Vetor de tuplas (base_url, selectors, max_results)
    #[allow(dead_code)]
    pub async fn fetch_multiple_pages(
        &self,
        pages: Vec<PageConfig>,
    ) -> Result<Vec<ArticleMetadata>> {
        let mut all_articles = Vec::new();
        let pages_count = pages.len();

        for (base_url, selectors, max_results) in pages {
            match self
                .fetch_page(&base_url, selectors.as_ref(), max_results, None)
                .await
            {
                Ok(articles) => {
                    all_articles.extend(articles);
                }
                Err(e) => {
                    error!(
                        base_url = %base_url,
                        error = %e,
                        "Failed to fetch HTML page"
                    );
                }
            }

            // Rate limiting: delay entre páginas
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        debug!(
            total_pages = pages_count,
            total_articles = all_articles.len(),
            "Fetched all HTML pages"
        );

        Ok(all_articles)
    }

    /// Busca conteúdo completo do artigo pela URL
    pub async fn fetch_full_article(
        &self,
        article_url: &str,
        selectors: Option<&HashMap<String, String>>,
    ) -> Result<ArticleMetadata> {
        // Verificar se precisa de JavaScript rendering usando função centralizada
        let needs_js = Self::needs_js_rendering_by_url(article_url);

        let html_content = if needs_js {
            // Usar Playwright para renderizar JavaScript
            match Self::fetch_with_js(article_url) {
                Some(html) => html,
                None => {
                    // Fallback para requisição HTTP normal
                    let response = self.client.get(article_url).send().await?;

                    if !response.status().is_success() {
                        return Err(anyhow::anyhow!("HTTP {}", response.status()));
                    }

                    response.text().await?
                }
            }
        } else {
            // Requisição HTTP normal
            let response = self.client.get(article_url).send().await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("HTTP {}", response.status()));
            }

            response.text().await?
        };

        // Parse full article page
        let web_article = WebParser::parse_html_page(&html_content, article_url, selectors).await?;

        Ok(WebParser::web_article_to_metadata(web_article, None))
    }
}
