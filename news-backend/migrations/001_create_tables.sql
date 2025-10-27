-- Create pages_config table
CREATE TABLE IF NOT EXISTS pages_config (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    sources TEXT[] DEFAULT '{}',
    frequency_minutes INT NOT NULL DEFAULT 60,
    writing_style TEXT NOT NULL DEFAULT 'scientific',
    linked_accounts JSONB DEFAULT '{}',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Create users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create collection_logs table
CREATE TABLE IF NOT EXISTS collection_logs (
    id SERIAL PRIMARY KEY,
    page_id INT REFERENCES pages_config(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    articles_collected INT DEFAULT 0,
    duration_ms INT,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pages_name ON pages_config(name);
CREATE INDEX IF NOT EXISTS idx_pages_active ON pages_config(active);
CREATE INDEX IF NOT EXISTS idx_logs_page_id ON collection_logs(page_id);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON collection_logs(created_at);

