# Candidate revision checks never borrow working bytes

Given an exact Git commit/tree or complete stage-zero index, when its literate
project is verified, then both Markdown and expected outputs come from the
captured revision object identities. Unstaged files and ancestor documents
cannot complete a partial candidate. Unmerged index stages and unsupported Git
entry kinds fail. Persisted CRDT input decodes the supplied immutable updates;
subsequent live edits do not change the captured tree.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for Git, Mercurial and supplied persisted CRDT snapshots.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, exact
  commit/index verification, optional revision sources and frozen CRDT state.
- Evidence: `hickory-cli/src/project_revision.rs::{git,persisted_crdt}` and
  `project_command.rs`; Git and Mercurial scope the captured tree to the selected
  project directory. Git reads captured object IDs without checkout or scratch
  materialization. Generation uses the same VCS-free evaluator afterwards.
- Witnesses and challenges: `hickory-cli/tests/project_revision.rs` stages only
  changed Markdown while keeping matching outputs unstaged, then reverses which
  side is unstaged; it also mutates a live CRDT after freezing its update.
- Assumptions and exclusions: supplied CRDT updates contain the existing `source`
  text root. Multi-document capture is a caller's frozen set, not a claim of an
  atomic distributed room transaction. The real Mercurial provider test freezes
  a committed revision despite changed working Markdown and outputs. Mercurial
  is an external test/provider CLI, not a dependency linked into the product.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; revision-adapter edits invalidate this review.
- Human judgment: not recorded.

## Continuation review (2026-10-12)

`project_index.rs::verify_with_coverage` discovers entry declarations and their
closures in the same frozen index as expected outputs. The hook from
`init_hook.sh` invokes this path even when only an output changed. It does not
borrow malformed or missing unstaged sources. Ordinary notes outside a closure
are not silently excluded from source inventory; a declared exclusion makes
the standalone-note fixture explicit.

The current six revision tests and the separate real Mercurial provider test
passed. Exact frozen-index and git-free-folder witnesses remain separate from
the native tool materialization path. Executed commands, assumptions and
dependency hashes are in `comparison-integration-evidence.md`. Human judgment
is not recorded.
