// Sophisticated prompt engineering for Nature/Science style content generation
// CRITICAL: Prompts are organized with most important instructions first
// (LLMs give more attention to the beginning of prompts)

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
- Emphasis on accuracy and scientific rigor WITH simple language"#.to_string(),
        
        "nature" => r#"Nature magazine - the world's leading scientific publication:
- Highest standards of scientific journalism
- News & Views and Perspectives editorial sections
- Global reach to researchers and policymakers
- Precise, authoritative language
- Emphasis on research impact and significance"#.to_string(),
        
        "science" => r#"Science magazine - official journal of AAAS:
- Perspectives section editorial style
- Broad interdisciplinary coverage
- Clear communication for diverse audiences
- Emphasis on scientific method and evidence
- International research community focus"#.to_string(),
        
        _ => r#"General scientific publication:
- Clear, accurate, accessible communication
- Emphasis on evidence-based reporting
- Professional academic tone
- Broad scientific audience"#.to_string()
    }
}

pub fn build_article_prompt(paper_text: &str, _figures: &[String], site: &str) -> String {
    let site_context = get_site_context(site);
    
    format!(r#"CRITICAL INSTRUCTIONS (READ FIRST):
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
ai, coding, crypto, database, ethics, games, hardware, legal, network, robotics, science, security, sound

CRITICAL CONSTRAINTS:
- ❌ DO NOT create new categories (like "biology", "physics", "medical", etc.)
- ❌ DO NOT use synonyms or variations
- ✅ ONLY use the 13 categories listed above
- ✅ Order by priority: most relevant first, second choice, third choice
- ✅ Must be lowercase, matching the list exactly

SELECTION GUIDELINES:
- ai: Artificial intelligence, machine learning, AI research
- coding: Programming, software development, code
- crypto: Cryptocurrency, blockchain, digital currency
- database: Data storage, data management, servers
- ethics: Ethical considerations, societal impact
- games: Gaming, game development, interactive tech
- hardware: Physical computing, electronics, processors
- legal: Legal issues, regulations, compliance
- network: Networking, connectivity, communication
- robotics: Robots, automation, mechanical AI
- science: General scientific research, experiments
- security: Cybersecurity, privacy, protection
- sound: Audio technology, sound processing

EXAMPLES (USE THESE EXACT NAMES):
✓ For neural networks → ["ai", "science", "network"]
✓ For robotics → ["robotics", "ai", "hardware"]
✓ For cybersecurity → ["security", "network", "crypto"]
✓ For data analysis → ["database", "ai", "coding"]
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
  "image_categories": ["category1", "category2", "category3"]
}}

⚠️ IMPORTANT RULES:
- "subtitle" MUST be a STRING field at the root level
- "subtitle" MUST be SEO-optimized, maximum 2 lines, and create curiosity or explain significance
- "article_text" MUST be a STRING field at the root level (NOT nested in an "article" object)
- "article_text" MUST contain the complete article text in one string
- DO NOT create nested objects like {{"article": {{"opening_hook": "...", "key_finding": "..."}}}}
- All article content goes directly into the "article_text" string field
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
"#, site, site_context, paper_text)
}

pub fn build_social_script_prompt(article: &str, paper_title: &str) -> String {
    format!(r#"CRITICAL INSTRUCTIONS (READ FIRST):
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

OUTPUT FORMAT (JSON):
{{
  "linkedin_post": "...",
  "x_post": "...",
  "shorts_script": "..."
}}
"#, article, paper_title)
}
