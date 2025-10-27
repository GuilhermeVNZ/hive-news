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


