# Literate changesets and output-preserving incorporation

*Status: agreed direction 2026-10-12, partially implemented. Audience: engineers
implementing patch elements, project verification, revision-aware language tools,
and agent finalization.*

**Implementation boundary (2026-10-12).** Declared Markdown projects now have a
portable virtual tree, explicit source closure/inventory, exact transactional
UTF-8 unified-diff replay, metadata/binary construction, deterministic ordering,
byte lineage, complete comparison, and Git commit/index/directory verification.
The normal declared-project run/weave/test path, native LSP staging and document
debug staging consume that evaluator. Patch elements render through the shared
diff view and actual WASM parser. Real EFCore migration/runtime/debugger and
Mercurial revision tests have passed on this machine. Refactor baselines retain complete project
outputs. Pure services cover N-source permissions, patch regeneration, A-to-B
reconstruction and separately verified incorporation candidates. See
`docs/developers/literate-projects.md` and
`docs/guarantees/literate-projects/implementation-evidence.md` for grammar, tests
and limitations.

**Continuation (2026-10-12).** The working implementation now wires editable
comparisons, explicit scopes, declared fragment/upstream/kept-recording composition,
cross-document native coordinates, built-in/Python/ACP capture and incorporation
review, automatic frozen-index project selection, and merged-view target routing
and guarded undo. Current executed evidence and remaining limits are recorded in
`docs/guarantees/literate-projects/comparison-integration-evidence.md`.
Changed binary/metadata transition reconstruction, runtime-dependent source
construction without a kept recording, live model explanation quality, and
additional native-platform acceptance remain outside the verified subset. This
does not declare every original acceptance flow complete.

An agent can implement EFCore persistence, discard unsuccessful approaches, and
finish with a literate explanation of the successful changes. Applying that
explanation updates the project's maintained literate source. The repository
does not accumulate every agent's final response as another source document.

## The two executable artifacts

**A final agent response describes a transition from A to B.** A is the exact
project state when the turn began, including existing uncommitted files. B is
the completed result. The explanation omits unsuccessful detours; the actual
session remains the record of what happened. This explanation is an authored
reconstruction, not an execution transcript.

**Maintained literate source constructs B from an empty file tree.** Copying
only the literate `.md` source closure into a new, git-free folder and processing
it must reproduce the same project source files as the original revision.
No previous commit, working-tree code, hidden cache, or sidecar is a required
source of project bytes. Source discovery and required configuration must be
recoverable from the Markdown closure itself. Existing optional project config
may select an entry point, but cannot be the only copy of necessary input.

Patches in maintained source change code constructed earlier by that source
closure. They are part of the current construction, not an append-only history
of prior commits. The literate documents can be rewritten, split, combined, or
deleted when organizing the present program. Repository history holds earlier
versions of that explanation.

This replaces the earlier discussion's previous-commit-base proposal. Git,
Mercurial, and persisted CRDT revisions are optional sources for comparisons and
agent transition bases; generating maintained source requires none of them.

## First successful workflow

1. Capture the agent's starting tree A and identify pre-existing user work.
2. Let the agent use its normal ReAct loop. Capture the completed tree B.
3. Generate a `.md` response explaining the successful transition A to B.
4. Apply its declared changes to A in memory and compare the result with B.
5. Show the response through the same comparison element used by change reviews
   and history readings. The actual session is separately available.
6. On **Apply to literate source**, use AI to update the maintained documents.
   New documents and deletion or reorganization of existing ones are allowed.
7. Generate their complete source tree from an empty in-memory tree and compare
   it with B. A failed comparison names missing, extra, or differing outputs.
8. Review and apply the verified literate-source edits. If the working code is
   already B, do not apply the code transition a second time. Recheck destination
   identities before writing; concurrent edits require a new candidate.

The two checks are distinct:

```text
apply(final_response, A) == B
generate(updated_literate_source, empty_tree) == B
```

