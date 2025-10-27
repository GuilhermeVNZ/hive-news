// Sophisticated prompt engineering for Nature/Science style content generation
// CRITICAL: Prompts are organized with most important instructions first
// (LLMs give more attention to the beginning of prompts)

fn get_site_context(site: &str) -> String {
    match site.to_lowercase().as_str() {
        "airesearch" => r#"AIResearch is a cutting-edge AI news platform focusing on:
- Latest breakthroughs in artificial intelligence research
- Practical applications of ML/deep learning
- Industry news and expert analysis
- Accessible explanations for technical audiences
- Emphasis on accuracy and scientific rigor"#.to_string(),
        
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

pub fn build_article_prompt(paper_text: &str, figures: &[String], site: &str) -> String {
    let figures_list = figures.join(", ");
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

WRITING STYLE (Nature/Science Editorial):
- **Opening:** Establish significance immediately (why readers should care)
- **Voice:** Active, direct, authoritative yet conversational
- **Structure:** Inverted pyramid - key findings first, details follow
- **Precision:** Reference specific figures, data points, methodology FROM THE PAPER
- **Clarity:** Define technical terms on first use
- **Flow:** Smooth transitions between concepts

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

REQUIRED ARTICLE STRUCTURE:
1. **Opening Hook** (2-3 sentences: significance and context from paper)
2. **Key Finding** (what researchers discovered - be specific)
3. **Methodology** (how they did it - brief but accurate to paper)
4. **Results Analysis** (what data shows - reference figures from paper)
5. **Context** (why it matters - based on paper's discussion)
6. **Limitations** (what remains unknown - from paper's limitations section)
7. **Figure Recommendation** (which figure best illustrates work)

EXAMPLES OF GOOD OPENING LINES (Nature/Science style):
✓ "Machine learning models can now predict protein structures with near-atomic accuracy."
✓ "A new analysis reveals the universe may be younger than previously thought."
✓ "Researchers have identified a molecular pathway in cancer cell evasion."

## PAPER TEXT (YOUR ONLY SOURCE):
{}

## AVAILABLE FIGURES (from paper):
{}

## DELIVERABLE:
Write a 500-800 word article in Nature/Science editorial style. Use ONLY information from the paper above. End with figure recommendation.

Format:
# [Compelling, Specific Title - Nature/Science Style]

[Article body - based ONLY on paper content...]

---
**Recommended Figure:** figure_X.png
**Reason:** [Why this figure best represents the work]
"#, site, site_context, paper_text, figures_list)
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
