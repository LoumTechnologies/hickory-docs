<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Document-View Styling Never Mutates the Source

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given any `.md` document open in the web app's Document view (the
Typora-style WYSIWYG editor), when the view styles headings, inline markdown,
hick tag chrome, cell/file/fragment frames, conditional banners, paste chips,
inline directive tokens, embedded-language syntax highlighting inside block
bodies, or attaches cell/env/fragment/banner widgets, then the underlying
editor text is byte-for-byte the raw source: styling uses CodeMirror mark,
line, and block-widget decorations only — never text replacement — clicking a
paste chip only dispatches a selection, so cursor movement into any styled
region cannot corrupt the document, and malformed input (unclosed tags, stray
closers, arbitrary garbage) degrades to plain text without throwing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/document-view-decorations-never-mutate-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
