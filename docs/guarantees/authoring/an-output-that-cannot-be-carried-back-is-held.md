# An Output That Cannot Be Carried Back Is Held, Never Undone

Given a running loop (`hick up`, or the app's own) and a generated file whose
bytes on disk changed by something other than the loop — a forced edit, a
`git checkout`, a `git pull`, the Git pane's Discard — when the change cannot
be carried back into the document (it touches generated text, or the
document moved under it), then the file is left exactly as it is on disk and
marked **held**: the loop says so by name and reason, the file tree shows the
mark, the generated pane shows the reason, and no later weave rewrites the
file until either the document produces exactly the held bytes or a person
asks for the file to be regenerated from the document.

It used to be restored: the loop put the file back to match the document, on
the argument that a silent fork is worse than a refused edit. The argument was
right about "silent" and wrong about the remedy. Restoring destroyed what a
person typed, and what git wrote, a second after it was written — a Discard
in the Git pane "worked" and the file stayed modified; a `git pull` that
updated a generated file was reverted before anyone saw it. The bytes on disk
came from somebody; the loop has nothing truer to put in their place. What
was silent is now loud, and what was undone now stays.

## Boundary

A held file is not an unresolved merge: nothing blocks. Editing the document
until it produces the held bytes lifts the hold; so does **Regenerate from
document**, which is the only path that overwrites held bytes and is taken
only when asked by name.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/up/state.rs` (`WovenState::held`,
  `hold_output`, the hold check in `write_output`, `restore_output` as the
  explicit way out); `crates/hickory-cli/src/up/mod.rs` `consume_output_save`
  holds on both refusal branches; `crates/hickory-cli/src/up/reverse.rs`
  `refusal_reason`; `crates/hickory-cli/src/serve/watch.rs` publishes the
  held set to `LocalState.held` and answers regenerate requests;
  `crates/hickory-cli/src/serve/api.rs` stamps `held` on tree nodes and
  serves `POST /api/outputs/regenerate`; `apps/web/src/shell/FolderTreePane.tsx`
  and `apps/web/src/components/OutputEditorPane.tsx` show it.
- Test coverage: `crates/hickory-cli/tests/up_loop.rs`
  (`a_fully_generated_file_is_read_only_and_a_forced_edit_is_held_not_undone`,
  including the hold lifting when the document catches up);
  `crates/hickory-cli/tests/round_trip.rs` (a `git checkout` of an output
  under the app's own loop stays on disk and is reported held).
