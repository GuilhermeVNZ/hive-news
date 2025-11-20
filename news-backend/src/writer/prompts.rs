// Sophisticated prompt engineering for Nature/Science style content generation
// CRITICAL: Prompts are organized with most important instructions first
// (LLMs give more attention to the beginning of prompts)

use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use rand::Rng;
use tracing::debug;

fn get_site_context(site: &str) -> String {
    match site.to_lowercase().as_str() {
        "airesearch" => r#"AIResearch is a cutting-edge AI news platform focusing on:
- Latest breakthroughs in artificial intelligence research
- Practical applications of ML/deep learning
- Industry news and expert analysis
- **News-style journalism**: Complex topics explained for general audience
- **Simple titles**: Focus on WHAT the discovery means, not HOW it works technically
- **Accessibility first**: Make readers understand WHY it matters
- **Clear explanations**: Use analogies and real-world comparisons
- Users who want technical details can read the original paper
- Emphasis on accuracy and scientific rigor WITH simple language"#
            .to_string(),

        "nature" => r#"Nature magazine - the world's leading scientific publication:
- Highest standards of scientific journalism
- News & Views and Perspectives editorial sections
- Global reach to researchers and policymakers
- Precise, authoritative language
- Emphasis on research impact and significance"#
            .to_string(),

        "science" => r#"Science magazine - official journal of AAAS:
- Perspectives section editorial style
- Broad interdisciplinary coverage
- Clear communication for diverse audiences
- Emphasis on scientific method and evidence
- International research community focus"#
            .to_string(),

        _ => r#"General scientific publication:
- Clear, accurate, accessible communication
- Emphasis on evidence-based reporting
- Professional academic tone
- Broad scientific audience"#
            .to_string(),
    }
}

