# A Provider Key Entered in Settings Stays on This Machine

Given the desktop app, when the user enters an LLM provider API key in the
Settings page, then the key is written to one JSON file under the app's own
config directory (`llm-keys.json`), owner-only (`0600`) on Unix, read by this
process alone, sent to exactly one place — the provider the user chose, on
that provider's own API — and never logged, echoed back over the local API,
or transmitted anywhere else. There is no telemetry to carry it and no server
of ours for it to reach; those do not exist in this product
(`docs/specs/freeform/local-only.md`).

Four mechanisms, each doing one part:

1. **Private at rest.** `KeyStore::save` creates the file with mode `0600`
   at open time (never a window of world-readable existence), writes
   atomically via rename, and re-restricts a leftover temp file. Windows
   relies on the per-user profile directory's default ACLs.
2. **Never logged.** `KeyStore`'s `Debug` impl is hand-written to print
   masked fragments, so a `{:?}` of any enclosing state struct — the routine
   way keys leak into logs — cannot betray it.
3. **Write-only over the local API.** `GET /api/settings/keys` answers
   `configured` plus a masked fragment of at most the first 4 and last 2
   characters (nothing at all for short keys); no route returns key
   material, and `PUT` errors name providers and paths, never values.
4. **Stored key first, environment second.** The agent route resolves
   through the store before the environment, so a key entered in Settings
   works on the next turn with no restart and no exported variable — and an
   empty store degrades byte-for-byte to the env-only behavior the CLI
   keeps.

## Boundary

The process necessarily holds the key in plaintext — it makes the provider
request — and the file itself is plaintext on the user's own disk, protected
by file permissions rather than encryption: there is no server-held sealing
key in a local-only product, and an encryption key stored beside the
ciphertext would be a lock taped to its own key. "Stays on this machine and
is shown to no one" is the claim; "unreadable to the machine's owner or
root" is not. The loopback API serves the masked listing without
authentication, like every other route of the local server — the one
principal is the machine's owner (see `serve/mod.rs`).

---

Last LLM verification:
- Date: 2026-08-16
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/key_store.rs` — `KeyStore::save`
  (0600 via `OpenOptions::mode` at creation plus `set_permissions` for a
  reused temp file, write-then-rename), the redacting `Debug` impl, and
  `masked_key` (4+2 max, `…` alone under 12 chars). Precedence:
  `crates/hickory-agent/src/provider.rs::resolve_selector_with_store` /
  `client_for_with_store` reuse the env-only internals with the store
  consulted first. Routes: `crates/hickory-cli/src/serve/api.rs::
  {get,put}_settings_keys` (no handler returns key material; PUT stages on a
  copy, persists, then swaps the live store). Wiring:
  `crates/hickory-cli/src/serve/mod.rs::{ServeOptions::key_store_path,
  KeySettings, prepare}`; the desktop app passes
  `<app config dir>/llm-keys.json` in
  `apps/desktop/src-tauri/src/server.rs::start`, threaded from `launch` in
  `apps/desktop/src-tauri/src/lib.rs`. The CLI passes `None` and stays
  env-only.
- Test coverage: `crates/hickory-agent/src/key_store.rs` unit tests
  (round trip, 0600, redacted Debug, short-key masking, unknown-selector
  refusal that names the valid set and never echoes the key) and
  `crates/hickory-agent/src/provider.rs` tests
  (`a_single_stored_key_auto_selects_its_provider`,
  `a_stored_key_wins_over_the_environment_for_the_same_provider`,
  `a_stored_key_builds_a_client_with_no_environment_at_all`).
  `crates/hickory-cli/tests/serve_settings.rs` drives the routes over a real
  socket: PUT → masked GET (full key asserted absent from every response
  body), 0600 on disk, live pickup by the agent route (202, not 503, with
  the environment scrubbed and `ANTHROPIC_BASE_URL` pointed at a dead
  loopback port so nothing leaves the machine), clearing via `null`, and the
  all-or-nothing 400 for an unknown provider id.
- Caveat / instructions-module tension: `.instructions/config-and-environments.md`
  says secrets are "read from the environment" and "never write one to a
  file we create". The maintainer superseded that rule **for the desktop
  app** on 2026-08-16: an app launched from a dock has no environment to
  read, so its keys live in this store. The spirit of the rule — the key
  belongs to the user, stays on their machine, is never logged and goes only
  to the chosen provider — is exactly what this guarantee pins down; the
  CLI remains environment-only.
