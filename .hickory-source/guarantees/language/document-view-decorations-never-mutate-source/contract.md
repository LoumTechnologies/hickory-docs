<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
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


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/document-view-decorations-never-mutate-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/web/src/editor/embedded.ts](../../../mechanisms/apps/web/src/editor/embedded.ts.md#output-df9508b237a8)
- [apps/web/src/editor/hickDoc.ts](../../../mechanisms/apps/web/src/editor/hickDoc.ts.md#output-c2f7bc821e36)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)

### Tests and fixtures

- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/editor/embedded.test.ts](../../../shared/files/apps/web/src/editor/embedded.test.ts.md)
- [apps/web/src/editor/hickDoc.test.ts](../../../shared/files/apps/web/src/editor/hickDoc.test.ts.md)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
