# 🔍 Debug: Problemas no Writer de PDF

## Problemas Identificados

### 1. **Texto Extraído do PDF pode estar sempre vazio ou sempre o mesmo**

**Localização:** `src/writer/content_generator.rs` - `process_pdf()`

**Problema:**
- O texto extraído do PDF pode estar sempre vazio, fazendo com que todos os PDFs sejam processados sem conteúdo
- Ou o texto pode estar sempre sendo o mesmo (bug na extração)

**Correções Aplicadas:**
- ✅ Adicionados logs detalhados mostrando:
  - Tamanho do texto extraído (`📊 Extracted text length: {} characters`)
  - Preview dos primeiros 200 caracteres (`🔍 Text preview (first 200 chars)`)
  - Hash do texto para verificar unicidade (`🔍 Text hash (for uniqueness check)`)
  - Verificação se texto está vazio (retorna erro crítico)
  - Aviso se texto é muito curto (< 100 caracteres)

**Próximos Passos:**
- Verificar logs do servidor para ver se:
  - Textos estão sendo extraídos corretamente
  - Textos são diferentes entre PDFs
  - Textos não estão vazios

---

### 2. **Prompts não estão sendo randomizados corretamente**

**Localização:** `src/writer/prompts.rs` - `load_random_article_prompt()`

**Problema:**
- Os prompts podem não estar sendo randomizados corretamente
- Pode estar sempre usando o mesmo prompt

**Correções Aplicadas:**
- ✅ Adicionados logs mostrando qual prompt foi selecionado (`🎲 Using randomized prompt: {}`)
- ✅ Hash do texto do prompt para verificar se é diferente

**Verificações Necessárias:**
- Verificar se o diretório `src/writer/prompts/article_randomizer/` contém múltiplos arquivos `.txt`
- Verificar se o gerador aleatório está funcionando corretamente
- Verificar logs para ver se diferentes prompts estão sendo usados

---

### 3. **Erros silenciosos impedindo geração**

**Localização:** `src/writer/content_generator.rs` - `process_pdf()`

**Problema:**
- Erros podem estar sendo suprimidos silenciosamente
- Falhas na extração de texto podem não estar sendo reportadas

**Correções Aplicadas:**
- ✅ Adicionado tratamento de erro explícito para `parse_pdf()` com logs detalhados
- ✅ Retorno de erro crítico se texto extraído estiver vazio
- ✅ Logs detalhados de cada etapa do processo

---

### 4. **Nenhum dos 9 PDFs gerou conteúdo**

**Problema:**
- Os 9 PDFs que passaram no filtro não geraram nenhum conteúdo

**Possíveis Causas:**
1. **Texto extraído está vazio** - o `parse_pdf_text()` está falhando e retornando string vazia
2. **Writer não está sendo chamado** - mas verificado que o loop chama `run_writer_pipeline()`
3. **Erros silenciosos** - o writer está falhando mas os erros não estão sendo reportados
4. **Textos sempre iguais** - todos os PDFs estão retornando o mesmo texto (bug)

**Correções Aplicadas:**
- ✅ Logs detalhados para identificar qual das causas acima está acontecendo
- ✅ Verificação explícita se texto está vazio antes de processar
- ✅ Logs de cada etapa (extração, prompt building, API call, salvamento)

---

## Estrutura do Writer de PDF

### Fluxo:

```
LoopManager (automático)
  └─> run_articles_pipeline()
       ├─> collect (coleta PDFs do arXiv)
       ├─> filter (filtra PDFs científicos)
       └─> write (processa PDFs aprovados)
            └─> run_writer_pipeline()
                 └─> WriterService.process_pdf()
                      ├─> parse_pdf() [extrai texto]
                      ├─> load_random_article_prompt() [seleciona prompt aleatório]
                      ├─> DeepSeek API [gera artigo]
                      └─> save_content() [salva arquivos]
```

---

## Logs Adicionados para Debug

### No `process_pdf()`:

```rust
// 1. Após extrair texto do PDF
println!("  ✅ PDF parsed successfully");
println!("  📊 Extracted text length: {} characters", p.text.len());
println!("  📝 Title extracted: {}", p.title);
println!("  🔍 Text preview (first 200 chars): {}", text_preview);
println!("  🔍 Text hash (for uniqueness check): {}", text_hash);

// 2. Verificações críticas
if p.text.is_empty() {
    eprintln!("  ❌ CRITICAL: Extracted text is EMPTY!");
    return Err(...);
}

// 3. Ao construir prompt
println!("  📊 Paper text length: {} characters", parsed.text.len());
println!("  🔍 Text hash (for uniqueness check): {}", text_hash);
println!("  📊 Final prompt length: {} characters", prompt.len());
```

---

## Próximas Ações

1. **Executar o writer manualmente** e verificar os logs:
   ```bash
   cargo run --bin news-backend write
   ```

2. **Verificar os logs** para identificar:
   - Se textos estão sendo extraídos (não vazios)
   - Se textos são diferentes entre PDFs (hashes diferentes)
   - Se prompts estão sendo randomizados (arquivos diferentes)
   - Se há erros sendo reportados

3. **Se texto estiver sempre vazio:**
   - Verificar se `pdftotext.exe` existe no caminho esperado
   - Verificar se `lopdf` está funcionando corretamente
   - Verificar permissões de leitura dos PDFs

4. **Se texto estiver sempre igual:**
   - Verificar se há cache sendo usado
   - Verificar se `parse_pdf()` está reutilizando o mesmo resultado
   - Verificar se todos os PDFs realmente contêm texto

---

## Arquivos Modificados

- ✅ `src/writer/content_generator.rs` - Adicionados logs detalhados e verificações
- 📝 `docs/DEBUG_PDF_WRITER.md` - Este documento

---

## Como Usar

1. Compile o backend:
   ```bash
   cd news-backend
   cargo build --release --bin news-backend
   ```

2. Execute o writer manualmente para ver os logs:
   ```bash
   cargo run --bin news-backend write
   ```

3. Analise os logs procurando por:
   - `❌ CRITICAL: Extracted text is EMPTY!`
   - `⚠️ WARNING: Extracted text is very short`
   - Hash do texto (deve ser diferente para cada PDF)
   - Nome do arquivo de prompt usado (deve variar)

