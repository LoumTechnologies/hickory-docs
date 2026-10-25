<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Use the adapter’s transport and the current measured language support

For: maintainers and authors selecting the current editor or debugger behavior.

Start an adapter that listens on TCP rather than reading stdin. Connection transport and language support must come from the current catalogue, not a dated installation note.

## What must happen

A server-style adapter starts with its declared port placeholder substituted and uses DAP over the selected TCP connection; pipe-style adapters retain their declared transport. Discovery carries required launch metadata. Current JavaScript/TypeScript discovery and live-suite declarations include multi-session support; the old claim that they cannot be driven is historical. React component files are not standalone Node launch programs, and Ruby remains unavailable in the current discovery guidance. Finding an executable or installing an archive alone does not establish a live debugging witness.

## Limits of the evidence

The source inspection does not run a real adapter. Catalogue/live-suite agreement is distinct from a current machine having that adapter installed; no installer or network operation was executed.

## Inspect implementation and tests

[Open implementation and witnesses](review.md).

<book:include file="review.md" />

Requirement decision: Replace the obsolete JavaScript/TypeScript unavailability paragraph with the current shared plugin discovery and live coverage declarations. Keep TCP transport and measured support as distinct conditions.

Original wording: [preserved history](../../../support/files/docs/guarantees/debugging/an-adapter-that-listens-is-connected-to.md.md).

Implementation review is not verified. Executed checks and human acceptance remain separate.

## Source constructed here

[Construction inputs](construction.md).
<book:include file="construction.md" />

</book:doc>
