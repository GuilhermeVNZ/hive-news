import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Clock, Calendar, User, ArrowLeft, Share2, BookOpen } from "lucide-react";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Button } from "@/components/ui/button";

// Mock data - will be replaced with actual API calls
const articles: Record<string, any> = {
  "nova-arquitetura-de-transformer-para-llms": {
    title: "Nova Arquitetura de Transformer para LLMs",
    category: "Machine Learning",
    author: "Dr. João Silva",
    publishedAt: "2025-10-26",
    readTime: 5,
    excerpt: "Pesquisadores desenvolvem arquitetura inovadora que reduz significativamente o custo computacional necessário para treinar e operar modelos de linguagem grandes (LLMs).",
    content: `
# Nova Arquitetura de Transformer para LLMs

## Introdução

Pesquisadores do MIT e da Stanford desenvolveram uma arquitetura de transformer inovadora que reduz significativamente o custo computacional necessário para treinar e operar modelos de linguagem grandes (LLMs).

## O Problema Atual

Os modelos de linguagem tradicionais, como GPT-4, requerem trilhões de operações para processar texto. Isso resulta em:
- Alto custo de treinamento
- Alto consumo de energia
- Latência significativa em inferência
- Necessidade de hardware especializado

## A Solução

A nova arquitetura, chamada **EfficientTransformer**, implementa várias otimizações:

### 1. Attention Esparsa
Em vez de computar attention entre todos os tokens, o modelo usa uma estratégia hierárquica onde apenas tokens-chave recebem atenção completa.

### 2. Compressão de Embeddings
Os embeddings são comprimidos usando técnicas de quantização adaptativa, reduzindo o uso de memória em até 60%.

### 3. Pipeline Paralelo
Novo esquema de paralelização permite treinar modelos maiores usando menos GPUs.

## Resultados

Experimentos em datasets padrão mostraram:
- **70% menos memória** durante inferência
- **45% mais rápido** para textos longos
- Qualidade **equivalente ou superior** a modelos tradicionais

## Implicações

Esta arquitetura permite que organizações menores tenham acesso a LLMs poderosos, democratizando a inteligência artificial e permitindo mais pesquisadores a explorar novas fronteiras.

## Conclusão

A EfficientTransformer representa um marco importante na evolução dos modelos de linguagem, equilibrando eficiência e desempenho de forma inovadora.
    `,
  },
  "avances-em-computer-vision-com-redes-neurais": {
    title: "Avances em Computer Vision com Redes Neurais",
    category: "Computer Vision",
    author: "Dra. Maria Santos",
    publishedAt: "2025-10-25",
    readTime: 7,
    excerpt: "Técnicas de visão computacional alcançam nova precisão em reconhecimento de objetos em tempo real.",
    content: `
# Avances em Computer Vision com Redes Neurais

## Introdução

Novas arquiteturas de redes neurais profundas estão revolucionando o campo de visão computacional, alcançando níveis de precisão sem precedentes em tarefas complexas.

## Breakthroughs Recentes

### Reconhecimento de Objetos em Tempo Real

Sistemas baseados em **YOLO v8** e **EfficientDet** conseguem detectar e classificar objetos com **95% de precisão** em vídeos de alta resolução processando em tempo real.

### Segmentação Semântica

Técnicas de segmentação avançada permitem identificar com precisão cada pixel de uma imagem, facilitando aplicações em:
- Diagnóstico médico
- Automação industrial
- Veículos autônomos

## Aplicações Práticas

### Saúde
- Detecção precoce de câncer em imagens médicas
- Análise automatizada de raios-X

### Automotivo
- Sistemas de assistência ao condutor
- Detecção de pedestres e sinais

### Manufatura
- Inspeção de qualidade automatizada
- Controle de produção

## Impacto

Essas inovações estão tornando a visão computacional mais acessível e confiável para aplicações do mundo real.
    `,
  },
  "pesquisa-em-nlp-para-linguas-menos-frequentes": {
    title: "Pesquisa em NLP para Línguas Menos Frequentes",
    category: "NLP",
    author: "Prof. Carlos Oliveira",
    publishedAt: "2025-10-24",
    readTime: 6,
    excerpt: "Novo modelo de linguagem processa e compreende idiomas com poucos recursos disponíveis.",
    content: `
# Pesquisa em NLP para Línguas Menos Frequentes

## Desafio

A maioria dos modelos de linguagem é treinada com foco em línguas de alta disponibilidade de dados, deixando milhares de idiomas sem suporte adequado.

## Solução

Novos modelos desenvolvidos usando transferência de aprendizado e few-shot learning mostram resultados promissores:

### Multilingual BERT Expandido
- Suporte para 100+ idiomas
- Arquitetura otimizada para baixos recursos
- Treinamento eficiente com dados limitados

### Aplicações
- Preservação cultural
- Acesso à informação
- Educação multilíngue

## Resultados

Modelos conseguem atingir 80% da performance de sistemas dedicados usando apenas 10% dos dados normalmente necessários.

Isso abre portas para inclusão digital de comunidades linguísticas menos representadas.
    `,
  },
  "reinforcement-learning-em-jogos-complexos": {
    title: "Reinforcement Learning em Jogos Complexos",
    category: "Robótica",
    author: "Dr. Ana Costa",
    publishedAt: "2025-10-23",
    readTime: 8,
    excerpt: "Agentes de IA superam jogadores humanos em jogos estratégicos de longa duração através de RL avançado.",
    content: `
# Reinforcement Learning em Jogos Complexos

## Introdução

Agentes de IA baseados em **Deep Reinforcement Learning** estão alcançando níveis de jogabilidade humana em jogos de estratégia complexos.

## Arquitetura

### Agent Alpha
Usa uma combinação de:
- Monte Carlo Tree Search (MCTS)
- Deep Neural Networks
- Self-play contínuo

## Desafios Superados

### Jogos de Informação Imperfeita
Modelos conseguem lidar com **informação incompleta** ao longo da partida, como em poker e bridge.

### Horizonte Temporal Longo
Estratégias que abrangem centenas de movimentos à frente.

## Aplicações Além de Jogos

- Otimização de logística
- Gestão de recursos
- Planejamento estratégico empresarial

## Próximos Passos

Explorando aplicações em:
- Veículos autônomos
- Sistemas de recomendação
- Robótica autônoma

O futuro mostra que RL será fundamental para sistemas inteligentes verdadeiramente adaptativos.
    `,
  },
  "generative-ai-criacao-de-conteudo-multimodal": {
    title: "Generative AI: Criação de Conteúdo Multimodal",
    category: "Generative AI",
    author: "Dra. Patricia Lima",
    publishedAt: "2025-10-22",
    readTime: 5,
    excerpt: "Novos modelos gerativos combinam texto, imagens e áudio para criar experiências imersivas.",
    content: `
# Generative AI: Criação de Conteúdo Multimodal

## Era Multimodal

Modelos de IA generativa estão se tornando verdadeiramente **multimodais**, capaz de criar e compreender múltiplos formatos simultaneamente.

## Geração Cross-Modal

### Text-to-Image-to-Video
- Entrada: descrição em texto
- Saída: vídeo gerado com alta qualidade

### Audio-to-Image
Geração de imagens a partir de descrições em áudio.

## Aplicações Criativas

- **Filmes e Animação**: Geração rápida de storyboards
- **Música**: Composição assistida por IA
- **Design**: Protótipos visuais instantâneos

## Desafios Técnicos

### Sincronização
Garantir que múltiplos modais estejam alinhados (áudio sincronizado com vídeo, por exemplo).

### Qualidade
Manter alta fidelidade em todos os formatos gerados.

## Futuro

Esses avanços estão democratizando a criação de conteúdo profissional, permitindo que qualquer pessoa seja um criador.
    `,
  },
  "etica-em-ia-desafios-e-melhores-praticas": {
    title: "Ética em IA: Desafios e Melhores Práticas",
    category: "AI Ethics",
    author: "Prof. Ricardo Mendes",
    publishedAt: "2025-10-21",
    readTime: 7,
    excerpt: "Especialistas discutem framework ético para desenvolvimento responsável de sistemas de IA.",
    content: `
# Ética em IA: Desafios e Melhores Práticas

## Necessidade Urgente

Com o crescimento acelerado da IA, questões éticas tornaram-se críticas para garantir benefícios para toda humanidade.

## Principais Preocupações

### Viés e Discriminação
- Modelos que perpetuam estereótipos sociais
- Sistemas que discriminam minorias
- Necessidade de datasets diversos

### Privacidade
- Uso não autorizado de dados pessoais
- Surveillance por IA
- Consentimento informado

### Transparência
- "Black box" em decisões automatizadas
- Explicabilidade de algoritmos
- Accountability

## Frameworks Existentes

### Princípios Asilomar
Guidelines estabelecidos por pesquisadores de IA líderes.

### UNESCO Ethics Recommendations
Framework internacional para governança de IA.

## Melhores Práticas

1. **Testes Contínuos de Viés**
2. **Transparência em Algoritmos**
3. **Inclusão de Stakeholders Diversos**
4. **Auditoria Regular**
5. **Trabalho Multidisciplinar**

## Conclusão

Ética não é um obstáculo para inovação, mas sim uma **necessidade** para garantir que a IA beneficie a humanidade de forma justa e equitativa.
    `,
  },
  "neural-architecture-search-automatizado": {
    title: "Neural Architecture Search Automatizado",
    category: "Machine Learning",
    author: "Dr. Luis Ferreira",
    publishedAt: "2025-10-20",
    readTime: 6,
    excerpt: "Técnicas de busca automática de arquiteturas neurais otimizam modelos com menor intervenção humana.",
    content: `
# Neural Architecture Search Automatizado

## Automação de Design

NAS (Neural Architecture Search) está revolucionando como projetamos redes neurais, automatizando o processo de design de arquitetura.

## Como Funciona

### Busca Automática
Algoritmos exploram milhões de possíveis arquiteturas:
- Combinações de camadas
- Conexões entre neurônios
- Funções de ativação
- Padrões de pooling

### Avaliação Eficiente
Usa técnicas como:
- Progressive search
- Early stopping
- Transfer learning

## Vantagens

### Eficiência
Encontra arquiteturas otimizadas em **dias** vs **meses** de trabalho manual.

### Performance
Muitas vezes supera designs manuais experientes.

### Acesso
Permite que não-especialistas criem redes de alta performance.

## Aplicações

- Computer vision
- NLP
- Speech recognition
- Recommendation systems

## Futuro

NAS está se tornando padrão em desenvolvimento de IA profissional, democratizando a criação de modelos state-of-the-art.
    `,
  },
  "edge-ai-inteligencia-em-dispositivos-iot": {
    title: "Edge AI: Inteligência em Dispositivos IoT",
    category: "IoT",
    author: "Dra. Fernanda Rocha",
    publishedAt: "2025-10-19",
    readTime: 5,
    excerpt: "Processamento de IA diretamente em dispositivos edge reduz latência e melhora privacidade.",
    content: `
# Edge AI: Inteligência em Dispositivos IoT

## Revolução Edge

IA não precisa mais estar na nuvem. Dispositivos IoT estão se tornando cada vez mais inteligentes com processamento **local**.

## Vantagens do Edge AI

### Latência Ultra-Baixa
Decisões em **milissegundos** vs segundos da nuvem.

### Privacidade Melhorada
Dados nunca saem do dispositivo local.

### Funcionamento Offline
Dispositivos funcionam mesmo sem conectividade.

### Redução de Custos
Menos transmissão de dados = economia de banda.

## Aplicações Práticas

### Saúde Wearable
- Monitoramento contínuo de sinais vitais
- Detecção de anomalias em tempo real

### Smart Homes
- Reconhecimento de voz local
- Detecção de intrusos

### Indústria 4.0
- Manutenção preditiva
- Controle de qualidade automatizado

## Desafios Técnicos

### Recursos Limitados
Como otimizar modelos para:
- **Baixo consumo de energia**
- **Pouca memória**
- **CPU limitada**

### Soluções
- Quantização (INT8, INT4)
- Pruning de redes
- Knowledge distillation
- Modelos especializados

## Impacto

Edge AI está tornando tecnologia inteligente **acessível, confiável e privada** em todos os lugares.
    `,
  },
  "transformers-em-analise-de-series-temporais": {
    title: "Transformers em Análise de Séries Temporais",
    category: "Machine Learning",
    author: "Prof. Gustavo Almeida",
    publishedAt: "2025-10-18",
    readTime: 6,
    excerpt: "Modelos Transformer adaptados para análise de dados temporais mostram resultados promissores.",
    content: `
# Transformers em Análise de Séries Temporais

## Novo Horizonte

Arquitetura Transformer, revolucionária em NLP, está mostrando excelência em **predição de séries temporais**.

## Desafios Tradicionais

Modelos tradicionais (ARIMA, LSTM) têm limitações:
- Dificuldade com dependências de longo prazo
- Necessidade de feature engineering extensivo
- Sensibilidade a ruído

## A Solução Transformer

### Temporal Transformers
Arquiteturas adaptadas para dados temporais:
- **Temporal Attention**: Foco em patterns temporais
- **Positional Encoding**: Captura ordem temporal
- **Multi-head Attention**: Identifica múltiplos padrões

## Aplicações

### Finanças
- Previsão de ações
- Detecção de fraude
- Análise de risco

### Energia
- Previsão de demanda
- Otimização de grid
- Energia renovável

### Manufatura
- Previsão de falhas
- Otimização de produção
- Controle de qualidade

## Resultados

### Performance Superior
- **15-30% melhor** que LSTM em diversas tasks
- Superior em séries com **alta variância**
- Excelente em dados **multivariáveis**

### Mas nem tudo são flores...
Alguns desafios permanecem:
- Treinamento computacionalmente caro
- Necessidade de grandes datasets
- Interpretabilidade ainda limitada

## Conclusão

Transformers estão abrindo novas possibilidades para análise temporal, com aplicações promissoras em múltiplos domínios.
    `,
  },
};

