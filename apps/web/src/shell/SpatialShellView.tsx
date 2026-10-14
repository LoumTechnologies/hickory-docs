import { PaneHeaderProvider, PaneHeaderSlots } from "./PaneHeader";
import { openCommandBar } from "./CommandBar";
import { useEffect, useMemo, useRef, useState, type PointerEvent } from "react";
import { EditorView } from "@codemirror/view";
import { bufferTarget } from "../lib/editorBufferTargets";
import { onRevealLine } from "../lib/revealLine";
import { samePath } from "../lib/paths";
import { useSpatialSides } from "../lib/spatialSides";
import { closeTab, focus, panes, split, type Pane, type Tab } from "./layout";
import { useShellKeys, type ShellViewProps } from "./ShellView";
import { type RibbonFile, type RibbonLink, type RibbonSource } from "./Ribbons";
import { SpatialController, type SpatialItem } from "./SpatialController";
import { SpatialFileEdges } from "./SpatialFileEdges";
import { readSpatialState, spatialize, spatialKey, spatialRegions, type Side } from "./spatialLayout";
import { spatialRelations } from "./spatialRelations";
import "./spatialShell.css";

interface Props extends ShellViewProps {
  sources: readonly RibbonSource[];
  files: readonly RibbonFile[];
  links: readonly RibbonLink[];
  folderRoot?: string;
}
interface Entry extends SpatialItem { tab?: Tab; pane?: Pane }
const storageKey = (folder?: string) => `hickory:spatial-layout:v1:${folder ?? "scratch"}`;
const load = (folder?: string) => { try { return readSpatialState(localStorage.getItem(storageKey(folder))); } catch { return readSpatialState(null); } };
const relative = (path: string, root?: string) => root && path.startsWith(`${root}/`) ? path.slice(root.length + 1) : path;

