<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Prove the construction workflow before rewriting the story

For: engineers reorganizing Hickory's own literate source, with product reviewers reading the behavior beside its implementation.

Update: the immutable-scaffold prerequisite below supersedes the first fence-ingestion baseline. Use the refreshed `engineer-first-ready.json` receipt for the narrative refactor.

The intended story starts with what Hickory builds, runs a scaffolder, and introduces requirements beside the authored changes and focused assertions that implement them. The scaffolder's captured bytes stay in Markdown. Hickory folds them by default so boilerplate does not overwhelm the reasoning. Multiple chapters are optional organization, not a requirement to reconstruct a small project.

## Compare that experience with the implementation

| Need | Current mechanism and evidence | Remaining boundary |
|---|---|---|
| Run a scaffolder from an executable fence | The CLI ingestion witness now covers backtick and tilde fences as well as native cells. Ingestion expands the selected fence into a native cell with captured children. | Fence-only `language` and `exec` attributes are omitted during expansion. Literal Hick markup in the command is refused before execution rather than reinterpreted. |
| Keep captured files in the same document | Ingested files remain ordinary inline file constructions. The CLI witness copies only the document into an empty Git-free directory and weaves both files exactly. | Ingestion is one-shot; a second ingest is refused. Initial execution needs the declared toolchain. This does not promise every external service or tool can run offline. Binary scaffold ingestion remains refused; binary project entries are a separate supported construction. |
| Hide boilerplate on arrival | `DocumentEditor` installs `ingestedFolds`; each captured file starts folded. The block and directory folds are also available. The focused folding, rendered-cell and provenance-chip suites passed: 38 tests. | The default shows a file tree, rather than collapsing the entire scaffold into one line. No presentation change was necessary. No new full-app visual acceptance is claimed. |
| Explain changes on top of the scaffold | The pure-project witness retains a Cargo-shaped scaffold and applies an explicit patch after its producing cell. It compares the whole result with direct file constructions and refuses mismatched patch input. | A plain diff fence is display text. Use an explicit patch construction with declared predecessors. Binary and mode-only patches are unsupported; exact entries can carry that data. |
| Reorganize without changing the program | Complete tree comparison covers paths, bytes, kinds, executable status and symlink targets. The focused inventory witness passed. | Timestamps, ownership and all POSIX permission bits are outside the portable comparison. Source/output agreement alone does not compare two different revisions. |
| Use the compiled program on this repository | The native CLI provides `project materialize` and `project verify`; both use the shared project evaluator. | Whole-project weave with lineage conversion has a separately documented scale limit. Use the project commands for reconstruction and inventory verification. |

The missing implementation was fence ingestion: it previously searched for an XML closing tag even when the parser had produced an executable fence. The fix uses the parser's fence spans and the existing post-ingestion byte gate. Its code and refusal rationale are [maintained together](../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#f-4e4b92369bcd-ingest_from_exec). The [CLI witness](../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#f-6682c6282b3c-the_ingested_document_produces_the_scaffold_from_a_clone) exercises actual execution, capture and fresh reconstruction. The [pure-project witness](../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md#f-d5caa71df0d7-owned_ingest_is_editable_source_without_claiming_generator_execution) checks patch composition without executing the retained command.

## Execute the proposed first chapter

A local trial used the rebuilt CLI and this actual scaffolder command:

```sh
cargo init --name hickory_seed --vcs none --edition 2024 out
```

It ingested `Cargo.toml` and `src/main.rs`, replayed the single captured document in a fresh directory, applied an explained greeting patch, and materialized the resulting project. It then replaced the scaffold-plus-patch representation with direct constructions. Native `project verify` confirmed identical two-entry output, and `cargo run --offline` printed `Hello, Hickory!`. The local receipt is `target/dogfood/cargo-init-trial.json`; it includes both SHA-256 hashes and the scratch source location. This is a specific executed authoring trial, not a claim that every Hickory feature has been audited for external dependencies.

The first trial was correctly filtered out when placed under this repository's ignored `target/` directory. Running it outside the repository avoided inheriting that ignore policy. Ingestion's Git-ignore filter and project verification's explicit inventory policy serve different purposes.

## Keep implementation changes separate from the narrative refactor

Before the prerequisite changes, `behavior.py check` passed for 2,986 outputs and 3,005 Markdown sources. A local inventory at `target/dogfood/engineer-first-before.json` pins every output's hash, kind and executable bit. The prerequisite application changes are limited to scaffold ingestion and its two focused test files; all other original output bytes remain fixed apart from explicitly recorded maintenance metadata.

Native verification exposed two ambient build artifacts omitted from the entry's exclusions: `apps/web/tsconfig.tsbuildinfo` and `hickory-dogfood/tools/__pycache__/`. They are now explicitly excluded. Do not weaken verification to silently ignore arbitrary unconstructed files. The entry's thesis, reading order and chapter organization have not yet been refactored in this prerequisite change.

Build the modified implementation before recording the baseline for the narrative-only phase:

