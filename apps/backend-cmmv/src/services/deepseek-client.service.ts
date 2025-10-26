/**
 * DeepSeek API Client Service
 * Handles AI writing and translation via DeepSeek API
 */

export interface DeepSeekConfig {
  apiKey: string;
  baseURL: string;
  model?: string;
}

export interface ArticleGenerationRequest {
  documentText: string;
  title: string;
  style: "scientific" | "tech" | "policy";
  baseLanguage: string;
  references?: string[];
}

export interface TranslationRequest {
  text: string;
  sourceLang: string;
  targetLang: string;
}

export class DeepSeekClientService {
  private config: DeepSeekConfig;
  private rateLimit: Map<string, number> = new Map();

  constructor(config: DeepSeekConfig) {
    this.config = {
      ...config,
      model: config.model || "deepseek-chat",
    };
  }

  /**
   * Generate article content using DeepSeek
   */
  async generateArticle(
    request: ArticleGenerationRequest
  ): Promise<{ body: string; dek: string; title: string }> {
    const prompt = this.buildArticlePrompt(request);

    try {
      await this.applyRateLimit();

      const response = await fetch(`${this.config.baseURL}/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${this.config.apiKey}`,
        },
        body: JSON.stringify({
          model: this.config.model,
          messages: [
            {
              role: "system",
              content: `You are an expert writer creating high-quality scientific content. Write in ${request.style} style.`,
            },
            {
              role: "user",
              content: prompt,
            },
          ],
          temperature: 0.7,
          max_tokens: 2000,
        }),
      });

      if (!response.ok) {
        throw new Error(`DeepSeek API error: ${response.status}`);
      }

      const data = (await response.json()) as { choices: Array<{ message: { content: string } }> };
      const articleText = data.choices[0].message.content;

      return {
        title: request.title,
        dek: this.extractDek(articleText),
        body: articleText,
      };
    } catch (error) {
      console.error("DeepSeek generation error:", error);
      throw new Error("Failed to generate article");
    }
  }

  /**
   * Translate text using DeepSeek
   */
  async translate(request: TranslationRequest): Promise<string> {
    try {
      await this.applyRateLimit();

      const response = await fetch(`${this.config.baseURL}/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${this.config.apiKey}`,
        },
        body: JSON.stringify({
          model: this.config.model,
          messages: [
            {
              role: "system",
              content: `Translate the following text from ${request.sourceLang} to ${request.targetLang}. Maintain the tone and style.`,
            },
            {
              role: "user",
              content: request.text,
            },
          ],
          temperature: 0.3,
        }),
      });

      if (!response.ok) {
        throw new Error(`DeepSeek translation error: ${response.status}`);
      }

      const data = (await response.json()) as { choices: Array<{ message: { content: string } }> };
      return data.choices[0].message.content;
    } catch (error) {
      console.error("DeepSeek translation error:", error);
      throw new Error("Failed to translate text");
    }
  }

  private buildArticlePrompt(request: ArticleGenerationRequest): string {
    return `
Write a comprehensive article based on this document:

TITLE: ${request.title}

DOCUMENT EXCERPT:
${request.documentText.substring(0, 1000)}

STYLE: ${request.style}
REFERENCES: ${request.references?.join(", ") || "N/A"}

Please generate:
1. A compelling title
2. A brief summary (dek) of 1-2 sentences
3. The full article body with proper structure

Ensure the content is accurate, well-structured, and engaging.
    `.trim();
  }

  private extractDek(articleText: string): string {
    const firstSentence = articleText.split(".")[0];
    return firstSentence.length > 200 ? firstSentence.substring(0, 197) + "..." : firstSentence;
  }

  private async applyRateLimit(): Promise<void> {
    const key = "deepseek";
    const lastRequest = this.rateLimit.get(key) || 0;
    const now = Date.now();

    if (now - lastRequest < 2000) {
      await new Promise((resolve) => setTimeout(resolve, 2000 - (now - lastRequest)));
    }

    this.rateLimit.set(key, Date.now());
  }
}
