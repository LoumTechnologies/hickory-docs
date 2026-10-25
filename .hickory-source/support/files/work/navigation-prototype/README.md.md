<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# work/navigation-prototype/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `ab4b95d99f96944528338d186deea696107b87dbe43b8c33c58cdbc413d4bf96`.

<book:entry path="work/navigation-prototype/README.md">
# Contextual tiling prototype

Refresh `preview.html` in the existing browser page. The current revision resets saved layouts from older prototype revisions; no manual cache clearing is needed. The center switches between an agent conversation and document search. Both use the same parallax passages, provenance brackets, minimap, and pins. There is no zoom mode.

**Passing → Edge crossing** is the new experiment. Each pin's amber top edge is the fixed trigger: when a linked passage's natural position crosses that edge, it moves through the outside corridor to the other side in a timed animation. The animation finishes even if scrolling stops, then returns to ordinary parallax. Crossing back triggers the return trip. **Continuous parallax** retains the previous scroll-following bypass for comparison.

Windows keep an eight-pixel gap and never overlap. Layout reserves both ends of a timed crossing and allows only one crossing per side at a time. Other passages use the remaining clear slots; surplus passages stay offscreen or wait until space becomes available. Resizing reallocates that space immediately.

In the middle reading band, a preview's actual linked line (or its collapsed header) stays within half a Files row of the source paragraph midpoint when space is clear; whole-row snapping takes priority over an exactly horizontal brace. Parallax offsets grow outside that band; pins and collision avoidance take priority over alignment. Near a list edge, catalogue rows use the frozen header's 36-pixel interval and only whole rows are shown, keeping neighboring filenames out from under the pin.

Passages start collapsed to a single filename row. Click a window filename to expand or collapse its excerpt. **Edges → Auto accordion** opts into expanding away from the top and bottom edges and collapsing near them. Clicking a window filename opens its passage. Up/down arrows reveal or hide three lines above or below independently; their labels include the hidden-line count.

Window controls sit at the upper right: **Center** (blue) makes that file the central source and brings its related files into parallax; **Restore** (amber) returns the previous center and its scroll position; **Minimize** collapses to the filename (click again to expand); the teal **pin icon** freezes or releases the view. Restore works through a stack of centered files.

Related filenames in the left and right Files lists extend the window headers: a continuous tinted strip and shared outline span the rail, gap, and window header in collapsed, expanded, pinned, and moving states. The strip follows the actual animated rectangle. Narrow passing views are briefly occluded while crossing an occupied header strip. Related filenames follow their passage views in parallax. Each filename keeps the same node, position, and dimensions when pinned or unpinned; only its pin styling changes. Pinned and unpinned filenames use identical alignment and row size. The underlying file catalogue scrolls independently. They remain fixed beside their window titles while other names scroll. Clicking a list filename pins or unpins that view. The catalogue and other passage labels retain their positions when pinning changes.

The side lists also experiment with **↑ N hidden** and **↓ N hidden**. Counts include unpinned passage filenames and catalogue rows outside that end of the visible list. Each click reveals the next hidden name. Passage headers move together with their windows, independently of central scrolling; pins remain fixed. The arrows use per-side offsets that continue to combine with source parallax.

Pinning and unpinning preserve all visible window positions, outer sizes, and content areas. Dotted resize dividers live in the gaps between windows, including unpinned views; windows contain no resize grips or bars. Resizing an unpinned view freezes it before applying the requested resize; dragging a view pins it when dropped. Creating a pin from a side file list uses free space without resizing existing pins.

Drag a window by its title or the tag on its sidebar label; clicking a line's line-number gutter pins or collapses it (the path text beside it is the editable buffer, so it takes the caret instead). The actual window moves to the nearest clear slot with a dashed, translucent tentative treatment. Dropping commits that exact rectangle and sidebar header position, preserving the other windows. Drag previews stay out of the center and pinned windows. Escape cancels placement. Dragging and scrolling the agent or other central source snap windows vertically to whole lines of the Files buffer. Each step eases into its row in a timed animation, which finishes even when dragging or scrolling stops; releasing a drag pins the target row while the animation completes. Headers, braces, and resize rules follow the moving window. Layout reserves the swept space until a step finishes, and collision avoidance takes priority when a route is blocked. Reduced motion uses immediate row snapping. A pin created by clicking or resizing still holds its exact current rectangle.

