CREATE TABLE IF NOT EXISTS evolution_series (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evolution_series ON evolution_snapshots(series_id, created_at);
