# 🚀 Comandos de Deploy - Hostinger

**Data:** 2025-11-06 11:15:00
**Status:** PRONTO PARA DEPLOY

---

## 📋 Checklist Final Antes do Deploy

### ✅ No Seu Computador (Local)
- [x] Vulnerabilidades corrigidas
- [x] Scripts de deploy criados
- [x] Documentação completa
- [x] Commits feitos e push para GitHub
- [x] Código testado localmente

### 🔧 No Servidor Hostinger

---

## 🚀 Deploy Passo-a-Passo

### 1. Acessar o Servidor
```bash
ssh usuario@seu-dominio.com
```

### 2. Atualizar Sistema
```bash
sudo apt update && sudo apt upgrade -y
```

### 3. Instalar Dependências Essenciais
```bash
# Instalar ferramentas básicas
sudo apt install -y curl wget git unzip build-essential pkg-config

# Instalar Node.js 18 LTS
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verificar versões
node --version  # Deve ser v18.x
npm --version   # Deve ser 9.x+
```

### 4. Instalar Rust
```bash
# Instalar Rust estável
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Carregar Rust no PATH
source $HOME/.cargo/env

# Verificar instalação
rustc --version  # Deve ser 1.7x.x
cargo --version  # Deve ser 1.7x.x
```

### 5. Clonar/Download do Projeto
```bash
# Navegar para diretório público
cd ~/public_html

# Clonar repositório
git clone https://github.com/seu-usuario/News-main.git

# Ou baixar ZIP se não tiver Git
# wget https://github.com/seu-usuario/News-main/archive/main.zip
# unzip main.zip && mv News-main-main News-main

# Entrar no projeto
cd News-main
```

### 6. Criar .env com Secrets
```bash
# Copiar template
cp .env.example .env

# Editar .env (IMPORTANTE!)
nano .env

# Conteúdo necessário:
# ====================================
# JWT Configuration (OBRIGATÓRIO)
# ====================================
JWT_SECRET=<gerar-com-openssl-rand>
DEFAULT_ADMIN_PASSWORD=<senha-forte-min-16-caracteres>

# ====================================
# Paths (ajustar para servidor)
# ====================================
NEWS_BASE_DIR=/home/usuario/News-main

# ====================================
# API Keys (Opcional)
# ====================================
NATURE_API_KEY=
SCIENCE_API_KEY=
IEEE_API_KEY=
SPRINGER_API_KEY=
ELSEVIER_API_KEY=

# ====================================
# Server Configuration
# ====================================
HOST=0.0.0.0
PORT=3000
RUST_LOG=info

# ====================================
# Database
# ====================================
DATABASE_URL=sqlite:./data/news.db
```

**IMPORTANTE:** Gerar JWT_SECRET seguro:
```bash
openssl rand -base64 32
# Copie o resultado e coloque no JWT_SECRET
```

### 7. Executar Setup Automático
```bash
# Tornar script executável
chmod +x deploy/setup.sh

# Executar setup (pode levar 10-15 minutos na primeira vez)
./deploy/setup.sh

# Verificar se compilou sem erros
echo $?
# Deve retornar 0 se tudo OK
```

### 8. Testar Backend Localmente
```bash
# Iniciar backend para teste
cd news-backend
./target/release/news-backend servers &
BACKEND_PID=$!

# Aguardar 5 segundos
sleep 5

# Testar health check
curl http://localhost:3000/api/health

# Parar teste
kill $BACKEND_PID
cd ..
```

### 9. Configurar Nginx (Reverse Proxy)
```bash
# Instalar Nginx
sudo apt install nginx -y

# Criar configuração do site
sudo nano /etc/nginx/sites-available/news-backend

# Conteúdo (substitua seu-dominio.com):
server {
    listen 80;
    server_name seu-dominio.com www.seu-dominio.com;

    # Redirect para HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name seu-dominio.com www.seu-dominio.com;

    # SSL (será configurado no passo 10)
    ssl_certificate /etc/letsencrypt/live/seu-dominio.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/seu-dominio.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Proxy para API
    location /api/ {
        proxy_pass http://localhost:3000/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 300s;
        proxy_connect_timeout 300s;
    }

    # Servir frontend (ScienceAI e AIResearch)
    location / {
        # Tentar servir ScienceAI primeiro
        try_files $uri $uri/ @scienceai;

        # Headers de segurança
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;
    }

    location @scienceai {
        root /home/usuario/News-main/apps/frontend-next/ScienceAI/dist;
        try_files /index.html =404;
    }

    # Logs
    access_log /var/log/nginx/news-backend-access.log;
    error_log /var/log/nginx/news-backend-error.log;
}

# Ativar site
sudo ln -s /etc/nginx/sites-available/news-backend /etc/nginx/sites-enabled/

# Testar configuração
sudo nginx -t

# Recarregar Nginx
sudo systemctl reload nginx
```

### 10. Configurar SSL (Let's Encrypt)
```bash
# Instalar Certbot
sudo apt install certbot python3-certbot-nginx -y

# Gerar certificado (substitua seu domínio)
sudo certbot --nginx -d seu-dominio.com -d www.seu-dominio.com

# Escolher opção: 2 (Redirect to HTTPS)
```

