# 📊 Comandos para Analisar Uso de Espaço no Servidor

## 🚀 Script Automático (Recomendado)

Execute o script completo de análise:

```bash
cd ~/hive-news
chmod +x scripts/analyze-disk-usage.sh
./scripts/analyze-disk-usage.sh
```

## 📋 Comandos Manuais Úteis

### 1. Espaço Total do Sistema

```bash
# Ver espaço total
df -h

# Ver apenas o disco principal
df -h /

# Ver em formato mais legível
df -h | grep -E '^/dev|Filesystem'
```

### 2. Top Diretórios no Projeto

```bash
# Top 10 diretórios (1 nível)
du -sh * | sort -rh | head -10

# Top 20 diretórios (1 nível)
du -sh * | sort -rh | head -20

# Top 10 com profundidade 2
du -h --max-depth=2 | sort -rh | head -10

# Análise completa do projeto
du -h --max-depth=1 . | sort -rh
```

### 3. Análise por Diretório Específico

#### Downloads
```bash
# Tamanho total
du -sh downloads/

# Tamanho por subdiretório
du -sh downloads/*

# Contar PDFs
find downloads/ -name "*.pdf" -type f | wc -l

# Tamanho total dos PDFs
find downloads/ -name "*.pdf" -type f -exec du -ch {} + | tail -1

# PDFs por data (mais antigos primeiro)
find downloads/ -name "*.pdf" -type f -printf '%T+ %p\n' | sort | head -10
```

#### Output (Artigos)
```bash
# Tamanho total
du -sh output/

# Tamanho por site
du -sh output/*

# Contar artigos
find output/ -name "article.md" -type f | wc -l

# Artigos por site
for site in output/*/; do
    echo "$(basename $site): $(find "$site" -name "article.md" -type f | wc -l) artigos"
done
```

#### Images
```bash
# Tamanho total
du -sh images/

# Por tipo de imagem
find images/ -name "*.jpg" -o -name "*.jpeg" | wc -l
find images/ -name "*.png" | wc -l
find images/ -name "*.webp" | wc -l

# Tamanho por tipo
find images/ \( -name "*.jpg" -o -name "*.jpeg" \) -exec du -ch {} + | tail -1
find images/ -name "*.png" -exec du -ch {} + | tail -1
find images/ -name "*.webp" -exec du -ch {} + | tail -1
```

#### Logs
```bash
# Tamanho total
du -sh logs/

# Contar arquivos de log
find logs/ -type f | wc -l

# Top 10 maiores logs
find logs/ -type f -exec du -h {} + | sort -rh | head -10

# Logs antigos (>30 dias)
find logs/ -type f -mtime +30 | wc -l
find logs/ -type f -mtime +30 -exec du -ch {} + | tail -1
```

### 4. Top Arquivos Mais Grandes

```bash
# Top 20 arquivos maiores
find . -type f -exec du -h {} + | sort -rh | head -20

# Top 10 arquivos maiores (excluindo node_modules e target)
find . -type f -not -path "*/node_modules/*" -not -path "*/target/*" -exec du -h {} + | sort -rh | head -10

# Arquivos maiores que 100MB
find . -type f -size +100M -exec du -h {} + | sort -rh
```

### 5. Análise por Tipo de Arquivo

```bash
# PDFs
find . -name "*.pdf" -type f -exec du -ch {} + | tail -1

# Imagens
find . \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" -o -name "*.webp" \) -type f -exec du -ch {} + | tail -1

# Logs
find . -name "*.log" -type f -exec du -ch {} + | tail -1

# JSON
find . -name "*.json" -type f -exec du -ch {} + | tail -1
```

### 6. Docker

```bash
# Espaço usado pelo Docker
docker system df

# Detalhado
docker system df -v

# Tamanho das imagens
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

# Tamanho dos volumes
docker volume ls
docker system df -v | grep -A 10 "VOLUME NAME"

# Limpar recursos não usados
docker system prune -a --volumes
```

