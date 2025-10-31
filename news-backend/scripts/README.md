# Scripts de Sincronização

## sync-env-from-config (Binário Rust)

Sincroniza o arquivo `.env` a partir de `system_config.json`.

### Uso

```bash
# No diretório news-backend
cargo run --bin sync-env-from-config
```

Ou após compilar:

```bash
target/debug/sync-env-from-config.exe
```

### O que faz

1. Lê `system_config.json`
2. Extrai API keys dos writers configurados para cada site
3. Atualiza o arquivo `.env` com as chaves encontradas:
   - `DEEPSEEK_API_KEY` - se algum site usa provider "deepseek"
   - `OPENAI_API_KEY` - se algum site usa provider "openai"
   - `ANTHROPIC_API_KEY` - se algum site usa provider "anthropic"
4. Preserva outras variáveis já existentes no `.env`

### Quando executar

- Após atualizar configurações de Writer no frontend
- Manualmente quando precisar sincronizar `.env` com `system_config.json`
- Pode ser adicionado como hook git ou tarefa agendada se necessário

### Exemplo

```bash
# Sincronizar .env
cargo run --bin sync-env-from-config

# Verificar resultado
cat .env
```

### Localização

O binário está em `src/bin/sync_env_from_config.rs` e pode ser executado como:

- `cargo run --bin sync-env-from-config` (desenvolvimento)
- `target/debug/sync-env-from-config.exe` (após compilar em modo debug)
- `target/release/sync-env-from-config.exe` (após compilar em modo release)

