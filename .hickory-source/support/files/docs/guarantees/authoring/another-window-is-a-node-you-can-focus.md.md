<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/another-window-is-a-node-you-can-focus.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `15588c6965b0e20cc81a5dc5a76df583396d7320760749b5e8f0382868fee936`.

<book:entry path="docs/guarantees/authoring/another-window-is-a-node-you-can-focus.md">
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
</book:entry>

</book:doc>
