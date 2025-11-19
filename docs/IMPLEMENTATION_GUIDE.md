# 🚀 Guia Prático: Como Implementar Otimizações de Infraestrutura

Guia passo a passo para implementar as otimizações que dependem de infraestrutura.

## 📋 Índice

1. [Opção Rápida: Cloudflare (Recomendado)](#opção-rápida-cloudflare-recomendado)
2. [Opção Avançada: Servidor Próprio](#opção-avançada-servidor-próprio)
3. [Verificação e Testes](#verificação-e-testes)

---

## Opção Rápida: Cloudflare (Recomendado) ⭐

A forma mais rápida e eficiente de implementar todas as otimizações.

### Passo 1: Criar Conta no Cloudflare

```bash
# 1. Acesse: https://dash.cloudflare.com/sign-up
# 2. Crie conta gratuita (plano Free é suficiente para começar)
# 3. Adicione seu domínio (airesearch.news, scienceai.news)
```

### Passo 2: Configurar DNS

```bash
# 1. No painel Cloudflare:
# Dashboard → DNS → Records

# 2. Adicionar registros:

# Para AIResearch:
Type: A
Name: @
Content: {IP_DO_SEU_SERVIDOR}
Proxy: 🟠 Proxied (IMPORTANTE - ativa CDN)

Type: A
Name: www
Content: {IP_DO_SEU_SERVIDOR}
Proxy: 🟠 Proxied

# Para ScienceAI (se usar domínio diferente):
Type: A
Name: @
Content: {IP_DO_SEU_SERVIDOR}
Proxy: 🟠 Proxied

Type: A
Name: www
Content: {IP_DO_SEU_SERVIDOR}
Proxy: 🟠 Proxied

# 3. Aguardar propagação DNS (1-5 minutos)
# 4. Verificar: dig airesearch.news
# Deve retornar IPs do Cloudflare (não seu servidor)
```

### Passo 3: Ativar SSL/TLS

```bash
# Dashboard → SSL/TLS → Overview
# ✅ Full (Strict) - recomendado
# Isso ativa HTTPS automaticamente

# Verificar: https://airesearch.news deve abrir com SSL válido
```

### Passo 4: Ativar Otimizações de Velocidade

```bash
# Dashboard → Speed → Optimization

# ✅ Auto Minify
- ✅ JavaScript
- ✅ CSS
- ✅ HTML

# ✅ Image Optimization
- ✅ Polish: Lossless (ou Lossy para compressão maior)
- ✅ WebP: ON
- ✅ AVIF: ON (se disponível no plano)
- ✅ Mirage: ON (otimização mobile)

# ✅ Caching
- ✅ Browser Cache TTL: 1 month
- ✅ Always Online: ON

# ✅ Network
- ✅ HTTP/2: ON
- ✅ HTTP/3 (with QUIC): ON
- ✅ 0-RTT Connection Resumption: ON

# ✅ Compression
- ✅ Brotli: ON

# ✅ Early Hints
- ✅ ON (automaticamente ativo)
```

### Passo 5: Configurar Cache Rules

```bash
# Dashboard → Rules → Page Rules (ou Cache Rules)

# Regra 1: Cache estático agressivo
URL: *airesearch.news/_next/static/*
Settings:
  - Cache Level: Cache Everything
  - Edge Cache TTL: 1 month
  - Browser Cache TTL: 1 month

# Regra 2: Imagens
URL: *airesearch.news/images/*
Settings:
  - Cache Level: Cache Everything
  - Edge Cache TTL: 1 year
  - Browser Cache TTL: 1 year
  - Polish: ON

# Regra 3: API com cache curto
URL: *airesearch.news/api/articles*
Settings:
  - Cache Level: Standard
  - Edge Cache TTL: 5 minutes
  - Browser Cache TTL: 1 minute

# Regra 4: HTML sem cache (páginas dinâmicas)
URL: *airesearch.news/*
Settings:
  - Cache Level: Bypass
  - (para páginas que mudam frequentemente)
```

### Passo 6: Verificar

```bash
# Testar HTTP/3
curl -I --http3 https://airesearch.news

# Testar AVIF
curl -H "Accept: image/avif" -I https://airesearch.news/images/article.jpg

# Testar Cache
curl -I https://airesearch.news/_next/static/css/main.css
# Deve mostrar: Cache-Control: public, max-age=31536000, immutable

# Ver no navegador:
# Chrome DevTools → Network → Protocol → deve mostrar "h3"
```

**✅ PRONTO!** Todas as otimizações estão ativas.

---

## Opção Avançada: Servidor Próprio

Para quem prefere controlar tudo no próprio servidor.

### Passo 1: Instalar Nginx com Módulos Necessários

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y nginx nginx-extras libnginx-mod-http-brotli

# Verificar versão (precisa ser 1.25.1+ para HTTP/3)
nginx -v

# Instalar Brotli (se não veio com nginx-extras)
sudo apt install -y libbrotli-dev
```

### Passo 2: Instalar SSL (Let's Encrypt)

```bash
# Instalar Certbot
sudo apt install -y certbot python3-certbot-nginx

# Obter certificado
sudo certbot --nginx -d airesearch.news -d www.airesearch.news
sudo certbot --nginx -d scienceai.news -d www.scienceai.news

# Renovação automática (já configurado por padrão)
sudo certbot renew --dry-run
```

### Passo 3: Configurar Nginx

```bash
# Copiar configuração otimizada
sudo cp News-main/docker/nginx/optimized.conf /etc/nginx/sites-available/optimized

# Ou editar diretamente
sudo nano /etc/nginx/sites-available/airesearch.news
```

**Conteúdo completo:** Ver `News-main/docker/nginx/optimized.conf`

**Principais configurações:**

```nginx
# HTTP/3 (se compilado com módulo)
listen 443 http3 reuseport;
listen 443 ssl http2;  # Fallback

# Alt-Svc header para HTTP/3
add_header Alt-Svc 'h3=":443"; ma=86400' always;

# AVIF/WebP conversion
location ~* \.(jpg|jpeg|png|webp)$ {
    # Verificar Accept header e servir formato adequado
    if ($http_accept ~* "image/avif") {
        rewrite ^(.+)\.(jpg|jpeg|png|webp)$ $1.avif last;
    }
}

# Cache forte
location ~* \.(jpg|jpeg|png|gif|webp|avif|ico|svg|woff|woff2)$ {
    expires 1y;
    add_header Cache-Control "public, max-age=31536000, immutable";
}

# Brotli + Gzip
brotli on;
brotli_comp_level 6;
gzip on;
gzip_comp_level 6;
```

### Passo 4: Compilar Nginx com HTTP/3 (Opcional)

Se quiser HTTP/3 no servidor próprio (recomendado usar Cloudflare):

```bash
# Ver guia completo em: docs/INFRASTRUCTURE_OPTIMIZATIONS.md
# Seção "2. HTTP/3 + QUIC" → Opção B
```

### Passo 5: Converter Imagens para AVIF

```bash
# Instalar ferramentas de conversão
sudo apt install -y libavif-bin imagemagick

# Criar script de conversão
cat > scripts/convert-to-avif.sh << 'EOF'
#!/bin/bash
IMAGE_DIR="/opt/news-system/images"

find "$IMAGE_DIR" -type f \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" \) | while read img; do
    avif_file="${img%.*}.avif"
    if [ ! -f "$avif_file" ]; then
        echo "Converting: $img → $avif_file"
        avifenc -c aom -s 6 "$img" "$avif_file" 2>/dev/null || \
        magick "$img" -quality 80 "$avif_file"
    fi
done

echo "✅ AVIF conversion completed"
EOF

chmod +x scripts/convert-to-avif.sh

# Executar conversão
./scripts/convert-to-avif.sh
```

### Passo 6: Ativar e Testar

```bash
# Testar configuração
sudo nginx -t

# Recarregar Nginx
sudo systemctl reload nginx

# Verificar logs
sudo tail -f /var/log/nginx/airesearch-access.log
```

---

## Verificação e Testes

### Teste 1: HTTP/3

```bash
# Terminal
curl -I --http3 https://airesearch.news

# Deve mostrar: HTTP/3 200

# Navegador:
# Chrome DevTools → Network → Protocol → deve mostrar "h3"
```

### Teste 2: AVIF

```bash
# Testar se AVIF está sendo servido
curl -H "Accept: image/avif" -I https://airesearch.news/images/article.jpg

# Deve retornar:
# Content-Type: image/avif
# OU Location: *.avif
```

### Teste 3: Cache

```bash
# Testar headers de cache
curl -I https://airesearch.news/_next/static/css/main.css

# Deve mostrar:
# Cache-Control: public, max-age=31536000, immutable
# Expires: (data futura ~1 ano)
```

### Teste 4: Early Hints

```bash
# Testar Early Hints (HTTP/2 ou superior)
curl -I --http2 https://airesearch.news

# Deve mostrar:
# HTTP/1.1 103 Early Hints
# Link: </_next/static/css/app.css>; rel=preload; as=style
# HTTP/1.1 200 OK
```

### Teste 5: Brotli/Gzip

```bash
# Testar compressão
curl -H "Accept-Encoding: br" -I https://airesearch.news/_next/static/js/main.js

# Deve mostrar:
# Content-Encoding: br
```

### Teste 6: PageSpeed Insights

```bash
# Testar no Google PageSpeed Insights:
# https://pagespeed.web.dev/

# Resultados esperados:
# ✅ Performance: 90+
# ✅ LCP: < 2.5s
# ✅ FID: < 100ms
# ✅ CLS: < 0.1
```

---

## 🔧 Troubleshooting

### HTTP/3 não funciona

```bash
# Verificar se Cloudflare está ativo
curl -I https://airesearch.news
# Deve mostrar: cf-ray (header do Cloudflare)

# Verificar Alt-Svc header
curl -I https://airesearch.news | grep -i alt-svc
# Deve mostrar: Alt-Svc: h3=":443"; ma=86400

# Se não usar Cloudflare, verificar se Nginx foi compilado com HTTP/3
nginx -V 2>&1 | grep -i quic
# Deve mostrar: --with-http_v3_module
```

### AVIF não está sendo servido

```bash
# Verificar se imagens AVIF existem
ls -la /opt/news-system/images/*.avif

# Se não existirem, converter:
./scripts/convert-to-avif.sh

# Verificar configuração Nginx
sudo nginx -t
sudo cat /etc/nginx/sites-available/airesearch.news | grep -A 10 "image/avif"

# Testar manualmente
curl -H "Accept: image/avif" -I https://airesearch.news/images/article.jpg
```

### Cache não está funcionando

```bash
# Verificar headers de resposta
curl -I https://airesearch.news/_next/static/css/main.css

# Verificar Cloudflare Cache Rules
# Dashboard → Rules → Cache Rules

# Verificar Nginx
sudo cat /etc/nginx/sites-available/airesearch.news | grep -A 5 "Cache-Control"

# Limpar cache do Cloudflare (se necessário)
# Dashboard → Caching → Purge Cache → Purge Everything
```

---

## 📊 Monitoramento

### Cloudflare Analytics

```bash
# Dashboard → Analytics → Web Traffic
# Ver métricas de:
# - Requisições por segundo
# - Cache hit rate (deve ser > 80%)
# - Bandwidth economizado
# - Requisições HTTP/3
```

### Nginx Logs

```bash
# Ver acesso
sudo tail -f /var/log/nginx/airesearch-access.log

# Ver erros
sudo tail -f /var/log/nginx/airesearch-error.log

# Analisar cache hits
sudo cat /var/log/nginx/airesearch-access.log | grep "X-Cache-Status" | sort | uniq -c
# Deve mostrar principalmente: HIT (não MISS)
```

---

## ✅ Checklist Final

### Configuração Básica

- [ ] DNS configurado no Cloudflare (ou servidor próprio)
- [ ] SSL/TLS ativado (Let's Encrypt)
- [ ] Nginx configurado e funcionando
- [ ] Sites acessíveis via HTTPS

### Otimizações de Infraestrutura

- [ ] **CDN**: Cloudflare ativo (🟠 Proxied) OU Nginx configurado
- [ ] **HTTP/3**: Ativado no Cloudflare OU Nginx compilado com QUIC
- [ ] **AVIF**: Conversão ativa no Cloudflare OU imagens convertidas + Nginx configurado
- [ ] **Cache Forte**: Regras configuradas (1 ano para estáticos, 5min para API)
- [ ] **Early Hints**: Ativo no Cloudflare OU configurado no Nginx
- [ ] **Brotli**: Ativado no Cloudflare OU Nginx com módulo brotli
- [ ] **Gzip**: Fallback configurado

### Verificação

- [ ] HTTP/3 funcionando (curl --http3)
- [ ] AVIF sendo servido (curl com Accept: image/avif)
- [ ] Cache funcionando (headers Cache-Control corretos)
- [ ] Early Hints ativo (HTTP 103 antes do 200)
- [ ] Brotli funcionando (Content-Encoding: br)
- [ ] PageSpeed Insights: Performance 90+
- [ ] Sem erros nos logs do Nginx

---

## 🎯 Resultados Esperados

Após implementar todas as otimizações:

**Antes:**
- Performance: ~70-80
- LCP: ~3-4s
- Cache hit rate: ~30%

**Depois:**
- Performance: 90-100 ✅
- LCP: < 2.5s ✅
- Cache hit rate: > 80% ✅
- HTTP/3: Ativo ✅
- AVIF: Servido automaticamente ✅

---

## 📚 Próximos Passos

1. **Monitorar métricas** por 1 semana
2. **Ajustar cache TTL** conforme necessário
3. **Otimizar imagens** ainda não convertidas
4. **Configurar alertas** para problemas de performance
5. **Documentar configurações** específicas do seu ambiente

---

**Documentação completa para implementação prática de otimizações de infraestrutura.**









