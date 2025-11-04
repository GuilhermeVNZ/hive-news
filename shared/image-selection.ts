/**
 * Shared Image Selection Utility
 * 
 * This utility provides consistent image selection logic across all sites.
 * Used by both server-side (Next.js API routes, Vite plugins) and client-side code.
 * 
 * Logic (same as AIResearch):
 * 1. Try each category in order of priority
 * 2. Map category to image directory name
 * 3. Read actual image files from the directory
 * 4. Sort images by number in filename
 * 5. Extract numbers from articleId and use modulo with actual file count
 * 6. This ensures the same article always gets the same image and no repetitions
 * 
 * SERVER-SIDE USAGE (Recommended):
 * - Use async version: selectArticleImageAsync
 * - Reads actual files from disk
 * - More accurate, prevents image repetition
 * 
 * CLIENT-SIDE USAGE (Fallback):
 * - Use sync version: selectArticleImage
 * - Estimates based on common image counts
 * - Used when server doesn't provide image path
 */

import fs from 'fs/promises';
import path from 'path';

/**
 * Category mapping for image directories
 * Same mapping used across all sites (AIResearch, ScienceAI, etc.)
 */
export const IMAGE_CATEGORY_MAP: Record<string, string> = {
  'ai': 'ai',
  'robotics': 'robotics',
  'science': 'science',
  'coding': 'coding',
  'crypto': 'crypto',
  'database': 'database',
  'ethics': 'ethics',
  'games': 'games',
  'hardware': 'hardware',
  'legal': 'legal',
  'network': 'network',
  'security': 'security',
  'sound': 'sound',
  // ScienceAI specific mappings
  'nvidia': 'hardware',
  'openai': 'ai',
  'google': 'ai',
  'anthropic': 'ai',
  'deepseek': 'ai',
};

/**
 * Server-side image selection (async)
 * Reads actual files from disk - recommended for server-side code
 * 
 * @param categories - Array of image categories (priority order)
 * @param articleId - Unique article identifier
 * @param imagesBaseDir - Base directory containing image folders (e.g., News-main/images)
 * @returns Image path or undefined if no image found
 */
export async function selectArticleImageAsync(
  categories: string[],
  articleId: string,
  imagesBaseDir: string
): Promise<string | undefined> {
  try {
    // Try each category in order of priority
    for (const category of categories) {
      const categoryLower = category.toLowerCase();
      const imageDirName = IMAGE_CATEGORY_MAP[categoryLower] || categoryLower;
      const categoryDir = path.join(imagesBaseDir, imageDirName);
      
      try {
        const stats = await fs.stat(categoryDir);
        if (!stats.isDirectory()) continue;
        
        const files = await fs.readdir(categoryDir);
        const imageFiles = files.filter(f => /\.(jpg|jpeg|png|webp)$/i.test(f));
        
        if (imageFiles.length > 0) {
          // Sort by number in filename (same as AIResearch)
          imageFiles.sort((a, b) => {
            const numA = parseInt(a.match(/\d+/)?.[0] || '0');
            const numB = parseInt(b.match(/\d+/)?.[0] || '0');
            return numA - numB;
          });
          
          // Use article ID to select image (same logic as AIResearch)
          // Extract numbers from articleId and use modulo with actual file count
          const imageIndex = parseInt(articleId.split('.').pop()?.replace(/[^0-9]/g, '') || '0') % imageFiles.length;
          const selectedImage = imageFiles[imageIndex];
          
          return `/images/${imageDirName}/${selectedImage}`;
        }
      } catch (err) {
        continue;
      }
    }
  } catch (err) {
    console.error('Error selecting article image:', err);
  }
  
  return undefined;
}

/**
 * Client-side image selection (sync)
 * Estimates based on common image counts - used as fallback
 * 
 * @param imageCategories - Array of image categories (priority order)
 * @param articleId - Unique article identifier
 * @returns Image path (always returns a path, never undefined)
 */
export function selectArticleImage(
  imageCategories: string[] | undefined,
  articleId: string
): string {
  if (!imageCategories || imageCategories.length === 0) {
    // Default image
    return '/images/ai/ai_1.jpg';
  }

  // Try each category in order of priority
  for (const category of imageCategories) {
    const categoryLower = category.toLowerCase();
    const imageDir = IMAGE_CATEGORY_MAP[categoryLower] || categoryLower;
    
    // Calculate image index based on articleId
    const articleIdNumbers = articleId.split('.').pop()?.replace(/[^0-9]/g, '') || '';
    const numericId = parseInt(articleIdNumbers) || 0;
    
    // If no numbers found, use hash of the full articleId
    let imageIndex = numericId;
    if (imageIndex === 0) {
      const hash = articleId.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
      imageIndex = hash;
    }
    
    // Use modulo to select image (estimates based on common image counts)
    const maxImages = imageDir === 'ai' ? 34 : 20;
    const imageNumber = (imageIndex % maxImages) + 1;
    
    return `/images/${imageDir}/${imageDir}_${imageNumber}.jpg`;
  }

  // Fallback to default
  return '/images/ai/ai_1.jpg';
}





















