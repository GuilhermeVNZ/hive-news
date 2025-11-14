import type { Plugin } from "vite";
import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WORKSPACE_ROOT = (() => {
  const baseDir = process.env.NEWS_BASE_DIR;
  if (!baseDir || baseDir.trim().length === 0) {
    throw new Error(
      "ENV NEWS_BASE_DIR must be set to the mounted data directory (e.g. /data) for ScienceAI",
    );
  }
  return baseDir;
})();

interface Article {
  id: string;
  slug: string;
  title: string;
  category: string;
  image?: string; // Image for feed (second category, non-repeating)
  imageCarousel?: string; // Image for carousel (first category, deterministic)
  imageArticle?: string; // Image for article detail (first category, deterministic)
  excerpt: string;
  content: string;
  date: string;
  author: string;
  readTime: number;
  featured?: boolean;
  imageCategories?: string[];
}

const ARTICLE_CONTENT_FILES = ["article.md", "article.txt"] as const;

// NEW APPROACH: Pool-based image selection for feed
// Format: { [category]: { [usageType]: { availableImages: string[], allImages: string[] } } }
interface ImagePool {
  availableImages: string[]; // Images not yet used in this cycle
  allImages: string[]; // All images in the directory
}

interface SelectImageOptions {
  categoryOverrideOrder?: string[];
  avoidImages?: Set<string>;
  allowDuplicates?: boolean;
}

const imagePools: Map<string, Map<string, ImagePool>> = new Map();

// Path to persistent tracker file
// Plugin is at: News-main/apps/frontend-next/scienceai/vite-plugin-articles-api.ts
// So we need to go up 3 levels to reach News-main root
const getTrackerFilePath = (): string => {
  const projectRoot = path.resolve(__dirname, "../../..");
  return path.join(projectRoot, ".image-tracker-scienceai.json");
};

// Load image pools from disk
async function loadImagePools(): Promise<void> {
  try {
    const trackerFile = getTrackerFilePath();
    console.log(`[Image Pool] Attempting to load from: ${trackerFile}`);

    const data = await fs.readFile(trackerFile, "utf-8");
    const poolData = JSON.parse(data);

    // Convert JSON back to Map structure
    let loadedCount = 0;
    for (const [category, usageTypes] of Object.entries(poolData)) {
      const categoryMap = new Map<string, ImagePool>();
      for (const [usageType, data] of Object.entries(usageTypes as any)) {
        // Check if it's old format (has usedImages) or new format (has availableImages)
        if ("usedImages" in data && !("availableImages" in data)) {
          // OLD FORMAT: Skip it, will be recreated with new format
          console.log(
            `[Image Pool] ⚠️  Skipping old format for ${category}/${usageType}, will recreate`,
          );
          continue;
        } else if ("availableImages" in data && "allImages" in data) {
          // NEW FORMAT: Use as is
          categoryMap.set(usageType, data as ImagePool);
          loadedCount++;
        } else {
          // Unknown format, skip
          console.warn(
            `[Image Pool] ⚠️  Unknown format for ${category}/${usageType}, skipping`,
          );
        }
      }
      if (categoryMap.size > 0) {
        imagePools.set(category, categoryMap);
      }
    }

    if (loadedCount > 0) {
      console.log(
        `[Image Pool] ✅ Loaded ${loadedCount} pools from ${trackerFile}`,
      );
    } else {
      console.log(
        `[Image Pool] ⚠️  No valid pools found in ${trackerFile}, will create new ones`,
      );
    }
  } catch (err: any) {
    // File doesn't exist yet, start with empty pools
    console.log(
      `[Image Pool] ⚠️  No existing pool file (${err?.message || "file not found"}), starting fresh`,
    );
  }
}

// Save image pools to disk
async function saveImagePools(): Promise<void> {
  try {
    const trackerFile = getTrackerFilePath();
    const poolData: any = {};

    // Convert Map to JSON-serializable object
    let savedCount = 0;
    for (const [category, usageTypes] of imagePools.entries()) {
      poolData[category] = {};
      for (const [usageType, pool] of usageTypes.entries()) {
        poolData[category][usageType] = pool;
        savedCount++;
      }
    }

    await fs.writeFile(trackerFile, JSON.stringify(poolData, null, 2), "utf-8");
    console.log(`[Image Pool] 💾 Saved ${savedCount} pools to ${trackerFile}`);
  } catch (err: any) {
    console.error(
      `[Image Pool] ❌ Error saving pools to ${getTrackerFilePath()}:`,
      err?.message || err,
    );
  }
}

async function readFirstExistingFile(
  directory: string,
  candidates: readonly string[],
): Promise<string> {
  for (const filename of candidates) {
    const fullPath = path.join(directory, filename);
    try {
      const content = await fs.readFile(fullPath, "utf-8");
      if (content.trim()) {
        return content;
      }
    } catch {
      // Try the next candidate
    }
  }
  return "";
}

// Get or create image pool for a category/usageType
// CRITICAL: This function must be synchronous for pool access to prevent race conditions
function getImagePoolSync(
  category: string,
  usageType: string,
  allImages: string[],
): ImagePool {
  // Use in-memory pool (loaded at server start)
  if (!imagePools.has(category)) {
    imagePools.set(category, new Map());
  }
  const categoryMap = imagePools.get(category)!;

  if (!categoryMap.has(usageType)) {
    // Create new pool with all images available
    const pool: ImagePool = {
      availableImages: [...allImages],
      allImages: [...allImages],
    };
    categoryMap.set(usageType, pool);
    console.log(
      `[Image Pool] 🆕 Created new pool for ${category}/${usageType} with ${allImages.length} images`,
    );
    return pool;
  }

  const pool = categoryMap.get(usageType)!;

  // Validate pool against actual images
  // If images changed, reset pool
  const allImagesSet = new Set(allImages);
  const poolAllImagesSet = new Set(pool.allImages);

  const imagesChanged =
    allImages.length !== pool.allImages.length ||
    !allImages.every((img) => poolAllImagesSet.has(img)) ||
    !pool.allImages.every((img) => allImagesSet.has(img));

  if (imagesChanged) {
    console.log(
      `[Image Pool] 🔄 Images changed for ${category}/${usageType}: ${pool.allImages.length} → ${allImages.length}, resetting pool`,
    );
    pool.availableImages = [...allImages];
    pool.allImages = [...allImages];
  } else {
    // Filter available images to only include images that still exist
    pool.availableImages = pool.availableImages.filter((img) =>
      allImagesSet.has(img),
    );

    // If pool is empty, refill it
    if (pool.availableImages.length === 0) {
      console.log(
        `[Image Pool] 🔄 Pool exhausted for ${category}/${usageType}, refilling with ${allImages.length} images`,
      );
      pool.availableImages = [...allImages];
    }
  }

  return pool;
}

type ImageUsageType = "feed" | "carousel" | "article";

// Helper function to normalize IDs for consistent matching
function normalizeId(id: string): string {
  return id.trim().toLowerCase().normalize("NFC");
}

// Helper function to check if a value is truthy (handles boolean, string, number)
function isTrue(value: any): boolean {
  if (typeof value === "boolean") return value;
  if (typeof value === "string") {
    const lower = value.toLowerCase().trim();
    return lower === "true" || lower === "1";
  }
  if (typeof value === "number") return value === 1;
  return false;
}

// Helper function to check if destinations array includes a specific site
function hasDestination(destinations: any, siteName: string): boolean {
  if (!destinations) return false;
  if (!Array.isArray(destinations)) return false;

  const siteLower = siteName.toLowerCase();
  return destinations.some((d: any) => {
    if (typeof d === "string") {
      return d.toLowerCase() === siteLower;
    }
    if (typeof d === "object" && d !== null) {
      // Check if it's an object with site_id or site_name
      const siteId = d.site_id?.toLowerCase();
      const siteNameField = d.site_name?.toLowerCase();
      return siteId === siteLower || siteNameField === siteLower;
    }
    return false;
  });
}

