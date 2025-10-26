-- Hive-News Database Schema
-- Initial migration for all core tables

-- Articles table
CREATE TABLE IF NOT EXISTS articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    content TEXT,
    summary TEXT,
    author TEXT,
    published_at TIMESTAMP,
    source_url TEXT,
    tags TEXT[],
    language VARCHAR(10) DEFAULT 'en',
    status VARCHAR(20) DEFAULT 'draft',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Sources table
CREATE TABLE IF NOT EXISTS sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portal_id VARCHAR(100) NOT NULL,
    url TEXT NOT NULL,
    kind VARCHAR(20) NOT NULL,
    title TEXT,
    description TEXT,
    last_fetch TIMESTAMP,
    refresh_cron VARCHAR(100),
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Sources index
CREATE INDEX IF NOT EXISTS idx_sources_portal ON sources(portal_id);
CREATE INDEX IF NOT EXISTS idx_sources_url ON sources(url);

-- Articles indexes
CREATE INDEX IF NOT EXISTS idx_articles_status ON articles(status);
CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at);
CREATE INDEX IF NOT EXISTS idx_articles_language ON articles(language);

-- Add updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add updated_at triggers
CREATE TRIGGER update_articles_updated_at 
    BEFORE UPDATE ON articles 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sources_updated_at 
    BEFORE UPDATE ON sources 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

