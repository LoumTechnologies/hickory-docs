<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A write exposes its proposed effects

For: Authors editing executable documents and the files those documents produce.

Given Insert, New Project, an environment command, a filesystem Dry Run,
an agent proposal, or unsaved literate-view edits, when the person inspects the
operation, then proposed bytes or effects are visible before committing the
write. A preview writes nothing. Literate previews compare the arrangement and
changed backing files; closed previews do not mount extra editors. The write
keeps its existing concurrency checks and reports the result locally.
Ordinary typing still autosaves without a confirmation for every edit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-write-exposes-its-proposed-effects.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