// Select article image based on usage type with duplicate avoidance and fallback support
async function selectArticleImage(
  categories: string[],
  articleId: string,
  imagesBaseDir: string,
  usageType: ImageUsageType = "article",
  options: SelectImageOptions = {},
): Promise<string | undefined> {
  try {
    const categoryMap: Record<string, string> = {
      ai: "ai",
      robotics: "robotics",
      science: "science",
      coding: "coding",
      crypto: "crypto",
      database: "database",
      ethics: "ethics",
      games: "games",
      hardware: "hardware",
      legal: "legal",
      network: "network",
      security: "security",
      sound: "sound",
      nvidia: "hardware",
      openai: "ai",
      google: "ai",
      anthropic: "ai",
      deepseek: "ai",
    };

    const sanitizedCategories = categories
      .map((category) => category.trim())
      .filter((category) => category.length > 0);

    const resolveCategoryDir = (categoryName?: string) => {
      if (!categoryName) {
        return null;
      }
      const categoryLower = categoryName.toLowerCase();
      const imageDirName = categoryMap[categoryLower] || categoryLower;
      return { categoryLower, imageDirName };
    };

    const determineCategoryOrder = (): string[] => {
      if (
        options.categoryOverrideOrder &&
        options.categoryOverrideOrder.length > 0
      ) {
        return options.categoryOverrideOrder
          .map((cat) => cat?.trim())
          .filter((cat): cat is string => !!cat && cat.length > 0);
      }

      if (usageType === "feed") {
        const order: string[] = [];
        if (sanitizedCategories[1]) order.push(sanitizedCategories[1]);
        if (sanitizedCategories[2]) order.push(sanitizedCategories[2]);
        if (sanitizedCategories[0]) order.push(sanitizedCategories[0]);
        return order;
      }

      if (sanitizedCategories[0]) {
        const order = [sanitizedCategories[0]];
        if (usageType !== "article" && sanitizedCategories[2]) {
          order.push(sanitizedCategories[2]);
        }
        return order;
      }

      return [];
    };

    const categoryOrder = determineCategoryOrder();

    if (usageType === "article") {
      const resolvedCategory = resolveCategoryDir(
        categoryOrder[0] ?? sanitizedCategories[0],
      );
      if (!resolvedCategory) {
        console.warn(
          `[Image Selection] No category available for article ${articleId.substring(0, 50)}...`,
        );
        return undefined;
      }

      const categoryDir = path.join(
        imagesBaseDir,
        resolvedCategory.imageDirName,
      );

      try {
        const stats = await fs.stat(categoryDir);
        if (!stats.isDirectory()) {
          console.warn(
            `[Image Selection] Category directory not found: ${categoryDir}`,
          );
          return undefined;
        }

        const files = await fs.readdir(categoryDir);
        const imageFiles = files
          .filter((f) => /\.(jpg|jpeg|png|webp)$/i.test(f))
          .sort((a, b) => {
            const numA = parseInt(a.match(/\d+/)?.[0] || "0");
            const numB = parseInt(b.match(/\d+/)?.[0] || "0");
            return numA - numB;
          });

        if (imageFiles.length === 0) {
          console.warn(
            `[Image Selection] No images found in category: ${categoryDir}`,
          );
          return undefined;
        }

        let hash = 5381;
        for (let i = 0; i < articleId.length; i++) {
          hash = (hash << 5) + hash + articleId.charCodeAt(i);
          hash = hash & hash;
        }

        const imageIndex = Math.abs(hash) % imageFiles.length;
        const selectedImage = imageFiles[imageIndex];

        console.log(
          `[Image Selection] ${usageType} - Article: ${articleId.substring(0, 50)}..., Category: ${resolvedCategory.imageDirName}, Hash: ${hash}, Index: ${imageIndex}/${imageFiles.length}, Image: ${selectedImage}`,
        );

        return `/images/${resolvedCategory.imageDirName}/${selectedImage}`;
      } catch (err) {
        console.error(
          `[Image Selection] Error reading category directory ${categoryDir}:`,
          err,
        );
        return undefined;
      }
    }

    if (categoryOrder.length === 0) {
      console.warn(
        `[Image Selection] No categories available for ${usageType} (${articleId.substring(0, 50)}...)`,
      );
      return undefined;
    }

    const avoidImages = options.avoidImages;
    const allowDuplicates = options.allowDuplicates === true;

    for (const categoryName of categoryOrder) {
      const resolvedCategory = resolveCategoryDir(categoryName);
      if (!resolvedCategory) {
        continue;
      }

      const categoryDir = path.join(
        imagesBaseDir,
        resolvedCategory.imageDirName,
      );

      let imageFiles: string[] = [];
      try {
        const stats = await fs.stat(categoryDir);
        if (!stats.isDirectory()) {
          console.warn(
            `[Image Selection] Category directory not found: ${categoryDir}`,
          );
          continue;
        }

        const files = await fs.readdir(categoryDir);
        imageFiles = files
          .filter((f) => /\.(jpg|jpeg|png|webp)$/i.test(f))
          .sort((a, b) => {
            const numA = parseInt(a.match(/\d+/)?.[0] || "0");
            const numB = parseInt(b.match(/\d+/)?.[0] || "0");
            return numA - numB;
          });
      } catch (err) {
        console.error(
          `[Image Selection] Error reading category directory ${categoryDir}:`,
          err,
        );
        continue;
      }

      if (imageFiles.length === 0) {
        console.warn(
          `[Image Selection] No images found in category: ${categoryDir}`,
        );
        continue;
      }

      const pool = getImagePoolSync(
        resolvedCategory.imageDirName,
        usageType,
        imageFiles,
      );

      if (pool.availableImages.length === 0) {
        pool.availableImages = [...imageFiles];
        console.log(
          `[Image Pool] 🔄 Pool refilled for ${resolvedCategory.imageDirName}/${usageType} with ${imageFiles.length} images`,
        );
      }

      const avoidFileNames = new Set<string>();
      if (!allowDuplicates && avoidImages && avoidImages.size > 0) {
        for (const avoidPath of avoidImages) {
          const parts = avoidPath.split("/");
          if (parts.length >= 4 && parts[2] === resolvedCategory.imageDirName) {
            avoidFileNames.add(parts[3]);
          }
        }
      }

      if (usageType === "feed") {
        let selectedIndex = -1;
        for (let i = 0; i < pool.availableImages.length; i += 1) {
          const img = pool.availableImages[i];
          if (!avoidFileNames.has(img)) {
            selectedIndex = i;
            break;
          }
        }

        if (selectedIndex === -1) {
          if (!allowDuplicates && avoidFileNames.size > 0) {
            console.log(
              `[Image Selection] ⚠️ All feed images recently used for ${resolvedCategory.imageDirName}, trying next category`,
            );
            continue;
          }

          if (pool.availableImages.length === 0) {
            continue;
          }

          selectedIndex = 0;
        }

        const [selectedImage] = pool.availableImages.splice(selectedIndex, 1);

        console.log(
          `[Image Selection] ✅ Feed - Article: ${articleId.substring(0, 50)}..., Category: ${resolvedCategory.imageDirName}, Image: ${selectedImage}, Remaining: ${pool.availableImages.length}/${imageFiles.length}`,
        );

        saveImagePools().catch((err) => {
          console.error(
            `[Image Pool] ❌ Error saving pool after selection:`,
            err,
          );
        });

        return `/images/${resolvedCategory.imageDirName}/${selectedImage}`;
      }

      let candidateList = [...pool.availableImages];

      if (avoidFileNames.size > 0) {
        candidateList = candidateList.filter((img) => !avoidFileNames.has(img));

        if (candidateList.length === 0) {
          const alternativeCandidates = pool.allImages.filter(
            (img) => !avoidFileNames.has(img),
          );
          if (alternativeCandidates.length === 0) {
            console.log(
              `[Image Selection] ⚠️ All images currently in use for ${resolvedCategory.imageDirName}/${usageType} when avoiding duplicates`,
            );
            continue;
          }

          pool.availableImages = [...alternativeCandidates];
          candidateList = [...pool.availableImages];
        }
      }

      if (candidateList.length === 0) {
        console.warn(
          `[Image Selection] ⚠️ No candidate images found in ${categoryDir}`,
        );
        continue;
      }

      const randomIndex = Math.floor(Math.random() * candidateList.length);
      const selectedImage = candidateList[randomIndex];

      const poolIndex = pool.availableImages.indexOf(selectedImage);
      if (poolIndex !== -1) {
        pool.availableImages.splice(poolIndex, 1);
      } else {
        pool.availableImages = pool.availableImages.filter(
          (img) => img !== selectedImage,
        );
      }

      const usageLabel =
        usageType === "feed"
          ? "Feed"
          : usageType === "carousel"
            ? "Carousel"
            : usageType;
      console.log(
        `[Image Selection] ✅ ${usageLabel} - Article: ${articleId.substring(0, 50)}..., Category: ${resolvedCategory.imageDirName}, Image: ${selectedImage}, Remaining: ${pool.availableImages.length}/${imageFiles.length}, allowDuplicates=${allowDuplicates}`,
      );

      saveImagePools().catch((err) => {
        console.error(
          `[Image Pool] ❌ Error saving pool after selection:`,
          err,
        );
      });

      return `/images/${resolvedCategory.imageDirName}/${selectedImage}`;
    }

    if (
      usageType !== "feed" &&
      !allowDuplicates &&
      avoidImages &&
      avoidImages.size > 0
    ) {
      console.warn(
        `[Image Selection] ⚠️ No unique image found for ${usageType} (${articleId.substring(0, 50)}...), retrying allowing duplicates`,
      );
      return selectArticleImage(
        categories,
        articleId,
        imagesBaseDir,
        usageType,
        {
          ...options,
          avoidImages: undefined,
          allowDuplicates: true,
          categoryOverrideOrder: categoryOrder,
        },
      );
    }

    console.warn(
      `[Image Selection] ⚠️ Unable to select image for ${usageType} (${articleId.substring(0, 50)}...)`,
    );
  } catch (err) {
    console.error("Error selecting article image:", err);
  }

  return undefined;
}

