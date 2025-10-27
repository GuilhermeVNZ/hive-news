-- Filtered Documents Table
CREATE TABLE IF NOT EXISTS filtered_documents (
    id SERIAL PRIMARY KEY,
    raw_doc_id INT REFERENCES raw_documents(id),
    source_type VARCHAR(20),        -- 'scientific' ou 'blog'
    doi_valid_ratio FLOAT,
    has_experiments BOOLEAN,
    author_valid_ratio FLOAT,
    fake_penalty FLOAT,
    score FLOAT,
    approved BOOLEAN,
    category VARCHAR(50),
    filtered_at TIMESTAMP DEFAULT NOW(),
    skipped_reason VARCHAR(100)     -- 'non-scientific-source' se pulado
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_filtered_approved ON filtered_documents(approved);
CREATE INDEX IF NOT EXISTS idx_filtered_category ON filtered_documents(category);
CREATE INDEX IF NOT EXISTS idx_filtered_source_type ON filtered_documents(source_type);

-- Generated Content Table
CREATE TABLE IF NOT EXISTS generated_content (
    id SERIAL PRIMARY KEY,
    filtered_doc_id INT REFERENCES filtered_documents(id),
    article_path TEXT,
    linkedin_path TEXT,
    x_path TEXT,
    shorts_script_path TEXT,
    metadata_path TEXT,
    images_extracted TEXT[],
    recommended_figure TEXT,  -- Best figure for article header
    original_tokens INT,      -- Tokens before compression
    compressed_tokens INT,    -- Tokens after compression
    compression_ratio FLOAT,  -- Savings percentage (0.0 to 1.0)
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_generated_filtered_doc ON generated_content(filtered_doc_id);
CREATE INDEX IF NOT EXISTS idx_generated_created_at ON generated_content(created_at);

