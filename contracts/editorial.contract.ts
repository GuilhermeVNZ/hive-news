import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import {
  IsString,
  MinLength,
  MaxLength,
  IsArray,
  ArrayMinSize,
  IsEnum,
  IsIn,
  IsNumber,
  Min,
  Max,
  ValidateNested,
} from "@cmmv/core";

/**
 * Editorial Contract
 * Defines editorial style, base language, publishing cadence, and sources per portal
 */
@Contract({
  namespace: "Editorial",
  controllerName: "Editorial",
  protoPackage: "editorial",
  options: {
    databaseSchemaName: "editorials",
  },
})
export class EditorialContract {
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
    validations: [
      { type: "MinLength", value: 3 },
      { type: "MaxLength", value: 100 },
    ],
  })
  name: string;

  @ContractField({
    protoType: "string",
    array: true,
    validations: [{ type: "IsArray" }, { type: "ArrayMinSize", value: 1 }],
  })
  sources: string[];

  @ContractField({
    protoType: "enum",
    validations: [{ type: "IsEnum", value: ["scientific", "tech", "policy"] }],
  })
  style: "scientific" | "tech" | "policy";

  @ContractField({
    protoType: "json",
    validations: [{ type: "ValidateNested" }],
    transform: ({ value }) => {
      if (typeof value === "string") return JSON.parse(value);
      return value;
    },
    toPlain: ({ value }) => {
      if (typeof value === "object") return JSON.stringify(value);
      return value;
    },
  })
  langs: {
    base: string;
    translate_to: string[];
  };

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  cadence: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  image_style: string;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  min_rank_to_translate: number;

  @ContractField({
    protoType: "enum",
    validations: [{ type: "IsEnum", value: ["high", "medium", "low"] }],
  })
  seo_priority: "high" | "medium" | "low";

  // CMMV Messages
  @ContractMessage({
    name: "CreateEditorialRequest",
    properties: {
      name: { type: "string", required: true },
      sources: { type: "simpleArray", required: true },
      style: { type: "enum", required: true },
      langs: { type: "json", required: true },
      cadence: { type: "string", required: true },
      image_style: { type: "string", required: true },
      min_rank_to_translate: { type: "float", required: true },
      seo_priority: { type: "enum", required: true },
    },
  })
  createRequest: any;

  @ContractMessage({
    name: "CreateEditorialResponse",
    properties: {
      id: { type: "string", required: true },
      name: { type: "string", required: true },
      sources: { type: "simpleArray", required: true },
      style: { type: "enum", required: true },
    },
  })
  createResponse: any;

  @ContractMessage({
    name: "UpdateEditorialRequest",
    properties: {
      id: { type: "string", required: true },
      name: { type: "string", required: false },
      sources: { type: "simpleArray", required: false },
      style: { type: "enum", required: false },
      langs: { type: "json", required: false },
      cadence: { type: "string", required: false },
      image_style: { type: "string", required: false },
      min_rank_to_translate: { type: "float", required: false },
      seo_priority: { type: "enum", required: false },
    },
  })
  updateRequest: any;

  @ContractMessage({
    name: "UpdateEditorialResponse",
    properties: {
      id: { type: "string", required: true },
      name: { type: "string", required: true },
    },
  })
  updateResponse: any;

  @ContractMessage({
    name: "GetEditorialResponse",
    properties: {
      id: { type: "string", required: true },
      name: { type: "string", required: true },
      sources: { type: "simpleArray", required: true },
      style: { type: "enum", required: true },
      langs: { type: "json", required: true },
      cadence: { type: "string", required: true },
      image_style: { type: "string", required: true },
      min_rank_to_translate: { type: "float", required: true },
      seo_priority: { type: "enum", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListEditorialsResponse",
    properties: {
      editorials: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/editorials",
    method: "POST",
    name: "CreateEditorial",
    request: "CreateEditorialRequest",
    response: "CreateEditorialResponse",
    functionName: "createEditorial",
  })
  createService: any;

  @ContractService({
    path: "/api/editorials/:id",
    method: "PUT",
    name: "UpdateEditorial",
    request: "UpdateEditorialRequest",
    response: "UpdateEditorialResponse",
    functionName: "updateEditorial",
  })
  updateService: any;

  @ContractService({
    path: "/api/editorials/:id",
    method: "GET",
    name: "GetEditorial",
    response: "GetEditorialResponse",
    functionName: "getEditorial",
  })
  getService: any;

  @ContractService({
    path: "/api/editorials",
    method: "GET",
    name: "ListEditorials",
    response: "ListEditorialsResponse",
    functionName: "listEditorials",
  })
  listService: any;
}
