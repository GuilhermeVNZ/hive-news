/**
 * Hive-News Backend Application
 * CMMV-based backend for automated scientific content generation
 */

import "reflect-metadata";
import { Application } from "@cmmv/core";
import { createHttpAdapter } from "@cmmv/http";
import { ProfileLoaderService } from "./services/profile-loader.service";
import { StyleSystemService } from "./services/style-system.service";
import { CronValidatorService } from "./services/cron-validator.service";
import { RSSParserService } from "./services/rss-parser.service";
import { APICollectorService } from "./services/api-collector.service";
import { HTMLScraperService } from "./services/html-scraper.service";
import { SourceManagerService } from "./services/source-manager.service";
import { VectorizerClientService } from "./services/vectorizer-client.service";
import { MetadataExtractorService } from "./services/metadata-extractor.service";
import { RankerService } from "./services/ranker.service";
import { DeepSeekClientService } from "./services/deepseek-client.service";
import { SDXLImageService } from "./services/sdxl-image.service";
import { PublisherService } from "./services/publisher.service";
import { SchedulerService } from "./services/scheduler.service";
import { MetricsService } from "./services/metrics.service";
import { QAValidatorService } from "./services/qa-validator.service";

/**
 * Application entry point
 */
async function bootstrap(): Promise<void> {
  console.log("🚀 Hive-News Backend Starting...");

  // Initialize CMMV Application
  const app = await Application.create();
  const httpAdapter = createHttpAdapter(app);

  // Register contracts and services
  // TODO: Register all contracts with auto-generated controllers

  // Initialize all services
  const profileLoader = new ProfileLoaderService();

  // Services initialized for future use
  // These will be registered with the scheduler and used when implementing full workflow
  new StyleSystemService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new CronValidatorService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new RSSParserService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new APICollectorService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new HTMLScraperService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new SourceManagerService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new VectorizerClientService({
    // eslint-disable-line @typescript-eslint/no-unused-vars
    baseURL: process.env.VECTORIZER_URL || "http://127.0.0.1:15002",
    mcpEndpoint: process.env.VECTORIZER_MCP_URL || "http://127.0.0.1:15002/mcp",
  });
  new MetadataExtractorService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new RankerService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new DeepSeekClientService({
    // eslint-disable-line @typescript-eslint/no-unused-vars
    apiKey: process.env.DEEPSEEK_API_KEY || "",
    baseURL: process.env.DEEPSEEK_BASE_URL || "https://api.deepseek.com",
  });
  new SDXLImageService({
    // eslint-disable-line @typescript-eslint/no-unused-vars
    baseURL: process.env.SDXL_URL || "http://127.0.0.1:7860",
    modelPath: "./sdxl/models",
  });
  new PublisherService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new SchedulerService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new MetricsService(); // eslint-disable-line @typescript-eslint/no-unused-vars
  new QAValidatorService(); // eslint-disable-line @typescript-eslint/no-unused-vars

  try {
    // Load portal profiles
    console.log("📋 Loading portal profiles...");
    const profiles = await profileLoader.loadAllProfiles();

    console.log(`✅ Loaded ${profiles.size} portal profiles:`);
    for (const [portalId] of profiles) {
      console.log(`  - ${portalId}: loaded`);
    }

    // Setup profile hot-reload
    profileLoader.watchProfiles((portalId) => {
      console.log(`🔄 Profile reloaded: ${portalId}`);
    });

    console.log("✅ Hive-News Backend initialized successfully");
    console.log("🎯 CMMV auto-generation: Contracts with decorators ready for API generation");
  } catch (error) {
    console.error("❌ Failed to initialize backend:", error);
    process.exit(1);
  }
}

// Start application
bootstrap();
