# 🚀 Guia de Deploy na Hostinger

Este guia explica como fazer deploy do News System na Hostinger VPS e como gerenciar atualizações futuras.

## 📋 Pré-requisitos

- Droplet Hostinger VPS (Ubuntu 22.04 LTS recomendado)
- Acesso SSH ao servidor
- Domínio configurado (ex: airesearch.news)
- Git configurado no servidor

## 🔧 Setup Inicial

### 1. Conectar ao Servidor

```bash
ssh root@seu-ip-ou-dominio
```

### 2. Instalar Dependências

```bash
# Atualizar sistema
apt update && apt upgrade -y

# Instalar dependências básicas
apt install -y build-essential curl git nginx certbot python3-certbot-nginx

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Instalar Node.js (para Next.js)
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt install -y nodejs

# Instalar PostgreSQL (opcional, se usar banco)
apt install -y postgresql postgresql-contrib
```

### 3. Configurar Estrutura de Diretórios

```bash
# Criar diretório base
mkdir -p /opt/news-system
cd /opt/news-system

# Clonar repositório
git clone https://github.com/seu-usuario/hive-hub.git .

# Criar diretórios necessários
mkdir -p downloads/arxiv downloads/filtered downloads/rejected
mkdir -p output/AIResearch output/Promotional output/ScienceAI
mkdir -p logs
```

### 4. Configurar Variáveis de Ambiente

```bash
# Criar arquivo .env
cd /opt/news-system/news-backend
nano .env
```

Adicione as variáveis necessárias:

```env
# DeepSeek API
DEEPSEEK_API_KEY=sua-chave-aqui

# Database (se usar)
DATABASE_URL=postgresql://user:password@localhost/news_db

# Paths (ajustar para /opt/news-system)
REGISTRY_PATH=/opt/news-system/articles_registry.json
DOWNLOAD_DIR=/opt/news-system/downloads
OUTPUT_DIR=/opt/news-system/output
```

### 5. Compilar Backend

```bash
cd /opt/news-system/news-backend
cargo build --release
```

### 6. Build Frontend

```bash
cd /opt/news-system/apps/frontend-next/AIResearch
npm install
npm run build
```

## 🔄 Configurar Pipeline Automático (systemd)

Criar service para manter pipeline rodando:

```bash
sudo nano /etc/systemd/system/news-pipeline.service
```

```ini
[Unit]
Description=News System Pipeline - Automated Article Collection
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/news-system
Environment="PATH=/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
ExecStart=/root/.cargo/bin/cargo run --bin start --manifest-path /opt/news-system/Cargo.toml collector
Restart=always
RestartSec=10
StandardOutput=append:/opt/news-system/logs/pipeline.log
StandardError=append:/opt/news-system/logs/pipeline.error.log

[Install]
WantedBy=multi-user.target
```

Ativar service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable news-pipeline
sudo systemctl start news-pipeline

# Verificar status
sudo systemctl status news-pipeline
# Ver logs
sudo journalctl -u news-pipeline -f
```

## 🌐 Configurar Nginx

### 1. Configuração do Nginx

```bash
sudo nano /etc/nginx/sites-available/airesearch.news
```

```nginx
# Frontend AIResearch
server {
    listen 80;
    server_name airesearch.news www.airesearch.news;

    root /opt/news-system/apps/frontend-next/AIResearch/.next/standalone;
    
    # Next.js estático (se usar export)
    # OU para server mode:
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # API routes
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}