export default async function ArticlePage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const article = articles[slug];

  if (!article) {
    notFound();
  }

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        {/* Article Hero Section */}
        <div className="relative bg-gradient-to-br from-primary/5 via-background to-background py-12">
          <div className="container mx-auto px-4">
            {/* Back Button */}
            <Link
              href="/"
              className="inline-flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors mb-8 group"
            >
              <ArrowLeft className="h-4 w-4 group-hover:-translate-x-1 transition-transform" />
              <span>Voltar para notícias</span>
            </Link>

            {/* Category Badge */}
            <div className="mb-6">
              <span className="px-4 py-2 text-sm font-semibold text-primary bg-primary/10 rounded-full border border-primary/20 inline-flex items-center gap-2">
                <BookOpen className="h-3.5 w-3.5" />
                {article.category}
              </span>
            </div>

            {/* Title */}
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6 text-foreground leading-tight">
              {article.title}
            </h1>

            {/* Excerpt */}
            {article.excerpt && (
              <p className="text-xl text-muted-foreground mb-8 max-w-3xl leading-relaxed">
                {article.excerpt}
              </p>
            )}

            {/* Meta Information */}
            <div className="flex flex-wrap items-center gap-6 pb-8 border-b border-border">
              <div className="flex items-center gap-2 text-muted-foreground">
                <User className="h-4 w-4" />
                <span className="text-sm font-medium">{article.author}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <Calendar className="h-4 w-4" />
                <span className="text-sm">{new Date(article.publishedAt).toLocaleDateString('pt-BR', { day: '2-digit', month: 'long', year: 'numeric' })}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <Clock className="h-4 w-4" />
                <span className="text-sm">{article.readTime} min de leitura</span>
              </div>
              <div className="flex-1" />
              <Button variant="outline" size="sm" className="gap-2">
                <Share2 className="h-4 w-4" />
                Compartilhar
              </Button>
            </div>
          </div>
        </div>

        {/* Article Content */}
        <article className="container mx-auto px-4 py-12 max-w-4xl">

          {/* Article Image Placeholder */}
          <div className="mb-12">
            <div className="aspect-video bg-gradient-to-br from-primary/20 via-primary/10 to-background rounded-2xl flex items-center justify-center border border-border">
              <div className="text-center p-12">
                <div className="h-24 w-24 mx-auto mb-6 bg-primary/10 rounded-full flex items-center justify-center">
                  <BookOpen className="h-12 w-12 text-primary" />
                </div>
                <p className="text-muted-foreground">Imagem do artigo</p>
              </div>
            </div>
          </div>

          {/* Content */}
          <div className="prose prose-lg max-w-none">
            <div className="article-content whitespace-pre-wrap leading-relaxed">
              {article.content.trim()}
            </div>
          </div>

          {/* Related Articles */}
          <div className="mt-16 pt-12 border-t border-border">
            <h2 className="text-2xl font-bold mb-6 bg-gradient-to-r from-foreground to-primary bg-clip-text text-transparent">
              Artigos Relacionados
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Placeholder for related articles */}
              <div className="p-6 border border-border rounded-lg hover:border-primary transition-colors">
                <span className="text-xs font-medium text-primary bg-primary/10 px-2 py-1 rounded">
                  Machine Learning
                </span>
                <h3 className="mt-2 font-semibold text-lg hover:text-primary cursor-pointer">
                  Neural Architecture Search Automatizado
                </h3>
                <p className="mt-2 text-sm text-muted-foreground line-clamp-2">
                  Técnicas de busca automática de arquiteturas neurais...
                </p>
              </div>
              <div className="p-6 border border-border rounded-lg hover:border-primary transition-colors">
                <span className="text-xs font-medium text-primary bg-primary/10 px-2 py-1 rounded">
                  LLMs
                </span>
                <h3 className="mt-2 font-semibold text-lg hover:text-primary cursor-pointer">
                  Fine-tuning de Modelos de Linguagem
                </h3>
                <p className="mt-2 text-sm text-muted-foreground line-clamp-2">
                  Como adaptar LLMs para tarefas específicas com poucos dados...
                </p>
              </div>
            </div>
          </div>
        </article>
      </main>
      <Footer />
    </div>
  );
}

