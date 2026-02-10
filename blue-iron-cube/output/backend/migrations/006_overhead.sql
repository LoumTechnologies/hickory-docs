CREATE TABLE IF NOT EXISTS overhead_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    frequency TEXT NOT NULL CHECK(frequency IN ('monthly', 'annual', 'per_job', 'one_time')),
    monthly_budget REAL NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS overhead_entries (
    id TEXT PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES overhead_categories(id),
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_overhead_entries_date ON overhead_entries(date);
CREATE INDEX idx_overhead_entries_cat ON overhead_entries(category_id);

-- Seed common overhead categories for a drain installation business
INSERT INTO overhead_categories (id, name, frequency, monthly_budget, notes) VALUES
    ('ohcat-insurance', 'General Liability Insurance', 'monthly', 500, 'GL insurance premium'),
    ('ohcat-vehicle-ins', 'Vehicle Insurance', 'monthly', 300, 'Commercial auto policy'),
    ('ohcat-workers-comp', 'Workers Compensation', 'monthly', 400, 'Workers comp premium'),
    ('ohcat-rent', 'Shop/Yard Rent', 'monthly', 800, 'Equipment storage yard'),
    ('ohcat-truck-payment', 'Truck Payments', 'monthly', 600, 'Fleet financing'),
    ('ohcat-phone', 'Phone/Internet', 'monthly', 150, 'Business phone and internet'),
    ('ohcat-accounting', 'Accounting/Bookkeeping', 'monthly', 200, 'Monthly bookkeeping service'),
    ('ohcat-marketing', 'Marketing/Advertising', 'monthly', 300, 'Google ads, yard signs, etc.'),
    ('ohcat-licenses', 'Licenses and Permits', 'annual', 50, 'Business license, contractor license'),
    ('ohcat-software', 'Software/Subscriptions', 'monthly', 100, 'Business software');