// Helper function to map source.txt to category
function mapSourceToCategory(
  sourceLower: string,
  imageCategories: string[],
  articleId: string,
): string {
  // Mapear source.txt para categorias válidas
  if (sourceLower.includes("openai") || sourceLower === "openai") {
    return "openai";
  } else if (sourceLower.includes("nvidia") || sourceLower === "nvidia") {
    return "nvidia";
  } else if (sourceLower.includes("google") || sourceLower === "google") {
    return "google";
  } else if (
    sourceLower.includes("meta") ||
    sourceLower === "meta" ||
    sourceLower.includes("facebook")
  ) {
    return "meta";
  } else if (
    sourceLower.includes("anthropic") ||
    sourceLower === "anthropic" ||
    sourceLower.includes("claude") ||
    sourceLower === "claude"
  ) {
    return "anthropic";
  } else if (sourceLower.includes("deepseek") || sourceLower === "deepseek") {
    return "deepseek";
  } else if (
    sourceLower.includes("x.ai") ||
    sourceLower === "x" ||
    sourceLower.includes("x.com") ||
    sourceLower.includes("grok")
  ) {
    return "x";
  } else if (sourceLower.includes("mistral") || sourceLower === "mistral") {
    return "mistral";
  } else if (
    sourceLower.includes("alibaba") ||
    sourceLower === "alibaba" ||
    sourceLower.includes("damo") ||
    sourceLower.includes("alizila")
  ) {
    return "alibaba";
  } else if (sourceLower.includes("microsoft") || sourceLower === "microsoft") {
    return "microsoft";
  } else if (
    sourceLower.includes("hivehub") ||
    sourceLower === "hivehub" ||
    sourceLower.includes("hive-hub")
  ) {
    return "hivehub";
  } else if (
    sourceLower.includes("perplexity") ||
    sourceLower === "perplexity"
  ) {
    return "perplexity";
  } else if (
    sourceLower.includes("huggingface") ||
    sourceLower === "huggingface" ||
    sourceLower.includes("hugging face")
  ) {
    return "huggingface";
  } else if (
    sourceLower.includes("stability") ||
    sourceLower === "stability" ||
    sourceLower.includes("stability ai")
  ) {
    return "stability";
  } else if (
    sourceLower.includes("elevenlabs") ||
    sourceLower === "elevenlabs" ||
    sourceLower.includes("eleven labs")
  ) {
    return "elevenlabs";
  } else if (
    sourceLower.includes("character.ai") ||
    sourceLower === "character" ||
    sourceLower.includes("character ai")
  ) {
    return "character";
  } else if (
    sourceLower.includes("inflection") ||
    sourceLower === "inflection" ||
    sourceLower.includes("pi ai")
  ) {
    return "inflection";
  } else if (
    sourceLower.includes("ibm") ||
    sourceLower === "ibm" ||
    sourceLower.includes("ibm research")
  ) {
    return "ibm";
  } else if (
    sourceLower.includes("apple") ||
    sourceLower === "apple" ||
    sourceLower.includes("machine learning journal")
  ) {
    return "apple";
  } else if (sourceLower.includes("intel") || sourceLower === "intel") {
    return "intel";
  } else if (sourceLower.includes("amd") || sourceLower === "amd") {
    return "amd";
  } else if (
    sourceLower.includes("salesforce") ||
    sourceLower === "salesforce"
  ) {
    return "salesforce";
  } else if (
    sourceLower.includes("stanford") ||
    sourceLower === "stanford" ||
    sourceLower.includes("sai")
  ) {
    return "stanford";
  } else if (
    sourceLower.includes("berkeley") ||
    sourceLower === "berkeley" ||
    sourceLower.includes("bair")
  ) {
    return "berkeley";
  } else if (sourceLower.includes("deepmind") || sourceLower === "deepmind") {
    return "deepmind";
  } else if (
    sourceLower.includes("techcrunch") ||
    sourceLower === "techcrunch"
  ) {
    return "techcrunch";
  } else if (
    sourceLower.includes("venturebeat") ||
    sourceLower === "venturebeat"
  ) {
    return "venturebeat";
  } else if (sourceLower.includes("the verge") || sourceLower === "verge") {
    return "verge";
  } else if (sourceLower.includes("wired") || sourceLower === "wired") {
    return "wired";
  } else if (
    sourceLower.includes("mit technology review") ||
    sourceLower === "mit" ||
    sourceLower.includes("technology review")
  ) {
    return "mit";
  } else if (sourceLower.includes("nature") || sourceLower === "nature") {
    return "nature";
  } else if (sourceLower.includes("science") || sourceLower === "science") {
    return "science";
  } else if (
    sourceLower.includes("menlo") ||
    sourceLower === "menlo" ||
    sourceLower.includes("menlo ventures")
  ) {
    return "menlo";
  }

  // Fallback: usar image_categories.txt ou articleId
  const firstCategory = imageCategories[0]?.toLowerCase();
  const articleIdLower = articleId.toLowerCase();
  const searchText = (firstCategory || "") + " " + articleIdLower;

  if (searchText.includes("nvidia")) return "nvidia";
  if (searchText.includes("openai")) return "openai";
  if (searchText.includes("google")) return "google";
  if (searchText.includes("meta") || searchText.includes("facebook"))
    return "meta";
  if (searchText.includes("anthropic") || searchText.includes("claude"))
    return "anthropic";
  if (searchText.includes("deepseek")) return "deepseek";
  if (
    searchText.includes("x.ai") ||
    searchText.includes("x.com") ||
    searchText.includes("grok")
  )
    return "x";
  if (searchText.includes("mistral")) return "mistral";
  if (
    searchText.includes("alibaba") ||
    searchText.includes("damo") ||
    searchText.includes("alizila")
  )
    return "alibaba";
  if (searchText.includes("microsoft")) return "microsoft";
  if (searchText.includes("hivehub") || searchText.includes("hive-hub"))
    return "hivehub";
  if (searchText.includes("perplexity")) return "perplexity";
  if (searchText.includes("huggingface") || searchText.includes("hugging face"))
    return "huggingface";
  if (searchText.includes("stability")) return "stability";
  if (searchText.includes("elevenlabs") || searchText.includes("eleven labs"))
    return "elevenlabs";
  if (
    searchText.includes("character.ai") ||
    searchText.includes("character ai")
  )
    return "character";
  if (searchText.includes("inflection") || searchText.includes("pi ai"))
    return "inflection";
  if (searchText.includes("ibm") || searchText.includes("ibm research"))
    return "ibm";
  if (
    searchText.includes("apple") ||
    searchText.includes("machine learning journal")
  )
    return "apple";
  if (searchText.includes("intel")) return "intel";
  if (searchText.includes("amd")) return "amd";
  if (searchText.includes("salesforce")) return "salesforce";
  if (searchText.includes("stanford") || searchText.includes("sai"))
    return "stanford";
  if (searchText.includes("berkeley") || searchText.includes("bair"))
    return "berkeley";
  if (searchText.includes("deepmind")) return "deepmind";
  if (searchText.includes("techcrunch")) return "techcrunch";
  if (searchText.includes("venturebeat")) return "venturebeat";
  if (searchText.includes("the verge") || searchText.includes("verge"))
    return "verge";
  if (searchText.includes("wired")) return "wired";
  if (searchText.includes("mit") || searchText.includes("technology review"))
    return "mit";
  if (searchText.includes("nature")) return "nature";
  if (searchText.includes("science")) return "science";
  if (searchText.includes("menlo") || searchText.includes("menlo ventures"))
    return "menlo";

  return "unknown";
}

