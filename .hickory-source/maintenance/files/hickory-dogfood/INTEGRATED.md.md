<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/INTEGRATED.md

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/INTEGRATED.md">
> Historical isolated-rewrite record. For the current root layout and commands,
> use [the literate workflow](../docs/developers/literate-workflow.md).

# Change a behavior and review its consequences

For: Hickory maintainers who need to reconcile guarantees, change the code behind
them, and show reviewers what was actually checked.

Start with [the behavior-organized project](integrated/project.md). Each guarantee
leads to the actual implementation and test constructions. Shared files are
constructed once and link back to the guarantees that cite them. The earlier
[component capture](workspace/project.md) remains available as the before-state.

For example, a patch must reject incorrect context without changing its input.
Open [that guarantee](integrated/.hickory-source/guarantees/literate-projects/replay-is-exact-and-atomic/contract.md),
read its exact-input rule, then follow the patch implementation and the two named
witnesses. The links reach maintained source entries, not copied code excerpts.

## Make and check one change

Prerequisites: run these commands from the original repository root. Rust and the
captured dependencies must already be installed for offline builds. Verification
also uses the existing `target/release/hick`. Keep build output in the dedicated
`hickory-dogfood/build` directory.

Build the small adapter against the integrated Hickory source:

```sh
CARGO_TARGET_DIR="$PWD/hickory-dogfood/build" cargo build --offline --release \
  --manifest-path hickory-dogfood/tools/behavior-engine/Cargo.toml
```

1. Open a guarantee from the project index. Read its scenario, implementation,
   witnesses and limits. Check its review status before treating it as current.
2. Follow an implementation link and edit the raw entry that constructs the file.
   Shared entries list the other guarantees to consider. Editing generated Rust
   directly would leave its maintained Markdown behind.
3. Generate the new working files and run the named witnesses:

```sh
python3 hickory-dogfood/tools/behavior.py generate
python3 hickory-dogfood/tools/behavior.py test \
  literate-projects/replay-is-exact-and-atomic.md --execute
```

4. Check that every working file agrees with its maintained source:

```sh
target/release/hick project verify project.md --root hickory-dogfood/integrated
python3 hickory-dogfood/tools/behavior.py audit
```

The test command requires exactly one passing assertion per named Rust or Vitest
witness. Frontend witnesses use the integrated source and the locally installed
frontend dependencies. A temporary dependency link and external cache are cleaned
up after each run. It records
commands, output and dependency hashes in `focused-results.json`. Omitting
`--execute` prints the commands. A guarantee without a curated witness refuses
execution; its earlier citations remain candidates to inspect.

If you change the parser or evaluator itself, generate its source, rebuild the
adapter, and check again using the new implementation. The adapter delegates
construction and safe writing to Hickory; it adds no alternative language engine.

## Review the specification reset

The current structure has 326 behavior pages: 32 newly reconciled requirements,
eight rewritten earlier, 277 retained requirements and nine historical contracts.
Each current promise is separate from its implementation review. Long requirements
have linked detail pages; earlier notes remain in the preserved originals and the
exact paragraph ledger. See [the reconciliation report](reconciliation/REPORT.md).

Use [the review queue](INTEGRATION-REVIEW.md) to choose the next contract. For each:

1. State the stakeholder's observable outcome and a concrete failure example.
2. Reconcile every amendment and exclusion against the current implementation.
3. Split unrelated outcomes; combine truly equivalent promises under one owner.
4. Select or add a focused witness. Record execution separately from code review.
5. Keep the old wording and decision source linked as history.

A short replacement must account for all earlier clauses. The prediction summary,
for example, still needs its adapter-specific clauses reconciled. Moving them to
history does not discharge that work.

`behavior-map.json` records every file's construction owner, guarantee references,
review flags and design-history disposition. It also lists 562 implementation
files with no direct guarantee reference. Those files remain constructed; their
absence from a contract is visible review debt. Reference discovery is mechanical
and may miss links or identify an unrelated nearby function. Inspect it before
claiming semantic coverage.

The one-time migration scripts created this layout. Maintain the emitted source;
rerunning capture or integration is not the editing workflow. The scripts refuse
to overwrite an existing target. Update the review map when changing ownership,
contracts or curated witness definitions, then run its audit.

## Recheck the unchanged baseline

Before intentional implementation changes, the entire integrated project should
still match the original capture:

```sh
python3 hickory-dogfood/tools/verify.py --root hickory-dogfood/integrated \
  --hick target/release/hick \
  --engine hickory-dogfood/build/release/hickory-dogfood-core-check --fresh
```

This copies only Markdown into an empty Git-free directory, reconstructs the
program, compares all 2,829 captured files and their portable metadata, and runs
extra-file, byte-drift and outside-symlink controls. Build the original core-check
adapter with the command in [the capture guide](CAPTURE.md) if its binary is absent.

Keep `baseline.json` immutable. Intentional implementation changes should differ
from it; use the ordinary project verifier for agreement with the new source.
The baseline check deliberately refuses a changed target.

[The validation record](INTEGRATED-VALIDATION.md) separates reconstruction,
executed witnesses and remaining acceptance work. Seventeen passing focused witnesses
do not certify all requirements or desktop readiness. The full CLI weave
path still has a scale limitation; the adapter avoids its repeated lineage parsing.
</maint:entry>
</maint:doc>
