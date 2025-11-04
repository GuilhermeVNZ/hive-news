# Comandos para Recompilar após as Mudanças

## 1. Recompilar news-backend.exe
```powershell
cd News-main/news-backend
cargo build --release --bin news-backend
```

## 2. Recompilar start.exe
```powershell
cd News-main
cargo build --release --bin start
```

## 3. Recompilar news-dashboard (frontend)
```powershell
cd News-main/news-dashboard
npm run build
```

## 4. Servers.exe - NÃO PRECISA RECOMPILAR
O `servers.exe` não foi modificado, então não precisa ser recompilado.

## Resumo das Mudanças

### news-backend
- ✅ Novos endpoints: `/api/system/collection/status`, `/api/system/loop/stats`, `/api/system/services/status`
- ✅ Modificações em `src/routes/system.rs` e `src/main.rs`

### start.rs
- ✅ Nova função `save_loop_stats()` para coletar estatísticas do loop
- ✅ Modificações em `start.rs`

### news-dashboard
- ✅ Dashboard atualizado com Collection Status, Services Status e Loop Statistics
- ✅ Modificações em `src/pages/Dashboard.tsx`

