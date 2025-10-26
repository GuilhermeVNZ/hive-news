import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsNumber, IsBoolean, IsDate } from "@cmmv/core";

/**
 * Validation Contract
 * QA and factuality checks
 */
@Contract({
  namespace: "Validation",
  controllerName: "Validation",
  protoPackage: "validation",
  options: {
    databaseSchemaName: "validations",
  },
})
export class ValidationContract {
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
    protoType: "float",
    validations: [{ type: "IsNumber" }],
  })
  factuality_score: number;

  @ContractField({
    protoType: "bool",
    validations: [{ type: "IsBoolean" }],
  })
  sources_verified: boolean;

  @ContractField({
    protoType: "bool",
    validations: [{ type: "IsBoolean" }],
  })
  technical_accuracy: boolean;

  @ContractField({
    protoType: "bool",
    validations: [{ type: "IsBoolean" }],
  })
  neutral_language: boolean;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  validated_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetValidationResponse",
    properties: {
      id: { type: "string", required: true },
      article_id: { type: "string", required: true },
      factuality_score: { type: "float", required: true },
      sources_verified: { type: "bool", required: true },
      technical_accuracy: { type: "bool", required: true },
      neutral_language: { type: "bool", required: true },
      validated_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListValidationsResponse",
    properties: {
      validations: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/validations/:id",
    method: "GET",
    name: "GetValidation",
    response: "GetValidationResponse",
    functionName: "getValidation",
  })
  getService: any;

  @ContractService({
    path: "/api/validations",
    method: "GET",
    name: "ListValidations",
    response: "ListValidationsResponse",
    functionName: "listValidations",
  })
  listService: any;
}