# Backend API (se necessário)
server {
    listen 80;
    server_name api.airesearch.news;

    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

```bash
# Ativar site
sudo ln -s /etc/nginx/sites-available/airesearch.news /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 2. Configurar SSL (Let's Encrypt)

```bash
sudo certbot --nginx -d airesearch.news -d www.airesearch.news
```

## 📦 Script de Deploy Automático

Criar script para facilitar atualizações futuras:

```bash
sudo nano /opt/news-system/deploy.sh
```

```bash
#!/bin/bash
set -e

echo "🚀 Starting deployment..."

# Ir para diretório do projeto
cd /opt/news-system

# Parar pipeline temporariamente
echo "⏸️  Stopping pipeline..."
sudo systemctl stop news-pipeline

# Fazer pull do Git
echo "📥 Pulling latest changes..."
git pull origin main

# Atualizar backend
echo "🔨 Building backend..."
cd news-backend
cargo build --release

# Atualizar frontend
echo "🏗️  Building frontend..."
cd ../apps/frontend-next/AIResearch
npm install
npm run build

# Restart pipeline
echo "▶️  Restarting pipeline..."
sudo systemctl start news-pipeline

# Restart Nginx
echo "🔄 Reloading Nginx..."
sudo systemctl reload nginx

echo "✅ Deployment completed!"
echo "📊 Check status: sudo systemctl status news-pipeline"
```

```bash
# Tornar executável
chmod +x /opt/news-system/deploy.sh
```

## 🔧 Adicionar Novos Collectors (APIs)

### Estrutura para Novo Collector

1. **Criar arquivo do collector:**
```bash
cd /opt/news-system/news-backend/src/collectors
nano novo_collector.rs
```

2. **Implementar trait/interface base:**

```rust
use crate::models::raw_document::ArticleMetadata;
use anyhow::Result;
use std::path::PathBuf;

pub struct NovoCollector {
    client: reqwest::Client,
    temp_dir: PathBuf,
}

impl NovoCollector {
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .user_agent("News-System-Collector/1.0")
                .build()
                .expect("Failed to create client"),
            temp_dir,
        }
    }

    pub async fn fetch_recent_papers(
        &self,
        category: &str,
        max_results: usize,
    ) -> Result<Vec<ArticleMetadata>> {
        // Implementar busca de artigos
        // Retornar Vec<ArticleMetadata>
        todo!()
    }

    pub async fn download_pdf(
        &self,
        paper_id: &str,
        pdf_url: &str,
        output_path: &PathBuf,
    ) -> Result<()> {
        // Implementar download do PDF
        todo!()
    }
}
```

3. **Registrar no mod.rs:**
```rust
// em collectors/mod.rs
pub mod novo_collector;
pub use novo_collector::*;
```

4. **Integrar no main.rs:**

```rust
// Em main.rs, adicionar função similar a run_arxiv_collection_direct

async fn run_novo_collection_direct() -> anyhow::Result<()> {
    use crate::collectors::novo_collector::NovoCollector;
    
    let registry_path = Path::new("/opt/news-system/articles_registry.json");
    let registry = RegistryManager::new(registry_path)?;
    
    let base_dir = Path::new("/opt/news-system/downloads");
    let temp_dir = base_dir.join("temp");
    let collector = NovoCollector::new(temp_dir);
    
    // Buscar artigos
    let articles = collector.fetch_recent_papers("ai", 10).await?;
    
    // Processar cada artigo (similar ao arXiv)
    // ...
    
    Ok(())
}
```

5. **Atualizar start.rs para incluir novo collector:**

```rust
// Em start.rs, adicionar opção para novo collector
match command {
    "collect" => {
        // Opção 1: Manter arXiv como padrão
        run_arxiv_collection_direct().await?;
        
        // Opção 2: Executar múltiplos collectors
        // run_novo_collection_direct().await?;
    }
    // ...
}
```

### Template de Collector

```rust
// Template base para novos collectors
// Path: news-backend/src/collectors/template_collector.rs
```

## 🔄 Workflow de Atualizações

### Processo de Atualização (Recomendado)

1. **Desenvolvimento Local:**
   ```bash
   # Fazer mudanças no código
   git add .
   git commit -m "feat: Adicionar novo collector para [API]"
   git push origin main
   ```

2. **Deploy no Servidor:**
   ```bash
   ssh root@seu-servidor
   cd /opt/news-system
   ./deploy.sh
   ```

3. **Verificar se está funcionando:**
   ```bash
   # Ver logs do pipeline
   sudo journalctl -u news-pipeline -f
   
   # Verificar se artigos estão sendo gerados
   ls -la /opt/news-system/output/AIResearch/
   ```

### Atualizações Específicas

#### Adicionar Nova API de Artigos

1. Criar collector (seguindo template acima)
2. Testar localmente
3. Commit + push
4. Deploy no servidor
5. Verificar logs

#### Atualizar Frontend

1. Fazer mudanças no frontend
2. Testar localmente (`npm run dev`)
3. Build (`npm run build`)
4. Commit + push
5. Deploy (script faz build automaticamente)

## 🔍 Monitoramento

### Scripts Úteis

```bash
# Ver status do pipeline
sudo systemctl status news-pipeline

