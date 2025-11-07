#!/usr/bin/env node

/**
 * Example: Converting Paper to Compressed Images
 * 
 * This example demonstrates how to compress text and render it as images
 * for vision model consumption.
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { StatisticalFilter, StatisticalFilterConfig, OutputFormat } from '../src';

async function main() {
  console.log('📄 Converting Paper to Compressed Images\n');

  // Read original paper
  const paperPath = join(__dirname, '../../benchmarks/datasets/arxiv_markdown/1505.07818.md');
  
  if (!existsSync(paperPath)) {
    console.error(`❌ Paper file not found: ${paperPath}`);
    process.exit(1);
  }
  
  const text = readFileSync(paperPath, 'utf-8');

  console.log('📝 Original paper:');
  console.log(`  File: ${paperPath}`);
  console.log(`  Size: ${text.length} chars`);
  console.log(`  Words: ${text.split(/\s+/).length}`);
  console.log();

  const sep = '='.repeat(80);

  // Configuration for different compression levels
  const configs: Array<[number, string, string]> = [
    [0.3, 'aggressive', '30% compression'],
    [0.5, 'balanced', '50% compression (default)'],
    [0.7, 'light', '70% compression'],
  ];

  for (const [ratio, label, description] of configs) {
    console.log(sep);
    console.log(`Processing: ${label.toUpperCase()} - ${description}`);
    console.log(sep);

    const config: StatisticalFilterConfig = {
      compressionRatio: ratio,
      idfWeight: 0.3,
      positionWeight: 0.2,
      posWeight: 0.2,
      entityWeight: 0.2,
      entropyWeight: 0.1,
      enableProtectionMasks: true,
      enableContextualStopwords: true,
      preserveNegations: true,
      preserveComparators: true,
      domainTerms: [],
      minGapBetweenCritical: 3,
    };

    const filter = new StatisticalFilter(config);
    const compressed = filter.compress(text);
    
    // Calculate token estimates
    const originalTokens = Math.floor(text.length / 4);
    const compressedTokens = Math.floor(compressed.length / 4);
    const compressionRatio = compressedTokens / originalTokens;
    const tokensRemoved = originalTokens - compressedTokens;

    console.log('✅ Compression complete:');
    console.log(`  Original tokens: ${originalTokens}`);
    console.log(`  Compressed tokens: ${compressedTokens}`);
    console.log(`  Compression ratio: ${(compressionRatio * 100).toFixed(1)}%`);
    console.log(`  Tokens saved: ${tokensRemoved}`);
    console.log(`  Token savings: ${((1 - compressionRatio) * 100).toFixed(1)}%`);

    // Save compressed text
    const textFilename = `rnn_paper_${label}_compressed.txt`;
    writeFileSync(textFilename, compressed);
    console.log(`  📝 Text saved: ${textFilename}`);

    // Generate and save image
    try {
      const { ImageRenderer } = await import('../src/image-renderer');
      const renderer = new ImageRenderer();
      const imgData = renderer.renderToPng(compressed);
      
      const imgFilename = `rnn_paper_${label}_compressed.png`;
      writeFileSync(imgFilename, imgData);
      
      console.log(`  🖼️  Image saved: ${imgFilename}`);
      console.log(`     Size: ${Math.floor(imgData.length / 1024)} KB (${imgData.length} bytes)`);
      console.log('     Dimensions: 1024x1024 PNG');
      
      // Verify PNG signature
      if (imgData.length >= 8 && 
          imgData[0] === 137 && imgData[1] === 80 && imgData[2] === 78 && imgData[3] === 71 &&
          imgData[4] === 13 && imgData[5] === 10 && imgData[6] === 26 && imgData[7] === 10) {
        console.log('     ✓ Valid PNG signature');
      }

      // Calculate compression efficiency
      const charsPerKb = compressed.length / (imgData.length / 1024);
      console.log(`     Density: ${charsPerKb.toFixed(1)} chars/KB`);
    } catch (error) {
      console.error(`  ❌ Image rendering failed: ${error instanceof Error ? error.message : String(error)}`);
    }

    console.log();
  }

  console.log(sep);
  console.log('✅ Paper conversion complete!');
  console.log(sep);
  console.log();
  console.log('📊 Summary:');
  console.log(`  Input: ${paperPath}`);
  console.log("  Paper: 'On the difficulty of training Recurrent Neural Networks'");
  console.log('  Generated files:');
  console.log('    - rnn_paper_aggressive_compressed.txt + .png (30% compression)');
  console.log('    - rnn_paper_balanced_compressed.txt + .png (50% compression)');
  console.log('    - rnn_paper_light_compressed.txt + .png (70% compression)');
  console.log();
  console.log('💡 Use these PNG images with vision models for optical context compression!');
}

main().catch((error) => {
  console.error('Error:', error);
  process.exit(1);
});

