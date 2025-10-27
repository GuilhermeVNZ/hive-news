# PHASE4: ILLUSTRATOR - Extração Automática de Imagens

## Objetivo

Extrair automaticamente as imagens dos PDFs científicos e identificar a figura mais relevante recomendada pelo DeepSeek para ilustrar o artigo no portal.

## Pipeline de Execução

```
Collector → Filter → Writer → Illustrator
   ↓           ↓        ↓          ↓
downloads  filtered  articles  featured_image.png
```

## Funcionamento

### Input
- **PDF original**: `downloads/filtered/<categoria>/<ID>.pdf`
- **Recomendação DeepSeek**: `output/<Site>/<ID>/metadata.json` → `recommended_figure`

### Processamento
1. Usar `pdfimages.exe` (poppler) para extrair todas as imagens do PDF
2. Mapear recomendação (ex: "figure_2.png") para arquivo extraído (ex: "img-001.png")
3. Copiar imagem identificada como `featured_image.png`
4. Limpar arquivos temporários

### Output
- `output/<Site>/<ID>/featured_image.png` - Imagem destacada para o artigo

## Mapeamento Inteligente

O DeepSeek recomenda figuras como:
- `"figure_2.png"`
- `"figure 2"`
- `"Fig. 2"`

O Illustrator extrai o número (2) e busca a segunda imagem do PDF (`img-001.png`, índice zero-based).

### Exemplo Prático

**Artigo 2510.21560:**
- DeepSeek recomenda: `"figure_2.png"`
- pdfimages extrai: `img-000.png, img-001.png, img-002.png, img-003.png, img-004.png`
- Nossa lógica: Figure 2 = índice 1 = `img-001.png`
- Resultado: Copiamos `img-001.png` → `featured_image.png`

**Artigo 2510.21610:**
- DeepSeek recomenda: `"figure_1.png"`
- pdfimages extrai: `img-000.png`
- Nossa lógica: Figure 1 = índice 0 = `img-000.png`
- Resultado: Copiamos `img-000.png` → `featured_image.png`

## Integração no Pipeline

O `start.rs` executa automaticamente:
1. **Collector** (coleta 10 PDFs)
2. **Filter** (aprova ~2-3 PDFs científicos)
3. **Writer** (gera artigos + conteúdo social)
4. **Illustrator** (extrai imagens) ← **NOVA FASE**

Executado automaticamente após Writer sem intervenção manual.

## Tecnologias

- **pdfimages**: Ferramenta poppler para extração de imagens em formato nativo
- **Regex**: Correspondência inteligente de nomes de figuras
- **Tokio fs**: Operações assíncronas de arquivo
- **Rust std::process::Command**: Execução de comandos externos

## Estrutura de Código

### Módulo: `news-backend/src/writer/image_extractor.rs`

```rust
// PHASE4: ILLUSTRATOR - Image Extraction Module

/// Extrai todas as imagens de um PDF usando pdfimages (poppler)
pub async fn extract_figures_from_pdf(
    pdf_path: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>>

/// Encontra a imagem correspondente à recomendação do DeepSeek
pub fn find_recommended_image(
    recommended_name: &str,
    extracted_images: &[PathBuf],
) -> Option<PathBuf>

/// Extrai o número de uma referência de figura
fn extract_figure_number(name: &str) -> Option<usize>
```

### Integração: `news-backend/src/writer/content_generator.rs`

Após geração do artigo (linha ~93), o Illustrator:

```rust
// PHASE 4: Extract and copy featured image
println!("  🖼️  Extracting images from PDF...");
match extract_figures_from_pdf(pdf_path, &output_dir).await {
    Ok(extracted_images) if !extracted_images.is_empty() => {
        // Encontrar e copiar imagem recomendada
        if let Some(recommended_img) = find_recommended_image(...) {
            tokio::fs::copy(&recommended_img, &dest_path).await?;
            println!("  ✅ Featured image saved");
        }
    }
    // ... tratamento de erros
}
```

## Estrutura de Output Final

Após execução completa do pipeline:

```
output/
└── AIResearch/
    └── 2510.21560/
        ├── article.md              (PHASE3: Writer)
        ├── linkedin.txt            (PHASE3: Writer)
        ├── x.txt                   (PHASE3: Writer)
        ├── shorts_script.txt       (PHASE3: Writer)
        ├── metadata.json           (PHASE3: Writer)
        └── featured_image.png      (PHASE4: Illustrator) ← NOVO
```

## Fluxo de Execução Detalhado

1. **User executa**: `cargo run --release -- write`
2. **start.rs** chama `run_writer()`
3. **Writer** processa cada PDF aprovado:
   - Lê PDF filtrado
   - Extrai texto e referências
   - Gera artigo via DeepSeek
   - DeepSeek recomenda figura (ex: "figure_2.png")
   - **Illustrator inicia automaticamente:**
     - Cria diretório `temp_images/`
     - Executa `pdfimages -all <pdf> temp_images/img`
     - Mapeia "figure_2" → índice 1 → `img-001.png`
     - Copia `img-001.png` para `featured_image.png`
     - Remove diretório `temp_images/`
4. **Output**: Artigo completo com imagem destacada pronta para publicação

## Logs Esperados

