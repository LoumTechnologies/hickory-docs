<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Collaborative Session Never Duplicates the Document

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document open in the collaborative editor, when the socket drops and
reconnects, when the last editor leaves and the room is re-created, or when
the server restarts, then the CRDT text still holds exactly one copy of the
document — the server resumes the document's stored operation history rather
than minting a rival copy of `docs.source`, and what is persisted back to
Postgres and git is the same single copy.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/collab-sessions-never-duplicate-the-document.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
