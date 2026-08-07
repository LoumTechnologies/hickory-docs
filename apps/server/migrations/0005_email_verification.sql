-- Email verification and password reset.
--
-- Until now an address was never proven: signup accepted anything containing
-- '@' and handed back a session. That is the wrong footing for team invites
-- (clearance would attach to an unproven identity) and it left no path back
-- from a forgotten password.

ALTER TABLE users
    ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT false;

-- Existing accounts predate verification. Grandfathering them in is the
-- honest choice: they were created when the product made no such claim, and
-- locking them out would punish people for our change.
UPDATE users SET email_verified = true;

-- One row per issued token. The token itself is NEVER stored — only its
-- SHA-256 — so a database read cannot be replayed as a verification or a
-- password reset, exactly as with password_hash.
CREATE TABLE email_tokens (
    token_hash TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- 'verify' | 'reset'. A verification link must not double as a password
    -- reset, so the purpose is checked on redemption.
    purpose TEXT NOT NULL CHECK (purpose IN ('verify', 'reset')),
    expires_at TIMESTAMPTZ NOT NULL,
    -- Set on redemption. Single use: a link in an inbox, a mail archive, or a
    -- forwarded message must not stay live.
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Redemption looks up by hash (the primary key). This index serves the other
-- direction: expiring a user's outstanding tokens when one is redeemed, and
-- rate-limiting how many are outstanding.
CREATE INDEX email_tokens_user_idx ON email_tokens (user_id, purpose, created_at DESC);
