import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Plus, Settings, ExternalLink, Globe } from 'lucide-react';
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
}

export default function Sites() {
  const [sites, setSites] = useState<Site[]>([]);
  const [selectedSite, setSelectedSite] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [promptArticle, setPromptArticle] = useState<string>('');
  const [promptSocial, setPromptSocial] = useState<string>('');
  const [promptBlog, setPromptBlog] = useState<string>('');
  const [enArticle, setEnArticle] = useState<boolean>(true);
  const [enSocial, setEnSocial] = useState<boolean>(false);
  const [enBlog, setEnBlog] = useState<boolean>(false);
  type PromptTab = 'article'|'social'|'blog';
  const [activeTab, setActiveTab] = useState<PromptTab>('article');
  const [saving, setSaving] = useState(false);
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
      const response = await axios.get('/api/sites');
      
      if (response.data.success) {
        setSites(response.data.sites || []);
        // Auto-select first site if none selected
        if (!selectedSite && response.data.sites?.length > 0) {
          setSelectedSite(response.data.sites[0].id);
        }
      } else {
        setError(response.data.error || 'Failed to load sites');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || 'Failed to load sites');
    } finally {
      setLoading(false);
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

  const handleSiteClick = (siteId: string) => {
    setSelectedSite(siteId);
    const next = sites.find(s => s.id === siteId);
    if (next) {
      // Use custom prompt if enabled, otherwise use default
      const articlePrompt = next.prompt_article_enabled 
        ? (next.prompt_article || '') 
        : getDefaultArticlePrompt(next.name);
      const socialPrompt = next.prompt_social_enabled 
        ? (next.prompt_social || '') 
        : getDefaultSocialPrompt();
      const blogPrompt = next.prompt_blog || '';
      
      setPromptArticle(articlePrompt);
      setPromptSocial(socialPrompt);
      setPromptBlog(blogPrompt);
      setEnArticle(next.prompt_article_enabled ?? true);
      setEnSocial(next.prompt_social_enabled ?? false);
      setEnBlog(next.prompt_blog_enabled ?? false);
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
        <Button variant="default" className="gap-2">
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
                <button
                  key={site.id}
                  onClick={() => handleSiteClick(site.id)}
                  className={`w-full text-left p-3 rounded-lg transition-all ${
                    selectedSite === site.id
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted hover:bg-accent'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">{site.name}</div>
                      {site.domain && (
                        <div className="text-xs opacity-70 flex items-center gap-1 mt-1">
                          <Globe size={12} />
                          {site.domain}
                        </div>
                      )}
                    </div>
                    <Badge variant={site.enabled ? 'default' : 'outline'}>
                      {site.enabled ? 'Active' : 'Inactive'}
                    </Badge>
                  </div>
                </button>
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
                    <Button variant={activeTab==='blog'?'default':'outline'} onClick={()=>setActiveTab('blog')}>Blog</Button>
                  </div>
                  {activeTab==='article' && (
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <div className="text-sm text-muted-foreground">
                          {currentSite.prompt_article_enabled ? 'Custom Prompt (Enabled)' : 'Default Prompt (Using system default)'}
                        </div>
                        {!currentSite.prompt_article_enabled && (
                          <Badge variant="secondary" className="text-xs">Using Default</Badge>
                        )}
                      </div>
                      <pre className="whitespace-pre-wrap text-sm bg-muted p-3 rounded border max-h-60 overflow-y-auto">
                        {currentSite.prompt_article_enabled 
                          ? (currentSite.prompt_article || 'No custom prompt configured')
                          : 'Default prompt from backend (edit and enable to use custom)'}
                      </pre>
                      <div className="flex items-center gap-2">
                        <input type="checkbox" checked={enArticle} onChange={(e)=>setEnArticle(e.target.checked)} />
                        <span className="text-sm">Use Custom Prompt (if disabled, uses default)</span>
                      </div>
                      <textarea 
                        className="w-full min-h-[300px] p-3 rounded-md border border-input bg-background text-sm font-mono text-xs" 
                        value={promptArticle} 
                        onChange={(e)=>setPromptArticle(e.target.value)}
                        placeholder="Enter your custom prompt here. Use {{paper_text}} placeholder to insert paper content."
                      />
                      <div className="flex gap-2 justify-end">
                        <Button variant="outline" onClick={()=>setPromptArticle(getDefaultArticlePrompt(currentSite.name))}>
                          Load Default
                        </Button>
                        <Button variant="outline" onClick={()=>setPromptArticle(currentSite.prompt_article || getDefaultArticlePrompt(currentSite.name))}>
                          Reset
                        </Button>
                        <Button disabled={saving} onClick={async ()=>{ try{ setSaving(true); await axios.put(`/api/sites/${currentSite.id}/writer`, { prompt_article: promptArticle, prompt_article_enabled: enArticle }); await loadSites(); handleSiteClick(currentSite.id); } catch(err:any){ setError(err.response?.data?.error || err.message || 'Failed to save'); } finally{ setSaving(false); } }}>
                          {saving?'Saving...':'Save'}
                        </Button>
                      </div>
                    </div>
                  )}
                  {activeTab==='social' && (
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <div className="text-sm text-muted-foreground">
                          {currentSite.prompt_social_enabled ? 'Custom Prompt (Enabled)' : 'Default Prompt (Using system default)'}
                        </div>
                        {!currentSite.prompt_social_enabled && (
                          <Badge variant="secondary" className="text-xs">Using Default</Badge>
                        )}
                      </div>
                      <pre className="whitespace-pre-wrap text-sm bg-muted p-3 rounded border max-h-60 overflow-y-auto">
                        {currentSite.prompt_social_enabled 
                          ? (currentSite.prompt_social || 'No custom prompt configured')
                          : 'Default prompt from backend (edit and enable to use custom)'}
                      </pre>
                      <div className="flex items-center gap-2">
                        <input type="checkbox" checked={enSocial} onChange={(e)=>setEnSocial(e.target.checked)} />
                        <span className="text-sm">Use Custom Prompt (if disabled, uses default)</span>
                      </div>
                      <textarea 
                        className="w-full min-h-[300px] p-3 rounded-md border border-input bg-background text-sm font-mono text-xs" 
                        value={promptSocial} 
                        onChange={(e)=>setPromptSocial(e.target.value)}
                        placeholder="Enter your custom prompt here. Use {{article_text}} and {{paper_title}} placeholders."
                      />
                      <div className="flex gap-2 justify-end">
                        <Button variant="outline" onClick={()=>setPromptSocial(getDefaultSocialPrompt())}>
                          Load Default
                        </Button>
                        <Button variant="outline" onClick={()=>setPromptSocial(currentSite.prompt_social || getDefaultSocialPrompt())}>
                          Reset
                        </Button>
                        <Button disabled={saving} onClick={async ()=>{ try{ setSaving(true); await axios.put(`/api/sites/${currentSite.id}/writer`, { prompt_social: promptSocial, prompt_social_enabled: enSocial }); await loadSites(); handleSiteClick(currentSite.id); } catch(err:any){ setError(err.response?.data?.error || err.message || 'Failed to save'); } finally{ setSaving(false); } }}>
                          {saving?'Saving...':'Save'}
                        </Button>
                      </div>
                    </div>
                  )}
                  {activeTab==='blog' && (
                    <div className="space-y-3">
                      <div className="text-sm text-muted-foreground">Current Prompt</div>
                      <pre className="whitespace-pre-wrap text-sm bg-muted p-3 rounded border">{currentSite.prompt_blog || 'No prompt configured'}</pre>
                      <div className="flex items-center gap-2">
                        <input type="checkbox" checked={enBlog} onChange={(e)=>setEnBlog(e.target.checked)} />
                        <span className="text-sm">Enabled</span>
                      </div>
                      <textarea className="w-full min-h-[150px] p-3 rounded-md border border-input bg-background text-sm" value={promptBlog} onChange={(e)=>setPromptBlog(e.target.value)} />
                      <div className="flex gap-2 justify-end">
                        <Button variant="outline" onClick={()=>setPromptBlog(currentSite.prompt_blog || '')}>Reset</Button>
                        <Button disabled={saving} onClick={async ()=>{ try{ setSaving(true); await axios.put(`/api/sites/${currentSite.id}/writer`, { prompt_blog: promptBlog, prompt_blog_enabled: enBlog }); await loadSites(); } catch(err:any){ setError(err.response?.data?.error || err.message || 'Failed to save'); } finally{ setSaving(false); } }}>{saving?'Saving...':'Save'}</Button>
                      </div>
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
    </div>
  );
}










