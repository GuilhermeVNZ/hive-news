use anyhow::Result;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::raw_document::ArticleMetadata;

/// Web content extracted from HTML or RSS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebArticle {
    pub url: String,
    pub title: String,
    pub published_date: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub content: String, // HTML or text
    pub image_url: Option<String>,
    pub meta: serde_json::Value, // Additional metadata
}

/// Parser for extracting content from web sources (RSS, HTML)
pub struct WebParser;

#[allow(dead_code)]
impl WebParser {
    /// Extract article metadata from RSS feed item
    pub fn parse_rss_item(item: &rss::Item, base_url: Option<&str>) -> Result<ArticleMetadata> {
        let title = item.title().unwrap_or("Untitled").trim().to_string();

        // Get URL from link, guid, or construct from base_url
        let url = item
            .link()
            .map(|s| s.trim().to_string())
            .or_else(|| {
                // Try to get link from guid
                item.guid().map(|g| g.value().trim().to_string())
            })
            .unwrap_or_else(|| {
                // Fallback: construct from base_url and title
                base_url
                    .map(|b| format!("{}/{}", b, urlencoding::encode(&title)))
                    .unwrap_or_default()
            });

        // Parse published date
        let published_date = item.pub_date().and_then(|date_str| {
            // Try multiple date formats
            DateTime::parse_from_rfc2822(date_str)
                .or_else(|_| DateTime::parse_from_rfc3339(date_str))
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        // Extract author
        let author = item.author().map(|s| s.trim().to_string());

        // Extract summary/description
        let summary = item.description().map(|s| {
            // Clean HTML tags from description
            Self::strip_html_tags(s)
        });

        // Try to extract image from description or content
        let image_url = item
            .description()
            .and_then(|desc| Self::extract_image_from_html(desc))
            .or_else(|| {
                item.content()
                    .and_then(|content| Self::extract_image_from_html(content))
            });

        // Generate ID from URL or title
        let id = Self::generate_id_from_url(&url)
            .unwrap_or_else(|| Self::generate_id_from_title(&title));

        let title_clone = title.clone();
        Ok(ArticleMetadata {
            id,
            title: title_clone.clone(), // Mantido para compatibilidade
            original_title: Some(title_clone), // Título original da fonte
            generated_title: None,      // Será preenchido quando o artigo for publicado
            url,
            published_date,
            author,
            summary: summary.clone(),
            image_url,
            source_type: Some("rss".to_string()),
            content_html: None,
            content_text: summary,
            category: None,
            slug: None,
        })
    }

    /// Extract article content from HTML page
    pub async fn parse_html_page(
        html_content: &str,
        url: &str,
        selectors: Option<&HashMap<String, String>>,
    ) -> Result<WebArticle> {
        let document = Html::parse_document(html_content);

        // Default selectors (can be overridden by config)
        let title_selector = selectors
            .and_then(|s| s.get("title"))
            .and_then(|s| Selector::parse(s).ok())
            .unwrap_or_else(|| {
                // Try common title selectors
                Selector::parse("h1, article h1, .article-title, .post-title")
                    .unwrap_or_else(|_| Selector::parse("title").unwrap())
            });

        let content_selector = selectors
            .and_then(|s| s.get("content"))
            .and_then(|s| Selector::parse(s).ok())
            .unwrap_or_else(|| {
                Selector::parse("article, .article-content, .post-content, main")
                    .unwrap_or_else(|_| Selector::parse("body").unwrap())
            });

        // Extract title
        let title = document
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .or_else(|| {
                document
                    .select(&Selector::parse("title").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
            .unwrap_or_else(|| "Untitled".to_string());

        // Extract main content
        let content_html = document
            .select(&content_selector)
            .next()
            .map(|e| e.html())
            .unwrap_or_else(|| html_content.to_string());

        let _content_text = Self::extract_text_from_html(&content_html);

        // Extract published date from meta tags
        let published_date = Self::extract_date_from_meta(&document)
            .or_else(|| Self::extract_date_from_time_tag(&document));

        // Extract author
        let author = Self::extract_author_from_meta(&document)
            .or_else(|| Self::extract_author_from_selectors(&document, selectors));

        // Extract image (og:image, twitter:image, or first large img)
        let image_url = Self::extract_image_from_document(&document);

        // Build metadata JSON
        let mut meta = serde_json::json!({});
        if let Some(date) = published_date {
            meta["published_date"] = serde_json::json!(date.to_rfc3339());
        }
        if let Some(auth) = &author {
            meta["author"] = serde_json::json!(auth);
        }

        Ok(WebArticle {
            url: url.to_string(),
            title,
            published_date,
            author,
            content: content_html,
            image_url,
            meta,
        })
    }

    /// Convert WebArticle to ArticleMetadata
    pub fn web_article_to_metadata(article: WebArticle, id: Option<String>) -> ArticleMetadata {
        let article_id = id.unwrap_or_else(|| {
            Self::generate_id_from_url(&article.url)
                .unwrap_or_else(|| Self::generate_id_from_title(&article.title))
        });

        // Apply content cleaning to remove noise
        use crate::collectors::content_cleaner::ContentCleaner;
        let (cleaned_html, cleaned_text) = ContentCleaner::clean_content_pipeline(&article.content);

        ArticleMetadata {
            id: article_id,
            title: article.title.clone(), // Mantido para compatibilidade
            original_title: Some(article.title.clone()), // Título original da fonte
            generated_title: None,        // Será preenchido quando o artigo for publicado
            url: article.url,
            published_date: article.published_date,
            author: article.author,
            summary: Some(cleaned_text.chars().take(500).collect()),
            image_url: article.image_url,
            source_type: Some("html".to_string()),
            content_html: Some(cleaned_html),
            content_text: Some(cleaned_text),
            category: None,
            slug: None,
        }
    }

    // Helper methods

    #[allow(dead_code)]
    fn strip_html_tags(html: &str) -> String {
        let document = Html::parse_fragment(html);
        document
            .root_element()
            .text()
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Extrai texto de HTML (método público para uso externo)
    pub fn extract_text_from_html(html: &str) -> String {
        let document = Html::parse_fragment(html);
        document
            .root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .replace('\n', " ")
            .replace('\r', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[allow(dead_code)]
    fn extract_image_from_html(html: &str) -> Option<String> {
        let document = Html::parse_fragment(html);
        let img_selector = Selector::parse("img").ok()?;

        document
            .select(&img_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .or_else(|| {
                document
                    .select(&img_selector)
                    .next()
                    .and_then(|img| img.value().attr("data-src"))
            })
            .map(|s| s.to_string())
    }

    fn extract_image_from_document(document: &Html) -> Option<String> {
        // Try og:image first
        let og_image_selector = Selector::parse("meta[property='og:image']").ok()?;
        if let Some(tag) = document.select(&og_image_selector).next() {
            if let Some(url) = tag.value().attr("content") {
                return Some(url.to_string());
            }
        }

        // Try twitter:image
        let twitter_image_selector = Selector::parse("meta[name='twitter:image']").ok()?;
        if let Some(tag) = document.select(&twitter_image_selector).next() {
            if let Some(url) = tag.value().attr("content") {
                return Some(url.to_string());
            }
        }

        // Try first large img tag
        let img_selector = Selector::parse("img").ok()?;
        document
            .select(&img_selector)
            .find(|img| {
                // Prefer images with width/height > 200
                if let Some(width) = img.value().attr("width") {
                    width.parse::<u32>().map(|w| w > 200).unwrap_or(false)
                } else {
                    true // Include if no width specified
                }
            })
            .and_then(|img| img.value().attr("src"))
            .map(|s| s.to_string())
    }

    fn extract_date_from_meta(document: &Html) -> Option<DateTime<Utc>> {
        let meta_selector = Selector::parse("meta[property='article:published_time']").ok()?;
        document
            .select(&meta_selector)
            .next()
            .and_then(|tag| tag.value().attr("content"))
            .and_then(|date_str| {
                DateTime::parse_from_rfc3339(date_str)
                    .or_else(|_| DateTime::parse_from_rfc2822(date_str))
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            })
    }

    fn extract_date_from_time_tag(document: &Html) -> Option<DateTime<Utc>> {
        let time_selector = Selector::parse("time[datetime]").ok()?;
        document
            .select(&time_selector)
            .next()
            .and_then(|tag| tag.value().attr("datetime"))
            .and_then(|date_str| {
                DateTime::parse_from_rfc3339(date_str)
                    .or_else(|_| DateTime::parse_from_rfc2822(date_str))
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            })
    }

    fn extract_author_from_meta(document: &Html) -> Option<String> {
        let author_selector =
            Selector::parse("meta[name='author'], meta[property='article:author']").ok()?;
        document
            .select(&author_selector)
            .next()
            .and_then(|tag| tag.value().attr("content"))
            .map(|s| s.trim().to_string())
    }

    fn extract_author_from_selectors(
        document: &Html,
        selectors: Option<&HashMap<String, String>>,
    ) -> Option<String> {
        let author_selector = selectors
            .and_then(|s| s.get("author"))
            .and_then(|s| Selector::parse(s).ok())
            .unwrap_or_else(|| {
                Selector::parse(".author, .byline, [rel='author']").unwrap_or_else(|_| {
                    // Fallback to any selector that might contain author
                    Selector::parse("span").unwrap()
                })
            });

        document
            .select(&author_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn generate_id_from_url_impl(url: &str) -> Option<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate a unique, consistent ID based on URL hash
        // This ensures the same URL always gets the same ID, preventing duplicates
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        // Return only numeric hash (pure numeric ID)
        // Format: decimal representation of hash (up to 20 digits for maximum uniqueness)
        let numeric_id = hash.to_string();

        Some(numeric_id)
    }

    /// Generate ID from URL (public for use in RSS collector)
    pub fn generate_id_from_url(url: &str) -> Option<String> {
        Self::generate_id_from_url_impl(url)
    }

    /// Generate ID from title (public for use in RSS collector)
    pub fn generate_id_from_title(title: &str) -> String {
        Self::generate_id_from_title_impl(title)
    }

    fn generate_id_from_title_impl(title: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate ID from title hash for consistency
        let mut hasher = DefaultHasher::new();
        title.to_lowercase().hash(&mut hasher);
        let hash = hasher.finish();

        // Return only numeric hash (pure numeric ID)
        // Format: decimal representation of hash (up to 20 digits for maximum uniqueness)
        hash.to_string()
    }
}
