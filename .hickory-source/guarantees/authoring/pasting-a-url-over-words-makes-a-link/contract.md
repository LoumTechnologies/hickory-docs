<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Pasting A URL Over Words Makes A Link

For: Authors editing executable documents and the files those documents produce.

Given a selection of prose in a document and a URL on the clipboard, when the
paste happens then the selected words become the label of a markdown link
pointing at the pasted address — `[the spec](https://example.com/spec)` — and
the caret lands after it. With nothing selected, or with something on the
clipboard that is not a URL, the paste is the ordinary paste it has always
been.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/pasting-a-url-over-words-makes-a-link.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/mdPaste.ts](../../../mechanisms/apps/web/src/editor/mdPaste.ts.md#output-722af83950b4)
- [apps/web/src/lib/mdLinks.ts](../../../mechanisms/apps/web/src/lib/mdLinks.ts.md#output-b86c148dc6ce)

### Tests and fixtures

- [apps/web/src/editor/mdPaste.test.ts](../../../mechanisms/apps/web/src/editor/mdPaste.ts.md#output-c4094026435c)
- [apps/web/src/lib/mdLinks.test.ts](../../../mechanisms/apps/web/src/lib/mdLinks.ts.md#output-9cdf9b2037e6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
