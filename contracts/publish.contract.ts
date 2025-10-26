import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsEnum, IsDate } from "@cmmv/core";

/**
 * Publish Contract
 * Tracks publishing status across channels
 */
@Contract({
  namespace: "Publish",
  controllerName: "Publish",
  protoPackage: "publish",
  options: {
    databaseSchemaName: "publishes",
  },
})
export class PublishContract {
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
  article_id: string;

  @ContractField({
    protoType: "enum",
    validations: [{ type: "IsEnum", value: ["website", "x_com", "linkedin", "rss"] }],
  })
  channel: "website" | "x_com" | "linkedin" | "rss";

  @ContractField({
    protoType: "enum",
    validations: [{ type: "IsEnum", value: ["pending", "published", "failed"] }],
  })
  status: "pending" | "published" | "failed";

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  url: string;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  published_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetPublishResponse",
    properties: {
      id: { type: "string", required: true },
      article_id: { type: "string", required: true },
      channel: { type: "enum", required: true },
      status: { type: "enum", required: true },
      url: { type: "string", required: true },
      published_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListPublishesResponse",
    properties: {
      publishes: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/publishes/:id",
    method: "GET",
    name: "GetPublish",
    response: "GetPublishResponse",
    functionName: "getPublish",
  })
  getService: any;

  @ContractService({
    path: "/api/publishes",
    method: "GET",
    name: "ListPublishes",
    response: "ListPublishesResponse",
    functionName: "listPublishes",
  })
  listService: any;
}
