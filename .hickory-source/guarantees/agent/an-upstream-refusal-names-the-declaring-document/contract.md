<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Upstream Refusal Names The Declaring Document

For: Authors using an agent to change documents while retaining review control.

Given an edit to a generated file whose bytes were pasted from a fragment
declared in an **upstream** document, when `edit_output` refuses that edit as
unreproducible, then the refusal names the document where the fragment is
**declared** — not the document where it is pasted — and shows the fragment,
so the next call needs no search.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-upstream-refusal-names-the-declaring-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