### 11. Configurar Systemd Service
```bash
# Verificar se serviço foi criado pelo setup.sh
sudo systemctl status news-backend

# Se não foi criado, criar manualmente:
sudo nano /etc/systemd/system/news-backend.service

# Conteúdo:
[Unit]
Description=News Backend Service
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/home/usuario/News-main/news-backend
Environment="RUST_LOG=info"
ExecStart=/home/usuario/News-main/news-backend/target/release/news-backend servers
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target

# Recarregar e iniciar
sudo systemctl daemon-reload
sudo systemctl enable news-backend
sudo systemctl start news-backend
```

### 12. Configurar Coleta Automática
```bash
# Editar crontab
crontab -e

# Adicionar jobs (ajuste caminhos):
# Coleta principal - 4x/dia
0 6,12,18,0 * * * /home/usuario/News-main/deploy/auto-collect.sh >> /home/usuario/News-main/logs/auto-collect.log 2>&1

# Health check - a cada hora
0 * * * * /home/usuario/News-main/deploy/health-check.sh >> /home/usuario/News-main/logs/health-check.log 2>&1

# Backup semanal
0 3 * * 0 cp /home/usuario/News-main/articles_registry.json /home/usuario/News-main/backups/registry-$(date +\%Y\%m\%d).json
```

### 13. Testar Sistema Completo
```bash
# Aguardar 30 segundos para tudo iniciar
sleep 30

# Testar health check
curl -k https://seu-dominio.com/api/health

# Testar coleta manual
cd /home/usuario/News-main/news-backend
./target/release/news-backend test-news-collector

# Verificar logs
tail -20 /home/usuario/News-main/logs/backend.log
```

### 14. Configurar Frontend
```bash
# Build do ScienceAI
cd /home/usuario/News-main/apps/frontend-next/ScienceAI
npm install
npm run build

# Copiar build para local correto (se necessário)
# cp -r dist/* /home/usuario/News-main/apps/frontend-next/ScienceAI/dist/
```

### 15. Primeiro Login e Configuração
```bash
# Acessar dashboard: https://seu-dominio.com/dashboard
# Username: admin
# Password: <DEFAULT_ADMIN_PASSWORD do .env>

# ⚠️  IMPORTANTE: Trocar senha imediatamente!
```

---

## 🔍 Verificação Pós-Deploy

### Logs para Monitorar
```bash
# Backend
tail -f ~/News-main/logs/backend.log

# Coleta automática
tail -f ~/News-main/logs/auto-collect.log

# Health check
tail -f ~/News-main/logs/health-check.log

# Nginx
sudo tail -f /var/log/nginx/news-backend-access.log
```

### Comandos Úteis
```bash
# Status dos serviços
sudo systemctl status news-backend
sudo systemctl status nginx

# Reiniciar serviços
sudo systemctl restart news-backend
sudo systemctl restart nginx

# Ver processos
ps aux | grep news-backend
ps aux | grep nginx

# Verificar portas
netstat -tlnp | grep :3000
netstat -tlnp | grep :80
netstat -tlnp | grep :443

# Teste de conectividade
curl -I https://seu-dominio.com
curl https://seu-dominio.com/api/health
```

---

## 🚨 Troubleshooting

### Backend não inicia
```bash
# Ver erro detalhado
sudo journalctl -u news-backend -xe -n 50

# Verificar .env
cat ~/News-main/news-backend/.env | grep JWT_SECRET
cat ~/News-main/news-backend/.env | grep DEFAULT_ADMIN_PASSWORD

# Testar manualmente
cd ~/News-main/news-backend
./target/release/news-backend servers
```

### Nginx erro 502
```bash
# Backend pode não estar rodando
sudo systemctl status news-backend

# Verificar porta
netstat -tlnp | grep :3000

# Testar proxy
curl http://localhost:3000/api/health
```

### Certificado SSL falha
```bash
# Renovar certificado
sudo certbot renew

# Verificar expiração
sudo certbot certificates
```

### Coleta não funciona
```bash
# Testar manualmente
cd ~/News-main/news-backend
./target/release/news-backend test-news-collector

# Ver logs
tail -50 ~/News-main/logs/auto-collect.log
```

---

## 📊 Métricas de Sucesso

Após deploy, monitorar:
- ✅ HTTPS funcionando (SSL Labs A+)
- ✅ API respondendo (/api/health)
- ✅ Dashboard acessível
- ✅ Coleta automática executando
- ✅ Artigos sendo gerados
- ✅ Frontend carregando
- ✅ Imagens sem repetição
- ✅ Performance aceitável (< 2s response)

---

## 🎉 Deploy Concluído!

Sistema operacional com:
- ✅ Backend Rust (API + coleta)
- ✅ Frontend Next.js (ScienceAI + AIResearch)
- ✅ Nginx (reverse proxy + SSL)
- ✅ Systemd (gerenciamento de serviços)
- ✅ Cron (coleta automática)
- ✅ Backup automático
- ✅ Monitoramento de saúde

**Próximos passos:**
1. Trocar senha admin
2. Configurar analytics (opcional)
3. Configurar CDN para imagens (opcional)
4. Monitorar performance por 24h

---

*Gerado automaticamente para deploy em produção*






