# A Ribbon Crosses Documents, Because Lineage Does

Given a generated file some of whose bytes came from a document other than
the one that generated it — a Slack message that pastes a meeting turn from
a note two `hick:upstream` hops away — when that file is open in the app,
then the overlay draws a ribbon from those bytes to **that** document:
pane-to-pane when its editor is on screen, otherwise to its tab, its row in
the Files tree, or an "open here" port laid out in the divider; and clicking
the ribbon opens that document.

The reason: provenance already names the file each span indexes
(`docs/guarantees/lineage/included-spans-name-their-own-file.md`), and
`hick lineage` prints it. An overlay that dropped every origin outside the
focused document was answering "where did this come from" with "not from
here" for exactly the bytes people most want to trace — the quoted decision,
the number a finding carried, the sentence from the meeting.

Three properties hold it up:

1. **One source per document the outputs reach.** `RibbonOverlay` takes
   `sources`, the focused document first and then every document named by
   the focused document's outputs' provenance, resolved from the path the
   engine reports (absolute) to the path the app knows (tree-relative). A
   document that is open contributes its text and editor; one that is not
   contributes only its path and gets chrome-terminated ribbons.
2. **Terminals are the way there.** A document with no tab gets a port
   (`document:<path>`) in the divider, the same as the focused document gets
   when its own pane is closed; a ribbon ending on a tree row tints that row.
3. **Colour is per fragment per document.** The same byte span in two
   documents is two fragments and two colours.

## Boundary

The overlay still follows the **focused** document's outputs. Focusing the
meeting note shows the meeting's lineage, not the message's ribbons back to
it — provenance is recorded on the generated side. When the target document
was not on screen, the click opens it but the span is not yet selected in
the freshly mounted editor (the same limit the focused document's own
"reopen" path has). Provenance kinds (typed, transcribed, summarized,
executed) are still not drawn differently; see
`docs/specs/freeform/provenance-and-standing.md`.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified by driving the served UI (Playwright) against
  `examples/receipts/claude-code`: opening `messages/2026-08-24-eng-reply.txt`
  drew one ribbon titled "Open meetings/2026-08-20-checkout-latency-sync.hick —
  this text came from it" ending on the meeting's inactive tab; clicking it
  navigated to the meeting document. Screenshot:
  `examples/receipts/ribbons-across-documents.png`.
- Evidence: `apps/web/src/shell/Ribbons.tsx` (`sources`, `fragmentIn`,
  document targets carry `path`), `apps/web/src/views/WorkspaceView.tsx`
  (`ribbonSources`, `docIdByPath`, `openDocumentByPath`, document ports).
- Test coverage: `apps/web/src/lib/ribbons.test.ts` covers derivation; the
  multi-source overlay is not unit-tested (it measures DOM) — LLM/manual review.

Amended 2026-10-08: a Files-tree endpoint uses the **visible part** of its
row, clipped on both axes by every scrolling or clipping ancestor. Scrollport
bounds exclude borders and scrollbar space. A long row must not place the
endpoint behind the neighboring pane; horizontally scrolling the Files pane
keeps the endpoint on the visible row edge. A fully clipped row falls through
to the divider port. Scrolling the tree schedules an immediate overlay update.
This applies to lineage, context and declared links in both brace and band mode.

Requirement: the user's screenshot and clarification identified a connection
ending behind a neighboring pane when the Files pane was narrow. The initial
interpretation as excessive context connections was corrected; the proposed
Input-marker presentation was removed. The implemented correction preserves
existing ribbon visibility and styling. Assumption: ancestor client rectangles
are the visible bounds for the current, untransformed shell layout. Rotated or
scaled custom hosts are outside this verification; they are not used by the
shell. This fixes endpoint geometry, not the semantic validity of a citation.

Implementation review: Codex inspected `Ribbons::findTerminal`,
`visibleElementRect` and `attachTerminal`. Previously only vertical visibility
against the pane body was checked, and the full row rectangle was then used.
`visibleElementRect.test.ts` checks an overflowing 600px row in a 145px client
scrollport, horizontal scrolling, nested clipping, scrollbar exclusion and
fully hidden rows. Its DOM measurements are controlled unit fixtures. The
browser regression `literate-editor.spec.ts` separately uses the real Files
editor with horizontal overflow and measures the browser's actual client area.
Human judgment of the corrected native-app presentation remains pending.

Executed checks: 12 focused geometry/terminal tests passed; the real Chromium
Files-editor regression passed before and after horizontal scrolling;
TypeScript checking and the file-length check passed. The installed desktop
binary was not rebuilt. These results describe the implementation above and
must be rechecked when its clipping or endpoint calculations change.

Directional opening (2026-10-08): clicking any
brace or band opens its destination on the side indicated by the connection,
independently of the currently focused pane. An already-open destination is
activated in its existing pane. Otherwise the nearest visible editor in that
direction takes the new tab; panes currently showing Files, Agent or another
tool are not replaced. If no editor exists on that side, an editor pane is
created beside the source on that side. Horizontal and vertical arrangements
use the same rule, including Document | Files | Agent and its mirror. An
explicit connection destination takes precedence over ordinary focus-based or
region-based opening. Target line/span reveal remains part of navigation.

Requirement: the user reported a file opening over Agent after clicking its
connection, and asked for direction-based opening for **all** braces. The
implementation reads the source pane and the existing target tab or visible
file-tree/port endpoint before navigation changes focus. Direction is the
largest center displacement between the source pane and destination. Among
editors in that direction, distance and perpendicular separation choose the
nearest aligned editor. This is an explicit geometric interpretation; a person
has not yet judged ambiguous diagonal layouts. Existing targets are activated,
not moved or duplicated. Normal non-connection opens retain their prior routing.

Implementation review: `ribbonNavigation::{ribbonOpenIntent, prepareRibbonOpen}`
is called by the common `RibbonOverlay` click handler for every provenance
family and both rendering styles. `WorkspaceView` carries the selected pane
through plain-file, document (including asynchronous loads), and generated-file
openers. `layout::split` can place a new editor before or after its source.
The custom layout's Agent tab survives restoration via `uiState::VIEW_KINDS`;
it had been omitted from the accepted kinds. Routing tests cover mirrored and
vertical arrangements, ordinary editor provenance, preserved Agent tabs, and
creating a missing editor on each of four sides. Chromium fixture flows click
a declared connection to a closed file in three real restored arrangements.
These fixture sessions do not involve an authenticated model call. Human
judgment of the resulting native-app behavior remains pending.

Executed checks for directional opening: 166 focused frontend tests passed;
TypeScript and file-length checks passed. Five Chromium flows passed, including
the three directional arrangements, the Files clipping regression, and the
Agent-side brace stacking regression. The ACP fixture compiled successfully.
These checks apply to the inspected source; subsequent routing or layout
changes require rechecking. The installed desktop app was not rebuilt.
