CREATE TABLE IF NOT EXISTS leads (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    status TEXT NOT NULL DEFAULT 'new_lead'
        CHECK(status IN ('new_lead','contacted','site_visit_scheduled','site_visit_completed',
                         'estimate_sent','negotiating','won','lost')),
    source TEXT NOT NULL,
    estimated_value REAL,
    job_id TEXT,
    assigned_to TEXT,
    lost_reason TEXT,
    lost_to_competitor TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS interactions (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    lead_id TEXT REFERENCES leads(id),
    date TEXT NOT NULL,
    interaction_type TEXT NOT NULL CHECK(interaction_type IN ('call','email','text','in_person','site_visit','voicemail','social_media')),
    direction TEXT NOT NULL CHECK(direction IN ('inbound','outbound')),
    summary TEXT NOT NULL,
    duration_minutes INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS follow_ups (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    lead_id TEXT REFERENCES leads(id),
    due_date TEXT NOT NULL,
    action TEXT NOT NULL,
    notes TEXT,
    completed BOOLEAN NOT NULL DEFAULT 0,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_leads_customer ON leads(customer_id);
CREATE INDEX idx_leads_status ON leads(status);
CREATE INDEX idx_leads_source ON leads(source);
CREATE INDEX idx_interactions_customer ON interactions(customer_id);
CREATE INDEX idx_interactions_lead ON interactions(lead_id);
CREATE INDEX idx_interactions_date ON interactions(date);
CREATE INDEX idx_follow_ups_due ON follow_ups(due_date);
CREATE INDEX idx_follow_ups_incomplete ON follow_ups(completed) WHERE completed = 0;
