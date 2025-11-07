import { NextResponse } from "next/server";
import fs from "fs/promises";
import path from "path";

import type { Article } from "@/types/article";

type RegistryMetadata = {
  status?: string;
  destinations?: unknown;
  featured?: unknown;
  hidden?: unknown;
  [key: string]: unknown;
};

type RegistryFile = {
  articles?: Record<string, RegistryMetadata>;
};

async function readArticlesFromDir(
  outputDir: string,
  isPromotional: boolean = false,
): Promise<Article[]> {
  const articles: Article[] = [];

  try {
    const dirs = await fs.readdir(outputDir);

    for (const articleId of dirs) {
      const articleDir = path.join(outputDir, articleId);

      try {
        const stats = await fs.stat(articleDir);
        if (!stats.isDirectory()) continue;

        // Ler arquivos necessários
        const titlePath = path.join(articleDir, "title.txt");
        const articlePath = path.join(articleDir, "article.md");
        const linkedinPostPath = path.join(articleDir, "linkedin.txt");

        const [title, articleContent, linkedinPostContent, dirStats] =
          await Promise.all([
            fs.readFile(titlePath, "utf-8").catch(() => ""),
            fs.readFile(articlePath, "utf-8").catch(() => ""),
            fs.readFile(linkedinPostPath, "utf-8").catch(() => ""),
            fs.stat(articleDir), // Para obter data de modificação
          ]);

        // Usar conteúdo de linkedin.txt como excerpt (subtítulo) para AIResearch
        // Se linkedin.txt não existir ou estiver vazio, usar fallback das primeiras linhas do artigo
        let excerpt = linkedinPostContent.trim();
        if (!excerpt) {
          // Fallback: extrair excerpt das primeiras 3 linhas do artigo
          excerpt =
            articleContent
              .split("\n")
              .filter((line) => line.trim())
              .slice(0, 3)
              .join(" ")
              .substring(0, 200) + "...";
        }

        // Ler image categories
        const categoriesPath = path.join(articleDir, "image_categories.txt");
        const categoriesContent = await fs
          .readFile(categoriesPath, "utf-8")
          .catch(() => "");
        const imageCategories = categoriesContent
          .split("\n")
          .filter((c) => c.trim());

        if (title && articleContent) {
          articles.push({
            id: articleId,
            title: title.trim(),
            excerpt,
            article: articleContent,
            // Usar data de modificação da pasta como publishedAt (mais recente = modificado mais recentemente)
            publishedAt: dirStats.mtime.toISOString(),
            author: "AI Research",
            category: imageCategories[0] || "ai",
            readTime: Math.ceil(articleContent.split(" ").length / 200),
            imageCategories,
            // Marcar como promocional
            isPromotional: isPromotional,
          });
        } else {
          console.warn(
            `[AIResearch Articles API] Skipping article ${articleId}: missing title or content`,
          );
          console.warn(
            `  - title: ${title ? "exists" : "missing"} (${title ? title.substring(0, 50) : "N/A"})`,
          );
          console.warn(
            `  - articleContent: ${articleContent ? "exists" : "missing"} (${articleContent ? articleContent.length + " chars" : "N/A"})`,
          );
        }
      } catch (error) {
        console.error(`Error reading article ${articleId}:`, error);
      }
    }
  } catch (error) {
    // Se a pasta não existir, não é erro (pode não ter artigos promocionais ainda)
    if (isPromotional) {
      console.log("Promotional directory not found, skipping...");
    } else {
      console.error("Error reading output directory:", error);
    }
  }

  return articles;
}

