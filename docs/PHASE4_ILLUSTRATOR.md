# PHASE4: ILLUSTRATOR - Extração Automática de Imagens

## Objetivo

Extrair automaticamente imagens da primeira página do PDF científico para criar dois assets:
1. **Banner** (`banner_<id>.png`) - Terço superior da primeira página (header visual)
2. **Page** (`page_<id>.png`) - Primeira página completa em alta resolução

NOTA: A partir de 2025-10-27, não usamos mais a extração de figuras do DeepSeek. Usamos a primeira página do PDF como imagem padrão.

## Pipeline de Execução

```
Collector → Filter → Writer → Illustrator
   ↓           ↓        ↓          ↓
downloads  filtered  articles  banner + page.png
```

## Funcionamento

### Input
- **PDF original**: `downloads/filtered/<categoria>/<ID>.pdf`

### Processamento
1. Usar `pdftoppm.exe` (poppler) para converter primeira página em PNG
2. Carregar PNG com Rust `image` crate
3. Recortar terço superior para criar banner
4. Salvar ambos: banner e página completa
5. Limpar arquivos temporários

### Output
- `output/<Site>/<ID>/banner_<id>.png` - Banner (topo da página)
- `output/<Site>/<ID>/page_<id>.png` - Página completa

NOTA: O `<id>` tem `.` substituído por `_` para evitar problemas de path (ex: `2510_21610.png`).

## Como Funciona

### Fluxo de Extração

1. **pdftoppm** converte a primeira página do PDF para PNG (150 DPI)
2. **Rust `image` crate** carrega o PNG em memória
3. **Crop** recorta o terço superior da página para o banner
4. **Salva** ambos: banner e página completa
5. **Cleanup** remove temporários (`temp_page*.png`)

### Exemplo Prático

**Artigo 2510.21610:**
- Input: `downloads/filtered/machine-learning/2510.21610.pdf`
- pdftoppm gera: `temp_page-1.png` ou `temp_page-01.png`
- Banner: Corta altura/3 do topo → `banner_2510_21610.png`
- Page: Imagem completa → `page_2510_21610.png`
- Output: `output/AIResearch/2510.21610/banner_2510_21610.png` e `page_2510_21610.png`

### Anti-Duplicação

O Illustrator verifica se os arquivos já existem antes de processar:
- Se `banner_<id>.png` E `page_<id>.png` existem → pula extração
- Isso evita reprocessamento desnecessário

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
    └── 2510.21610/
        ├── article.md              (PHASE3: Writer)
        ├── linkedin.txt            (PHASE3: Writer)
        ├── x.txt                   (PHASE3: Writer)
        ├── shorts_script.txt       (PHASE3: Writer)
        ├── banner_2510_21610.png   (PHASE4: Illustrator) ← Banner (topo)
        └── page_2510_21610.png     (PHASE4: Illustrator) ← Página completa
```

NOTA: **Não geramos mais `metadata.json`** - removido por não ser necessário.

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
[1/4] Processing: 2510.21610.pdf
  Phase 1: Generating article (Nature/Science style)...
  📄 Parsing PDF...
  📁 Saving to: G:/Hive-Hub/News-main/output\AIResearch\2510.21610
  📝 Building article prompt for: AIResearch
  🗜️  Compressing prompt (~9252 tokens)...
  ✅ Compressed to 5926 tokens (36.0% savings)
  🤖 Sending to DeepSeek API...
  ✅ Article generated
  
  🖼️  Extracting first page images (banner + full page)...
  ✅ Banner saved: G:/Hive-Hub/News-main/output\AIResearch\2510.21610\banner_2510_21610.png
  ✅ Full page saved: G:/Hive-Hub/News-main/output\AIResearch\2510.21610\page_2510_21610.png
  
  📱 Building social media prompts...
  🗜️  Compressing social prompt (~1316 tokens)...
  ✅ Compressed to 804 tokens (39.0% savings)
  🤖 Generating social content...
  ✅ Social content generated
  
  💾 Saving content to disk...
  ✅ Content saved → G:/Hive-Hub/News-main/output\AIResearch\2510.21610
     Tokens: 10570 → 6730 (37.5% savings)
```

## Tratamento de Erros

O Illustrator é **não-bloqueante**: se a extração de imagens falhar, o pipeline continua e o artigo é salvo sem as imagens.

