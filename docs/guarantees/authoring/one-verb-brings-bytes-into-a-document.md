# One Verb Brings Bytes Into A Document

Given bytes a document did not write — a cell's output volume, a plain file,
a session, what a session carries forward, another harness's session log,
an inbox transcript — when they are to become a document's own, then the
command is `hick ingest`, with `--from` naming the source (`'#cell'`,
`file`, `session`, `carry`, `claude-code`; absent for the inbox), and every
source does the same three things: gives the bytes a home in a document,
records where they came from, and — where the bytes are generated — proves
the weave is unchanged before writing anything. `adopt`, `promote`, `carry`
and `import` no longer exist as commands.

They were five spellings of one act, and a person met five names for it.
Axis 2 of `docs/specs/freeform/three-axes.md`: the document either owns bytes
or points at them, and crossing that line is one verb. The distinct words
survive as the *reason* in the provenance record, which is what
`provenance-and-standing.md` wants said anyway.

## Boundary

The underlying modules are unchanged and keep their names — `adopt.rs`,
`promote.rs`, `carry.rs`, `claude_code.rs` — because the mechanisms differ
even though the verb does not. The app's "Make literate" button is
`POST /api/adopt` and still is: it is a door onto `--from file`, not a
second mechanism. `--from recording` is named and refused with a pointer,
until step 4 of the spec builds it.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/main.rs` — `IngestArgs` (`paths`,
  `--from`, `--into`, `--out`, `--force`, `--stdout`), `cmd_ingest` routing
  by source to `ingest_file`, `ingest_session`, `ingest_carry`,
  `ingest_claude_code` and `cmd_ingest_from_exec`; the four removed
  `Command` variants. `hick --help` lists none of the old names.
- Test coverage: `crates/hickory-cli/tests/ingest.rs` (inbox and `'#cell'`);
  the library-level tests behind each source
  (`adopt`, `agent_promote_e2e`, `carry`, `claude_code`) are unchanged.
  A CLI smoke of `hick ingest --from file` was run by hand on 2026-09-03.
- Caveats: no CLI-level test drives `--from session|carry|claude-code`
  through the binary; the routing is by review.
