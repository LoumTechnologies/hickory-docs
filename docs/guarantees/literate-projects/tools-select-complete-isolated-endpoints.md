# Native project tools select complete endpoints

Given an evaluated project closure, when native services select New, then every
file comes from its completed tree, including changes in later patches and other
files. Old selects the captured input and refuses materialization as a runnable
project when empty. Local states require an explicit selection. Explicit native
materializations use distinct private temporary directories. Deleted bytes have
no endpoint mapping; ambiguous origins refuse routing. Coordinates stay bytes
until conversion to editor/LSP UTF-16 positions.

The live LSP entry and native document debugger use the same project evaluator
for declared closures. Working-tree manifests are not copied into declared LSP
projects to supplement missing construction outputs.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for declared closures on the tested macOS arm64 toolchain;
  cross-document protocol mapping and additional scope selectors remain excluded.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, atomic
  endpoints and separate native identities.
- Evidence: `hick-lsp/src/project.rs`, `document.rs`, `backend.rs`,
  `position_map.rs`; `hick-dap/src/project.rs`, `program.rs`, `session.rs`;
  `hickory-cli/src/project_documents.rs::debug_state` and `debug_sessions.rs`.
- Witnesses and challenges: `hick-lsp/tests/project.rs` checks final library bytes
  despite the incomplete middle patch, distinct materializations, empty Old,
  deleted-line refusal, Unicode coordinates and existing-symlink refusal. The
  explicit `hickory-cli/tests/literate_project_csharp.rs` acceptance test builds
  EFCore, applies the final migration, persists a note, and stops netcoredbg at
  patch-inserted code with the correct document line and watched value. It caught
  diff-prefix column routing and macOS `/var` versus `/private/var` source alias
  failures; line breakpoints and canonical native identities now fix both.
- Assumptions and exclusions: the UI defaults to New; Old/local selection currently
  exists at the native API rather than a new toolbar selector. Cross-document
  origins remain exact in the portable map; the legacy single-document LSP/DAP
  mapping refuses foreign spans instead of binding them to the entry document.
  Explicit nested atomic scopes and cross-worktree tool selection remain pending.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; native entry/mapping changes invalidate this review.
- Human judgment: not recorded; discovering an adapter is not successful debugging.

## Continuation review (2026-10-12)

This supersedes the pending selector/foreign-document exclusions above for the
working continuation. `project_contexts.rs`, `project_positions.rs` and
`location_map.rs` carry captured project and source identities. The document
selector exposes explicit scopes and advanced local states. Comparison tool
registration is lazy and memory-only; native tools materialize on demand.
Revision-local paths keep Old tools usable across a rename.

Executed witnesses: `just test-literate-comparison-editor` checks complete later
file state, explicit enabling, empty Old, scopes and renamed paths through the
actual WASM evaluator; its native transport is mocked. The real HTTP
`native_context_registration_freezes_memory_without_borrowing_working_code` test
checks registration, retrieval and removal despite changed working bytes.
`just test-literate-projects-csharp` passed the real migration/runtime/debugger
flow on macOS arm64. Whole native toolbar automation and other platforms remain
unverified. Dependency hashes and executed status are retained in
`comparison-integration-evidence.md`; human judgment is not recorded.
