<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A New Project Uses The Toolchain The Language Uses

For: Authors editing executable documents and the files those documents produce.

Given a machine with more than one scaffolder installed, when File → New
Project is opened, then it offers each one — `.NET`, `Python (uv)`,
`Rust (cargo)` — and every entry in that picker is a tool this machine
actually has, measured by running it, never a list written down. A machine
with one scaffolder draws no picker and looks exactly as it did.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
