-- Generated outputs & lineage (api.md v0.2): per-run persisted output files
-- with byte-precise provenance, so GET /api/docs/:id/outputs* never
-- re-executes the pipeline.

CREATE TABLE run_outputs (
    run_id UUID NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    doc_id UUID NOT NULL REFERENCES docs(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    language TEXT NOT NULL,
    content TEXT NOT NULL,
    -- api.md Provenance[] (byte ranges in `content` -> source-doc origins)
    provenance JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (run_id, path)
);
CREATE INDEX run_outputs_doc_idx ON run_outputs (doc_id);
