/**
 * Run database migrations
 * Execute SQL files in order
 */

import { readFileSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import pg from "pg";

const { Client } = pg;

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function runMigrations() {
  // Get connection details from environment or use defaults
  const connectionString =
    process.env.DATABASE_URL ||
    `postgresql://${process.env.POSTGRES_USER || "hivenews"}:${process.env.POSTGRES_PASSWORD || "hivenews123"}@${process.env.POSTGRES_HOST || "localhost"}:${process.env.POSTGRES_PORT || "5432"}/${process.env.POSTGRES_DB || "hivenews"}`;

  const client = new Client({
    connectionString,
  });

  try {
    await client.connect();
    console.log("✅ Connected to database");

    // Read and execute migrations
    const migrations = [
      resolve(__dirname, "001_initial_schema.sql"),
    ];

    for (const migrationPath of migrations) {
      const sql = readFileSync(migrationPath, "utf-8");
      await client.query(sql);
      console.log(`✅ Applied: ${migrationPath}`);
    }

    console.log("✅ All migrations completed");
  } catch (error) {
    console.error("❌ Migration failed:", error);
    process.exit(1);
  } finally {
    await client.end();
  }
}

runMigrations();


