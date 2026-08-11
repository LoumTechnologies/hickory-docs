# One Key Is Chosen Without Asking; A Second Key Creates A Question

Given an account with exactly one stored provider key, when it starts an agent
session, then that key is used and the account is never asked to confirm it.

Given an account with more than one stored key and no selection, when it
starts a session, then the run stops with a message naming the stored
providers and where to choose between them — not with a guess, and not with a
silent pick of the first row.

A selection that names a provider whose key has since been deleted is treated
as absent rather than as an error: deleting a key is not an implicit request
to break the agent. If one key remains, that key is used.

## Why "never asked" is a guarantee and not a UI detail

A confirmation whose answer is forced is not a choice; it is a keystroke tax
on the common case. With one key installed there is nothing to decide, so the
product decides. The question appears exactly when the account has created
ambiguity — and then it is answerable, because the message lists what the
alternatives are.

The same rule holds in the CLI, where a provider is chosen by
`HICKORY_LLM_PROVIDER`/`--provider` and the key comes from that provider's
environment variable.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/server/src/byok.rs::select_provider` — a stored preference
  wins only when it resolves to an existing row; otherwise a lone row is
  selected and anything else selects nothing. `resolve_agent_credential`
  turns "nothing selected, several stored" into a 400 that lists the stored
  providers. `apps/web/src/views/SettingsView.tsx` renders a "Use this" button
  only when there is more than one key, and marks the active row.
- Test coverage: `byok.rs` unit tests —
  `one_key_is_selected_without_being_asked_about`,
  `several_keys_without_a_preference_select_nothing`,
  `a_preference_wins_and_a_stale_one_is_ignored`, `no_keys_select_nothing`.
  `apps/server/tests/integration.rs::a_second_key_creates_a_choice_and_says_so`
  drives it through the API, including selecting a provider with no stored key.
  `apps/web/src/views/SettingsView.test.tsx` covers the UI side: no "Use this"
  with one key, a prompt and two buttons with two.
