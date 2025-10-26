/**
 * Hive-News Backend Application
 * Simple HTTP server with Express
 */

import express from "express";
import cors from "cors";

// Services
import { ProfileLoaderService } from "./services/profile-loader.service";
import { StyleSystemService } from "./services/style-system.service";
import { SourceManagerService } from "./services/source-manager.service";

const app = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(cors());
app.use(express.json());

// Initialize services
const profileLoader = new ProfileLoaderService();
const styleSystem = new StyleSystemService();
const sourceManager = new SourceManagerService();

// Health check
app.get("/health", (req, res) => {
  res.json({ status: "ok", timestamp: new Date().toISOString() });
});

// Articles endpoint
app.get("/api/articles", async (req, res) => {
  try {
    res.json([]);
  } catch (error) {
    res.status(500).json({ error: "Internal server error" });
  }
});

// Sources endpoint
app.get("/api/sources", async (req, res) => {
  try {
    res.json([]);
  } catch (error) {
    res.status(500).json({ error: "Internal server error" });
  }
});

// Start server
app.listen(PORT, () => {
  console.log("🚀 Hive-News Backend Starting...");
  console.log(`✅ Server listening on http://localhost:${PORT}`);
  console.log("📡 API endpoints ready:");
  console.log(`   GET  /health`);
  console.log(`   GET  /api/articles`);
  console.log(`   GET  /api/sources`);
});
