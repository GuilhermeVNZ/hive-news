# Dimensões de Imagens - Frontend ScienceAI e AIResearch

Análise das dimensões de imagens utilizadas nos portais ScienceAI e AIResearch.

## 📐 Dimensões Utilizadas

### 1. **Carrossel Hero (Página Principal)**

#### ScienceAI
- **Altura**: 600px (`h-[600px]`)
- **Largura**: 100% (full-width responsivo)
- **Aspect Ratio**: ~16:9 ou mais largo
- **Uso**: Imagens de destaque no carrossel da homepage

```tsx
// HeroCarousel.tsx linha 48
<section className="relative h-[600px] w-full overflow-hidden rounded-xl">
```

#### AIResearch
- **Altura**: ~500px - 700px (variável)
- **Largura**: 100%
- **Uso**: Hero section

### 2. **Cards de Artigo (Grid de Artigos)**

#### ScienceAI
- **Altura**: 192px (`h-48` = 12rem = 192px)
- **Largura**: 100% (responsivo)
- **Aspect Ratio**: ~16:9
- **Uso**: Imagens dos cards de artigos na página principal

```tsx
// ArticleCard.tsx linha 28
<div className="relative h-48 overflow-hidden">
```

#### AIResearch
- **Altura**: ~200px - 240px (estimado)
- **Largura**: 100%
- **Uso**: Cards de artigos

### 3. **Página Detalhada do Artigo**

#### ScienceAI
- **Altura**: 500px (`h-[500px]`)
- **Largura**: 100%
- **Aspect Ratio**: ~16:9
- **Uso**: Imagem principal no topo do artigo

```tsx
// ArticleDetail.tsx linha 74
<div className="relative h-[500px] w-full">
```

#### AIResearch
- **Altura**: Variável (baseado no conteúdo)
- **Largura**: 100%

### 4. **Ícones e Logos**

#### Header
- **Logo**: 40x40px (`w-40 h-40` ou automático)
- **Ícone**: 24x24px a 40x40px

```tsx
// Header.tsx linhas 40-42
className="h-10 w-auto"  // Logo
```

#### Footer
- **Ícones sociais**: 20x20px (`h-5 w-5`)
- **Logo**: 24x24px

### 5. **Avatar / Author**

#### ScienceAI
- **Tamanho**: 64x64px (`w-16 h-16`)
- **Shape**: Circular
- **Uso**: Foto do autor

```tsx
// ArticleDetail.tsx linha 184
<div className="w-16 h-16 bg-primary rounded-full">
```

## 📊 Resumo por Localização

| Local | ScienceAI | AIResearch | Aspecto |
|-------|-----------|------------|---------|
| **Carrossel Hero** | 600px altura | ~500-700px | 16:9 |
| **Card de Artigo** | 192px altura | ~200-240px | 16:9 |
| **Página Artigo** | 500px altura | Variável | 16:9 |
| **Logo Header** | 40x40px | 40x40px | 1:1 |
| **Ícones** | 20-24px | 20-24px | 1:1 |
| **Avatar** | 64x64px | ~64px | Circular |

## 🎨 Orientação e Estilo

- **Todas as imagens**: `object-cover` (preenche o container mantendo proporção)
- **Responsivo**: 100% da largura (`w-full`)
- **Overlay**: Gradiente sobre algumas imagens hero
- **Hover**: Efeito de zoom/scale em cards

## 🔧 Recomendações de Dimensões para Upload

### Carrossel Hero
- **Dimensão ideal**: 1920x1080px (Full HD)
- **Aspect Ratio**: 16:9
- **Orientação**: Horizontal
- **Formato**: JPG/PNG

### Cards de Artigo
- **Dimensão ideal**: 1200x675px
- **Aspect Ratio**: 16:9
- **Orientação**: Horizontal
- **Formato**: JPG

### Página Detalhada
- **Dimensão ideal**: 1920x1080px ou 1600x900px
- **Aspect Ratio**: 16:9
- **Orientação**: Horizontal
- **Formato**: JPG

### Ícones/Logos
- **Dimensão ideal**: 512x512px (exportar em múltiplos tamanhos)
- **Aspect Ratio**: 1:1
- **Formato**: PNG (transparente) ou SVG

## 📱 Responsividade

As imagens são responsivas e se adaptam automaticamente:
- **Mobile**: Redimensiona proporcionalmente
- **Tablet**: 100% da largura disponível
- **Desktop**: Full-width com max-width do container

## 🖼️ Sistema de Imagens Local

As imagens estão organizadas em:
```
News-main/images/
├── ai/          # Categoria AI
├── coding/      # Programação
├── crypto/      # Criptomoedas
├── database/    # Bancos de dados
├── ethics/      # Ética
├── games/       # Jogos
├── hardware/    # Hardware
├── legal/       # Legal
├── network/     # Redes
├── robotics/    # Robótica
├── science/     # Ciência
├── security/    # Segurança
└── sound/       # Áudio
```

Total: **185 imagens** organizadas e numeradas por categoria.


