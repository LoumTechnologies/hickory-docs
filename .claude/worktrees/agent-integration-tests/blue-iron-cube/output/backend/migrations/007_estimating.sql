CREATE TABLE IF NOT EXISTS regression_models (
    version INTEGER PRIMARY KEY,
    trained_at TEXT NOT NULL DEFAULT (datetime('now')),
    sample_size INTEGER NOT NULL,
    r_squared REAL NOT NULL,
    mae REAL NOT NULL,
    weights TEXT NOT NULL,  -- JSON array of feature weights
    intercept REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS estimates (
    id TEXT PRIMARY KEY,
    customer_id TEXT,
    linear_feet REAL NOT NULL,
    trench_depth_inches REAL NOT NULL,
    soil_type TEXT NOT NULL,
    yard_grade TEXT NOT NULL,
    site_access TEXT NOT NULL,
    includes_sump_pump BOOLEAN NOT NULL DEFAULT 0,
    includes_downspout_tie_in BOOLEAN NOT NULL DEFAULT 0,
    number_of_catch_basins INTEGER NOT NULL DEFAULT 0,
    predicted_direct_cost REAL NOT NULL,
    predicted_overhead REAL NOT NULL,
    predicted_total_cost REAL NOT NULL,
    predicted_crew_hours REAL NOT NULL,
    desired_margin_pct REAL NOT NULL,
    quoted_price REAL NOT NULL,
    model_version INTEGER NOT NULL,
    confidence_note TEXT NOT NULL,
    sample_size INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
