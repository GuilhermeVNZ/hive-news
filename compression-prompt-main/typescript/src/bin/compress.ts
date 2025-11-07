#!/usr/bin/env node
/**
 * CLI tool for compressing text using compression-prompt
 */

import * as fs from 'fs';
import * as process from 'process';
import { Compressor, CompressionError, OutputFormat } from '../compressor';

interface CliArgs {
  inputFile?: string;
  outputFile?: string;
  ratio: number;
  stats: boolean;
  help: boolean;
  format: 'text' | 'png' | 'jpeg';
  jpegQuality: number;
}

function printUsage(): void {
  console.log(`Usage: compress [OPTIONS] [INPUT_FILE]

Options:
  -r, --ratio <RATIO>        Compression ratio (0.0-1.0, default: 0.5)
  -o, --output <FILE>        Output file (default: stdout for text, compressed.png for image)
  -f, --format <FORMAT>      Output format: text, png, jpeg (default: text)
  --jpeg-quality <QUALITY>   JPEG quality (1-100, default: 85)
  -s, --stats                Show compression statistics
  -h, --help                 Show this help message

Examples:
  compress input.txt                          # Compress to stdout
  compress -r 0.7 input.txt                   # Conservative (70%)
  compress -r 0.3 input.txt                   # Aggressive (30%)
  compress -f png -o out.png input.txt        # Generate PNG image
  compress -f jpeg --jpeg-quality 90 in.txt   # Generate high-quality JPEG
  cat input.txt | compress                    # Read from stdin
  compress -s input.txt                       # Show statistics
`);
}

function parseArgs(argv: string[]): CliArgs {
  const args: CliArgs = {
    ratio: 0.5,
    stats: false,
    help: false,
    format: 'text',
    jpegQuality: 85,
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];

    switch (arg) {
      case '-h':
      case '--help':
        args.help = true;
        break;

      case '-r':
      case '--ratio':
        i++;
        if (i >= argv.length) {
          throw new Error('Missing value for --ratio');
        }
        args.ratio = parseFloat(argv[i]);
        if (isNaN(args.ratio) || args.ratio < 0 || args.ratio > 1) {
          throw new Error('Ratio must be between 0.0 and 1.0');
        }
        break;

      case '-o':
      case '--output':
        i++;
        if (i >= argv.length) {
          throw new Error('Missing value for --output');
        }
        args.outputFile = argv[i];
        break;

      case '-s':
      case '--stats':
        args.stats = true;
        break;

      case '-f':
      case '--format': {
        i++;
        if (i >= argv.length) {
          throw new Error('Missing value for --format');
        }
        const format = argv[i].toLowerCase();
        if (!['text', 'png', 'jpeg', 'jpg'].includes(format)) {
          throw new Error(`Invalid format '${format}'. Use: text, png, or jpeg`);
        }
        args.format = (format === 'jpg' ? 'jpeg' : format) as 'text' | 'png' | 'jpeg';
        break;
      }

      case '--jpeg-quality':
        i++;
        if (i >= argv.length) {
          throw new Error('Missing value for --jpeg-quality');
        }
        args.jpegQuality = parseInt(argv[i]);
        if (isNaN(args.jpegQuality) || args.jpegQuality < 1 || args.jpegQuality > 100) {
          throw new Error('JPEG quality must be between 1 and 100');
        }
        break;

      default:
        if (arg.startsWith('-')) {
          throw new Error(`Unknown option: ${arg}`);
        }
        args.inputFile = arg;
        break;
    }
  }

  return args;
}

async function readInput(args: CliArgs): Promise<string> {
  if (args.inputFile) {
    return fs.readFileSync(args.inputFile, 'utf8');
  } else {
    // Read from stdin
    const chunks: Buffer[] = [];
    for await (const chunk of process.stdin) {
      chunks.push(chunk);
    }
    return Buffer.concat(chunks).toString('utf8');
  }
}

function writeOutput(args: CliArgs, result: any): void {
  if (args.format === 'text') {
    // Text output
    if (args.outputFile) {
      fs.writeFileSync(args.outputFile, result.compressed, 'utf8');
      if (args.stats) {
        console.error(`Text saved to: ${args.outputFile}`);
      }
    } else {
      process.stdout.write(result.compressed);
    }
  } else {
    // Image output
    if (!result.imageData) {
      throw new Error('Image generation failed');
    }

    const outputPath =
      args.outputFile || (args.format === 'jpeg' ? 'compressed.jpg' : 'compressed.png');

    fs.writeFileSync(outputPath, result.imageData);

    if (args.stats) {
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
}

async function main() {
  try {
    const args = parseArgs(process.argv);

    if (args.help) {
      printUsage();
      process.exit(0);
    }

    const input = await readInput(args);

    const compressor = new Compressor({
      targetRatio: args.ratio,
      minInputBytes: 100,
      minInputTokens: 10,
    });

    // Determine output format
    const format = args.format === 'text' ? OutputFormat.TEXT : OutputFormat.IMAGE;

    const formatOptions = {
      imageFormat: args.format === 'jpeg' ? ('jpeg' as const) : ('png' as const),
      jpegQuality: args.jpegQuality,
    };

    const result = compressor.compressWithFormat(input, format, formatOptions);

    if (args.stats) {
      console.error('Compression Statistics:');
      console.error(`  Original tokens:   ${result.originalTokens}`);
      console.error(`  Compressed tokens: ${result.compressedTokens}`);
      console.error(`  Tokens removed:    ${result.tokensRemoved}`);
      console.error(`  Compression ratio: ${((1.0 - result.compressionRatio) * 100).toFixed(1)}%`);
      console.error(`  Target ratio:      ${((1.0 - args.ratio) * 100).toFixed(1)}%`);
      console.error('');
    }

    writeOutput(args, result);
  } catch (error) {
    if (error instanceof CompressionError) {
      console.error(`Compression error: ${error.message}`);
      process.exit(1);
    } else if (error instanceof Error) {
      console.error(`Error: ${error.message}`);
      printUsage();
      process.exit(1);
    } else {
      console.error('Unknown error occurred');
      process.exit(1);
    }
  }
}

main();
