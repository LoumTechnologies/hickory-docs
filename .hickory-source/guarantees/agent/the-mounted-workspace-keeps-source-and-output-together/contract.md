<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A protected workspace save is validated against the revision it read

For: Authors using an agent to change documents while retaining review control.

Given an agent workspace frontend using `workspace_fs::Engine`, when a buffered
save replaces a source document or a generated text product, then the engine
checks the source revision and original bytes, maps product edits through the
existing lineage implementation, validates the candidate weave, and refuses a
save that cannot reproduce the proposed bytes before publishing the source.
The durable write and live-room update occur
under one room lock with a final source comparison; debounced room persistence
uses that lock too, so an older snapshot cannot finish saving over it.
A reverse save spanning several source documents is refused. Generated reads
are derived from the source view; they do not trust stale backing products.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-mounted-workspace-keeps-source-and-output-together.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