These checks prove output equality. They do not prove that every explanatory
sentence is accurate or that equal source files have correct runtime behavior.

## Verification happens in memory

Use one virtual project tree: normalized relative paths mapped to exact byte
content and relevant entry metadata. Preserve file kinds, executable modes, and
symlink targets where supported. Unsupported kinds must produce an explicit
error, not disappear from comparison. Text patches initially operate on UTF-8;
binary files remain byte-exact entries with an explicit supported construction
or a named unsupported operation.

Construct, patch, rename, remove, and compare entries without writing the
generated project to disk. Snapshots may share immutable entries; copying the
whole tree for each patch is unnecessary. Paths must remain inside the virtual
project; validate collisions, traversal, and platform path ambiguities.

Compare the complete output inventory as well as content. Extra source files
are failures, not ignored because no document claimed them. Define the source
inventory separately from input `.md` files, woven reading artifacts, VCS
metadata, caches, installed dependencies, and transient build outputs. Include
project files, lockfiles, migrations, and source assets needed to reconstruct
the program. The inventory policy must travel with the Markdown closure and
cannot silently exclude unexplained source files to make verification pass.

Commit verification reads the documents and expected outputs of the exact
candidate revision or Git index. It does not compare staged documents with
unstaged outputs. Historical verification requires only that revision's
documents and expected tree, not its ancestors. A git-free directory uses the
same generator and comparator.

Pure source generation and equality checking require no scratch-tree writes.
Reading source inputs or VCS objects is allowed. External cells, compilers,
tests, and native language tools may require disk and processes: use kept
in-document recordings where sufficient, otherwise report that execution is
required. Never silently run a process from an in-memory verification request.
Real execution checks are separate from source-equivalence evidence. Existing
toolchain requirements still apply; a git-free folder is not a toolchain-free
machine. A retained generator result must survive copying the Markdown files.

## Patch construction and composition

Start with concrete textual changes, not CRDT operation streams or structural
merging. Implement exact unified-diff application over a known virtual input.
Preserve BOMs, CRLF, final-newline state, Unicode, and all unchanged bytes.
Reject wrong context, malformed hunks, and unsupported encodings. No fuzzy
offset search, implicit merge, or model call occurs during deterministic replay.

Expose one patch element through the existing registry. The proposed source
spelling is `<h:patch>` containing unified diff bytes; `h:` here is the literal
documentation example prefix, not a second runtime namespace. Finalize its
attributes and raw-body grammar during milestone 1. A patch-looking ordinary
Markdown fence remains display-only unless explicitly converted to the executable
element; do not introduce a second Hick parser or silently execute quoted diffs.
The view presents the element as a patch code block.

A patch can change several files as one act. Within a document, explicit
construction dependencies establish patch order. Across documents, integrate
with the existing DAG, includes, and ambient copy/paste contributions. Establish
one predecessor project state for each transformation; independent contributions
may coexist, but unordered transformations of the same bytes must be refused.
Do not turn filename order into an undocumented last-writer-wins rule. Preserve
existing upstream write boundaries; any new composition edge must explicitly
declare its generated-file input and output rather than mutate upstream source.

An EFCore example should read: construct the initial app, add persistence code,
add the final migration, and verify the completed program. Its maintained source
can later fold those constructions into a simpler explanation. Incorporation
need not preserve the old patch shapes or the agent's chronological steps.

## Atomic project services and editable sides

**Debug selection amendment (2026-10-17):** a debug action selects the latest
whole evaluated project containing the exact file bytes displayed beside that
action. This supersedes the Old/New debug defaults below; explicit endpoint
APIs and ordinary document language-service defaults remain available. There
is no global project-version selector. Comparison panes use the same resolved
capture for breakpoints, launch and language tools. The candidates are the
input and cumulative evaluated project states in dependency order, not lexical
construction-id order, not mixed-file synthetic projects, and not Git history.
Later changes to other files are included while the displayed file remains
unchanged. A later replacement selects an older compatible state; a byte-equal
revert can select the latest state again. Renames require unique provenance.
An unavailable matching state is explained without launching different code or
opening a replacement editor. Native builds can still fail: selecting a state
does not certify it builds, and selection does not run speculative builds.


