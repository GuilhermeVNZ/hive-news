import { Controller, Get, Post, Param, Body } from "@cmmv/http";
import { Article } from "../contracts/article.contract";

@Controller("articles")
export class ArticlesController {
  @Get()
  async findAll(): Promise<Article[]> {
    // Return empty array for now - will be connected to database
    return [];
  }

  @Get(":id")
  async findOne(@Param("id") id: string): Promise<Article | null> {
    // Return null for now - will be connected to database
    return null;
  }

  @Post()
  async create(@Body() article: Article): Promise<Article> {
    // Echo back for now - will be connected to database
    return article;
  }

  @Get("health")
  async health(): Promise<{ status: string; timestamp: string }> {
    return {
      status: "ok",
      timestamp: new Date().toISOString(),
    };
  }
}

