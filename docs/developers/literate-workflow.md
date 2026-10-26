# Change Hickory through its behavioral contracts

For maintainers who already build Hickory and need to change one behavior.

Open [project.md](../../project.md), choose the stakeholder task, and follow its
contract to the Markdown entries containing implementation and tests. For example,
[exact and atomic patches](../../.hickory-source/guarantees/literate-projects/replay-is-exact-and-atomic/contract.md)
links the actual patch implementation and two named witnesses.

## Bootstrap once, then change a behavior

A clone includes generated working files, so the existing build commands still
work. Generation needs Python 3 and the repository's Rust toolchain and crate
dependencies. Run from the repository root:

```sh
just literate-bootstrap
```

1. Read the contract's outcome, review status, evidence and exclusions.
2. Edit the actual `copy` fragment beside its explanation. `file`/`paste`
   constructions specify output order. For a known generated path, run
   `python3 hickory-dogfood/tools/behavior.py locate <output-path>` to find the
   assembly owner and every fragment, including tests hosted by other contracts.
   Edit shared mechanisms once and consider the contracts that use them.
3. Regenerate and check agreement:

```sh
just literate-generate
just literate-check
```

4. Execute only the focused witness relevant to your change. For example:

```sh
python3 hickory-dogfood/tools/behavior.py test \
  literate-projects/replay-is-exact-and-atomic.md --execute
```

Omit `--execute` to print the exact commands. A contract without a curated witness
refuses execution. Add an appropriate focused witness after inspecting its actual
coverage. Frontend witnesses require the installed `apps/web` dependencies. New execution
receipts go to `target/dogfood/focused-results.json`; copy relevant evidence into
the maintained contract or evidence entry before committing it.

Commit the Markdown entries, generated working files, and relevant evidence
together. `check` compares bytes, file kinds and Unix executable bits for every
constructed output. `audit` checks actual output constructions, unique fragment definitions, selected
fragment references, editorial file links and original paragraph dispositions; neither certifies all promises. New output files
need one construction, inclusion in the source closure, and a constructor-map
record. Several outputs may be explained on the same page. New fragments also
need their owner recorded in `hickory-dogfood/narrative-map.json`.

If changing the evaluator or adapter, regenerate, rebuild with
`just literate-bootstrap`, and repeat the checks with that build. Generation uses
Hickory's existing loader, evaluator and safe writer through a thin native adapter.
The full CLI weave path has a known scale limitation from repeated lineage
parsing; this workflow does not claim that issue is fixed.

## Read and maintain a narrative

Keep the explanation beside the code whose decision it explains. A behavior page
contains its requirement and directly owned code/tests. Larger behaviors link to
named implementation chapters. Shared mechanisms explain reused code once;
callers explain why they use it. Literal entries remain appropriate for assets
and source containing embedded document syntax.

Do not assume every declaration has an adequate explanation. The
[explanation ledger](../../hickory-dogfood/narrative-map.json) distinguishes
source-comment rationale, explanations inspected during reorganization, and gaps.
The [reorganization record](../../hickory-dogfood/REORGANIZATION.md) states the
actual scope. Give unexplained logic a concrete rationale after inspecting it;
link ownership and passing reconstruction do not supply that rationale.

## Regenerate the API adapters

The [route chapter](../../.hickory-source/mechanisms/apps/web/src/api/routes.ts.md)
contains typed declarations for simple API forwarding. Keep conditional
serialization and transport decisions in the manual client. To change a route:

```sh
npm --prefix apps/web ci
npm --prefix code-models/typescript ci
# Edit routes.ts.md or the generator's owning Markdown, then materialize inputs:
just literate-generate
python3 scripts/generate-api.py --write
just literate-generate
just literate-check
```

The write step refreshes a kept Hickory exec result. Ordinary reconstruction
replays that result offline; `literate-check` additionally reruns the model and
refuses stale generated adapters. It requires Python 3 and Node with both locked
npm dependency sets installed. CI uses Node 22; the pilot was verified locally
with Node 24.14.0. The host recording does not claim container provenance.

## Review the migration

[The source index](../../project.md) is canonical. The generated old files in
`docs/guarantees/` and `docs/specs/freeform/` remain historical inputs. The
[reconciliation report](../../hickory-dogfood/reconciliation/REPORT.md), immutable
baseline, paragraph ledger and earlier focused receipts preserve the reset's
limits. Older receipts describe the isolated layout in which they ran.

The migration preserves application source bytes. Deliberate maintenance changes
are recorded in `hickory-dogfood/promotion.json`; it also records one newer upstream
file preserved during promotion. Code-model simplification is deferred to
[issue #115](https://github.com/LoumTechnologies/hickory-docs/issues/115).

To reconstruct in an empty directory using only the maintained Markdown and run
focused drift/symlink controls:

```sh
python3 hickory-dogfood/tools/promotion-check.py
```

Add `--baseline` to also enforce the original migration byte hashes. That check
will intentionally fail after later implementation changes; ordinary reconstruction
and source agreement checks continue to apply.

The source closure also constructs the migration tools and records. `project.md`
and `.hickory-source/` are the maintained inputs; generated working files remain
committed for bootstrapping and existing CI/build commands. Earlier local
`hickory-dogfood/workspace` and `integrated` copies are ignored snapshots.

## Return to the old layout

The recovery branch `pre-literate-dogfood` contains the ordinary source layout,
including the release work present before promotion. Its exact commit is in
`hickory-dogfood/promotion.json`. Inspect it in a separate worktree:

```sh
git fetch origin pre-literate-dogfood
git worktree add ../hickory-docs-recovery origin/pre-literate-dogfood
```

The first command requires GitHub access. The worktree leaves current work in
place. Alternatively, after committing or stashing local changes, use
`git switch pre-literate-dogfood` to work on the recovery branch here.

Post-commit checks run asynchronously. Broad regressions and status monitoring
belong to the maintainer. Use `just checks-status` for local status (see
`just --list` for the current command surface), or `gh run list --limit 5` and
`gh run view <run-id>` for GitHub Actions. Do not infer readiness from the migration
checks alone.
