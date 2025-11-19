#!/usr/bin/env node
/**
 * Script para converter todas as imagens JPG/PNG para WebP
 * 
 * Uso: npm run convert-images
 * ou: npx tsx scripts/convert-images-to-webp.ts
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import sharp from 'sharp';

const IMAGES_DIR = path.join(__dirname, '..', 'images');
const SUPPORTED_FORMATS = ['.jpg', '.jpeg', '.png'];
const QUALITY = 85; // Qualidade WebP (0-100)

interface ConversionStats {
  converted: number;
  skipped: number;
  errors: number;
  totalSizeBefore: number;
  totalSizeAfter: number;
}

async function convertImageToWebP(
  inputPath: string,
  outputPath: string
): Promise<{ success: boolean; sizeBefore: number; sizeAfter: number }> {
  try {
    const stats = await fs.stat(inputPath);
    const sizeBefore = stats.size;

    // Converter para WebP
    await sharp(inputPath)
      .webp({ quality: QUALITY })
      .toFile(outputPath);

    const outputStats = await fs.stat(outputPath);
    const sizeAfter = outputStats.size;

    return { success: true, sizeBefore, sizeAfter };
  } catch (error) {
    console.error(`❌ Failed to convert ${inputPath}:`, error);
    return { success: false, sizeBefore: 0, sizeAfter: 0 };
  }
}

async function processDirectory(
  dirPath: string,
  stats: ConversionStats
): Promise<void> {
  try {
    const entries = await fs.readdir(dirPath, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dirPath, entry.name);

      if (entry.isDirectory()) {
        // Processar subdiretórios recursivamente
        await processDirectory(fullPath, stats);
      } else if (entry.isFile()) {
        const ext = path.extname(entry.name).toLowerCase();
        
        if (SUPPORTED_FORMATS.includes(ext)) {
          const webpPath = fullPath.replace(/\.(jpg|jpeg|png)$/i, '.webp');
          
          // Verificar se WebP já existe
          try {
            await fs.access(webpPath);
            console.log(`⏭️  Skipping ${fullPath} (WebP already exists)`);
            stats.skipped += 1;
          } catch {
            // WebP não existe, converter
            console.log(`🔄 Converting ${fullPath} → ${webpPath}`);
            const result = await convertImageToWebP(fullPath, webpPath);
            
            if (result.success) {
              stats.converted += 1;
              stats.totalSizeBefore += result.sizeBefore;
              stats.totalSizeAfter += result.sizeAfter;
              
              const savings = result.sizeBefore - result.sizeAfter;
              const savingsPercent = ((savings / result.sizeBefore) * 100).toFixed(1);
              console.log(`   ✅ Saved ${(savings / 1024).toFixed(1)} KB (${savingsPercent}%)`);
            } else {
              stats.errors += 1;
            }
          }
        }
      }
    }
  } catch (error) {
    console.error(`❌ Error processing directory ${dirPath}:`, error);
  }
}

async function main() {
  console.log('🖼️  Converting images to WebP format...\n');
  console.log(`📁 Images directory: ${IMAGES_DIR}\n`);

  const stats: ConversionStats = {
    converted: 0,
    skipped: 0,
    errors: 0,
    totalSizeBefore: 0,
    totalSizeAfter: 0,
  };

  try {
    await processDirectory(IMAGES_DIR, stats);

    console.log('\n' + '='.repeat(60));
    console.log('📊 Conversion Summary:');
    console.log('='.repeat(60));
    console.log(`✅ Converted: ${stats.converted} images`);
    console.log(`⏭️  Skipped: ${stats.skipped} images (WebP already exists)`);
    console.log(`❌ Errors: ${stats.errors} images`);
    
    if (stats.converted > 0) {
      const totalSavings = stats.totalSizeBefore - stats.totalSizeAfter;
      const totalSavingsMB = (totalSavings / (1024 * 1024)).toFixed(2);
      const savingsPercent = ((totalSavings / stats.totalSizeBefore) * 100).toFixed(1);
      
      console.log(`\n💾 Total size before: ${(stats.totalSizeBefore / (1024 * 1024)).toFixed(2)} MB`);
      console.log(`💾 Total size after: ${(stats.totalSizeAfter / (1024 * 1024)).toFixed(2)} MB`);
      console.log(`📉 Total savings: ${totalSavingsMB} MB (${savingsPercent}% reduction)`);
    }
    
    console.log('\n✅ Conversion complete!');
  } catch (error) {
    console.error('❌ Fatal error:', error);
    process.exit(1);
  }
}

main();





