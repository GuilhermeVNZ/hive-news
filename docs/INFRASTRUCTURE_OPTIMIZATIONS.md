# 🚀 Guia de Implementação: Otimizações de Infraestrutura

Este guia explica como implementar as otimizações de PageSpeed que dependem de configuração no servidor/CDN.

## 📋 Índice

1. [AVIF - Conversão de Imagens](#1-avif---conversão-de-imagens)
2. [HTTP/3 + QUIC](#2-http3--quic)
3. [CDN (Cloudflare ou similar)](#3-cdn-cloudflare-ou-similar)
4. [Early Hints (103)](#4-early-hints-103)
5. [Cache Forte](#5-cache-forte)
6. [Image CDN / Redimensionamento Dinâmico](#6-image-cdn--redimensionamento-dinâmico)

---

## 1. AVIF - Conversão de Imagens

### O que é AVIF?
AVIF é um formato de imagem moderno que oferece melhor compressão que WebP/JPEG, reduzindo o tamanho dos arquivos em 50-80% mantendo a mesma qualidade visual.

### Implementação

#### Opção A: Cloudflare (Mais Fácil) ⭐ Recomendado

Se você usa Cloudflare como CDN, a conversão AVIF é automática:

```bash
# 1. Ativar no painel Cloudflare
Dashboard → Speed → Optimization → Auto Minify
→ Marcar "Convert images to AVIF"

# Ou via API
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/{zone_id}/settings/automatic_platform_optimization" \
  -H "Authorization: Bearer {api_token}" \
  -H "Content-Type: application/json" \
  --data '{"value":{"enabled":true,"cf":true,"wordpress":false,"wp_plugin":false}}'
```

**Vantagens:**
- ✅ Automático - converte todas as imagens
- ✅ Sem configuração no servidor
- ✅ Fallback automático para navegadores antigos
- ✅ Cache inteligente por formato suportado

#### Opção B: Nginx com módulo AVIF (Servidor próprio)

**1. Instalar dependências:**

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    libavif-dev \
    libavif-bin \
    nginx-extras

# Compilar nginx com módulo imagem dinâmico (ou usar nginx-extras)
```

**2. Configurar Nginx para conversão on-the-fly:**

```nginx
# /etc/nginx/sites-available/airesearch.news

server {
    listen 443 ssl http2;
    server_name airesearch.news www.airesearch.news;
    
    # ... SSL config ...

    # Diretório de imagens
    location ~* \.(jpg|jpeg|png|webp)$ {
        root /opt/news-system/images;
        
        # Verificar se navegador suporta AVIF
        set $avif_supported "";
        
        # Detectar suporte AVIF via Accept header
        if ($http_accept ~* "image/avif") {
            set $avif_supported "1";
        }
        
        # Se suporta AVIF e arquivo .avif existe, servir
        if ($avif_supported = "1") {
            rewrite ^(.+)\.(jpg|jpeg|png|webp)$ $1.avif last;
        }
        
        # Cache forte para imagens
        expires 1y;
        add_header Cache-Control "public, immutable";
        add_header Vary "Accept";
    }
    
    # Servir AVIF quando disponível
    location ~* \.avif$ {
        root /opt/news-system/images;
        expires 1y;
        add_header Cache-Control "public, immutable";
        add_header Content-Type "image/avif";
        add_header Vary "Accept";
    }
}
```

**3. Script para converter imagens em batch:**

```bash
#!/bin/bash
# scripts/convert-to-avif.sh

IMAGE_DIR="/opt/news-system/images"

# Converter todas as imagens para AVIF
find "$IMAGE_DIR" -type f \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" \) | while read img; do
    avif_file="${img%.*}.avif"
    if [ ! -f "$avif_file" ]; then
        echo "Converting: $img → $avif_file"
        # Usar libavif (avifenc) ou imagemagick
        avifenc -c aom -s 6 "$img" "$avif_file" 2>/dev/null || \
        magick "$img" -quality 80 "$avif_file"
    fi
done

echo "✅ AVIF conversion completed"
```

```bash
# Tornar executável
chmod +x scripts/convert-to-avif.sh

# Executar após upload de novas imagens
./scripts/convert-to-avif.sh
```

#### Opção C: Next.js Image Optimization (AIResearch)

O Next.js já tem suporte a AVIF nativo:

```javascript
// next.config.mjs (já configurado)
export default {
  images: {
    formats: ['image/avif', 'image/webp'], // AVIF primeiro
    deviceSizes: [640, 750, 828, 1080, 1200, 1920],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
  },
};
```

**Next.js automaticamente:**
- ✅ Converte imagens para AVIF durante build
- ✅ Serve AVIF para navegadores que suportam
- ✅ Fallback para WebP/JPEG para navegadores antigos
- ✅ Gera múltiplos tamanhos com `srcset`

#### Opção D: Vite Plugin (ScienceAI)

Para ScienceAI (Vite), usar plugin:

```bash
npm install vite-plugin-imagemin imagemin-avif --save-dev
```

```javascript
// vite.config.ts
import { defineConfig } from 'vite';
import { imagemin } from 'vite-plugin-imagemin';
import imageminAvif from 'imagemin-avif';

export default defineConfig({
  plugins: [
    imagemin({
      plugins: [
        imageminAvif({
          quality: 80,
        }),
      ],
    }),
  ],
});
```

**Recomendação:** Para produção, use **Cloudflare** (Opção A) - mais simples e eficiente.

---

## 2. HTTP/3 + QUIC

### O que é HTTP/3?
HTTP/3 é a versão mais recente do protocolo HTTP, usando QUIC sobre UDP. Oferece:
- 🚀 15-30% mais rápido que HTTP/2
- 🔒 Melhor segurança (TLS 1.3 nativo)
- 📡 Recuperação mais rápida de perda de pacotes
- 🔄 Menos latência em conexões instáveis

### Implementação

#### Opção A: Cloudflare (Automático) ⭐ Mais Fácil

Cloudflare já oferece HTTP/3 automaticamente:

```bash
# Verificar no painel:
Dashboard → Network → HTTP/3 (with QUIC)
→ Ativar (já vem ativado por padrão)

# Ou verificar via curl:
curl -I --http3 https://airesearch.news
# Deve mostrar: HTTP/3 200
```

#### Opção B: Nginx com HTTP/3

**1. Compilar Nginx com suporte HTTP/3:**

```bash
# Ubuntu/Debian
cd /tmp
git clone --recursive https://github.com/cloudflare/quiche

# Instalar dependências Rust (se necessário)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compilar quiche
cd quiche
cargo build --release
```

**2. Compilar Nginx com módulo QUIC:**

```bash
# Baixar source do nginx
cd /tmp
NGINX_VERSION=1.25.3
wget http://nginx.org/download/nginx-${NGINX_VERSION}.tar.gz
tar xzf nginx-${NGINX_VERSION}.tar.gz
cd nginx-${NGINX_VERSION}

# Configurar build
./configure \
    --prefix=/etc/nginx \
    --sbin-path=/usr/sbin/nginx \
    --modules-path=/usr/lib/nginx/modules \
    --conf-path=/etc/nginx/nginx.conf \
    --error-log-path=/var/log/nginx/error.log \
    --http-log-path=/var/log/nginx/access.log \
    --pid-path=/var/run/nginx.pid \
    --lock-path=/var/run/nginx.lock \
    --http-client-body-temp-path=/var/cache/nginx/client_temp \
    --http-proxy-temp-path=/var/cache/nginx/proxy_temp \
    --http-fastcgi-temp-path=/var/cache/nginx/fastcgi_temp \
    --http-uwsgi-temp-path=/var/cache/nginx/uwsgi_temp \
    --http-scgi-temp-path=/var/cache/nginx/scgi_temp \
    --with-file-aio \
    --with-http_ssl_module \
    --with-http_realip_module \
    --with-http_addition_module \
    --with-http_sub_module \
    --with-http_dav_module \
    --with-http_flv_module \
    --with-http_mp4_module \
    --with-http_gunzip_module \
    --with-http_gzip_static_module \
    --with-http_random_index_module \
    --with-http_secure_link_module \
    --with-http_stub_status_module \
    --with-http_auth_request_module \
    --with-http_xslt_module=dynamic \
    --with-http_image_filter_module=dynamic \
    --with-http_geoip_module=dynamic \
    --with-threads \
    --with-stream \
    --with-stream_ssl_module \
    --with-stream_ssl_preread_module \
    --with-stream_realip_module \
    --with-stream_geoip_module=dynamic \
    --with-http_slice_module \
    --with-http_v2_module \
    --with-http_v3_module \
    --with-openssl=../quiche/deps/boringssl \
    --with-quiche=../quiche

# Compilar e instalar
make -j$(nproc)
sudo make install
```

**3. Configurar Nginx para HTTP/3:**

```nginx
# /etc/nginx/sites-available/airesearch.news

server {
    # HTTP/3 (443 QUIC)
    listen 443 http3 reuseport;
    listen 443 ssl http2;  # Fallback HTTP/2
    
    server_name airesearch.news www.airesearch.news;
    
    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/airesearch.news/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/airesearch.news/privkey.pem;
    ssl_protocols TLSv1.3;
    ssl_ciphers TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256;
    ssl_prefer_server_ciphers off;
    
    # HTTP/3 Alt-Svc header (informa ao cliente que HTTP/3 está disponível)
    add_header Alt-Svc 'h3=":443"; ma=86400' always;
    
    # ... resto da configuração ...
}
```

**4. Verificar HTTP/3:**

```bash
# Testar
curl -I --http3 https://airesearch.news

# Ou usar navegador:
# Chrome DevTools → Network → Protocol → deve mostrar "h3"
```

**Recomendação:** Use **Cloudflare** (Opção A) - HTTP/3 já vem ativado automaticamente.

---

## 3. CDN (Cloudflare ou Similar)

### O que é CDN?
CDN (Content Delivery Network) distribui conteúdo através de servidores próximos aos usuários, reduzindo latência e melhorando velocidade.

### Implementação com Cloudflare ⭐ Recomendado

#### Passo 1: Configurar DNS

```bash
# 1. No painel Cloudflare:
# Dashboard → DNS → Records
# Adicionar registros:

Type: A
Name: @
Content: {IP_DO_SERVIDOR}
Proxy: 🟠 Proxied (ON - importante para CDN)

Type: A
Name: www
Content: {IP_DO_SERVIDOR}
Proxy: 🟠 Proxied (ON)
```

#### Passo 2: Ativar Otimizações

```bash
# Dashboard → Speed → Optimization

# ✅ Auto Minify
- ✅ JavaScript
- ✅ CSS
- ✅ HTML

# ✅ Image Optimization
- ✅ Polish: Lossless (ou Lossy para compressão mais agressiva)
- ✅ WebP: ON
- ✅ AVIF: ON (se disponível)
- ✅ Mirage: ON (otimização automática para mobile)

# ✅ Caching
- ✅ Browser Cache TTL: 1 month
- ✅ Always Online: ON

# ✅ Network
- ✅ HTTP/2: ON
- ✅ HTTP/3 (with QUIC): ON
- ✅ 0-RTT Connection Resumption: ON

# ✅ Compression
- ✅ Brotli: ON
```

#### Passo 3: Configurar Cache Rules

```bash
# Dashboard → Rules → Page Rules (ou Transform Rules)

# Regra 1: Cache estático agressivo
URL: *airesearch.news/images/*
Settings:
  - Cache Level: Cache Everything
  - Edge Cache TTL: 1 month
  - Browser Cache TTL: 1 month

# Regra 2: API com cache curto
URL: *airesearch.news/api/articles*
Settings:
  - Cache Level: Standard
  - Edge Cache TTL: 5 minutes
  - Browser Cache TTL: 1 minute

# Regra 3: HTML sem cache
URL: *airesearch.news/*
Settings:
  - Cache Level: Bypass (para páginas dinâmicas)
```

#### Passo 4: Configurar Firewall e Segurança

```bash
# Dashboard → Security → WAF
# Ativar proteções automáticas:
- ✅ Managed Rules (OWASP, etc.)
- ✅ Rate Limiting (proteger APIs)

# Dashboard → Security → DDoS
# ✅ Automatic DDoS protection: ON
```

#### Passo 5: Workers (Opcional - Advanced)

Para lógica customizada na borda:

```javascript
// cloudflare-workers/image-optimizer.js
export default {
  async fetch(request) {
    const url = new URL(request.url);
    
    // Redirecionar imagens para formato otimizado
    if (url.pathname.match(/\.(jpg|jpeg|png)$/i)) {
      const accept = request.headers.get('Accept') || '';
      
      if (accept.includes('image/avif')) {
        url.pathname = url.pathname.replace(/\.(jpg|jpeg|png)$/i, '.avif');
      } else if (accept.includes('image/webp')) {
        url.pathname = url.pathname.replace(/\.(jpg|jpeg|png)$/i, '.webp');
      }
      
      return fetch(url.toString(), request);
    }
    
    return fetch(request);
  }
};
```

**Alternativas ao Cloudflare:**
- **Bunny CDN:** Mais barato, boa performance
- **Fastly:** Enterprise, mais caro
- **AWS CloudFront:** Integração com AWS
- **KeyCDN:** Alternativa econômica

---

## 4. Early Hints (103)

### O que é Early Hints?
Early Hints (HTTP 103) permite ao servidor enviar dicas sobre recursos antes da resposta final, iniciando downloads paralelos mais cedo.

### Implementação

#### Opção A: Cloudflare (Automático) ⭐

Cloudflare já oferece Early Hints automaticamente para recursos comuns (CSS, JS, fonts).

#### Opção B: Nginx com módulo Early Hints

**1. Nginx 1.25.1+ já suporta Early Hints nativo:**

```nginx
# /etc/nginx/nginx.conf ou site config

server {
    listen 443 ssl http2;
    server_name airesearch.news;
    
    # ... SSL config ...
    
    # Early Hints: pré-carregar recursos críticos
    location / {
        proxy_pass http://localhost:3000;
        
        # Enviar Early Hints para recursos críticos
        http2_push_preload on;
        
        # Headers para Early Hints (103)
        add_header Link "</assets/css/main.css>; rel=preload; as=style" always;
        add_header Link "</assets/js/main.js>; rel=preload; as=script" always;
        add_header Link "</fonts/inter.woff2>; rel=preload; as=font; type=font/woff2; crossorigin" always;
        
        # ... proxy settings ...
    }
}
```

**2. Configurar no Next.js (AIResearch):**

```javascript
// next.config.mjs
export default {
  // Next.js já envia Early Hints automaticamente para recursos críticos
  // Para customizar, use headers:
  async headers() {
    return [
      {
        source: '/',
        headers: [
          {
            key: 'Link',
            value: '</_next/static/css/app.css>; rel=preload; as=style, </_next/static/chunks/main.js>; rel=preload; as=script',
          },
        ],
      },
    ];
  },
};
```

**3. Verificar Early Hints:**

```bash
# Ver headers de resposta
curl -I https://airesearch.news

# Deve mostrar:
# HTTP/1.1 103 Early Hints
# Link: </assets/css/main.css>; rel=preload; as=style

# HTTP/2/3:
# 103 Early Hints (enviado antes do 200 OK)
```

**Recomendação:** Cloudflare já faz isso automaticamente. Se usar servidor próprio, configure manualmente no Nginx.

---

## 5. Cache Forte

### Configuração no Nginx

```nginx
# /etc/nginx/sites-available/airesearch.news

server {
    listen 443 ssl http2;
    server_name airesearch.news;
    
    # ... SSL config ...
    
    # Cache para arquivos estáticos (1 ano - immutable)
    location ~* \.(jpg|jpeg|png|gif|webp|avif|ico|svg|woff|woff2|ttf|eot)$ {
        root /opt/news-system/apps/frontend-next/airesearch/.next/static;
        
        expires 1y;
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Pragma "public";
        add_header Vary "Accept-Encoding";
        
        # ETag para validação condicional
        etag on;
    }
    
    # Cache para CSS/JS (1 mês)
    location ~* \.(css|js)$ {
        root /opt/news-system/apps/frontend-next/airesearch/.next/static;
        
        expires 1M;
        add_header Cache-Control "public, max-age=2592000";
        add_header Vary "Accept-Encoding";
        
        # Gzip/Brotli
        gzip_static on;
        brotli_static on;
    }
    
    # Cache para HTML (1 hora - ISR do Next.js)
    location ~* \.(html)$ {
        proxy_pass http://localhost:3000;
        
        expires 1h;
        add_header Cache-Control "public, max-age=3600, s-maxage=3600, stale-while-revalidate=86400";
        
        # ... proxy settings ...
    }
    
    # API com cache curto (5 minutos)
    location /api/articles {
        proxy_pass http://localhost:3000;
        
        expires 5m;
        add_header Cache-Control "public, max-age=300, s-maxage=300, stale-while-revalidate=600";
        
        # ... proxy settings ...
    }
}
```

### Configuração no Cloudflare

```bash
# Dashboard → Rules → Page Rules

# Regra: Static Assets
URL: *airesearch.news/_next/static/*
Settings:
  - Cache Level: Cache Everything
  - Edge Cache TTL: 1 month
  - Browser Cache TTL: 1 month
  - Respect Existing Headers: OFF

# Regra: Images
URL: *airesearch.news/images/*
Settings:
  - Cache Level: Cache Everything
  - Edge Cache TTL: 1 year
  - Browser Cache TTL: 1 year

# Regra: API
URL: *airesearch.news/api/*
Settings:
  - Cache Level: Standard
  - Edge Cache TTL: 5 minutes
  - Browser Cache TTL: 1 minute
```

---

## 6. Image CDN / Redimensionamento Dinâmico

### Opção A: Cloudflare Images (Recomendado) ⭐

Cloudflare oferece redimensionamento automático de imagens:

```bash
# Dashboard → Images → Setup
# 1. Ativar Cloudflare Images
# 2. Fazer upload de imagens via API ou painel
# 3. Usar URLs com parâmetros de redimensionamento

# Exemplo de URL:
# https://imagedelivery.net/{account_hash}/{image_id}/{variant_name}

# Variantes pré-configuradas:
# - thumbnail (400x400)
# - medium (800x800)
# - large (1200x1200)
# - original
```

**Integração no código:**

```typescript
// src/lib/imageUtils.ts
export function getCloudflareImageUrl(
  imageId: string,
  width?: number,
  height?: number,
  format?: 'avif' | 'webp' | 'jpg',
): string {
  const accountHash = process.env.CLOUDFLARE_IMAGES_ACCOUNT_HASH;
  const variant = width && height 
    ? `w=${width},h=${height},f=${format || 'avif'}`
    : 'original';
  
  return `https://imagedelivery.net/${accountHash}/${imageId}/${variant}`;
}
```

### Opção B: Next.js Image (AIResearch)

Next.js já oferece redimensionamento automático:

```typescript
// Já configurado em next.config.mjs
import Image from 'next/image';

// Uso automático com srcset
<Image
  src="/images/article.jpg"
  width={800}
  height={600}
  alt="Article"
  sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 800px"
  quality={82}
/>
```

**Next.js automaticamente:**
- ✅ Gera múltiplos tamanhos
- ✅ Cria `srcset` dinamicamente
- ✅ Converte para AVIF/WebP
- ✅ Lazy loading nativo

### Opção C: Servidor próprio com ImageMagick/Sharp

**1. Endpoint para redimensionamento:**

```rust
// news-backend/src/routes/images.rs
use image::ImageFormat;
use std::path::PathBuf;

pub async fn resize_image(
    path: PathBuf,
    width: Option<u32>,
    height: Option<u32>,
    format: Option<String>,
) -> Result<Vec<u8>, Error> {
    let img = image::open(&path)?;
    
    // Redimensionar
    let resized = if let (Some(w), Some(h)) = (width, height) {
        img.resize_exact(w, h, image::imageops::FilterType::Lanczos3)
    } else if let Some(w) = width {
        img.resize(w, u32::MAX, image::imageops::FilterType::Lanczos3)
    } else if let Some(h) = height {
        img.resize(u32::MAX, h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    
    // Converter formato
    let output_format = match format.as_deref() {
        Some("avif") => ImageFormat::Avif,
        Some("webp") => ImageFormat::WebP,
        _ => ImageFormat::Jpeg,
    };
    
    let mut output = Vec::new();
    resized.write_to(&mut std::io::Cursor::new(&mut output), output_format)?;
    
    Ok(output)
}
```

**2. Nginx para servir imagens redimensionadas:**

```nginx
# Redimensionamento via query params
location ~* ^/images/(.+)\.(jpg|jpeg|png)$ {
    set $image_path $1.$2;
    set $width $arg_w;
    set $height $arg_h;
    set $format $arg_f;
    
    # Se tem parâmetros, redimensionar
    if ($width$height) {
        rewrite ^ /api/images/resize?path=$image_path&w=$width&h=$height&f=$format last;
    }
    
    # Senão, servir original
    root /opt/news-system/images;
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

**Recomendação:** Para produção, use **Cloudflare Images** ou **Next.js Image** - mais simples e eficiente.

---

## 📊 Checklist de Implementação

### Prioridade Alta (Impacto Alto)

- [ ] **CDN (Cloudflare)**: Configurar DNS e ativar proxy
- [ ] **Cache Forte**: Configurar regras de cache no Cloudflare/Nginx
- [ ] **AVIF**: Ativar no Cloudflare ou converter imagens
- [ ] **HTTP/3**: Ativar no Cloudflare (automático)

### Prioridade Média

- [ ] **Early Hints**: Configurar no Nginx ou usar Cloudflare automático
- [ ] **Image CDN**: Usar Cloudflare Images ou Next.js Image
- [ ] **Brotli Compression**: Ativar no Cloudflare/Nginx

### Prioridade Baixa (Opcional)

- [ ] **HTTP/3 no servidor próprio**: Compilar Nginx com QUIC (se não usar Cloudflare)
- [ ] **Redimensionamento custom**: Se precisar de lógica específica

---

## 🎯 Recomendação Final

**Para implementação rápida e eficiente:**

1. **Use Cloudflare como CDN** (gratuito até certo limite)
   - ✅ HTTP/3 automático
   - ✅ AVIF automático
   - ✅ Early Hints automático
   - ✅ Cache inteligente
   - ✅ Brotli automático

2. **Configure DNS:**
   ```bash
   # DNS Records no Cloudflare
   A     @      {IP_SERVIDOR}    🟠 Proxied
   A     www    {IP_SERVIDOR}    🟠 Proxied
   ```

3. **Ative otimizações no painel:**
   - Speed → Auto Minify: ON
   - Speed → Image Optimization: Polish + WebP + AVIF
   - Speed → Caching: Cache Everything para estáticos
   - Network → HTTP/3: ON

4. **Pronto!** ✅ Todas as otimizações de infraestrutura estão ativas.

---

## 🔍 Verificação

### Testar AVIF:

```bash
# Verificar se AVIF está sendo servido
curl -H "Accept: image/avif" -I https://airesearch.news/images/article.jpg
# Deve retornar Content-Type: image/avif ou Location: *.avif
```

### Testar HTTP/3:

```bash
# Verificar HTTP/3
curl -I --http3 https://airesearch.news
# Deve mostrar: HTTP/3 200

# Ou ver no navegador:
# Chrome DevTools → Network → Protocol → deve mostrar "h3"
```

### Testar Cache:

```bash
# Verificar headers de cache
curl -I https://airesearch.news/_next/static/css/main.css
# Deve mostrar: Cache-Control: public, max-age=31536000, immutable
```

### Testar Early Hints:

```bash
# Verificar Early Hints (HTTP/2 ou superior)
curl -I --http2 https://airesearch.news
# Deve mostrar: HTTP/1.1 103 Early Hints antes do 200 OK
```

---

## 📚 Recursos Adicionais

- [Cloudflare Images Docs](https://developers.cloudflare.com/images/)
- [Nginx HTTP/3 Module](https://nginx.org/en/docs/http/ngx_http_v3_module.html)
- [AVIF Specification](https://aomediacodec.github.io/av1-avif/)
- [HTTP/3 RFC](https://datatracker.ietf.org/doc/html/rfc9114)

---

**Documentação completa para implementação de otimizações de infraestrutura.**













