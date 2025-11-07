#!/usr/bin/env python3
"""Example: Compress text and render as image for vision models"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from compression_prompt import Compressor, ImageRenderer, OutputFormat
    from compression_prompt.statistical_filter import StatisticalFilterConfig
except ImportError as e:
    print(f"Error: {e}")
    print("\nImage rendering requires Pillow. Install with:")
    print("  pip install Pillow")
    sys.exit(1)


def main():
    """Demonstrate image output for vision models."""
    
    # Sample text (large enough to compress)
    base_text = """
    Machine Learning revolutionizes artificial intelligence by enabling systems to 
    learn from data. Deep Learning uses neural networks with multiple layers to 
    process complex patterns. Natural Language Processing allows computers to 
    understand human language. Computer Vision interprets visual information. 
    Reinforcement Learning optimizes decision-making through trial and error.
    Transfer Learning leverages knowledge across tasks. Generative AI creates 
    new content including text, images, and code.
    """
    
    text = base_text * 5  # Repeat to meet minimum size
    
    print("=" * 70)
    print("IMAGE OUTPUT EXAMPLE - Optical Context Compression")
    print("=" * 70)
    print()
    
    # Compress text
    compressor = Compressor()
    result = compressor.compress(text)
    
    print(f"Original tokens:   {result.original_tokens}")
    print(f"Compressed tokens: {result.compressed_tokens}")
    print(f"Compression:       {(1.0 - result.compression_ratio) * 100:.1f}%")
    print()
    
    # Render to PNG
    print("Rendering to PNG...")
    renderer = ImageRenderer()
    png_data = renderer.render_to_png(result.compressed)
    
    png_path = Path("compressed_output.png")
    with open(png_path, 'wb') as f:
        f.write(png_data)
    
    print(f"✅ PNG saved: {png_path} ({len(png_data):,} bytes)")
    
    # Render to JPEG (smaller file)
    print("\nRendering to JPEG...")
    jpeg_data = renderer.render_to_jpeg(result.compressed, quality=85)
    
    jpeg_path = Path("compressed_output.jpg")
    with open(jpeg_path, 'wb') as f:
        f.write(jpeg_data)
    
    print(f"✅ JPEG saved: {jpeg_path} ({len(jpeg_data):,} bytes)")
    
    # Show size comparison
    print("\n" + "=" * 70)
    print("FILE SIZE COMPARISON:")
    print("-" * 70)
    print(f"PNG:  {len(png_data):,} bytes")
    print(f"JPEG: {len(jpeg_data):,} bytes ({len(jpeg_data)/len(png_data)*100:.1f}% of PNG)")
    print()
    
    print("🎯 Use Case:")
    print("  Send these images to vision models (GPT-4V, Claude 3, Gemini Vision)")
    print("  for processing compressed context efficiently!")
    print("=" * 70)


if __name__ == '__main__':
    main()

