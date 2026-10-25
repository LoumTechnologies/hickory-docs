<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep a patch edit as a draft until the whole construction still works

For: Maintainers reorganizing source while preserving the complete generated program.

Change an early patch so a later patch no longer applies. The edit must remain an invalid draft rather than silently publishing a half-valid project.

## What must happen

Editing authored patch code regenerates the patch against its captured predecessor and reevaluates the complete supplied closure. A downstream-breaking edit remains unchecked and cannot publish. Current source identity is checked before local publication; cross-document candidates use the native publication guard. Executable patch identities remain required for this editing surface. Old-side editing follows its declared permission rather than inheriting New-side authority.

## Limits of the evidence

Passing patch evaluation does not certify explanatory prose or runtime behavior. Actual native visual acceptance is separate from editor-controller tests.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the implemented editor/WASM/native-guard continuation into the contract and retire the old pending-wiring exclusion.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
