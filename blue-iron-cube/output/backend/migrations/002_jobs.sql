CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    status TEXT NOT NULL DEFAULT 'lead',

    -- Site characteristics (cost drivers)
    site_address TEXT NOT NULL,
    soil_type TEXT NOT NULL DEFAULT 'mixed',
    yard_grade TEXT NOT NULL DEFAULT 'flat',
    site_access TEXT NOT NULL DEFAULT 'easy',
    site_notes TEXT,

    -- Scope
    linear_feet_planned REAL,
    linear_feet_actual REAL,
    trench_depth_inches REAL,
    includes_sump_pump BOOLEAN NOT NULL DEFAULT 0,
    includes_downspout_tie_in BOOLEAN NOT NULL DEFAULT 0,
    number_of_catch_basins INTEGER NOT NULL DEFAULT 0,

    -- Schedule
    scheduled_date TEXT,
    start_date TEXT,
    end_date TEXT,
    estimated_crew_hours REAL,
    actual_crew_hours REAL,

    -- Conditions
    weather_conditions TEXT,
    ground_was_wet BOOLEAN,

    -- Completion tracking
    completion_pct INTEGER NOT NULL DEFAULT 0,
    phase_assessment BOOLEAN NOT NULL DEFAULT 0,
    phase_excavation BOOLEAN NOT NULL DEFAULT 0,
    phase_pipe_install BOOLEAN NOT NULL DEFAULT 0,
    phase_backfill BOOLEAN NOT NULL DEFAULT 0,
    phase_restoration BOOLEAN NOT NULL DEFAULT 0,
    phase_walkthrough BOOLEAN NOT NULL DEFAULT 0,
    cancellation_reason TEXT,

    -- Financials
    quoted_price REAL,
    actual_revenue REAL,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS job_line_items (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    category TEXT NOT NULL CHECK(category IN ('material', 'labor', 'equipment', 'overhead', 'processing_fee')),
    description TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit TEXT NOT NULL,
    unit_cost REAL NOT NULL,
    total_cost REAL NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_jobs_customer ON jobs(customer_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_scheduled ON jobs(scheduled_date);
CREATE INDEX idx_jobs_completion ON jobs(completion_pct);
CREATE INDEX idx_line_items_job ON job_line_items(job_id);
CREATE INDEX idx_line_items_category ON job_line_items(category);
