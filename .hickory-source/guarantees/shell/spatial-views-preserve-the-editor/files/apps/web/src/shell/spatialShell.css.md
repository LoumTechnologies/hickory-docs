<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/spatialShell.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../contract.md)

Captured SHA-256: `567498ed4c591c250ea218456c76466cc7b8399d73aa17d442a0e22b9017861a`.

<book:entry path="apps/web/src/shell/spatialShell.css">
.spatial-shell { position: relative; width: 100%; flex: 1; min-height: 0; overflow: hidden; isolation: isolate; }
.spatial-shell .spatial-view, .spatial-shell .spatial-files { position: absolute; margin: 0; min-width: 0; min-height: 0; overflow: hidden; border: 1px solid var(--border); background: var(--bg); display: flex; flex-direction: column; }
.spatial-shell .spatial-files { border: 0; z-index: 1; }
.spatial-files-title { height: 36px; min-height: 36px; padding: 8px; color: var(--fg-muted); font-size: 12px; border: 0; background: var(--bg); text-align: left; }
.spatial-shell .spatial-view { z-index: 2; }
.spatial-drawer-handle { position: absolute; top: 42px; z-index: 13; writing-mode: vertical-rl; border: 1px solid var(--border); border-radius: 0 5px 5px 0; padding: 8px 5px; background: var(--bg-raised); color: var(--fg-muted); cursor: pointer; }
.spatial-drawer-handle[data-drawer-handle=left] { left: 0; }
.spatial-drawer-handle[data-drawer-handle=right] { right: 0; border-radius: 5px 0 0 5px; }
.spatial-drawer-backdrop { position: absolute; z-index: 7; background: var(--bg); border: 1px solid var(--border); box-shadow: var(--shadow); pointer-events: none; }
.spatial-drawer-toolbar { position: absolute; z-index: 12; display: flex; align-items: center; gap: 6px; padding: 0 8px; box-sizing: border-box; background: var(--bg-raised); border-bottom: 1px solid var(--border); font-size: 12px; }
.spatial-drawer-toolbar strong { flex: 1; }
.spatial-drawer-toolbar button { border: 0; background: transparent; color: var(--fg-muted); cursor: pointer; }
.spatial-shell .spatial-view[data-drawer-side], .spatial-shell .spatial-files[data-drawer-side] { z-index: 9; }
.spatial-shell .spatial-header-extension[data-drawer-side], .spatial-shell .spatial-resize-rule[data-drawer-side] { z-index: 11; }
.spatial-view-header { height: 35px; min-height: 35px; display: flex; align-items: center; background: var(--bg); border-bottom: 1px solid var(--border); cursor: grab; overflow: hidden; }
.spatial-filename { flex: 1; min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; text-align: left; background: none; border: 0; color: var(--fg); font: inherit; font-size: 12px; padding: 6px 8px; cursor: pointer; }
.spatial-center-tabs { position: relative; display: flex; flex: 1; min-width: 0; height: 100%; overflow-x: auto; scrollbar-width: thin; }
.spatial-center-tab { flex: 0 0 auto; max-width: 12rem; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; padding: 5px 10px; background: transparent; border: 0; border-right: 1px solid var(--border); border-bottom: 2px solid transparent; color: var(--fg-muted); font: inherit; font-size: 12px; cursor: pointer; }
.spatial-center-tab[aria-selected=true] { color: var(--fg); border-bottom-color: var(--accent); background: var(--bg-raised); }
.spatial-center-tab:hover { color: var(--fg); background: var(--border); }
.spatial-center-tab:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
.spatial-window-controls { display: flex; flex: none; padding-right: 3px; }
.spatial-window-controls button { border: 0; background: transparent; color: var(--accent); padding: 4px; font-size: 14px; min-width: 22px; cursor: pointer; }
.spatial-window-controls button[aria-pressed=true] { color: #e7b75e; }
.spatial-window-controls button[aria-label=Minimize] { color: #78bbb1; }
.spatial-window-controls button[aria-label=Close] { color: #d78080; }
.spatial-window-controls button:hover { background: var(--border); }
.spatial-shell .spatial-view-body { overflow: hidden; visibility: hidden; pointer-events: none; }
.spatial-shell .spatial-view[data-expanded=true] .spatial-view-body { visibility: visible; pointer-events: auto; }
.spatial-shell .shell-pane__body > .tab-zoom { display: flex; flex-direction: column; min-height: 0; overflow: hidden; }
.spatial-shell .shell-pane__body > .tab-zoom > * { flex: 1; min-height: 0; }
.spatial-view--pinned { border-color: var(--accent) !important; }
.spatial-view--tentative { outline: 2px dashed var(--accent); outline-offset: -3px; opacity: .8; }
.spatial-view--crossing .spatial-view-body, .spatial-view--crossing .spatial-window-controls { visibility: hidden; }
.spatial-header-extension { position: absolute; z-index: 3; background: var(--bg); height: 36px; pointer-events: none; border-top: 1px solid var(--border); border-bottom: 1px solid var(--border); }
.spatial-header-extension::after { position: absolute; content: ""; top: 0; bottom: 0; width: calc(100% - var(--spatial-rail-width) - 30px); background: transparent; }
.spatial-header-extension[data-spatial-side=left] { background: linear-gradient(to right, var(--bg) var(--spatial-rail-width), transparent var(--spatial-rail-width)); }
.spatial-header-extension[data-spatial-side=right] { background: linear-gradient(to left, var(--bg) var(--spatial-rail-width), transparent var(--spatial-rail-width)); }
.spatial-file-tab { position: absolute; left: 0; width: var(--spatial-rail-width); height: 35px; pointer-events: auto; padding: 0 7px; text-align: left; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; background: var(--bg); color: var(--fg); border: 0; font: inherit; font-size: 12px; cursor: grab; }
.spatial-header-extension[data-spatial-side=right] .spatial-file-tab { left: auto; right: 0; }
.spatial-header--pinned, .spatial-header--pinned .spatial-file-tab { border-color: var(--accent); color: var(--accent); }
.spatial-header--tentative { border-top-style: dashed; border-bottom-style: dashed; opacity: .8; }
.spatial-resize-rule { position: absolute; height: 8px; z-index: 4; cursor: ns-resize; border: 0; background: transparent; touch-action: none; user-select: none; }
.spatial-resize-rule::after { content: ""; position: absolute; inset: 3px 0 auto; border-top: 1px dotted var(--fg-muted); opacity: .4; }
.spatial-resize-rule:hover::after, .spatial-resize-rule:focus::after { opacity: 1; border-color: var(--accent); }
.spatial-shell [hidden] { display: none !important; }
.spatial-shell .spatial-files .cm-scroller { overflow-x: hidden; }
.spatial-file-space { pointer-events: none; }
.spatial-excerpt-edge { display: flex; gap: 8px; padding: 4px 8px; background: var(--bg); color: var(--fg-muted); border-block: 1px dotted var(--border); }
.spatial-excerpt-edge button { color: var(--fg-muted); background: transparent; border: 0; cursor: pointer; font: inherit; }
.spatial-minimap { position: absolute; right: calc(50% - 3px); top: 40px; bottom: 0; width: 6px; z-index: 5; opacity: .35; cursor: pointer; }
.spatial-minimap i { position: absolute; left: 0; width: 100%; min-height: 6px; background: var(--accent); }
.spatial-minimap b { position: absolute; height: 2px; width: 6px; left: 0; background: #e7b75e; }
.spatial-minimap:hover { opacity: .9; }
.spatial-column-rule { position: absolute; z-index: 4; cursor: ew-resize; touch-action: none; }
.spatial-column-rule::after { position: absolute; content: ""; inset: 0 3px; border-left: 1px dotted var(--fg-muted); opacity: .4; }
.spatial-column-rule:hover::after { opacity: 1; border-color: var(--accent); }
.spatial-column-rule[hidden] { display: none; }
.spatial-empty { position: absolute; inset: 40% 30% auto; color: var(--fg-muted); }
.spatial-open { color: var(--accent); background: none; border: 0; }
.spatial-file-edges { display: flex; flex: none; font-size: 10px; justify-content: space-between; }
.spatial-file-edges button { background: transparent; border: 0; color: var(--fg-muted); padding: 2px 4px; cursor: pointer; }
.spatial-file-edges button:disabled { opacity: .4; cursor: default; }
.spatial-tab-item { display: flex; flex: 0 0 auto; min-width: 0; max-width: 14rem; border-right: 1px solid var(--border); }
.spatial-tab-item .spatial-center-tab { flex: 1; min-width: 0; border-right: 0; cursor: grab; touch-action: none; }
.spatial-tab-close { flex: none; border: 0; border-bottom: 2px solid transparent; background: transparent; color: var(--fg-muted); padding: 0 6px; cursor: pointer; }
.spatial-center-tab[aria-selected=true] + .spatial-tab-close { border-bottom-color: var(--accent); }
.spatial-tab-close:hover { color: var(--fg); background: var(--border); }
.spatial-drop-preview { position: absolute; z-index: 30; pointer-events: none; box-sizing: border-box; border: 2px solid var(--accent); background: color-mix(in srgb, var(--accent) 18%, transparent); display: grid; place-items: center; color: var(--accent); }
.spatial-view-header { touch-action: none; }
.spatial-drawer-toolbar button[aria-pressed=true] { color: #e7b75e; }
/* The filename is the Files editor's text, with syntax-like context coloring. */
.spatial-file-row { color: var(--accent); text-decoration: underline; text-decoration-style: dotted; text-underline-offset: 3px; }
.spatial-file-row--pinned { color: #e7b75e; text-decoration-style: solid; }
.spatial-shell .spatial-view[data-spatial-text-header=true] .spatial-filename { visibility: hidden; pointer-events: none; }
.spatial-shell .spatial-view[data-spatial-text-header=true] .spatial-view-header { background: var(--bg); border-bottom: 0; }
.spatial-drawer-toolbar { cursor: grab; touch-action: none; user-select: none; }
.spatial-drawer-toolbar:active { cursor: grabbing; }
.spatial-preview-gutter { width: 14px; color: var(--accent); }
.spatial-file-grip { display: inline-block; cursor: grab; touch-action: none; font-size: 10px; }
.spatial-file-grip[data-preview-pinned=true] { color: #e7b75e; }
/* Narrow Files panes share one column: text name, controls, then content. */
.spatial-shell .spatial-view[data-files-below=true][data-spatial-text-header=true] { background: transparent; pointer-events: none; }
.spatial-shell .spatial-view[data-files-below=true][data-spatial-text-header=true] .spatial-view-header { background: transparent; pointer-events: none; }
.spatial-shell .spatial-view[data-files-below=true][data-spatial-text-header=true][data-expanded=true] .spatial-view-header { height: 64px; min-height: 64px; align-items: flex-end; }
.spatial-shell .spatial-view[data-files-below=true] .spatial-window-controls,
.spatial-shell .spatial-view[data-files-below=true] .pane-header-slots { background: var(--bg); pointer-events: auto; }
.spatial-shell .spatial-view[data-files-below=true] .spatial-view-body { background: var(--bg); }
.spatial-header-extension[data-files-first=true] { background: linear-gradient(to right, var(--bg) var(--spatial-rail-width), transparent var(--spatial-rail-width)); }
.spatial-header-extension[data-files-first=true] .spatial-file-tab { left: 0; right: auto; }
.spatial-column-rule[data-drawer-side] { z-index: 12; }
</book:entry>

</book:doc>
