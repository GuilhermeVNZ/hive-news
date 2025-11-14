/**
 * Shared image selection utility
 * Uses the same logic as AIResearch to ensure consistent image selection across all sites
 * 
 * This function is used as a fallback when the server doesn't provide an image
 * The server-side selection (in vite-plugin-articles-api.ts) should be preferred
 * 
 * Logic (same as AIResearch):
 * 1. Try each category in order of priority
 * 2. Map category to image directory name
 * 3. Extract numbers from articleId and use modulo with available images
 * 4. This ensures the same article always gets the same image
 * 
 * NOTE: This is a simplified client-side version. The server-side version
 * in vite-plugin-articles-api.ts reads actual files and is more accurate.
 */
export function selectArticleImage(
  imageCategories: string[] | undefined,
  articleId: string
): string {
  if (!imageCategories || imageCategories.length === 0) {
    // Default image
    return '/images/ai/ai_1.jpg';
  }

  // Map category names to image directory names (same as AIResearch)
  const categoryMap: Record<string, string> = {
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
    'nvidia': 'hardware', // NVIDIA falls under hardware
    'openai': 'ai',
    'google': 'ai', // Google AI
    'anthropic': 'ai',
    'deepseek': 'ai',
  };

  // Try each category in order of priority (same logic as AIResearch)
  for (const category of imageCategories) {
    const categoryLower = category.toLowerCase();
    const imageDir = categoryMap[categoryLower] || categoryLower;
    
    // Calculate image index based on articleId (same logic as AIResearch)
    // Extract numbers from articleId and use modulo to select image
    // This ensures the same article always gets the same image
    const articleIdNumbers = articleId.split('.').pop()?.replace(/[^0-9]/g, '') || '';
    const numericId = parseInt(articleIdNumbers) || 0;
    
    // If no numbers found, use hash of the full articleId
    let imageIndex = numericId;
    if (imageIndex === 0) {
      // Fallback: create hash from articleId string
      const hash = articleId.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
      imageIndex = hash;
    }
    
    // Use modulo to select image (matching AIResearch's logic)
    // Common image counts: ai has 34, most others have 10-20
    // Using a reasonable default that works for most categories
    const maxImages = imageDir === 'ai' ? 34 : 20;
    const imageNumber = (imageIndex % maxImages) + 1;
    
    // Return the image path (same format as AIResearch)
    return `/images/${imageDir}/${imageDir}_${imageNumber}.jpg`;
  }

  // Fallback to default
  return '/images/ai/ai_1.jpg';
}
