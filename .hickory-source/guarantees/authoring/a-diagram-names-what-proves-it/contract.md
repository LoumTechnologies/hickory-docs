<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Diagram Names What Proves It

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:diagram>` in a document, when the document is run or woven,
then the diagram weaves into a fenced block tagged with its `renderer` — so
the woven markdown renders wherever markdown renders — and its `asserts`
attribute is checked against the document: a reference to an id no tag carries
is a warning naming the missing id, and a diagram with no `asserts` at all is
a warning saying that nothing will fail when the picture stops being true.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-diagram-names-what-proves-it.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `hick`

This is the maintained test fragment from `crates/hickory-cli/tests/diagrams.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-60028fe1d41a-hick"></a>
### `hick`

Diagrams, and the assertions that keep them honest.
Protects docs/guarantees/authoring/a-diagram-names-what-proves-it.md
A diagram is the one claim in a repository that nothing re-reads, so the
two things worth testing are: it weaves into something a reader can
actually see, and the binding between the picture and the cell that proves
it cannot rot silently.

<storycode:copy id="f-60028fe1d41a-hick">
//! Diagrams, and the assertions that keep them honest.
//!
//! Protects docs/guarantees/authoring/a-diagram-names-what-proves-it.md
//!
//! A diagram is the one claim in a repository that nothing re-reads, so the
//! two things worth testing are: it weaves into something a reader can
//! actually see, and the binding between the picture and the cell that proves
//! it cannot rot silently.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/DiagramPanel.tsx](../../../mechanisms/apps/web/src/components/DiagramPanel.tsx.md#output-80582538ea4e)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [apps/web/src/components/DiagramPanel.test.tsx](../../../mechanisms/apps/web/src/components/DiagramPanel.tsx.md#output-56af562b717e)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [crates/hickory-cli/tests/diagrams.rs](../../../mechanisms/crates/hickory-cli/tests/diagrams.rs.md#output-60028fe1d41a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
