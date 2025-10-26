import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsNumber, IsDate, Min, Max } from "@cmmv/core";

/**
 * Image Contract
 * SEO and social media optimized images
 */
@Contract({
  namespace: "Image",
  controllerName: "Image",
  protoPackage: "image",
  options: {
    databaseSchemaName: "images",
  },
})
export class ImageContract {
  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  article_id: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  style: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  prompt: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  url_cover: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  url_thumb: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  alt_text: string;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  seo_relevance: number;

  // CMMV Messages
  @ContractMessage({
    name: "GetImageResponse",
    properties: {
      article_id: { type: "string", required: true },
      style: { type: "string", required: true },
      prompt: { type: "string", required: true },
      url_cover: { type: "string", required: true },
      url_thumb: { type: "string", required: true },
      alt_text: { type: "string", required: true },
      seo_relevance: { type: "float", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListImagesResponse",
    properties: {
      images: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/images/:article_id",
    method: "GET",
    name: "GetImage",
    response: "GetImageResponse",
    functionName: "getImage",
  })
  getService: any;

  @ContractService({
    path: "/api/images",
    method: "GET",
    name: "ListImages",
    response: "ListImagesResponse",
    functionName: "listImages",
  })
  listService: any;
}
