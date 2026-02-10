CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    method TEXT NOT NULL CHECK(method IN ('cash', 'check', 'credit_card', 'debit_card', 'ach', 'financing', 'other')),
    processing_fee_pct REAL NOT NULL DEFAULT 0,
    processing_fee_amount REAL NOT NULL DEFAULT 0,
    fee_recovered BOOLEAN NOT NULL DEFAULT 0,
    reference TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS refunds (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id),
    payment_id TEXT REFERENCES payments(id),
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    reason TEXT NOT NULL,
    method TEXT NOT NULL,
    processor_refunded_fee BOOLEAN NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_payments_job ON payments(job_id);
CREATE INDEX idx_payments_method ON payments(method);
CREATE INDEX idx_payments_date ON payments(date);
CREATE INDEX idx_refunds_job ON refunds(job_id);
CREATE INDEX idx_refunds_payment ON refunds(payment_id);
