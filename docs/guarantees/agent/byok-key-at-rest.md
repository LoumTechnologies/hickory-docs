# A Stored Provider Key Is Sealed At Rest And Never Readable Back

Given an account that stores its own provider API key, when the key is
written, then it is encrypted with the deployment's key-encryption key before
it reaches the database, bound to the `(account, provider)` row it belongs to,
and no API response ever returns it — not even to the account that saved it.

Three consequences, each of which is the point of a separate mechanism:

1. **A database dump is not a bag of API keys.** The sealing key lives in the
   process environment (`KEY_ENCRYPTION_KEY`), never in Postgres, so a backup,
   a replica, or a table pasted into a support ticket carries only ciphertext.
2. **A ciphertext moved to another row does not open.** `user_id:provider` is
   the AEAD's associated data. Copying Alice's row onto Bob's account fails to
   decrypt rather than handing Bob a working credential — the failure mode
   that would otherwise be invisible, because the agent would run normally and
   only the billing would be wrong.
3. **The API is write-only for key material.** `last4` is the only plaintext
   fragment kept, so a stolen session token or an XSS can learn *which* key is
   installed but cannot exfiltrate it. The user already has the value; they
   got it from the vendor.

A key sealed under a key-encryption key the deployment no longer holds is
reported as such, naming the variable and telling the account to re-enter the
key — rather than failing deep inside a run as an unexplained provider error.

## Boundary

The server necessarily sees the key in plaintext: it makes the provider
request. "Never stored in the clear" is the claim; "never seen" is not, and
this document says so rather than letting a marketing sentence imply
otherwise. Rotation is *possible* (every row records its `key_version`) but no
rotation path is implemented — changing the key today means every account
re-enters its key, which is what the error message tells them to do.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/server/src/keyvault.rs` — XChaCha20-Poly1305, 32-byte key
  from base64, fresh 24-byte nonce per write, `user_id:provider` as associated
  data, `Debug` manually redacted so an enclosing struct cannot log the key.
  `apps/server/migrations/0007_user_llm_keys.sql` stores `ciphertext`, `nonce`,
  `key_version`, and `last4` only. `apps/server/src/routes/llm_keys.rs` has no
  handler that returns key material; `StoredKey` (in `byok.rs`) has no field
  that could carry it.
- Test coverage: `keyvault.rs` unit tests —
  `a_ciphertext_does_not_open_for_a_different_row` (both the other-account and
  the other-provider case), `a_different_deployment_key_cannot_open_it`,
  `every_write_uses_a_fresh_nonce`, `a_malformed_configuration_names_the_variable`.
  `apps/server/tests/integration.rs::a_stored_key_is_sealed_write_only_and_needs_no_choosing`
  reads the row straight out of Postgres and asserts the plaintext is absent
  from `ciphertext`, then asserts the listing response does not contain it
  either.
