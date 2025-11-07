/**
 * Dense text-to-image rendering for vision model consumption.
 *
 * Inspired by DeepSeek-OCR's optical context compression approach, this module
 * renders compressed text into fixed-size 1024x1024 monospace images optimized
 * for vision model processing.
 */

import { createCanvas, CanvasRenderingContext2D } from '@napi-rs/canvas';

/**
 * Errors that can occur during image rendering.
 */
export class ImageError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ImageError';
  }
}

export class TextTooLargeError extends ImageError {
  constructor(
    public requiredLines: number,
    public maxLines: number
  ) {
    super(`Text too large to fit in image (requires ${requiredLines} lines, max ${maxLines})`);
    this.name = 'TextTooLargeError';
  }
}

/**
 * Configuration for image rendering.
 */
export interface ImageRendererConfig {
  /** Image width in pixels (default: 1024) */
  width: number;

  /** Image height in pixels (default: 1024) */
  height: number;

  /** Base font size for monospace text (default: 12.5) */
  fontSize: number;

  /** Line spacing multiplier (default: 1.2 for comfortable reading) */
  lineSpacing: number;

  /** Horizontal margin in pixels (default: 20) */
  marginX: number;

  /** Vertical margin in pixels (default: 20) */
  marginY: number;

  /** Background color RGB (default: white [255, 255, 255]) */
  bgColor: [number, number, number];

  /** Text color RGB (default: black [0, 0, 0]) */
  textColor: [number, number, number];

  /** Minimum font size when auto-scaling (default: 7.0) */
  minFontSize: number;

  /** Font family for text rendering (default: 'monospace') */
  fontFamily: string;
}

/**
 * Default configuration for image rendering.
 */
export const DEFAULT_IMAGE_RENDERER_CONFIG: ImageRendererConfig = {
  width: 1024,
  height: 1024,
  fontSize: 12.5,
  lineSpacing: 1.2,
  marginX: 20,
  marginY: 20,
  bgColor: [255, 255, 255],
  textColor: [0, 0, 0],
  minFontSize: 7.0,
  fontFamily: 'monospace',
};

/**
 * Image format for output.
 */
export enum ImageFormat {
  PNG = 'png',
  JPEG = 'jpeg',
}

/**
 * Image renderer for converting text to PNG/JPEG images.
 */
export class ImageRenderer {
  public config: ImageRendererConfig;

  constructor(config?: Partial<ImageRendererConfig>) {
    this.config = {
      ...DEFAULT_IMAGE_RENDERER_CONFIG,
      ...config,
    };
  }

  /**
   * Render text to PNG image bytes.
   *
   * This method:
   * 1. Calculates optimal font size to fit text in image
   * 2. Wraps text into lines
   * 3. Renders each line with monospace font
   * 4. Encodes final image as PNG
   *
   * @param text - The text to render
   * @returns PNG image data as Buffer
   * @throws ImageError if rendering fails
   */
  renderToPng(text: string): Buffer {
    return this.renderToFormat(text, ImageFormat.PNG, 100);
  }

  /**
   * Render text to JPEG image bytes.
   *
   * @param text - The text to render
   * @param quality - JPEG quality (1-100, higher = better quality but larger file)
   * @returns JPEG image data as Buffer
   * @throws ImageError if rendering fails
   */
  renderToJpeg(text: string, quality: number = 85): Buffer {
    if (quality < 1 || quality > 100) {
      throw new ImageError('JPEG quality must be between 1 and 100');
    }
    return this.renderToFormat(text, ImageFormat.JPEG, quality);
  }

  /**
   * Internal method to render text to any supported image format.
   */
  private renderToFormat(text: string, format: ImageFormat, quality: number): Buffer {
    // Calculate optimal font size
    const fontSize = this.calculateOptimalFontSize(text);

    // Wrap text into lines
    const lines = this.wrapText(text, fontSize);

    // Create canvas with background color
    const canvas = createCanvas(this.config.width, this.config.height);
    const ctx = canvas.getContext('2d');

    // Fill background
    const [r, g, b] = this.config.bgColor;
    ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
    ctx.fillRect(0, 0, this.config.width, this.config.height);

    // Render each line
    this.renderLines(ctx, lines, fontSize);

    // Encode to requested format
    if (format === ImageFormat.PNG) {
      return canvas.toBuffer('image/png');
    } else {
      return canvas.toBuffer('image/jpeg', quality);
    }
  }

  /**
   * Calculate optimal font size to fit text in image.
   *
   * Starts with config fontSize and reduces until text fits.
   */
  private calculateOptimalFontSize(text: string): number {
    let fontSize = this.config.fontSize;

    // Try progressively smaller font sizes
    while (fontSize >= this.config.minFontSize) {
      const lines = this.wrapText(text, fontSize);
      const maxLines = this.calculateMaxLines(fontSize);

      if (lines.length <= maxLines) {
        return fontSize;
      }

      // Reduce font size by 0.5
      fontSize -= 0.5;
    }

    // Text still doesn't fit with minimum font size
    const lines = this.wrapText(text, this.config.minFontSize);
    const maxLines = this.calculateMaxLines(this.config.minFontSize);
    throw new TextTooLargeError(lines.length, maxLines);
  }

  /**
   * Calculate maximum number of lines that fit in image height.
   */
  private calculateMaxLines(fontSize: number): number {
    const availableHeight = this.config.height - this.config.marginY * 2;
    const lineHeight = fontSize * this.config.lineSpacing;

    if (lineHeight === 0) {
      return 0;
    }

    return Math.floor(availableHeight / lineHeight);
  }

  /**
   * Break text into lines that fit within image width.
   */
  private wrapText(text: string, fontSize: number): string[] {
    // Create temporary canvas to measure text
    const canvas = createCanvas(100, 100);
    const ctx = canvas.getContext('2d');
    ctx.font = `${fontSize}px ${this.config.fontFamily}`;

    const availableWidth = this.config.width - this.config.marginX * 2;

    const lines: string[] = [];
    let currentLine = '';
    let currentWidth = 0;

    for (const ch of text) {
      // Handle newlines
      if (ch === '\n') {
        lines.push(currentLine);
        currentLine = '';
        currentWidth = 0;
        continue;
      }

      // Calculate character width
      const charWidth = ctx.measureText(ch).width;

      // Check if adding this character would exceed width
      if (currentWidth + charWidth > availableWidth && currentLine.length > 0) {
        lines.push(currentLine);
        currentLine = '';
        currentWidth = 0;
      }

      currentLine += ch;
      currentWidth += charWidth;
    }

    // Add last line if not empty
    if (currentLine.length > 0) {
      lines.push(currentLine);
    }

    return lines;
  }

  /**
   * Render lines of text onto the canvas context.
   */
  private renderLines(ctx: CanvasRenderingContext2D, lines: string[], fontSize: number): void {
    // Set up text rendering
    ctx.font = `${fontSize}px ${this.config.fontFamily}`;
    const [r, g, b] = this.config.textColor;
    ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
    ctx.textBaseline = 'top';

    const lineHeight = fontSize * this.config.lineSpacing;
    let y = this.config.marginY;

    for (const line of lines) {
      const x = this.config.marginX;

      // Render the line
      ctx.fillText(line, x, y);

      y += lineHeight;

      // Stop if we've exceeded image height
      if (y >= this.config.height - this.config.marginY) {
        break;
      }
    }
  }
}
