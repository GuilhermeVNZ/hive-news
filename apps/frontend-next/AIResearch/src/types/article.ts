export interface Article {
  id: string;
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
}

export type ArticleCollection = Article[];

export interface ArticleApiResponse {
  articles: Article[];
}

