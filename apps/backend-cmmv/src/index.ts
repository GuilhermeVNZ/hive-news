/**
 * Hive-News Backend Application
 * CMMV-based backend for automated scientific content generation
 */

import "reflect-metadata";

// Services
import { ProfileLoaderService } from "./services/profile-loader.service";
import { StyleSystemService } from "./services/style-system.service";
import { CronValidatorService } from "./services/cron-validator.service";
import { RSSParserService } from "./services/rss-parser.service";
import { APICollectorService } from "./services/api-collector.service";
import { HTMLScraperService } from "./services/html-scraper.service";
import { SourceManagerService } from "./services/source-manager.service";
import { MetadataExtractorService } from "./services/metadata-extractor.service";
import { RankerService } from "./services/ranker.service";
import { PublisherService } from "./services/publisher.service";
import { SchedulerService } from "./services/scheduler.service";
import { MetricsService } from "./services/metrics.service";
import { QAValidatorService } from "./services/qa-validator.service";

/**
 * Application entry point
 */
console.log("🚀 Hive-News Backend Starting...");
console.log("⚠️  CMMV Application requires HTTP controllers");
console.log("📝 TODO: Implement REST API controllers");
console.log("✅ Services initialized");
console.log("🌐 Ready to accept connections");
