<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Debug the latest atomic project that still contains the displayed bytes

For: Authors debugging generated code while navigating its document source.

A caller and library change belong to one atomic scope. Debugging must not combine the new caller with the old library simply because that intermediate state existed during editing.

## What must happen

A launch selects the latest supported complete project endpoint containing the exact displayed code. Candidate states honor atomic scopes; dependency-local editing states do not automatically become launch candidates. Changed source or unavailable exact mapping refuses selection. The launch profile chooses the executable within that selected tree, while breakpoint routing preserves the viewed source identity.

## Limits of the evidence

Source selection is separate from language-server endpoint choice and from a successful native launch. Exact-source checks must be repeated at the host boundary.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Use the atomic-prefix correction and exact-source launch-profile rule, replacing the earlier permissive cumulative-state interpretation.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