pub fn build_article_prompt(paper_text: &str, _figures: &[String], site: &str) -> String {
    let site_context = get_site_context(site);

    format!(
        r#"CRITICAL INSTRUCTIONS (READ FIRST):
1. You are writing for {} in Nature/Science magazine editorial style (News & Views, Perspectives sections)
2. **NEVER FABRICATE**: Do not invent citations, references, authors, studies, or data that are not explicitly in the paper below
3. **ONLY USE PAPER CONTENT**: Reference only what exists in the provided paper text
4. NO AI clichés: "delve", "revolutionize", "game-changer", "unlock", "harness", "dive into", "shed light on"
5. NO emojis, NO excessive dashes (—), NO ellipses (...)

---

TARGET PUBLICATION:
{}

---

WRITING STYLE (Nature/Science Editorial - Simplified for General Audience):
- **Opening:** Establish significance immediately (why non-technical readers should care)
- **Voice:** Active, direct, authoritative yet conversational and accessible
- **Structure:** Inverted pyramid - key findings first, details follow
- **Precision:** Reference specific figures, data points, methodology FROM THE PAPER
- **Clarity:** Define technical terms on first use AND use plain language alternatives
- **Flow:** Smooth transitions between concepts - explain as if to an intelligent layperson
- **Accessibility:** Every technical concept should have a simple analogy or real-world comparison
- **Purpose:** Make complex science understandable so readers grasp the importance, then they can read the paper for details

CRITICAL RULES:
- ✅ "The researchers found X (as shown in Figure 2)..."
- ✅ "This approach builds on the methods described in the paper..."
- ✅ "The data shows a 23% increase..." (if paper states this)
- ❌ "Previous work by Zhang et al. (Nature, 2023)..." (unless paper cites this)
- ❌ "Experts suggest..." (unless paper includes expert quotes)
- ❌ "This could lead to cures for cancer..." (unless paper discusses this)

WHAT TO AVOID (AI writing patterns):
- "In a groundbreaking study that could revolutionize..."
- "Scientists have unlocked the secrets of..."
- "This research sheds new light on..."
- "Paradigm-shifting", "game-changing" (unless genuinely warranted)
- Inventing related research not mentioned in paper
- Speculating beyond what paper's data supports

IMAGE CATEGORY SELECTION (REQUIRED - CRITICAL RULES):

You MUST select exactly 3 categories from THIS EXACT LIST ONLY:
ai, coding, crypto, data, ethics, games, hardware, legal, network, quantum_computing, robotics, science, security, sound

CRITICAL CONSTRAINTS:
- ❌ DO NOT create new categories (like "biology", "physics", "medical", etc.)
- ❌ DO NOT use synonyms or variations
- ✅ ONLY use the 14 categories listed above
- ✅ Order by priority: most relevant first, second choice, third choice
- ✅ Must be lowercase, matching the list exactly

SELECTION GUIDELINES:
- ai: Artificial intelligence, machine learning, AI research
- coding: Programming, software development, code
- crypto: Cryptocurrency, blockchain, digital currency
- data: Data storage, data management, servers, datasets, data analysis
- ethics: Ethical considerations, societal impact
- games: Gaming, game development, interactive tech
- hardware: Physical computing, electronics, processors
- legal: Legal issues, regulations, compliance
- network: Networking, connectivity, communication
- quantum_computing: Quantum computing, quantum algorithms, qubits, quantum mechanics
- robotics: Robots, automation, mechanical AI
- science: General scientific research, experiments
- security: Cybersecurity, privacy, protection
- sound: Audio technology, sound processing

EXAMPLES (USE THESE EXACT NAMES):
✓ For neural networks → ["ai", "science", "network"]
✓ For robotics → ["robotics", "ai", "hardware"]
✓ For cybersecurity → ["security", "network", "crypto"]
✓ For data analysis → ["data", "ai", "coding"]
✓ For chip research → ["hardware", "science", "ai"]

Include this as "image_categories" array in your JSON response.

REQUIRED ARTICLE STRUCTURE:
1. **Opening Hook** (2-3 sentences: significance and context from paper - explain WHY non-technical readers should care)
2. **Key Finding** (what researchers discovered - explain in plain language, avoid jargon)
3. **Methodology** (how they did it - simplified explanation focusing on the approach, not technical details)
4. **Results Analysis** (what data shows - reference figures from paper, use simple language)
5. **Context** (why it matters - real-world implications for regular readers)
6. **Limitations** (what remains unknown - from paper's limitations section)

TITLE REQUIREMENTS (CRITICAL):
- **CRITICAL**: The generated title MUST be DIFFERENT from the original title in the paper. NEVER use the same title as the original source.
- **News-focused**: Explain what the breakthrough means to everyday readers
- **Simple language**: Avoid technical jargon, write for general audience
- **Active voice**: Make it engaging and accessible
- **Specific**: Include what was achieved or discovered
- **Hook**: Capture attention like a news headline

BAD TITLES (Too technical):
❌ "Graph Neural Networks for Spatiotemporal Dynamics"
❌ "Gradient-Based Optimization in High-Dimensional Space"
❌ "Multi-Agent Path Planning Algorithm for AUV Coordination"

GOOD TITLES (News-focused):
✓ "AI Can Now Predict Complex Data Relationships Without Violating Privacy"
✓ "New Method Helps Scientists Share Data Securely Without Losing Accuracy"
✓ "Robots Navigate Ocean Currents Using Real-Time Weather Data"

SUBTITLE REQUIREMENTS (CRITICAL):
- **SEO-optimized**: A compelling summary optimized for search engines
- **Maximum 2 lines**: Keep it concise and impactful
- **Add tension**: Should create curiosity, consequence, or reason why this matters
- **Clear value proposition**: Explain the significance in simple terms
- **No technical jargon**: Write for general audience understanding

GOOD SUBTITLES:
✓ "A new AI method can generate fake data that captures real-world patterns so accurately that researchers can use it for sensitive analysis—without ever touching the original information."
✓ "Scientists discovered the universe might be two billion years younger than previously thought by using more precise measurements."
✓ "Cancer cells have a hidden escape route that researchers just identified, opening new doors for treatment."

EXAMPLES OF GOOD OPENING LINES (News style):
✓ "A new AI method can generate fake data that captures real-world patterns so accurately that researchers can use it for sensitive analysis—without ever touching the original information."
✓ "Scientists discovered the universe might be two billion years younger than previously thought by using more precise measurements."
✓ "Cancer cells have a hidden escape route that researchers just identified, opening new doors for treatment."

## PAPER TEXT (YOUR ONLY SOURCE):
{}

## DELIVERABLE:
Write a 500-800 word article in Nature/Science editorial style. Use ONLY information from the paper above.

Format:
# [Compelling, Specific Title - Nature/Science Style]

[Article body - based ONLY on paper content...]

CRITICAL JSON FORMAT - YOU MUST FOLLOW THIS EXACT STRUCTURE:
{{
  "title": "Your title here",
  "subtitle": "SEO-optimized subtitle (max 2 lines) - compelling summary that adds tension or explains significance",
  "article_text": "Full article body text here - all content in one string field",
  "image_categories": ["category1", "category2", "category3"],
  "linkedin_post": "Your LinkedIn post text here (300 chars max) - engaging summary for professionals",
  "x_post": "Your X/Twitter post text here (280 chars max) - concise, punchy, shareable",
  "shorts_script": "Your YouTube Shorts script here (2 minutes, ~300 words) - hook, key points, call to action"
}}

⚠️ IMPORTANT RULES:
- "subtitle" MUST be a STRING field at the root level
- "subtitle" MUST be SEO-optimized, maximum 2 lines, and create curiosity or explain significance
- "article_text" MUST be a STRING field at the root level (NOT nested in an "article" object)
- "article_text" MUST contain the complete article text in one string
- DO NOT create nested objects like {{"article": {{"opening_hook": "...", "key_finding": "..."}}}}
- All article content goes directly into the "article_text" string field
- "linkedin_post": Professional, engaging summary (300 chars max) for LinkedIn audience
- "x_post": Concise, punchy, shareable post (280 chars max) for X/Twitter
- "shorts_script": YouTube Shorts script (2 minutes, ~300 words) with hook, key points, call to action
- Return ONLY valid JSON - no markdown, no extra formatting

TITLE REQUIREMENTS (CRITICAL):
- **CRITICAL**: The generated title MUST be DIFFERENT from the original title in the paper. NEVER use the same title as the original source.
- MAXIMUM 8 WORDS (short, punchy, viral)
- STRONG HOOK to make users WANT to click and read
- Create curiosity, tension, or surprise
- Make readers NEED to know more

BAD TITLES (too long, no hook):
❌ "A New Approach to Machine Learning Optimization in Deep Neural Networks"
❌ "Understanding the Fundamentals of Quantum Computing Applications"

GOOD TITLES (short, hooky, irresistible):
✓ "AI Agents Fall Short at Scientific Discovery"
✓ "Scientists Find Hidden Pattern in Neural Networks"
✓ "This AI Breakthrough May Be Wrong"  
"#,
        site, site_context, paper_text
    )
}

/// Loads a random prompt from the news_randomizer directory
/// Returns the prompt with {paper_text} placeholder replaced by the actual article text
pub fn load_random_news_prompt(article_text: &str) -> Result<String> {
    // Use relative path that works both locally and in Docker
    // The prompts directory is at: src/writer/prompts/news_randomizer/
    
    let prompt_dir = get_news_randomizer_dir()?;
    
    // List all .txt files in the directory
    let mut prompt_files = Vec::new();
    if prompt_dir.exists() && prompt_dir.is_dir() {
        let entries = fs::read_dir(&prompt_dir)
            .with_context(|| format!("Failed to read prompt directory: {}", prompt_dir.display()))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "txt" {
                        prompt_files.push(path);
                    }
                }
            }
        }
    }
    
    if prompt_files.is_empty() {
        anyhow::bail!(
            "No prompt files found in: {}. Falling back to default prompt.",
            prompt_dir.display()
        );
    }
    
    // Select a random file
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..prompt_files.len());
    let selected_file = &prompt_files[random_index];
    
    // Read the prompt template
    let prompt_template = fs::read_to_string(selected_file)
        .with_context(|| format!("Failed to read prompt file: {}", selected_file.display()))?;
    
    // Replace {paper_text} placeholder with actual article text
    let final_prompt = if prompt_template.contains("{paper_text}") {
        prompt_template.replace("{paper_text}", article_text)
    } else {
        // If no placeholder, append article text at the end (backward compatibility)
        format!("{}\n\n## ARTICLE TEXT (YOUR ONLY SOURCE):\n{}", prompt_template, article_text)
    };
    
    println!(
        "  🎲 Using randomized news prompt: {}",
        selected_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    );
    
    Ok(final_prompt)
}