# Ver logs em tempo real
sudo journalctl -u news-pipeline -f

# Verificar últimos artigos gerados
ls -lt /opt/news-system/output/AIResearch/ | head -10

# Verificar tamanho do registry
wc -l /opt/news-system/articles_registry.json

# Verificar uso de recursos
htop
```

### Logs

- Pipeline: `/opt/news-system/logs/pipeline.log`
- Erros: `/opt/news-system/logs/pipeline.error.log`
- Nginx: `/var/log/nginx/access.log` e `/var/log/nginx/error.log`

## 🛠️ Manutenção

### Backup

```bash
# Criar script de backup
sudo nano /opt/news-system/backup.sh
```

```bash
#!/bin/bash
BACKUP_DIR="/opt/backups/news-system"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Backup do registry
cp /opt/news-system/articles_registry.json $BACKUP_DIR/registry_$DATE.json

# Backup do output (apenas estrutura, não PDFs)
tar -czf $BACKUP_DIR/output_$DATE.tar.gz /opt/news-system/output/

# Backup do código (opcional, já está no Git)
# git archive --format=tar.gz --output=$BACKUP_DIR/code_$DATE.tar.gz HEAD

echo "✅ Backup completed: $BACKUP_DIR"
```

```bash
chmod +x /opt/news-system/backup.sh

# Adicionar ao crontab (backup diário)
crontab -e
# Adicionar: 0 2 * * * /opt/news-system/backup.sh
```

### Atualizações de Dependências

```bash
# Atualizar Rust
rustup update

# Atualizar Node.js packages
cd /opt/news-system/apps/frontend-next/AIResearch
npm update

# Rebuild após atualizações
cd /opt/news-system/news-backend
cargo build --release
```

## 📝 Checklist de Deploy

- [ ] Instalar dependências (Rust, Node.js, Nginx)
- [ ] Clonar repositório Git
- [ ] Configurar variáveis de ambiente (.env)
- [ ] Compilar backend (`cargo build --release`)
- [ ] Build frontend (`npm run build`)
- [ ] Configurar systemd service
- [ ] Configurar Nginx
- [ ] Configurar SSL (Let's Encrypt)
- [ ] Testar pipeline manualmente
- [ ] Ativar pipeline automático
- [ ] Configurar backups
- [ ] Testar atualização (deploy.sh)

## 🔐 Segurança

### Firewall (UFW)

```bash
# Permitir apenas portas necessárias
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable
```

### Chaves SSH

```bash
# Gerar chave SSH no seu computador
ssh-keygen -t ed25519 -C "seu-email@example.com"

# Copiar chave para servidor
ssh-copy-id root@seu-servidor
```

## 🆘 Troubleshooting

### Pipeline não está rodando
```bash
sudo systemctl status news-pipeline
sudo journalctl -u news-pipeline -n 50
```

### Frontend não carrega
```bash
# Verificar se Next.js está rodando
ps aux | grep node

# Verificar logs do Nginx
sudo tail -f /var/log/nginx/error.log
```

### Erro de compilação
```bash
# Limpar cache do Cargo
cd /opt/news-system/news-backend
cargo clean
cargo build --release
```

## 📚 Próximos Passos

1. Configurar múltiplos domínios (ScienceAI, etc.)
2. Adicionar novos collectors (ver seção acima)
3. Configurar monitoramento avançado
4. Setup de CI/CD automático (opcional)





























































