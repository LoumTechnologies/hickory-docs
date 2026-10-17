import { PRIMARY, dockTile, pruneTiles, tileRects, type DockEdge } from "./spatialTiles";
import { SPATIAL_RIBBON_OPEN, type SpatialRibbonOpen } from "./spatialNavigation";
import { PaneHeaderProvider, PaneHeaderSlots } from "./PaneHeader";
import { openCommandBar } from "./CommandBar";
import { useCallback, useEffect, useMemo, useRef, useState, type SetStateAction, type PointerEvent } from "react";
import { EditorView } from "@codemirror/view";
import { bufferTarget } from "../lib/editorBufferTargets";
import { onRevealLine } from "../lib/revealLine";
import { samePath } from "../lib/paths";
import { useFilesWindows } from "../lib/filesWindows";
import { useFilesParallax } from "../lib/filesParallax";
import { useSpatialSides } from "../lib/spatialSides";
import { OPEN_READING, readingStatus, useReadingsVersion } from "../lib/readingViews";
import { closeTab, focus, panes, split, type Pane, type Tab } from "./layout";
import { useShellKeys, type ShellViewProps } from "./ShellView";
import { type RibbonFile, type RibbonLink, type RibbonSource } from "./Ribbons";
import { SpatialController, type SpatialItem } from "./SpatialController";
import { SpatialFileEdges } from "./SpatialFileEdges";
import { readSpatialState, spatialize, spatialKey, type Side, type SpatialState } from "./spatialLayout";
import { drawerContent } from "./spatialDrawers";
import { spatialRelations } from "./spatialRelations";
import "./spatialShell.css";

