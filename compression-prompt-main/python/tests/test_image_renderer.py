"""Tests for image renderer module."""

import pytest

try:
    from compression_prompt import ImageRenderer, ImageRendererConfig
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_render_to_png():
    """Test PNG rendering."""
    renderer = ImageRenderer()
    text = "Hello, World! " * 100
    
    png_data = renderer.render_to_png(text)
    
    assert isinstance(png_data, bytes)
    assert len(png_data) > 0
    assert png_data.startswith(b'\x89PNG')  # PNG magic number


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_render_to_jpeg():
    """Test JPEG rendering."""
    renderer = ImageRenderer()
    text = "Hello, World! " * 100
    
    jpeg_data = renderer.render_to_jpeg(text, quality=85)
    
    assert isinstance(jpeg_data, bytes)
    assert len(jpeg_data) > 0
    assert jpeg_data.startswith(b'\xff\xd8')  # JPEG magic number


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_jpeg_quality():
    """Test JPEG quality parameter."""
    renderer = ImageRenderer()
    text = "Test text " * 100
    
    # Higher quality = larger file
    high_quality = renderer.render_to_jpeg(text, quality=95)
    low_quality = renderer.render_to_jpeg(text, quality=50)
    
    assert len(high_quality) > len(low_quality)


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_empty_text():
    """Test rendering empty text."""
    renderer = ImageRenderer()
    
    png_data = renderer.render_to_png("")
    assert len(png_data) > 0
    
    jpeg_data = renderer.render_to_jpeg("", quality=85)
    assert len(jpeg_data) > 0


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_custom_config():
    """Test custom configuration."""
    config = ImageRendererConfig(
        width=800,
        height=600,
        font_size=14,
        background_color=(255, 255, 255),
        text_color=(0, 0, 0)
    )
    
    renderer = ImageRenderer(config)
    text = "Custom config test " * 50
    
    png_data = renderer.render_to_png(text)
    assert len(png_data) > 0


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_render_to_file(tmp_path):
    """Test rendering to file."""
    renderer = ImageRenderer()
    text = "File output test " * 50
    
    # Test PNG
    png_file = tmp_path / "test.png"
    renderer.render_to_file(text, str(png_file), format='png')
    assert png_file.exists()
    assert png_file.stat().st_size > 0
    
    # Test JPEG
    jpeg_file = tmp_path / "test.jpg"
    renderer.render_to_file(text, str(jpeg_file), format='jpeg', quality=85)
    assert jpeg_file.exists()
    assert jpeg_file.stat().st_size > 0


@pytest.mark.skipif(not PIL_AVAILABLE, reason="Pillow not installed")
def test_long_text():
    """Test rendering long text (multi-page)."""
    renderer = ImageRenderer()
    text = "This is a very long text that will span multiple pages. " * 500
    
    png_data = renderer.render_to_png(text)
    assert len(png_data) > 0

