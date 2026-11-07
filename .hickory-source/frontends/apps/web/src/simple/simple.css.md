<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep chrome compact and content stationary

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Use one fixed menu row, flexible content and one status row. Tabs and gutter widths remain stable when actions appear. Narrow screens retain all commands and allow scrolling the workspace rather than dropping capabilities.

<ui:copy id="simple-35d92a1eac-0">
* { box-sizing: border-box; }
:root { color-scheme: light dark; font: 13px system-ui, sans-serif; background: Canvas; color: CanvasText; }
body { margin: 0; }
button, textarea { font: inherit; color: inherit; }
button { border: 0; background: transparent; padding: 4px 8px; text-align: left; cursor: pointer; }
button:disabled { opacity: .45; cursor: default; }
button:hover:not(:disabled), summary:hover { background: color-mix(in srgb, CanvasText 8%, Canvas); }
button:focus-visible, summary:focus-visible, .resize:focus-visible { outline: 2px solid Highlight; outline-offset: -2px; }
main { height: 100dvh; display: grid; grid-template-rows: 29px minmax(0,1fr) 23px; }
nav { display: flex; align-items: center; border-bottom: 1px solid GrayText; padding: 0 8px; gap: 14px; }
nav details { position: relative; }
nav summary { list-style: none; cursor: pointer; padding: 3px; }
.menu { position: absolute; z-index: 20; top: 23px; left: 0; min-width: 190px; padding: 4px; background: Canvas; border: 1px solid GrayText; box-shadow: 0 4px 12px #0003; }
.menu button { display: block; width: 100%; white-space: nowrap; }
.app-name { margin-left: auto; color: GrayText; font-size: 11px; }
.workspace { min-height: 0; overflow: auto; }
.split { display: grid; width: 100%; height: 100%; min-width: 0; min-height: 0; }
.group { display: grid; grid-template-rows: 29px minmax(0,1fr); min-width: 0; min-height: 0; overflow: hidden; }
.tabs { display: flex; overflow: auto; background: color-mix(in srgb, CanvasText 4%, Canvas); border-bottom: 1px solid color-mix(in srgb, CanvasText 20%, Canvas); min-height: 29px; }
.tab { position: relative; display: flex; flex: 0 0 auto; border-right: 1px solid color-mix(in srgb, CanvasText 15%, Canvas); }
.tab button { white-space: nowrap; padding: 3px 7px; }
.tab[data-active=true] { background: Canvas; }
.tab[data-active=true]::after { content: ""; position: absolute; top: 0; left: 0; right: 0; height: 2px; background: Highlight; pointer-events: none; z-index: 1; }
.tab button { background: transparent; }
.pane, .document-body, .editor-host { min-height: 0; height: 100%; overflow: hidden; }
.document-body { display: flex; flex-direction: column; }
.editor-host { flex: 1; }
.resize { background: color-mix(in srgb, CanvasText 13%, Canvas); touch-action: none; }
.horizontal > .resize { cursor: col-resize; }
.vertical > .resize { cursor: row-resize; }
.file-tree { height: 100%; overflow: auto; padding: 5px; }
.file-tree ul { list-style: none; padding-left: 12px; margin: 0; }
.file-tree > ul { padding-left: 0; }
.file-tree button, .file-tree summary { white-space: nowrap; line-height: 22px; padding: 0 4px; }
.empty { margin: 16px; color: GrayText; }
footer { border-top: 1px solid GrayText; padding: 3px 9px; font-size: 11px; overflow: auto; white-space: nowrap; }
.agent-body { height: 100%; display: flex; flex-direction: column; }
.conversation { overflow: auto; flex: 1; padding: 0 12px; }
.message { white-space: pre-wrap; overflow-wrap: anywhere; }
.agent-body form { display: flex; gap: 4px; padding: 6px; border-top: 1px solid GrayText; }
.agent-body textarea { flex: 1; min-width: 0; resize: vertical; background: Canvas; border: 1px solid GrayText; }
.notice { padding: 6px; color: #b33; background: Canvas; max-height: 100px; overflow: auto; }
.terminal-body { height: 100%; display: flex; flex-direction: column; }
.terminal-screen { flex: 1; min-height: 0; overflow: auto; }
.terminal-screen pre { margin: 6px; font: 13px/17px ui-monospace, monospace; }
.terminal-body form { display: flex; padding: 6px; border-top: 1px solid GrayText; }
.terminal-body input { flex: 1; min-width: 0; font: 13px ui-monospace, monospace; background: Canvas; color: CanvasText; border: 1px solid GrayText; }
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/simple.css"><ui:paste select="#simple-35d92a1eac-0"/></ui:file>

</ui:doc>
