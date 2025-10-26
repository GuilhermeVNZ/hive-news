import { Controller, Get, Post, Param, Body } from "@cmmv/http";
import { Source } from "../contracts/source.contract";

@Controller("sources")
export class SourcesController {
  @Get()
  async findAll(): Promise<Source[]> {
    // TODO: Connect to actual service
    return [];
  }

  @Get(":id")
  async findOne(@Param("id") id: string): Promise<Source | null> {
    // TODO: Connect to actual service
    return null;
  }

  @Post()
  async create(@Body() source: Source): Promise<Source> {
    // TODO: Connect to actual service
    return source;
  }
}

