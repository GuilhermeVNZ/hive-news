# Fase II - Filtragem e Validação Científica

## 🎯 Status Atual (Última Atualização: 2025-10-27)

### ✅ Implementado e Funcionando

#### 1. Estrutura Completa de Módulos
- **11 módulos** criados em `news-backend/src/filter/`
- Pipeline integrado via `start.rs` → `cargo run --bin start collector`
- Busca recursiva de PDFs funcionando corretamente

**Módulos:**
- `parser.rs` - Extração de texto com pdftotext
- `source_detector.rs` - Detecção científica vs blog
- `experiments.rs` - Detecção de testes em resultados
- `validator.rs` - Validação de DOIs
- `authors.rs` - Verificação de autores
- `fake_detector.rs` - Detecção de padrões IA
- `categorizer.rs` - Classificação por categoria
- `scorer.rs` - Cálculo de score científico
- `cache.rs` - Cache em memória
- `pipeline.rs` - Orquestrador principal

#### 2. Extração de Texto com pdftotext ✅
- **Instalado**: `apps/Release-25.07.0-0/poppler-25.07.0/Library/bin/pdftotext.exe`
- **Sistema multi-estratégia**: pdftotext → lopdf → bytes brutos
- **Extração**: 100% funcional
- **Threshold**: 0.5 (ajustável)

#### 3. Movimentação Automática de Arquivos ✅
- **Aprovados**: Movidos para `downloads/filtered/<categoria>/`
- **Rejeitados**: Movidos para `downloads/rejected/`
- **Categorias**: machine-learning, nlp, robotics, computer-vision, theory, security, general
- **Criação automática**: Diretórios criados quando necessário

#### 4. Integração Completa
- Filtro executa automaticamente após coleta
- Logs detalhados de progresso
- Estatísticas completas (Approved, Rejected, Skipped, Total)

---

## 🔬 Teste de Execução

```bash
cargo run --bin start collector
```

### Resultados (10 PDFs - Teste mais recente)
- ✅ **2 aprovados** (20%) → `machine-learning`
  - `2510.21638v1.pdf`
  - `2510.21652v1.pdf`
- ❌ **8 rejeitados** (80%) → `downloads/rejected/`

### Resultados (70 PDFs - Teste completo anterior)
- ✅ **11 aprovados** (15.7%)
- ❌ **59 rejeitados** (84.3%)
- Categorias identificadas: machine-learning, nlp, robotics, general

---

## Pipeline de Execução

### Fluxo Completo

```
1. Coletar PDFs do arXiv
   ↓
2. Extrair texto com pdftotext
   ↓
3. Detectar tipo de fonte (científica vs blog)
   ↓
4. Se blog → Skip (não processar)
   ↓
5. Se científico → Aplicar filtros:
   - Verificar testes em resultados
   - Validar DOIs
   - Verificar autores
   - Detectar padrões IA-fake
   ↓
6. Calcular score científico
   - doi_ratio * 0.4
   - has_tests * 0.3
   - author_ratio * 0.2
   - fake_penalty * 0.1
   ↓
7. Score >= 0.5 → APROVADO
   ↓
8. Mover para categoria apropriada
   ↓
9. Score < 0.5 → REJEITADO
   ↓
10. Mover para /rejected/
```

---

## Fórmula de Score

```rust
score = (doi_ratio * 0.4) + (has_tests * 0.3) + (author_ratio * 0.2) + ((1.0 - fake_penalty) * 0.1)

Critérios:
- doi_ratio: Proporção de DOIs válidos (0.0-1.0)
- has_tests: Presença de testes em resultados (bool)
- author_ratio: Proporção de autores com ORCID (0.0-1.0)
- fake_penalty: Penalidade por padrões IA-fake (0.0-1.0)

Aprovação: score >= 0.5
```

---

## Estrutura de Diretórios

```
downloads/
├── arxiv/              # PDFs originais do arXiv
│   └── 2025-10-27/
├── filtered/           # Artigos aprovados por categoria
│   ├── machine-learning/
│   ├── nlp/
│   ├── computer-vision/
│   ├── robotics/
│   ├── theory/
│   ├── security/
│   └── general/
├── rejected/           # Artigos rejeitados (score < 0.5)
├── cache/              # Cache de validações (JSON)
└── temp/               # Arquivos temporários
```

---

## Categorias Disponíveis

- **machine-learning**: ML, deep learning, neural networks
- **nlp**: Natural language processing, language models
- **computer-vision**: Image processing, object detection
- **robotics**: Robotics, autonomous systems
- **theory**: Theoretical, complexity, algorithms
- **security**: Security, cryptography, privacy
- **general**: Outros temas

---

## ⚠️ Pendências (Futuras Melhorias)

### 1. Validação via APIs Reais
- Implementar chamadas reais a CrossRef, Semantic Scholar, PubMed
- Atualmente retorna valores placeholder (0.5)
- Adicionar tratamento de erros e timeouts

### 2. Cache Persistente
- Implementar salvamento em JSON flat files
- Atualmente cache em memória (DashMap)
- Implementar carregamento de cache ao iniciar

### 3. Testes Unitários
- Criar `news-backend/tests/filter_test.rs`
- Testar detecção de fonte, parsing, validação
- Testar pipeline completo com PDFs reais

### 4. Paralelização
- Implementar pipeline paralelo com rayon + tokio
- Atualmente sequencial (30 PDFs = ~30s)
- Objetivo: 30 PDFs em ~15-20s com 32 threads

---

## Performance Atual

- **Com pdftotext**: ~1s por PDF (extração robusta)
- **30 PDFs científicos**: ~30s (sequencial)
- **Taxa de aprovação**: 15-20% (threshold 0.5)
- **Memória**: Mínima (estruturas otimizadas)
- **Concorrência**: Sequencial (pronto para rayon + tokio)

---

## Configuração

### Dependências (Cargo.toml)

```toml
[dependencies]
lopdf = "0.34"
rayon = "1.10"
dashmap = "6.1"
futures = "0.3"
lazy_static = "1.5"
urlencoding = "2.1"
regex = "1.10"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### pdftotext

**Localização**: `apps/Release-25.07.0-0/poppler-25.07.0/Library/bin/pdftotext.exe`

**Caminho no código**: `news-backend/src/filter/parser.rs`
```rust
let pdftotext_path = "G:/Hive-Hub/News-main/apps/Release-25.07.0-0/poppler-25.07.0/Library/bin/pdftotext.exe";
```

---

## Troubleshooting

1. **pdftotext não encontrado**
   - Verificar se o executável existe no caminho
   - Sistema usará fallback para lopdf automaticamente

2. **Arquivos não movidos**
   - Verificar permissões de escrita em `downloads/`
   - Verificar se diretórios existem

3. **Score sempre baixo**
   - Threshold pode estar muito alto (atual: 0.5)
   - Verificar se PDFs contêm testes em resultados
   - Verificar validação de DOIs

4. **Cache não persiste**
   - Sistema usa cache em memória por enquanto
   - Implementar cache em JSON na próxima versão

---

## Próximos Passos

1. ✅ Parsing com pdftotext - **CONCLUÍDO**
2. ✅ Movimentação automática - **CONCLUÍDO**
3. ✅ Threshold 0.5 - **CONCLUÍDO**
4. ⏳ Validação via APIs reais
5. ⏳ Cache persistente
6. ⏳ Paralelização com rayon
7. ⏳ Testes unitários
