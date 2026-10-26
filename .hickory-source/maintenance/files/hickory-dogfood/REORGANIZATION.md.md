<maintain:doc xmlns:maintain="http://www.hickorydocs.com/1.0">
# Maintain hickory-dogfood/REORGANIZATION.md

For: maintainers locating this exact input or preserving the migration evidence.

The entry below owns this output. Its metadata is separate from semantic verification.

<maintain:entry path="hickory-dogfood/REORGANIZATION.md">
# Read code and its explanation together

For maintainers reviewing the literate reorganization and choosing the next explanation to improve.

Start with [the source index](../project.md). [Exact patch replay](../.hickory-source/guarantees/literate-projects/replay-is-exact-and-atomic/contract.md)
now places the actual implementation and its annotated tests alongside the contract.
[Generated-file saves](../.hickory-source/mechanisms/apps/web/src/lib/outputSave.ts.md)
explain acknowledged baselines and serialized requests next to their code.

## What changed

The old contract/review/construction/file wrapper chains were consolidated into
behavior narratives. Larger behaviors use named implementation chapters; shared
code has a mechanism home. Arbitrary numbered batches were removed. 125 directly importing test modules now share their implementation narrative, while larger combinations retain named chapters.

The layout reorganizes 1,834 code outputs into 13,330
maintained fragments and literal constructions, with 184
annotated test fragments beside their contracts. Output assembly is explicit and
retains one construction per file. The requirement details, review notes and
paragraph dispositions remain linked. The new `behavior.py locate` command maps
an output path to its assembly and each fragment's narrative home.

## Explanation quality remains separate from ownership

3,899 fragment explanations retain existing source-comment rationale;
44 explanations were written from inspected implementation in this pass.
9,387 fragments still lack an adequate explanation. The ledger also lists
literal-file review work: 1,713 files have at least one recorded gap.

This is not a claim that every fragment is now well explained or every guarantee
has been semantically reconciled. A component home identifies responsibility;
it does not prove the code belongs to a particular product promise. Original
annotations choose test placement but do not establish test coverage.

Use `narrative-map.json` to choose a gap, read the implementation and its callers,
and replace the gap with a concrete explanation of its decision, invariant or
failure case. Keep the code beside that explanation. Do not add a generic
paragraph to clear an accounting check. Stable fragment identifiers describe
symbols; byte ranges record the pre-reorganization source positions as history.

## Byte and evidence boundaries

`reorganization-before.json` pins the 2,870 outputs at starting revision
`9461830eeee3b5c7f8cfe9c45edf3c1cdd060350`. Application implementation, tests, fixtures,
assets and executable modes must remain unchanged. Deliberate maintained-output
changes are limited to README, AGENTS, the developer workflow, the ownership map
and the maintenance audit/locator tool; new skill/evidence files are additions.

Regeneration, source agreement, paragraph/ownership checks and fresh Git-free
reconstruction are the relevant focused gates. Neither reconstruction nor moved
test annotations renew historical semantic verification. Broad regressions,
post-commit checks and GitHub status monitoring remain with the maintainer:

```sh
just checks-status
gh run list --limit 5
gh run view <run-id>
```

## Executed focused checks

On 2026-10-25, generation and `just literate-check` passed for 2,877 constructed
outputs, 2,964 Markdown inputs and 13,330 single-owner fragments. All local
editorial file links resolved, and all 3,715 original paragraph ranges retained
their dispositions. Heading fragments and semantic entailment were not checked.

`promotion-check.py --baseline` reconstructed the complete output inventory in an
empty Git-free directory. All 2,823 baseline outputs outside the six original
maintenance exceptions matched bytes, kinds and executable modes; the recorded
upstream addition matched. Its byte-drift, extra-output and outside-symlink
controls passed.

A separate comparison with `reorganization-before.json` found exactly the five
listed maintenance changes among the 2,870 starting outputs. The other 2,865
outputs, including application implementation and tests, retained bytes, kind
and executable metadata. Seven new skill, evidence and focused-test outputs are
additions rather than unexplained differences.

The four `test_behavior_audit.py` witnesses passed: several outputs can share a
narrative, while duplicate outputs, duplicate fragments and missing selected
fragments are refused. The two named Rust patch witnesses and the two curated
frontend generated-save assertions also passed through `behavior.py test`.
New runtime receipts remain local under `target/dogfood/focused-results.json`;
these results do not renew every historical verification claim.

The pre-dogfood recovery branch remains `pre-literate-dogfood`. Code-model
simplification remains deferred to [issue #115](https://github.com/LoumTechnologies/hickory-docs/issues/115).
</maintain:entry>
</maintain:doc>
