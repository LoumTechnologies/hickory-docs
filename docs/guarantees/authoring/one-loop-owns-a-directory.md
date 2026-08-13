# One Loop Owns A Directory, And Never Reacts To Itself

Given `hick up` running over a folder, when the loop writes the files it is
watching, then it does not treat those writes as user edits; and when a second
`hick up` is started on the same folder, then it refuses to start and says
which lock it is talking about.

A loop that writes into the directory it watches is a feedback loop by
construction. Two such loops on one directory is worse: each one's writes look
like the user's edits to the other, and a woven file ping-pongs between them
until something diverges. Neither failure is one a user could diagnose from
the outside, so both are prevented rather than detected.

Two properties hold that up:

1. **An event is an echo when the file still holds the bytes we last wrote.**
   Not a timestamp, not a counter of expected writes — the content itself.
   This covers the ordinary write, the restore after a refused edit, and any
   write the loop makes while a burst of events is still draining, without
   any of them needing to know about the others.
2. **The lock is held for the life of the process** on
   `.hick-cache/up.lock`, and is an OS advisory lock rather than a
   file-exists check, so a loop killed with `SIGKILL` releases it and the next
   one starts cleanly instead of hitting a stale lockfile.

Two smaller things follow from the same concern. The loop skips a write whose
bytes are already on disk, because rewriting a file an editor has open makes
it report an external modification and, in some editors, drop the undo
history. And events are debounced before being acted on, because no editor
saves a file in one write — vim writes a backup, truncates, writes, and
renames; VS Code writes a sibling and renames over the target — and reacting
to the first event in that sequence reads a half-written file.

## Boundary

The lock stops another `hick up`. It does not stop `hick run`, `hick serve`,
or a text editor, none of which ask for it. Running `hick serve` over the same
directory as `hick up` is untested and currently unsound: both write the
`.hick` file, and only one of them knows about echoes.

The lock covers the directory being watched, not the documents. Two loops on
two overlapping directory trees are not prevented.

---

Last LLM verification:
- Date: 2026-08-12
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/up/state.rs::WovenState::is_echo` compares
  the file's current bytes against `OutputState::content`, which is set by
  every path that writes (`write_output`, and `restore_output` which leaves it
  unchanged on purpose). `write_output` returns early without writing when the
  bytes already match. `crates/hickory-cli/src/up/mod.rs::DirectoryLock`
  acquires `fs2::FileExt::try_lock_exclusive` on `.hick-cache/up.lock` and
  holds the handle in a value that lives as long as `run`; the error names the
  directory, why two loops are a problem, and the lock path. `handle_batch`
  drops every echoing path before classifying the rest, and `is_noise` filters
  editor scratch files (`.swp`, `.swx`, `~`, `4913`, `.#*`) plus `.git`,
  `.hick-cache`, `node_modules`, and `target`. Events are collected until the
  directory is quiet for `WEAVE_DEBOUNCE` (120ms), or `RUN_DEBOUNCE` (500ms)
  under `--run`.
- Test coverage: `crates/hickory-cli/tests/up_loop.rs` —
  `a_second_loop_on_the_same_directory_refuses_to_start` asserts the exit
  status and both halves of the message. The echo property is covered
  indirectly but decisively by every other test in that file: each one waits
  on a settled filesystem after a weave, which a loop reacting to its own
  writes would never reach.
- Caveat requiring review: the `hick serve` interaction named under Boundary
  is a known gap, not a covered case.
