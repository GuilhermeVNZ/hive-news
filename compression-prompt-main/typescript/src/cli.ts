#!/usr/bin/env node

import { Command } from 'commander';
import * as fs from 'fs';
import { Compressor, OutputFormat } from './compressor';
import { QualityMetricsCalculator } from './quality-metrics';

const program = new Command();

program
  .name('compress')
  .description('Compress text for LLM prompts - 50% token reduction with 91% quality retention')
  .version('0.1.0')
  .argument('[input]', 'Input file (defaults to stdin)')
  .option(
    '-o, --output <file>',
    'Output file (defaults to stdout for text, compressed.png for image)'
  )
  .option('-r, --ratio <ratio>', 'Compression ratio (0.0-1.0, default: 0.5)', parseFloat, 0.5)
  .option('-s, --stats', 'Show compression statistics')
  .option('--quality', 'Show quality metrics')
  .option('-f, --format <format>', 'Output format: text, png, jpeg (default: text)', 'text')
  .option('--jpeg-quality <quality>', 'JPEG quality (1-100, default: 85)', parseFloat, 85)
  .action(async (input: string | undefined, options: any) => {
    try {
      // Read input
      let text: string;
      if (input) {
        text = fs.readFileSync(input, 'utf-8');
      } else {
        // Read from stdin
        text = fs.readFileSync(0, 'utf-8');
      }

      if (text.trim().length === 0) {
        console.error('Error: Empty input');
        process.exit(1);
      }

      // Validate format option
      const validFormats = ['text', 'png', 'jpeg', 'jpg'];
      if (!validFormats.includes(options.format.toLowerCase())) {
        console.error(`Error: Invalid format '${options.format}'. Use: text, png, or jpeg`);
        process.exit(1);
      }

      // Determine output format
      const format =
        options.format.toLowerCase() === 'text' ? OutputFormat.TEXT : OutputFormat.IMAGE;

      // Configure compressor
      const compressor = new Compressor({
        targetRatio: options.ratio,
        minInputTokens: 10,
        minInputBytes: 10,
      });

      // Compress with format options
      const formatOptions = {
        imageFormat:
          options.format.toLowerCase() === 'jpeg' || options.format.toLowerCase() === 'jpg'
            ? ('jpeg' as const)
            : ('png' as const),
        jpegQuality: options.jpegQuality,
      };

      const result = compressor.compressWithFormat(text, format, formatOptions);

      // Calculate quality metrics if requested
      let quality;
      if (options.quality) {
        quality = QualityMetricsCalculator.calculate(text, result.compressed);
      }

      // Show stats if requested
      if (options.stats) {
        console.error('Compression Statistics:');
        console.error(`  Original tokens:   ${result.originalTokens}`);
        console.error(`  Compressed tokens: ${result.compressedTokens}`);
        console.error(`  Tokens removed:    ${result.tokensRemoved}`);
        console.error(`  Compression ratio: ${((1 - result.compressionRatio) * 100).toFixed(1)}%`);
        console.error(`  Target ratio:      ${((1 - options.ratio) * 100).toFixed(1)}%`);
        console.error('');

        if (quality) {
          console.error('Quality Metrics:');
          console.error(`  Overall score:      ${(quality.overallScore * 100).toFixed(1)}%`);
          console.error(`  Keyword retention:  ${(quality.keywordRetention * 100).toFixed(1)}%`);
          console.error(`  Entity retention:   ${(quality.entityRetention * 100).toFixed(1)}%`);
          console.error(`  Vocabulary ratio:   ${(quality.vocabularyRatio * 100).toFixed(1)}%`);
          console.error(`  Info density:       ${(quality.informationDensity * 100).toFixed(1)}%`);
          console.error('');
        }
      }

      // Write output
      if (format === OutputFormat.TEXT) {
        // Text output
        if (options.output) {
          fs.writeFileSync(options.output, result.compressed);
          if (options.stats) {
            console.error(`Text saved to: ${options.output}`);
          }
        } else {
          process.stdout.write(result.compressed);
        }
      } else {
        // Image output
        if (!result.imageData) {
          console.error('Error: Image generation failed');
          process.exit(1);
        }

        const outputPath =
          options.output ||
          (options.format.toLowerCase() === 'jpeg' || options.format.toLowerCase() === 'jpg'
            ? 'compressed.jpg'
            : 'compressed.png');

        fs.writeFileSync(outputPath, result.imageData);

        if (options.stats) {
          console.error(`Image saved to: ${outputPath}`);
          console.error(
            `Image size: ${Math.floor(result.imageData.length / 1024)} KB (${result.imageData.length} bytes)`
          );
          console.error(`Dimensions: 1024x1024`);

          // Verify format
          const isPng = result.imageData[0] === 137 && result.imageData[1] === 80;
          const isJpeg = result.imageData[0] === 0xff && result.imageData[1] === 0xd8;

          if (isPng) {
            console.error('Format: PNG ✓');
          } else if (isJpeg) {
            console.error('Format: JPEG ✓');
          }

          // Text saved separately
          const textPath = outputPath.replace(/\.(png|jpg|jpeg)$/i, '.txt');
          fs.writeFileSync(textPath, result.compressed);
          console.error(`Compressed text saved to: ${textPath}`);
        } else {
          console.error(`Saved to: ${outputPath}`);
        }
      }
    } catch (error) {
      if (error instanceof Error) {
        console.error(`Error: ${error.message}`);
      } else {
        console.error('Unknown error occurred');
      }
      process.exit(1);
    }
  });

program.parse();
