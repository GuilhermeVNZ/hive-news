# 📚 Documentação do News System

## 📋 Documentos Principais

### 🎯 [`ORCHESTRATOR.md`](./ORCHESTRATOR.md)
Documentação completa do orquestrador `start.rs`:
- Visão geral e função principal
- Arquitetura e componentes
- Fluxo de inicialização
- Interação com Collector
- Comandos disponíveis
- Testes e debugging

### 🧱 [`PHASE1_COLLECTOR.md`](./PHASE1_COLLECTOR.md)
Documentação do Collector Service:
- Objetivo e fluxo
- Estrutura de downloads (por origem e data)
- Implementação arXiv (10 papers)
- Configuração de chaves API (todas as fontes)
- Fontes disponíveis (arXiv, Nature, Science, PubMed, etc.)
- Como usar e testar

### 🏗️ [`ARCHITECTURE.md`](./ARCHITECTURE.md)
Arquitetura geral do sistema:
- Componentes principais
- Fluxo de dados
- Integrações
- Diagramas

### 🧪 [`TESTING_GUIDE.md`](./TESTING_GUIDE.md)
Guia de testes:
- Como executar testes
- Cobertura esperada
- Testes unitários e de integração

### ✍️ [`PHASE3_WRITER.md`](./PHASE3_WRITER.md)
Documentação do Writer (geração de conteúdo):
- Geração de artigos com DeepSeek API
- Prompts customizados por site
- Limpeza automática de markdown
- Tratamento de erros da API
- Suporte a formatos JSON alternativos

### 🧹 [`PHASE4_CLEANUP.md`](./PHASE4_CLEANUP.md)
Documentação do Cleanup:
- Limpeza automática de PDFs
- Sistema de registry
- Reparo automático de JSON corrompido
- Logging aprimorado

---

## 📝 Changelog

Todas as mudanças do projeto estão documentadas em:
- **Root:** [`../CHANGELOG.md`](../CHANGELOG.md) - Mudanças recentes e histórico completo

---

## 🎯 Quick Start

```bash
cd G:\Hive-Hub\News-main

# Ver status do collector
cargo run --bin start collector

# Iniciar sistema completo
cargo run --bin start start
```

---

**📚 Toda a documentação está centralizada e organizada para referência rápida!**

