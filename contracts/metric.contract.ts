import { Contract, ContractField, ContractMessage, ContractService } from "@cmmv/core";
import { IsString, IsNotEmpty, IsNumber, IsDate, Min } from "@cmmv/core";

/**
 * Metric Contract
 * Engagement metrics and analytics
 */
@Contract({
  namespace: "Metric",
  controllerName: "Metric",
  protoPackage: "metric",
  options: {
    databaseSchemaName: "metrics",
  },
})
export class MetricContract {
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
    protoType: "int",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }],
  })
  views: number;

  @ContractField({
    protoType: "int",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }],
  })
  clicks: number;

  @ContractField({
    protoType: "float",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }],
  })
  ctr: number;

  @ContractField({
    protoType: "int",
    validations: [{ type: "IsNumber" }, { type: "Min", value: 0 }],
  })
  average_read_time: number;

  @ContractField({
    protoType: "date",
    validations: [{ type: "IsDate" }],
  })
  updated_at: Date;

  // CMMV Messages
  @ContractMessage({
    name: "GetMetricResponse",
    properties: {
      id: { type: "string", required: true },
      article_id: { type: "string", required: true },
      views: { type: "int", required: true },
      clicks: { type: "int", required: true },
      ctr: { type: "float", required: true },
      average_read_time: { type: "int", required: true },
      updated_at: { type: "date", required: true },
    },
  })
  getResponse: any;

  @ContractMessage({
    name: "ListMetricsResponse",
    properties: {
      metrics: { type: "simpleArray", required: true },
      total: { type: "int", required: true },
    },
  })
  listResponse: any;

  // CMMV Services
  @ContractService({
    path: "/api/metrics/:id",
    method: "GET",
    name: "GetMetric",
    response: "GetMetricResponse",
    functionName: "getMetric",
  })
  getService: any;

  @ContractService({
    path: "/api/metrics",
    method: "GET",
    name: "ListMetrics",
    response: "ListMetricsResponse",
    functionName: "listMetrics",
  })
  listService: any;
}