async function readArticles(): Promise<Article[]> {
  // Helper function to check if value is truthy (handles true, "true", 1, "1")
  const isTrue = (value: unknown): boolean => {
    if (typeof value === "boolean") {
      return value;
    }
    if (typeof value === "number") {
      return value === 1;
    }
    if (typeof value === "string") {
      return value.toLowerCase() === "true" || value === "1";
    }
    return false;
  };

  // Helper function to normalize ID (trim, lowercase, NFC normalize)
  const normalizeId = (s: string): string => {
    return s.normalize("NFC").trim().toLowerCase();
  };

  // Função para extrair arXiv ID do nome da pasta
  // Formato da pasta: "2025-10-29_unknown_2510.21560" -> "2510.21560"
  function extractArxivId(folderName: string): string {
    const arxivIdMatch = folderName.match(/\d{4}\.\d{4,6}(v\d+)?/);
    if (arxivIdMatch) {
      return arxivIdMatch[0].replace(/v\d+$/, "");
    }
    const parts = folderName.split("_");
    if (parts.length >= 3) {
      const lastPart = parts[parts.length - 1];
      if (lastPart.match(/^\d{4}\.\d{4,6}(v\d+)?$/)) {
        return lastPart.replace(/v\d+$/, "");
      }
    } else if (
      parts.length === 1 &&
      folderName.match(/^\d{4}\.\d{4,6}(v\d+)?$/)
    ) {
      return folderName.replace(/v\d+$/, "");
    }
    return folderName;
  }

  // PRIMEIRO: Ler o registry para obter lista de artigos publicados para AIResearch
  const registryMap = new Map<string, RegistryMetadata>();
  const featuredMap = new Map<string, boolean>();
  const hiddenMap = new Map<string, boolean>();

  try {
    const possibleRegistryPaths = [
      path.join(process.cwd(), "../../../../articles_registry.json"),
      path.join(process.cwd(), "../../../articles_registry.json"),
      path.join(process.cwd(), "../articles_registry.json"),
      path.resolve("G:/Hive-Hub/News-main/articles_registry.json"),
    ];

    let registryPath: string | null = null;
    let registryContent: string = "";

    for (const testPath of possibleRegistryPaths) {
      try {
        await fs.access(testPath);
        registryPath = testPath;
        registryContent = await fs.readFile(testPath, "utf-8");
        console.log(
          `[AIResearch Articles API] Reading registry from: ${testPath}`,
        );
        break;
      } catch {
        continue;
      }
    }

    if (registryPath && registryContent) {
      const registry = JSON.parse(registryContent) as RegistryFile;
      if (registry.articles) {
        for (const [id, metadata] of Object.entries(registry.articles)) {
          if (!metadata || typeof metadata !== "object") {
            console.warn(
              `[AIResearch Articles API] ⚠️  Skipping invalid metadata for article ${id}`,
            );
            continue;
          }
          const destinations = Array.isArray(metadata.destinations)
            ? (metadata.destinations as unknown[]).filter(
                (destination): destination is string =>
                  typeof destination === "string",
              )
            : [];

          // Verificar se artigo está publicado e tem destino AIResearch
          const isPublished = metadata.status === "Published";
          const hasAIResearchDest = destinations.some(
            (destination) => destination.toLowerCase() === "airesearch",
          );

          if (isPublished && hasAIResearchDest) {
            // Armazenar metadata do registry
            registryMap.set(id, metadata);

            // Normalizar ID para lookup
            const normalizedId = normalizeId(id);
            registryMap.set(normalizedId, metadata);

            // Extrair arXiv ID do ID do registry
            const arxivId = extractArxivId(id);
            if (arxivId !== id) {
              registryMap.set(arxivId, metadata);
              registryMap.set(normalizeId(arxivId), metadata);
            }

            // Marcar featured/hidden (usar valores seguros)
            const isFeatured = isTrue(metadata.featured);
            const isHidden = isTrue(metadata.hidden);

            if (isFeatured) {
              featuredMap.set(id, true);
              featuredMap.set(normalizedId, true);
              if (arxivId !== id) {
                featuredMap.set(arxivId, true);
                featuredMap.set(normalizeId(arxivId), true);
              }
            }

            if (isHidden) {
              hiddenMap.set(id, true);
              hiddenMap.set(normalizedId, true);
              if (arxivId !== id) {
                hiddenMap.set(arxivId, true);
                hiddenMap.set(normalizeId(arxivId), true);
              }
            }
          }
        }
        const uniqueArticles = registryMap.size / 4; // Dividido por 4 porque armazenamos ID original, normalizado, arXiv ID, e arXiv ID normalizado
        console.log(
          `[AIResearch Articles API] Found ${uniqueArticles} unique articles in registry for AIResearch`,
        );
        console.log(
          `[AIResearch Articles API] Registry map size: ${registryMap.size} (includes normalized variants)`,
        );
        // Log sample IDs from registry
        const sampleRegistryIds = Array.from(registryMap.keys()).slice(0, 5);
        console.log(
          `[AIResearch Articles API] Sample registry IDs:`,
          sampleRegistryIds,
        );
      }
    }
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(
      "[AIResearch Articles API] ⚠️  Error reading registry:",
      message,
    );
  }

  // SEGUNDO: Ler artigos do filesystem e cruzar com registry
  const possibleBasePaths = [
    path.join(process.cwd(), "../../../../output"),
    path.join(process.cwd(), "../../../output"),
    path.resolve("G:/Hive-Hub/News-main/output"),
  ];

  let baseOutputDir: string | null = null;
  for (const testPath of possibleBasePaths) {
    try {
      await fs.access(testPath);
      baseOutputDir = testPath;
      console.log(
        `[AIResearch Articles API] Using output directory: ${testPath}`,
      );
      break;
    } catch {
      continue;
    }
  }

  if (!baseOutputDir) {
    console.error("[AIResearch Articles API] ❌ Output directory not found!");
    return [];
  }

  const promotionalDir = path.join(baseOutputDir, "Promotional");
  const normalDir = path.join(baseOutputDir, "AIResearch");

  // Ler artigos do filesystem
  const promotionalArticles = await readArticlesFromDir(promotionalDir, true);
  const normalArticles = await readArticlesFromDir(normalDir, false);
  const allFilesystemArticles = [...promotionalArticles, ...normalArticles];

  console.log(
    `[AIResearch Articles API] Found ${allFilesystemArticles.length} articles in filesystem`,
  );

  // Log sample IDs from filesystem
  const sampleFilesystemIds = allFilesystemArticles
    .slice(0, 5)
    .map((a) => a.id);
  console.log(
    `[AIResearch Articles API] Sample filesystem IDs:`,
    sampleFilesystemIds,
  );

  // TERCEIRO: Cruzar informações - só incluir artigos que existem tanto no registry quanto no filesystem
  const allArticles: Article[] = [];
  let matchedCount = 0;
  let skippedCount = 0;
  let notInRegistryCount = 0;

  // Para cada artigo no filesystem, verificar se existe no registry
  for (const article of allFilesystemArticles) {
    const arxivId = extractArxivId(article.id);
    const normalizedArticleId = normalizeId(article.id);
    const normalizedArxivId = normalizeId(arxivId);

    // Tentar encontrar no registry usando múltiplas variações do ID
    const registryMeta =
      registryMap.get(article.id) ||
      registryMap.get(normalizedArticleId) ||
      registryMap.get(arxivId) ||
      registryMap.get(normalizedArxivId);

    if (!registryMeta) {
      // Artigo não está no registry ou não está publicado para AIResearch
      notInRegistryCount++;
      if (notInRegistryCount <= 5) {
        // Log apenas os primeiros 5 para não poluir
        console.log(
          `[AIResearch Articles API] ⚠️  Skipping article ${article.id} (arXiv ID: ${arxivId}) - not in registry or not published for AIResearch`,
        );
      }
      skippedCount++;
      continue;
    }

    // Artigo existe no registry - verificar se está hidden
    const isHidden =
      hiddenMap.get(article.id) === true ||
      hiddenMap.get(normalizedArticleId) === true ||
      hiddenMap.get(arxivId) === true ||
      hiddenMap.get(normalizedArxivId) === true;

    if (isHidden) {
      skippedCount++;
      console.log(
        `[AIResearch Articles API] 🚫 Article ${article.id} (arXiv ID: ${arxivId}) is HIDDEN - filtering out`,
      );
      continue; // Skip hidden articles
    }

    // Verificar se está featured
    const featured =
      featuredMap.get(article.id) === true ||
      featuredMap.get(normalizedArticleId) === true ||
      featuredMap.get(arxivId) === true ||
      featuredMap.get(normalizedArxivId) === true;

    // Adicionar artigo com informações do registry
    article.featured = featured;
    allArticles.push(article);
    matchedCount++;

    if (featured) {
      console.log(
        `[AIResearch Articles API] ✓ Article ${article.id} (arXiv ID: ${arxivId}) "${article.title.substring(0, 50)}" is FEATURED`,
      );
    }
  }

  const featuredCount = allArticles.filter((a) => a.featured === true).length;
  console.log(
    `[AIResearch Articles API] Matched ${matchedCount} articles (skipped ${skippedCount}: ${notInRegistryCount} not in registry, ${skippedCount - notInRegistryCount} hidden), ${featuredCount} featured`,
  );

  if (featuredCount === 0) {
    console.warn(
      "[AIResearch Articles API] ⚠️  No featured articles found! Featured map size:",
      featuredMap.size,
    );
    console.warn(
      "[AIResearch Articles API] Sample article IDs (from folders):",
      allArticles.map((a) => a.id).slice(0, 5),
    );
    console.warn(
      "[AIResearch Articles API] Sample extracted arXiv IDs:",
      allArticles.map((a) => extractArxivId(a.id)).slice(0, 5),
    );
    console.warn(
      "[AIResearch Articles API] Featured map keys (from registry):",
      Array.from(featuredMap.keys()).slice(0, 5),
    );
  }

  // Debug: Log total featured count
  console.log(
    `[AIResearch Articles API] Total featured articles: ${featuredCount} out of ${allArticles.length} articles`,
  );

  // Ordenar: featured primeiro, depois promocionais, depois por data (mais recente primeiro)
  allArticles.sort((a, b) => {
    const aFeatured = a.featured === true;
    const bFeatured = b.featured === true;

    // Featured sempre vem primeiro
    if (aFeatured && !bFeatured) return -1;
    if (!aFeatured && bFeatured) return 1;

    // Se ambos são featured ou ambos não são featured, considerar promocional
    if (aFeatured === bFeatured) {
      if (a.isPromotional && !b.isPromotional) return -1;
      if (!a.isPromotional && b.isPromotional) return 1;
      return (
        new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime()
      );
    }

    return (
      new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime()
    );
  });

  return allArticles;
}

