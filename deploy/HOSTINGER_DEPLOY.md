# 🚀 Guia de Deploy - Hostinger

Este guia detalha o processo completo de deploy do News Backend na Hostinger para produção.

---

## 📋 Pré-requisitos na Hostinger

### Requisitos Mínimos
- Plano VPS ou Cloud Hosting (compartilhado NÃO suporta Rust)
- SSH habilitado
- Root ou sudo access
- 2GB RAM mínimo
- 10GB espaço em disco

### Verificar Plano Atual
```bash
# Conectar via SSH
ssh usuario@seu-dominio.com

# Verificar sistema
uname -a
free -h
df -h
```

---

## 🔧 Passo 1: Instalar Rust no Servidor

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Escolha opção 1 (default installation)

# Carregar ambiente Rust
source $HOME/.cargo/env

# Verificar instalação
rustc --version
cargo --version
```

---

## 🔧 Passo 2: Instalar Node.js e Playwright

```bash
# Instalar Node.js via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Carregar nvm
source ~/.bashrc

# Instalar Node.js LTS
nvm install --lts
nvm use --lts

# Verificar
node --version
npm --version
```

---

## 📦 Passo 3: Upload dos Arquivos

### Opção A: Git (Recomendado)

```bash
# No servidor
cd ~
git clone https://github.com/seu-usuario/News-main.git
cd News-main

# Ou se já tem os arquivos localmente, fazer push primeiro:
# git push origin main
```

### Opção B: FTP/SFTP

Use FileZilla ou similar para enviar:
```
Local: G:\Hive-Hub\News-main\
Remoto: ~/News-main/
```

**IMPORTANTE:** Não envie:
- `target/` (será compilado no servidor)
- `node_modules/` (será instalado no servidor)
- `.env` (criar manualmente no servidor)
- `*.log`
- `users.json`

---

## 🔐 Passo 4: Configurar .env

```bash
# Criar .env no servidor
cd ~/News-main/news-backend
nano .env
```

**Conteúdo do .env:**
```bash
# ====================================
# JWT Configuration (OBRIGATÓRIO)
# ====================================
JWT_SECRET=<gerar-com-comando-abaixo>
DEFAULT_ADMIN_PASSWORD=<senha-forte-minimo-16-caracteres>

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

### Gerar JWT_SECRET Forte

```bash
# Gerar secret aleatório de 256 bits
openssl rand -base64 32

# Copiar resultado e adicionar ao .env:
# JWT_SECRET=<resultado-aqui>
```

### Salvar e Sair
- Ctrl+O (salvar)
- Enter (confirmar)
- Ctrl+X (sair)

---

## 🏗️ Passo 5: Executar Setup

```bash
# Voltar para raiz do projeto
cd ~/News-main

# Tornar script executável
chmod +x deploy/setup.sh

# Executar setup
./deploy/setup.sh
```

O script irá:
1. ✅ Verificar pré-requisitos
2. ✅ Criar diretórios necessários
3. ✅ Validar .env
4. ✅ Instalar Playwright
5. ✅ Compilar backend (5-10 minutos)
6. ✅ Configurar serviço systemd

---

## 🚀 Passo 6: Iniciar Backend

### Opção A: Systemd Service (Recomendado)

```bash
# Iniciar serviço
sudo systemctl start news-backend

# Verificar status
sudo systemctl status news-backend

# Ver logs
sudo journalctl -u news-backend -f

# Parar serviço
sudo systemctl stop news-backend

# Reiniciar serviço
sudo systemctl restart news-backend
```

### Opção B: Screen (Alternativa)

```bash
# Instalar screen
sudo apt install screen -y

# Criar sessão
screen -S news-backend

# Iniciar backend
cd ~/News-main/news-backend
./target/release/news-backend servers

# Detach: Ctrl+A, depois D
# Reattach: screen -r news-backend
```

### Opção C: Processo Direto (Teste)

```bash
cd ~/News-main/news-backend
./target/release/news-backend servers &
```

---

## 🌐 Passo 7: Configurar Nginx (Reverse Proxy)

### Instalar Nginx

```bash
sudo apt install nginx -y
```

### Configurar Site

```bash
sudo nano /etc/nginx/sites-available/news-backend
```

**Conteúdo:**
```nginx
server {
    listen 80;
    server_name seu-dominio.com www.seu-dominio.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name seu-dominio.com www.seu-dominio.com;

    # SSL Configuration (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/seu-dominio.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/seu-dominio.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # API Proxy
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

    # Dashboard (se estiver servindo frontend também)
    location / {
        root /home/usuario/News-main/news-dashboard/dist;
        try_files $uri $uri/ /index.html;
    }

    # Logs
    access_log /var/log/nginx/news-backend-access.log;
    error_log /var/log/nginx/news-backend-error.log;
}
```

### Ativar Site

```bash
# Criar link simbólico
sudo ln -s /etc/nginx/sites-available/news-backend /etc/nginx/sites-enabled/

# Testar configuração
sudo nginx -t

# Recarregar Nginx
sudo systemctl reload nginx
```