/** Each editor has one stable mount. Centering, pinning and parallax only change its rectangle. */
export function SpatialShellView(props: Props) {
  const sides = useSpatialSides();
  const root = useRef<HTMLDivElement>(null);
  const [scene, setScene] = useState(() => load(props.folderRoot));
  const [minimap, setMinimap] = useState({ scroll: 0, total: 1, viewport: 1 });
  const previousFolder = useRef(props.folderRoot), lastChat = useRef<string | null>(null), pendingSide = useRef<Side | null>(null);
  const columnGesture = useRef<{ side: Side; x: number; start: number } | null>(null);
  const previousTabs = useRef(new Set<string>());
  const layout = useMemo(() => spatialize(props.layout), [props.layout]);
  const all = panes(layout.root), trees = all.filter(pane => pane.tabs.some(tab => tab.kind === "tree"));
  const sourcePaths = new Map(props.sources.flatMap(source => { const key = source.view?.dom.closest<HTMLElement>("[data-spatial-key]")?.dataset.spatialKey; return key ? [[key, source.docPath] as const] : []; }));
  const entries: Entry[] = all.flatMap(pane => pane.tabs.filter(tab => tab.kind !== "tree").map((tab, index) => ({ key: spatialKey(tab), path: ["document", "generated", "file"].includes(tab.kind) ? tab.target : sourcePaths.get(spatialKey(tab)) ?? (tab.kind === "untitled" ? bufferTarget(tab.id) : tab.target), tab, pane, side: scene.views[spatialKey(tab)]?.side ?? (all.indexOf(pane) % 2 || index % 2 ? "right" : "left") })));
  for (const port of props.ports ?? []) if (!entries.some(entry => entry.key === port.id || entry.path === port.id.slice(port.id.indexOf(":") + 1))) entries.push({ key: port.id, path: port.id.slice(port.id.indexOf(":") + 1), side: scene.views[port.id]?.side ?? (entries.length % 2 ? "right" : "left"), port: true });
  const fallback = entries.find(entry => entry.tab?.kind === "chat") ?? entries.find(entry => entry.tab?.kind === "document") ?? entries.find(entry => entry.tab);
  const center = entries.find(entry => entry.tab && entry.key === scene.center) ?? fallback;
  const effective = { ...scene, center: center?.key ?? null, sides };
  const relations = spatialRelations(props.sources, props.files, props.links);
  const latest = useRef({ state: effective, entries, relations }); latest.current = { state: effective, entries, relations };
  const controller = useMemo(() => new SpatialController({ root: () => root.current, state: () => latest.current.state, items: () => latest.current.entries, relations: () => latest.current.relations,
    update: make => setScene(current => make({ ...current, center: latest.current.state.center })) }), []);
  useShellKeys(layout, props.onLayout, props.onSearch, props.onRequestCloseTab);
  useEffect(() => { if (layout !== props.layout) props.onLayout(layout); }, [layout, props.layout, props.onLayout]);
  useEffect(() => {
    if (previousFolder.current !== props.folderRoot) { previousFolder.current = props.folderRoot; const loaded = load(props.folderRoot); setScene(current => ({ ...loaded, center: loaded.center ?? current.center, stack: loaded.stack.length ? loaded.stack : current.stack })); }
  }, [props.folderRoot]);
  useEffect(() => { try { localStorage.setItem(storageKey(props.folderRoot), JSON.stringify({ version: 1, ...scene })); } catch { /* An unavailable cache leaves the workspace usable. */ } }, [scene, props.folderRoot]);
  useEffect(() => controller.start(), [controller]);
  useEffect(() => onRevealLine(path => { const entry = latest.current.entries.find(entry => samePath(entry.path, path)); if (entry && latest.current.state.views[entry.key]?.pin?.h === 36) controller.toggle(entry.key); setScene(current => entry ? { ...current, ...(latest.current.state.sides[entry.side] === false ? { center: entry.key, stack: current.center && current.center !== entry.key ? [...current.stack, current.center].slice(-24) : current.stack } : {}), views: { ...current.views, [entry.key]: { ...current.views[entry.key], side: current.views[entry.key]?.side ?? entry.side, expanded: true } } } : current); }), [controller]);
  useEffect(() => { controller.schedule(); });
  useEffect(() => {
    const strip = root.current?.querySelector<HTMLElement>(".spatial-center-tabs"), selected = strip?.querySelector<HTMLElement>('[aria-selected="true"]');
    if (!strip || !selected) return;
    if (selected.offsetLeft < strip.scrollLeft) strip.scrollLeft = selected.offsetLeft;
    else if (selected.offsetLeft + selected.offsetWidth > strip.scrollLeft + strip.clientWidth) strip.scrollLeft = selected.offsetLeft + selected.offsetWidth - strip.clientWidth;
  }, [center?.key, entries.length]);
  useEffect(() => {
    const hadTabs = previousTabs.current.size > 0;
    const freshTabs = entries.filter(entry => entry.tab && !previousTabs.current.has(entry.key));
    previousTabs.current = new Set(entries.filter(entry => entry.tab).map(entry => entry.key));
    const unnamed = freshTabs.find(entry => entry.tab?.kind === "untitled" && entry.pane?.id === layout.focus);
    if (unnamed) setScene(current => ({ ...current, center: unnamed.key, stack: current.center && current.center !== unnamed.key ? [...current.stack, current.center] : current.stack }));
    else if (!scene.center && center) setScene(current => ({ ...current, center: center.key }));
    const chat = entries.filter(entry => entry.tab?.kind === "chat").at(-1);
    if (chat && chat.key !== lastChat.current) { lastChat.current = chat.key; setScene(current => ({ ...current, center: chat.key, stack: current.center && current.center !== chat.key ? [...current.stack, current.center] : current.stack })); }
    const fresh = freshTabs.find(entry => entry.key !== center?.key);
    if (hadTabs && fresh && !sides[pendingSide.current ?? fresh.side]) setScene(current => ({ ...current, center: fresh.key, stack: current.center && current.center !== fresh.key ? [...current.stack, current.center].slice(-24) : current.stack }));
    if (fresh?.tab?.kind === "tool" && fresh.tab.target.startsWith("reading:")) setScene(current => ({ ...current, views: { ...current.views, [fresh.key]: { ...current.views[fresh.key], side: current.views[fresh.key]?.side ?? fresh.side, expanded: true } } }));
    if (fresh && pendingSide.current) { const side = pendingSide.current; pendingSide.current = null; setScene(current => ({ ...current, views: { ...current.views, [fresh.key]: { side, expanded: true } } })); }
  }, [layout]); // Only newly opened tabs affect the center; changing presentation does not.
  useEffect(() => {
    const el = root.current;
    const update = () => { const dom = el?.querySelector<HTMLElement>("[data-spatial-center=true] .cm-editor"), view = dom && EditorView.findFromDOM(dom); if (view) setMinimap({ scroll: view.scrollDOM.scrollTop, total: view.scrollDOM.scrollHeight, viewport: view.scrollDOM.clientHeight }); };
    el?.addEventListener("scroll", update, true); const timer = window.setInterval(update, 500);
    return () => { el?.removeEventListener("scroll", update, true); clearInterval(timer); };
  }, []);
  const centerOn = (entry: Entry) => {
    controller.preserveSelections();
    if (!entry.tab || !entry.pane) { openPort(entry); return; }
    setScene(current => ({ ...current, center: entry.key, stack: center && center.key !== entry.key ? [...current.stack, center.key].slice(-24) : current.stack }));
    props.onLayout(focus(layout, entry.pane.id));
  };
  const restore = () => { controller.preserveSelections(); setScene(current => { const stack = [...current.stack]; let key: string | undefined; while ((key = stack.pop())) if (entries.some(entry => entry.key === key && entry.tab)) return { ...current, center: key, stack }; return current; }); };
  const openPort = (entry: Entry) => {
    pendingSide.current = entry.side;
    const tree = trees[entry.side === "left" ? 0 : 1] ?? trees[0];
    if (tree) props.onLayout(split(layout, tree.id, "row", entry.side === "right"));
    props.ports?.find(port => port.id === entry.key)?.onOpen();
  };
  const click = (action: () => void) => { if (!controller.suppressClick) action(); };
  const begin = (entry: Entry, event: PointerEvent, kind: "drag" | "top" | "bottom" = "drag") => { if (entry.key !== center?.key && !(event.target as HTMLElement).closest(".spatial-window-controls")) controller.begin(entry.key, event, kind); };
  const close = (entry: Entry) => { if (!entry.tab || !entry.pane) return; if (props.onRequestCloseTab) props.onRequestCloseTab(entry.pane.id, entry.tab.id); else props.onLayout(closeTab(layout, entry.pane.id, entry.tab.id)); };
  return <div ref={root} className="spatial-shell" data-spatial-shell="true"
    onPointerMove={event => { const gesture = columnGesture.current; if (gesture && root.current) { const regions = spatialRegions(root.current.clientWidth, root.current.clientHeight, scene.columns, sides), inner = root.current.clientWidth - regions.leftRail.w - regions.rightRail.w, delta = (event.clientX - gesture.x) / inner * (gesture.side === "left" ? 1 : -1); setScene(current => ({ ...current, columns: current.columns.map((value, index) => index === (gesture.side === "left" ? 0 : 1) ? Math.max(.16, Math.min(.36, gesture.start + delta)) : value) as [number, number] })); } }}
    onPointerUp={() => { columnGesture.current = null; controller.finish(); }} onPointerCancel={() => { columnGesture.current = null; controller.finish(true); }}
    onKeyDown={event => { if (event.key === "Escape") controller.finish(true); }}>
    {trees.slice(0, 2).map((pane, index) => { const side: Side = index ? "right" : "left", tab = pane.tabs.find(tab => tab.kind === "tree")!;
      return <section key={tab.id} hidden={!sides[side]} className="shell-pane spatial-files" data-spatial-files={side} data-shell-pane-id={pane.id}
        onDoubleClickCapture={event => { if ((event.target as HTMLElement).closest("[data-tree-kind=file]")) { pendingSide.current = side; props.onLayout(split(layout, pane.id, "row", side === "right")); } }}>
        <button role="tab" aria-label={side === "left" ? "Files" : "Right file buffer"} aria-selected={layout.focus === pane.id} className="spatial-files-title" onClick={() => props.onLayout(focus(layout, pane.id))}>Files</button><SpatialFileEdges /><div className="shell-pane__body">{props.render(tab, pane)}</div>
      </section>;
    })}
    {entries.map(entry => { const tab = entry.tab, centered = entry.key === center?.key, name = relative(entry.path, props.folderRoot), title = tab?.title ?? props.ports?.find(port => port.id === entry.key)?.label ?? name;
      return <PaneHeaderProvider key={tab?.id ?? entry.key}><section hidden={!centered && !sides[entry.side]} id={centered ? "spatial-center" : undefined} className="shell-pane spatial-view" data-spatial-key={entry.key} data-spatial-side={entry.side} data-spatial-center={centered} data-expanded={centered || scene.views[entry.key]?.expanded || false}
        data-shell-pane-id={entry.pane?.id} data-shell-target={entry.path} onFocusCapture={() => { if (entry.pane && props.layout.focus !== entry.pane.id) props.onLayout(focus(layout, entry.pane.id)); }}>
        <header className="spatial-view-header" onPointerDown={event => begin(entry, event)}>
          {centered ? <div className="spatial-center-tabs" role="tablist" aria-label="Center views">
            {entries.filter(candidate => candidate.tab).map(candidate => {
              const candidateTab = candidate.tab!, selected = candidate.key === entry.key;
              const label = candidateTab.title ?? relative(candidate.path, props.folderRoot);
              return <button key={candidateTab.id} id={`spatial-tab-${candidateTab.id}`} type="button" role="tab" aria-label={label} aria-selected={selected} aria-controls={`spatial-content-${candidateTab.id}`} tabIndex={selected ? 0 : -1}
                className="spatial-center-tab" data-tip={relative(candidate.path, props.folderRoot)}
                data-shell-tab-kind={candidateTab.kind} data-shell-tab-target={candidate.path} data-shell-tab-source={selected ? candidate.path : undefined} data-shell-tab-buffer={selected ? bufferTarget(candidateTab.id) : undefined}
                onClick={() => centerOn(candidate)} onKeyDown={event => {
                  if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
                  event.preventDefault(); event.stopPropagation();
                  const tabs = entries.filter(item => item.tab), index = tabs.findIndex(item => item.key === candidate.key);
                  const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
                  centerOn(tabs[next]);
                  requestAnimationFrame(() => root.current?.querySelector<HTMLButtonElement>('.spatial-center-tabs [aria-selected="true"]')?.focus());
                }}>{label}{props.dirtyTabIds?.has(candidateTab.id) ? " *" : ""}</button>;
            })}
          </div> : <button type="button" role={tab ? "tab" : undefined} aria-label={title} aria-selected={tab ? layout.focus === entry.pane?.id : undefined} className="spatial-filename" data-tip={name} data-shell-tab-kind={tab?.kind} data-shell-tab-target={entry.path} data-shell-tab-source={entry.path} data-shell-tab-vertical={true} data-shell-tab-buffer={tab ? bufferTarget(tab.id) : undefined}
            data-ribbon-port={entry.port ? entry.key : undefined} onClick={() => click(() => { if (entry.port) openPort(entry); else if (!centered) { if (tab?.kind === "untitled") centerOn(entry); else controller.toggle(entry.key); } })}>{tab && !["document", "generated", "file"].includes(tab.kind) ? title : name || title}{tab && props.dirtyTabIds?.has(tab.id) ? " *" : ""}</button>}
          <PaneHeaderSlots />
          <div className="spatial-window-controls">
            {centered ? <><button data-tip="Search (Ctrl/Cmd+P)" aria-label="Open search" onClick={() => openCommandBar()}>⌕</button><button data-tip="Restore previous center" aria-label="Restore previous center" onClick={restore}>▣</button><button data-tip="Automatic accordion" aria-label="Automatic accordion" aria-pressed={scene.auto} onClick={() => setScene(current => ({ ...current, auto: !current.auto }))}>↕</button></>
              : <><button data-tip={scene.views[entry.key]?.pin ? "Unpin view" : "Pin view"} aria-label={scene.views[entry.key]?.pin ? "Unpin view" : "Pin view"} aria-pressed={!!scene.views[entry.key]?.pin} onClick={() => click(() => controller.pin(entry.key))}><svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" aria-hidden="true"><path d="M5 2h6l-1 5 3 3H3l3-3-1-5ZM8 10v5" /></svg></button>
                <button data-tip="Center" aria-label="Center" onClick={() => centerOn(entry)}>□</button><button data-tip="Minimize" aria-label="Minimize" onClick={() => { if (controller.frames.get(entry.key)?.h !== 36) controller.toggle(entry.key); }}>—</button></>}
            {tab && <button data-tip={`Close ${title}`} aria-label={`Close ${title}`} onClick={() => close(entry)}>×</button>}
          </div>
        </header>
        <div id={tab ? `spatial-content-${tab.id}` : undefined} className="shell-pane__body spatial-view-body">{tab && entry.pane ? props.render(tab, entry.pane) : <button className="spatial-open" onClick={() => openPort(entry)}>Open {title}</button>}</div>
      </section></PaneHeaderProvider>;
    })}
    {entries.map(entry => <div key={`strip-${entry.key}`} hidden={!sides[entry.side]} className="spatial-header-extension" data-spatial-strip={entry.key} onPointerDown={event => begin(entry, event)}>
      <button data-tip={`${relative(entry.path, props.folderRoot)} — click to pin or unpin`} className="spatial-file-tab" onClick={() => click(() => entry.port ? openPort(entry) : controller.pin(entry.key))}>{entry.tab && !["document", "generated", "file"].includes(entry.tab.kind) ? entry.tab.title ?? entry.tab.target : relative(entry.path, props.folderRoot)}</button>
    </div>)}
    {entries.flatMap(entry => (["top", "bottom"] as const).map(edge => <div key={`${entry.key}-${edge}`} role="separator" aria-label={`Resize ${relative(entry.path, props.folderRoot)} ${edge}`} aria-orientation="horizontal" tabIndex={0}
      hidden={!sides[entry.side]} className="spatial-resize-rule" data-spatial-resize={entry.key} data-resize-edge={edge} onPointerDown={event => { event.preventDefault(); controller.begin(entry.key, event, edge); }}
      onKeyDown={event => { if (event.key === "ArrowUp" || event.key === "ArrowDown") { event.preventDefault(); controller.resize(entry.key, event.key === "ArrowUp" ? -10 : 10, edge); } }} />))}
    {!center && <div className="spatial-empty">{props.empty}</div>}
    {(["left", "right"] as const).map(side => <div key={`column-${side}`} hidden={!sides[side]} className="spatial-column-rule" data-spatial-column={side} role="separator" aria-label={`Resize ${side} column`} aria-orientation="vertical" tabIndex={0}
      onPointerDown={event => { event.preventDefault(); event.currentTarget.setPointerCapture(event.pointerId); columnGesture.current = { side, x: event.clientX, start: scene.columns[side === "left" ? 0 : 1] }; }}
      onKeyDown={event => { if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); setScene(current => ({ ...current, columns: current.columns.map((value, index) => index === (side === "left" ? 0 : 1) ? Math.max(.16, Math.min(.36, value + (event.key === "ArrowLeft" ? -.01 : .01) * (side === "left" ? 1 : -1))) : value) as [number, number] })); } }} />)}
    <div className="spatial-minimap" data-tip="Scroll the center view" role="scrollbar" aria-label="Center minimap" aria-valuenow={Math.round(minimap.scroll)} aria-valuemin={0} aria-valuemax={minimap.total} aria-controls="spatial-center"
      onPointerDown={event => { const dom = root.current?.querySelector<HTMLElement>("[data-spatial-center=true] .cm-editor"), view = dom && EditorView.findFromDOM(dom), box = event.currentTarget.getBoundingClientRect(); if (view) view.scrollDOM.scrollTop = (event.clientY - box.top) / box.height * Math.max(0, minimap.total - minimap.viewport); }}>
      <i style={{ top: `${minimap.scroll / minimap.total * 100}%`, height: `${Math.min(100, minimap.viewport / minimap.total * 100)}%` }} />
      {relations.filter(relation => relation.fromPath === center?.path || relation.toPath === center?.path).map(relation => { const reverse = relation.toPath === center?.path, view = reverse ? relation.to : relation.from, range = reverse ? relation.toRange : relation.fromRange; if (!view || !range) return null; const line = view.state.doc.lineAt(Math.min(range[0], view.state.doc.length)).number; return <b key={relation.key} style={{ top: `${(line - 1) / view.state.doc.lines * 100}%` }} />; })}
    </div>
  </div>;
}