The sidebars replace the center Files pane. Each is a **single** editable numbered text buffer: one monospace line per entry showing its path relative to the sample open folder, a trailing `/` for a folder, and a fold marker in place of a file's line number. The directory part of a path is dimmed once the folder it names already has a line above it, so only the final name is normal text; a line moved above its own folder keeps that part normal too (`src/` still dims). The arrow keys walk between file names because the lines are one buffer, not separate inputs. **Edit a line's path** to rename or move it; editing a folder's path moves everything beneath it, and a name that is empty, has a doubled slash, or collides with another line is refused. The line-number/fold gutter is the control that pins or collapses; the path text is the buffer. A pinned header's filename is editable too, and the tag beside it pins or unpins. Editing either the buffer or the pinned header renames the same document, so the window title follows. **This is the Files buffer's text model**; the prototype still renders discrete rows rather than a real CodeMirror buffer, and every line shows the full path (which the narrow rail truncates) rather than an indented tree. Headers show paths relative to the sample open folder, **Retry workspace**; hover reveals a path if space truncates it. Each sidebar has an independent folder tree. Collapsing a folder hides its unpinned previews and collects provenance braces at the folder row. Pinned descendants retain their rectangles. Expanding restores individual file endpoints; clicking a collected brace opens that folder and reveals the passage. Catalogue rows flow around the attached headers so folder controls remain accessible.

The center pane is always present. **Left files** and **Right files** in the toolbar show or hide each side independently — the main pane with the left file list, with the right file list, or with both. Only the left is shown by default, so the main pane is wide and a single file list sits beside it; checking the other box brings that side's list and its windows back and the main pane yields the space. Each file list is resizable like every other divider: drag the dotted rule on its inner edge, or focus that rule and use the arrow keys (Shift for larger steps). The main pane takes whatever is left.

Dotted resize rules sit directly above and below each window. Adjacent windows share one rule: dragging it upward grows the lower window; dragging downward grows the upper window. Growth uses the gap first, then shifts the neighboring pin into free space while preserving its height. Unpinned passages reflow and never block that shift. Another pin or the lane edge limits available space; only after that space is exhausted does the neighbor shrink. Contracting an independent edge leaves neighbors in place and creates a gap. The dividers beside the center resize both adjoining columns. Focus a divider and use arrow keys (Shift for larger increments).

Blue text has no evidence of AI and is marked protected in this sample; teal text is AI-touched. These labels do not prove human authorship. **Propose agent edit…** shows a before/after dialog. Denying keeps the text; allowing changes only that passage and colors it AI-touched. A subsequent edit needs fresh permission. This is a simulation, not an authorization system connected to the editor.

Click or drag the minimap to navigate. It shows actual source blocks, linked passages/search matches, and pins. Sample stdout in the third agent turn traces to its print statement using execution brackets. All content and log mappings are samples; nothing runs or modifies real files.

The standalone demo fits the browser viewport, including short and narrow windows. Only the source, excerpts, and file lists scroll; the whole demo has no outer scrollbar.

`spatial-navigation.html` is the editable inline source. `preview.html` is generated from it by `build.cjs`, which splices the prototype into the fixed host wrapper in `preview.template.html` (the Codex visualization iframe, its theme variables and its tooltip/lucide bootstrap). Never edit `preview.html` by hand; rebuild instead:

```sh
node work/navigation-prototype/build.cjs
```

This remains an independent experiment, not a change to the app's panes.

Run with the web project's Playwright dependency and Chromium:

```sh
node work/navigation-prototype/check.cjs
```

Set `PROTOTYPE_SCREENSHOTS=1` to capture desktop and narrow-layout screenshots.
</book:entry>

</book:doc>
