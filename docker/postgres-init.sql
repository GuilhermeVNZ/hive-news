-- Script de inicialização seguro para PostgreSQL
-- Cria um usuário de aplicação SEM privilégios SUPERUSER
-- Isso previne execução de comandos como COPY ... FROM PROGRAM

-- Criar usuário da aplicação (não-superuser)
CREATE USER news_app WITH PASSWORD 'changeme123' NOSUPERUSER NOCREATEDB NOCREATEROLE;

-- Dar permissões necessárias apenas no banco news_system
GRANT CONNECT ON DATABASE news_system TO news_app;
GRANT USAGE ON SCHEMA public TO news_app;
GRANT CREATE ON SCHEMA public TO news_app;

-- Dar permissões nas tabelas existentes (será aplicado após migrações)
-- As migrações devem ser executadas com o usuário postgres
-- Depois, executar: GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO news_app;

-- Revogar permissões perigosas (já não tem por não ser superuser, mas para garantir)
-- REVOKE ALL ON FUNCTION pg_read_file(text) FROM news_app;
-- REVOKE ALL ON FUNCTION pg_read_file(text, bigint, bigint) FROM news_app;
-- REVOKE ALL ON FUNCTION pg_read_file(text, bigint, bigint, boolean) FROM news_app;

-- Desabilitar COPY FROM PROGRAM para todos os usuários (requer configuração no postgresql.conf)
-- Necessário adicionar no postgresql.conf:
-- allow_system_table_mods = off

-- Log de auditoria
ALTER DATABASE news_system SET log_connections = on;
ALTER DATABASE news_system SET log_disconnections = on;

