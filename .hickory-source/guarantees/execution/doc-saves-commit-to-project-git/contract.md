<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Doc Save Is a Commit in the Project's Git Repository

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a project, when a document is created, saved via `PUT /api/docs/:id`,
or persisted from the collaborative CRDT session, then the server writes the
file into the project's plain git repository under `GIT_DATA_DIR` and
commits it (one commit per save; unchanged content is a no-op) — git is the
durable truth for `.md` sources, and run checkouts are seeded from it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/doc-saves-commit-to-project-git.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
