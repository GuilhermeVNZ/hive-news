"""
Compression Prompt - Fast statistical compression for LLM prompts

Statistical compression using intelligent filtering to achieve 50% token reduction 
with 91% quality retention.
"""

from .compressor import Compressor, CompressorConfig, CompressionResult, OutputFormat
from .statistical_filter import StatisticalFilter, StatisticalFilterConfig, WordImportance
from .quality_metrics import QualityMetrics

# Image rendering (optional - requires Pillow)
try:
    from .image_renderer import ImageRenderer, ImageRendererConfig
    __all__ = [
        "Compressor",
        "CompressorConfig",
        "CompressionResult",
        "OutputFormat",
        "StatisticalFilter",
        "StatisticalFilterConfig",
        "WordImportance",
        "QualityMetrics",
        "ImageRenderer",
        "ImageRendererConfig",
    ]
except ImportError:
    __all__ = [
        "Compressor",
        "CompressorConfig",
        "CompressionResult",
        "OutputFormat",
        "StatisticalFilter",
        "StatisticalFilterConfig",
        "WordImportance",
        "QualityMetrics",
    ]

__version__ = "0.1.0"

