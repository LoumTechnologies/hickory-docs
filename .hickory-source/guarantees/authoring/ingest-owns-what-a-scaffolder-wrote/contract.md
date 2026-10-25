<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Scaffolder's Output Becomes Bytes The Document Owns, Or The Ingest Is Refused

For: Authors editing executable documents and the files those documents produce.

Given an `<hick:exec>` cell whose command is wrapped in a `<hick:copy id=…>`
child and which mounts a volume declared with `output=`, when `hick ingest
--from '#id' <doc>.md` runs, then the cell's output volume is written into
the document as ordinary `<hick:file>` blocks inside an `<hick:ingested>`
element recording the run — and the document is verified to weave those exact
bytes before the ingest is accepted.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/ingest-owns-what-a-scaffolder-wrote.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
