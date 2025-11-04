# Changelog

Todas as mudanças notáveis do projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/).

## [Unreleased] - 2025-11-01

### 🎉 Adicionado

- **Pipeline Debug Mode**: Novo comando `pipeline-debug` para logging ultra-detalhado de todas as etapas (coleta, filtro, escrita, cleanup)
- **Logging Aprimorado**: Logs detalhados em tempo real para cada etapa do pipeline de news e artigos
- **Tratamento de Erros da API DeepSeek**: Logs detalhados quando a API retorna erros ou formatos inesperados
- **Suporte a Formatos JSON Alternativos**: Sistema agora aceita formatos alternativos de resposta da API DeepSeek (objeto `article` nested)
- **Limpeza Automática de Markdown**: Remoção automática de formatação markdown indesejada (**Label:**) nos artigos
- **Binário Rust para Limpeza**: Novo binário `clean-articles` para limpar formatação markdown de artigos existentes
- **Reparo Automático de JSON**: Sistema de reparo automático para `articles_registry.json` corrompido (múltiplas estratégias)

### 🔧 Melhorado

- **Logging do Pipeline de News**:
  - Logs detalhados para coleta de RSS/HTML (progresso, duplicatas, salvamento)
  - Logs detalhados para escrita de news (artigos encontrados, processamento, resultados)
  - Logs detalhados para cleanup de news (artigos limpos, estatísticas)
  - Logs com timestamps e separadores visuais para melhor leitura
  
- **Logging do Pipeline de Artigos**:
  - Logs detalhados para download de PDFs do arXiv (progresso, tamanho, tempo)
  - Logs detalhados para processamento de PDFs no writer (parsing, prompt, compressão, API)
  - Logs com separadores visuais para cada etapa
  
- **Sistema de Registry**:
  - Registry sempre salvo após cleanup, garantindo consistência
  - Reparo automático de JSON corrompido com múltiplas estratégias:
    - Trim simples
    - Busca por última chave válida `}`
    - Extração da seção `articles`
    - Backup e criação de novo registry se todas as estratégias falharem
  - Logs detalhados para cada operação de registry

- **Tratamento de Erros da API DeepSeek**:
  - Mensagens de erro detalhadas com contexto completo
  - Validação de resposta da API (choices, content, formato)
  - Logs específicos para cada tipo de erro (parse, API, empty response)
  - Reconstrução inteligente de `article_text` quando API retorna formato nested

- **Reconstrução de Article Text**:
  - Extração de todos os campos string do objeto `article` nested
  - Ordenação lógica dos campos (opening_hook → key_finding → methodology → results → conclusion)
  - Remoção de duplicatas
  - Concatenação natural sem formatação markdown indesejada

- **Limpeza de Markdown**:
  - Remoção automática de padrões `**Label:**` durante salvamento
  - Função dedicada para limpeza de formatação markdown
  - Aplicada automaticamente em todos os novos artigos

### 🐛 Corrigido

- **Cleanup de Artigos**: Registry agora sempre salvo após cleanup, mesmo quando não há mudanças de conteúdo
- **Registry JSON Corrompido**: Sistema de reparo automático previne crashes do pipeline
- **Porta do Backend**: Corrigida referência de porta 3001 para 3005 em todas as mensagens
- **Formato JSON Inesperado**: Sistema agora aceita e reconstrói corretamente formatos alternativos da API
- **Formatação Markdown Indesejada**: Removida automaticamente durante salvamento
- **Logging Insuficiente**: Pipeline agora mostra cada etapa em tempo real
- **Warnings de Compilação**: Removidos todos os warnings (unused imports, unused variables, dead code)
- **Compatibilidade SQLx**: Atualizado de versão 0.7 para 0.8 com nova API `PgPoolOptions`

### 📝 Mudanças Técnicas

- **Dependências**:
  - `sqlx`: `0.7` → `0.8` (nova API para conexões PostgreSQL)
  - Feature flags ajustados: `runtime-tokio-native-tls` → `runtime-tokio` + `tls-native-tls`
  
- **Cargo.toml**:
  - Adicionado `default-run = "news-backend"` para especificar binário padrão
  - Novo binário: `clean-articles` em `src/bin/clean_articles.rs`

- **Estrutura de Código**:
  - `#[allow(dead_code)]` adicionado para structs e métodos não utilizados (mas necessários para API futura)
  - Remoção sistemática de imports não utilizados
  - Prefixo `_` para variáveis não utilizadas

### 📚 Documentação

- Este CHANGELOG.md criado para documentar todas as mudanças
- Melhorias documentadas nos arquivos principais

---

## Formato de Versão

Usaremos [Semantic Versioning](https://semver.org/):
- **MAJOR**: Mudanças incompatíveis na API
- **MINOR**: Funcionalidades novas compatíveis
- **PATCH**: Correções de bugs compatíveis









