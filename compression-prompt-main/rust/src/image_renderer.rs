//! Dense text-to-image rendering for vision model consumption.
//!
//! Inspired by DeepSeek-OCR's optical context compression approach, this module
//! renders compressed text into fixed-size 1024x1024 monospace images optimized
//! for vision model processing.

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{ImageBuffer, ImageFormat, Rgb, RgbImage, codecs::jpeg::JpegEncoder};
use std::io::Cursor;
use thiserror::Error;

/// Errors that can occur during image rendering.
#[derive(Error, Debug)]
pub enum ImageError {
    /// Font loading error.
    #[error("Failed to load font: {0}")]
    FontError(String),

    /// Image encoding error.
    #[error("Failed to encode image: {0}")]
    EncodingError(#[from] image::ImageError),

    /// Text is too large to fit in image even with minimum font size.
    #[error("Text too large to fit in image (requires {0} lines, max {1})")]
    TextTooLarge(usize, usize),
}

/// Configuration for image rendering.
#[derive(Debug, Clone)]
pub struct ImageRendererConfig {
    /// Image width in pixels (default: 1024).
    pub width: u32,

    /// Image height in pixels (default: 1024).
    pub height: u32,

    /// Base font size for monospace text (default: 12.0).
    pub font_size: f32,

    /// Line spacing multiplier (default: 1.2 for comfortable reading).
    pub line_spacing: f32,

    /// Horizontal margin in pixels (default: 20).
    pub margin_x: u32,

    /// Vertical margin in pixels (default: 20).
    pub margin_y: u32,

    /// Background color RGB (default: white [255, 255, 255]).
    pub bg_color: [u8; 3],

    /// Text color RGB (default: black [0, 0, 0]).
    pub text_color: [u8; 3],

    /// Minimum font size when auto-scaling (default: 6.0).
    pub min_font_size: f32,
}

impl Default for ImageRendererConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            font_size: 12.5, // Aumentado levemente de 12.0 para 12.5 (melhor legibilidade)
            line_spacing: 1.2,
            margin_x: 20,
            margin_y: 20,
            bg_color: [255, 255, 255],
            text_color: [0, 0, 0],
            min_font_size: 7.0,
        }
    }
}

/// Image renderer for converting text to PNG images.
pub struct ImageRenderer {
    /// Rendering configuration.
    pub config: ImageRendererConfig,
    /// Embedded monospace font (DejaVu Sans Mono).
    font_data: &'static [u8],
}

// Embed DejaVu Sans Mono font at compile time
// This is a common open-source monospace font
// Path is relative to the source file location (src/)
const DEJAVU_MONO: &[u8] = include_bytes!("../fonts/DejaVuSansMono.ttf");

impl ImageRenderer {
    /// Create a new image renderer with the given configuration.
    pub fn new(config: ImageRendererConfig) -> Self {
        Self {
            config,
            font_data: DEJAVU_MONO,
        }
    }

    /// Render text to PNG image bytes.
    ///
    /// This method:
    /// 1. Calculates optimal font size to fit text in image
    /// 2. Wraps text into lines
    /// 3. Renders each line with monospace font
    /// 4. Encodes final image as PNG
    ///
    /// # Arguments
    ///
    /// * `text` - The text to render
    ///
    /// # Returns
    ///
    /// PNG image data as bytes, or an error if rendering fails.
    pub fn render_to_png(&self, text: &str) -> Result<Vec<u8>, ImageError> {
        self.render_to_format(text, ImageFormat::Png, 100)
    }

    /// Render text to JPEG image bytes.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to render
    /// * `quality` - JPEG quality (1-100, higher = better quality but larger file)
    ///
    /// # Returns
    ///
    /// JPEG image data as bytes, or an error if rendering fails.
    pub fn render_to_jpeg(&self, text: &str, quality: u8) -> Result<Vec<u8>, ImageError> {
        self.render_to_format(text, ImageFormat::Jpeg, quality)
    }

    /// Internal method to render text to any supported image format.
    fn render_to_format(
        &self,
        text: &str,
        format: ImageFormat,
        quality: u8,
    ) -> Result<Vec<u8>, ImageError> {
        // Load font
        let font = FontRef::try_from_slice(self.font_data)
            .map_err(|e| ImageError::FontError(format!("{:?}", e)))?;

        // Calculate optimal font size
        let font_size = self.calculate_optimal_font_size(text, &font)?;

        // Wrap text into lines
        let lines = self.wrap_text(text, font_size, &font);

        // Create image buffer with background color
        let mut img: RgbImage = ImageBuffer::from_pixel(
            self.config.width,
            self.config.height,
            Rgb(self.config.bg_color),
        );

        // Render each line
        self.render_lines(&mut img, &lines, font_size, &font);

        // Encode to requested format
        let mut image_data = Vec::new();
        let mut cursor = Cursor::new(&mut image_data);

        match format {
            ImageFormat::Png => {
                img.write_to(&mut cursor, ImageFormat::Png)?;
            }
            ImageFormat::Jpeg => {
                let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
                encoder.encode(
                    img.as_raw(),
                    self.config.width,
                    self.config.height,
                    image::ExtendedColorType::Rgb8,
                )?;
            }
            _ => {
                return Err(ImageError::EncodingError(image::ImageError::Unsupported(
                    image::error::UnsupportedError::from_format_and_kind(
                        image::error::ImageFormatHint::Unknown,
                        image::error::UnsupportedErrorKind::Format(
                            image::error::ImageFormatHint::Unknown,
                        ),
                    ),
                )));
            }
        }

        Ok(image_data)
    }