async function readArticlesFromDir(
  outputDir: string,
  imagesBaseDir: string,
  registryPath?: string,
): Promise<Article[]> {
  // PRIMEIRO: Ler o registry para obter metadata de artigos publicados para ScienceAI
  const registryMap = new Map<string, any>(); // ID do registry -> metadata completo

  try {
    // Try multiple possible registry paths
    const possibleRegistryPaths = registryPath
      ? [registryPath]
      : [
          path.join(outputDir, "../../articles_registry.json"),
          path.join(outputDir, "../../../articles_registry.json"),
          path.join(outputDir, "../../../../articles_registry.json"),
          path.resolve(WORKSPACE_ROOT, "articles_registry.json"),
        ];

    let foundRegistryPath: string | null = null;
    let registryContent: string = "";

    for (const testPath of possibleRegistryPaths) {
      try {
        await fs.access(testPath);
        foundRegistryPath = testPath;
        registryContent = await fs.readFile(testPath, "utf-8");
        console.log(`[ScienceAI Articles] Reading registry from: ${testPath}`);
        break;
      } catch (err) {
        continue;
      }
    }

    if (foundRegistryPath && registryContent) {
      const registry = JSON.parse(registryContent);
      if (registry.articles) {
        for (const [id, meta] of Object.entries(registry.articles)) {
          const metadata = meta as any;

          // Verificar se metadata é válido (não null/undefined)
          if (!metadata || typeof metadata !== "object") {
            continue;
          }

          // Verificar se artigo está publicado e tem destino ScienceAI
          const isPublished = metadata.status === "Published";
          const hasScienceAIDest = hasDestination(
            metadata.destinations,
            "scienceai",
          );

          if (isPublished && hasScienceAIDest) {
            // Armazenar metadata completo do registry
            registryMap.set(id, metadata);
            // Também armazenar por ID normalizado para lookup
            const normalizedId = normalizeId(id);
            if (normalizedId !== id) {
              registryMap.set(normalizedId, metadata);
            }
          }
        }
        console.log(
          `[ScienceAI Articles] Found ${Math.floor(registryMap.size / 2)} unique articles in registry for ScienceAI`,
        );
      }
    }
  } catch (err: any) {
    console.error(
      "[ScienceAI Articles] ⚠️  Error reading registry:",
      err?.message || err,
    );
  }

  // SEGUNDO: Ler diretórios do filesystem (output/ScienceAI/)
  const featuredArticles: Article[] = [];
  const otherArticles: Article[] = [];
  const usedHomepageImages = new Set<string>(); // Track images already used on homepage

  console.log(`[ScienceAI Articles] Scanning filesystem: ${outputDir}`);

  // Verificar se diretório existe
  try {
    const stats = await fs.stat(outputDir);
    if (!stats.isDirectory()) {
      console.error(
        `[ScienceAI Articles] OutputDir is not a directory: ${outputDir}`,
      );
      return [];
    }
  } catch (err) {
    console.error(
      `[ScienceAI Articles] OutputDir does not exist: ${outputDir}`,
      err,
    );
    return [];
  }

  try {
    const dirs = await fs.readdir(outputDir);
    console.log(
      `[ScienceAI Articles] Found ${dirs.length} items in filesystem:`,
      dirs.slice(0, 5),
    );

    for (const dir of dirs) {
      const dirPath = path.join(outputDir, dir);

      try {
        const stats = await fs.stat(dirPath);
        if (!stats.isDirectory()) {
          continue;
        }

        // Tentar encontrar o ID do artigo no registry
        // O ID pode ser o nome da pasta ou pode estar no registry pelo output_dir
        let articleMetadata: any = null;
        let articleId = dir;

        // Procurar no registry pelo ID da pasta
        if (registryMap.has(dir)) {
          articleMetadata = registryMap.get(dir);
          articleId = dir;
        } else {
          // Procurar no registry pelo ID normalizado
          const normalizedDir = normalizeId(dir);
          if (registryMap.has(normalizedDir)) {
            articleMetadata = registryMap.get(normalizedDir);
            articleId = normalizedDir;
          } else {
            // Procurar no registry pelo output_dir
            for (const [id, meta] of registryMap.entries()) {
              if (normalizeId(id) !== id) continue; // Skip normalized IDs
              if (!meta.output_dir) continue;

              let metaOutputDir = meta.output_dir;
              if (typeof metaOutputDir === "string") {
                metaOutputDir = metaOutputDir
                  .replace(/\\\\/g, "\\")
                  .replace(/\//g, "\\");
                if (
                  metaOutputDir.toLowerCase().endsWith(dirPath.toLowerCase()) ||
                  metaOutputDir.toLowerCase().includes(dir.toLowerCase())
                ) {
                  articleMetadata = meta;
                  articleId = id;
                  break;
                }
              }
            }
          }
        }

        // Se encontrou no registry, verificar se está hidden
        if (articleMetadata) {
          const isHidden =
            articleMetadata.hidden !== undefined &&
            articleMetadata.hidden !== null &&
            isTrue(articleMetadata.hidden);
          if (isHidden) {
            console.log(`[ScienceAI Articles] Skipping hidden article: ${dir}`);
            continue;
          }
        }

        // Tentar ler arquivos do artigo
        const titlePath = path.join(dirPath, "title.txt");

        let title = "";
        let articleContent = "";

        try {
          [title, articleContent] = await Promise.all([
            fs.readFile(titlePath, "utf-8").catch(() => ""),
            readFirstExistingFile(dirPath, ARTICLE_CONTENT_FILES),
          ]);
        } catch (err) {
          console.warn(
            `[ScienceAI Articles] Error reading files for ${dir}:`,
            err,
          );
          continue;
        }

        if (!title || !articleContent) {
          console.warn(
            `[ScienceAI Articles] Missing title or content for ${dir}`,
          );
          continue;
        }

        // Ler arquivos restantes
        const sourcePath = path.join(dirPath, "source.txt");
        const subtitlePath = path.join(dirPath, "subtitle.txt");
        const categoriesPath = path.join(dirPath, "image_categories.txt");
        const slugPath = path.join(dirPath, "slug.txt");

        let sourceContent = "";
        let subtitle = "";
        let categoriesContent = "";
        let slugContent = "";

        try {
          [sourceContent, subtitle, categoriesContent, slugContent] =
            await Promise.all([
              fs.readFile(sourcePath, "utf-8").catch(() => ""),
              fs.readFile(subtitlePath, "utf-8").catch(() => ""),
              fs.readFile(categoriesPath, "utf-8").catch(() => ""),
              fs.readFile(slugPath, "utf-8").catch(() => ""),
            ]);
        } catch (err) {
          // Continuar mesmo se alguns arquivos não existirem
        }

        // Gerar slug
        let slug = slugContent.trim();
        if (!slug && title) {
          slug = title
            .toLowerCase()
            .replace(/[^\w\s-]/g, "")
            .replace(/\s+/g, "-")
            .replace(/-+/g, "-")
            .replace(/^-|-$/g, "");
        }

        // Extrair excerpt
        const excerpt =
          subtitle.trim() ||
          articleContent
            .split("\n")
            .filter((line) => line.trim())
            .slice(0, 3)
            .join(" ")
            .substring(0, 200) + "...";

        // Ler image categories
        const imageCategories = categoriesContent
          .split("\n")
          .filter((c) => c.trim());

        // Determinar categoria do source.txt
        const sourceLower = sourceContent.trim().toLowerCase();
        const category = mapSourceToCategory(
          sourceLower,
          imageCategories,
          articleId,
        );

        // Verificar se é featured (do registry ou false por padrão)
        const isFeatured = articleMetadata
          ? articleMetadata.featured !== undefined &&
            articleMetadata.featured !== null &&
            isTrue(articleMetadata.featured)
          : false;

        // Select images with duplicate avoidance and fallback category logic
        const carouselCategoryOrder: string[] = [];
        if (imageCategories[0]) {
          carouselCategoryOrder.push(imageCategories[0]);
        }
        if (imageCategories[2]) {
          carouselCategoryOrder.push(imageCategories[2]);
        }

        const carouselOptions: SelectImageOptions = {
          avoidImages: usedHomepageImages,
        };
        if (carouselCategoryOrder.length > 0) {
          carouselOptions.categoryOverrideOrder = carouselCategoryOrder;
        }

        const imageCarousel = await selectArticleImage(
          imageCategories,
          articleId,
          imagesBaseDir,
          "carousel",
          carouselOptions,
        );

        if (imageCarousel) {
          usedHomepageImages.add(imageCarousel);
        }

        const articleOptions: SelectImageOptions = {};
        if (carouselCategoryOrder.length > 0) {
          articleOptions.categoryOverrideOrder = [carouselCategoryOrder[0]];
        }

        const imageArticle = await selectArticleImage(
          imageCategories,
          articleId,
          imagesBaseDir,
          "article",
          articleOptions,
        );

        const feedCategoryOrder: string[] = [];
        if (imageCategories[1]) {
          feedCategoryOrder.push(imageCategories[1]);
        } else if (imageCategories[0]) {
          feedCategoryOrder.push(imageCategories[0]);
        }
        if (imageCategories[2]) {
          feedCategoryOrder.push(imageCategories[2]);
        }

        const feedOptions: SelectImageOptions = {
          avoidImages: usedHomepageImages,
        };
        if (feedCategoryOrder.length > 0) {
          feedOptions.categoryOverrideOrder = feedCategoryOrder;
        }

        const imageFeed = await selectArticleImage(
          imageCategories,
          articleId,
          imagesBaseDir,
          "feed",
          feedOptions,
        );

        if (imageFeed) {
          usedHomepageImages.add(imageFeed);
        }

        // Usar published_at do registry, ou fallback para mtime do diretório
        let date = "";
        if (articleMetadata && articleMetadata.published_at) {
          date = new Date(articleMetadata.published_at)
            .toISOString()
            .split("T")[0];
        } else {
          try {
            const dirStats = await fs.stat(dirPath);
            date = dirStats.mtime.toISOString().split("T")[0];
          } catch {
            date = new Date().toISOString().split("T")[0];
          }
        }

        const article: Article = {
          id: articleId,
          slug: slug || articleId,
          title: title.trim(),
          excerpt,
          content: articleContent,
          date,
          author: sourceContent.trim() || "ScienceAI Team",
          category,
          readTime: Math.ceil(articleContent.split(" ").length / 200),
          imageCategories,
          image: imageFeed || "/images/ai/ai_1.jpg",
          imageCarousel: imageCarousel || "/images/ai/ai_1.jpg",
          imageArticle: imageArticle || "/images/ai/ai_1.jpg",
          featured: isFeatured,
        };

        if (isFeatured) {
          featuredArticles.push(article);
        } else {
          otherArticles.push(article);
        }
      } catch (err) {
        console.error(
          `[ScienceAI Articles] Error processing article ${dir}:`,
          err,
        );
      }
    }
  } catch (err) {
    console.error(`[ScienceAI Articles] Error reading filesystem:`, err);
  }

  // Ordenar featured por published_at (mais recente primeiro)
  featuredArticles.sort((a, b) => {
    const dateA = new Date(a.date).getTime();
    const dateB = new Date(b.date).getTime();
    return dateB - dateA; // Most recent first
  });

  // Ordenar outros por published_at (mais recente primeiro)
  otherArticles.sort((a, b) => {
    const dateA = new Date(a.date).getTime();
    const dateB = new Date(b.date).getTime();
    return dateB - dateA; // Most recent first
  });

  // Combinar: featured primeiro, depois outros
  const allArticles = [...featuredArticles, ...otherArticles];

  console.log(
    `[ScienceAI Articles] Featured: ${featuredArticles.length}, Other: ${otherArticles.length}, Total: ${allArticles.length}`,
  );

  return allArticles;
}

export function articlesApiPlugin(): Plugin {
  return {
    name: "vite-plugin-articles-api",
    async configureServer(server) {
      // Get project root directory (News-main)
      // __dirname points to the directory containing this plugin file
      // Plugin is at: News-main/apps/frontend-next/scienceai/vite-plugin-articles-api.ts
      // So we need to go up 3 levels to reach News-main
      const projectRoot = WORKSPACE_ROOT;

      console.log(
        "[Articles API Plugin] Project root (WORKSPACE_ROOT):",
        projectRoot,
      );

      // Load persistent image pools on server start
      await loadImagePools();
      console.log("[Articles API Plugin] Plugin dirname:", __dirname);
      console.log("[Articles API Plugin] Process cwd:", process.cwd());

      // Serve images from News-main/images directory
      server.middlewares.use("/images", async (req, res, next) => {
        if (!req.url) {
          next();
          return;
        }

        try {
          const imagesPath = path.join(projectRoot, "images");
          const filePath = path.join(imagesPath, req.url.replace(/^\//, ""));

          const stats = await fs.stat(filePath);
          if (stats.isFile()) {
            const ext = path.extname(filePath).toLowerCase();
            const contentType: Record<string, string> = {
              ".jpg": "image/jpeg",
              ".jpeg": "image/jpeg",
              ".png": "image/png",
              ".gif": "image/gif",
              ".webp": "image/webp",
            };

            res.setHeader("Content-Type", contentType[ext] || "image/jpeg");
            const fileContent = await fs.readFile(filePath);
            res.end(fileContent);
          } else {
            next();
          }
        } catch (error) {
          // Image not found, continue to next middleware
          next();
        }
      });

      // Categories API - retorna as 5 categorias com conteúdo mais recente
      server.middlewares.use(async (req, res, next) => {
        if (req.url === "/api/categories") {
          try {
            const baseOutputDir = path.join(projectRoot, "output", "ScienceAI");
            const imagesBaseDir = path.join(projectRoot, "images");
            const registryPath = path.join(
              projectRoot,
              "articles_registry.json",
            );

            // readArticlesFromDir já lê o registry e aplica filtros (destinations=scienceai, hidden=false)
            // Também já marca featured=true do registry
            const articles = await readArticlesFromDir(
              baseOutputDir,
              imagesBaseDir,
              registryPath,
            );

            // Ordenar: featured primeiro, depois por data (mais recente primeiro)
            articles.sort((a, b) => {
              const dateDiff =
                new Date(b.date).getTime() - new Date(a.date).getTime();
              if (dateDiff !== 0) {
                return dateDiff;
              }
              const aFeatured = (a as any).featured || false;
              const bFeatured = (b as any).featured || false;
              if (aFeatured !== bFeatured) {
                return aFeatured ? -1 : 1;
              }
              return String(b.id).localeCompare(String(a.id));
            });

            // Agrupar por categoria e encontrar a data mais recente de cada categoria
            const categoryMap = new Map<
              string,
              { name: string; slug: string; latestDate: string; icon: string }
            >();

            // Mapeamento de slugs para nomes e ícones
            const categoryInfo: Record<string, { name: string; icon: string }> =
              {
                nvidia: { name: "NVIDIA", icon: "Cpu" },
                openai: { name: "OpenAI", icon: "Brain" },
                google: { name: "Google", icon: "Search" },
                anthropic: { name: "Anthropic", icon: "Sparkles" },
                deepseek: { name: "DeepSeek", icon: "Target" },
                meta: { name: "Meta", icon: "Sparkles" },
                x: { name: "X", icon: "MessageSquare" },
                mistral: { name: "Mistral", icon: "Wind" },
                alibaba: { name: "Alibaba", icon: "Package" },
                microsoft: { name: "Microsoft", icon: "Monitor" },
                hivehub: { name: "HiveHub", icon: "Home" },
                perplexity: { name: "Perplexity", icon: "Search" },
                huggingface: { name: "Hugging Face", icon: "Users" },
                stability: { name: "Stability AI", icon: "Image" },
                elevenlabs: { name: "ElevenLabs", icon: "Volume2" },
                character: { name: "Character.AI", icon: "MessageSquare" },
                inflection: { name: "Inflection AI", icon: "Sparkles" },
                ibm: { name: "IBM Research", icon: "Database" },
                apple: { name: "Apple ML", icon: "Laptop" },
                intel: { name: "Intel AI", icon: "Cpu" },
                amd: { name: "AMD AI", icon: "Cpu" },
                salesforce: { name: "Salesforce AI", icon: "Briefcase" },
                stanford: { name: "Stanford AI", icon: "GraduationCap" },
                berkeley: { name: "Berkeley AI", icon: "GraduationCap" },
                deepmind: { name: "DeepMind", icon: "Brain" },
                techcrunch: { name: "TechCrunch", icon: "Newspaper" },
                venturebeat: { name: "VentureBeat", icon: "Newspaper" },
                verge: { name: "The Verge", icon: "Newspaper" },
                wired: { name: "Wired", icon: "Newspaper" },
                mit: { name: "MIT Tech Review", icon: "BookOpen" },
                nature: { name: "Nature", icon: "BookOpen" },
                science: { name: "Science", icon: "BookOpen" },
                menlo: { name: "Menlo Ventures", icon: "TrendingUp" },
                unknown: { name: "Technology", icon: "Circle" },
                technology: { name: "Technology", icon: "Circle" },
              };

            // PRIMEIRO: Encontrar categorias das featured news (prioridade no topo)
            const featuredArticles = articles.filter(
              (a: Article) => (a as any).featured === true,
            );
            const featuredCategories = new Set<string>();

            // Coletar categorias das featured (sem repetir)
            for (const article of featuredArticles) {
              const categorySlug = article.category.toLowerCase();
              if (!featuredCategories.has(categorySlug)) {
                featuredCategories.add(categorySlug);

                if (!categoryMap.has(categorySlug)) {
                  const info = categoryInfo[categorySlug] || {
                    name:
                      categorySlug.charAt(0).toUpperCase() +
                      categorySlug.slice(1),
                    icon: "Circle",
                  };
                  categoryMap.set(categorySlug, {
                    name: info.name,
                    slug: categorySlug,
                    latestDate: article.date,
                    icon: info.icon,
                  });
                } else {
                  const existing = categoryMap.get(categorySlug)!;
                  // Atualizar se este artigo for mais recente
                  if (
                    new Date(article.date).getTime() >
                    new Date(existing.latestDate).getTime()
                  ) {
                    existing.latestDate = article.date;
                  }
                }
              }
            }

            // SEGUNDO: Encontrar categorias dos demais artigos (mais recentes)
            // Limitar a 5 categorias no total (featured + recentes)
            const maxCategories = 5;
            const currentCategoryCount = categoryMap.size;

            // Ordenar artigos por data (mais recente primeiro)
            const sortedArticles = [...articles].sort((a, b) => {
              const dateA = new Date(a.date).getTime();
              const dateB = new Date(b.date).getTime();
              return dateB - dateA; // Most recent first
            });

            // Adicionar categorias dos demais artigos até completar 5 (sem repetir)
            for (const article of sortedArticles) {
              if (categoryMap.size >= maxCategories) break;

              const categorySlug = article.category.toLowerCase();

              // Só adicionar se ainda não está no mapa (não repetir)
              if (!categoryMap.has(categorySlug)) {
                const info = categoryInfo[categorySlug] || {
                  name:
                    categorySlug.charAt(0).toUpperCase() +
                    categorySlug.slice(1),
                  icon: "Circle",
                };
                categoryMap.set(categorySlug, {
                  name: info.name,
                  slug: categorySlug,
                  latestDate: article.date,
                  icon: info.icon,
                });
              } else {
                // Atualizar se este artigo for mais recente
                const existing = categoryMap.get(categorySlug)!;
                if (
                  new Date(article.date).getTime() >
                  new Date(existing.latestDate).getTime()
                ) {
                  existing.latestDate = article.date;
                }
              }
            }

            // Converter para array e ordenar: featured primeiro, depois por data mais recente
            const categoriesArray = Array.from(categoryMap.values()).sort(
              (a, b) => {
                const aIsFeatured = featuredCategories.has(a.slug);
                const bIsFeatured = featuredCategories.has(b.slug);

                // Featured categories primeiro
                if (aIsFeatured !== bIsFeatured) {
                  return aIsFeatured ? -1 : 1;
                }

                // Depois ordenar por data mais recente
                return (
                  new Date(b.latestDate).getTime() -
                  new Date(a.latestDate).getTime()
                );
              },
            );

            // Retornar apenas as 5 (garantir máximo 5, sem repetir)
            const topCategories = categoriesArray.slice(0, maxCategories);

            res.setHeader("Content-Type", "application/json");
            res.setHeader("Access-Control-Allow-Origin", "*");
            // Disable caching to ensure fresh data
            res.setHeader(
              "Cache-Control",
              "no-store, no-cache, must-revalidate, proxy-revalidate",
            );
            res.setHeader("Pragma", "no-cache");
            res.setHeader("Expires", "0");
            res.end(JSON.stringify({ categories: topCategories }));
          } catch (error) {
            console.error("Error in categories API:", error);
            res.statusCode = 500;
            res.end(JSON.stringify({ error: "Failed to fetch categories" }));
          }
        } else if (req.url?.startsWith("/api/articles")) {
          try {
            const baseOutputDir = path.join(projectRoot, "output", "ScienceAI");
            const imagesBaseDir = path.join(projectRoot, "images");
            const registryPath = path.join(
              projectRoot,
              "articles_registry.json",
            );

            // readArticlesFromDir já lê o registry e aplica filtros (destinations=scienceai, hidden=false)
            // Também já marca featured=true do registry
            const articles = await readArticlesFromDir(
              baseOutputDir,
              imagesBaseDir,
              registryPath,
            );

            // Log featured articles
            const featuredArticles = articles.filter(
              (a: Article) => (a as any).featured === true,
            );
            console.log(
              `[ScienceAI Articles API] Total articles: ${articles.length}, Featured: ${featuredArticles.length}`,
            );
            if (featuredArticles.length > 0) {
              console.log(
                `[ScienceAI Articles API] Featured articles:`,
                featuredArticles.map((a: Article) => ({
                  id: a.id,
                  title: a.title.substring(0, 50),
                  category: a.category,
                })),
              );
            }

            // Ordenar: featured primeiro, depois por data (mais recente primeiro)
            articles.sort((a, b) => {
              const dateDiff =
                new Date(b.date).getTime() - new Date(a.date).getTime();
              if (dateDiff !== 0) {
                return dateDiff;
              }
              const aFeatured = (a as any).featured || false;
              const bFeatured = (b as any).featured || false;
              if (aFeatured !== bFeatured) {
                return aFeatured ? -1 : 1;
              }
              return String(b.id).localeCompare(String(a.id));
            });

            // Images are already selected in readArticlesFromDir with proper logic:
            // - image: for feed (second category, non-repeating)
            // - imageCarousel: for carousel (first category, deterministic)
            // - imageArticle: for article detail (first category, deterministic)
            // No need for additional image processing here

            res.setHeader("Content-Type", "application/json");
            res.setHeader("Access-Control-Allow-Origin", "*");
            // Disable caching to ensure fresh data
            res.setHeader(
              "Cache-Control",
              "no-store, no-cache, must-revalidate, proxy-revalidate",
            );
            res.setHeader("Pragma", "no-cache");
            res.setHeader("Expires", "0");
            res.end(JSON.stringify({ articles }));
          } catch (error) {
            console.error("Error in articles API:", error);
            res.statusCode = 500;
            res.end(JSON.stringify({ error: "Failed to fetch articles" }));
          }
        } else if (req.url?.startsWith("/api/article/")) {
          // API para artigo individual - busca pelo slug
          try {
            const requestedSlug = decodeURIComponent(
              req.url.replace("/api/article/", ""),
            );
            const baseOutputDir = path.join(projectRoot, "output", "ScienceAI");
            const imagesBaseDir = path.join(projectRoot, "images");

            // Procurar pasta do artigo pelo slug
            let articleDir: string | null = null;
            let articleId: string | null = null;

            try {
              const dirs = await fs.readdir(baseOutputDir);

              for (const dir of dirs) {
                const dirPath = path.join(baseOutputDir, dir);
                const stats = await fs.stat(dirPath);
                if (!stats.isDirectory()) continue;

                if (!articleDir && dir === requestedSlug) {
                  articleDir = dirPath;
                  articleId = dir;
                  break;
                }

                // Ler slug.txt da pasta
                const slugPath = path.join(dirPath, "slug.txt");
                try {
                  const slugContent = await fs.readFile(slugPath, "utf-8");
                  const slug = slugContent.trim();

                  // Se slug corresponder, ou se slug não existir, gerar slug do título e comparar
                  if (slug === requestedSlug) {
                    articleDir = dirPath;
                    articleId = dir;
                    break;
                  }
                } catch {
                  // Se slug.txt não existir, ler título e gerar slug para comparar
                  const titlePath = path.join(dirPath, "title.txt");
                  try {
                    const titleContent = await fs.readFile(titlePath, "utf-8");
                    const title = titleContent.trim();
                    if (title) {
                      const generatedSlug = title
                        .toLowerCase()
                        .replace(/[^\w\s-]/g, "")
                        .replace(/\s+/g, "-")
                        .replace(/-+/g, "-")
                        .replace(/^-|-$/g, "");

                      if (generatedSlug === requestedSlug) {
                        articleDir = dirPath;
                        articleId = dir;
                        break;
                      }
                    }
                  } catch {
                    // Continuar procurando
                  }
                }
              }
            } catch (err) {
              console.error("Error searching for article by slug:", err);
            }

            if (!articleDir || !articleId) {
              res.statusCode = 404;
              res.end(JSON.stringify({ error: "Article not found" }));
              return;
            }

            const titlePath = path.join(articleDir, "title.txt");
            const subtitlePath = path.join(articleDir, "subtitle.txt");
            const categoriesPath = path.join(
              articleDir,
              "image_categories.txt",
            );
            const sourcePath = path.join(articleDir, "source.txt");
            const slugPath = path.join(articleDir, "slug.txt");

            const [
              title,
              subtitle,
              categoriesContent,
              sourceContent,
              slugContent,
              dirStats,
            ] = await Promise.all([
              fs.readFile(titlePath, "utf-8").catch(() => ""),
              fs.readFile(subtitlePath, "utf-8").catch(() => ""),
              fs.readFile(categoriesPath, "utf-8").catch(() => ""),
              fs.readFile(sourcePath, "utf-8").catch(() => ""),
              fs.readFile(slugPath, "utf-8").catch(() => ""),
              fs.stat(articleDir).catch(() => null),
            ]);

            const articleContent = await readFirstExistingFile(
              articleDir,
              ARTICLE_CONTENT_FILES,
            );

            if (!title || !articleContent) {
              res.statusCode = 404;
              res.end(JSON.stringify({ error: "Article not found" }));
              return;
            }

            const imageCategories = categoriesContent
              .split("\n")
              .filter((c) => c.trim());

            // Determinar categoria baseada no source.txt (prioritário)
            let category = "unknown";
            const sourceLower = sourceContent.trim().toLowerCase();

            // Mapear source.txt para categorias válidas (todas as possibilidades do backend)
            // IMPORTANTE: Claude pertence à Anthropic, não OpenAI!
            if (sourceLower.includes("openai") || sourceLower === "openai") {
              category = "openai";
            } else if (
              sourceLower.includes("nvidia") ||
              sourceLower === "nvidia"
            ) {
              category = "nvidia";
            } else if (
              sourceLower.includes("google") ||
              sourceLower === "google"
            ) {
              category = "google";
            } else if (
              sourceLower.includes("meta") ||
              sourceLower === "meta" ||
              sourceLower.includes("facebook")
            ) {
              category = "meta";
            } else if (
              sourceLower.includes("anthropic") ||
              sourceLower === "anthropic" ||
              sourceLower.includes("claude") ||
              sourceLower === "claude"
            ) {
              category = "anthropic"; // Claude é da Anthropic!
            } else if (
              sourceLower.includes("deepseek") ||
              sourceLower === "deepseek"
            ) {
              category = "deepseek";
            } else if (
              sourceLower.includes("x.ai") ||
              sourceLower === "x" ||
              sourceLower.includes("x.com") ||
              sourceLower.includes("grok")
            ) {
              category = "x";
            } else if (
              sourceLower.includes("mistral") ||
              sourceLower === "mistral"
            ) {
              category = "mistral";
            } else if (
              sourceLower.includes("alibaba") ||
              sourceLower === "alibaba" ||
              sourceLower.includes("damo") ||
              sourceLower.includes("alizila")
            ) {
              category = "alibaba";
            } else if (
              sourceLower.includes("microsoft") ||
              sourceLower === "microsoft"
            ) {
              category = "microsoft";
            } else if (
              sourceLower.includes("hivehub") ||
              sourceLower === "hivehub" ||
              sourceLower.includes("hive-hub")
            ) {
              category = "hivehub";
            } else if (
              sourceLower.includes("perplexity") ||
              sourceLower === "perplexity"
            ) {
              category = "perplexity";
            } else if (
              sourceLower.includes("huggingface") ||
              sourceLower === "huggingface" ||
              sourceLower.includes("hugging face")
            ) {
              category = "huggingface";
            } else if (
              sourceLower.includes("stability") ||
              sourceLower === "stability" ||
              sourceLower.includes("stability ai")
            ) {
              category = "stability";
            } else if (
              sourceLower.includes("elevenlabs") ||
              sourceLower === "elevenlabs" ||
              sourceLower.includes("eleven labs")
            ) {
              category = "elevenlabs";
            } else if (
              sourceLower.includes("character.ai") ||
              sourceLower === "character" ||
              sourceLower.includes("character ai")
            ) {
              category = "character";
            } else if (
              sourceLower.includes("inflection") ||
              sourceLower === "inflection" ||
              sourceLower.includes("pi ai")
            ) {
              category = "inflection";
            } else if (
              sourceLower.includes("ibm") ||
              sourceLower === "ibm" ||
              sourceLower.includes("ibm research")
            ) {
              category = "ibm";
            } else if (
              sourceLower.includes("apple") ||
              sourceLower === "apple" ||
              sourceLower.includes("machine learning journal")
            ) {
              category = "apple";
            } else if (
              sourceLower.includes("intel") ||
              sourceLower === "intel"
            ) {
              category = "intel";
            } else if (sourceLower.includes("amd") || sourceLower === "amd") {
              category = "amd";
            } else if (
              sourceLower.includes("salesforce") ||
              sourceLower === "salesforce"
            ) {
              category = "salesforce";
            } else if (
              sourceLower.includes("stanford") ||
              sourceLower === "stanford" ||
              sourceLower.includes("sai")
            ) {
              category = "stanford";
            } else if (
              sourceLower.includes("berkeley") ||
              sourceLower === "berkeley" ||
              sourceLower.includes("bair")
            ) {
              category = "berkeley";
            } else if (
              sourceLower.includes("deepmind") ||
              sourceLower === "deepmind"
            ) {
              category = "deepmind";
            } else if (
              sourceLower.includes("techcrunch") ||
              sourceLower === "techcrunch"
            ) {
              category = "techcrunch";
            } else if (
              sourceLower.includes("venturebeat") ||
              sourceLower === "venturebeat"
            ) {
              category = "venturebeat";
            } else if (
              sourceLower.includes("the verge") ||
              sourceLower === "verge"
            ) {
              category = "verge";
            } else if (
              sourceLower.includes("wired") ||
              sourceLower === "wired"
            ) {
              category = "wired";
            } else if (
              sourceLower.includes("mit technology review") ||
              sourceLower === "mit" ||
              sourceLower.includes("technology review")
            ) {
              category = "mit";
            } else if (
              sourceLower.includes("nature") ||
              sourceLower === "nature"
            ) {
              category = "nature";
            } else if (
              sourceLower.includes("science") ||
              sourceLower === "science"
            ) {
              category = "science";
            } else if (
              sourceLower.includes("menlo") ||
              sourceLower === "menlo" ||
              sourceLower.includes("menlo ventures")
            ) {
              category = "menlo";
            } else {
              // Fallback: usar image_categories.txt ou articleId
              const firstCategory = imageCategories[0]?.toLowerCase();
              const articleIdLower = articleId.toLowerCase();

              if (firstCategory || articleIdLower) {
                const searchText = (firstCategory || "") + " " + articleIdLower;

                if (searchText.includes("nvidia")) {
                  category = "nvidia";
                } else if (searchText.includes("openai")) {
                  category = "openai";
                } else if (searchText.includes("google")) {
                  category = "google";
                } else if (
                  searchText.includes("meta") ||
                  searchText.includes("facebook")
                ) {
                  category = "meta";
                } else if (
                  searchText.includes("anthropic") ||
                  searchText.includes("claude")
                ) {
                  category = "anthropic";
                } else if (searchText.includes("deepseek")) {
                  category = "deepseek";
                } else if (
                  searchText.includes("x.ai") ||
                  searchText.includes("x.com") ||
                  searchText.includes("grok")
                ) {
                  category = "x";
                } else if (searchText.includes("mistral")) {
                  category = "mistral";
                } else if (
                  searchText.includes("alibaba") ||
                  searchText.includes("damo") ||
                  searchText.includes("alizila")
                ) {
                  category = "alibaba";
                } else if (searchText.includes("microsoft")) {
                  category = "microsoft";
                } else if (
                  searchText.includes("hivehub") ||
                  searchText.includes("hive-hub")
                ) {
                  category = "hivehub";
                } else if (searchText.includes("perplexity")) {
                  category = "perplexity";
                } else if (
                  searchText.includes("huggingface") ||
                  searchText.includes("hugging face")
                ) {
                  category = "huggingface";
                } else if (searchText.includes("stability")) {
                  category = "stability";
                } else if (
                  searchText.includes("elevenlabs") ||
                  searchText.includes("eleven labs")
                ) {
                  category = "elevenlabs";
                } else if (
                  searchText.includes("character.ai") ||
                  searchText.includes("character ai")
                ) {
                  category = "character";
                } else if (
                  searchText.includes("inflection") ||
                  searchText.includes("pi ai")
                ) {
                  category = "inflection";
                } else if (
                  searchText.includes("ibm") ||
                  searchText.includes("ibm research")
                ) {
                  category = "ibm";
                } else if (
                  searchText.includes("apple") ||
                  searchText.includes("machine learning journal")
                ) {
                  category = "apple";
                } else if (searchText.includes("intel")) {
                  category = "intel";
                } else if (searchText.includes("amd")) {
                  category = "amd";
                } else if (searchText.includes("salesforce")) {
                  category = "salesforce";
                } else if (
                  searchText.includes("stanford") ||
                  searchText.includes("sai")
                ) {
                  category = "stanford";
                } else if (
                  searchText.includes("berkeley") ||
                  searchText.includes("bair")
                ) {
                  category = "berkeley";
                } else if (searchText.includes("deepmind")) {
                  category = "deepmind";
                } else if (searchText.includes("techcrunch")) {
                  category = "techcrunch";
                } else if (searchText.includes("venturebeat")) {
                  category = "venturebeat";
                } else if (
                  searchText.includes("the verge") ||
                  searchText.includes("verge")
                ) {
                  category = "verge";
                } else if (searchText.includes("wired")) {
                  category = "wired";
                } else if (
                  searchText.includes("mit") ||
                  searchText.includes("technology review")
                ) {
                  category = "mit";
                } else if (searchText.includes("nature")) {
                  category = "nature";
                } else if (searchText.includes("science")) {
                  category = "science";
                } else if (
                  searchText.includes("menlo") ||
                  searchText.includes("menlo ventures")
                ) {
                  category = "menlo";
                }
              }
            }

            // Select image for article detail: use first category, always deterministic (based on articleId hash)
            const imageArticle = await selectArticleImage(
              imageCategories,
              articleId,
              imagesBaseDir,
              "article",
            );

            // Log for debugging
            console.log(
              `[Article API] Article: ${articleId}, Selected Image: ${imageArticle}`,
            );

            // Gerar slug
            let slug = slugContent.trim();
            if (!slug && title) {
              slug = title
                .toLowerCase()
                .replace(/[^\w\s-]/g, "")
                .replace(/\s+/g, "-")
                .replace(/-+/g, "-")
                .replace(/^-|-$/g, "");
            }

            const article: Article = {
              id: articleId,
              slug: slug || articleId,
              title: title.trim(),
              excerpt:
                subtitle.trim() ||
                articleContent
                  .split("\n")
                  .filter((l) => l.trim())
                  .slice(0, 3)
                  .join(" ")
                  .substring(0, 200) + "...",
              content: articleContent,
              date: dirStats
                ? new Date(dirStats.mtime).toISOString().split("T")[0]
                : new Date().toISOString().split("T")[0],
              author: sourceContent.trim() || "ScienceAI Team",
              category,
              readTime: Math.ceil(articleContent.split(" ").length / 200),
              imageCategories,
              imageArticle: imageArticle || "/images/ai/ai_1.jpg", // Image for article detail (first category)
            };

            res.setHeader("Content-Type", "application/json");
            res.setHeader("Access-Control-Allow-Origin", "*");
            res.end(JSON.stringify({ article }));
          } catch (error) {
            console.error("Error in article API:", error);
            res.statusCode = 500;
            res.end(JSON.stringify({ error: "Failed to fetch article" }));
          }
        } else if (req.url === "/api/subscribe" && req.method === "POST") {
          // API para salvar emails de subscribers
          try {
            let body = "";
            req.on("data", (chunk) => {
              body += chunk.toString();
            });

            req.on("end", async () => {
              try {
                const { email } = JSON.parse(body);

                // Validar email
                if (
                  !email ||
                  typeof email !== "string" ||
                  !email.includes("@")
                ) {
                  res.statusCode = 400;
                  res.setHeader("Content-Type", "application/json");
                  res.setHeader("Access-Control-Allow-Origin", "*");
                  res.end(JSON.stringify({ error: "Invalid email address" }));
                  return;
                }

                // Normalizar email (lowercase, trim)
                const normalizedEmail = email.toLowerCase().trim();

                // Caminho para o arquivo de subscribers
                const subscribersPath = path.join(
                  projectRoot,
                  "scienceai_subscribers.json",
                );

                // Ler arquivo existente ou criar novo
                let subscribers: {
                  emails: string[];
                  subscribedAt: Record<string, string>;
                } = {
                  emails: [],
                  subscribedAt: {},
                };

                try {
                  const existingContent = await fs.readFile(
                    subscribersPath,
                    "utf-8",
                  );
                  subscribers = JSON.parse(existingContent);
                  // Garantir estrutura
                  if (!subscribers.emails) subscribers.emails = [];
                  if (!subscribers.subscribedAt) subscribers.subscribedAt = {};
                } catch (err) {
                  // Arquivo não existe, criar novo
                  console.log("[Subscribe API] Creating new subscribers file");
                }

                // Verificar se email já existe
                if (subscribers.emails.includes(normalizedEmail)) {
                  res.statusCode = 200;
                  res.setHeader("Content-Type", "application/json");
                  res.setHeader("Access-Control-Allow-Origin", "*");
                  res.end(
                    JSON.stringify({
                      success: true,
                      message: "Email already subscribed",
                      alreadySubscribed: true,
                    }),
                  );
                  return;
                }

                // Adicionar email
                subscribers.emails.push(normalizedEmail);
                subscribers.subscribedAt[normalizedEmail] =
                  new Date().toISOString();

                // Ordenar emails alfabeticamente
                subscribers.emails.sort();

                // Salvar arquivo
                await fs.writeFile(
                  subscribersPath,
                  JSON.stringify(subscribers, null, 2),
                  "utf-8",
                );

                console.log(
                  `[Subscribe API] Email subscribed: ${normalizedEmail}`,
                );

                res.statusCode = 200;
                res.setHeader("Content-Type", "application/json");
                res.setHeader("Access-Control-Allow-Origin", "*");
                res.end(
                  JSON.stringify({
                    success: true,
                    message: "Email subscribed successfully",
                    email: normalizedEmail,
                  }),
                );
              } catch (parseError) {
                console.error(
                  "[Subscribe API] Error parsing request:",
                  parseError,
                );
                res.statusCode = 400;
                res.setHeader("Content-Type", "application/json");
                res.setHeader("Access-Control-Allow-Origin", "*");
                res.end(JSON.stringify({ error: "Invalid request body" }));
              }
            });
          } catch (error) {
            console.error("[Subscribe API] Error:", error);
            res.statusCode = 500;
            res.setHeader("Content-Type", "application/json");
            res.setHeader("Access-Control-Allow-Origin", "*");
            res.end(JSON.stringify({ error: "Failed to subscribe email" }));
          }
        } else {
          next();
        }
      });
    },
  };
}
