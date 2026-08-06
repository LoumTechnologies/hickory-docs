-- Hickory Docs server schema (v0).

CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    plan_key TEXT NOT NULL DEFAULT 'open',
    price_key TEXT,
    stripe_customer_id TEXT,
    -- 'active' | 'past_due' (dunning: restrict, never delete)
    billing_status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE projects (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    visibility TEXT NOT NULL CHECK (visibility IN ('public', 'private')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE docs (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, path)
);

CREATE TABLE runs (
    id UUID PRIMARY KEY,
    doc_id UUID NOT NULL REFERENCES docs(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    kind TEXT NOT NULL CHECK (kind IN ('run', 'check', 'agent')),
    status TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    wall_ms BIGINT,
    -- api.md run shape: [{exec_id, status, transcript}]
    blocks JSONB NOT NULL DEFAULT '[]',
    error TEXT
);
CREATE INDEX runs_doc_idx ON runs (doc_id, started_at DESC);

-- Execution minutes metered per account per month (sum of run wall time).
CREATE TABLE usage_ms (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    month TEXT NOT NULL, -- 'YYYY-MM'
    wall_ms BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, month)
);

-- Second-layer webhook event log (skip event ids already seen). The
-- fulfillment guarantee itself is the UNIQUE constraint on subscriptions.
CREATE TABLE stripe_events (
    id TEXT PRIMARY KEY,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Fulfillment claim: one row per Stripe subscription. Idempotency is
-- INSERT ... ON CONFLICT DO NOTHING RETURNING against this primary key.
CREATE TABLE subscriptions (
    stripe_subscription_id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    price_key TEXT NOT NULL,
    plan_key TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