---

## 🔒 Passo 8: Configurar SSL (Let's Encrypt)

```bash
# Instalar Certbot
sudo apt install certbot python3-certbot-nginx -y

# Obter certificado
sudo certbot --nginx -d seu-dominio.com -d www.seu-dominio.com

# Escolha opção 2 (redirect HTTP to HTTPS)

# Renovação automática (já configurado por padrão)
sudo certbot renew --dry-run
```

---

## ⏰ Passo 9: Configurar Coleta Automática

Consulte: `deploy/scheduler-setup.md`

**Resumo rápido:**

```bash
# Editar cron
crontab -e

# Adicionar (ajustar caminhos):
0 6,12,18,0 * * * /home/usuario/News-main/deploy/auto-collect.sh
```

---

## 🧪 Passo 10: Testar Sistema

### 10.1. Health Check

```bash
curl http://localhost:3000/api/health
# Esperado: {"status":"ok"}
```

### 10.2. Teste de Login

```bash
# Tentar login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"<DEFAULT_ADMIN_PASSWORD>"}'

# Deve retornar token JWT
```

### 10.3. Teste de Coleta

```bash
cd ~/News-main/news-backend
./target/release/news-backend test-news-collector
```

### 10.4. Pipeline Completo

```bash
./target/release/news-backend pipeline
```

---

## 🔧 Passo 11: Manutenção e Monitoramento

### Logs do Backend

```bash
# Systemd
sudo journalctl -u news-backend -f

# Arquivo
tail -f ~/News-main/logs/backend.log
```

### Logs de Coleta

```bash
tail -f ~/News-main/logs/auto-collect-*.log
```

### Verificar Espaço

```bash
df -h
du -sh ~/News-main/*
```

### Backup

```bash
# Script de backup
cat > ~/backup-news.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=~/backups/news-$(date +%Y%m%d)
mkdir -p $BACKUP_DIR
cp ~/News-main/articles_registry.json $BACKUP_DIR/
cp ~/News-main/news-backend/.env $BACKUP_DIR/
tar -czf $BACKUP_DIR/output.tar.gz ~/News-main/output/
EOF

chmod +x ~/backup-news.sh

# Adicionar ao cron (semanal)
# 0 2 * * 0 ~/backup-news.sh
```

---

## 🚨 Troubleshooting

### Backend não inicia

```bash
# Verificar logs
sudo journalctl -u news-backend -xe

# Verificar .env
cat ~/News-main/news-backend/.env | grep JWT_SECRET
cat ~/News-main/news-backend/.env | grep DEFAULT_ADMIN_PASSWORD

# Testar manualmente
cd ~/News-main/news-backend
./target/release/news-backend servers
```

### Erro de compilação

```bash
# Limpar e recompilar
cd ~/News-main/news-backend
cargo clean
cargo build --release
```

### Playwright não funciona

```bash
# Reinstalar
cd ~/News-main/news-backend
npm install
npx playwright install chromium
npx playwright install-deps
```

### Permissões

```bash
# Ajustar proprietário
sudo chown -R $USER:$USER ~/News-main

# Ajustar permissões
chmod -R 755 ~/News-main
```

---

## ✅ Checklist de Deploy

- [ ] Rust instalado
- [ ] Node.js instalado
- [ ] Arquivos enviados
- [ ] .env configurado
- [ ] JWT_SECRET gerado (32+ caracteres)
- [ ] DEFAULT_ADMIN_PASSWORD definido (16+ caracteres)
- [ ] Setup executado (`deploy/setup.sh`)
- [ ] Backend compilado
- [ ] Backend iniciado (systemd ou screen)
- [ ] Nginx configurado
- [ ] SSL/HTTPS ativado
- [ ] Coleta automática agendada (cron)
- [ ] Health check funcionando
- [ ] Login testado
- [ ] Coleta testada
- [ ] Senha admin trocada após primeiro login
- [ ] Backup configurado
- [ ] Monitoramento ativo

---

## 📞 Comandos Úteis

```bash
# Status do serviço
sudo systemctl status news-backend

# Reiniciar
sudo systemctl restart news-backend

# Ver logs (últimas 50 linhas)
sudo journalctl -u news-backend -n 50

# Testar coleta
cd ~/News-main/news-backend && ./target/release/news-backend test-news-collector

# Limpar artigos antigos (30 dias)
cd ~/News-main/news-backend && ./target/release/news-backend clean-old-articles 30

# Verificar processos
ps aux | grep news-backend

# Uso de recursos
htop
```

---

## 🎯 Próximos Passos

1. ✅ Deploy concluído
2. 🔐 Trocar senha admin
3. 📊 Monitorar primeira coleta automática
4. 🌐 Configurar frontends (ScienceAI, AIResearch)
5. 📧 Configurar alertas (opcional)
6. 📈 Configurar analytics (opcional)

---

**Documentação gerada para deploy em produção na Hostinger**




