The default atomic scope is a document's evaluated project closure. Patches
retain ordered local before/after states internally, but invoking project
services halfway down the document does not select a half-built program.

**New** uses the completed scope result for every file, project configuration,
dependency selection, LSP operation, and DAP launch. **Old** uses the scope's
input project. For a transition response that is A; for a from-scratch program
it may be empty. A narrower project boundary needs an explicit atomic scope;
do not invent an executable old project when the input is empty. Include support
for explicit scopes after proving the default, with no implicit per-file scopes.
Intermediate-state inspection remains an explicit advanced action.

The viewer must identify the project context beside language/debug actions.
Displayed patch before/after text can differ from atomic endpoints. Map surviving
code to the chosen endpoint; code absent from that project has no valid breakpoint
or endpoint hover. Explain this and offer explicit local-state inspection.

Edit permissions are independent: neither, old, new, or both. New-only review is
the normal editable proposal. Editing Old edits an authored input construction
when one exists; it never rewrites an immutable VCS revision. Immutable readings
can be forked into editable comparisons. Regenerate the transition and invalidate
verification after any edit. In maintained programs, both sides must remain
consistent with their constructions and downstream dependencies; an old-side
edit can therefore invalidate later patches. Keep the new-side endpoint fixed
when regenerating an old-only standalone comparison, or report why preserving
it cannot be expressed against the edited input.

Output provenance carries untouched bytes through their earlier origins and
maps inserted bytes to patch source spans. Preserve before-side mappings for
deleted code. Reverse edits route to the contributing literal or patch bytes;
synthetic or ambiguous locations refuse an edit with the source destination
named. Position maps retain byte spans and convert to editor/LSP coordinates
at their existing boundaries.

## Shared views without shared storage rules

Use one comparison model for saved patch elements, agent proposals, ACP diff
activity, working-tree diffs, commit readings, and merge/worktree views. It names
revision sources, comparison relationships, writable targets, project contexts,
and provenance. Its foundation supports N sources even when the initial
presentation has two panes. Merely importing an ACP diff is not replay evidence.

Reuse `DocumentComparison`, `CommitReading`, `DiffView`, and the element view
registry. Git history defaults to immutable source states; existing draft-history
actions continue to respect the publication floor. A lens has no save path.
Temporary branch/worktree variants remain synthesized views, not branch-named
conditionals committed to literate source. Permanent product variants remain
ordinary declared document variants.

Reuse merged-view target routing and per-target undo/status. Native LSP/DAP
materialize the selected complete revision only when their tools need it, with
separate project identities so old/new or several worktrees cannot share stale
diagnostics or build artifacts. A multi-target view debugs one selected project,
never a mixture. Rendering alone does not launch tools or execute cells.

## Implementation milestones

1. **Contracts and a runnable construction fixture.** Settle the patch source
   grammar, source inventory, entry discovery, explicit composition rules, and
   atomic endpoint labels. Build a two-file example with several patches and a
   temporarily uncompilable intermediate state. Add strict guarantees when each
   behavior is implemented, with tests naming their guarantee paths.
2. **Virtual tree and exact replay.** Put portable project construction and
   comparison below the host layer. Reuse `FileContent` and `compare_outputs`
   where possible, extending metadata and reporting without a parallel checker.
   Implement patches and provenance through `hick-lang`, `hick-exec`/
   `hick-literate`, and `hick-blocks`; keep new modules small and explicit facades.
   Prove fresh-folder generation and verification with no materialization calls.
3. **One comparison editor.** Add the patch view under
   `apps/web/src/elements`, through the parser/WASM and element/action seams.
   Implement permissions, patch regeneration, mappings, invalidation, and common
   inline/side-by-side rendering. Connect agent proposals and commit readings.
