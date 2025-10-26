import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsEnum, IsDate } from "@cmmv/core";

/**
 * Job Contract
 * Scheduler jobs and their status
 */
@Contract({
  namespace: "Job",
  controllerName: "Job",
  protoPackage: "job",
  options: {
    databaseSchemaName: "jobs",
  },
})
export class JobContract {
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
  type: string;

  @ContractField({
    protoType: "enum",
    validations: [{ type: "IsEnum", value: ["pending", "running", "completed", "failed"] }],
  })
  status: "pending" | "running" | "completed" | "failed";

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  scheduled_at: Date;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  executed_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetJobResponse",
    properties: {
      id: { type: "string", required: true },
      portal_id: { type: "string", required: true },
      type: { type: "string", required: true },
      status: { type: "enum", required: true },
      scheduled_at: { type: "date", required: true },
      executed_at: { type: "date", required: false },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListJobsResponse",
    properties: {
      jobs: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/jobs/:id",
    method: "GET",
    name: "GetJob",
    response: "GetJobResponse",
    functionName: "getJob",
  })
  getService: any;

  @ContractService({
    path: "/api/jobs",
    method: "GET",
    name: "ListJobs",
    response: "ListJobsResponse",
    functionName: "listJobs",
  })
  listService: any;
}
