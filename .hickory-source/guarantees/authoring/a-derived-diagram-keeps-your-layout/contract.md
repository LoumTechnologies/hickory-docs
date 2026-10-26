<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Derived Diagram Keeps Your Layout

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:diagram renderer="graph">` whose body takes its topology
through a `<hick:paste>` (`{"topology": <hick:paste select="#…"/>, "layout":
{…}}`), when the reader arranges it in the graph editor, then commits rewrite
**only** the `layout` object and write the paste tag back byte-for-byte —
adding or removing nodes and edges is refused, and the panel says the topology
is derived — and when the fragment is regenerated, nodes whose ids survive
keep their recorded places, new ids appear beside the layout rather than over
it, and layout entries for departed ids are dropped on the next commit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-derived-diagram-keeps-your-layout.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/graph/GraphEditorPanel.tsx](../a-drawn-diagram-is-document-text/contract.md#output-b1b82d77ca55)
- [apps/web/src/components/graph/scene.ts](../../../mechanisms/apps/web/src/components/graph/scene.ts.md#output-3f46decc2b79)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hick-literate/src/scene.rs](../../../mechanisms/crates/hick-literate/src/scene.rs.md#output-34790cc193bc)

### Tests and fixtures

- [apps/web/src/components/graph/GraphEditorPanel.test.tsx](../../../mechanisms/apps/web/src/components/graph/GraphEditorPanel.test.tsx.md#output-b7404652ca7f)
- [apps/web/src/components/graph/scene.test.ts](../../../mechanisms/apps/web/src/components/graph/scene.ts.md#output-9d6e99c5706d)
- [crates/hickory-cli/tests/diagrams.rs](../../../mechanisms/crates/hickory-cli/tests/diagrams.rs.md#output-60028fe1d41a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
