> Historical isolated-rewrite record. For the current root layout and commands,
> use [the literate workflow](../docs/developers/literate-workflow.md).

# Observed validation

Date: 2026-10-25. The exact starting revision and dirty working-tree inventory
are in `baseline.json`. All additions for this exercise are under
`hickory-dogfood/`; no commit was created.

## Passed

- Captured 2,829 existing Git-visible working-tree files, totaling 27,762,533
  bytes. Included uncommitted changes and untracked nonignored files. Preserved
  the pre-existing deletion of `.github/workflows/unstable-release.yml`.
- Constructed the tree from 210 Markdown sources: one entry plus 209 component
  chapters. 2,797 entries use raw UTF-8; 32 use supported hex encoding for binary
  content or raw-tag delimiter collisions.
- Bootstrapped with the existing `hick-lang` discovery/evaluator and
  `hick-project-env::write_tree` APIs through the isolated harness. See
  `bootstrap-core.log`. No generation semantics were implemented in Python.
- Recompiled the harness with dependencies pointing exclusively at the
  **generated** `workspace/crates/hick-lang`, `hick-project-env`, and its
  generated workspace dependencies. Re-evaluated and materialized the complete
  tree, matching every baseline entry. See `reproduced-core.log`.
- Compared all 2,829 original/generated pairs directly with byte equality and
  executable flags. Every pair matched. Separately checked SHA-256 hashes,
  sizes, kinds, and complete inventory against the capture manifest.
- Native `target/release/hick project verify project.md --root
  hickory-dogfood/workspace` passed for all 2,829 entries.
- `tools/verify.py --hick target/release/hick --engine
  hickory-dogfood/build/release/hickory-dogfood-core-check --fresh` passed.
  It copied only the Markdown closure into a fresh directory without Git or
  generated outputs, materialized it using the reproduced engine, then passed
  both the independent baseline check and native complete-project verification.
- Extra-file and byte-drift negative controls were rejected by both the native
  verifier and independent comparator. The reproduced native writer refused a
  destination symlink pointing outside the project, and the outside sentinel
  bytes remained unchanged. The temporary tree was removed afterward.
- Original Git status outside the new subfolder matched the captured status.
  Original file hashes/modes remained unchanged.

The exact successful fresh-check output is in `verification.log`. These are
focused construction/equality/safety checks, not a broad regression run.

## Failed or stopped attempts

- Initial bare chapters failed parsing because quoted namespace declarations in
  embedded source could select a different prefix. Explicit `xmlns:hick` roots
  on the literate chapters fixed the construction. The original parser source
  was not changed.
- Debug and optimized whole-tree CLI `weave` attempts were stopped after
  sustained CPU use, before materialization. Inspection of
  `crates/hick-literate/src/project.rs::pipeline` and `origin_kind` found source
  parsing repeated for every provenance range, plus repeated prefix scans for
  line coordinates. The direct evaluator/native-writer harness avoids this
  path. This establishes a practical current limitation, not a measured
  benchmark or proof that it is the only performance cost.

## Boundaries

This demonstrates byte-exact complete-source construction and core-engine
self-hosting. It does not demonstrate a full CLI/app build from the generated
source, interactive editor round trips, full-scale lineage rendering, desktop
acceptance, model quality, or current validity of every old guarantee.

Equality covers file paths, content, regular/symlink kind, and executable status.
The captured tree has no symlinks; the safety check exercises a hostile existing
symlink. Ownership, timestamps, empty directories, full permission bits, Git
history and ignored local data are not reproduced. Installed Rust dependencies
and toolchains are bootstrap inputs. No cell/provider execution or dependency
network fetch was needed by the successful offline harness builds.

The whole application and all broad regression checks remain unrun. Human
judgment on revised requirements remains unrecorded. No post-commit checks were
started or monitored; after a later commit, inspect GitHub Actions using
`gh run list --limit 5` and `gh run view <run-id>`.
