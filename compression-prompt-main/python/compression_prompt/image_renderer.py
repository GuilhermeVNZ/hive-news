"""
Image rendering for compressed text (Optical Context Compression)

Inspired by DeepSeek-OCR's optical context compression:
https://arxiv.org/html/2510.18234v1

Renders compressed text as 1024x1024 monospace images for vision model consumption.
"""

from dataclasses import dataclass
from typing import List, Tuple, Optional
import io

try:
    from PIL import Image, ImageDraw, ImageFont
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False


@dataclass
class ImageRendererConfig:
    """Configuration for image rendering."""
    
    width: int = 1024
    height: int = 1024
    font_size: int = 12
    line_spacing: int = 4
    margin: int = 20
    background_color: Tuple[int, int, int] = (255, 255, 255)  # White
    text_color: Tuple[int, int, int] = (0, 0, 0)  # Black


class ImageRenderer:
    """Renders text as monospace images for vision models."""
    
    def __init__(self, config: Optional[ImageRendererConfig] = None):
        """
        Create a new image renderer.
        
        Args:
            config: Renderer configuration (optional)
            
        Raises:
            ImportError: If Pillow is not installed
        """
        if not PIL_AVAILABLE:
            raise ImportError(
                "Pillow is required for image rendering. "
                "Install with: pip install Pillow"
            )
        
        self.config = config or ImageRendererConfig()
        
        # Try to load a monospace font, fallback to default
        try:
            # Try common monospace fonts
            for font_name in ['DejaVuSansMono.ttf', 'LiberationMono-Regular.ttf', 
                             'Courier New.ttf', 'Consolas.ttf']:
                try:
                    self.font = ImageFont.truetype(font_name, self.config.font_size)
                    break
                except:
                    continue
            else:
                # Fallback to default font
                self.font = ImageFont.load_default()
        except:
            self.font = ImageFont.load_default()
    
    def _wrap_text(self, text: str, max_width: int) -> List[str]:
        """
        Wrap text to fit within max width.
        
        Args:
            text: Text to wrap
            max_width: Maximum width in pixels
            
        Returns:
            List of wrapped lines
        """
        words = text.split()
        lines = []
        current_line = []
        
        for word in words:
            test_line = ' '.join(current_line + [word])
            bbox = self.font.getbbox(test_line)
            width = bbox[2] - bbox[0]
            
            if width <= max_width:
                current_line.append(word)
            else:
                if current_line:
                    lines.append(' '.join(current_line))
                current_line = [word]
        
        if current_line:
            lines.append(' '.join(current_line))
        
        return lines
    
    def _calculate_pages(self, text: str) -> List[List[str]]:
        """
        Calculate how many pages are needed and split text accordingly.
        
        Args:
            text: Text to paginate
            
        Returns:
            List of pages, each containing list of lines
        """
        max_text_width = self.config.width - (2 * self.config.margin)
        max_text_height = self.config.height - (2 * self.config.margin)
        
        # Wrap text into lines
        lines = self._wrap_text(text, max_text_width)
        
        # Calculate line height
        bbox = self.font.getbbox("Test")
        line_height = bbox[3] - bbox[1] + self.config.line_spacing
        
        # Calculate max lines per page
        max_lines_per_page = max_text_height // line_height
        
        # Split lines into pages
        pages = []
        for i in range(0, len(lines), max_lines_per_page):
            pages.append(lines[i:i + max_lines_per_page])
        
        return pages
    
    def render_to_png(self, text: str) -> bytes:
        """
        Render text to PNG image bytes.
        
        Args:
            text: Text to render
            
        Returns:
            PNG image as bytes
        """
        pages = self._calculate_pages(text)
        
        if not pages:
            # Empty text, return blank image
            img = Image.new('RGB', 
                          (self.config.width, self.config.height), 
                          self.config.background_color)
            buffer = io.BytesIO()
            img.save(buffer, format='PNG')
            return buffer.getvalue()
        
        # For now, render only first page
        # TODO: Support multi-page rendering
        page_lines = pages[0]
        
        # Create image
        img = Image.new('RGB', 
                       (self.config.width, self.config.height), 
                       self.config.background_color)
        draw = ImageDraw.Draw(img)
        
        # Calculate line height
        bbox = self.font.getbbox("Test")
        line_height = bbox[3] - bbox[1] + self.config.line_spacing
        
        # Draw text
        y = self.config.margin
        for line in page_lines:
            draw.text((self.config.margin, y), line, 
                     fill=self.config.text_color, font=self.font)
            y += line_height
        
        # Convert to bytes
        buffer = io.BytesIO()
        img.save(buffer, format='PNG')
        return buffer.getvalue()
    
    def render_to_jpeg(self, text: str, quality: int = 85) -> bytes:
        """
        Render text to JPEG image bytes.
        
        Args:
            text: Text to render
            quality: JPEG quality (1-100, default: 85)
            
        Returns:
            JPEG image as bytes
        """
        if not 1 <= quality <= 100:
            raise ValueError("Quality must be between 1 and 100")
        
        pages = self._calculate_pages(text)
        
        if not pages:
            # Empty text, return blank image
            img = Image.new('RGB', 
                          (self.config.width, self.config.height), 
                          self.config.background_color)
            buffer = io.BytesIO()
            img.save(buffer, format='JPEG', quality=quality)
            return buffer.getvalue()
        
        # For now, render only first page
        page_lines = pages[0]
        
        # Create image
        img = Image.new('RGB', 
                       (self.config.width, self.config.height), 
                       self.config.background_color)
        draw = ImageDraw.Draw(img)
        
        # Calculate line height
        bbox = self.font.getbbox("Test")
        line_height = bbox[3] - bbox[1] + self.config.line_spacing
        
        # Draw text
        y = self.config.margin
        for line in page_lines:
            draw.text((self.config.margin, y), line, 
                     fill=self.config.text_color, font=self.font)
            y += line_height
        
        # Convert to bytes
        buffer = io.BytesIO()
        img.save(buffer, format='JPEG', quality=quality)
        return buffer.getvalue()
    
    def render_to_file(self, text: str, output_path: str, 
                      format: str = 'png', quality: int = 85) -> None:
        """
        Render text to image file.
        
        Args:
            text: Text to render
            output_path: Path to save image
            format: Image format ('png' or 'jpeg')
            quality: JPEG quality (only used for JPEG)
        """
        if format.lower() == 'png':
            data = self.render_to_png(text)
        elif format.lower() in ['jpeg', 'jpg']:
            data = self.render_to_jpeg(text, quality)
        else:
            raise ValueError(f"Unsupported format: {format}")
        
        with open(output_path, 'wb') as f:
            f.write(data)


# Export for backwards compatibility
__all__ = ['ImageRenderer', 'ImageRendererConfig']

