# Agent-Authored Bytes Always Report Session, Turn, And A Git-Derived Author — Even When The Session Cannot Be Read

Given a byte of generated output whose provenance origin is
`SourceOrigin::Agent { session, turn }`, when a reader asks for its lineage,
then the report always names the session id and the turn, and names the author
by deriving it from `git` rather than from any stored field — and when the
session file is missing, unreadable, unparseable, or does not record that turn,
the report says so with the path it looked for and why it failed, instead of
erroring, panicking, or degrading the origin to `synthetic`.

Corollaries that are part of the guarantee:

- **No `author` field exists anywhere in the provenance data.** Authorship is
  composed at read time: lineage maps the byte to a document span and
  `git blame` on that span gives the commit author. An author baked in at
  promote time would be *asserted* by whoever ran promote; a git author is
  anchored in a commit and can be signed.
- **An uncommitted working tree is attributed to the current user.** `git blame`
  reports uncommitted lines as `Not Committed Yet`; those changes are the
  reader's own by definition, so the report names the repository's configured
  identity and says the span is not committed yet.
- **Missing `git`, a directory that is not a repository, an untracked file, and
  an absent document span each resolve to a stated outcome**, never an error.
- **Adding the variant cannot break stored provenance.** `SourceOrigin` is a
  serde-tagged enum; the `Agent` variant is additive, and its optional
  document-span fields default to absent so an origin serialized without them
  still deserializes.

Rendered shape:

```
foo.rs:42 ← session abc123 turn 7 · committed by … · reasoning not available to you
```

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified for the reporting path; **partially unverifiable in
  production** until issue #7 lands (see caveat).
- Evidence:
  - `crates/hick-flow/src/node.rs` — `SourceOrigin::Agent { session, turn,
    file, span }`, additive on the `#[serde(tag = "type")]` enum, with
    `#[serde(default)]` on both optional locating fields and no `author` field.
  - `crates/hickory-lineage/src/lib.rs` — `from_provenance_map` maps an agent
    origin to `Origin::Agent` unconditionally (it is the one origin kind that
    never falls through to `Origin::Synthetic`), carrying `doc_path`/`span`
    only when the recorded span is byte-precise. `Origin::agent()` exposes the
    session and turn; `Origin::location()`/`source()` expose the span when
    present.
  - `crates/hickory-cli/src/agent_lineage.rs` — `blame` (every git failure mode
    resolves to `Committed`, `Uncommitted`, or `Unknown { reason }`),
    `resolve_reasoning` (missing / unreadable / unparseable / turn-not-recorded
    all resolve to `Reasoning::Unavailable { reason }`), `describe`, and the
    `Display` impl producing the shape above. No function in the module returns
    `Result` or unwraps a fallible git or filesystem call.
  - `crates/hickory-cli/src/lib.rs` — `agent_lineage_report` assembles the
    per-output report; `crates/hickory-cli/src/main.rs` — `cmd_lineage` prints
    an `agent` row in the provenance table and the composed lines beneath it.
- Test coverage:
  - `crates/hick-flow/src/provenance.rs`
    (`agent_origin_round_trips_and_is_additive`) — serde round-trip with and
    without the optional span, plus an existing-variant regression check.
    Note: `hick-flow` now carries a `features = ["serde"]` self-dev-dependency,
    because before this change nothing in the workspace enabled the feature and
    every `#[cfg(feature = "serde")]` test compiled out silently.
  - `crates/hickory-lineage/src/lib.rs`
    (`agent_origin_keeps_session_and_turn_with_or_without_a_span`).
  - `crates/hickory-cli/src/agent_lineage.rs` unit tests — real temporary git
    repositories cover the committed, uncommitted, untracked, and
    not-a-repository cases; session tests cover missing, present, and
    turn-out-of-range.
  - `crates/hickory-cli/tests/agent_lineage.rs` — the composed end-to-end
    shape, including the target rendering.
- Caveat requiring later review: nothing *produces* an `Agent` origin yet. The
  `hick:agent` DAG vertex — a `build_dag` arm, an `ExecInfo` that describes an
  agent cell rather than a container and a command, and a cache key including
  the prompt and model — is issue #7. Every test here therefore constructs
  agent-origin spans synthetically. When #7 lands, re-verify that the spans it
  emits carry a byte-precise document span whenever the agent wrote through
  `edit_doc`, and that `hick lineage` on a real agent-authored output prints
  the shape above.
