# 🚀 Quick Start - Deploy na Hostinger

Guia rápido para fazer deploy do News System na Hostinger VPS.

## 📋 Passo a Passo

### 1. Conectar ao Servidor

```bash
ssh root@seu-ip-ou-dominio
```

### 2. Setup Inicial (Automático)

```bash
# Clonar repositório
cd /opt
git clone https://github.com/seu-usuario/hive-hub.git news-system
cd news-system

# Executar setup automático
bash scripts/setup-production.sh
```

### 3. Configurar Variáveis de Ambiente

```bash
cd /opt/news-system/news-backend
nano .env
```

Adicionar:

```env
DEEPSEEK_API_KEY=sua-chave-aqui
REGISTRY_PATH=/opt/news-system/articles_registry.json
DOWNLOAD_DIR=/opt/news-system/downloads
OUTPUT_DIR=/opt/news-system/output
```

### 4. Compilar e Build

```bash
# Backend
cd /opt/news-system/news-backend
cargo build --release

# Frontend
cd /opt/news-system/apps/frontend-next/AIResearch
npm install
npm run build
```

### 5. Configurar Nginx

```bash
sudo nano /etc/nginx/sites-available/airesearch.news
```

Ver configuração completa em `docs/DEPLOY_HOSTINGER.md`

### 6. Iniciar Pipeline

```bash
sudo systemctl start news-pipeline
sudo systemctl enable news-pipeline
```

### 7. Verificar

```bash
# Status
sudo systemctl status news-pipeline

# Logs
sudo journalctl -u news-pipeline -f
```

## 🔄 Atualizações Futuras

Depois do setup inicial, para atualizar o código:

```bash
cd /opt/news-system
./deploy.sh
```

Isso irá:
- ✅ Parar pipeline
- ✅ Fazer backup
- ✅ Pull do Git
- ✅ Rebuild backend e frontend
- ✅ Reiniciar pipeline

## ➕ Adicionar Novos Collectors (APIs)

1. **Criar collector:**
   ```bash
   cd news-backend/src/collectors
   cp template_collector.rs meu_novo_collector.rs
   ```

2. **Implementar métodos** (ver `src/collectors/README.md`)

3. **Registrar em `mod.rs`**

4. **Adicionar função em `main.rs`**

5. **Deploy:**
   ```bash
   cd /opt/news-system
   ./deploy.sh
   ```

**Documentação completa**: `docs/DEPLOY_HOSTINGER.md`














