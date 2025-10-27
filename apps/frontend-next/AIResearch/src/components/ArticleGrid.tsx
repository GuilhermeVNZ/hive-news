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
  {
    id: "4",
    title: "Reinforcement Learning em Jogos Complexos",
    excerpt:
      "Agentes de IA superam jogadores humanos em jogos estratégicos de longa duração através de RL avançado...",
    publishedAt: "2025-10-23T16:20:00Z",
    author: "Dr. Ana Costa",
    category: "Robótica",
    readTime: 8,
  },
  {
    id: "5",
    title: "Generative AI: Criação de Conteúdo Multimodal",
    excerpt:
      "Novos modelos gerativos combinam texto, imagens e áudio para criar experiências imersivas...",
    publishedAt: "2025-10-22T11:45:00Z",
    author: "Dra. Patricia Lima",
    category: "Generative AI",
    readTime: 5,
  },
  {
    id: "6",
    title: "Ética em IA: Desafios e Melhores Práticas",
    excerpt:
      "Especialistas discutem framework ético para desenvolvimento responsável de sistemas de IA...",
    publishedAt: "2025-10-21T13:30:00Z",
    author: "Prof. Ricardo Mendes",
    category: "AI Ethics",
    readTime: 7,
  },
  {
    id: "7",
    title: "Neural Architecture Search Automatizado",
    excerpt:
      "Técnicas de busca automática de arquiteturas neurais otimizam modelos com menor intervenção humana...",
    publishedAt: "2025-10-20T09:00:00Z",
    author: "Dr. Luis Ferreira",
    category: "Machine Learning",
    readTime: 6,
  },
  {
    id: "8",
    title: "Edge AI: Inteligência em Dispositivos IoT",
    excerpt:
      "Processamento de IA diretamente em dispositivos edge reduz latência e melhora privacidade...",
    publishedAt: "2025-10-19T15:15:00Z",
    author: "Dra. Fernanda Rocha",
    category: "IoT",
    readTime: 5,
  },
  {
    id: "9",
    title: "Transformers em Análise de Séries Temporais",
    excerpt:
      "Modelos Transformer adaptados para análise de dados temporais mostram resultados promissores...",
    publishedAt: "2025-10-18T10:30:00Z",
    author: "Prof. Gustavo Almeida",
    category: "Machine Learning",
    readTime: 6,
  },
];

interface ArticleGridProps {
  selectedCategory?: string;
}

const ArticleGrid = ({ selectedCategory }: ArticleGridProps) => {
  const filteredArticles = selectedCategory 
    ? mockArticles.filter(article => article.category === selectedCategory)
    : mockArticles;

  return (
    <section className="container mx-auto px-4 py-16" id="articles">
      <div className="mb-10">
        <h2 className="text-3xl md:text-4xl font-bold mb-3 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
          {selectedCategory ? `Artigos em ${selectedCategory}` : "Artigos em Destaque"}
        </h2>
        <p className="text-muted-foreground text-lg">
          {selectedCategory 
            ? `${filteredArticles.length} ${filteredArticles.length === 1 ? 'artigo encontrado' : 'artigos encontrados'}`
            : "Explore as últimas pesquisas e desenvolvimentos em IA"
          }
        </p>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredArticles.map((article, index) => (
          <div 
            key={article.id}
            className="animate-fade-in-up"
            style={{ animationDelay: `${index * 150}ms`, animationFillMode: 'both' }}
          >
            <ArticleCard {...article} />
          </div>
        ))}
      </div>
    </section>
  );
};

export default ArticleGrid;
