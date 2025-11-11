# Como Testar Apenas o Collector de Notícias

## Comando para Testar

```bash
cd News-main/news-backend
cargo run test-news-collector
```

Este comando executa apenas o collector de notícias (RSS/HTML), sem executar o filtro ou o writer.

## O que foi Corrigido

### 1. **Comando de Teste Isolado**
- ✅ Novo comando `test-news-collector` que executa apenas `run_collect_news_only()`
- ✅ Não executa filtro nem writer
- ✅ Permite testar apenas a coleta de notícias

### 2. **Lógica de Retentativa**
- ✅ Se um artigo foi registrado mas não tem destinations configurados, permite retentativa
- ✅ Verifica se o artigo tem destinations antes de considerar como duplicata
- ✅ Remove registro anterior se não tem destinations para permitir novo registro completo

### 3. **Tratamento de Erros**
- ✅ Se falhar ao set destinations, o artigo NÃO é considerado como "salvo completamente"
- ✅ Artigo fica registrado mas sem destinations, permitindo retentativa no próximo ciclo
- ✅ Mensagens claras indicando quando um artigo foi parcialmente salvo

## Como Funciona

### Fluxo Normal (Sucesso)
1. Coleta artigo ✅
2. Salva JSON ✅
3. Registra no registry ✅
4. Define destinations ✅
5. Marca como salvo completamente ✅

### Fluxo com Erro (Retentativa)
1. Coleta artigo ✅
2. Salva JSON ✅
3. Registra no registry ✅
4. **Falha ao definir destinations** ❌
5. Artigo fica registrado mas sem destinations ⚠️
6. **No próximo ciclo**: Detecta que artigo não tem destinations
7. Remove registro anterior e tenta novamente 🔄

## Verificação de Duplicatas

Agora a verificação de duplicatas considera:
- ✅ Se artigo tem destinations configurados → É duplicata válida (pular)
- ⚠️ Se artigo não tem destinations → Permitir retentativa (remover e tentar novamente)

## Exemplo de Uso

```bash
# Testar apenas o collector
cd News-main/news-backend
cargo run test-news-collector

# Verificar logs
# Os logs mostrarão:
# - Artigos coletados com sucesso
# - Artigos parcialmente salvos (sem destinations)
# - Artigos que serão retentados no próximo ciclo
```

## Próximos Passos

1. ✅ Testar o collector isoladamente
2. ✅ Verificar se artigos sem destinations são retentados corretamente
3. ✅ Validar que artigos com destinations não são retentados desnecessariamente







