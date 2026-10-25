<maint:doc xmlns:maint="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/README.md

For: maintainers changing the literate workflow and reviewers checking migration evidence.

Edit this entry, regenerate, and commit its output with the Markdown. Historical evidence retains its original scope.

<maint:entry path="hickory-dogfood/README.md">
# Inspect the literate migration records

For maintainers reviewing how the behavior-organized source became canonical.

[The promotion record](PROMOTION.md) describes the root-layout checks and their limits.

The active project is now [../project.md](../project.md). Follow
[the maintainer workflow](../docs/developers/literate-workflow.md) to edit,
generate, check or recover it. `master` retains the existing build layout as
committed generated output; `pre-literate-dogfood` preserves the earlier layout.

`baseline.json` is the immutable original 2,829-file capture. `behavior-map.json`
records current constructor ownership. `promotion.json` identifies the recovery
commit, deliberate maintenance changes, and the additional upstream file.
`reconciliation/paragraph-ledger.json` accounts for 3,715 original paragraph
ranges; `original-contracts.json.gz` retains their exact source text without
requiring a historical checkout.

The [reconciliation report](reconciliation/REPORT.md), earlier validation records
and `focused-results.json` are evidence from the isolated rewrite. References to
`integrated/` and `workspace/` in historical records describe that earlier layout.
Those duplicate local snapshots and build logs are ignored. They are unnecessary
to reconstruct this repository from its maintained Markdown.

Code-model changes are deferred to
[issue #115](https://github.com/LoumTechnologies/hickory-docs/issues/115).
</maint:entry>
</maint:doc>