4. **Whole-project LSP and DAP.** Adapt `hick-lsp` virtual files/staging and
   `hick-dap` program/build/mapping paths to consume the engine's evaluated tree.
   Do not reconstruct code with a separate copy/paste-only implementation.
   Add atomic scopes and explicit local-state inspection using the same evaluator.
5. **Output-preserving refactoring and verification.** Generalize
   `serve/refactor.rs` from a document's outputs to its complete source closure.
   Keep the baseline virtual and compare exact outputs on each check. Add full
   inventory verification against an exact candidate commit/index and a directory
   snapshot. Wire selective checks through root `just` and existing CI/hooks.
6. **Agent finalization and incorporation.** Share A/B capture, reconstruction,
   and incorporation services across built-in/Python and ACP-integrated agents.
   Native ACP writes may bypass document tools: finalization must capture their
   actual output state or report incomplete capture. Concurrent user writes must
   not be silently attributed to an agent. Resolve ambiguity before finalizing.
   Reuse session/provenance and review policy rather than adding a second queue.
7. **Other revision sources and multiple targets.** Implement comparison input
   adapters for Mercurial and persisted CRDT snapshots alongside Git/directory
   snapshots; generation remains VCS-free throughout. Integrate the existing
   merged view with the shared model and test target routing, permissions, and
   project-tool isolation. No patch syntax change is needed for extra targets.

Each milestone delivers a usable boundary and records its remaining limits.
No implementation step may claim support merely because an adapter is discoverable.

## Acceptance evidence

- Copy only the Markdown source closure into a fresh folder with no `.git`,
  cache, generated code, or sidecars. Generate exactly the original source tree.
  Repeat after reorganizing and after revising the maintained explanation.
- Evaluate and compare virtual trees with disk materialization disabled. Catch
  missing/extra paths, byte changes, modes, and unsupported entries. Preserve
  Unicode, BOM, CRLF, and final-newline distinctions.
- Modify several documents contributing to one file; reject ambiguous ordering
  and incomplete candidates. A staged partial change cannot pass by borrowing
  an unstaged source or output.
- A middle patch is temporarily uncompilable. Default New LSP/DAP use the final
  complete project, including later changes to other files. Old uses the declared
  scope input. Removed lines do not bind to unrelated final lines.
- Exercise all four edit policies and reverse editing. After an early edit,
  report invalid downstream patches rather than silently dropping them.
- Reorganize documents while preserving outputs; then intentionally omit an asset
  or migration and observe a failed comparison. Verify a completed EFCore/C#
  flow with the actual supported toolchain, not only a mocked debugger.
- Finalize an agent session containing abandoned writes into A-to-B explanation,
  then incorporate it into from-scratch source producing B. Cover dirty A,
  native ACP file edits, and concurrent destination changes.
- Compare Git, Mercurial, CRDT, and directory revisions through the same model;
  freeze moving CRDT state. Test multi-worktree writes and partial-write undo
  without treating them as a cross-target filesystem transaction.

## Scope and existing decisions

Work items, Jira/PR orchestration, structural merging, syntax-tree CRDTs, trained
merge models, automatic rebasing/adaptation, and Jujutsu history editing are out
of scope. No new hosted service, account, telemetry, or mobile execution is added.
Browser embedding retains its documented execution limits; portable replay and
native project tools must not imply browser container or native DAP support.

The architecture follows [the minimal core](the-minimal-core.md). Maintained
source follows [expression and log](expression-and-log.md); synthetic targets
follow [the merged view](the-merged-view.md) and [lenses](lenses.md). This plan
does not revive the scaffold-base-in-a-cache proposal from
[scaffolded files and derived edits](scaffolded-files-and-derived-edits.md): a
patch here has exact inputs constructed by the source closure or captured for
the transition artifact. Kept generator data uses the existing in-document
recording mechanism. There is no durable dependency on a hidden cache.
