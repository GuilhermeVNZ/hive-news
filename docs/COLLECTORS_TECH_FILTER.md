# Filtros de Tecnologia para Collectors (PMC e Semantic Scholar)

## 📋 Resumo

Os collectors de **PubMed Central (PMC)** e **Semantic Scholar** agora estão configurados para buscar **apenas artigos relacionados a tecnologia**, filtrando automaticamente artigos de outras áreas (medicina, biologia, química, etc.).

## 🔧 Configuração

### PubMed Central (PMC)

#### Filtros Aplicados

O collector do PMC agora busca apenas artigos que contenham termos de tecnologia em título/abstract:

**Termos de Busca:**
- artificial intelligence
- machine learning
- deep learning
- neural network
- computer vision
- natural language processing
- NLP
- data science
- computer science
- software engineering
- programming
- algorithm
- deep neural network
- transformer
- reinforcement learning

**Query de Busca:**
```
("artificial intelligence"[Title/Abstract] OR 
 "machine learning"[Title/Abstract] OR 
 "deep learning"[Title/Abstract] OR 
 "neural network"[Title/Abstract] OR 
 "computer vision"[Title/Abstract] OR 
 "natural language processing"[Title/Abstract] OR 
 "NLP"[Title/Abstract] OR 
 "data science"[Title/Abstract] OR 
 "computer science"[Title/Abstract] OR 
 "software engineering"[Title/Abstract] OR 
 "programming"[Title/Abstract] OR 
 "algorithm"[Title/Abstract] OR 
 "deep neural network"[Title/Abstract] OR 
 "transformer"[Title/Abstract] OR 
 "reinforcement learning"[Title/Abstract])
```

**Configuração Padrão:**
- Time range: 30 dias (últimos 30 dias)
- Sort: Por data de publicação (mais recente primeiro)
- Database: PMC

### Semantic Scholar

#### Filtros Aplicados

O collector do Semantic Scholar filtra por **campos de estudo (fieldsOfStudy)** relacionados a tecnologia:

**Campos de Estudo Permitidos:**
- Computer Science
- Artificial Intelligence
- Machine Learning
- Computer Vision
- Natural Language Processing
- Data Science
- Software Engineering

**Query Padrão:**
```
"computer science artificial intelligence machine learning deep learning"
```

**Filtragem em Duas Etapas:**
1. **Por campo de estudo**: Verifica se o paper tem `fieldsOfStudy` relacionado a tecnologia
2. **Por título/abstract**: Se não tiver `fieldsOfStudy`, verifica palavras-chave no título/abstract

**Palavras-chave no Título/Abstract:**
- artificial intelligence
- machine learning
- deep learning
- neural network
- computer vision
- natural language processing
- computer science
- data science
- algorithm
- software engineering

## 📝 Configuração JSON

### PMC (`collectors_config.json`)

```json
{
  "id": "pmc",
  "name": "PubMed Central",
  "enabled": false,
  "api_key": null,
  "config": {
    "filter_technology_only": true,
    "search_terms": [
      "artificial intelligence",
      "machine learning",
      "deep learning",
      "neural network",
      "computer vision",
      "natural language processing",
      "data science",
      "computer science",
      "software engineering",
      "programming",
      "algorithm",
      "transformer",
      "reinforcement learning"
    ],
    "time_range_days": 30
  }
}
```

### Semantic Scholar (`collectors_config.json`)

```json
{
  "id": "semantic_scholar",
  "name": "Semantic Scholar",
  "enabled": false,
  "api_key": null,
  "config": {
    "filter_technology_only": true,
    "fields_of_study": [
      "Computer Science",
      "Artificial Intelligence",
      "Machine Learning",
      "Computer Vision",
      "Natural Language Processing",
      "Data Science",
      "Software Engineering"
    ],
    "query": "computer science artificial intelligence machine learning",
    "max_results": 10
  }
}
```

## 🎯 Como Funciona

### PubMed Central (PMC)

1. **Constrói query de busca** com termos de tecnologia
2. **Busca na API do PMC** usando os termos
3. **Retorna apenas artigos** que contenham termos de tecnologia em título/abstract
4. **Artigos de outras áreas** (medicina, biologia, etc.) são automaticamente excluídos

### Semantic Scholar

1. **Executa busca** com query de tecnologia
2. **Filtra por campos de estudo**: Verifica se `fieldsOfStudy` contém categorias de tecnologia
3. **Filtro secundário**: Se não tiver `fieldsOfStudy`, verifica palavras-chave no título/abstract
4. **Retorna apenas artigos de tecnologia**: Garante que apenas CS/AI/ML sejam incluídos

## ✅ Resultado

**Apenas artigos relacionados a tecnologia** são coletados dos collectors PMC e Semantic Scholar:

- ✅ Artigos de AI/ML são coletados
- ✅ Artigos de Computer Science são coletados
- ✅ Artigos de Data Science são coletados
- ✅ Artigos de NLP são coletados
- ✅ Artigos de Computer Vision são coletados
- ❌ Artigos de medicina são excluídos
- ❌ Artigos de biologia são excluídos
- ❌ Artigos de química são excluídos
- ❌ Artigos de física não relacionada a CS são excluídos

## 🔄 Como Modificar Filtros

### Via Dashboard

1. Acesse o Dashboard
2. Vá em **Sites** → Selecione um site
3. Configure **Collectors**
4. Edite o collector **PMC** ou **Semantic Scholar**
5. Modifique o campo `config` para adicionar/remover termos de busca

### Via Código

Edite `site_config_manager.rs` e modifique os termos em `config`:

```rust
config: serde_json::json!({
    "filter_technology_only": true,
    "search_terms": ["artificial intelligence", "machine learning", ...],
    "time_range_days": 30,
}),
```

## 📚 Exemplo de Uso

Quando você ativar os collectors PMC ou Semantic Scholar:

1. **Eles buscarão artigos** das respectivas APIs
2. **Aplicarão filtros de tecnologia** automaticamente
3. **Retornarão apenas artigos relevantes** para tecnologia/AI/ML

**Não é necessário configuração adicional** - os filtros estão ativos por padrão quando os collectors são habilitados.

## 🛠️ Detalhes Técnicos

### PMC Collector

- **Arquivo**: `news-backend/src/collectors/pmc_collector.rs`
- **Método**: `fetch_recent_papers()`
- **API**: PubMed Central E-utilities
- **Endpoint**: `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi`

### Semantic Scholar Collector

- **Arquivo**: `news-backend/src/collectors/semantic_scholar_collector.rs`
- **Método**: `fetch_recent_papers()`
- **API**: Semantic Scholar Graph API
- **Endpoint**: `https://api.semanticscholar.org/graph/v1/paper/search`

## ⚠️ Notas Importantes

1. **API Keys**: Semantic Scholar pode requerer API key para maiores volumes de requests
2. **Rate Limiting**: Ambos os collectors respeitam rate limits das APIs
3. **PDF Downloads**: Apenas PDFs open access são baixados do Semantic Scholar
4. **Cache**: Resultados podem ser cacheados para melhorar performance

## 📊 Comparação

| Aspecto | PMC | Semantic Scholar |
|--------|-----|------------------|
| **Filtro Principal** | Título/Abstract | Fields of Study |
| **Filtro Secundário** | N/A | Título/Abstract |
| **Time Range** | 30 dias (configurável) | Sem limite de tempo |
| **Ordenação** | Por data (mais recente) | Por data (mais recente) |
| **API Key Necessária** | Não | Opcional (recomendada) |
| **PDF Open Access** | Sim | Sim (se disponível) |























































