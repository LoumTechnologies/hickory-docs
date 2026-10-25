<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Drawn Diagram Is Document Text

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:diagram renderer="graph">` whose body is a JSON scene, when the
reader drags a node, connects or re-plugs an edge, adds, renames, resizes, or
deletes a node, restyles a selection from the palette, or applies auto-layout
in the app's graph editor, then the finished gesture is
written back into the document's own bytes as the scene's canonical
serialization (stable key order, one node/edge/layout entry per line, integer
positions) as a user edit — so undo undoes it, collaborators receive it, and a
diff of a diagram edit is a readable diff. When the document is woven, the
scene is drawn as an SVG output file (content-named, referenced as an image
from the woven markdown) carrying the author's positions, sizes, shapes and
colours, a body that does not parse weaves as its raw JSON rather than
nothing, and a scene string containing the literal `</hick:` is refused at
commit with a message saying why.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-drawn-diagram-is-document-text.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
