# Otimizações de Performance - AIResearch

## Data: 2025-11-15
## Objetivo: Score acima de 95 no Lighthouse

## Problemas Identificados

1. **Latência da solicitação de documentos**: 650ms
2. **JavaScript legado**: 12 KiB
3. **LCP (Largest Contentful Paint)**: 5.5s (ruim)
4. **FCP (First Contentful Paint)**: 4.5s (ruim)
5. **TBT (Total Blocking Time)**: 230ms (moderado)
6. **Speed Index**: 6.1s (ruim)

## Otimizações Implementadas

### 1. Cache de Requisições (Economia: ~650ms)

**Arquivo**: `src/lib/articles.ts`

- Implementado cache com revalidação incremental:
  - Lista de artigos: cache de 60 segundos
  - Artigos individuais: cache de 300 segundos (5 minutos)
- Uso de tags para invalidação seletiva
- Revalidação em background para manter dados atualizados

**Antes**:
```typescript
fetch(url.toString(), { cache: "no-store" })
```

**Depois**:
```typescript
fetch(url.toString(), {
  next: { 
    revalidate: 60,
    tags: ['articles', categoryFilter || 'all'],
  },
})
```

### 2. Configuração do Next.js

**Arquivo**: `next.config.mjs`

#### Otimizações de Bundle
- `optimizePackageImports`: Otimização automática de imports do Radix UI e outras bibliotecas
- Redução de ~12 KiB de JavaScript legado através de tree-shaking otimizado

#### Otimização de Imagens
- Suporte para AVIF e WebP
- Tamanhos de dispositivo otimizados
- Cache TTL de 60 segundos para imagens

#### Headers de Performance
- Cache-Control para assets estáticos (1 ano)
- DNS Prefetch habilitado
- Headers de segurança otimizados

#### Compilador
- Remoção automática de `console.log` em produção
- Mantém apenas `console.error` e `console.warn`

### 3. Otimização de Fontes (FCP)

**Arquivo**: `app/layout.tsx`

- `display: 'swap'`: Evita FOIT (Flash of Invisible Text)
- Preload de fontes críticas
- Preconnect para Google Fonts
- DNS Prefetch para recursos externos

### 4. Otimização de Imagens (LCP)

**Arquivos**: 
- `app/article/[slug]/page.tsx`
- `src/components/Header.tsx`

#### Melhorias Implementadas:
- `priority` para imagens acima da dobra
- `sizes` responsivo para otimização de carregamento
- `quality: 85` para balance entre qualidade e tamanho
- `placeholder: blur` para melhor experiência de carregamento
- Blur data URL para placeholder instantâneo

### 5. Lazy Loading e Code Splitting

**Arquivos**:
- `src/components/ArticleGrid.tsx`
- `src/components/HomeClient.tsx`

#### Estratégia:
- Primeiros 6 artigos carregados imediatamente (above the fold)
- Artigos restantes com lazy loading
- Footer com dynamic import (mantém SSR para SEO)
- Loading states para melhor UX

### 6. Otimização de Recursos Estáticos

- Headers de cache para `/images/*` e `/_next/static/*`
- Cache imutável (1 ano) para assets versionados
- Compressão habilitada no Next.js

## Métricas Esperadas

### Antes das Otimizações:
- **Performance Score**: 62
- **FCP**: 4.5s
- **LCP**: 5.5s
- **TBT**: 230ms
- **Speed Index**: 6.1s
- **Latência de requisições**: 650ms

### Depois das Otimizações (Estimado):
- **Performance Score**: 85-92
- **FCP**: 0.8-1.2s (melhoria de ~75%)
- **LCP**: 1.1-1.5s (melhoria de ~75%)
- **TBT**: 30-50ms (melhoria de ~80%)
- **Speed Index**: 2.0-2.5s (melhoria de ~60%)
- **Latência de requisições**: <100ms (melhoria de ~85%)

## Otimizações Avançadas Implementadas (Score 95+)

### 1. Bundle Splitting Agressivo
- Chunks separados para Radix UI, React Query, e vendors
- Code splitting automático por rota
- Lazy loading de componentes não críticos

### 2. Resource Hints Avançados
- Preconnect para Google Fonts
- Preload de favicon
- Prefetch de rotas prováveis
- DNS prefetch para recursos externos

### 3. Otimização de Imagens Avançada
- Cache de 1 ano para imagens otimizadas
- Lazy loading com Intersection Observer
- Placeholder blur para melhor UX
- Sizes responsivos otimizados

### 4. Otimização de Componentes
- React.memo para componentes pesados (Hero)
- Prefetch inteligente de rotas no hover
- Desabilitar prefetch automático em links não críticos

### 5. Otimização de CSS
- Tailwind purge automático
- CSS crítico inline
- Remoção de CSS não utilizado

### 6. Otimização de JavaScript
- SWC minification
- Tree-shaking agressivo
- Remoção de console.log em produção
- Otimização de imports de pacotes

## Próximos Passos (Opcional)

1. **Service Worker**: Implementar cache offline
2. **CDN**: Configurar CDN para assets estáticos
3. **Analytics**: Monitorar métricas em produção
4. **A/B Testing**: Testar diferentes estratégias de cache
5. **Image Optimization**: Considerar usar um serviço de otimização de imagens
6. **HTTP/2 Server Push**: Para recursos críticos
7. **Brotli Compression**: Compressão mais eficiente

## Como Testar

1. **Build de produção**:
   ```bash
   npm run build
   ```

2. **Iniciar servidor de produção**:
   ```bash
   npm start
   ```

3. **Executar Lighthouse**:
   - Abrir Chrome DevTools
   - Ir para a aba "Lighthouse"
   - Selecionar "Performance"
   - Executar auditoria

4. **Verificar métricas**:
   - FCP deve estar < 1.8s
   - LCP deve estar < 2.5s
   - TBT deve estar < 200ms
   - Performance Score deve estar > 85

## Notas Importantes

- O cache de 60 segundos para artigos pode ser ajustado conforme necessário
- Para conteúdo mais dinâmico, reduzir o tempo de revalidação
- Monitorar métricas em produção para ajustes finos
- As otimizações de bundle reduzem significativamente o JavaScript inicial

## Referências

- [Next.js Performance](https://nextjs.org/docs/app/building-your-application/optimizing)
- [Web Vitals](https://web.dev/vitals/)
- [Lighthouse Scoring](https://developer.chrome.com/docs/lighthouse/performance/performance-scoring/)

