import { Contract, ContractField } from "@cmmv/core";
import { IsString, IsNotEmpty, IsDate } from "@cmmv/core";

/**
 * Vector Contract
 * Handles Float32Array serialization fix (store as string)
 */
@Contract({
  namespace: "Vector",
  controllerName: "Vector",
  protoPackage: "vector",
  options: {
    databaseSchemaName: "vectors",
  },
})
export class VectorContract {
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
  doc_ref: string;

  @ContractField({
    protoType: "string",
    validations: [{ type: "IsString" }, { type: "IsNotEmpty" }],
    transform: ({ value }) => {
      if (value instanceof Float32Array) {
        return Array.from(value).join(",");
      }
      return value;
    },
    toPlain: ({ value }) => {
      if (typeof value === "string") {
        return new Float32Array(value.split(",").map(parseFloat));
      }
      return value;
    },
  })
  vector: string;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  created_at: Date;
}