/// Gets the path to the news_randomizer prompts directory
/// Uses relative paths that work both locally and in Docker
fn get_news_randomizer_dir() -> Result<PathBuf> {
    // Try multiple possible paths (works in different environments)
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    
    // Get executable path to find source code location in Docker
    let exe_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
    
    // Build list of possible paths
    let mut possible_paths = Vec::new();
    
    // 1. Relative to current working directory (most common case - local development)
    possible_paths.push(current_dir.join("src/writer/prompts/news_randomizer"));
    
    // 2. Relative to workspace root (if running from workspace root)
    possible_paths.push(current_dir.join("news-backend/src/writer/prompts/news_randomizer"));
    
    // 3. Docker: /app/news-backend/src/writer/prompts/news_randomizer (where source code is mounted)
    possible_paths.push(PathBuf::from("/app/news-backend/src/writer/prompts/news_randomizer"));
    
    // 4. Using path resolver (if NEWS_BASE_DIR is set) - for data directories
    possible_paths.push(crate::utils::path_resolver::resolve_workspace_path("news-backend/src/writer/prompts/news_randomizer"));
    
    // 5. Direct from NEWS_BASE_DIR if it points to source (Docker scenario where code might be in /data)
    if let Ok(base_dir) = std::env::var("NEWS_BASE_DIR") {
        let base_path = PathBuf::from(base_dir);
        // Try /data/src/writer/prompts/news_randomizer (if source is mounted there)
        possible_paths.push(base_path.join("src/writer/prompts/news_randomizer"));
        // Try /data/news-backend/src/writer/prompts/news_randomizer
        possible_paths.push(base_path.join("news-backend/src/writer/prompts/news_randomizer"));
    }
    
    // 6. From executable location - navigate back to source code
    if let Some(exe_dir) = exe_path {
        // If executable is in /usr/local/bin/, try /app/news-backend (Docker default)
        if exe_dir.to_string_lossy().contains("/usr/local/bin") {
            possible_paths.push(PathBuf::from("/app/news-backend/src/writer/prompts/news_randomizer"));
        }
        // Try relative to executable (if running from news-backend directory)
        possible_paths.push(exe_dir.join("src/writer/prompts/news_randomizer"));
        // Try going up from executable (for different Docker setups)
        if let Some(parent) = exe_dir.parent() {
            possible_paths.push(parent.join("news-backend/src/writer/prompts/news_randomizer"));
            // Try /app (Docker common location)
            if parent.to_string_lossy() == "/usr/local" {
                possible_paths.push(PathBuf::from("/app/news-backend/src/writer/prompts/news_randomizer"));
            }
        }
    }
    
    // 7. Fallback: try from environment or default
    possible_paths.push(PathBuf::from("src/writer/prompts/news_randomizer"));
    
    // Find the first path that exists
    for path in &possible_paths {
        if path.exists() && path.is_dir() {
            debug!("Found news_randomizer prompts directory at: {}", path.display());
            return Ok(path.clone());
        }
    }
    
    // If none exist, return error with all tried paths for debugging
    let tried_paths: Vec<String> = possible_paths.iter()
        .map(|p| p.display().to_string())
        .collect();
    
    Err(anyhow::anyhow!(
        "No prompt files found in: {}. Tried paths: {:?}",
        possible_paths[0].display(),
        tried_paths
    ))
}

