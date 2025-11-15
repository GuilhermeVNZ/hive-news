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

  // Dados são cacheados e revalidados em background (ISR)
  // Não bloqueia renderização, mantém TTFB baixo
  const articles = await getArticles();

  return (
    <HomeClient
      initialArticles={articles}
      initialCategory={initialCategory}
      initialQuery={initialQuery}
    />
  );
}
