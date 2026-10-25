<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Typed Text Is Prose, Not A Transcript

For: Authors editing executable documents and the files those documents produce.

Given text typed into the app's scratchpad, when it is saved, then it becomes a
note whose body is **prose** — no `<hick:transcript>`, no derived speaker turns,
no summary transforms — recording that it came from the scratchpad and, for
agent-assisted drafts, explicit agent-assisted authorship; and
when the text is empty, or contains hick markup, then it is refused with a
reason and the text is left where the person can act on it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/typed-text-is-prose-not-a-transcript.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
