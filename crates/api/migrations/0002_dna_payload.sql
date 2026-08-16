-- File-level Digital DNA payload for refresh/reload without reconstructing chunks.

ALTER TABLE dna_fingerprints
    ADD COLUMN IF NOT EXISTS payload JSONB;
