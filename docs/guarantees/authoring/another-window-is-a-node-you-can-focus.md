# Another Window Is A Node You Can Focus

Given two live Hickory Docs desktop processes with different folders, when one
workspace tree shows the other's window node and that node is activated, then
the other native window is unminimized, raised, and focused; stale registrations
are not shown; and a browser-only `hick up` process does not pretend it has a
window to focus.

---

Last LLM verification:

- Date: 2026-09-14
- Reviewer: Codex (GPT-5)
- Result: not verified
- Evidence: `crates/hickory-cli/src/serve/shell.rs` currently exposes open,
  pick-folder, and close powers but no focus power or process registry. Desired
  local-IPC behavior is specified in `docs/specs/freeform/the-workspace-tree.md`.
- Test coverage: none yet. The implementation needs registry lifecycle tests,
  stale-process cleanup, per-window title identity, focus routing, and an honest
  no-shell response.
