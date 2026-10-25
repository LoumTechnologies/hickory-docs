<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A frame stays in document coordinates

For: Authors debugging generated code while navigating its document source.

Given a woven file with document lineage, when a debug adapter reports a frame
through a filesystem alias or symlink, then the frame still resolves to the
owning document and source line. Canonical spellings such as macOS `/private/var`
and `/var` name the same file. Files without document lineage keep their own file
coordinates. This lookup does not change breakpoint placement or file bytes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-frame-stays-in-document-coordinates.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