```
[2/2] Processing: 2510.21560.pdf
  Phase 1: Generating article (Nature/Science style)...
  📄 Parsing PDF...
  🖼️  Finding figure references...
  📁 Saving to: G:/Hive-Hub/News-main/output\AIResearch\2510.21560
  📝 Building article prompt for: AIResearch
  🗜️  Compressing prompt (~13448 tokens)...
  ✅ Compressed to 8667 tokens (35.6% savings)
  🤖 Sending to DeepSeek API...
  ✅ Article generated
  
  🖼️  Extracting images from PDF...
  ✅ Extracted 5 images
  ✅ Featured image saved: G:/Hive-Hub/News-main/output\AIResearch\2510.21560\featured_image.png
  
  📱 Building social media prompts...
  🗜️  Compressing social prompt (~1832 tokens)...
  ✅ Compressed to 1202 tokens (34.4% savings)
  🤖 Generating social content...
  ✅ Social content generated
  
  💾 Saving content to disk...
  ✅ Content saved → G:/Hive-Hub/News-main/output\AIResearch\2510.21560
     Tokens: 15282 → 9869 (35.0% savings)
```

## Tratamento de Erros

O Illustrator é **não-bloqueante**: se a extração de imagens falhar, o pipeline continua e o artigo é salvo sem a imagem destacada.

### Cenários de Erro:

1. **pdfimages.exe não encontrado**:
   ```
   ⚠️  Image extraction failed: pdfimages.exe not found
   ```

2. **PDF sem imagens**:
   ```
   ⚠️  No images found in PDF
   ```

3. **Figura recomendada não encontrada**:
   ```
   ⚠️  Could not find recommended image: figure_2.png
       Available: 3 images extracted
   ```

4. **Falha na cópia**:
   ```
   ⚠️  Image extraction failed: Failed to copy featured image
   ```

Em todos os casos, o artigo é salvo normalmente sem a imagem.

## Benefícios

1. **Automático**: Integrado no pipeline, sem ação manual
2. **Inteligente**: Mapeia recomendação do DeepSeek para imagem real do PDF
3. **Robusto**: Usa pdfimages (ferramenta padrão da indústria para extração de imagens)
4. **Limpo**: Remove arquivos temporários automaticamente
5. **Informativo**: Logs claros em cada etapa do processo
6. **Não-bloqueante**: Erros de imagem não impedem publicação do artigo

## Requisitos Técnicos

### Dependências Externas
- **poppler-utils**: Já instalado em `apps/Release-25.07.0-0/poppler-25.07.0/`
- **pdfimages.exe**: Disponível em `Library/bin/pdfimages.exe`

### Dependências Rust
- `tokio::fs` - Operações assíncronas de arquivo
- `regex` - Correspondência de padrões de texto
- `std::process::Command` - Execução de comandos externos
- `anyhow` - Tratamento de erros

## Implementação

### Arquivos a Criar/Modificar

1. ✅ **CRIADO**: `docs/PHASE4_ILLUSTRATOR.md` - Esta documentação
2. ⏳ **MODIFICAR**: `news-backend/src/writer/image_extractor.rs` - Implementar funções de extração
3. ⏳ **MODIFICAR**: `news-backend/src/writer/content_generator.rs` - Integrar Illustrator no fluxo
4. ⏳ **MODIFICAR**: `start.rs` - Atualizar documentação do pipeline

### Tarefas de Implementação

- [ ] Reescrever `image_extractor.rs` com funções completas:
  - [ ] `extract_figures_from_pdf()` - Extração via pdfimages
  - [ ] `find_recommended_image()` - Mapeamento inteligente
  - [ ] `extract_figure_number()` - Parsing de números
  - [ ] Testes unitários
  
- [ ] Integrar no `content_generator.rs`:
  - [ ] Adicionar chamada após geração do artigo
  - [ ] Atualizar imports
  - [ ] Adicionar tratamento de erros
  
- [ ] Atualizar `start.rs`:
  - [ ] Documentar PHASE4 em `run_writer()`
  - [ ] Atualizar `show_help()`
  
- [ ] Testar pipeline completo:
  - [ ] Executar com 2510.21560.pdf
  - [ ] Executar com 2510.21610.pdf
  - [ ] Verificar `featured_image.png` criada
  - [ ] Verificar `temp_images/` removido
  
- [ ] Atualizar documentação relacionada:
  - [ ] Mencionar PHASE4 em `PHASE3_WRITER.md`
  - [ ] Atualizar README principal se necessário

## Próximos Passos

Após implementação do PHASE4, o sistema estará completo para publicação automatizada:

1. **Collector** busca papers recentes
2. **Filter** valida qualidade científica
3. **Writer** gera conteúdo editorial
4. **Illustrator** adiciona imagem destacada
5. **Frontend** (futuro) publica automaticamente no portal

## Notas de Desenvolvimento

- O mapeamento assume que pdfimages extrai imagens na ordem em que aparecem no PDF
- Figuras numeradas no texto (Figure 1, Figure 2, etc.) correspondem a essa ordem
- Se houver imagens decorativas antes das figuras, o mapeamento pode precisar de ajuste manual
- Para PDFs complexos, considerar análise de legendas no futuro (PHASE5?)

## Referências

- **Poppler Utils**: https://poppler.freedesktop.org/
- **pdfimages manual**: https://linux.die.net/man/1/pdfimages
- **Projeto News System**: `G:/Hive-Hub/News-main/`
- **PHASE1**: `docs/PHASE1_COLLECTOR.md`
- **PHASE2**: `docs/PHASE2_FILTER.md`
- **PHASE3**: `docs/PHASE3_WRITER.md`


