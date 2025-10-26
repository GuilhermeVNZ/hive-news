import { Controller, Get, Post, Param, Body } from "@cmmv/http";
import { Article } from "../contracts/article.contract";

@Controller("articles")
export class ArticlesController {
  @Get()
  async findAll(): Promise<Article[]> {
    // TODO: Connect to actual service
    return [];
  }

  @Get(":id")
  async findOne(@Param("id") id: string): Promise<Article | null> {
    // TODO: Connect to actual service
    return null;
  }

  @Post()
  async create(@Body() article: Article): Promise<Article> {
    // TODO: Connect to actual service
    return article;
  }
}

