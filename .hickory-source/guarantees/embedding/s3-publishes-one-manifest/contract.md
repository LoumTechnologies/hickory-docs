<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# S3 publishes one manifest

For: Host developers embedding the document editor without a native execution service.

Given an S3-compatible host callback that passes the adapter's conditional-write
and visible-ETag probe, a workspace batch uploads immutable content-addressed
objects and a manifest before conditionally updating its one head object.
Readers follow a single published manifest. A failed upload does not publish
part of a batch; a stale file revision or head cannot silently overwrite another
session's publication. Object reads verify content hashes; ETags remain opaque
concurrency tokens. No object revision claims git history or verified execution.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/s3-publishes-one-manifest.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
