import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Plus, Settings, ExternalLink, Globe, Edit, Trash2 } from 'lucide-react';
import axios from 'axios';
import { useAuth } from '../context/AuthContext';

interface Collector {
  id: string;
  name: string;
  enabled: boolean;
  api_key: string | null;
  config: any;
}

interface Writer {
  provider: string;
  model: string;
  api_key: string | null;
  base_url: string | null;
  temperature: number | null;
  max_tokens: number | null;
  enabled: boolean;
  use_compressor?: boolean | null;
}

interface SocialMedia {
  id: string;
  name: string;
  enabled: boolean;
  api_key: string | null;
  channel_id: string | null;
  username: string | null;
}

interface EducationSource {
  id: string;
  name: string;
  enabled: boolean;
  api_key: string | null;
}

interface Site {
  id: string;
  name: string;
  domain: string | null;
  enabled: boolean;
  collectors: Collector[];
  writer: Writer;
  education_sources: EducationSource[];
  social_media: SocialMedia[];
  prompt_article?: string | null;
  prompt_social?: string | null;
  prompt_blog?: string | null;
  prompt_article_enabled?: boolean | null;
  prompt_social_enabled?: boolean | null;
  prompt_blog_enabled?: boolean | null;
  temperature_article?: number | null;
  temperature_social?: number | null;
  temperature_blog?: number | null;
}

