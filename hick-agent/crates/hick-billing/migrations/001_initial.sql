-- Initial schema for hick-billing.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    github_id        BIGINT      UNIQUE NOT NULL,
    github_login     TEXT        NOT NULL,
    github_email     TEXT,
    stripe_customer_id TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One row per LLM API call reported by an agent.
CREATE TABLE IF NOT EXISTS usage_events (
    id                     BIGSERIAL   PRIMARY KEY,
    user_id                UUID        REFERENCES users(id),
    machine_id             TEXT        NOT NULL,
    call_timestamp         TIMESTAMPTZ NOT NULL,
    recorded_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    provider               TEXT        NOT NULL,
    model                  TEXT        NOT NULL,
    input_tokens           BIGINT      NOT NULL DEFAULT 0,
    output_tokens          BIGINT      NOT NULL DEFAULT 0,
    cache_creation_tokens  BIGINT      NOT NULL DEFAULT 0,
    cache_read_tokens      BIGINT      NOT NULL DEFAULT 0,
    -- Estimated provider cost in USD and our 1% fee, computed at insert time.
    estimated_cost_usd     NUMERIC(12, 8) NOT NULL DEFAULT 0,
    fee_usd                NUMERIC(12, 8) NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS usage_events_user_id_idx    ON usage_events(user_id);
CREATE INDEX IF NOT EXISTS usage_events_timestamp_idx  ON usage_events(call_timestamp);
