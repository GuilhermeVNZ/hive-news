import { Contract, ContractField } from "@cmmv/core";

/**
 * Article Contract
 * Defines the structure of a news article
 */
@Contract()
export class Article {
  @ContractField({ protoType: 'string' })
  id: string;

  @ContractField({ protoType: 'string' })
  title: string;

  @ContractField({ protoType: 'string' })
  content: string;

  @ContractField({ protoType: 'string', nullable: true })
  summary: string;

  @ContractField({ protoType: 'string' })
  sourceUrl: string;

  @ContractField({ protoType: 'string', nullable: true })
  author: string;

  @ContractField({ protoType: 'string' })
  publishedAt: Date;

  @ContractField({ protoType: 'string' })
  createdAt: Date;

  @ContractField({ protoType: 'string' })
  updatedAt: Date;

  @ContractField({ protoType: 'string', nullable: true })
  imageUrl: string;

  @ContractField({ protoType: 'string', array: true, nullable: true })
  tags: string[];

  @ContractField({ protoType: 'string', nullable: true })
  language: string;

  @ContractField({ protoType: 'string' })
  portalId: string;

  @ContractField({ protoType: 'string', nullable: true })
  category: string;

  @ContractField({ protoType: 'number' })
  rank: number;

  @ContractField({ protoType: 'string', nullable: true })
  translationStatus: string;
}
