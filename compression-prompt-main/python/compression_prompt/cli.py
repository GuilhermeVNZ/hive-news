#!/usr/bin/env python3
"""CLI tool for compressing text using compression-prompt"""

import sys
import argparse
from typing import Optional

from .compressor import (
    Compressor, CompressorConfig, StatisticalFilterConfig,
    OutputFormat, CompressionError
)


def main() -> int:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description='Compress text using statistical filtering',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  compress input.txt                        # Compress to stdout
  compress -r 0.7 input.txt                 # Conservative (70%)
  compress -r 0.3 input.txt                 # Aggressive (30%)
  cat input.txt | compress                  # Read from stdin
  compress -s input.txt                     # Show statistics
"""
    )
    
    parser.add_argument('input_file', nargs='?', type=str,
                       help='Input file (default: stdin)')
    parser.add_argument('-r', '--ratio', type=float, default=0.5,
                       help='Compression ratio (0.0-1.0, default: 0.5)')
    parser.add_argument('-o', '--output', type=str,
                       help='Output file (default: stdout)')
    parser.add_argument('-s', '--stats', action='store_true',
                       help='Show compression statistics')
    parser.add_argument('-v', '--version', action='version', version='%(prog)s 0.1.0')
    
    args = parser.parse_args()
    
    # Validate ratio
    if not 0.0 <= args.ratio <= 1.0:
        print("Error: Ratio must be between 0.0 and 1.0", file=sys.stderr)
        return 1
    
    # Read input
    try:
        if args.input_file:
            with open(args.input_file, 'r', encoding='utf-8') as f:
                input_text = f.read()
        else:
            input_text = sys.stdin.read()
    except Exception as e:
        print(f"Error reading input: {e}", file=sys.stderr)
        return 1
    
    # Configure compressor
    compressor_config = CompressorConfig(
        target_ratio=args.ratio,
        min_input_bytes=100,  # Lower threshold for CLI
        min_input_tokens=10
    )
    
    filter_config = StatisticalFilterConfig(
        compression_ratio=args.ratio
    )
    
    compressor = Compressor(compressor_config, filter_config)
    
    # Compress
    try:
        result = compressor.compress_with_format(input_text, OutputFormat.TEXT)
    except CompressionError as e:
        print(f"Compression error: {e}", file=sys.stderr)
        return 1
    
    # Show stats if requested
    if args.stats:
        print("Compression Statistics:", file=sys.stderr)
        print(f"  Original tokens:   {result.original_tokens}", file=sys.stderr)
        print(f"  Compressed tokens: {result.compressed_tokens}", file=sys.stderr)
        print(f"  Tokens removed:    {result.tokens_removed}", file=sys.stderr)
        print(f"  Compression ratio: {(1.0 - result.compression_ratio) * 100:.1f}%", file=sys.stderr)
        print(f"  Target ratio:      {(1.0 - args.ratio) * 100:.1f}%", file=sys.stderr)
        print(file=sys.stderr)
    
    # Write output
    try:
        if args.output:
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(result.compressed)
        else:
            print(result.compressed)
    except Exception as e:
        print(f"Error writing output: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == '__main__':
    sys.exit(main())

