-- Create raw_documents table for storing downloaded files metadata
CREATE TABLE IF NOT EXISTS raw_documents (
    id SERIAL PRIMARY KEY,
    portal_id INT REFERENCES pages_config(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    source_url TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size BIGINT,
    metadata JSONB DEFAULT '{}',
    downloaded_at TIMESTAMP DEFAULT NOW(),
    processed BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_raw_documents_portal_id ON raw_documents(portal_id);
CREATE INDEX IF NOT EXISTS idx_raw_documents_processed ON raw_documents(processed);
CREATE INDEX IF NOT EXISTS idx_raw_documents_downloaded_at ON raw_documents(downloaded_at);
CREATE INDEX IF NOT EXISTS idx_raw_documents_created_at ON raw_documents(created_at);