/// Loads a random prompt from the article_randomizer directory
/// Returns the prompt with {paper_text} placeholder replaced by the actual paper text
pub fn load_random_article_prompt(paper_text: &str) -> Result<String> {
    // Use relative path that works both locally and in Docker
    // The prompts directory is at: src/writer/prompts/article_randomizer/
    // We need to resolve this relative to the executable or workspace root
    
    let prompt_dir = get_prompt_randomizer_dir()?;
    
    // List all .txt files in the directory
    let mut prompt_files = Vec::new();
    if prompt_dir.exists() && prompt_dir.is_dir() {
        let entries = fs::read_dir(&prompt_dir)
            .with_context(|| format!("Failed to read prompt directory: {}", prompt_dir.display()))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "txt" {
                        prompt_files.push(path);
                    }
                }
            }
        }
    }
    
    if prompt_files.is_empty() {
        anyhow::bail!(
            "No prompt files found in: {}. Falling back to default prompt.",
            prompt_dir.display()
        );
    }
    
    // Select a random file
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..prompt_files.len());
    let selected_file = &prompt_files[random_index];
    
    // Read the prompt template
    let prompt_template = fs::read_to_string(selected_file)
        .with_context(|| format!("Failed to read prompt file: {}", selected_file.display()))?;
    
    // Replace {paper_text} placeholder
    let mut final_prompt = prompt_template.replace("{paper_text}", paper_text);
    
    // CRITICAL: DeepSeek API requires the word "json" in the prompt when using response_format: json_object
    // Add it if not present (case-insensitive check)
    if !final_prompt.to_lowercase().contains("json") {
        final_prompt.push_str("\n\n⚠️ IMPORTANT: You MUST return your response as valid JSON format with the required fields.");
    }
    
    println!(
        "  🎲 Using randomized prompt: {}",
        selected_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    );
    
    Ok(final_prompt)
}

