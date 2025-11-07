"""Main compression pipeline and result structures."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

from .statistical_filter import StatisticalFilter, StatisticalFilterConfig


class OutputFormat(Enum):
    """Output format for compression result."""
    TEXT = "text"
    IMAGE = "image"


class CompressionError(Exception):
    """Base class for compression errors."""
    pass


class NegativeGainError(CompressionError):
    """Compression would increase token count."""
    def __init__(self, ratio: float):
        super().__init__(f"Compression ratio {ratio:.2f} >= 1.0, would increase tokens")
        self.ratio = ratio


class InputTooShortError(CompressionError):
    """Input too short to compress."""
    def __init__(self, size: int, minimum: int):
        super().__init__(f"Input too short ({size} tokens/bytes), minimum is {minimum}")
        self.size = size
        self.minimum = minimum


@dataclass
class CompressorConfig:
    """Configuration for the compressor."""
    
    target_ratio: float = 0.5  # 50% of original size
    min_input_tokens: int = 100
    min_input_bytes: int = 1024


@dataclass
class CompressionResult:
    """Result of compression operation."""
    
    compressed: str
    image_data: Optional[bytes] = None
    format: OutputFormat = OutputFormat.TEXT
    original_tokens: int = 0
    compressed_tokens: int = 0
    compression_ratio: float = 0.0
    tokens_removed: int = 0


class Compressor:
    """Main compressor."""
    
    def __init__(self, config: Optional[CompressorConfig] = None, 
                 filter_config: Optional[StatisticalFilterConfig] = None):
        """
        Create a new compressor with given configuration.
        
        Args:
            config: Compressor configuration (optional)
            filter_config: Statistical filter configuration (optional)
        """
        self.config = config or CompressorConfig()
        
        if filter_config is None:
            filter_config = StatisticalFilterConfig(
                compression_ratio=self.config.target_ratio
            )
        
        self.filter = StatisticalFilter(filter_config)
    
    def compress(self, input_text: str) -> CompressionResult:
        """
        Compress input text using statistical filtering.
        
        Args:
            input_text: The text to compress
            
        Returns:
            CompressionResult with compressed text and statistics
            
        Raises:
            CompressionError: If compression would be counterproductive
        """
        return self.compress_with_format(input_text, OutputFormat.TEXT)
    
    def compress_with_format(self, input_text: str, 
                            format: OutputFormat) -> CompressionResult:
        """
        Compress input text with specified output format.
        
        Args:
            input_text: The text to compress
            format: Output format (Text or Image)
            
        Returns:
            CompressionResult with compressed text and optional image data
            
        Raises:
            CompressionError: If input is too short or compression would increase size
        """
        # Step 1: Check input size (bytes)
        input_bytes = len(input_text.encode('utf-8'))
        if input_bytes < self.config.min_input_bytes:
            raise InputTooShortError(input_bytes, self.config.min_input_bytes)
        
        # Step 2: Estimate tokens (using char count / 4 as rough estimate)
        original_tokens = len(input_text) // 4
        if original_tokens < self.config.min_input_tokens:
            raise InputTooShortError(original_tokens, self.config.min_input_tokens)
        
        # Step 3: Apply statistical filtering
        compressed = self.filter.compress(input_text)
        
        # Step 4: Validate compression ratio
        compressed_tokens = len(compressed) // 4
        compression_ratio = compressed_tokens / original_tokens if original_tokens > 0 else 1.0
        
        if compression_ratio >= 1.0:
            raise NegativeGainError(compression_ratio)
        
        tokens_removed = max(0, original_tokens - compressed_tokens)
        
        # Step 5: Generate image if requested
        image_data = None
        if format == OutputFormat.IMAGE:
            try:
                from .image_renderer import ImageRenderer
                renderer = ImageRenderer()
                image_data = renderer.render_to_png(compressed)
            except ImportError:
                # Pillow not installed, skip image generation
                pass
        
        return CompressionResult(
            compressed=compressed,
            image_data=image_data,
            format=format,
            original_tokens=original_tokens,
            compressed_tokens=compressed_tokens,
            compression_ratio=compression_ratio,
            tokens_removed=tokens_removed
        )

