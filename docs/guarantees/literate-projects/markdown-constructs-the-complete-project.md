# The Markdown closure constructs the current project

Given entry Markdown declaring a project and its contributing Markdown paths,
when pure generation starts from an empty virtual tree, then construction uses
only that supplied closure. No ancestor, cache, generated code, sidecar or native
process supplies project bytes. Explicit predecessor identities order acts;
missing or cyclic predecessors and unordered transformations of the same path
fail. Local predecessor states use declared ancestors, while completed New
combines independent contributions; their naming does not select intermediate
project contents. Unsupported execution or workspace-dependent semantics fail by name.
The ordinary document run/weave path for a declared project uses this evaluator.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for the declared portable construction grammar.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, maintained
  source starts from empty, and milestone 1's temporarily incomplete fixture.
- Evidence: `hick-lang/src/project/{discovery,evaluate}.rs`; portable facades in
  `hick-lang/src/lib.rs` and `hick-lang-wasm/src/lib.rs`; pipeline seam in
  `hick-literate/src/project.rs`; native document adapter in
  `hickory-cli/src/project_documents.rs`.
- Witnesses and challenges: the two-file fixture in `hick-lang/tests/fixtures/`
  has a caller patch preceding the library patch. `hick-lang/tests/project.rs`
  checks folded/reorganized source equality and missing sources. The CLI
  `fresh_git_free_folder_runs_the_markdown_closure` test copies only Markdown,
  checks no generated file exists during evaluation, writes explicitly, and
  verifies exact outputs with no Git or cache.
- Assumptions and exclusions: source paths are relative to the declaring Markdown;
  closure traversal cannot escape. Top-level literal copy/paste is supported;
  conditionals, ambient/upstream contributions, cell recordings and execution are
  explicitly unsupported in this pure subset, rather than silently skipped.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; changed grammar/evaluator invalidates this review.
- Human judgment: not recorded.

## Continuation review (2026-10-12)

The portable evaluator now supports explicit declared fragment contributions,
read-only upstream fragments and kept cell recordings. This supersedes those
specific exclusions above. `project/recorded.rs` shares fingerprint and transcript
formatting with native callers through `recording.rs`; unknown cell inputs,
mounts/capabilities and unresolved nested operations still refuse pure replay.
Literal command wrappers and bare command text share the same kept-run meaning.

`just test-literate-projects` passed the current construction, lineage, stale
recording, expectations, native endpoint and candidate-revision witnesses.
The full fixture closures also passed `just verify-literate-project-fixtures`.
Construction/verification remain virtual; the command that materializes output
is explicit. Current dependency hashes, partial grammar limits, and executed
evidence are in `comparison-integration-evidence.md`. Human judgment is not recorded.
