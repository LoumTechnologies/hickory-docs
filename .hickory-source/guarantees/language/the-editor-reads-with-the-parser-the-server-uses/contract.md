<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Editor Reads A Document With The Parser The Server Uses

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given any `.md` source — well formed or half typed — when the Document
view draws its structure (which spans are tags, which are blocks, where a
block's content starts and ends, and which prefix the document binds),
then that structure is what `hick-lang` reads: the same parser, the same
bytes, the same offsets. In particular a document that rebinds the prefix
(`<h:doc xmlns:h="…">`, `<slack:doc …>`) is drawn with *its* tags and its
literal `<hick:…>` text as prose; the content of a verbatim element
(`hick:input`, `hick:tool-result`, `hick:transcript`, `hick:reasoning`,
`hick:context`) is drawn as text however many tags it quotes; and a document
`hick run` would refuse is still drawn, with the first thing it would refuse
reported beside the structure.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/the-editor-reads-with-the-parser-the-server-uses.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
