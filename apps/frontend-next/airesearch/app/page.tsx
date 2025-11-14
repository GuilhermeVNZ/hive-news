import HomeClient from "@/components/HomeClient";
import { getArticles } from "@/lib/articles";

interface HomePageProps {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}

export default async function Home({ searchParams }: HomePageProps) {
  const resolvedSearchParams = await searchParams;
  const initialCategory = Array.isArray(resolvedSearchParams.category)
    ? resolvedSearchParams.category[0] ?? ""
    : resolvedSearchParams.category ?? "";
  const initialQuery = Array.isArray(resolvedSearchParams.q)
    ? resolvedSearchParams.q[0] ?? ""
    : resolvedSearchParams.q ?? "";

  const articles = await getArticles();

  return (
    <HomeClient
      initialArticles={articles}
      initialCategory={initialCategory}
      initialQuery={initialQuery}
    />
  );
}
