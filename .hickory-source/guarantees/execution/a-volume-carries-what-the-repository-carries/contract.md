<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Apply repository exclusions when seeding execution inputs

For: Authors running documents on their own machine and reviewers assessing their side effects.

Mount the repository as a volume. Ignored build outputs must not become accidental inputs, and a cell's own transcript must not make its next recording unreachable.

## What must happen

Repository volume seeding applies the repository's intended ignore filtering. Recording input digests additionally exclude the document's unstable generated products and weave targets. Literal authored inputs remain inputs. Removing the old self-mount warning does not remove ignore filtering or authorize a blanket exclusion of everything under an output directory.

## Limits of the evidence

Volume input filtering and executor confinement are separate boundaries. This does not police arbitrary filesystem writes by an unconfined process.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep the original ignore-filter promise and fold in the later removal of the obsolete self-mount warning. Delegate exact digest policy to the recording-key contract.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
