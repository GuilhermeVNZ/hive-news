import HomeClient from "@/components/HomeClient";
import { getArticles } from "@/lib/articles";

interface HomePageProps {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}

// ISR (Incremental Static Regeneration) com revalidate otimizado
// Página estática gerada em build, revalidada em background a cada 5 minutos
// Mantém TTFB baixo e visual idêntico
export const revalidate = 300; // Revalida a cada 5 minutos (ISR)

export default async function Home({ searchParams }: HomePageProps) {
  const resolvedSearchParams = await searchParams;
  const initialCategory = Array.isArray(resolvedSearchParams.category)
    ? resolvedSearchParams.category[0] ?? ""
    : resolvedSearchParams.category ?? "";
  const initialQuery = Array.isArray(resolvedSearchParams.q)
    ? resolvedSearchParams.q[0] ?? ""
    : resolvedSearchParams.q ?? "";

  // Carregar apenas 50 artigos iniciais para melhor performance
  // Mais artigos serão carregados via lazy loading quando necessário
  const { articles, hasMore, total } = await getArticles(undefined, 50, 0);

  return (
    <HomeClient
      initialArticles={articles}
      initialHasMore={hasMore}
      initialTotal={total}
      initialCategory={initialCategory}
      initialQuery={initialQuery}
    />
  );
}
