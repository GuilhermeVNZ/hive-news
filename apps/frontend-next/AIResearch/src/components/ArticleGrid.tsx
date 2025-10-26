import ArticleCard from "./ArticleCard";

// Placeholder data - will be replaced with actual API calls
const mockArticles = [
  {
    id: "1",
    title: "Nova Arquitetura de Transformer para LLMs",
    excerpt:
      "Pesquisadores desenvolvem arquitetura inovadora que reduz significativamente o custo computacional...",
    publishedAt: "2025-10-26T10:00:00Z",
    author: "Dr. João Silva",
    category: "Machine Learning",
    readTime: 5,
  },
  {
    id: "2",
    title: "Avances em Computer Vision com Redes Neurais",
    excerpt:
      "Técnicas de visão computacional alcançam nova precisão em reconhecimento de objetos...",
    publishedAt: "2025-10-25T14:30:00Z",
    author: "Dra. Maria Santos",
    category: "Computer Vision",
    readTime: 7,
  },
  {
    id: "3",
    title: "Pesquisa em NLP para Línguas Menos Frequentes",
    excerpt:
      "Novo modelo de linguagem processa e compreende idiomas com poucos recursos disponíveis...",
    publishedAt: "2025-10-24T09:15:00Z",
    author: "Prof. Carlos Oliveira",
    category: "NLP",
    readTime: 6,
  },
];

const ArticleGrid = () => {
  return (
    <section className="container mx-auto px-4 py-12">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {mockArticles.map((article) => (
          <ArticleCard key={article.id} {...article} />
        ))}
      </div>
    </section>
  );
};

export default ArticleGrid;
