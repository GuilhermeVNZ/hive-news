export interface Article {
  id: string;
  slug: string;
  title: string;
  excerpt: string;
  article: string;
  publishedAt: string;
  author: string;
  category: string;
  readTime: number;
  imageCategories: string[];
  imagePath?: string;
  isPromotional?: boolean;
  featured?: boolean;
  hidden?: boolean;
  linkedinPost?: string; // LinkedIn post content from linkedin.txt
  xPost?: string; // X/Twitter post content from x.txt
  sourceUrl?: string; // Original article URL used by the writer
}

export type ArticleCollection = Article[];

export interface ArticleApiResponse {
  articles: Article[];
}
