# Filtro de Tecnologia para APIs de Educação

## 📋 Resumo

As APIs de educação agora estão configuradas para buscar **apenas cursos relacionados a tecnologia**, filtrando automaticamente cursos que não sejam de:

- Computer Science
- Artificial Intelligence
- Machine Learning
- Data Science
- Programming
- NLP (Natural Language Processing)
- Computer Vision
- Software Development

## 🔧 Configuração

### Backend (`site_config_manager.rs`)

As APIs de educação têm configuração padrão com filtros de tecnologia:

#### edX
```json
{
  "categories": ["computer-science", "artificial-intelligence", "data-science", "programming"],
  "filter_technology_only": true
}
```

#### MIT OpenCourseWare
```json
{
  "departments": ["Electrical-Engineering-and-Computer-Science", "Computer-Science"],
  "filter_technology_only": true
}
```

#### Class Central
```json
{
  "categories": ["computer-science", "ai", "programming"],
  "filter_technology_only": true
}
```

### Frontend (`courses/route.ts`)

A API já filtra automaticamente:
- Por categorias de tecnologia
- Por palavras-chave no título (AI, Machine Learning, Deep Learning, Computer Science, Programming)
- Garante que apenas cursos relacionados a tecnologia sejam retornados

## 📝 Implementação

### Categorias de Tecnologia Suportadas

1. **Machine Learning**
2. **Artificial Intelligence**
3. **Introdução à IA**
4. **NLP** (Natural Language Processing)
5. **Computer Vision**
6. **Certificação** (certificações de tecnologia)
7. **Data Science**
8. **Programming**
9. **Computer Science**

### Filtros Implementados

#### Por Categoria
- Filtra automaticamente cursos com categorias relacionadas a tecnologia
- Remove cursos de outras áreas (medicina, engenharia civil, etc.)

#### Por Título
- Busca palavras-chave no título:
  - "ai" / "artificial intelligence"
  - "machine learning"
  - "deep learning"
  - "computer science"
  - "programming"
  - "data science"

#### Por Descrição (futuro)
- Pode ser expandido para filtrar por palavras-chave na descrição

## 🎯 Como Funciona

1. **Quando a API é chamada**, ela busca cursos de todas as fontes
2. **Antes de retornar**, filtra automaticamente para incluir apenas:
   - Cursos com categorias de tecnologia OU
   - Cursos com títulos contendo palavras-chave de tecnologia
3. **O frontend também filtra** para garantir dupla proteção

## ✅ Resultado

Apenas cursos relacionados a **tecnologia** são exibidos na página de educação do AIResearch.

## 🔄 Como Modificar Filtros

### Via Dashboard

1. Acesse o Dashboard
2. Vá em **Sites** → Selecione um site
3. Configure **Education APIs**
4. Edite o campo `config` para adicionar/remover categorias

### Via Código

Edite `site_config_manager.rs` e modifique as categorias em `config`:

```rust
config: serde_json::json!({
    "categories": ["computer-science", "artificial-intelligence", ...],
    "filter_technology_only": true,
}),
```

## 📚 Exemplo de Uso

Quando você ativar as APIs de educação e elas começarem a buscar cursos, elas automaticamente:

1. Buscarão cursos das APIs (edX, MIT OCW, Class Central)
2. Filtrarão apenas cursos de tecnologia
3. Retornarão apenas cursos relevantes

Não é necessário configuração adicional - o filtro está ativo por padrão.





























































