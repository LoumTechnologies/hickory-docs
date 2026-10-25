<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Literal files use native bytes

For: Host developers embedding the document editor without a native execution service.

Given an unconditional top-level literal `file` in a document,
when the browser materializes it for local debugging,
then the Rust parser, native opening-break operation and native dedent operation
produce its bytes. Each emitted segment points to exactly equal UTF-8 source
bytes, including Unicode, BOMs and normalized indented CRLF. Nonliteral file
children, includes, conditions, duplicate file declarations, and declared
project/patch/entry construction are refused rather than executed as an incomplete
program. Complete project generation uses the separate pure evaluator with its
full Markdown closure; this does not add browser container or native execution.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/literal-files-use-native-bytes.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
