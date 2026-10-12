# Finalization and incorporation check distinct equalities

Given captured starting tree A and completed tree B, when the shared finalization
service reconstructs a textual response, then replay of that response over A
must equal B. An incomplete capture or unresolved concurrent write refuses
finalization. Dirty starting bytes are part of A. Abandoned intermediate writes
are not emitted. Separately, incorporation validates candidate Markdown from an
empty tree against B. It pins destination documents and B for a later recheck;
changed destinations require a new candidate. Neither service writes code again.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for the portable lifecycle service; harness wiring is pending.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, distinct
  transition and maintained-source artifacts, dirty A and concurrent destination guards.
- Evidence: `hick-lang/src/project/reconstruct.rs::{AgentProjectCapture,transition_response,IncorporationCandidate}`.
- Witnesses and challenges: `hick-lang/tests/project.rs` checks dirty A, unchanged
  binary assets, an added empty file, unresolved concurrency, incomplete capture,
  fresh-tree incorporation and changed document/code destinations.
- Assumptions and exclusions: textual transition reconstruction explicitly refuses
  binary/metadata changes; maintained source supports binary/modes. Explanatory
  prose is supplied by a caller and is not semantically verified. Concurrent-write
  detection and actual capture must be supplied by harnesses; built-in/Python/ACP
  turn lifecycles and the AI incorporation review action are not yet wired to this
  service, so this does not claim successful automatic session finalization.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; lifecycle/evaluator edits invalidate this review.
- Human judgment: not recorded.

## Continuation review (2026-10-12)

The built-in, Python and ACP turn paths now call `serve/agent_projects.rs` for
actual A/B capture. `project_incorporation.rs` requests maintained source from
the selected model and uses the existing review queue. Both proposed and
reviewed candidates must produce B; destination documents and completed code are
rechecked before `project_publish.rs` writes source. Deletions retire live rooms
so delayed persistence cannot recreate removed source files. This supersedes the
pending harness/review integration exclusions above.

`just test-literate-agent-projects` passed four real HTTP flows, ten existing
agent-edit-session tests and seven collaboration tests. The HTTP flows exercise
a real ACP subprocess with abandoned and final native writes. One prior ACP
initialization timeout and the subsequent successful rerun are both recorded in
`comparison-integration-evidence.md`. Models are scripted witnesses; paid-model
explanation quality and authenticated authorship are not established. Changed
binary/metadata transition responses still refuse by name. Current hashes and
remaining limits are in that evidence file; human judgment is not recorded.
