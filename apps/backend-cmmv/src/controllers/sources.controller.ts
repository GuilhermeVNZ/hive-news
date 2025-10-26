import { Controller, Get, Post, Param, Body } from "@cmmv/http";
import { Source } from "../contracts/source.contract";

@Controller("sources")
export class SourcesController {
  @Get()
  async findAll(): Promise<Source[]> {
    // Return empty array for now - will be connected to database
    return [];
  }

  @Get(":id")
  async findOne(@Param("id") id: string): Promise<Source | null> {
    // Return null for now - will be connected to database
    return null;
  }

  @Post()
  async create(@Body() source: Source): Promise<Source> {
    // Echo back for now - will be connected to database
    return source;
  }
}

