<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Generated File Starts At Its First Byte

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given a `<hick:file path="…">` whose content begins on the line *after* the
open tag — which is how every readable document writes one — when the file is
produced, then its first byte is the first byte of the content, not the line
break that ended the tag's line.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