### 7. Rust Build Artifacts (target/)

```bash
# Tamanho do target
du -sh target/ 2>/dev/null || du -sh news-backend/target/

# Tamanho por tipo
du -sh target/release/ 2>/dev/null
du -sh target/debug/ 2>/dev/null

# Limpar (cuidado: vai remover builds)
cd news-backend && cargo clean
```

### 8. Node Modules

```bash
# Encontrar todos os node_modules
find . -name "node_modules" -type d

# Tamanho de cada node_modules
find . -name "node_modules" -type d -exec du -sh {} +

# Total de node_modules
find . -name "node_modules" -type d -exec du -ch {} + | tail -1
```

## 🔍 Comandos de Investigação

### Verificar o que está ocupando mais espaço

```bash
# Análise completa em uma linha
du -h --max-depth=1 . | sort -rh | head -20

# Verificar um diretório específico
du -h --max-depth=2 downloads/ | sort -rh | head -10

# Verificar arquivos grandes em um diretório
find downloads/ -type f -size +10M -exec ls -lh {} + | awk '{print $5, $9}' | sort -rh
```

### Comparar antes/depois de limpeza

```bash
# Antes da limpeza
du -sh . > /tmp/disk-before.txt

# Depois da limpeza
du -sh . > /tmp/disk-after.txt

# Comparar
diff /tmp/disk-before.txt /tmp/disk-after.txt
```

## 💡 Comandos Rápidos (One-Liners)

```bash
# Espaço total usado pelo projeto
du -sh .

# Top 10 diretórios
du -sh * | sort -rh | head -10

# Top 10 arquivos maiores
find . -type f -exec du -h {} + | sort -rh | head -10

# Contar PDFs e tamanho
echo "PDFs: $(find downloads/ -name '*.pdf' -type f | wc -l) arquivos ($(find downloads/ -name '*.pdf' -type f -exec du -ch {} + | tail -1 | cut -f1))"

# Espaço usado pelo Docker
docker system df

# Espaço usado por logs antigos (>30 dias)
find logs/ -type f -mtime +30 -exec du -ch {} + | tail -1
```

## 📊 Interpretação dos Resultados

### Baseado na sua saída atual:

```
577M    downloads    → PDFs baixados (pode limpar com cleanup-disk.sh)
398M    apps         → Frontends (normal, necessário)
191M    target       → Build artifacts Rust (pode limpar com cargo clean)
177M    images       → Imagens (normal, necessário)
89M     output       → Artigos gerados (normal, necessário)
40M     news-backend → Código fonte (normal, necessário)
```

### Oportunidades de Limpeza:

1. **downloads/ (577M)**: 
   - Limpar PDFs antigos: `./scripts/cleanup-disk.sh`
   - Verificar cache: `du -sh downloads/cache`

2. **target/ (191M)**:
   - Limpar builds antigos: `cd news-backend && cargo clean`
   - Isso remove builds de debug, mantém apenas release se necessário

3. **Docker**:
   - Limpar imagens não usadas: `docker system prune -a`
   - Limpar volumes órfãos: `docker volume prune`

## ⚠️ Cuidados

- **NÃO delete** `output/` - contém artigos gerados
- **NÃO delete** `images/` - imagens usadas pelos sites
- **NÃO delete** `apps/` - código dos frontends
- **CUIDADO** com `target/` - pode precisar rebuildar depois
- **CUIDADO** com `node_modules/` - será necessário `npm install` depois

## 🎯 Próximos Passos

1. Execute o script de análise: `./scripts/analyze-disk-usage.sh`
2. Identifique os maiores consumidores de espaço
3. Use `./scripts/cleanup-disk.sh` para limpar PDFs
4. Limpe Docker se necessário: `docker system prune -a`
5. Limpe target se necessário: `cd news-backend && cargo clean`

