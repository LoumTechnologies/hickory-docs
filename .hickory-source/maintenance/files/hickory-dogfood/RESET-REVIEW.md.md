<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/RESET-REVIEW.md

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/RESET-REVIEW.md">
> Historical isolated-rewrite record. For the current root layout and commands,
> use [the literate workflow](../docs/developers/literate-workflow.md).

# Specification reset review

The literate construction preserves the current working tree. It does not make
its requirements consistent. Treat the captured `docs/guarantees` and
`docs/specs/freeform` trees as frozen evidence while deciding the next contract.
No original file was edited for this exercise.

The full mechanical inventory is `spec-inventory.json`: 334 guarantee-area
Markdown files (including audits/evidence), 61 freeform specs, their hashes,
headings, sizes, and 24 distinct long paragraphs repeated across files. These
are duplication candidates, not 24 proven redundant requirements. This review
inspected selected conflicts; it is not a semantic certification of all 395 files.

## Findings that need decisions

| Finding | Concrete evidence in the captured tree | Proposed disposition |
|---|---|---|
| A contract still requires a deleted backend | `docs/guarantees/execution/output-lineage-round-trips-byte-for-byte.md` requires a Postgres update and Git commit; its own verification explicitly says that hosted server no longer exists and the local path reweaves on request. | Rewrite the current contract around current file-backed behavior. Preserve old hosted wording in historical evidence, not the normative paragraph. Inspect local write atomicity before adopting stronger claims. |
| Sharing is simultaneously absent and supported | `docs/guarantees/collaboration/local-session-writes-your-files.md`, Boundary: no sharing, capability link or relay, explicitly not deferred. `collaboration/the-peer-channel-carries-grants.md` promises authenticated fleet access and permits number0 relay/DNS infrastructure. | Separate loopback document persistence from optional peer transport. Decide whether the product promises no operated service or no third-party service; these are different promises. |
| The architecture sends readers through obsolete product decisions | `docs/specs/freeform/architecture.md` still describes Canopy/cloud execution and points to `local-first.md` for product shape; `local-first.md` is explicitly superseded by `local-only.md`, which has further amendments. | Write one current product-boundary document and one implementation map. Move the hosted/relay evolution into decision history. Do not infer current authority from the newest paragraph alone. |
| The round-trip promise has an encoding boundary | `docs/developers/literate-projects.md` says hex entries have no byte-equal reverse edit map. `docs/guarantees/execution/output-lineage-round-trips-byte-for-byte.md` is limited to literal/pasted byte-precise ranges. | Keep source equality and reversible editing as separate contracts. This capture uses readable UTF-8 entries where possible and hex entries for binary/delimiter-colliding files; the latter need edits to their construction. |
| Verification history has overtaken the contract | `docs/guarantees/agent/ui-action-dispatch.md` has 1,016 lines; `verification/predictions-precede-managed-observation.md` 990; `shell/spatial-views-preserve-the-editor.md` 443. The spatial file explicitly supersedes its own gap-only evidence in later corrections. | Keep a short current contract per observable behavior. Move dated witnesses, corrections and hashes to evidence records, retaining links and supersession chains. Preserve historical mistakes without making readers interpret them as current requirements. |
| Shared evidence is repeatedly copied | `spec-inventory.json` identifies exact paragraph duplication, including Python-runtime amendments across seven agent guarantees and shared-controller evidence across multiple domains. | Store a shared evidence record once; each current guarantee cites the specific relevant witness and any local caveat. A shared dependency is not a reason to merge distinct behavioral guarantees. |
| Byte equality cannot resolve changed intent | `docs/specs/freeform/literate-changesets.md` distinguishes transition A-to-B from maintained empty-tree-to-B construction, and explicitly says equality does not prove explanatory truth or runtime correctness. | Use this construction as the fixed baseline. Agree changed requirements separately, then change the source and its focused witnesses together. |

## Proposed current organization

The review map below replaces amendment chains with one clear place to start;
it does not silently adopt every old promise.

1. Product boundary: local storage, optional peers, provider network access,
   release/install expectations, and supported platforms.
2. Language and construction: parsing, source closure, complete inventory,
   deterministic generation, paths, binary assets, and metadata.
3. Editing and persistence: authored/generated views, lineage, refused edits,
   unsaved work, atomic saves, and local history.
4. Execution and trust: confinement, timeouts, authorization, secrets, and
   observations. Separate offline construction from running cells.
5. Agents and evidence: context, approvals, changesets, assumptions,
   recordings, prediction scope, and completion status.
6. User surfaces: editor intelligence, navigation, tables, diagrams,
   terminal/debugging, browser embedding, and desktop integration.
7. Historical decisions and check evidence: separate appendable evidence from
   current contracts; one dated evidence record may support several guarantees.

Use `spec-inventory.json` to assign every existing file to keep, rewrite,
split, merge, archive, or retire. Prioritize the incompatible promises above.
The reset is complete only when every file has a disposition and every retained
contract has one owner, clear scope, and a focused observable witness. The
current capture provides that inventory, not those unmade product decisions.

## Assumptions, evidence and limits

- Requirement: evaluate a separate version that uses Hickory to construct the
  desired source byte-exactly without changing the existing repository files.
  Interpretation: the desired baseline is the current Git-visible working tree,
  including dirty and untracked nonignored files, not HEAD or ignored secrets.
  Alternative: generate HEAD only; rejected because it would omit existing work.
- Captured revision, paths, hashes, construction encodings and initial dirty
  state are in `baseline.json`. The deleted `unstable-release.yml` remains
  absent. All captured files are regular files; the baseline contains executable
  bits but no symlink test case. Git history, ownership, timestamps, full POSIX
  permission bits, empty directories and ignored local state are outside equality.
- Bootstrap initially attempted the existing native `hick` executable, then used
  the existing Hickory Rust APIs through the isolated harness. Source closure
  discovery, parsing, generation, native writing and comparison are Hickory
  features, not a Python reimplementation.
  The Python capture writes declarations; the independent checker hashes files.
- Challenge/falsifier: any missing, extra, different-kind, differently executable,
  or byte-different file disproves capture equality. A Markdown-only rebuild
  challenges hidden dependencies on the original source tree.
- A real dogfooding failure: a bare chapter containing an embedded `xmlns:h`
  example can change global prefix detection before raw entry parsing. Explicit
  `xmlns:hick` roots on all chapters avoid this without changing the engine.
  The underlying namespace-scanner behavior remains in the frozen implementation.
- A second dogfooding issue: `crates/hick-literate/src/project.rs::origin_kind`
  reparses the source document for each byte-origin range. Whole-tree CLI weave
  attempts were stopped after sustained CPU use. Direct project evaluation and
  native tree writing bypass that conversion; no original implementation was
  patched to make equality pass.
- Engine scale matters: this is a roughly 28 MB whole-repository construction,
  including tracked screenshots, generated WASM, and a tracked debug artifact.
  The baseline deliberately retains those rather than silently filtering them.
- LLM implementation review: selected discovery/evaluation/materialization and
  inventory paths inspected; whole-application correctness not verified.
- Executed checks: see `VALIDATION.md`. Earlier checks quoted inside frozen
  specs are historical claims and were not rerun by this exercise.
- Human judgment: no confirmation of revised product requirements recorded.
  No guarantee has been made current merely by copying or generating it.
</maint:entry>
</maint:doc>
