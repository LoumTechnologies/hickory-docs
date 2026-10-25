<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain docs/developers/literate-workflow.md

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="docs/developers/literate-workflow.md">
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
2. Edit its linked Markdown entry. For a known output path, look up `source` in
   `hickory-dogfood/behavior-map.json`. Edit shared entries once and consider the
   other contracts that cite them.
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
constructed output. `audit` checks unique ownership, editorial file links and the
original paragraph dispositions; neither certifies all promises. New output files
need one entry, inclusion in the source closure, and a constructor-map record.

If changing the evaluator or adapter, regenerate, rebuild with
`just literate-bootstrap`, and repeat the checks with that build. Generation uses
Hickory's existing loader, evaluator and safe writer through a thin native adapter.
The full CLI weave path has a known scale limitation from repeated lineage
parsing; this workflow does not claim that issue is fixed.

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
</maint:entry>
</maint:doc>
