import { Controller, Get, Post, Param, Body } from "@cmmv/http";
import { Article } from "../contracts/article.contract";
import type pg from "pg";

@Controller("articles")
export class ArticlesController {
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
  async findAll(): Promise<Article[]> {
    try {
      const client = await this.getClient();
      const result = await client.query("SELECT * FROM articles ORDER BY created_at DESC LIMIT 100");
      return result.rows;
    } catch (error) {
      console.error("Error fetching articles:", error);
      return [];
    }
  }

  @Get(":id")
  async findOne(@Param("id") id: string): Promise<Article | null> {
    try {
      const client = await this.getClient();
      const result = await client.query("SELECT * FROM articles WHERE id = $1", [id]);
      return result.rows[0] || null;
    } catch (error) {
      console.error("Error fetching article:", error);
      return null;
    }
  }

  @Post()
  async create(@Body() article: Article): Promise<Article> {
    try {
      const client = await this.getClient();
      const result = await client.query(
        "INSERT INTO articles (id, title, content, summary, author, published_at, source_url, tags, language, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
        [
          article.id || crypto.randomUUID(),
          article.title,
          article.content,
          article.summary,
          article.author,
          article.published_at || new Date(),
          article.source_url,
          article.tags,
          article.language || "en",
          article.status || "draft",
        ]
      );
      return result.rows[0];
    } catch (error) {
      console.error("Error creating article:", error);
      throw error;
    }
  }

  @Get("health")
  async health(): Promise<{ status: string; timestamp: string }> {
    return {
      status: "ok",
      timestamp: new Date().toISOString(),
    };
  }
}

