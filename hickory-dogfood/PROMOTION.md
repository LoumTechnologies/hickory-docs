# Review the move to canonical literate source

For maintainers assessing the migration on 2026-10-25 and choosing its next checks.

The active entry is now [project.md](../project.md). Its guarantee pages contain
the maintained implementation and tests. Ordinary build files remain committed
generated outputs. Follow [the workflow](../docs/developers/literate-workflow.md)
for an edit and regeneration.

The recovery branch `pre-literate-dogfood` points to `a2e482f9df644338aa8015f7bb12ada1e34de88a`, also recorded in `promotion.json`.
It includes the previously uncommitted release work and the newer upstream debug
example. `master` remains the default branch. No application implementation was
intentionally changed during promotion.

## Executed checks

- `behavior.py audit`: unique construction owners, local editorial file links,
  all 334 original guarantee/evidence file dispositions, and 3,715 original
  paragraph ranges across 326 contracts passed. Heading fragments and semantic
  equivalence are outside this audit.
- `behavior.py check`: constructed output bytes, kinds and Unix executable modes
  agreed with the maintained Markdown.
- `promotion-check.py --baseline`: an empty, Git-free directory reconstructed
  the complete output inventory from Markdown only. All 2,823 original outputs
  outside the six maintenance exceptions matched the immutable capture; the newer
  upstream file matched its separately recorded hash. Byte drift and an extra
  output were rejected. Native generation refused an outside symlink and left
  the outside sentinel unchanged.
- Two exact Rust witnesses passed through the root-layout test runner:
  `patch_preserves_bom_crlf_unicode_and_untouched_origins` and
  `newline_markers_add_delete_rename_and_failure_are_atomic`.
- One exact frontend assertion passed against the root application source:
  `serializes overlapping saves so each diff sees the previous result` in
  `apps/web/src/lib/outputSave.test.ts`. Eight other cases in that file were
  skipped. Existing installed dependencies were used.

The six deliberate maintenance changes are enumerated in `promotion.json`:
README, agent instructions, specification-location instructions, Git ignores,
task-runner commands, and a focused literate-source CI job. The source closure
also constructs the new maintenance tools and records; its own Markdown inputs
remain directly maintained.

The adapter was built from this repository's unchanged Hickory crates. Its loader,
evaluator and safe writer are Hickory features. It avoids the existing full-weave
lineage-parsing scale limitation, which remains unresolved.

## Remaining judgment

These results establish construction agreement and selected safety behaviors.
They do not renew the historical verification claims, settle the contracts still
awaiting semantic review, or certify desktop/release readiness. Earlier isolated
receipts retain their original scope. Broad regression tests, asynchronous local
checks and GitHub Actions monitoring remain with the maintainer:

```sh
just checks-status
gh run list --limit 5
gh run view <run-id>
```

Code-model simplification is deferred to the single backlog item,
[issue #115](https://github.com/LoumTechnologies/hickory-docs/issues/115).
