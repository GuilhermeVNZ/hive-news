import { Contract, ContractField } from "@cmmv/core";

/**
 * Source Type
 */
export enum SourceType {
  RSS = "rss",
  API = "api",
  SCRAPER = "scraper",
}

/**
 * Source Contract
 * Defines the structure of a content source
 */
@Contract()
export class Source {
  @ContractField({ protoType: 'string' })
  id: string;

  @ContractField({ protoType: 'string' })
  name: string;

  @ContractField({ protoType: 'string' })
  type: SourceType;

  @ContractField({ protoType: 'string' })
  url: string;

  @ContractField({ protoType: 'string', nullable: true })
  description: string;

  @ContractField({ protoType: 'bool' })
  enabled: boolean;

  @ContractField({ protoType: 'string', nullable: true })
  apiKey: string;

  @ContractField({ protoType: 'string', nullable: true })
  headers: string;

  @ContractField({ protoType: 'string', nullable: true })
  portalId: string;

  @ContractField({ protoType: 'string', nullable: true })
  category: string;

  @ContractField({ protoType: 'string' })
  cronExpression: string;

  @ContractField({ protoType: 'string', nullable: true })
  lastCheckAt: Date;

  @ContractField({ protoType: 'string', nullable: true })
  lastSuccessAt: Date;

  @ContractField({ protoType: 'number' })
  errorCount: number;
}