    /// Calculate optimal font size to fit text in image.
    ///
    /// Starts with config font_size and reduces until text fits.
    fn calculate_optimal_font_size(&self, text: &str, font: &FontRef) -> Result<f32, ImageError> {
        let mut font_size = self.config.font_size;

        // Try progressively smaller font sizes
        while font_size >= self.config.min_font_size {
            let lines = self.wrap_text(text, font_size, font);
            let max_lines = self.calculate_max_lines(font_size);

            if lines.len() <= max_lines {
                return Ok(font_size);
            }

            // Reduce font size by 0.5
            font_size -= 0.5;
        }

        // Text still doesn't fit with minimum font size
        let lines = self.wrap_text(text, self.config.min_font_size, font);
        let max_lines = self.calculate_max_lines(self.config.min_font_size);
        Err(ImageError::TextTooLarge(lines.len(), max_lines))
    }

    /// Calculate maximum number of lines that fit in image height.
    fn calculate_max_lines(&self, font_size: f32) -> usize {
        let available_height = self.config.height - (self.config.margin_y * 2);
        let line_height = (font_size * self.config.line_spacing) as u32;

        if line_height == 0 {
            return 0;
        }

        (available_height / line_height) as usize
    }

    /// Break text into lines that fit within image width.
    fn wrap_text(&self, text: &str, font_size: f32, font: &FontRef) -> Vec<String> {
        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);

        let available_width = (self.config.width - (self.config.margin_x * 2)) as f32;

        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;

        for ch in text.chars() {
            // Handle newlines
            if ch == '\n' {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
                continue;
            }

            // Calculate character width
            let glyph_id = font.glyph_id(ch);
            let advance = scaled_font.h_advance(glyph_id);

            // Check if adding this character would exceed width
            if current_width + advance > available_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
            }

            current_line.push(ch);
            current_width += advance;
        }

        // Add last line if not empty
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Render lines of text onto the image buffer.
    fn render_lines(&self, img: &mut RgbImage, lines: &[String], font_size: f32, font: &FontRef) {
        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);

        let line_height = (font_size * self.config.line_spacing) as i32;
        let mut y = self.config.margin_y as i32;

        for line in lines {
            let x_start = self.config.margin_x as f32;
            let mut x_offset = 0.0f32;

            for ch in line.chars() {
                let glyph = scaled_font.scaled_glyph(ch);
                let glyph_id = glyph.id;

                // Outline and rasterize glyph
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();

                    outlined.draw(|gx, gy, v| {
                        // Calculate pixel position with x offset
                        let px = ((x_start + x_offset) as i32 + gx as i32) as u32;
                        let py = (bounds.min.y as i32 + gy as i32 + y) as u32;

                        // Check bounds
                        if px < self.config.width && py < self.config.height {
                            // Alpha blending
                            let alpha = v;
                            let inv_alpha = 1.0 - alpha;

                            let bg = img.get_pixel(px, py);
                            let r = (self.config.text_color[0] as f32 * alpha
                                + bg[0] as f32 * inv_alpha)
                                as u8;
                            let g = (self.config.text_color[1] as f32 * alpha
                                + bg[1] as f32 * inv_alpha)
                                as u8;
                            let b = (self.config.text_color[2] as f32 * alpha
                                + bg[2] as f32 * inv_alpha)
                                as u8;

                            img.put_pixel(px, py, Rgb([r, g, b]));
                        }
                    });
                }

                // Advance x position
                x_offset += scaled_font.h_advance(glyph_id);
            }

            y += line_height;

            // Stop if we've exceeded image height
            if y >= self.config.height as i32 {
                break;
            }
        }
    }
}

impl Default for ImageRenderer {
    fn default() -> Self {
        Self::new(ImageRendererConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_renderer() {
        let renderer = ImageRenderer::default();
        assert_eq!(renderer.config.width, 1024);
        assert_eq!(renderer.config.height, 1024);
    }

    #[test]
    fn test_render_simple_text() {
        let renderer = ImageRenderer::default();
        let text = "Hello, World!";

        let result = renderer.render_to_png(text);
        if let Err(e) = &result {
            eprintln!("Error rendering text: {:?}", e);
        }
        assert!(result.is_ok());

        let png_data = result.unwrap();
        assert!(!png_data.is_empty());

        // Verify PNG signature
        assert_eq!(&png_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_render_multiline_text() {
        let renderer = ImageRenderer::default();
        let text = "Line 1\nLine 2\nLine 3";

        let result = renderer.render_to_png(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_long_text() {
        let renderer = ImageRenderer::default();
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(50);

        let result = renderer.render_to_png(&text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_config() {
        let config = ImageRendererConfig {
            width: 512,
            height: 512,
            font_size: 10.0,
            bg_color: [240, 240, 240],
            text_color: [32, 32, 32],
            ..Default::default()
        };

        let renderer = ImageRenderer::new(config);
        let result = renderer.render_to_png("Test");
        assert!(result.is_ok());
    }
}
