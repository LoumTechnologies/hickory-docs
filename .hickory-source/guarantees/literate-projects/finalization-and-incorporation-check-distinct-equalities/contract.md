<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Check the agent's transition separately from maintained-source reconstruction

For: Maintainers reorganizing source while preserving the complete generated program.

An agent finishes B from captured A. Its explanation must reproduce that transition, while the maintained source must reproduce B from an empty tree.

## What must happen

Finalization checks apply(explanation, A) equals B. Incorporation separately checks generate(maintained source, empty tree) equals B. Both proposed and reviewed candidates are rechecked before publication, including destination source identity and completed code. Already-present B is not applied a second time. Removing source retires its live rooms so late persistence cannot recreate it. Captured editor and dirty-source identities remain part of the boundary.

## Limits of the evidence

Observed filesystem snapshots are not global transactions. Equal source trees do not prove that the explanation is accurate or that B behaves correctly.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Use the implemented built-in/Python/ACP capture and review-queue continuation. Remove obsolete pending-integration language while retaining the distinct equalities.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
