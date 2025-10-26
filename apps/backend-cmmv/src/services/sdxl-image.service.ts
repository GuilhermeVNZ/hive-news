/**
 * SDXL Image Generator Service
 * Generates images locally using SDXL
 */

export interface ImageGenerationRequest {
  prompt: string;
  style: string;
  aspectRatio: "16:9" | "1:1" | "4:3";
  size: "cover" | "thumbnail";
  articleId: string;
}

export interface GeneratedImage {
  url: string;
  filePath: string;
  altText: string;
  dimensions: { width: number; height: number };
  fileSize: number;
}

export class SDXLImageService {
  private config: {
    baseURL: string;
    modelPath: string;
  };

  constructor(config: { baseURL: string; modelPath: string }) {
    this.config = config;
  }

  /**
   * Generate cover image (16:9)
   */
  async generateCoverImage(request: ImageGenerationRequest): Promise<GeneratedImage> {
    const enhancedPrompt = this.enhancePromptForCover(request.prompt, request.style);

    return this.generateImage({
      ...request,
      aspectRatio: "16:9",
      size: "cover",
      prompt: enhancedPrompt,
    });
  }

  /**
   * Generate thumbnail image (1:1)
   */
  async generateThumbnail(request: ImageGenerationRequest): Promise<GeneratedImage> {
    const enhancedPrompt = this.enhancePromptForThumbnail(request.prompt, request.style);

    return this.generateImage({
      ...request,
      aspectRatio: "1:1",
      size: "thumbnail",
      prompt: enhancedPrompt,
    });
  }

  /**
   * Generate OG:image for social media
   */
  async generateOGImage(request: ImageGenerationRequest): Promise<GeneratedImage> {
    return this.generateImage({
      ...request,
      aspectRatio: "16:9",
      size: "cover",
    });
  }

  /**
   * Generate image via SDXL API
   */
  private async generateImage(
    request: ImageGenerationRequest & { prompt: string }
  ): Promise<GeneratedImage> {
    const dimensions = this.getDimensions(request.aspectRatio);

    try {
      const response = await fetch(`${this.config.baseURL}/api/v1/generate`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          prompt: request.prompt,
          width: dimensions.width,
          height: dimensions.height,
          steps: 30,
          guidance: 7.5,
        }),
      });

      if (!response.ok) {
        throw new Error(`SDXL generation failed: ${response.status}`);
      }

      const data = (await response.json()) as { url?: string; image_url?: string };
      const imageUrl = data.url || data.image_url || "";
      const filePath = await this.saveImage(imageUrl, request.articleId, request.size);

      return {
        url: imageUrl,
        filePath,
        altText: this.generateAltText(request.prompt),
        dimensions,
        fileSize: 0, // Would be calculated from saved file
      };
    } catch (error) {
      console.error("SDXL generation error:", error);
      throw new Error("Failed to generate image");
    }
  }

  /**
   * Generate descriptive ALT text
   */
  generateAltText(prompt: string): string {
    // Extract key concepts from prompt
    const keywords = prompt.split(",").slice(0, 3).join(", ");
    return `Generated image: ${keywords}`.substring(0, 125);
  }

  private getDimensions(aspectRatio: string): { width: number; height: number } {
    switch (aspectRatio) {
      case "16:9":
        return { width: 1200, height: 675 };
      case "1:1":
        return { width: 400, height: 400 };
      case "4:3":
        return { width: 800, height: 600 };
      default:
        return { width: 1200, height: 675 };
    }
  }

  private enhancePromptForCover(prompt: string, style: string): string {
    return `${prompt}, ${style} style, professional, high quality, detailed, cover image`;
  }

  private enhancePromptForThumbnail(prompt: string, style: string): string {
    return `${prompt}, ${style} style, thumbnail, square format, high quality`;
  }

  private async saveImage(_imageUrl: string, articleId: string, size: string): Promise<string> {
    // Download and save to MinIO/S3
    const fileName = `${articleId}-${size}-${Date.now()}.png`;
    return `/images/${fileName}`;
  }
}
