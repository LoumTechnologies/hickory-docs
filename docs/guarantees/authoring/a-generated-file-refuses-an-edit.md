# Generated Text Refuses An Edit, And Says So Before You Type

Given `hick up` running over a folder, when a generated output file contains
no byte that maps back to a document, then that file is marked read-only on
disk for as long as the loop runs; and when an edit is forced into generated
text anyway, then the file is restored to the woven bytes, the document is
left untouched, and the refusal names the line, the reason, and the file to
edit instead.

Command output, transcripts, interpolated values, and the separators between
blocks have no source to carry an edit back to. The question is only *when*
the user finds out. A read-only file is a signal every editor already knows
how to show — vim's `[readonly]`, a padlock in VS Code — so the answer arrives
when the file is opened rather than after the typing.

Three properties hold that up:

1. **Read-only means "no byte of this file is editable", not "some byte is
   not".** A file mixing document text with generated text stays writable,
   because marking it read-only would block the edits that *are* legal. Within
   such a file, protection is per-range and decided on save.
2. **A refusal restores, it does not half-apply.** Either every edit in the
   save maps or none of them are applied, and the file goes back to the bytes
   the weave produced.
3. **The marking is a property of the running loop, not of the files.** It is
   cleared when the loop exits — however it exits — so `hick run`, `hick test`,
   git, and every other tool find ordinary files afterwards. `write_outputs`
   also clears it defensively, because a loop killed with `SIGKILL` never gets
   to run anything.

## Boundary

The read-only bit is advisory. It stops an accident, not a determined `:w!`,
which is why property 2 exists behind it.

On Unix the write bits are toggled and the rest of the mode is left alone;
unlocking restores the owner's write bit only. `Permissions::set_readonly`
would make the file world-writable, which is not something a tool should do to
a file in someone's working tree.

---

Last LLM verification:
- Date: 2026-08-12
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/up/state.rs` — `OutputState::editable`
  is true when any provenance entry carries a source span; `write_output`
  marks a file read-only exactly when it is false; `set_read_only` toggles
  `mode & !0o222` / `mode | 0o200` on Unix and defers to
  `Permissions::set_readonly` elsewhere; `release_read_only` clears every mark.
  `crates/hickory-cli/src/up/mod.rs::run` calls it on the way out of the loop,
  with `tokio::signal::ctrl_c` selected against the event channel so Ctrl-C
  reaches it. `restore_output` clears the mark, rewrites the woven bytes, and
  re-applies the mark. `crates/hickory-cli/src/lib.rs::clear_read_only` is
  called from `write_outputs` before each write.
  `crates/hickory-cli/src/up/reverse.rs::refusal_message` names the line, what
  kind of text it is, an excerpt of the refused bytes, and the document path.
- Test coverage: `crates/hickory-cli/tests/up_loop.rs` —
  `a_fully_generated_file_is_read_only_and_restores_a_forced_edit` asserts the
  read-only bit on the woven markdown, forces an edit through a `chmod`, and
  asserts the file returns byte-identical while the document stays untouched.
- Caveat requiring review: the `SIGKILL` case in property 3 rests on
  `write_outputs` clearing the bit, which is covered by reading the code —
  there is no test that kills a loop uncatchably and then runs `hick run`.
