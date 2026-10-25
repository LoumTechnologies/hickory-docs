<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Ingest Says When It Could Not Check `.gitignore`

For: Authors editing executable documents and the files those documents produce.

Given a directory that is not a git repository, when `hick ingest --from`
runs there, then it says on stderr that `.gitignore` filtering was skipped
and every produced file was ingested unfiltered — never silently, and never
indistinguishable from a real run that checked and found nothing to skip.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-ingest-says-when-it-could-not-check-gitignore.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