interface Props extends ShellViewProps {
  sources: readonly RibbonSource[];
  files: readonly RibbonFile[];
  links: readonly RibbonLink[];
  folderRoot?: string;
  hasFiles?: boolean;
  spatialState?: SpatialState;
  onSpatialState?: (state: SpatialState) => void;
  spatialHydrated?: boolean;
}
interface Entry extends SpatialItem { tab?: Tab; pane?: Pane }
function PinIcon() { return <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" aria-hidden="true"><path d="M5 2h6l-1 5 3 3H3l3-3-1-5ZM8 10v5" /></svg>; }
const storageKey = (folder?: string) => `hickory:spatial-layout:v1:${folder ?? "scratch"}`;
const load = (folder?: string) => { try { return readSpatialState(localStorage.getItem(storageKey(folder))); } catch { return readSpatialState(null); } };
const relative = (path: string, root?: string) => root && path.startsWith(`${root}/`) ? path.slice(root.length + 1) : path;
const paneId = (entry: Entry) => `spatial-pane-${entry.tab?.id ?? encodeURIComponent(entry.key)}`;

/** Each editor has one stable mount. Centering, pinning and parallax only change its rectangle. */
export function SpatialShellView(props: Props) {
  const sides = useSpatialSides();
  const filesParallax = useFilesParallax();
  const filesWindows = useFilesWindows();
  const parallaxRef = useRef(filesParallax); parallaxRef.current = filesParallax;
  useReadingsVersion();
  const root = useRef<HTMLDivElement>(null);
  const [localScene, setLocalScene] = useState(() => load(props.folderRoot));
  const scene = props.spatialState ?? localScene;
  const sceneRef = useRef(scene); sceneRef.current = scene;
  const updateRef = useRef(props.onSpatialState); updateRef.current = props.onSpatialState;
  const setScene = useCallback((update: SetStateAction<SpatialState>) => {
    const next = typeof update === "function" ? update(sceneRef.current) : update;
    sceneRef.current = next;
    setLocalScene(next);
    updateRef.current?.(next);
  }, []);
  const [drawerOpen, setDrawerOpen] = useState({ left: false, right: false });
  const [requestedFiles, setRequestedFiles] = useState({ left: false, right: false });
  const [minimap, setMinimap] = useState({ scroll: 0, total: 1, viewport: 1 });
  const previousFolder = useRef(props.folderRoot), lastChat = useRef<string | null>(null), pendingSide = useRef<Side | null>(null);
  const columnGesture = useRef<{ side: Side; x: number; start: number } | null>(null);
  const previousTabs = useRef(new Set<string>());
  const ribbonRequest = useRef<SpatialRibbonOpen | null>(null);
  const dockHandler = useRef<(key: string, target: string, edge: DockEdge) => void>(() => {});
  const pendingReading = useRef<string | null>(null);
  const layout = useMemo(() => spatialize(props.layout), [props.layout]);
  const all = panes(layout.root), trees = all.filter(pane => pane.tabs.some(tab => tab.kind === "tree"));
  const sourcePaths = new Map(props.sources.flatMap(source => { const key = source.view?.dom.closest<HTMLElement>("[data-spatial-key]")?.dataset.spatialKey; return key ? [[key, source.docPath] as const] : []; }));
  const entries: Entry[] = all.flatMap(pane => pane.tabs.filter(tab => tab.kind !== "tree").map((tab, index) => ({ key: spatialKey(tab), path: ["document", "generated", "file"].includes(tab.kind) ? tab.target : sourcePaths.get(spatialKey(tab)) ?? (tab.kind === "untitled" ? bufferTarget(tab.id) : tab.target), tab, pane, side: scene.views[spatialKey(tab)]?.side ?? (all.indexOf(pane) % 2 || index % 2 ? "right" : "left") })));
  for (const port of props.ports ?? []) if (!entries.some(entry => entry.key === port.id || entry.path === port.id.slice(port.id.indexOf(":") + 1))) entries.push({ key: port.id, path: port.id.slice(port.id.indexOf(":") + 1), side: scene.views[port.id]?.side ?? (entries.length % 2 ? "right" : "left"), port: true });
  for (const entry of entries) if (!scene.views[entry.key]?.side && !sides[entry.side]) { const other = entry.side === "left" ? "right" : "left"; if (sides[other]) entry.side = other; }
  const activePane = all.find(pane => pane.id === layout.focus);
  const fallback = entries.find(entry => entry.tab?.id === activePane?.tabs[activePane.active]?.id) ?? entries.find(entry => entry.tab?.kind === "document") ?? entries.find(entry => entry.tab);
  const center = entries.find(entry => entry.tab && entry.key === scene.center) ?? fallback;
  const tiles = pruneTiles(scene.tiles ?? PRIMARY, key => key === PRIMARY || key !== center?.key && entries.some(entry => entry.tab && entry.key === key)) ?? PRIMARY;
  const tiledKeys = new Set(tileRects(tiles, { x: 0, y: 0, w: 1, h: 1 }).keys());
  useEffect(() => { if (JSON.stringify(scene.tiles ?? PRIMARY) !== JSON.stringify(tiles)) setScene(current => ({ ...current, tiles })); }, [scene.tiles, tiles]);
  const relations = spatialRelations(props.sources, props.files, props.links);
  const pending = new Set(entries.filter(entry => entry.tab?.kind === "tool" && entry.tab.target.startsWith("reading:") && readingStatus(entry.tab.target.slice(8)) === "pending").map(entry => entry.key));
  const contextual = filesWindows ? drawerContent(entries, center, relations, scene.views, pending, entries.filter(entry => entry.key === center?.key || tiledKeys.has(entry.key))).filter(entry => !tiledKeys.has(entry.key)) : [];
  const files = { left: !!trees[0] && (!!(props.hasFiles ?? trees.length) || requestedFiles.left), right: !!trees[1] && (!!(props.hasFiles ?? trees.length) || requestedFiles.right) };
  const contexts = { left: contextual.some(entry => entry.side === "left"), right: contextual.some(entry => entry.side === "right") };
  const available = { left: sides.left && (files.left || contexts.left), right: sides.right && (files.right || contexts.right) };
  const pinned = scene.drawerPins ?? { left: false, right: false };
  const visible = { left: available.left && (drawerOpen.left || pinned.left), right: available.right && (drawerOpen.right || pinned.right) };
  const effective = { ...scene, tiles, center: center?.key ?? null, sides: visible, drawers: { available, visible, files, contexts, pinned, keys: contextual.map(entry => entry.key) } };
  const latest = useRef({ state: effective, entries, relations, enabled: sides, windows: filesWindows }); latest.current = { state: effective, entries, relations, enabled: sides, windows: filesWindows };
  const controller = useMemo(() => new SpatialController({ root: () => root.current, state: () => latest.current.state, items: () => latest.current.entries, relations: () => latest.current.relations,
    parallax: () => parallaxRef.current,
    dock: (key, target, edge) => dockHandler.current(key, target, edge),
    update: make => setScene(current => make({ ...current, center: latest.current.state.center })) }), []);
  useShellKeys(layout, props.onLayout, props.onSearch, props.onRequestCloseTab);
  useEffect(() => { if (layout !== props.layout) props.onLayout(layout); }, [layout, props.layout, props.onLayout]);
  useEffect(() => {
    if (!props.onSpatialState && previousFolder.current !== props.folderRoot) { previousFolder.current = props.folderRoot; const loaded = load(props.folderRoot); setScene(current => ({ ...loaded, center: loaded.center ?? current.center, stack: loaded.stack.length ? loaded.stack : current.stack })); }
  }, [props.folderRoot]);
  useEffect(() => { try { localStorage.setItem(storageKey(props.folderRoot), JSON.stringify({ version: 1, ...scene })); } catch { /* An unavailable cache leaves the workspace usable. */ } }, [scene, props.folderRoot]);
  useEffect(() => controller.start(), [controller]);
  useEffect(() => {
    const openReading = (event: Event) => {
      const { id, background } = (event as CustomEvent<{ id: string; background?: boolean }>).detail;
      if (background) return;
      const key = `tool:reading:${id}`;
      pendingReading.current = key;
      if (latest.current.entries.some(entry => entry.key === key)) {
        pendingReading.current = null;
        controller.preserveSelections();
        setScene(current => ({ ...current, center: key, stack: current.center && current.center !== key ? [...current.stack, current.center].slice(-24) : current.stack }));
      }
    };
    window.addEventListener(OPEN_READING, openReading);
    return () => window.removeEventListener(OPEN_READING, openReading);
  }, [controller]);
  useEffect(() => {
    const el = root.current;
    const request = (event: Event) => {
      const detail = (event as CustomEvent<SpatialRibbonOpen>).detail;
      ribbonRequest.current = detail;
      const entry = latest.current.entries.find(item => samePath(item.path, detail.path));
      const side = detail.side && latest.current.enabled[detail.side] ? detail.side : latest.current.enabled.left ? "left" : "right";
      if (entry) requestAnimationFrame(() => { if (ribbonRequest.current === detail) ribbonRequest.current = null; });
      if (!detail.central && latest.current.windows && (latest.current.enabled.left || latest.current.enabled.right)) {
        setDrawerOpen(current => ({ ...current, [side]: true }));
        if (entry) setScene(current => ({ ...current, views: { ...current.views, [entry.key]: { ...current.views[entry.key], side, revealed: true, expanded: true } } }));
      } else if (!detail.central && entry) setScene(current => ({ ...current, center: entry.key, stack: current.center ? [...current.stack, current.center].slice(-24) : current.stack }));
    };
    el?.addEventListener(SPATIAL_RIBBON_OPEN, request);
    return () => el?.removeEventListener(SPATIAL_RIBBON_OPEN, request);
  }, []);
  useEffect(() => onRevealLine(path => {
    const entry = latest.current.entries.find(entry => samePath(entry.path, path)); if (!entry) return;
    if (ribbonRequest.current && samePath(ribbonRequest.current.path, path)) return;
    const inDrawer = latest.current.enabled[entry.side] && latest.current.state.drawers.keys.includes(entry.key);
    if (inDrawer) setDrawerOpen(current => ({ ...current, [entry.side]: true }));
    setScene(current => ({ ...current, ...(!inDrawer ? { center: entry.key, stack: current.center && current.center !== entry.key ? [...current.stack, current.center].slice(-24) : current.stack } : {}), views: { ...current.views, [entry.key]: { ...current.views[entry.key], side: entry.side, expanded: true } } }));
  }), [controller]);
  useEffect(() => { setDrawerOpen({ left: false, right: false }); setRequestedFiles({ left: false, right: false }); }, [center?.key, props.folderRoot]);
  const focusKey = `${layout.focus}:${entries.find(entry => entry.pane?.id === layout.focus)?.key ?? "tree"}`;
  const previousFocus = useRef(focusKey);
  useEffect(() => {
    if (previousFocus.current === focusKey) return;
    previousFocus.current = focusKey;
    const tree = trees.findIndex(pane => pane.id === layout.focus);
    if (tree < 0) {
      const entry = entries.find(item => item.pane?.id === layout.focus);
      if (!ribbonRequest.current && entry?.tab && !tiledKeys.has(entry.key) && entry.key !== center?.key && !(entry.tab.kind === "tool" && entry.tab.target.startsWith("reading:")) && (!visible[entry.side] || !effective.drawers.keys.includes(entry.key))) {
        setScene(current => ({ ...current, center: entry.key, stack: current.center && current.center !== entry.key ? [...current.stack, current.center].slice(-24) : current.stack }));
      }
      return;
    }
    const side = tree ? "right" : "left";
    if (sides[side]) { setRequestedFiles(current => ({ ...current, [side]: true })); setDrawerOpen(current => ({ ...current, [side]: true })); }
  }, [focusKey]);
  useEffect(() => { controller.schedule(); });
  useEffect(() => {
    const strip = root.current?.querySelector<HTMLElement>(".spatial-center-tabs"), selected = strip?.querySelector<HTMLElement>('[aria-selected="true"]');
    if (!strip || !selected) return;
    if (selected.offsetLeft < strip.scrollLeft) strip.scrollLeft = selected.offsetLeft;
    else if (selected.offsetLeft + selected.offsetWidth > strip.scrollLeft + strip.clientWidth) strip.scrollLeft = selected.offsetLeft + selected.offsetWidth - strip.clientWidth;
  }, [center?.key, entries.length]);
  useEffect(() => {
    if (!props.spatialHydrated) return;
    previousTabs.current = new Set(entries.filter(entry => entry.tab).map(entry => entry.key));
    lastChat.current = entries.find(entry => entry.tab?.kind === "chat")?.key ?? null;
  }, [props.spatialHydrated, props.folderRoot]);
  useEffect(() => {
    const requested = ribbonRequest.current;
    const destination = requested && entries.find(entry => samePath(entry.path, requested.path));
    if (requested && destination) {
      const side = requested.side && sides[requested.side] ? requested.side : sides.left ? "left" : "right";
      if (!requested.central && (sides.left || sides.right)) {
        setDrawerOpen(current => ({ ...current, [side]: true }));
        setScene(current => ({ ...current, views: { ...current.views, [destination.key]: { ...current.views[destination.key], side, revealed: true, expanded: true } } }));
      } else if (!requested.central) setScene(current => ({ ...current, center: destination.key, stack: current.center ? [...current.stack, current.center].slice(-24) : current.stack }));
      requestAnimationFrame(() => { if (ribbonRequest.current === requested) ribbonRequest.current = null; });
    }
    const hadTabs = previousTabs.current.size > 0;
    const freshTabs = entries.filter(entry => entry.tab && !previousTabs.current.has(entry.key));
    previousTabs.current = new Set(entries.filter(entry => entry.tab).map(entry => entry.key));
    const unnamed = freshTabs.find(entry => entry.tab?.kind === "untitled" && entry.pane?.id === layout.focus);
    if (unnamed) setScene(current => ({ ...current, center: unnamed.key, stack: current.center && current.center !== unnamed.key ? [...current.stack, current.center] : current.stack }));
    else if (center && !entries.some(entry => entry.tab && entry.key === scene.center)) setScene(current => ({ ...current, center: center.key }));
    const chat = entries.filter(entry => entry.tab?.kind === "chat").at(-1);
    if (chat && chat.key !== lastChat.current) { lastChat.current = chat.key; setScene(current => ({ ...current, center: chat.key, stack: current.center && current.center !== chat.key ? [...current.stack, current.center] : current.stack })); }
    const fresh = freshTabs.find(entry => entry.key !== center?.key);
    if (pendingReading.current && entries.some(entry => entry.key === pendingReading.current)) {
      const key = pendingReading.current; pendingReading.current = null;
      setScene(current => ({ ...current, center: key, stack: current.center && current.center !== key ? [...current.stack, current.center].slice(-24) : current.stack }));
    }
    if (!requested && hadTabs && fresh && (!sides[pendingSide.current ?? fresh.side] || pendingSide.current && ["file", "document", "generated"].includes(fresh.tab!.kind))) setScene(current => ({ ...current, center: fresh.key, stack: current.center && current.center !== fresh.key ? [...current.stack, current.center].slice(-24) : current.stack }));
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
    if (controller.suppressClick) return;
    ribbonRequest.current = null;
    controller.preserveSelections();
    if (!entry.tab || !entry.pane) { openPort(entry); return; }
    setScene(current => ({ ...current, center: entry.key, stack: center && center.key !== entry.key ? [...current.stack, center.key].slice(-24) : current.stack }));
    props.onLayout(focus(layout, entry.pane.id));
  };
  const restore = () => {
    controller.preserveSelections();
    const stack = [...scene.stack];
    let key: string | undefined;
    while ((key = stack.pop())) {
      const entry = entries.find(entry => entry.key === key && entry.tab && entry.pane);
      if (!entry) continue;
      setScene(current => ({ ...current, center: entry.key, stack }));
      props.onLayout(focus(layout, entry.pane!.id));
      return;
    }
  };
  const openPort = (entry: Entry) => {
    pendingSide.current = entry.side;
    const tree = trees[entry.side === "left" ? 0 : 1] ?? trees[0];
    if (tree) props.onLayout(split(layout, tree.id, "row", entry.side === "right"));
    props.ports?.find(port => port.id === entry.key)?.onOpen();
  };
  const click = (action: () => void) => { if (!controller.suppressClick) action(); };
  const begin = (entry: Entry, event: PointerEvent, kind: "drag" | "top" | "bottom" = "drag") => { if (!(event.target as HTMLElement).closest(".spatial-window-controls, .spatial-tab-close")) controller.begin(entry.key, event, kind); };
  const close = (entry: Entry) => { if (!entry.tab || !entry.pane) return; if (props.onRequestCloseTab) props.onRequestCloseTab(entry.pane.id, entry.tab.id); else props.onLayout(closeTab(layout, entry.pane.id, entry.tab.id)); };
  const showDrawer = (side: Side) => {
    controller.preserveSelections(); setDrawerOpen({ left: side === "left", right: side === "right" });
    requestAnimationFrame(() => root.current?.querySelector<HTMLButtonElement>(`[data-drawer-toolbar=${side}] button[aria-label="Close ${side} drawer"]`)?.focus());
  };
  const closeDrawer = (side: Side) => {
    controller.preserveSelections(); setDrawerOpen(current => ({ ...current, [side]: false })); setRequestedFiles(current => ({ ...current, [side]: false }));
    requestAnimationFrame(() => { const dom = root.current?.querySelector<HTMLElement>("[data-spatial-center=true] .cm-editor"); if (dom) EditorView.findFromDOM(dom)?.focus(); });
  };
  const pinView = (entry: Entry) => { const wasPinned = !!scene.views[entry.key]?.pin; controller.pin(entry.key); if (!wasPinned) setScene(current => ({ ...current, drawerPins: { ...(current.drawerPins ?? { left: false, right: false }), [entry.side]: true } })); };
  const pinCentered = (entry: Entry) => {
    const next = [...scene.stack].reverse().map(key => entries.find(item => item.key === key && item.tab && key !== entry.key)).find(Boolean) ?? entries.find(item => item.tab && item.key !== entry.key);
    if (!next) return;
    const side = sides[entry.side] ? entry.side : sides.left ? "left" : "right";
    controller.alignNewPin(entry.key);
    controller.preserveSelections();
    setScene(current => ({ ...current, center: next.key, stack: [...current.stack, entry.key].slice(-24), drawerPins: { ...(current.drawerPins ?? { left: false, right: false }), [side]: true },
      views: { ...current.views, [entry.key]: { ...current.views[entry.key], side, expanded: true, pin: { x: 0, y: 72, w: 320, h: 253 } } } }));
    props.onLayout(focus(layout, next.pane!.id));
  };
  dockHandler.current = (key, target, edge) => {
    const entry = entries.find(item => item.key === key && item.tab);
    if (!entry) return;
    if (target === "left" || target === "right") {
      const next = key === center?.key ? entries.find(item => item.tab && item.key !== key) : center;
      if (!sides[target] || !next || next.key === key) return;
      setScene(current => ({ ...current, center: next.key, tiles: pruneTiles(tiles, slot => slot !== key) ?? PRIMARY,
        drawerPins: { ...(current.drawerPins ?? { left: false, right: false }), [target]: true },
        views: { ...current.views, [key]: { ...current.views[key], side: target, expanded: true, revealed: true } } }));
      if (key === center?.key) props.onLayout(focus(layout, next.pane!.id));
      return;
    }
    if (edge === "center") { centerOn(entry); return; }
    const slot = key === center?.key ? PRIMARY : key;
    if (slot === target) return;
    setScene(current => ({ ...current, tiles: dockTile(tiles, slot, target, edge), views: { ...current.views, [key]: { ...current.views[key], pin: undefined } } }));
  };
  return <div ref={root} className="spatial-shell" data-spatial-shell="true"
    onPointerDownCapture={event => { if (!(event.target as HTMLElement).closest("[data-drawer-side]")) setDrawerOpen(current => current.left || current.right ? { left: false, right: false } : current); }}
    onPointerMove={event => { const gesture = columnGesture.current; if (gesture && root.current) { const inner = root.current.clientWidth, delta = (event.clientX - gesture.x) / inner * (gesture.side === "left" ? 1 : -1); setScene(current => ({ ...current, columns: current.columns.map((value, index) => index === (gesture.side === "left" ? 0 : 1) ? Math.max(.16, Math.min(.36, gesture.start + delta)) : value) as [number, number] })); } }}
    onPointerUp={() => { columnGesture.current = null; controller.finish(); }} onPointerCancel={() => { columnGesture.current = null; controller.finish(true); }}
    onKeyDown={event => { if (event.key === "Escape" && !event.defaultPrevented) { controller.finish(true); for (const side of ["left", "right"] as const) if (drawerOpen[side] && !pinned[side]) { event.preventDefault(); closeDrawer(side); } } }}>
    {(["left", "right"] as const).map(side => available[side] && <div key={`drawer-${side}`} data-drawer-side={side}>
      {!visible[side] && <button className="spatial-drawer-handle" data-drawer-handle={side} aria-label={`Show ${side} drawer`} aria-expanded={false}
        aria-controls={[...(files[side] ? [`spatial-files-${trees[side === "left" ? 0 : 1]?.tabs.find(tab => tab.kind === "tree")?.id}`] : []), ...contextual.filter(entry => entry.side === side).map(paneId)].join(" ")}
        onClick={() => showDrawer(side)}>{files[side] ? "Files" : "Related"}{contextual.filter(entry => entry.side === side).length ? ` · ${contextual.filter(entry => entry.side === side).length}` : ""}</button>}
      <div hidden={!visible[side]} id={`spatial-${side}-drawer`} className="spatial-drawer-backdrop" data-drawer-backdrop={side} />
      <div hidden={!visible[side]} className="spatial-drawer-toolbar" data-drawer-toolbar={side} onPointerDown={event => controller.beginFiles(side, event)}>
        <strong>{files[side] ? "Files" : "Related"}</strong>
        <button data-tip={`${pinned[side] ? "Unpin" : "Pin"} ${side} drawer`} aria-label={`${pinned[side] ? "Unpin" : "Pin"} ${side} drawer`} aria-pressed={pinned[side]} onClick={() => { setDrawerOpen(current => ({ ...current, [side]: true })); setScene(current => ({ ...current, drawerPins: { ...(current.drawerPins ?? { left: false, right: false }), [side]: !pinned[side] } })); }}><PinIcon /></button>
        <button aria-label={`Close ${side} drawer`} disabled={pinned[side]} onClick={() => closeDrawer(side)}>×</button>
      </div>
    </div>)}
    {trees.slice(0, 2).map((pane, index) => { const side: Side = index ? "right" : "left", tab = pane.tabs.find(tab => tab.kind === "tree")!;
      return <section key={tab.id} id={`spatial-files-${tab.id}`} hidden={!visible[side] || !files[side]} data-drawer-side={side} data-drawer-pinned={pinned[side]} className="shell-pane spatial-files" data-spatial-files={side} data-shell-pane-id={pane.id}
        onPointerDownCapture={event => {
          const row = (event.target as HTMLElement).closest<HTMLElement>("[data-spatial-drag]");
          if (row?.dataset.spatialDrag) controller.begin(row.dataset.spatialDrag, event);
        }}
        onClickCapture={event => {
          const grip = (event.target as HTMLElement).closest<HTMLElement>("[data-spatial-drag]");
          const entry = grip && entries.find(entry => entry.key === grip.dataset.spatialDrag);
          if (entry && !controller.suppressClick) pinView(entry);
        }}
        onDoubleClickCapture={event => { if ((event.target as HTMLElement).closest("[data-tree-path]")) { pendingSide.current = side; props.onLayout(split(layout, pane.id, "row", side === "right")); } }}>
        <button role="tab" aria-label={side === "left" ? "Files" : "Right file buffer"} aria-selected={layout.focus === pane.id} className="spatial-files-title" onClick={() => props.onLayout(focus(layout, pane.id))}>Files</button><SpatialFileEdges /><div className="shell-pane__body">{props.render(tab, pane)}</div>
      </section>;
    })}
    {entries.map(entry => { const tab = entry.tab, centered = entry.key === center?.key, tiled = tiledKeys.has(entry.key), central = centered || tiled, name = relative(entry.path, props.folderRoot), title = tab?.title ?? props.ports?.find(port => port.id === entry.key)?.label ?? name;
      return <PaneHeaderProvider key={tab?.id ?? entry.key}><section hidden={!central && (!visible[entry.side] || !effective.drawers.keys.includes(entry.key))} data-drawer-side={!central ? entry.side : undefined} data-drawer-pinned={!central ? pinned[entry.side] : undefined} id={centered ? "spatial-center" : paneId(entry)} className="shell-pane spatial-view" data-spatial-key={entry.key} data-spatial-side={entry.side} data-spatial-center={centered} data-spatial-tiled={tiled} data-expanded={central || scene.views[entry.key]?.expanded || false}
        data-shell-pane-id={entry.pane?.id} data-shell-target={entry.path} onFocusCapture={() => { if (entry.pane && props.layout.focus !== entry.pane.id) props.onLayout(focus(layout, entry.pane.id)); }}>
        <header className="spatial-view-header" onPointerDown={event => begin(entry, event)}>
          {centered ? <div className="spatial-center-tabs" role="tablist" aria-label="Center views">
            {entries.filter(candidate => candidate.tab).map(candidate => {
              const candidateTab = candidate.tab!, selected = candidate.key === entry.key;
              const label = candidateTab.title ?? relative(candidate.path, props.folderRoot);
              return <div key={candidateTab.id} className="spatial-tab-item"><button id={`spatial-tab-${candidateTab.id}`} type="button" role="tab" aria-label={label} aria-selected={selected} aria-controls={`spatial-content-${candidateTab.id}`} tabIndex={selected ? 0 : -1}
                onContextMenu={event => props.onTabContextMenu?.(event, candidateTab)}
                className="spatial-center-tab" data-tip={relative(candidate.path, props.folderRoot)}
                data-shell-tab-kind={candidateTab.kind} data-shell-tab-target={candidate.path} data-shell-tab-source={selected ? candidate.path : undefined} data-shell-tab-buffer={selected ? bufferTarget(candidateTab.id) : undefined}
                onPointerDown={event => { event.stopPropagation(); begin(candidate, event); }} onClick={() => centerOn(candidate)} onKeyDown={event => {
                  if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
                  event.preventDefault(); event.stopPropagation();
                  const tabs = entries.filter(item => item.tab), index = tabs.findIndex(item => item.key === candidate.key);
                  const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
                  centerOn(tabs[next]);
                  requestAnimationFrame(() => root.current?.querySelector<HTMLButtonElement>('.spatial-center-tabs [aria-selected="true"]')?.focus());
                }}>{label}{props.dirtyTabIds?.has(candidateTab.id) ? " *" : ""}</button><button type="button" className="spatial-tab-close" aria-label={`Close ${label}`} data-tip={`Close ${label}`} onPointerDown={event => event.stopPropagation()} onClick={event => { event.stopPropagation(); close(candidate); }}>×</button></div>;
            })}
          </div> : <button type="button" role={tab ? "tab" : undefined} aria-label={title} aria-selected={tab ? layout.focus === entry.pane?.id : undefined} className="spatial-filename" data-tip={name} data-shell-tab-kind={tab?.kind} data-shell-tab-target={entry.path} data-shell-tab-source={entry.path} data-shell-tab-vertical={true} data-shell-tab-buffer={tab ? bufferTarget(tab.id) : undefined}
            onContextMenu={event => { if (tab) props.onTabContextMenu?.(event, tab); }}
            data-ribbon-port={entry.port ? entry.key : undefined} onClick={() => click(() => { if (entry.port) openPort(entry); else if (!centered) { if (tab?.kind === "untitled") centerOn(entry); else controller.toggle(entry.key); } })}>{tab && !["document", "generated", "file"].includes(tab.kind) ? title : name || title}{tab && props.dirtyTabIds?.has(tab.id) ? " *" : ""}</button>}
          <PaneHeaderSlots />
          <div className="spatial-window-controls">
            {centered ? <><button data-tip="Search (Ctrl/Cmd+P)" aria-label="Open search" onClick={() => openCommandBar()}>⌕</button><button data-tip="Restore previous center" aria-label="Restore previous center" onClick={restore}>▣</button>{filesWindows && (sides.left || sides.right) && entries.some(item => item.tab && item.key !== entry.key) && <button data-tip="Pin view in drawer" aria-label="Pin view in drawer" onClick={() => pinCentered(entry)}><PinIcon /></button>}<button data-tip="Automatic accordion" aria-label="Automatic accordion" aria-pressed={scene.auto} onClick={() => setScene(current => ({ ...current, auto: !current.auto }))}>↕</button></>
              : <><button data-tip={scene.views[entry.key]?.pin ? "Unpin view" : "Pin view"} aria-label={scene.views[entry.key]?.pin ? "Unpin view" : "Pin view"} aria-pressed={!!scene.views[entry.key]?.pin} onClick={() => click(() => { if (tiled) { controller.pin(entry.key); dockHandler.current(entry.key, sides[entry.side] ? entry.side : sides.left ? "left" : "right", "center"); } else pinView(entry); })}><PinIcon /></button>
                <button data-tip="Center" aria-label="Center" onClick={() => click(() => centerOn(entry))}>□</button>{!tiled && <button data-tip="Minimize" aria-label="Minimize" onClick={() => { if (controller.frames.get(entry.key)?.h !== 36) controller.toggle(entry.key); }}>—</button>}</>}
            {tab && !centered && <button data-tip={`Close ${title}`} aria-label={`Close ${title}`} onClick={() => close(entry)}>×</button>}
          </div>
        </header>
        <div id={tab ? `spatial-content-${tab.id}` : undefined} className="shell-pane__body spatial-view-body">{tab && entry.pane ? props.render(tab, entry.pane) : <button className="spatial-open" onClick={() => openPort(entry)}>Open {title}</button>}</div>
      </section></PaneHeaderProvider>;
    })}
    {entries.map(entry => <div key={`strip-${entry.key}`} hidden={!visible[entry.side] || !effective.drawers.keys.includes(entry.key)} data-drawer-side={entry.side} className="spatial-header-extension" data-spatial-strip={entry.key} onPointerDown={event => begin(entry, event)}>
      <button data-tip={`${relative(entry.path, props.folderRoot)} — click to pin or unpin`} className="spatial-file-tab" onContextMenu={event => { if (entry.tab) props.onTabContextMenu?.(event, entry.tab); }} onClick={() => click(() => entry.port ? openPort(entry) : pinView(entry))}>{entry.tab && !["document", "generated", "file"].includes(entry.tab.kind) ? entry.tab.title ?? entry.tab.target : relative(entry.path, props.folderRoot)}</button>
    </div>)}
    {entries.flatMap(entry => (["top", "bottom"] as const).map(edge => <div key={`${entry.key}-${edge}`} role="separator" aria-label={`Resize ${relative(entry.path, props.folderRoot)} ${edge}`} aria-orientation="horizontal" tabIndex={0}
      hidden={!visible[entry.side] || !effective.drawers.keys.includes(entry.key)} data-drawer-side={entry.side} className="spatial-resize-rule" data-spatial-resize={entry.key} data-resize-edge={edge} onPointerDown={event => { event.preventDefault(); controller.begin(entry.key, event, edge); }}
      onKeyDown={event => { if (event.key === "ArrowUp" || event.key === "ArrowDown") { event.preventDefault(); controller.resize(entry.key, event.key === "ArrowUp" ? -10 : 10, edge); } }} />))}
    <div className="spatial-drop-preview" data-spatial-drop-preview hidden />
    {!center && <div className="spatial-empty">{props.empty}</div>}
    {(["left", "right"] as const).map(side => <div key={`column-${side}`} hidden={!visible[side] || !pinned[side]} data-drawer-side={side} className="spatial-column-rule" data-spatial-column={side} role="separator" aria-label={`Resize ${side} column`} aria-orientation="vertical" tabIndex={0}
      onPointerDown={event => { event.preventDefault(); event.currentTarget.setPointerCapture(event.pointerId); columnGesture.current = { side, x: event.clientX, start: scene.columns[side === "left" ? 0 : 1] }; }}
      onKeyDown={event => { if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); setScene(current => ({ ...current, columns: current.columns.map((value, index) => index === (side === "left" ? 0 : 1) ? Math.max(.16, Math.min(.36, value + (event.key === "ArrowLeft" ? -.01 : .01) * (side === "left" ? 1 : -1))) : value) as [number, number] })); } }} />)}
    <div className="spatial-minimap" data-tip="Scroll the center view" role="scrollbar" aria-label="Center minimap" aria-valuenow={Math.round(minimap.scroll)} aria-valuemin={0} aria-valuemax={minimap.total} aria-controls="spatial-center"
      onPointerDown={event => { const dom = root.current?.querySelector<HTMLElement>("[data-spatial-center=true] .cm-editor"), view = dom && EditorView.findFromDOM(dom), box = event.currentTarget.getBoundingClientRect(); if (view) view.scrollDOM.scrollTop = (event.clientY - box.top) / box.height * Math.max(0, minimap.total - minimap.viewport); }}>
      <i style={{ top: `${minimap.scroll / minimap.total * 100}%`, height: `${Math.min(100, minimap.viewport / minimap.total * 100)}%` }} />
      {relations.filter(relation => relation.fromPath === center?.path || relation.toPath === center?.path).map(relation => { const reverse = relation.toPath === center?.path, view = reverse ? relation.to : relation.from, range = reverse ? relation.toRange : relation.fromRange; if (!view || !range) return null; const line = view.state.doc.lineAt(Math.min(range[0], view.state.doc.length)).number; return <b key={relation.key} style={{ top: `${(line - 1) / view.state.doc.lines * 100}%` }} />; })}
    </div>
  </div>;
}
