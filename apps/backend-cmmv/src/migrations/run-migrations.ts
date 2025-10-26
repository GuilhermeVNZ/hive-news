/**
 * Run database migrations
 * Execute SQL files in order
 */

import { readFileSync } from "fs";
import { resolve } from "path";
import pg from "pg";

const { Client } = pg;

async function runMigrations() {
  const client = new Client({
    connectionString: process.env.DATABASE_URL,
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

