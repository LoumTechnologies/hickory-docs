CREATE TABLE IF NOT EXISTS procedures (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    estimated_duration_minutes INTEGER,
    required_roles TEXT,
    required_equipment TEXT,
    required_materials TEXT,
    safety_notes TEXT,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS procedure_steps (
    id TEXT PRIMARY KEY,
    procedure_id TEXT NOT NULL REFERENCES procedures(id),
    step_number INTEGER NOT NULL,
    instruction TEXT NOT NULL,
    is_checkpoint BOOLEAN NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX idx_procedures_category ON procedures(category);
CREATE INDEX idx_steps_procedure ON procedure_steps(procedure_id);
