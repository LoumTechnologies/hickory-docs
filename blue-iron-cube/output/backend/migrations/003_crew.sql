CREATE TABLE IF NOT EXISTS crew_members (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    hourly_rate REAL NOT NULL,
    phone TEXT,
    certifications TEXT,
    hire_date TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS time_entries (
    id TEXT PRIMARY KEY,
    crew_member_id TEXT NOT NULL REFERENCES crew_members(id),
    job_id TEXT NOT NULL,
    date TEXT NOT NULL,
    hours_regular REAL NOT NULL,
    hours_overtime REAL NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_time_entries_crew ON time_entries(crew_member_id);
CREATE INDEX idx_time_entries_job ON time_entries(job_id);
CREATE INDEX idx_time_entries_date ON time_entries(date);
