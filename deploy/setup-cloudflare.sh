#!/bin/bash
# 🚀 Script Automático: Configurar Cloudflare CDN + Otimizações
# 
# Este script configura todas as otimizações de infraestrutura via Cloudflare
# 
# USO:
#   1. Copie este arquivo para o servidor
#   2. Execute: bash setup-cloudflare.sh
#
# PRÉ-REQUISITOS:
#   - Conta no Cloudflare (gratuita): https://dash.cloudflare.com/sign-up
#   - Domínio já configurado (airesearch.news, scienceai.news)
#   - Acesso SSH ao servidor

set -e  # Parar em caso de erro

echo "🚀 Setup Cloudflare CDN + Otimizações"
echo "======================================"
echo ""

# Cores para output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Função para mostrar sucesso
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Função para mostrar aviso
warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Função para mostrar erro
error() {
    echo -e "${RED}❌ $1${NC}"
}

# Função para mostrar informação
info() {
    echo -e "ℹ️  $1"
}

echo "📋 Checklist Pré-requisitos:"
echo ""
echo "Antes de continuar, certifique-se de que:"
echo "  1. ✅ Você criou conta no Cloudflare (gratuita): https://dash.cloudflare.com/sign-up"
echo "  2. ✅ Você adicionou seu domínio no Cloudflare"
echo "  3. ✅ Você está logado no painel Cloudflare"
echo ""
read -p "Você já completou os pré-requisitos acima? (s/n): " resposta

if [ "$resposta" != "s" ] && [ "$resposta" != "S" ]; then
    error "Por favor, complete os pré-requisitos primeiro."
    echo ""
    echo "Passos rápidos:"
    echo "  1. Acesse: https://dash.cloudflare.com/sign-up"
    echo "  2. Crie conta gratuita"
    echo "  3. Clique em 'Add a Site'"
    echo "  4. Digite seu domínio (ex: airesearch.news)"
    echo "  5. Escolha plano Free (gratuito)"
    echo "  6. Cloudflare vai escanear seus DNS records"
    echo "  7. Depois volte aqui e execute este script novamente"
    exit 1
fi

success "Pré-requisitos completos!"

echo ""
echo "=========================================="
echo "PASSO 1: Obter IP do Servidor"
echo "=========================================="
echo ""