/// Gets the path to the article_randomizer prompts directory
/// Uses relative paths that work both locally and in Docker
fn get_prompt_randomizer_dir() -> Result<PathBuf> {
    // Try multiple possible paths (works in different environments)
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    
    // Get executable path to find source code location in Docker
    let exe_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
    
    // Build list of possible paths
    let mut possible_paths = Vec::new();
    
    // 1. Relative to current working directory (most common case - local development)
    possible_paths.push(current_dir.join("src/writer/prompts/article_randomizer"));
    
    // 2. Relative to workspace root (if running from workspace root)
    possible_paths.push(current_dir.join("news-backend/src/writer/prompts/article_randomizer"));
    
    // 3. Docker: /app/news-backend/src/writer/prompts/article_randomizer (where source code is mounted)
    possible_paths.push(PathBuf::from("/app/news-backend/src/writer/prompts/article_randomizer"));
    
    // 4. Using path resolver (if NEWS_BASE_DIR is set) - for data directories
    possible_paths.push(crate::utils::path_resolver::resolve_workspace_path("news-backend/src/writer/prompts/article_randomizer"));
    
    // 5. Direct from NEWS_BASE_DIR if it points to source (Docker scenario where code might be in /data)
    if let Ok(base_dir) = std::env::var("NEWS_BASE_DIR") {
        let base_path = PathBuf::from(base_dir);
        // Try /data/src/writer/prompts/article_randomizer (if source is mounted there)
        possible_paths.push(base_path.join("src/writer/prompts/article_randomizer"));
        // Try /data/news-backend/src/writer/prompts/article_randomizer
        possible_paths.push(base_path.join("news-backend/src/writer/prompts/article_randomizer"));
    }
    
    // 6. From executable location - navigate back to source code
    if let Some(exe_dir) = exe_path {
        // If executable is in /usr/local/bin/, try /app/news-backend (Docker default)
        if exe_dir.to_string_lossy().contains("/usr/local/bin") {
            possible_paths.push(PathBuf::from("/app/news-backend/src/writer/prompts/article_randomizer"));
        }
        // Try relative to executable (if running from news-backend directory)
        possible_paths.push(exe_dir.join("src/writer/prompts/article_randomizer"));
        // Try going up from executable (for different Docker setups)
        if let Some(parent) = exe_dir.parent() {
            possible_paths.push(parent.join("news-backend/src/writer/prompts/article_randomizer"));
            // Try /app (Docker common location)
            if parent.to_string_lossy() == "/usr/local" {
                possible_paths.push(PathBuf::from("/app/news-backend/src/writer/prompts/article_randomizer"));
            }
        }
    }
    
    // 7. Fallback: try from environment or default
    possible_paths.push(PathBuf::from("src/writer/prompts/article_randomizer"));
    
    // Find the first path that exists
    for path in &possible_paths {
        if path.exists() && path.is_dir() {
            debug!("Found article_randomizer prompts directory at: {}", path.display());
            return Ok(path.clone());
        }
    }
    
    // If none exist, return error with all tried paths for debugging
    let tried_paths: Vec<String> = possible_paths.iter()
        .map(|p| p.display().to_string())
        .collect();
    
    Err(anyhow::anyhow!(
        "No prompt files found in: {}. Tried paths: {:?}",
        possible_paths[0].display(),
        tried_paths
    ))
}

