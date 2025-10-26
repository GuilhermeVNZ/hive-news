import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsNumber, IsBoolean, Min, Max, IsDate } from "@cmmv/core";

/**
 * Scientific Validation Contract 🧬
 * Verifies authenticity and scientific integrity of academic papers
 */
@Contract({
  namespace: "ScientificValidation",
  controllerName: "ScientificValidation",
  protoPackage: "scientific_validation",
  options: {
    databaseSchemaName: "scientific_validations",
  },
})
export class ScientificValidationContract {
  @ContractField({
    protoType: "string",
    defaultValue: "",
    index: true,
    unique: true,
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  document_id: string;

  @ContractField({
    protoType: "bool",
    validations: [{ type: "IsBoolean" }],
  })
  source_verified: boolean;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  reputation_score: number;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  citation_resolve_rate: number;

  @ContractField({
    protoType: "bool",
    validations: [{ type: "IsBoolean" }],
  })
  author_verified: boolean;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
  })
  ai_generated_prob: number;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }, { type: "Max", value: 1 }],
    index: true,
  })
  validation_score: number;

  @ContractField({
    protoType: "bool",
    validations: [{ type: "IsBoolean" }],
    index: true,
  })
  flagged: boolean;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  created_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetScientificValidationResponse",
    properties: {
      document_id: { type: "string", required: true },
      source_verified: { type: "bool", required: true },
      reputation_score: { type: "float", required: true },
      citation_resolve_rate: { type: "float", required: true },
      author_verified: { type: "bool", required: true },
      ai_generated_prob: { type: "float", required: true },
      validation_score: { type: "float", required: true },
      flagged: { type: "bool", required: true },
      created_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListScientificValidationsResponse",
    properties: {
      validations: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/scientific-validations/:document_id",
    method: "GET",
    name: "GetScientificValidation",
    response: "GetScientificValidationResponse",
    functionName: "getScientificValidation",
  })
  getService: any;

  @ContractService({
    path: "/api/scientific-validations",
    method: "GET",
    name: "ListScientificValidations",
    response: "ListScientificValidationsResponse",
    functionName: "listScientificValidations",
  })
  listService: any;
}
