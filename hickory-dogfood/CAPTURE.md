> Historical isolated-rewrite record. For the current root layout and commands,
> use [the literate workflow](../docs/developers/literate-workflow.md).

# Recheck the original component capture

For: maintainers checking that the first Markdown conversion preserves the
working tree that existed before the guarantee-organized rewrite.

`workspace/project.md` declares the earlier component-based construction.
`baseline.json` records its exact files, hashes, kinds, executable status,
starting revision and dirty state. Keep that manifest unchanged.

Run from the original repository root with Rust, the captured dependencies and
the existing `target/release/hick` installed:

```sh
CARGO_TARGET_DIR="$PWD/hickory-dogfood/build" cargo run --offline --release \
  --manifest-path hickory-dogfood/tools/core-check/Cargo.toml -- \
  hickory-dogfood/workspace hickory-dogfood/baseline.json --materialize
python3 hickory-dogfood/tools/verify.py --hick target/release/hick \
  --engine hickory-dogfood/build/release/hickory-dogfood-core-check --fresh
```

The harness compiles the reproduced Hickory evaluator, checks the complete
construction against the manifest, then materializes it. The fresh check copies
only the Markdown closure into a temporary directory and repeats generation,
inventory comparison and refusal controls. It removes that temporary directory.

The capture includes tracked files and untracked nonignored files. Deleted
tracked paths remain absent. Git history, ignored local data, ownership,
timestamps, empty directories and full POSIX permissions are outside the capture.
Executable status is retained. Binary and delimiter-colliding payloads use hex
entries, which have no byte-equal reverse editing map.

The ordinary whole-tree CLI weave attempts were stopped because lineage
conversion repeatedly reparses source. The harness uses the existing project
APIs directly. Successful reconstruction does not establish full UI dogfooding
readiness or semantic validity of old specifications.

[The original validation record](VALIDATION.md) and [reset findings](RESET-REVIEW.md)
retain that stage's evidence. Use [the integrated guide](INTEGRATED.md) for the
current guarantee-oriented editing workflow.
