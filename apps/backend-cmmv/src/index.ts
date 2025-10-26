/**
 * Hive-News Backend Application
 * CMMV-based backend for automated scientific content generation
 */

import "reflect-metadata";
import { Application } from "@cmmv/core";
import { DefaultAdapter, DefaultHTTPModule } from "@cmmv/http";
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

// Import contracts for CMMV auto-generation
import { Article } from "./contracts/article.contract";
import { Source } from "./contracts/source.contract";

/**
 * Application entry point
 */
console.log("🚀 Hive-News Backend Starting...");

// Initialize CMMV Application with HTTP support
// The Application will automatically start the HTTP server
Application.create({
  httpAdapter: DefaultAdapter,
  modules: [DefaultHTTPModule],
  providers: [
    ProfileLoaderService,
    StyleSystemService,
    CronValidatorService,
    RSSParserService,
    APICollectorService,
    HTMLScraperService,
    SourceManagerService,
    MetadataExtractorService,
    RankerService,
    PublisherService,
    SchedulerService,
    MetricsService,
    QAValidatorService,
  ],
  contracts: [Article, Source],
});

console.log("✅ Hive-News Backend initialized successfully");
console.log("🎯 CMMV auto-generation: Contracts registered, APIs generated");
console.log("🌐 Server listening on http://localhost:3000");
