import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsArray, IsDate } from "@cmmv/core";

/**
 * Document Contract
 * Post-processes Vectorizer output, extracting metadata
 */
@Contract({
  namespace: "Document",
  controllerName: "Document",
  protoPackage: "document",
  options: {
    databaseSchemaName: "documents",
  },
})
export class DocumentContract {
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
  title: string;

  @ContractField({
    protoType: "string",
    array: true,
    validations: [{ type: "IsArray" }],
  })
  authors: string[];

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  abstract: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
  })
  source_url: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }],
  })
  vector_id: string;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  published_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetDocumentResponse",
    properties: {
      id: { type: "string", required: true },
      portal_id: { type: "string", required: true },
      title: { type: "string", required: true },
      authors: { type: "simpleArray", required: true },
      abstract: { type: "string", required: true },
      source_url: { type: "string", required: true },
      vector_id: { type: "string", required: true },
      published_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListDocumentsResponse",
    properties: {
      documents: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/documents/:id",
    method: "GET",
    name: "GetDocument",
    response: "GetDocumentResponse",
    functionName: "getDocument",
  })
  getService: any;

  @ContractService({
    path: "/api/documents",
    method: "GET",
    name: "ListDocuments",
    response: "ListDocumentsResponse",
    functionName: "listDocuments",
  })
  listService: any;
}