### Cenários de Erro:

1. **pdftoppm.exe não encontrado**:
   ```
   Error: pdftoppm.exe not found at: G:/Hive-Hub/News-main/apps/Release-25.07.0-0/poppler-25.07.0/Library/bin/pdftoppm.exe
   ⚠️  Image extraction failed
   ```

2. **Falha na conversão**:
   ```
   Error: pdftoppm failed: <stderr output>
   ⚠️  Image extraction failed
   ```

3. **Arquivo temporário não encontrado**:
   ```
   Error: Failed to generate first page image - no output file found
   ```

4. **Anti-duplicação**:
   ```
   ⏭️  Images already exist (banner + page)
   ```
   (Imagens são ignoradas se já existem)

Em todos os casos, o artigo é salvo normalmente e o pipeline continua para o próximo artigo.

## Benefícios

1. **Automático**: Integrado no pipeline, sem ação manual
2. **Padronizado**: Usa a primeira página como imagem padrão (não depende de recomendações)
3. **Duplo Output**: Gera banner E página completa para maior flexibilidade
4. **Anti-Duplicação**: Não reprocessa se imagens já existem
5. **Limpo**: Remove arquivos temporários automaticamente
6. **Informativo**: Logs claros em cada etapa do processo
7. **Não-bloqueante**: Erros de imagem não impedem publicação do artigo

## Requisitos Técnicos

### Dependências Externas
- **poppler-utils**: Já instalado em `apps/Release-25.07.0-0/poppler-25.07.0/`
- **pdftoppm.exe**: Disponível em `Library/bin/pdftoppm.exe`

### Dependências Rust
- `image = "0.24"` - Processamento de imagens (crop, save)
- `tokio::fs` - Operações assíncronas de arquivo
- `std::process::Command` - Execução de comandos externos (pdftoppm)
- `anyhow` - Tratamento de erros

### Estrutura de Código

```rust
// PHASE4: Illustrator - Image Extraction from First Page

/// Extrai banner (topo) e página completa da primeira página do PDF
pub async fn extract_first_page_images(
    pdf_path: &Path,
    output_dir: &Path,
    article_id: &str,
) -> Result<(PathBuf, PathBuf)>
```

## Implementação

### Arquivos Criados/Modificados

1. ✅ **MODIFICADO**: `news-backend/src/writer/illustrator.rs` - Renomeado de `image_extractor.rs`
   - Função `extract_first_page_images()` implementada
   - Usa `pdftoppm.exe` para conversão PDF→PNG
   - Usa `image` crate para crop e save
   - Anti-duplicação implementada

2. ✅ **MODIFICADO**: `news-backend/src/writer/content_generator.rs`
   - Integração com Illustrator após geração do artigo
   - Anti-duplicação: verifica se imagens já existem
   - Tratamento de erros não-bloqueante
   - Não cria pasta antes de verificar se vai processar

3. ✅ **MODIFICADO**: `news-backend/src/writer/prompt_compressor.rs`
   - Preserva instrução JSON quando compressão remove "json"
   - Fix para erro DeepSeek API `response_format`

4. ✅ **MODIFICADO**: `news-backend/src/writer/prompts.rs`
   - Removido prompt para figuras (não usamos mais)
   - Simplificado JSON output (só title e article_text)

5. ✅ **MODIFICADO**: `news-backend/src/writer/deepseek_client.rs`
   - Adicionado `response_format: { "type": "json_object" }`
   - Melhor tratamento de erros
   - Logs de debug expandidos

6. ✅ **MODIFICADO**: `news-backend/Cargo.toml`
   - Adicionado `image = "0.24"` dependency

7. ✅ **MODIFICADO**: `docs/PHASE4_ILLUSTRATOR.md` - Esta documentação

### Testes Realizados

- ✅ Testado com 2510.21610.pdf
- ✅ Testado com 2510.21560.pdf
- ✅ Testado com 2510.21638.pdf (difícil)
- ✅ Testado com 2510.21652.pdf
- ✅ Verificado `banner_<id>.png` criada
- ✅ Verificado `page_<id>.png` criada
- ✅ Verificado `temp_page*.png` removido
- ✅ Verificado anti-duplicação funcionando
- ✅ Verificado que não gera `metadata.json` mais

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


