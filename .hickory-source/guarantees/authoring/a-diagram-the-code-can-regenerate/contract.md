<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Diagram The Code Can Regenerate

For: Authors editing executable documents and the files those documents produce.

Given a folder of source code, when `hick diagram <path>` runs — at a
terminal, in CI, or inside a `hick:exec` cell, which is the intended home —
then a scene topology is deduced deterministically —
no model, no API key, no toolchain: tree-sitter compiled into the binary,
walking gitignore-aware — as nodes per file or per top-level directory and
edges from name-resolved references, **never** carrying a `layout`. Node
order is **path order, not walk order**: the same folder yields the same
topology on every machine, which is what makes an `<hick:expect>` around it a
pin rather than a record of one filesystem. And when
`hick diagram <path> --refresh <doc> --fragment <id>` runs, exactly that
`<hick:copy>` fragment's content is rewritten with the fresh topology, so a
derived diagram pasting it updates while the layout in the diagram's own body
is untouched by construction.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-diagram-the-code-can-regenerate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
