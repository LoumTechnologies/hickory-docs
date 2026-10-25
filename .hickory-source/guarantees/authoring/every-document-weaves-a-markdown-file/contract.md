<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Document Weaves A Markdown File Of Its Own Name

For: Authors editing executable documents and the files those documents produce.

Given a `.md` document loaded from a path, when it is run or woven, then it
writes a markdown file of its own name beside it — `notes/standup.md` weaves
`notes/standup.md` — unless it names a different one with `weave=`, or opts out
entirely with `weave="none"`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/every-document-weaves-a-markdown-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
