import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsNumber, IsDate, Min, Max } from "@cmmv/core";

/**
 * Translation Contract
 * Multilingual article translations
 */
@Contract({
  namespace: "Translation",
  controllerName: "Translation",
  protoPackage: "translation",
  options: {
    databaseSchemaName: "translations",
  },
})
export class TranslationContract {
  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  article_id: string;

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
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  body: string;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  seo_score: number;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  created_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetTranslationResponse",
    properties: {
      article_id: { type: "string", required: true },
      lang: { type: "string", required: true },
      title: { type: "string", required: true },
      body: { type: "string", required: true },
      seo_score: { type: "float", required: true },
      created_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListTranslationsResponse",
    properties: {
      translations: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/translations/:article_id/:lang",
    method: "GET",
    name: "GetTranslation",
    response: "GetTranslationResponse",
    functionName: "getTranslation",
  })
  getService: any;

  @ContractService({
    path: "/api/translations",
    method: "GET",
    name: "ListTranslations",
    response: "ListTranslationsResponse",
    functionName: "listTranslations",
  })
  listService: any;
}
