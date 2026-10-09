# Contextual tiling prototype

Refresh `preview.html` in the existing browser page. The center switches between an agent conversation and document search. Both use the same parallax passages, provenance brackets, minimap, and pins. There is no zoom mode.

**Passing → Edge crossing** is the new experiment. Each pin's amber top edge is the fixed trigger: when a linked passage's natural position crosses that edge, it moves through the outside corridor to the other side in a timed animation. The animation finishes even if scrolling stops, then returns to ordinary parallax. Crossing back triggers the return trip. **Continuous parallax** retains the previous scroll-following bypass for comparison.

Windows keep an eight-pixel gap and never overlap. Layout reserves both ends of a timed crossing and allows only one crossing per side at a time. Other passages use the remaining clear slots; surplus passages stay offscreen or wait until space becomes available. Resizing reallocates that space immediately.

In the middle reading band, a preview's actual linked line (or its collapsed header) aligns with the source paragraph midpoint, producing a horizontal brace connection when space is clear. Parallax offsets grow outside that band; pins and collision avoidance take priority over alignment. Near a list edge, catalogue rows use the frozen header's 36-pixel interval and only whole rows are shown, keeping neighboring filenames out from under the pin.

Passages start collapsed to a single filename row. Click a window filename to expand or collapse its excerpt. **Edges → Auto accordion** opts into expanding away from the top and bottom edges and collapsing near them. Clicking a window filename opens its passage. Up/down arrows reveal or hide three lines above or below independently; their labels include the hidden-line count.

Window controls sit at the upper right: **Center** (blue) makes that file the central source and brings its related files into parallax; **Restore** (amber) returns the previous center and its scroll position; **Minimize** collapses to the filename (click again to expand); the teal **pin icon** freezes or releases the view. Restore works through a stack of centered files.

Related filenames in the left and right Files lists extend the window headers: a continuous tinted strip and shared outline span the rail, gap, and window header in collapsed, expanded, pinned, and moving states. The strip follows the actual animated rectangle. Narrow passing views are briefly occluded while crossing an occupied header strip. Related filenames follow their passage views in parallax. Each filename keeps the same node, position, and dimensions when pinned or unpinned; only its pin styling changes. Pinned and unpinned filenames use identical alignment and row size. The underlying file catalogue scrolls independently. They remain fixed beside their window titles while other names scroll. Clicking a list filename pins or unpins that view. The catalogue and other passage labels retain their positions when pinning changes.

The side lists also experiment with **↑ N hidden** and **↓ N hidden**. Counts include unpinned passage filenames and catalogue rows outside that end of the visible list. Each click reveals the next hidden name. Passage headers move together with their windows, independently of central scrolling; pins remain fixed. The arrows use per-side offsets that continue to combine with source parallax.

Pinning and unpinning preserve all visible window positions, outer sizes, and content areas. Dotted resize dividers live in the gaps between windows, including unpinned views; windows contain no resize grips or bars. Resizing an unpinned view freezes it before applying the requested resize; dragging a view pins it when dropped. Creating a pin from a side file list uses free space without resizing existing pins.

Drag a filename from a view or either side list. The actual window moves to the nearest clear slot with a dashed, translucent tentative treatment. Dropping commits that exact rectangle and sidebar header position, preserving the other windows. Drag previews stay out of the center and pinned windows. Escape cancels placement.

The sidebars replace the center Files pane. Headers show paths relative to the sample open folder, **Retry workspace**; hover reveals a path if space truncates it. Each sidebar has an independent folder tree. Collapsing a folder hides its unpinned previews and collects provenance braces at the folder row. Pinned descendants retain their rectangles. Expanding restores individual file endpoints; clicking a collected brace opens that folder and reveals the passage. Catalogue rows flow around the attached headers so folder controls remain accessible.

Dotted resize rules sit directly above and below each window. Adjacent windows share one rule: dragging it upward grows the lower window; dragging downward grows the upper window. Growth uses the gap first, then shifts the neighboring pin into free space while preserving its height. Unpinned passages reflow and never block that shift. Another pin or the lane edge limits available space; only after that space is exhausted does the neighbor shrink. Contracting an independent edge leaves neighbors in place and creates a gap. The dividers beside the center resize both adjoining columns. Focus a divider and use arrow keys (Shift for larger increments).

Blue text has no evidence of AI and is marked protected in this sample; teal text is AI-touched. These labels do not prove human authorship. **Propose agent edit…** shows a before/after dialog. Denying keeps the text; allowing changes only that passage and colors it AI-touched. A subsequent edit needs fresh permission. This is a simulation, not an authorization system connected to the editor.

Click or drag the minimap to navigate. It shows actual source blocks, linked passages/search matches, and pins. Sample stdout in the third agent turn traces to its print statement using execution brackets. All content and log mappings are samples; nothing runs or modifies real files.

The standalone demo fits the browser viewport, including short and narrow windows. Only the source, excerpts, and file lists scroll; the whole demo has no outer scrollbar.

`spatial-navigation.html` is the editable inline source; `preview.html` is its browser-ready snapshot. This remains an independent experiment, not a change to the app's panes.

Run with the web project's Playwright dependency and Chromium:

```sh
node work/navigation-prototype/check.cjs
```

Set `PROTOTYPE_SCREENSHOTS=1` to capture desktop and narrow-layout screenshots.