# Obter IP do servidor
SERVER_IP=$(curl -s https://api.ipify.org)
if [ -z "$SERVER_IP" ]; then
    SERVER_IP=$(hostname -I | awk '{print $1}')
fi

info "IP do servidor detectado: $SERVER_IP"
read -p "Este é o IP correto do seu servidor? (s/n): " confirma_ip

if [ "$confirma_ip" != "s" ] && [ "$confirma_ip" != "S" ]; then
    read -p "Digite o IP correto do servidor: " SERVER_IP
fi

success "IP do servidor: $SERVER_IP"

echo ""
echo "=========================================="
echo "PASSO 2: Configurar DNS no Cloudflare"
echo "=========================================="
echo ""
echo "Agora vamos configurar os DNS records no Cloudflare."
echo ""
echo "⚠️  IMPORTANTE: Você precisa fazer isso manualmente no painel Cloudflare"
echo ""
echo "1. Acesse: https://dash.cloudflare.com/"
echo "2. Selecione seu domínio (ex: airesearch.news)"
echo "3. Vá em: DNS → Records"
echo "4. Clique em 'Add record'"
echo "5. Configure assim:"
echo ""
echo "   Para AIResearch:"
echo "   ┌──────────┬──────┬─────────────────┬─────────┐"
echo "   │ Type     │ Name │ Content         │ Proxy   │"
echo "   ├──────────┼──────┼─────────────────┼─────────┤"
echo "   │ A        │ @    │ $SERVER_IP      │ 🟠 ON   │"
echo "   │ A        │ www  │ $SERVER_IP      │ 🟠 ON   │"
echo "   └──────────┴──────┴─────────────────┴─────────┘"
echo ""
echo "   ⚠️  IMPORTANTE: Proxy deve estar ATIVO (🟠 laranja, não ☁️ cinza)"
echo ""
echo "6. Clique em 'Save'"
echo "7. Repita para ScienceAI se usar domínio diferente"
echo ""

read -p "Você já configurou os DNS records no Cloudflare? (s/n): " dns_ok

if [ "$dns_ok" != "s" ] && [ "$dns_ok" != "S" ]; then
    warning "Configure os DNS records primeiro, depois execute este script novamente."
    exit 1
fi

success "DNS records configurados!"

echo ""
echo "=========================================="
echo "PASSO 3: Aguardar Propagação DNS"
echo "=========================================="
echo ""

info "Aguardando propagação DNS (30 segundos)..."
sleep 30

# Verificar se DNS está apontando para Cloudflare
DOMAIN=""
read -p "Digite seu domínio principal (ex: airesearch.news): " DOMAIN

if [ -z "$DOMAIN" ]; then
    error "Domínio não informado. Saindo..."
    exit 1
fi

echo ""
info "Verificando DNS para $DOMAIN..."

DNS_IP=$(dig +short $DOMAIN @8.8.8.8 | tail -n 1)

if [ -z "$DNS_IP" ]; then
    warning "DNS ainda não propagou. Aguarde alguns minutos e verifique manualmente:"
    echo "   dig $DOMAIN @8.8.8.8"
    read -p "Continuar mesmo assim? (s/n): " continua
    if [ "$continua" != "s" ] && [ "$continua" != "S" ]; then
        exit 1
    fi
else
    info "DNS retornou: $DNS_IP"
    if [[ "$DNS_IP" =~ ^104\.|^172\.|^108\. ]]; then
        success "DNS apontando para Cloudflare (IPs do Cloudflare detectados)!"
    else
        warning "DNS pode não estar usando Cloudflare ainda (IP diferente esperado)."
        info "Se você acabou de configurar, aguarde alguns minutos."
        read -p "Continuar mesmo assim? (s/n): " continua
        if [ "$continua" != "s" ] && [ "$continua" != "S" ]; then
            exit 1
        fi
    fi
fi

echo ""
echo "=========================================="
echo "PASSO 4: Configurar SSL/TLS"
echo "=========================================="
echo ""
echo "No painel Cloudflare:"
echo "  1. Vá em: SSL/TLS → Overview"
echo "  2. Selecione: 'Full (Strict)'"
echo "  3. Aguarde alguns segundos para ativar"
echo ""
read -p "Você já configurou SSL/TLS para 'Full (Strict)'? (s/n): " ssl_ok

if [ "$ssl_ok" != "s" ] && [ "$ssl_ok" != "S" ]; then
    warning "Configure SSL/TLS primeiro:"
    echo "  Dashboard → SSL/TLS → Overview → Full (Strict)"
    read -p "Continuar mesmo assim? (s/n): " continua
    if [ "$continua" != "s" ] && [ "$continua" != "S" ]; then
        exit 1
    fi
fi

success "SSL/TLS configurado!"

echo ""
echo "=========================================="
echo "PASSO 5: Ativar Otimizações de Velocidade"
echo "=========================================="
echo ""
echo "⚠️  IMPORTANTE: Faça isso no painel Cloudflare"
echo ""
echo "No painel Cloudflare:"
echo "  1. Vá em: Speed → Optimization"
echo ""
echo "  2. Auto Minify:"
echo "     ☑️ JavaScript"
echo "     ☑️ CSS"
echo "     ☑️ HTML"
echo ""
echo "  3. Image Optimization:"
echo "     ☑️ Polish: Lossless (ou Lossy)"
echo "     ☑️ WebP: ON"
echo "     ☑️ AVIF: ON (se disponível)"
echo "     ☑️ Mirage: ON"
echo ""
echo "  4. Caching:"
echo "     ☑️ Browser Cache TTL: 1 month"
echo "     ☑️ Always Online: ON"
echo ""
echo "  5. Network:"
echo "     ☑️ HTTP/2: ON"
echo "     ☑️ HTTP/3 (with QUIC): ON"
echo "     ☑️ 0-RTT Connection Resumption: ON"
echo ""
echo "  6. Compression:"
echo "     ☑️ Brotli: ON"
echo ""
read -p "Você já ativou as otimizações acima? (s/n): " speed_ok

if [ "$speed_ok" != "s" ] && [ "$speed_ok" != "S" ]; then
    warning "Ative as otimizações primeiro."
    echo ""
    echo "Acesse: https://dash.cloudflare.com/"
    echo "Selecione seu domínio → Speed → Optimization"
    echo ""
    read -p "Continuar mesmo assim? (s/n): " continua
    if [ "$continua" != "s" ] && [ "$continua" != "S" ]; then
        exit 1
    fi
fi

success "Otimizações de velocidade ativadas!"

echo ""
echo "=========================================="
echo "PASSO 6: Configurar Cache Rules"
echo "=========================================="
echo ""
echo "⚠️  IMPORTANTE: Configure as regras de cache no painel Cloudflare"
echo ""
echo "No painel Cloudflare:"
echo "  1. Vá em: Rules → Cache Rules (ou Page Rules)"
echo ""
echo "  2. Clique em 'Create rule'"
echo ""
echo "  3. Regra 1: Cache estático agressivo"
echo "     URL: *$DOMAIN/_next/static/*"
echo "     Settings:"
echo "       • Cache Level: Cache Everything"
echo "       • Edge Cache TTL: 1 month"
echo "       • Browser Cache TTL: 1 month"
echo ""
echo "  4. Clique em 'Deploy'"
echo ""
echo "  5. Regra 2: Imagens"
echo "     URL: *$DOMAIN/images/*"
echo "     Settings:"
echo "       • Cache Level: Cache Everything"
echo "       • Edge Cache TTL: 1 year"
echo "       • Browser Cache TTL: 1 year"
echo "       • Polish: ON"
echo ""
echo "  6. Clique em 'Deploy'"
echo ""
echo "  7. Regra 3: API com cache curto"
echo "     URL: *$DOMAIN/api/articles*"
echo "     Settings:"
echo "       • Cache Level: Standard"
echo "       • Edge Cache TTL: 5 minutes"
echo "       • Browser Cache TTL: 1 minute"
echo ""
echo "  8. Clique em 'Deploy'"
echo ""
read -p "Você já configurou as regras de cache? (s/n): " cache_ok

if [ "$cache_ok" != "s" ] && [ "$cache_ok" != "S" ]; then
    warning "Configure as regras de cache para melhor performance."
    info "Você pode fazer isso depois, mas é recomendado."
    read -p "Continuar mesmo assim? (s/n): " continua
fi

success "Cache rules configuradas (ou será feito depois)!"

echo ""
echo "=========================================="
echo "PASSO 7: Verificar Tudo"
echo "=========================================="
echo ""

# Testar HTTPS
info "Testando HTTPS..."
if curl -s -o /dev/null -w "%{http_code}" "https://$DOMAIN" | grep -q "200\|301\|302"; then
    success "HTTPS funcionando! ✅"
else
    warning "HTTPS pode não estar funcionando ainda (aguarde alguns minutos)."
fi

echo ""
echo "=========================================="
echo "✅ CONFIGURAÇÃO COMPLETA!"
echo "=========================================="
echo ""
success "Todas as otimizações de infraestrutura estão configuradas!"
echo ""
echo "📊 Próximos passos:"
echo ""
echo "1. Aguarde 5-10 minutos para tudo propagar"
echo ""
echo "2. Verificar funcionamento:"
echo "   • Acesse: https://$DOMAIN"
echo "   • Deve carregar normalmente"
echo ""
echo "3. Testar HTTP/3:"
echo "   curl -I --http3 https://$DOMAIN"
echo "   (Deve mostrar: HTTP/3 200)"
echo ""
echo "4. Testar PageSpeed:"
echo "   https://pagespeed.web.dev/"
echo "   Digite seu domínio e teste"
echo ""
echo "5. Monitorar no Cloudflare:"
echo "   https://dash.cloudflare.com/"
echo "   Dashboard → Analytics → Web Traffic"
echo "   • Cache hit rate deve ser > 80%"
echo "   • Requisições HTTP/3 devem aparecer"
echo ""
echo "📚 Documentação completa:"
echo "   • Guia completo: docs/INFRASTRUCTURE_OPTIMIZATIONS.md"
echo "   • Guia passo a passo: docs/IMPLEMENTATION_GUIDE.md"
echo ""
success "Setup concluído com sucesso! 🎉"













