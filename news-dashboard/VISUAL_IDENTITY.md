# Identidade Visual - News Dashboard

## Inspiração

A identidade visual do dashboard é inspirada no portal **AIResearch** (`apps/frontend-next/AIResearch`), mantendo consistência visual e experiência de usuário.

## Paleta de Cores

### Tema Claro (Light Mode)
```css
--primary: 187 100% 45%;           /* Azul turquesa vibrante */
--secondary: 187 30% 96%;           /* Azul turquesa claro */
--accent: 187 30% 96%;              /* Azul turquesa suave */
--background: 0 0% 100%;            /* Branco */
--foreground: 192 10% 3.9%;         /* Preto quase */
--muted: 187 30% 96%;               /* Cinza azulado */
--border: 187 20% 89.8%;            /* Borda suave */
```

### Tema Escuro (Dark Mode)
```css
--primary: 187 100% 45%;            /* Azul turquesa vibrante (mesmo) */
--secondary: 187 30% 20%;           /* Azul escuro */
--accent: 187 30% 20%;              /* Azul escuro suave */
--background: 192 10% 3.9%;         /* Fundo escuro */
--foreground: 0 0% 98%;             /* Texto claro */
--muted: 187 30% 20%;               /* Cinza escuro */
--border: 187 30% 20%;              /* Borda escura */
```

### Cores Semânticas
- **Primary**: Azul turquesa (#00CED1) - Ações principais
- **Secondary**: Azul turquesa suave - Elementos secundários
- **Accent**: Destaque para hover states
- **Destructive**: Vermelho (#F87171) - Ações destrutivas
- **Muted**: Textos secundários e backgrounds suaves

## Tipografia

- **Font**: Inter (via Next.js Google Fonts)
- **Headings**: Semibold/Bold
- **Body**: Regular 400
- **Feature Settings**: `"rlig" 1, "calt" 1` para melhor tipografia

## Componentes

### Botões
- **Default**: Background primary, texto branco
- **Outline**: Borda, hover com background
- **Secondary**: Background secundário
- **Ghost**: Apenas hover
- **Link**: Aparência de link

### Cards
- Bordas arredondadas (`border-radius: 0.75rem`)
- Shadow suave
- Border color sutil
- Hover effect com elevação

### Badges
- Borda arredondada completa
- Padding adequado
- Diferentes variantes (default, secondary, destructive, outline)

## Animações

### Fade In
```css
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
```

### Fade In Up
```css
@keyframes fadeInUp {
  from { 
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

### Hover Lift
```css
.hover-lift {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hover-lift:hover {
  transform: translateY(-4px);
  box-shadow: 0 20px 25px -5px rgba(0,0,0,0.1);
}
```

### Gradient Animation
```css
.animate-gradient {
  background-size: 200% 200%;
  animation: gradient 8s ease infinite;
}
```

## Efeitos Especiais

### Glass Effect (Glassmorphism)
```css
.glass-effect {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}
```

### Custom Selection
```css
::selection {
  background-color: hsl(var(--primary));
  color: hsl(var(--primary-foreground));
}
```

## Espaçamento

- **Base**: 0.75rem (1.2rem em Tailwind)
- **Padding**: p-4, p-6, p-8 (padrão: 1rem, 1.5rem, 2rem)
- **Gaps**: gap-2, gap-4, gap-6
- **Margins**: Seguindo hierarquia de 0.5rem, 1rem, 1.5rem, 2rem

## Responsividade

- **Mobile**: < 768px
- **Tablet**: 768px - 1024px
- **Desktop**: > 1024px

Grid adaptativo:
- 1 coluna em mobile
- 2 colunas em tablet
- 4+ colunas em desktop

## Acessibilidade

- Contraste adequado (WCAG AA)
- Focus states visíveis
- Estados de hover claros
- Suporte a modo escuro
- Animações respeitam `prefers-reduced-motion`

## Implementação

A identidade visual é aplicada através de:

1. **Tailwind CSS** com variáveis CSS customizadas
2. **Componentes UI** (Radix UI pattern)
3. **Class Variance Authority** para variantes
4. **Tailwind Merge** para combinação de classes

## Conformidade

✅ Consistente com AIResearch
✅ Componentes modernos e acessíveis
✅ Animações suaves e profissionais
✅ Dark mode support
✅ Responsivo em todos os dispositivos