```sh
CARGO_TARGET_DIR="$PWD/target/dogfood" cargo build --locked -p hickory-cli --bin hick
CARGO_TARGET_DIR="$PWD/target/dogfood" cargo build --locked --release \
  --manifest-path hickory-dogfood/tools/behavior-engine/Cargo.toml
./target/dogfood/debug/hick project verify project.md
```

Freeze the complete post-prerequisite output tree outside the declared project inventory. Keep it immutable while changing the Markdown. Reconstruct the new Markdown in a fresh directory with the rebuilt CLI, compare against that frozen tree, and check exact paths, bytes, kinds and executable bits. Compare constructed outputs, not just Rust or TypeScript. Distinguish intentionally changed maintenance outputs from application bytes; do not silently exempt a difference. `behavior.py check` establishes agreement with the current source, while the independent frozen inventory establishes before/after equivalence.

## Executed whole-repository result

The rebuilt debug CLI materialized all 2,986 outputs into a fresh Git-free directory and verified the complete inventory there. An independent comparison of each reconstructed output against the working repository matched every hash, kind and executable bit. Against the pre-prerequisite inventory, 2,982 outputs are unchanged. The four deliberate differences are `ingest_exec.rs`, its CLI test file, the pure-project test file and the explanation ledger; no output paths were added or removed.

The frozen post-fix tree and CLI hash are recorded in `target/dogfood/engineer-first-ready.json`, together with the complete output inventory. The baseline directory is local scratch evidence and must survive until the narrative refactor comparison is complete. Copy it before comparing new Markdown; do not regenerate over the frozen outputs. In the copy, replace only `project.md` and `.hickory-source/` with the refactored source, then run the compiled `hick project verify project.md --root COMPARISON_DIRECTORY`. This evaluates the new narrative against the old output bytes without overwriting them. The command also rejects extra or missing inventory entries and portable metadata drift.

The rebuilt CLI's working-tree verification, maintained-source generation/check/audit, and formatting checks also passed. The final focused CLI and pure-project witnesses each executed one passing case. These are reconstruction and workflow checks; they do not establish the semantic correctness of all 2,986 outputs.

Continue to regenerate through the maintained source, then run `behavior.py check` and `behavior.py audit`. The ownership ledger still contains explicit explanation gaps; this readiness check does not make those explanations adequate or renew historical guarantees. Broad regression runs and post-commit monitoring remain with the maintainer (`just checks-status`).

## Immutable scaffold prerequisite

Captured scaffold output is now a read-only base. The existing `sha256` attribute is checked against sorted paths and exact UTF-8 bytes before replay or execution, including before include expansion can replace children. Nonliteral content, unsupported output metadata and mismatched hashes are refused. No sidecar or historical Git base is required. The fingerprint format is unchanged, so authentic original captures remain valid; an old capture whose bytes were hand-edited now fails explicitly rather than silently receiving a new hash.

The [shared integrity mechanism and its tests](../mechanisms/crates/hick-lang/src/scaffold.rs.md) distinguish immutable captures from keyed stdout recordings. Source editors protect the whole capture, including the hash and path attributes; namespace/raw-fence changes cannot make the base writable. Generated-output editors and LSP edit mapping reject base edits while retaining navigation, and authored patch text remains editable. Source and collaborative saves, reverse edits and direct agent source edits use the preservation rule as well, so an internal caller cannot evade the lock by editing both the bytes and the hash. A receiving room or server refresh can still reconcile authoritative source.

Ordinary standalone replay now bypasses path interpolation and substitutions for captured files. Its focused test keeps literal placeholders, BOM, CRLF and Unicode unchanged. The maintained [scaffolding example](../../examples/scaffolded-service.md) restores its original, authentic base hash and puts the recipient check in an explicit patch. Both its greeting and empty-recipient refusal ran successfully after single-document reconstruction.

Focused results: 38 frontend cases passed, including actual output-pane editing, browser-WASM integrity checking, folding, provenance and LSP boundaries. Nine Rust cases passed across the integrity, pipeline, lineage, source-save and CLI ingestion witnesses. The CLI witness verifies native/backtick/tilde ingestion, fresh replay and rejection of a tampered base without overwriting existing outputs. TypeScript compilation passed. Native CLI and browser WASM were rebuilt; all four browser artifacts matched their staged build. The literal Cargo trial still compiled and printed `Hello, Hickory!` after applying an authored patch over the captured scaffold.

New local receipts use `target/dogfood/immutable-*.log`, `immutable-example-trial.json` and `cargo-init-trial.json`. The refreshed whole-repository baseline is recorded by `engineer-first-ready.json`; the previous receipt and output tree remain separate. The first four-change inventory above describes the earlier prerequisite phase, not the additional implementation changes needed for immutability. Repository-specific dogfooding tools remain in this repository and were not moved into the product CLI. Scaffold refresh/rerunning remains deferred.

Changed Rust fragments were formatted. A recursive formatter also reported preexisting ordering/layout differences in unrelated serve modules; those were left unchanged. Broad regression tests and their monitoring remain with the maintainer.

</book:doc>
