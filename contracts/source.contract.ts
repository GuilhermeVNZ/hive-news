import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsEnum, IsDate } from "@cmmv/core";

/**
 * Source / Collector Contract
 * Collects content from multiple sources (RSS, APIs, HTML)
 */
@Contract({
  namespace: "Source",
  controllerName: "Source",
  protoPackage: "source",
  options: {
    databaseSchemaName: "sources",
  },
})
export class SourceContract {
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
  url: string;

  @ContractField({
    protoType: "enum",
    validations: [{ type: "IsEnum", value: ["rss", "api", "html"] }],
  })
  kind: "rss" | "api" | "html";

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  last_fetch: Date;

  // CMMV Messages
  @ContractMessage({
    name: "CreateSourceRequest",
    properties: {
      portal_id: { type: "string", required: true },
      url: { type: "string", required: true },
      kind: { type: "enum", required: true },
    },
  })
  createRequest: any;

  @ContractMessage({
    name: "CreateSourceResponse",
    properties: {
      id: { type: "string", required: true },
      portal_id: { type: "string", required: true },
      url: { type: "string", required: true },
      kind: { type: "enum", required: true },
      last_fetch: { type: "date", required: true },
    },
  })
  createResponse: any;

  @ContractMessage({
    name: "GetSourceResponse",
    properties: {
      id: { type: "string", required: true },
      portal_id: { type: "string", required: true },
      url: { type: "string", required: true },
      kind: { type: "enum", required: true },
      last_fetch: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListSourcesResponse",
    properties: {
      sources: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/sources",
    method: "POST",
    name: "CreateSource",
    request: "CreateSourceRequest",
    response: "CreateSourceResponse",
    functionName: "createSource",
  })
  createService: any;

  @ContractService({
    path: "/api/sources/:id",
    method: "GET",
    name: "GetSource",
    response: "GetSourceResponse",
    functionName: "getSource",
  })
  getService: any;

  @ContractService({
    path: "/api/sources",
    method: "GET",
    name: "ListSources",
    response: "ListSourcesResponse",
    functionName: "listSources",
  })
  listService: any;
}
