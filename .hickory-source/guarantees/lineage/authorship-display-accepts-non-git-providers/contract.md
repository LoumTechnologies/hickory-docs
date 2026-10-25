<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Authorship display accepts non-Git providers

For: Authors editing generated text and reviewers tracing a result to its source.

Given an attribution provider supplying line metadata, when author colors are
turned on in Lineage settings, then the editor distinguishes credited authors
without replacing syntax colors. Git blame is the default provider. A host can
install a replacement provider without Git. Recorded agent writes supplement
Git attribution using the existing strong session-write fingerprints. Session
user and assistant elements expose their recorded roles. Absent origin evidence
remains unknown; a Git identity alone never classifies text as human or agent.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
