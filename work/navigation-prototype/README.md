# Spatial navigation prototype

Open `preview.html` in a browser. It is a standalone experiment with an agent above Files in the center, linked documents on either side, and Hickory-style provenance brackets. Refresh the same browser page after updates.

Documents start as filenames. Click a filename or its `+` to expand a passage. The ellipses above and below reveal three more lines at a time; `− Above` and `− Below` shrink each end independently. These are windows onto passages rather than folds in the document's syntax.

Pin freezes the view's current position, including an animation in progress. Unpin resumes parallax from that position without a jump. Neither action changes sides, rearranges other documents, or changes the zoom. Drag a pinned filename to move its view and drag the `↘` corner to resize it. Arrow keys on the filename move a pinned view; arrow keys on the resize corner change its size. Hold Shift for larger increments. Drag a file from Files into the workspace to place and pin its view.

The **Filenames** selector compares horizontal and vertical arrangements of pinned names, or names only on the views. Clicking a name in this index reveals its existing view without replacing other documents. A filename is the common handle: Files introduces a view, a bracket locates its passage, parallax follows the conversation, and pinning keeps it in place.

The **Movement** selector retains conversation parallax, spatial zoom, and accordion alternatives. Sample stdout events lead to the print statement through execution brackets.

`spatial-navigation.html` is the editable visualization source; `preview.html` is its browser-ready snapshot. This prototype uses sample content and simulated log mappings, and does not connect to the app or a running agent.

With the web project's Playwright dependency and Chromium installed, run from the repository root:

```sh
node work/navigation-prototype/check.cjs
```

The check exercises stationary pin/unpin (including animation in progress), manual drag and pointer/keyboard resize, excerpt growth and shrinking, filename arrangements, conversation navigation, log-to-source tracing, brackets, zoom, movement styles, and a narrow viewport. Set `PROTOTYPE_SCREENSHOTS=1` to also capture desktop and mobile screenshots.
