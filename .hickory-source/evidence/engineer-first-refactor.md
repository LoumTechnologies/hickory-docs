<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Preserve the program while changing its construction story

For engineers reviewing the engineer-first source refactor.

## Separate implementation from reorganization

Commit `8f20b8b6` adds immutable scaffold capture, its hash verification and the
focused native and editor checks. The subsequent reorganization changes maintained
Markdown only. Its baseline is that implementation commit, not the earlier
migration snapshot or the pre-immutability application.

[The entry](../../project.md) now starts with the product thesis, a real Cargo
scaffolding command and the workspace construction. The workspace chapter contains
the actual captured manifest plus the authored patch producing today's manifest.
Six engineering chapters bring implementation and relevant witnesses forward;
the remaining mechanism reference follows implementation layers. Current contracts
remain available alongside their decisions and through secondary review indexes.

Constructor and fragment owners retain their original paths. This keeps ownership
metadata, application source, generated documentation, assets and the browser Wasm
binary fixed. Include edges change the reading order without duplicating code or
silently removing historical requirements. The unchanged ledger still records
1,713 files with explanation review work; this is not a claim to have explained
every inherited fragment or to impose a prose/code quota on literal assets.

## Compare exact outputs

The rebuilt native CLI used for the prerequisite implementation is also the
refactor verifier. Its SHA-256 is
`e891b9b3aa6d5cf9faa34117daa66b5e8d59ef5d5dd0e42b3941b8b8319f4116`.
The local frozen inventory is `target/dogfood/engineer-first-ready.json`.

The comparison first replaces only the maintained Markdown in a copy of the frozen
output tree and runs `hick project verify project.md`. It then materializes a
separate Git-free directory containing only `project.md` and `.hickory-source/`.
Every reconstructed output is compared independently with the frozen inventory
for path, SHA-256, file kind and portable executable status. Symlink identity uses
the exact target bytes. The old tree is never regenerated to make it agree.

The generated-file comparison can also be repeated directly from Git:

```sh
git diff --exit-code 8f20b8b6 -- . \
  ':(exclude)project.md' ':(exclude).hickory-source/**'
python3 hickory-dogfood/tools/behavior.py check
python3 hickory-dogfood/tools/behavior.py audit
```

Git equality checks the committed output representation; reconstruction checks
that the new Markdown still produces it. Neither establishes behavior beyond the
unchanged bytes. The baseline commit and maintained source remain the durable
review inputs; local scratch directories and receipts are optional evidence.

## Exercise the new workspace construction

The workspace chapter was copied alone to an empty directory and given a project
declaration. Native materialization reproduced the repository's exact `Cargo.toml`
without any sidecar, transcript cache or Cargo execution. A changed captured byte
and a patch with incorrect context were refused without modifying the existing
manifest. Initial ingestion used the actual host Cargo toolchain through the local
executor; no container-image reproducibility is claimed.

The local comparison receipt is `target/dogfood/engineer-first-refactor.json`.
Broad regressions and full-app visual acceptance are outside this source-only
comparison. Post-commit checks remain asynchronous; maintainers can monitor them
with `just checks-status`.
</book:doc>
