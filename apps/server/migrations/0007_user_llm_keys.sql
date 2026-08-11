-- Bring-your-own-key: the provider credentials an account runs the agent on.
--
-- `plans.json` sells `"agent": "byo_key"` on Open and Pro. Until this table
-- existed there was nowhere for that key to live, so those plans either ran
-- on the deployment's credential (spending the operator's money on a feature
-- sold as the user's own) or answered 503. One row per (account, provider):
-- an account may keep keys for several vendors and switch between them
-- without re-pasting.
--
-- The key itself is never stored in the clear. `ciphertext` is
-- XChaCha20-Poly1305 output under the deployment's key-encryption key
-- (`KEY_ENCRYPTION_KEY`), with `nonce` unique per write and the
-- `user_id:provider` pair bound in as associated data — so a ciphertext
-- moved to another row, or another user, fails to open rather than
-- decrypting into someone else's account.
--
-- `last4` is the only plaintext fragment kept, so the UI can show *which*
-- key is installed without being able to reconstruct it.
CREATE TABLE user_llm_keys (
    user_id      uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- `anthropic` | `openai` | `deepseek` | `grok` (ProviderSelection).
    provider     text NOT NULL,
    ciphertext   bytea NOT NULL,
    nonce        bytea NOT NULL,
    -- Which key-encryption key sealed this row. Bumped by a future rotation
    -- so re-encrypted and not-yet-re-encrypted rows can coexist.
    key_version  integer NOT NULL DEFAULT 1,
    -- Last four characters of the plaintext key, for display only.
    last4        text NOT NULL,
    -- Model override for this provider; NULL uses the provider default.
    model        text,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz,
    PRIMARY KEY (user_id, provider)
);

-- Which stored key the agent should use, when more than one is installed.
--
-- NULL is the normal state and is not a missing setting: with exactly one
-- key stored there is nothing to choose, and asking would be ceremony. It
-- only has to be set once a second provider appears.
ALTER TABLE users ADD COLUMN preferred_llm_provider text;
