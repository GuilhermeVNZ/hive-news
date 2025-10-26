import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsArray, IsNumber, IsDate, Min, Max } from "@cmmv/core";

/**
 * Article Contract
 * Generated natural language journalistic articles from documents
 */
@Contract({
  namespace: "Article",
  controllerName: "Article",
  protoPackage: "article",
  options: {
    databaseSchemaName: "articles",
  },
})
export class ArticleContract {
  @ContractField({
    protoType: "string",
    defaultValue: "",
    index: true,
    unique: true,
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  id: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  portal_id: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  lang: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  title: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  dek: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  body: string;

  @ContractField({
    protoType: "string",
    array: true,
    validations: [{ type: "IsArray" }],
  })
  references: string[];

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  seo_score: number;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  published_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "CreateArticleRequest",
    properties: {
      portal_id: { type: "string", required: true },
      lang: { type: "string", required: true },
      title: { type: "string", required: true },
      dek: { type: "string", required: false },
      body: { type: "string", required: true },
      references: { type: "simpleArray", required: false },
      seo_score: { type: "float", required: false },
    },
  })
  createRequest: any;

  @ContractMessage({
    name: "CreateArticleResponse",
    properties: {
      id: { type: "string", required: true },
      portal_id: { type: "string", required: true },
      lang: { type: "string", required: true },
      title: { type: "string", required: true },
      published_at: { type: "date", required: true },
    },
  })
  createResponse: any;

  @ContractMessage({
    name: "GetArticleResponse",
    properties: {
      id: { type: "string", required: true },
      portal_id: { type: "string", required: true },
      lang: { type: "string", required: true },
      title: { type: "string", required: true },
      dek: { type: "string", required: true },
      body: { type: "string", required: true },
      references: { type: "simpleArray", required: true },
      seo_score: { type: "float", required: true },
      published_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListArticlesResponse",
    properties: {
      articles: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/articles",
    method: "POST",
    name: "CreateArticle",
    request: "CreateArticleRequest",
    response: "CreateArticleResponse",
    functionName: "createArticle",
  })
  createService: any;

  @ContractService({
    path: "/api/articles/:id",
    method: "GET",
    name: "GetArticle",
    response: "GetArticleResponse",
    functionName: "getArticle",
  })
  getService: any;

  @ContractService({
    path: "/api/articles",
    method: "GET",
    name: "ListArticles",
    response: "ListArticlesResponse",
    functionName: "listArticles",
  })
  listService: any;
}
