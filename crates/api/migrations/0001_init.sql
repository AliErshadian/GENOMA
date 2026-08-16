-- Create GENOMA schema. Large binaries live in object storage, not PostgreSQL.

CREATE TABLE IF NOT EXISTS analyses (
    id UUID PRIMARY KEY,
    status TEXT NOT NULL,
    original_name TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    mime_type TEXT,
    storage_key TEXT NOT NULL,
    config JSONB NOT NULL,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS file_metadata (
    id UUID PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    original_name TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    mime_type TEXT,
    storage_key TEXT NOT NULL,
    storage_checksum TEXT
);

CREATE TABLE IF NOT EXISTS analysis_jobs (
    id UUID PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    stage TEXT NOT NULL,
    progress DOUBLE PRECISION NOT NULL DEFAULT 0,
    processed_bytes BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chunks (
    id UUID PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    chunk_index BIGINT NOT NULL,
    file_offset BIGINT NOT NULL,
    size_bytes INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chunk_features (
    id UUID PRIMARY KEY,
    chunk_id UUID NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    features JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS dna_fingerprints (
    id UUID PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    chunk_id UUID,
    raw JSONB NOT NULL,
    pi_derived JSONB NOT NULL,
    visual JSONB NOT NULL,
    pi_offset BIGINT NOT NULL,
    pi_wrapped BOOLEAN NOT NULL DEFAULT FALSE,
    generator_version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS anomalies (
    id UUID PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    chunk_index BIGINT NOT NULL,
    offset_bytes BIGINT NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    details JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS mutations (
    id UUID PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    baseline_analysis_id UUID REFERENCES analyses(id) ON DELETE SET NULL,
    chunk_index BIGINT NOT NULL,
    offset_bytes BIGINT NOT NULL,
    impact DOUBLE PRECISION NOT NULL,
    confidence DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS evolution_snapshots (
    id UUID PRIMARY KEY,
    series_id UUID NOT NULL,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    version_label TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chunks_analysis ON chunks(analysis_id);
CREATE INDEX IF NOT EXISTS idx_dna_analysis ON dna_fingerprints(analysis_id);
CREATE INDEX IF NOT EXISTS idx_anomalies_analysis ON anomalies(analysis_id);
CREATE INDEX IF NOT EXISTS idx_jobs_analysis ON analysis_jobs(analysis_id);