// Force Node.js runtime and disable caching
export const runtime = "nodejs";
export const revalidate = 0;
export const dynamic = "force-dynamic";

export async function GET(request: Request) {
  try {
    // Disable caching to ensure fresh data
    const { searchParams } = new URL(request.url);
    const category = searchParams.get("category");

    const articles = await readArticles();

    let filteredArticles = articles;
    if (category && category !== "all") {
      filteredArticles = articles.filter((a) => a.category === category);
    }

    // Debug: Log featured articles in response
    const featuredInResponse = filteredArticles.filter(
      (a) => a.featured === true,
    );
    console.log(
      `[AIResearch Articles API] Returning ${filteredArticles.length} articles, ${featuredInResponse.length} featured`,
    );

    return NextResponse.json(
      { articles: filteredArticles },
      {
        headers: {
          "Cache-Control":
            "no-store, no-cache, must-revalidate, proxy-revalidate",
          Pragma: "no-cache",
          Expires: "0",
        },
      },
    );
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const stack = error instanceof Error ? error.stack : undefined;
    console.error(
      "[AIResearch Articles API] ❌ Error in GET handler:",
      message,
    );
    if (stack) {
      console.error("[AIResearch Articles API] Error stack:", stack);
    }

    // Retornar erro 500 com mensagem de erro
    return NextResponse.json(
      {
        error: "Internal server error",
        message,
        articles: [], // Retornar array vazio em caso de erro
      },
      {
        status: 500,
        headers: {
          "Cache-Control":
            "no-store, no-cache, must-revalidate, proxy-revalidate",
          Pragma: "no-cache",
          Expires: "0",
        },
      },
    );
  }
}