pub fn build_social_script_prompt(article: &str, paper_title: &str) -> String {
    format!(
        r#"CRITICAL INSTRUCTIONS (READ FIRST):
1. Create viral social media content based on Nature/Science style article below
2. **FACT-BASED ONLY**: Do not add information not in the article
3. Start each piece with a VIRAL HOOK (surprising fact, bold question, tension)
4. Match Nature/Science credibility - no clickbait lies

---

## ARTICLE (Nature/Science style - YOUR ONLY SOURCE):
{}

## ORIGINAL PAPER TITLE:
{}

---

DELIVERABLES:

### 1. LINKEDIN POST (300 characters max)
STRUCTURE:
- Line 1: VIRAL HOOK (surprising fact/question from article)
- Line 2: Core finding in 1 sentence
- Line 3: Thought-provoking question or call-to-action

STYLE:
- Professional tone (Nature/Science authority)
- No emojis, no hashtag spam
- Fact-based (from article only)

EXAMPLE HOOKS:
✓ "What if AI could predict every protein structure in existence?"
✓ "The universe might be 2 billion years younger than we thought."
✓ "Cancer cells have a hidden escape route. Researchers just found it."

### 2. X POST (280 characters max)
STRUCTURE:
- HOOK first (stop the scroll)
- Key insight compressed
- Paper relevance

STYLE:
- Match Nature/Science credibility
- No fabrication
- No hashtags unless natural

### 3. YOUTUBE SHORTS SCRIPT (2 minutes / ~300 words)

FORMAT (alternating blocks with timestamps):

[VOICEOVER 0:00-0:05]
[Viral hook - surprising statement/question from article creating immediate tension]

[VISUAL DIRECTION]
[Director note: "Show Figure 2 with zoom on peak distribution" OR "Animated text: key stat"]

[VOICEOVER 0:05-0:15]
[Setup context - why viewer should care, from article]

[VISUAL DIRECTION]
[Transition guidance, what to emphasize]

SCRIPT STRUCTURE (2 minutes):
- Hook (0:00-0:05) - Grab attention instantly
- Context (0:05-0:20) - Why this matters
- Problem/Question (0:20-0:40) - What researchers asked
- Key Finding (0:40-1:00) - What they discovered
- How It Works (1:00-1:30) - Explain the science
- Implications (1:30-1:50) - Real-world impact FROM ARTICLE
- Closing Hook (1:50-2:00) - Memorable takeaway

VOICEOVER STYLE:
- Conversational but intelligent
- No AI clichés or buzzwords
- Active voice, direct statements
- Assume intelligent audience

VISUAL REQUIREMENTS:
- Reference specific figures from paper
- Include zoom/pan directions
- Suggest text overlays for key numbers
- Indicate pacing (fast cuts vs holds)

CRITICAL: You MUST return ONLY a JSON object with these EXACT fields:
{{
  "linkedin_post": "Your LinkedIn post text here (300 chars max)",
  "x_post": "Your X/Twitter post text here (280 chars max)",
  "shorts_script": "Your YouTube Shorts script here (2 minutes, ~300 words)"
}}

⚠️ DO NOT include "title" or "article_text" fields. ONLY return linkedin_post, x_post, and shorts_script.

CRITICAL JSON FORMAT REQUIREMENT:
You MUST return ONLY a JSON object with EXACTLY these 3 fields (no more, no less):
{{
  "linkedin_post": "...",
  "x_post": "...",
  "shorts_script": "..."
}}

⚠️ FORBIDDEN FIELDS: Do NOT include "title", "article_text", "subtitle", or any other fields.
⚠️ REQUIRED FIELDS: ONLY "linkedin_post", "x_post", and "shorts_script" are allowed.

Return your response as valid JSON format with ONLY the 3 required fields.
"#,
        article, paper_title
    )
}
