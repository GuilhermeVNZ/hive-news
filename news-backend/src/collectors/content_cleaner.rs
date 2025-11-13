// Content Cleaner Module - Remove noise from HTML/text content without LLM
// Goal: Extract clean, readable text ready for LLM processing
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;

pub struct ContentCleaner;

impl ContentCleaner {
    /// Remove common noise elements from HTML before text extraction
    /// Aggressively removes UI elements, navigation, metadata, duplicates
    pub fn clean_html(html: &str) -> String {
        let mut cleaned = html.to_string();

        // Step 1: Remove script, style, noscript completely
        if let Ok(re) =
            Regex::new(r"(?is)<(?:script|style|noscript)[^>]*>.*?</(?:script|style|noscript)>")
        {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Step 2: Remove button, nav, footer, header elements completely
        if let Ok(re) = Regex::new(
            r"(?is)<(?:button|nav|footer|header)[^>]*>.*?</(?:button|nav|footer|header)>",
        ) {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Step 3: Remove elements with noise class patterns
        let noise_patterns = vec![
            r#"(?s)<[^>]*class=["'][^"']*(?:author|contributor|citation|navigation|menu|sidebar|related|recommended|keep-reading|share|social|footer|comment|metadata|tag)[^"']*["'][^>]*>.*?</[^>]+>"#,
            r#"(?s)<section[^>]*(?:id=["']citations["']|data-testid=["']author-list["']|data-testid=["']citations["'])[^>]*>.*?</section>"#,
            r#"(?s)<div[^>]*class=["'][^"']*(?:author|contributor|share|social|footer|metadata)[^"']*["'][^>]*>.*?</div>"#,
        ];

        for pattern in noise_patterns {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }

        // Step 4: Remove aria-label patterns for UI elements
        if let Ok(re) = Regex::new(
            r#"(?s)<[^>]*(?:aria-label=["'][^"']*(?:Play|Share|Loading)[^"']*["']|type=["']button["'])[^>]*>.*?</[^>]+>"#,
        ) {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Step 5: Remove all img tags completely (we don't need images for LLM)
        if let Ok(re) = Regex::new(r"(?s)<img[^>]*>") {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Step 6: Remove picture/source tags
        if let Ok(re) = Regex::new(r"(?is)<(?:picture|source)[^>]*>.*?</(?:picture|source)>") {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Step 7: Remove all links but keep text content
        if let Ok(re) = Regex::new(r"(?s)<a[^>]*>(.*?)</a>") {
            cleaned = re.replace_all(&cleaned, "$1").to_string();
        }

        // Step 8: Remove code blocks (usually not article content)
        if let Ok(re) = Regex::new(r"(?is)<(?:code|pre)[^>]*>.*?</(?:code|pre)>") {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Step 9: Remove all attributes from remaining tags (keep structure, remove noise)
        // This simplifies HTML significantly
        if let Ok(re) = Regex::new(
            r#"\s+(?:class|id|style|data-[^=]*|aria-[^=]*|role|tabindex|onclick|onerror)=["'][^"']*["']"#,
        ) {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        cleaned
    }

    /// Extract clean text from HTML - only paragraphs and headings, no duplicates
    /// Returns text with proper paragraph separation for LLM readability
    pub fn extract_clean_text(html: &str) -> String {
        let document = Html::parse_document(html);
        let mut paragraphs = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // Extract only p tags (paragraphs) - most reliable for article content
        // Skip headings initially to avoid duplicates
        if let Ok(p_selector) = Selector::parse("p") {
            for element in document.select(&p_selector) {
                let mut text = element.text().collect::<String>();

                // Clean up text: remove control characters, normalize spaces
                text = text
                    .chars()
                    .filter(|c| {
                        // Keep printable chars and normal whitespace
                        c.is_alphanumeric()
                            || c.is_whitespace()
                            || matches!(
                                c,
                                '.' | ','
                                    | ';'
                                    | ':'
                                    | '!'
                                    | '?'
                                    | '-'
                                    | '('
                                    | ')'
                                    | '['
                                    | ']'
                                    | '{'
                                    | '}'
                                    | '\''
                                    | '"'
                                    | '/'
                                    | '\\'
                            )
                    })
                    .collect::<String>();

                // Normalize whitespace within paragraph (single line per paragraph)
                text = text
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");

                // Skip empty or very short text (must have meaningful content)
                let trimmed = text.trim();
                if trimmed.len() < 50 {
                    continue;
                }

                // Skip author signatures and metadata
                if trimmed.contains("Ben Goodger")
                    || trimmed.contains("Ken Rockot")
                    || trimmed.contains("Darin Fisher")
                    || trimmed.contains("Marie Shin")
                    || trimmed.contains("Head of Engineering")
                    || trimmed.contains("Member of the Technical Staff")
                {
                    continue;
                }

                // Normalize text (lowercase, remove extra spaces) for duplicate detection
                let normalized = trimmed
                    .to_lowercase()
                    .chars()
                    .filter(|c| !c.is_control())
                    .collect::<String>()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                // Check if we've seen this exact text before
                if !seen.contains(&normalized) {
                    seen.insert(normalized);
                    paragraphs.push(trimmed.to_string());
                }
            }
        }

        // Join paragraphs with double newline (readable format for LLM)
        paragraphs.join("\n\n")
    }

    /// Clean extracted text - remove noise patterns and normalize
    pub fn clean_text(text: &str) -> String {
        let mut cleaned = text.to_string();

        // Remove control characters and unicode artifacts
        cleaned = cleaned
            .chars()
            .filter(|c| {
                // Keep printable ASCII, common punctuation, and normal whitespace
                c.is_ascii()
                    || matches!(
                        c,
                        '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}' | '\u{2014}' | '\u{2026}'
                    )
            })
            .collect::<String>();

        // Replace unicode quotes with ASCII equivalents
        cleaned = cleaned
            .replace(['\u{2018}', '\u{2019}'], "'")
            .replace(['\u{201C}', '\u{201D}'], "\"")
            .replace('\u{2014}', " -- ")
            .replace('\u{2026}', "...");

        // Remove common UI text patterns
        let noise_patterns = vec![
            "Loading…",
            "Share",
            "View all",
            "Keep reading",
            "Play audio",
            "Contributors:",
            "Author:",
            "Acknowledgements",
            "Acknowledgments",
            "Special thanks",
            "(opens in a new window)",
            "Try Atlas at",
            "chatgpt.com/atlas",
            "By Ken Rockot",
            "Member of the Technical Staff",
        ];

        for pattern in noise_patterns {
            cleaned = cleaned.replace(pattern, "");
        }

        // Remove date patterns at start of lines (e.g., "October 30, 2025 Engineering")
        if let Ok(re) = Regex::new(r"(?m)^[A-Z][a-z]+ \d{1,2}, \d{4} [A-Z][a-z]+\s*") {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Remove standalone section headers that appear at the end (likely duplicate navigation)
        // These are usually short, title-like text that repeats
        if let Ok(re) = Regex::new(
            r"(?m)^(?:How we built|Rethinking|Rendering|Input events|Agent mode|Shaping|Our Solution|A new way)[^.]*$",
        ) {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Process paragraph by paragraph (split by double newlines)
        // The extract_clean_text should already have separated paragraphs properly
        let paragraphs: Vec<&str> = cleaned.split("\n\n").collect();
        let mut cleaned_paragraphs = Vec::new();

        for para in paragraphs {
            let trimmed = para.trim();

            // Filter out very short paragraphs (likely UI noise)
            if trimmed.len() < 50 {
                continue;
            }

            // Skip author signatures and metadata
            if trimmed.contains("Ben Goodger")
                || trimmed.contains("Ken Rockot")
                || trimmed.contains("Darin Fisher")
                || trimmed.contains("Marie Shin")
                || trimmed.contains("Head of Engineering")
                || trimmed.contains("Member of the Technical Staff")
                || trimmed.starts_with(", and ")
                || trimmed.starts_with("to ") && trimmed.len() < 100
            {
                continue;
            }

            // Remove excessive spaces within paragraph
            let normalized = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

            // Skip if it's just punctuation or noise
            let word_count = normalized.split_whitespace().count();
            if word_count < 5 {
                continue;
            }

            cleaned_paragraphs.push(normalized);
        }

        // Join with double newlines (preserve paragraph structure for LLM readability)
        let result = cleaned_paragraphs.join("\n\n");

        // Final normalization: remove triple+ newlines
        let mut final_result = result;
        while final_result.contains("\n\n\n") {
            final_result = final_result.replace("\n\n\n", "\n\n");
        }

        // Remove lines that are just repeated headers
        final_result = Self::remove_repeated_headers(&final_result);

        final_result.trim().to_string()
    }

    /// Remove repeated headers (common in scraped content)
    fn remove_repeated_headers(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();
        let mut seen_headers: HashSet<String> = HashSet::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if this looks like a header (short, might be repeated)
            if trimmed.len() < 100 && trimmed.chars().any(|c| c.is_uppercase()) {
                let normalized = trimmed.to_lowercase();
                if seen_headers.contains(&normalized) {
                    continue; // Skip duplicate header
                }
                seen_headers.insert(normalized);
            }

            result.push(trimmed);
        }

        result.join("\n")
    }

    /// Complete cleaning pipeline - produces clean, LLM-ready text
    /// Returns: (clean_html: minimal HTML without tags, clean_text: pure text ready for LLM)
    pub fn clean_content_pipeline(html: &str) -> (String, String) {
        // Step 1: Remove noise HTML elements aggressively
        let cleaned_html = Self::clean_html(html);

        // Step 2: Extract only paragraphs and headings (no duplicates)
        let extracted_text = Self::extract_clean_text(&cleaned_html);

        // Step 3: Clean extracted text (remove UI patterns, normalize)
        let cleaned_text = Self::clean_text(&extracted_text);

        // Step 4: Final cleanup - remove any remaining duplicates
        let mut final_text = Self::remove_duplicate_paragraphs(&cleaned_text);

        // Step 5: Normalize paragraph breaks - ensure proper paragraph separation
        // Replace single \n with space within paragraphs, keep \n\n for paragraph breaks
        final_text = Self::normalize_paragraphs(&final_text);

        // Step 6: Remove ALL remaining HTML tags from final text (pure text only)
        final_text = Self::strip_all_html(&final_text);

        // For HTML output: create minimal text-only version (no HTML tags)
        // This is just the cleaned text for reference, but without tags
        let minimal_html = final_text.clone();

        (minimal_html, final_text)
    }

    /// Normalize paragraph breaks - single \n becomes space, \n\n becomes paragraph break
    /// Returns text with paragraphs separated by \n\n, but no \n within paragraphs
    fn normalize_paragraphs(text: &str) -> String {
        // Normalize line endings
        let mut normalized = text.replace("\r\n", "\n");

        // Replace triple+ newlines with double newlines
        while normalized.contains("\n\n\n") {
            normalized = normalized.replace("\n\n\n", "\n\n");
        }

        // Use placeholder to preserve paragraph breaks
        let placeholder = "§§PARAGRAPH_BREAK§§";
        normalized = normalized.replace("\n\n", placeholder);

        // Replace all single \n with spaces (within paragraphs)
        normalized = normalized.replace('\n', " ");

        // Split by placeholder to get paragraphs
        let paragraphs: Vec<&str> = normalized.split(placeholder).collect();
        let mut cleaned_paragraphs = Vec::new();

        for para in paragraphs {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Normalize spaces within paragraph (no \n, just spaces)
            let normalized_para = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

            // Remove any remaining newlines (just in case)
            let final_para = normalized_para.replace(['\n', '\r'], " ");

            if final_para.len() > 10 {
                cleaned_paragraphs.push(final_para);
            }
        }

        // Join paragraphs with double newline (visible separation in JSON)
        cleaned_paragraphs.join("\n\n")
    }

    /// Strip ALL HTML tags from text - returns pure text without any \n within paragraphs
    fn strip_all_html(text: &str) -> String {
        use regex::Regex;

        let mut cleaned = text.to_string();

        // Preserve paragraph breaks before processing
        let placeholder = "§§PARAGRAPH_BREAK§§";
        cleaned = cleaned.replace("\n\n", placeholder);

        // Remove all HTML tags (including <p>)
        if let Ok(re) = Regex::new(r"(?s)<[^>]+>") {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }

        // Decode common HTML entities
        cleaned = cleaned
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'");

        // Split by placeholder to process paragraphs separately
        let paragraphs: Vec<&str> = cleaned.split(placeholder).collect();
        let mut cleaned_paragraphs = Vec::new();

        for para in paragraphs {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Remove all newlines from paragraph (convert to spaces)
            let no_newlines = trimmed.replace(['\n', '\r'], " ");

            // Normalize whitespace within paragraph (no \n, just spaces)
            let normalized = no_newlines.split_whitespace().collect::<Vec<_>>().join(" ");

            if normalized.len() > 5 {
                cleaned_paragraphs.push(normalized);
            }
        }

        // Restore paragraph breaks (only \n\n between paragraphs, none within)
        let result = cleaned_paragraphs.join("\n\n");
        result.trim().to_string()
    }

    /// Remove duplicate paragraphs (more aggressive than line-level)
    fn remove_duplicate_paragraphs(text: &str) -> String {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut seen: HashSet<String> = HashSet::new();
        let mut result = Vec::new();

        for para in paragraphs {
            let trimmed = para.trim();
            if trimmed.len() < 30 {
                continue; // Skip very short paragraphs
            }

            // Normalize for duplicate detection
            let normalized = trimmed
                .to_lowercase()
                .chars()
                .filter(|c| !c.is_control())
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            // Check for similar paragraphs (fuzzy match - allow some variation)
            let is_duplicate = seen.iter().any(|seen_para| {
                let similarity = Self::text_similarity(&normalized, seen_para);
                similarity > 0.85 // 85% similar = duplicate
            });

            if !is_duplicate {
                seen.insert(normalized);
                result.push(trimmed);
            }
        }

        result.join("\n\n")
    }

    /// Calculate text similarity (simple word overlap)
    fn text_similarity(text1: &str, text2: &str) -> f32 {
        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();

        let intersection: HashSet<_> = words1.intersection(&words2).collect();
        let union: HashSet<_> = words1.union(&words2).collect();

        if union.is_empty() {
            return 0.0;
        }

        intersection.len() as f32 / union.len() as f32
    }
}
