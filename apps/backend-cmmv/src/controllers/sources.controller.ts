import { Controller, Get, Post, Param, Body } from "@cmmv/http";
import { Source } from "../contracts/source.contract";
import type pg from "pg";

@Controller("sources")
export class SourcesController {
  private client: pg.Client | null = null;

  constructor() {
    // Database connection will be injected via CMMV
  }

  private async getClient(): Promise<pg.Client> {
    if (!this.client) {
      const { Client } = await import("pg");
      this.client = new Client({
        connectionString: process.env.DATABASE_URL,
      });
      await this.client.connect();
    }
    return this.client;
  }

  @Get()
  async findAll(): Promise<Source[]> {
    try {
      const client = await this.getClient();
      const result = await client.query("SELECT * FROM sources ORDER BY created_at DESC");
      return result.rows;
    } catch (error) {
      console.error("Error fetching sources:", error);
      return [];
    }
  }

  @Get(":id")
  async findOne(@Param("id") id: string): Promise<Source | null> {
    try {
      const client = await this.getClient();
      const result = await client.query("SELECT * FROM sources WHERE id = $1", [id]);
      return result.rows[0] || null;
    } catch (error) {
      console.error("Error fetching source:", error);
      return null;
    }
  }

  @Post()
  async create(@Body() source: Source): Promise<Source> {
    try {
      const client = await this.getClient();
      const result = await client.query(
        "INSERT INTO sources (id, portal_id, url, kind, title, description, last_fetch, refresh_cron, enabled) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
        [
          source.id || crypto.randomUUID(),
          source.portal_id,
          source.url,
          source.kind,
          source.title,
          source.description,
          source.last_fetch || new Date(),
          source.refresh_cron || "0 * * * *",
          source.enabled ?? true,
        ]
      );
      return result.rows[0];
    } catch (error) {
      console.error("Error creating source:", error);
      throw error;
    }
  }
}