export default function Sites() {
  const [sites, setSites] = useState<Site[]>([]);
  const [selectedSite, setSelectedSite] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [promptArticle, setPromptArticle] = useState<string>(''); // Default prompt
  const [promptSocial, setPromptSocial] = useState<string>(''); // Default prompt
  const [promptBlog, setPromptBlog] = useState<string>(''); // Default prompt
  const [customArticlePrompt, setCustomArticlePrompt] = useState<string>(''); // Custom prompt
  const [customSocialPrompt, setCustomSocialPrompt] = useState<string>(''); // Custom prompt
  const [customNewsPrompt, setCustomNewsPrompt] = useState<string>(''); // Custom prompt
  const [promptModeArticle, setPromptModeArticle] = useState<'default' | 'custom'>('default'); // Which mode is active
  const [promptModeSocial, setPromptModeSocial] = useState<'default' | 'custom'>('default'); // Which mode is active
  const [promptModeNews, setPromptModeNews] = useState<'default' | 'custom'>('default'); // Which mode is active
  const [useCompressor, setUseCompressor] = useState<boolean>(false); // Compressor state
  const [temperatureArticle, setTemperatureArticle] = useState<number>(0.7); // Temperature for article prompt
  const [temperatureSocial, setTemperatureSocial] = useState<number>(0.8); // Temperature for social prompt
  const [temperatureBlog, setTemperatureBlog] = useState<number>(0.7); // Temperature for blog prompt
  type PromptTab = 'article'|'social'|'blog';
  const [activeTab, setActiveTab] = useState<PromptTab>('article');
  const [saving, setSaving] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [siteToDelete, setSiteToDelete] = useState<string | null>(null);
  const [newSite, setNewSite] = useState({
    id: '',
    name: '',
    domain: '',
    frequency_minutes: 60,
    writing_style: 'scientific',
    enabled: true,
  });
  const [editSite, setEditSite] = useState({
    id: '',
    name: '',
    domain: '',
    frequency_minutes: 60,
    writing_style: 'scientific',
    enabled: true,
  });
  const navigate = useNavigate();
  const { isAuthenticated } = useAuth();

  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login');
      return;
    }
    loadSites();
  }, [isAuthenticated, navigate]);

  const loadSites = async () => {
    try {
      setLoading(true);
      setError(''); // Clear previous errors
      const response = await axios.get('/api/sites');
      
      if (response.data.success) {
        const sitesList = response.data.sites || [];
        const previousSelectedSite = selectedSite;
        
        // Update sites list first
        setSites(sitesList);
        
        // Check if the previously selected site was deleted
        if (previousSelectedSite && !sitesList.find((s: Site) => s.id === previousSelectedSite)) {
          // Selected site was deleted, clear selection
          setSelectedSite(null);
        } else if (previousSelectedSite && sitesList.find((s: Site) => s.id === previousSelectedSite)) {
          // Selected site still exists, ensure it's still selected and reload its data
          setSelectedSite(previousSelectedSite);
          // Reload prompts for the still-selected site
          setTimeout(() => loadActualPrompts(previousSelectedSite, sitesList), 100);
        }
        
        // Auto-select first site if none selected
        if (!selectedSite && sitesList.length > 0) {
          const firstSiteId = sitesList[0].id;
          setSelectedSite(firstSiteId);
          // Load prompts for the first site - pass sitesList to avoid state dependency
          setTimeout(() => loadActualPrompts(firstSiteId, sitesList), 100);
          // Load compressor state and temperatures
          const firstSite = sitesList[0];
          if (firstSite?.writer) {
            setUseCompressor(firstSite.writer.use_compressor ?? false);
          }
          if (firstSite) {
            setTemperatureArticle(firstSite.temperature_article ?? 0.7);
            setTemperatureSocial(firstSite.temperature_social ?? 0.8);
            setTemperatureBlog(firstSite.temperature_blog ?? 0.7);
          }
        }
      } else {
        setError(response.data.error || 'Failed to load sites');
      }
    } catch (err: any) {
      const errorMessage = err.response?.data?.error || err.message || 'Failed to load sites';
      setError(errorMessage);
      console.error('Failed to load sites:', err);
      // Don't show network error if it's just the prompts endpoint (non-critical)
      if (err.code === 'ERR_NETWORK' && err.config?.url?.includes('/prompt/')) {
        // This is expected if backend hasn't been restarted yet
        console.warn('Backend may need to be restarted to support new prompt endpoints');
      }
    } finally {
      setLoading(false);
    }
  };

  // Load actual prompts from backend
  const loadActualPrompts = async (siteId: string, sitesList?: typeof sites) => {
    if (!siteId) return;
    
    // Use provided sitesList or state
    const sitesToUse = sitesList || sites;
    const next = sitesToUse.find(s => s.id === siteId);
    
    try {
      // Try to load from backend API - don't let failures block each other
      const [articleResponse, socialResponse, newsResponse] = await Promise.allSettled([
        axios.get(`/api/sites/${siteId}/prompt/article`).catch(err => ({ error: err })),
        axios.get(`/api/sites/${siteId}/prompt/social`).catch(err => ({ error: err })),
        axios.get(`/api/sites/${siteId}/prompt/news`).catch(err => ({ error: err })),
      ]);
      
      // Process article prompt
      if (articleResponse.status === 'fulfilled' && !('error' in articleResponse.value) && articleResponse.value.data?.success) {
        setPromptArticle(articleResponse.value.data.prompt || ''); // Always load default
        // Load custom prompt if it exists
        if (next?.prompt_article) {
          setCustomArticlePrompt(next.prompt_article);
        }
        // Set mode based on enabled flag
        setPromptModeArticle(next?.prompt_article_enabled ? 'custom' : 'default');
      } else if (next) {
        // Fallback
        setPromptArticle(getDefaultArticlePrompt(next.name));
        if (next.prompt_article) {
          setCustomArticlePrompt(next.prompt_article);
        }
        setPromptModeArticle(next.prompt_article_enabled ? 'custom' : 'default');
      }
      
      // Process social prompt
      if (socialResponse.status === 'fulfilled' && !('error' in socialResponse.value) && socialResponse.value.data?.success) {
        setPromptSocial(socialResponse.value.data.prompt || ''); // Always load default
        // Load custom prompt if it exists
        if (next?.prompt_social) {
          setCustomSocialPrompt(next.prompt_social);
        }
        // Set mode based on enabled flag
        setPromptModeSocial(next?.prompt_social_enabled ? 'custom' : 'default');
      } else if (next) {
        // Fallback
        setPromptSocial(getDefaultSocialPrompt());
        if (next.prompt_social) {
          setCustomSocialPrompt(next.prompt_social);
        }
        setPromptModeSocial(next.prompt_social_enabled ? 'custom' : 'default');
      }
      
      // Process news prompt
      if (newsResponse.status === 'fulfilled' && !('error' in newsResponse.value) && newsResponse.value.data?.success) {
        setPromptBlog(newsResponse.value.data.prompt || ''); // Always load default
        // Load custom prompt if it exists
        if (next?.prompt_blog) {
          setCustomNewsPrompt(next.prompt_blog);
        }
        // Set mode based on enabled flag
        setPromptModeNews(next?.prompt_blog_enabled ? 'custom' : 'default');
      } else if (next) {
        // Fallback
        setPromptBlog(getDefaultNewsPrompt());
        if (next.prompt_blog) {
          setCustomNewsPrompt(next.prompt_blog);
        }
        setPromptModeNews(next.prompt_blog_enabled ? 'custom' : 'default');
      }
      
      // Load compressor state from writer config
      if (next?.writer) {
        setUseCompressor(next.writer.use_compressor ?? false);
      }
      
      // Load temperature states
      if (next) {
        setTemperatureArticle(next.temperature_article ?? 0.7);
        setTemperatureSocial(next.temperature_social ?? 0.8);
        setTemperatureBlog(next.temperature_blog ?? 0.7);
      }
      
      // If site not found, log warning
      if (!next) {
        console.warn('Site not found for prompts:', siteId);
      }
    } catch (err: any) {
      console.error('Failed to load actual prompts:', err);
      // Final fallback
      if (next) {
        setPromptArticle(getDefaultArticlePrompt(next.name));
        setPromptSocial(getDefaultSocialPrompt());
        setPromptBlog(getDefaultNewsPrompt());
        if (next.prompt_article) setCustomArticlePrompt(next.prompt_article);
        if (next.prompt_social) setCustomSocialPrompt(next.prompt_social);
        if (next.prompt_blog) setCustomNewsPrompt(next.prompt_blog);
        setPromptModeArticle(next.prompt_article_enabled ? 'custom' : 'default');
        setPromptModeSocial(next.prompt_social_enabled ? 'custom' : 'default');
        setPromptModeNews(next.prompt_blog_enabled ? 'custom' : 'default');
        // Load compressor state and temperatures from writer config
        if (next.writer) {
          setUseCompressor(next.writer.use_compressor ?? false);
        }
        if (next) {
          setTemperatureArticle(next.temperature_article ?? 0.7);
          setTemperatureSocial(next.temperature_social ?? 0.8);
          setTemperatureBlog(next.temperature_blog ?? 0.7);
        }
      }
    }
  };

  const currentSite = sites.find(s => s.id === selectedSite);

  // Default prompts from backend (from prompts.rs)
  const getDefaultArticlePrompt = (siteName: string) => {
    const siteContext = siteName.toLowerCase() === 'airesearch' 
      ? `AIResearch is a cutting-edge AI news platform focusing on:
- Latest breakthroughs in artificial intelligence research
- Practical applications of ML/deep learning
- Industry news and expert analysis
- **News-style journalism**: Complex topics explained for general audience
- **Simple titles**: Focus on WHAT the discovery means, not HOW it works technically
- **Accessibility first**: Make readers understand WHY it matters
- **Clear explanations**: Use analogies and real-world comparisons
- Users who want technical details can read the original paper
- Emphasis on accuracy and scientific rigor WITH simple language`
      : `General scientific publication:
- Clear, accurate, accessible communication
- Emphasis on evidence-based reporting
- Professional academic tone
- Broad scientific audience`;

    return `CRITICAL INSTRUCTIONS (READ FIRST):
1. You are writing for ${siteName} in Nature/Science magazine editorial style (News & Views, Perspectives sections)
2. **NEVER FABRICATE**: Do not invent citations, references, authors, studies, or data that are not explicitly in the paper below
3. **ONLY USE PAPER CONTENT**: Reference only what exists in the provided paper text
4. NO AI clichés: "delve", "revolutionize", "game-changer", "unlock", "harness", "dive into", "shed light on"
5. NO emojis, NO excessive dashes (—), NO ellipses (...)

---

TARGET PUBLICATION:
${siteContext}

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

EXAMPLES OF GOOD OPENING LINES (News style):
✓ "A new AI method can generate fake data that captures real-world patterns so accurately that researchers can use it for sensitive analysis—without ever touching the original information."
✓ "Scientists discovered the universe might be two billion years younger than previously thought by using more precise measurements."
✓ "Cancer cells have a hidden escape route that researchers just identified, opening new doors for treatment."

## PAPER TEXT (YOUR ONLY SOURCE):
{{paper_text}}

## DELIVERABLE:
Write a 500-800 word article in Nature/Science editorial style. Use ONLY information from the paper above.

Format:
# [Compelling, Specific Title - Nature/Science Style]

[Article body - based ONLY on paper content...]

IMPORTANT: You MUST return your response as valid JSON only (no markdown, no formatting):
{
  "title": "...",                           // SHORT HOOK TITLE (max 8 words, strong clickbait)
  "article_text": "...",                    // Full article body (500-800 words)
  "image_categories": [                      // Array of image categories in priority order
    "category1", "category2", "category3"    // Based on: ai, coding, crypto, database, ethics, games, hardware, legal, network, robotics, science, security, sound
  ]
}

TITLE REQUIREMENTS (CRITICAL):
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
✓ "This AI Breakthrough May Be Wrong"`;
  };

  const getDefaultSocialPrompt = () => {
    return `CRITICAL INSTRUCTIONS (READ FIRST):
1. Create viral social media content based on Nature/Science style article below
2. **FACT-BASED ONLY**: Do not add information not in the article
3. Start each piece with a VIRAL HOOK (surprising fact, bold question, tension)
4. Match Nature/Science credibility - no clickbait lies

---

## ARTICLE (Nature/Science style - YOUR ONLY SOURCE):
{{article_text}}

## ORIGINAL PAPER TITLE:
{{paper_title}}

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
{
  "linkedin_post": "...",
  "x_post": "...",
  "shorts_script": "..."
}`;
  };

  const getDefaultNewsPrompt = () => {
    return `You are an expert technology journalist writing for a major international news portal (style: Wired, The Verge, TechCrunch).

You will receive:
- Raw cleaned content extracted from a website by a scraper (title, body text, date if available).
- Your task is to transform this into a polished news article in **native, natural English**, undetectable as AI-generated.

### 🔹 OUTPUT STRUCTURE (must follow exactly this format):

Title:
- A strong SEO-friendly headline.
- Must contain a clear keyword (AI, model, GPU, language model, etc).
- Must include a "hook" that makes the reader curious.
- Max 60 characters.

Subtitle:
- A compelling summary.
- Max 2 lines.
- Should add tension, consequence, or reason why this matters.

Article:
- 6 to 8 paragraphs.
- Clear journalistic tone, informative but engaging.
- Write like a human: vary sentence length, avoid robotic structure, add light narrative context.
- Make complex ideas simple.
- Never praise a company in a commercial tone. If the scraped text is promotional, rewrite neutrally, e.g.:
  "Grok just launched version 4.5, which claims to improve reasoning by 20%" instead of "Grok proudly revolutionizes AI with its innovative 4.5 model".

### 🔹 LANGUAGE & STYLE RULES

✔ Write in **native-level English**, clear, fluent, and natural.  
✔ Use active voice unless passive is necessary.  
✔ Keep paragraphs short for online reading (2–4 sentences).  
✔ Add context: "This follows previous updates from…", "The move comes as…", "Industry analysts suggest…"  
✔ No filler phrases like "In the ever-changing world of technology…"  
✔ No moralizing or opinions — just informative, sharp writing.

### 🔹 IMAGE CATEGORIES

You must select exactly 3 categories from this exact list ONLY:
ai, coding, crypto, database, ethics, games, hardware, legal, network, robotics, science, security, sound

CRITICAL CONSTRAINTS:
- ❌ DO NOT create new categories
- ❌ DO NOT use synonyms or variations
- ✅ ONLY use the 13 categories listed above
- ✅ Order by priority: most relevant first, second choice, third choice
- ✅ Must be lowercase, matching the list exactly

### 🔹 SOCIAL MEDIA CONTENT

You must also generate:
1. X (Twitter) post - 280 characters max, engaging hook
2. LinkedIn post - Professional tone, 300 characters max
3. TikTok Shorts script - 2 minutes (~300 words), max 5 seconds per take/frase

TikTok Script Format:
- Each take/frase should be exactly 5 seconds or less
- Include visual directions when needed
- Conversational, engaging, hook-driven

### 🔹 OUTPUT FORMAT (JSON):

{
  "title": "...",                           // Max 60 characters, SEO-friendly, hook
  "subtitle": "...",                        // Max 2 lines, compelling summary
  "article_text": "...",                    // 6-8 paragraphs, journalistic tone
  "image_categories": [                     // Top 3 categories from exact list
    "category1", "category2", "category3"
  ],
  "x_post": "...",                          // Twitter/X post, 280 chars max
  "linkedin_post": "...",                   // LinkedIn post, 300 chars max
  "shorts_script": "..."                    // TikTok 2min script, 5sec per take
}`;
  };

  const handleSiteClick = (siteId: string) => {
    setSelectedSite(siteId);
    const next = sites.find(s => s.id === siteId);
    if (next) {
      // Load compressor state and temperatures from writer config
      if (next.writer) {
        setUseCompressor(next.writer.use_compressor ?? false);
      }
      if (next) {
        setTemperatureArticle(next.temperature_article ?? 0.7);
        setTemperatureSocial(next.temperature_social ?? 0.8);
        setTemperatureBlog(next.temperature_blog ?? 0.7);
      }
      // Load actual prompts from backend (the real prompts that will be sent to API)
      // Pass sites array to avoid state dependency issues
      loadActualPrompts(siteId, sites).catch(err => {
        console.error('Error loading prompts:', err);
      });
    }
  };

  const handleAddSite = () => {
    setNewSite({
      id: '',
      name: '',
      domain: '',
      frequency_minutes: 60,
      writing_style: 'scientific',
      enabled: true,
    });
    setDialogOpen(true);
  };

  const handleCreateSite = async () => {
    try {
      if (!newSite.id.trim() || !newSite.name.trim()) {
        setError('Site ID and Name are required');
        return;
      }

      setSaving(true);
      setError('');
      
      const response = await axios.post('/api/pages', {
        id: newSite.id.trim().toLowerCase().replace(/\s+/g, '_'),
        name: newSite.name.trim(),
        domain: newSite.domain.trim() || null,
        frequency_minutes: newSite.frequency_minutes,
        writing_style: newSite.writing_style,
        enabled: newSite.enabled,
      });

      if (response.data.success) {
        setDialogOpen(false);
        await loadSites();
        // Auto-select the newly created site
        if (response.data.message) {
          setSelectedSite(newSite.id.trim().toLowerCase().replace(/\s+/g, '_'));
        }
      } else {
        setError(response.data.error || 'Failed to create site');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to create site');
    } finally {
      setSaving(false);
    }
  };

  const handleEditSite = async () => {
    try {
      if (!editSite.name.trim()) {
        setError('Site name is required');
        return;
      }

      setSaving(true);
      setError('');
      
      const response = await axios.put(`/api/pages/${editSite.id}`, {
        name: editSite.name.trim(),
        domain: editSite.domain.trim() || null,
        frequency_minutes: editSite.frequency_minutes,
        writing_style: editSite.writing_style,
        enabled: editSite.enabled,
      });

      if (response.data.success) {
        setEditDialogOpen(false);
        await loadSites();
        // Reload current site if it was edited
        if (selectedSite === editSite.id) {
          await handleSiteClick(editSite.id);
        }
      } else {
        setError(response.data.error || 'Failed to update site');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to update site');
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteSite = async () => {
    if (!siteToDelete) return;
    
    try {
      setSaving(true);
      setError('');
      
      // Ensure siteToDelete is properly encoded (only if needed for special characters)
      // For most IDs, we can send directly, but encode if it contains special characters
      const siteId = siteToDelete;
      const url = `/api/pages/${siteId}`;
      console.log('[DELETE SITE] Attempting to delete:', siteToDelete);
      console.log('[DELETE SITE] URL:', url);
      
      const response = await axios.delete(url, {
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (response.data.success) {
        // Close dialog first to provide immediate feedback
        setDeleteDialogOpen(false);
        
        // Clear selection if deleted site was selected (currentSite is derived from selectedSite)
        // Don't set currentSite - it's a computed value from selectedSite
        if (selectedSite === siteToDelete) {
          setSelectedSite(null);
        }
        
        // Clear site to delete
        setSiteToDelete(null);
        
        // Clear any error messages
        setError('');
        
        // Reload sites list - this will automatically update the UI
        // loadSites will handle clearing selection and auto-selecting first site if needed
        await loadSites();
      } else {
        setError(response.data.error || 'Failed to delete site');
      }
    } catch (err: any) {
      console.error('[DELETE SITE] Error:', err);
      const errorMsg = err.response?.data?.error || err.response?.statusText || err.message || 'Failed to delete site';
      setError(`Error ${err.response?.status || 'unknown'}: ${errorMsg}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Sites</h1>
          <p className="text-muted-foreground mt-2">Edit the prompt template sent to the Writer per site</p>
        </div>
        <Button variant="default" className="gap-2" onClick={handleAddSite}>
          <Plus size={20} />
          Add Site
        </Button>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Sidebar - List of Sites */}
        <div className="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle>Sites</CardTitle>
              <CardDescription>Select a site to configure</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {sites.map((site) => (
                <div
                  key={site.id}
                  className={`w-full p-3 rounded-lg transition-all ${
                    selectedSite === site.id
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted hover:bg-accent'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <button
                      onClick={() => handleSiteClick(site.id)}
                      className="flex-1 text-left"
                    >
                      <div>
                        <div className="font-medium">{site.name}</div>
                        {site.domain && (
                          <div className={`text-xs opacity-70 flex items-center gap-1 mt-1 ${selectedSite === site.id ? 'text-primary-foreground' : ''}`}>
                            <Globe size={12} />
                            {site.domain}
                          </div>
                        )}
                      </div>
                    </button>
                    <div className="flex items-center gap-2 ml-2">
                      <Badge 
                        variant={site.enabled ? 'default' : 'outline'}
                        title={site.enabled ? 'Site is enabled and will collect/write articles' : 'Site is disabled and will not collect/write articles'}
                        className="cursor-help"
                      >
                        {site.enabled ? 'Active' : 'Inactive'}
                      </Badge>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditSite({
                            id: site.id,
                            name: site.name,
                            domain: site.domain || '',
                            frequency_minutes: 60, // Default value
                            writing_style: 'scientific', // Default value
                            enabled: site.enabled,
                          });
                          setEditDialogOpen(true);
                        }}
                        className="h-8 w-8 p-0"
                      >
                        <Edit size={16} />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          setSiteToDelete(site.id);
                          setDeleteDialogOpen(true);
                        }}
                        className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                      >
                        <Trash2 size={16} />
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>

        {/* Main Content - Prompt Editor */}
        <div className="lg:col-span-3">
          {currentSite ? (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>{currentSite.name}</CardTitle>
                      <CardDescription>
                        {currentSite.domain || 'No domain configured'}
                      </CardDescription>
                    </div>
                    <Badge variant={currentSite.enabled ? 'default' : 'outline'}>
                      {currentSite.enabled ? 'Active' : 'Inactive'}
                    </Badge>
                  </div>
                </CardHeader>
              </Card>
              {/* Prompt Tabs */}
              <Card>
                <CardHeader>
                  <CardTitle>Prompts</CardTitle>
                  <CardDescription>Select which prompt to view/edit</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex gap-2">
                    <Button variant={activeTab==='article'?'default':'outline'} onClick={()=>setActiveTab('article')}>Article</Button>
                    <Button variant={activeTab==='social'?'default':'outline'} onClick={()=>setActiveTab('social')}>Social</Button>
                    <Button variant={activeTab==='blog'?'default':'outline'} onClick={()=>setActiveTab('blog')}>News</Button>
                  </div>
                  {activeTab==='article' && (
                    <div className="space-y-4">
                      {/* Prompt Type Selector */}
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                        <Label className="text-sm font-semibold">Select Active Prompt:</Label>
                          <div className="flex items-center gap-4">
                            <label className="flex items-center gap-2 cursor-pointer">
                              <input 
                                type="checkbox" 
                                checked={useCompressor} 
                                onChange={async (e) => {
                                  const newValue = e.target.checked;
                                  setUseCompressor(newValue);
                                  try {
                                    setSaving(true);
                                    await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                      use_compressor: newValue
                                    }); 
                                    await loadSites();
                                  } catch(err: any) {
                                    setError(err.response?.data?.error || err.message || 'Failed to update compressor');
                                    // Revert on error
                                    setUseCompressor(!newValue);
                                  } finally {
                                    setSaving(false);
                                  }
                                }}
                                className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                                disabled={saving}
                              />
                              <span className="text-sm">Use Compressor</span>
                            </label>
                            <div className="flex items-center gap-2">
                              <Label className="text-sm">Temperature:</Label>
                              <input 
                                type="number" 
                                step="0.1" 
                                min="0" 
                                max="2" 
                                value={temperatureArticle} 
                                onChange={async (e) => {
                                  const newValue = parseFloat(e.target.value) || 0.7;
                                  setTemperatureArticle(newValue);
                                  try {
                                    setSaving(true);
                                    await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                      temperature_article: newValue
                                    }); 
                                    await loadSites();
                                  } catch(err: any) {
                                    setError(err.response?.data?.error || err.message || 'Failed to update temperature');
                                    // Revert on error
                                    setTemperatureArticle(currentSite.temperature_article ?? 0.7);
                                  } finally {
                                    setSaving(false);
                                  }
                                }}
                                className="w-20 px-2 py-1 rounded border border-input text-sm"
                                disabled={saving}
                              />
                            </div>
                          </div>
                        </div>
                        <div className="flex gap-4">
                          <label className="flex items-center gap-2 cursor-pointer">
                            <input 
                              type="radio" 
                              name="prompt-mode-article" 
                              checked={promptModeArticle === 'default'} 
                              onChange={async ()=>{
                                setPromptModeArticle('default');
                                // Auto-save: switch to default
                                try {
                                  setSaving(true);
                                  await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                    prompt_article_enabled: false 
                                  }); 
                                  await loadSites(); 
                                  await loadActualPrompts(currentSite.id, sites);
                                } catch(err:any) {
                                  setError(err.response?.data?.error || err.message || 'Failed to switch to default');
                                  // Revert on error
                                  setPromptModeArticle('custom');
                                } finally {
                                  setSaving(false);
                                }
                              }}
                              className="cursor-pointer"
                              disabled={saving}
                            />
                            <span className="text-sm">Default Prompt</span>
                            {promptModeArticle === 'default' && (
                              <Badge variant="secondary" className="text-xs">Currently Active</Badge>
                            )}
                          </label>
                          <label className="flex items-center gap-2 cursor-pointer">
                            <input 
                              type="radio" 
                              name="prompt-mode-article" 
                              checked={promptModeArticle === 'custom'} 
                              onChange={async ()=>{
                                setPromptModeArticle('custom');
                                // Auto-save: switch to custom (ensure custom prompt exists)
                                try {
                                  setSaving(true);
                                  // Use existing custom prompt or current custom text
                                  const promptToSave = customArticlePrompt || currentSite.prompt_article || '';
                                  await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                    prompt_article: promptToSave,
                                    prompt_article_enabled: true 
                                  }); 
                                  await loadSites(); 
                                  await loadActualPrompts(currentSite.id, sites);
                                } catch(err:any) {
                                  setError(err.response?.data?.error || err.message || 'Failed to switch to custom');
                                  // Revert on error
                                  setPromptModeArticle('default');
                                } finally {
                                  setSaving(false);
                                }
                              }}
                              className="cursor-pointer"
                              disabled={saving}
                            />
                            <span className="text-sm">Custom Prompt</span>
                            {promptModeArticle === 'custom' && (
                              <Badge variant="default" className="text-xs">Currently Active</Badge>
                            )}
                          </label>
                        </div>
                      </div>

                      {/* Default Prompt (Read-only) */}
                      {promptModeArticle === 'default' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-sm font-semibold">Default Prompt (Read-only - Safe Reference):</Label>
                            <Badge variant="secondary" className="text-xs">Active</Badge>
                          </div>
                          <div className="p-3 bg-muted rounded-md text-sm font-mono whitespace-pre-wrap max-h-[600px] overflow-y-auto border border-input">
                            {promptArticle || 'Loading default prompt...'}
                          </div>
                          <p className="text-xs text-muted-foreground">
                            This is the fixed default prompt sent to DeepSeek API. It cannot be edited and serves as a safe reference.
                          </p>
                        </div>
                      )}

                      {/* Custom Prompt (Editable) */}
                      {promptModeArticle === 'custom' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-sm font-semibold">Custom Prompt (Editable):</Label>
                            <Badge variant="default" className="text-xs">Active</Badge>
                          </div>
                          <textarea 
                            className="w-full min-h-[600px] p-3 rounded-md border border-input bg-background text-sm font-mono text-xs" 
                            value={customArticlePrompt} 
                            onChange={(e)=>setCustomArticlePrompt(e.target.value)}
                            placeholder="Enter your custom prompt here. Use {{paper_text}} placeholder for paper content."
                          />
                          <p className="text-xs text-muted-foreground">
                            Edit your custom prompt freely. Use {'{{paper_text}}'} to reference the paper content when this prompt is sent to the API.
                          </p>
                          <div className="flex gap-2 justify-end">
                            <Button variant="outline" onClick={()=>{
                              // Reset to saved custom prompt or empty
                              setCustomArticlePrompt(currentSite.prompt_article || '');
                            }}>
                              Reset Changes
                            </Button>
                            <Button disabled={saving} onClick={async ()=>{ 
                              try{ 
                                setSaving(true); 
                                // Only save the prompt text, don't change enabled flag (already set by radio button)
                                await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                  prompt_article: customArticlePrompt
                                  // Don't send prompt_article_enabled - keep current state
                                }); 
                                await loadSites(); 
                                await loadActualPrompts(currentSite.id, sites);
                              } catch(err:any){ 
                                setError(err.response?.data?.error || err.message || 'Failed to save'); 
                              } finally{ 
                                setSaving(false); 
                              } 
                            }}>
                              {saving?'Saving...':'Save Custom Prompt'}
                            </Button>
                          </div>
                        </div>
                      )}

                    </div>
                  )}
                  {activeTab==='social' && (
                    <div className="space-y-4">
                      {/* Prompt Type Selector */}
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                        <Label className="text-sm font-semibold">Select Active Prompt:</Label>
                          <div className="flex items-center gap-4">
                            <label className="flex items-center gap-2 cursor-pointer">
                              <input 
                                type="checkbox" 
                                checked={useCompressor} 
                                onChange={async (e) => {
                                  const newValue = e.target.checked;
                                  setUseCompressor(newValue);
                                  try {
                                    setSaving(true);
                                    await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                      use_compressor: newValue
                                    }); 
                                    await loadSites();
                                  } catch(err: any) {
                                    setError(err.response?.data?.error || err.message || 'Failed to update compressor');
                                    // Revert on error
                                    setUseCompressor(!newValue);
                                  } finally {
                                    setSaving(false);
                                  }
                                }}
                                className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                                disabled={saving}
                              />
                              <span className="text-sm">Use Compressor</span>
                            </label>
                            <div className="flex items-center gap-2">
                              <Label className="text-sm">Temperature:</Label>
                              <input 
                                type="number" 
                                step="0.1" 
                                min="0" 
                                max="2" 
                                value={temperatureSocial} 
                                onChange={async (e) => {
                                  const newValue = parseFloat(e.target.value) || 0.8;
                                  setTemperatureSocial(newValue);
                                  try {
                                    setSaving(true);
                                    await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                      temperature_social: newValue
                                    }); 
                                    await loadSites();
                                  } catch(err: any) {
                                    setError(err.response?.data?.error || err.message || 'Failed to update temperature');
                                    // Revert on error
                                    setTemperatureSocial(currentSite.temperature_social ?? 0.8);
                                  } finally {
                                    setSaving(false);
                                  }
                                }}
                                className="w-20 px-2 py-1 rounded border border-input text-sm"
                                disabled={saving}
                              />
                            </div>
                          </div>
                        </div>
                        <div className="flex gap-4">
                          <label className="flex items-center gap-2 cursor-pointer">
                            <input 
                              type="radio" 
                              name="prompt-mode-social" 
                              checked={promptModeSocial === 'default'} 
                              onChange={async ()=>{
                                setPromptModeSocial('default');
                                try {
                                  setSaving(true);
                                  await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                    prompt_social_enabled: false 
                                  }); 
                                  await loadSites(); 
                                  await loadActualPrompts(currentSite.id, sites);
                                } catch(err:any) {
                                  setError(err.response?.data?.error || err.message || 'Failed to switch to default');
                                  setPromptModeSocial('custom');
                                } finally {
                                  setSaving(false);
                                }
                              }}
                              className="cursor-pointer"
                              disabled={saving}
                            />
                            <span className="text-sm">Default Prompt</span>
                            {promptModeSocial === 'default' && (
                              <Badge variant="secondary" className="text-xs">Currently Active</Badge>
                            )}
                          </label>
                          <label className="flex items-center gap-2 cursor-pointer">
                            <input 
                              type="radio" 
                              name="prompt-mode-social" 
                              checked={promptModeSocial === 'custom'} 
                              onChange={async ()=>{
                                setPromptModeSocial('custom');
                                try {
                                  setSaving(true);
                                  const promptToSave = customSocialPrompt || currentSite.prompt_social || '';
                                  await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                    prompt_social: promptToSave,
                                    prompt_social_enabled: true 
                                  }); 
                                  await loadSites(); 
                                  await loadActualPrompts(currentSite.id, sites);
                                } catch(err:any) {
                                  setError(err.response?.data?.error || err.message || 'Failed to switch to custom');
                                  setPromptModeSocial('default');
                                } finally {
                                  setSaving(false);
                                }
                              }}
                              className="cursor-pointer"
                              disabled={saving}
                            />
                            <span className="text-sm">Custom Prompt</span>
                            {promptModeSocial === 'custom' && (
                              <Badge variant="default" className="text-xs">Currently Active</Badge>
                            )}
                          </label>
                        </div>
                      </div>

                      {/* Default Prompt (Read-only) */}
                      {promptModeSocial === 'default' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-sm font-semibold">Default Prompt (Read-only - Safe Reference):</Label>
                            <Badge variant="secondary" className="text-xs">Active</Badge>
                          </div>
                          <div className="p-3 bg-muted rounded-md text-sm font-mono whitespace-pre-wrap max-h-[600px] overflow-y-auto border border-input">
                            {promptSocial || 'Loading default prompt...'}
                          </div>
                          <p className="text-xs text-muted-foreground">
                            This is the fixed default prompt sent to DeepSeek API. It cannot be edited and serves as a safe reference.
                          </p>
                        </div>
                      )}

                      {/* Custom Prompt (Editable) */}
                      {promptModeSocial === 'custom' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-sm font-semibold">Custom Prompt (Editable):</Label>
                            <Badge variant="default" className="text-xs">Active</Badge>
                          </div>
                          <textarea 
                            className="w-full min-h-[600px] p-3 rounded-md border border-input bg-background text-sm font-mono text-xs" 
                            value={customSocialPrompt} 
                            onChange={(e)=>setCustomSocialPrompt(e.target.value)}
                            placeholder="Enter your custom prompt here. Use {{article_text}} and {{paper_title}} placeholders."
                          />
                          <p className="text-xs text-muted-foreground">
                            Edit your custom prompt freely. Use {'{{article_text}}'} and {'{{paper_title}}'} to reference content when this prompt is sent to the API.
                          </p>
                          <div className="flex gap-2 justify-end">
                            <Button variant="outline" onClick={()=>{
                              setCustomSocialPrompt(currentSite.prompt_social || '');
                            }}>
                              Reset Changes
                            </Button>
                            <Button disabled={saving} onClick={async ()=>{ 
                              try{ 
                                setSaving(true); 
                                // Only save the prompt text, don't change enabled flag
                                await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                  prompt_social: customSocialPrompt
                                  // Don't send prompt_social_enabled - keep current state
                                }); 
                                await loadSites(); 
                                await loadActualPrompts(currentSite.id, sites); 
                              } catch(err:any){ 
                                setError(err.response?.data?.error || err.message || 'Failed to save'); 
                              } finally{ 
                                setSaving(false); 
                              } 
                            }}>
                              {saving?'Saving...':'Save Custom Prompt'}
                            </Button>
                          </div>
                        </div>
                      )}

                    </div>
                  )}
                  {activeTab==='blog' && (
                    <div className="space-y-4">
                      {/* Prompt Type Selector */}
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                        <Label className="text-sm font-semibold">Select Active Prompt:</Label>
                          <div className="flex items-center gap-4">
                            <label className="flex items-center gap-2 cursor-pointer">
                              <input 
                                type="checkbox" 
                                checked={useCompressor} 
                                onChange={async (e) => {
                                  const newValue = e.target.checked;
                                  setUseCompressor(newValue);
                                  try {
                                    setSaving(true);
                                    await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                      use_compressor: newValue
                                    }); 
                                    await loadSites();
                                  } catch(err: any) {
                                    setError(err.response?.data?.error || err.message || 'Failed to update compressor');
                                    // Revert on error
                                    setUseCompressor(!newValue);
                                  } finally {
                                    setSaving(false);
                                  }
                                }}
                                className="rounded border-gray-300 w-4 h-4 accent-primary cursor-pointer"
                                disabled={saving}
                              />
                              <span className="text-sm">Use Compressor</span>
                            </label>
                            <div className="flex items-center gap-2">
                              <Label className="text-sm">Temperature:</Label>
                              <input 
                                type="number" 
                                step="0.1" 
                                min="0" 
                                max="2" 
                                value={temperatureBlog} 
                                onChange={async (e) => {
                                  const newValue = parseFloat(e.target.value) || 0.7;
                                  setTemperatureBlog(newValue);
                                  try {
                                    setSaving(true);
                                    await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                      temperature_blog: newValue
                                    }); 
                                    await loadSites();
                                  } catch(err: any) {
                                    setError(err.response?.data?.error || err.message || 'Failed to update temperature');
                                    // Revert on error
                                    setTemperatureBlog(currentSite.temperature_blog ?? 0.7);
                                  } finally {
                                    setSaving(false);
                                  }
                                }}
                                className="w-20 px-2 py-1 rounded border border-input text-sm"
                                disabled={saving}
                              />
                            </div>
                          </div>
                        </div>
                        <div className="flex gap-4">
                          <label className="flex items-center gap-2 cursor-pointer">
                            <input 
                              type="radio" 
                              name="prompt-mode-news" 
                              checked={promptModeNews === 'default'} 
                              onChange={async ()=>{
                                setPromptModeNews('default');
                                try {
                                  setSaving(true);
                                  await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                    prompt_blog_enabled: false 
                                  }); 
                                  await loadSites(); 
                                  await loadActualPrompts(currentSite.id, sites);
                                } catch(err:any) {
                                  setError(err.response?.data?.error || err.message || 'Failed to switch to default');
                                  setPromptModeNews('custom');
                                } finally {
                                  setSaving(false);
                                }
                              }}
                              className="cursor-pointer"
                              disabled={saving}
                            />
                            <span className="text-sm">Default Prompt</span>
                            {promptModeNews === 'default' && (
                              <Badge variant="secondary" className="text-xs">Currently Active</Badge>
                            )}
                          </label>
                          <label className="flex items-center gap-2 cursor-pointer">
                            <input 
                              type="radio" 
                              name="prompt-mode-news" 
                              checked={promptModeNews === 'custom'} 
                              onChange={async ()=>{
                                setPromptModeNews('custom');
                                try {
                                  setSaving(true);
                                  const promptToSave = customNewsPrompt || currentSite.prompt_blog || '';
                                  await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                    prompt_blog: promptToSave,
                                    prompt_blog_enabled: true 
                                  }); 
                                  await loadSites(); 
                                  await loadActualPrompts(currentSite.id, sites);
                                } catch(err:any) {
                                  setError(err.response?.data?.error || err.message || 'Failed to switch to custom');
                                  setPromptModeNews('default');
                                } finally {
                                  setSaving(false);
                                }
                              }}
                              className="cursor-pointer"
                              disabled={saving}
                            />
                            <span className="text-sm">Custom Prompt</span>
                            {promptModeNews === 'custom' && (
                              <Badge variant="default" className="text-xs">Currently Active</Badge>
                            )}
                          </label>
                        </div>
                      </div>

                      {/* Default Prompt (Read-only) */}
                      {promptModeNews === 'default' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-sm font-semibold">Default Prompt (Read-only - Safe Reference):</Label>
                            <Badge variant="secondary" className="text-xs">Active</Badge>
                          </div>
                          <div className="p-3 bg-muted rounded-md text-sm font-mono whitespace-pre-wrap max-h-[600px] overflow-y-auto border border-input">
                            {promptBlog || 'Loading default prompt...'}
                          </div>
                          <p className="text-xs text-muted-foreground">
                            This is the fixed default prompt sent to DeepSeek API. It cannot be edited and serves as a safe reference.
                          </p>
                        </div>
                      )}

                      {/* Custom Prompt (Editable) */}
                      {promptModeNews === 'custom' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-sm font-semibold">Custom Prompt (Editable):</Label>
                            <Badge variant="default" className="text-xs">Active</Badge>
                          </div>
                          <textarea 
                            className="w-full min-h-[600px] p-3 rounded-md border border-input bg-background text-sm font-mono text-xs" 
                            value={customNewsPrompt} 
                            onChange={(e)=>setCustomNewsPrompt(e.target.value)}
                            placeholder="Enter your custom prompt here. Use {{article_json}} placeholder for article content."
                          />
                          <p className="text-xs text-muted-foreground">
                            Edit your custom prompt freely. Use {'{{article_json}}'} to reference the article content when this prompt is sent to the API.
                          </p>
                          <div className="flex gap-2 justify-end">
                            <Button variant="outline" onClick={()=>{
                              setCustomNewsPrompt(currentSite.prompt_blog || '');
                            }}>
                              Reset Changes
                            </Button>
                            <Button disabled={saving} onClick={async ()=>{ 
                              try{ 
                                setSaving(true); 
                                // Only save the prompt text, don't change enabled flag
                                await axios.put(`/api/sites/${currentSite.id}/writer`, { 
                                  prompt_blog: customNewsPrompt
                                  // Don't send prompt_blog_enabled - keep current state
                                }); 
                                await loadSites(); 
                                await loadActualPrompts(currentSite.id, sites); 
                              } catch(err:any){ 
                                setError(err.response?.data?.error || err.message || 'Failed to save'); 
                              } finally{ 
                                setSaving(false); 
                              } 
                            }}>
                              {saving?'Saving...':'Save Custom Prompt'}
                            </Button>
                          </div>
                        </div>
                      )}

                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          ) : (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                <p>No site selected</p>
                <p className="text-sm mt-2">Select a site from the sidebar to configure it</p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      {/* Add Site Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent onClose={() => setDialogOpen(false)} className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Add New Site</DialogTitle>
            <DialogDescription>
              Create a new site to configure its prompts and settings
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="site-id">Site ID *</Label>
              <Input
                id="site-id"
                placeholder="e.g., mysite"
                value={newSite.id}
                onChange={(e) => setNewSite({ ...newSite, id: e.target.value })}
              />
              <p className="text-xs text-muted-foreground">
                Unique identifier (lowercase, no spaces) - will be converted to lowercase with underscores
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="site-name">Site Name *</Label>
              <Input
                id="site-name"
                placeholder="e.g., My Site"
                value={newSite.name}
                onChange={(e) => setNewSite({ ...newSite, name: e.target.value })}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="site-domain">Domain</Label>
              <Input
                id="site-domain"
                placeholder="e.g., mysite.news"
                value={newSite.domain}
                onChange={(e) => setNewSite({ ...newSite, domain: e.target.value })}
              />
              <p className="text-xs text-muted-foreground">
                Optional - website domain for this site
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="site-frequency">Collection Frequency (minutes)</Label>
              <Input
                id="site-frequency"
                type="number"
                min="1"
                value={newSite.frequency_minutes}
                onChange={(e) => setNewSite({ ...newSite, frequency_minutes: parseInt(e.target.value) || 60 })}
              />
              <p className="text-xs text-muted-foreground">
                How often to collect articles (default: 60 minutes)
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="site-writing-style">Writing Style</Label>
              <select
                id="site-writing-style"
                className="w-full h-9 px-3 rounded-md border border-input bg-background text-sm"
                value={newSite.writing_style}
                onChange={(e) => setNewSite({ ...newSite, writing_style: e.target.value })}
              >
                <option value="scientific">Scientific</option>
                <option value="technical">Technical</option>
                <option value="general">General</option>
                <option value="news">News</option>
              </select>
            </div>

            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="site-enabled"
                checked={newSite.enabled}
                onChange={(e) => setNewSite({ ...newSite, enabled: e.target.checked })}
                className="rounded border-gray-300 w-4 h-4 accent-primary"
              />
              <Label htmlFor="site-enabled" className="cursor-pointer">
                Enable site immediately
              </Label>
            </div>
          </div>

          {error && (
            <div className="mt-4 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
              {error}
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)} disabled={saving}>
              Cancel
            </Button>
            <Button onClick={handleCreateSite} disabled={saving || !newSite.id.trim() || !newSite.name.trim()}>
              {saving ? 'Creating...' : 'Create Site'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Site Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent onClose={() => setEditDialogOpen(false)} className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Edit Site</DialogTitle>
            <DialogDescription>
              Update site information and settings
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-site-name">Site Name *</Label>
              <Input
                id="edit-site-name"
                placeholder="e.g., My Site"
                value={editSite.name}
                onChange={(e) => setEditSite({ ...editSite, name: e.target.value })}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="edit-site-domain">Domain</Label>
              <Input
                id="edit-site-domain"
                placeholder="e.g., mysite.news"
                value={editSite.domain}
                onChange={(e) => setEditSite({ ...editSite, domain: e.target.value })}
              />
              <p className="text-xs text-muted-foreground">
                Optional - website domain for this site
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="edit-site-frequency">Collection Frequency (minutes)</Label>
              <Input
                id="edit-site-frequency"
                type="number"
                min="1"
                value={editSite.frequency_minutes}
                onChange={(e) => setEditSite({ ...editSite, frequency_minutes: parseInt(e.target.value) || 60 })}
              />
              <p className="text-xs text-muted-foreground">
                How often to collect articles (default: 60 minutes)
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="edit-site-writing-style">Writing Style</Label>
              <select
                id="edit-site-writing-style"
                className="w-full h-9 px-3 rounded-md border border-input bg-background text-sm"
                value={editSite.writing_style}
                onChange={(e) => setEditSite({ ...editSite, writing_style: e.target.value })}
              >
                <option value="scientific">Scientific</option>
                <option value="technical">Technical</option>
                <option value="general">General</option>
                <option value="news">News</option>
              </select>
            </div>

            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="edit-site-enabled"
                checked={editSite.enabled}
                onChange={(e) => setEditSite({ ...editSite, enabled: e.target.checked })}
                className="rounded border-gray-300 w-4 h-4 accent-primary"
              />
              <Label htmlFor="edit-site-enabled" className="cursor-pointer">
                Enable site
              </Label>
            </div>
          </div>

          {error && (
            <div className="mt-4 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
              {error}
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)} disabled={saving}>
              Cancel
            </Button>
            <Button onClick={handleEditSite} disabled={saving || !editSite.name.trim()}>
              {saving ? 'Saving...' : 'Save Changes'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Site Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent onClose={() => setDeleteDialogOpen(false)} className="max-w-md">
          <DialogHeader>
            <DialogTitle>Delete Site</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this site? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>

          {siteToDelete && (
            <div className="py-4">
              <p className="text-sm text-muted-foreground">
                Site: <span className="font-medium text-foreground">{sites.find(s => s.id === siteToDelete)?.name || siteToDelete}</span>
              </p>
              <p className="text-sm text-destructive mt-2">
                All site configuration, prompts, and settings will be permanently deleted.
              </p>
            </div>
          )}

          {error && (
            <div className="mt-4 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
              {error}
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)} disabled={saving}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDeleteSite} disabled={saving || !siteToDelete}>
              {saving ? 'Deleting...' : 'Delete Site'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}










