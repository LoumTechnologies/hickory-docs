<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/SpatialShellView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-64456f67b44e"></a>
## `apps/web/src/shell/SpatialShellView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-64456f67b44e-Props"></a>
### `Props`

<storycode:copy id="f-64456f67b44e-Props">
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiOutcome } from "../lib/uiActions";
import { PRIMARY, dockTile, pruneTiles, tileRects, type DockEdge } from "./spatialTiles";
import { SPATIAL_RIBBON_OPEN, type SpatialRibbonOpen } from "./spatialNavigation";
import { PaneHeaderProvider, PaneHeaderSlots } from "./PaneHeader";
import { openCommandBar } from "./CommandBar";
import { useCallback, useEffect, useMemo, useRef, useState, useId, type SetStateAction, type PointerEvent } from "react";
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
</storycode:copy>
<a id="f-64456f67b44e-Entry"></a>
### `Entry`

<storycode:copy id="f-64456f67b44e-Entry">
interface Entry extends SpatialItem { tab?: Tab; pane?: Pane }
</storycode:copy>
<a id="f-64456f67b44e-PinIcon"></a>
### `PinIcon`

<storycode:copy id="f-64456f67b44e-PinIcon">
function PinIcon() { return <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" aria-hidden="true"><path d="M5 2h6l-1 5 3 3H3l3-3-1-5ZM8 10v5" /></svg>; }
</storycode:copy>
<a id="f-64456f67b44e-storageKey"></a>
### `storageKey`

<storycode:copy id="f-64456f67b44e-storageKey">
const storageKey = (folder?: string) => `hickory:spatial-layout:v1:${folder ?? "scratch"}`;
</storycode:copy>
<a id="f-64456f67b44e-load"></a>
### `load`

<storycode:copy id="f-64456f67b44e-load">
const load = (folder?: string) => { try { return readSpatialState(localStorage.getItem(storageKey(folder))); } catch { return readSpatialState(null); } };
</storycode:copy>
<a id="f-64456f67b44e-relative"></a>
### `relative`

<storycode:copy id="f-64456f67b44e-relative">
const relative = (path: string, root?: string) => root && path.startsWith(`${root}/`) ? path.slice(root.length + 1) : path;
</storycode:copy>
<a id="f-64456f67b44e-paneId"></a>
### `paneId`

<storycode:copy id="f-64456f67b44e-paneId">
const paneId = (entry: Entry) => `spatial-pane-${entry.tab?.id ?? encodeURIComponent(entry.key)}`;

</storycode:copy>
<a id="f-64456f67b44e-SpatialShellView"></a>
### `SpatialShellView`

Each editor has one stable mount. Centering, pinning and parallax only change its rectangle.

<storycode:copy id="f-64456f67b44e-SpatialShellView">
/** Each editor has one stable mount. Centering, pinning and parallax only change its rectangle. */
export function SpatialShellView(props: Props) {
  const instance = useId();
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
  const close = async (entry: Entry, signal?: AbortSignal): Promise<UiOutcome> => {
    if (!entry.tab || !entry.pane) return { status: "unavailable", reason: "This view has no closeable tab" };
    if (props.onRequestCloseTab) return await (signal ? props.onRequestCloseTab(entry.pane.id, entry.tab.id, signal) : props.onRequestCloseTab(entry.pane.id, entry.tab.id)) ?? requested();
    props.onLayout(closeTab(layout, entry.pane.id, entry.tab.id)); return requested();
  };
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
  const toggleDrawerPin = (side: Side) => { setDrawerOpen(current => ({ ...current, [side]: true })); setScene(current => ({ ...current, drawerPins: { ...(current.drawerPins ?? { left: false, right: false }), [side]: !pinned[side] } })); };
  const setAccordion = (auto: boolean) => setScene(current => ({ ...current, auto }));
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
  const actionParent = `spatial-layout:${instance}`;
  const revision = useUiRevision(() => [layout, sceneRef.current, drawerOpen, visible, pinned, [...controller.frames]]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], subject = actionParent) {
    commands.push({ key: `${actionParent}/${id}`, command: { label, parent: actionParent, subject, parameters, prerequisites: ["Current window layout and mounted view identity; dirty-close decisions remain in the normal prompt"], read: () => ({ revision: revision(), enabled: !!root.current && !controller.suppressClick }), execute } });
  }
  const requested = () => ({ status: "completed" as const, value: { dispatch: "requested", result: "Observe actual layout, frame or unsaved-close prompt" } });
  add("read", "Read spatial layout bytes", { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ scene: sceneRef.current, center: center?.key ?? null, drawers: { available, visible, pinned, files }, frames: Object.fromEntries(controller.frames), views: entries.map(entry => ({ key: entry.key, path: entry.path, title: entry.tab?.title ?? null, side: entry.side, tab: entry.tab?.id ?? null, pane: entry.pane?.id ?? null, centered: entry.key === center?.key, tiled: tiledKeys.has(entry.key), visible: entry.key === center?.key || tiledKeys.has(entry.key) || (visible[entry.side] && effective.drawers.keys.includes(entry.key)), dirty: !!entry.tab && !!props.dirtyTabIds?.has(entry.tab.id) })) }));
    const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current layout state" };
    return { status: "completed", value: { encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null } };
  });
  for (const operation of ["center", "pin", "expand", "close"] as const) add(operation, operation === "expand" ? "Toggle spatial view expansion" : operation === "pin" ? "Toggle spatial view pin" : `${operation[0].toUpperCase() + operation.slice(1)} spatial view`, { key: { type: "string", maxLength: 8192 } }, async (args, _actor, signal) => {
    const entry = entries.find(entry => entry.key === args.key);
    if (!entry) return { status: "unavailable", reason: "View does not belong to this window's current layout" };
    if (operation === "center") centerOn(entry);
    else if (operation === "close") { if (!entry.tab || !entry.pane) return { status: "unavailable", reason: "This reading port has no closeable tab" }; return close(entry, signal); }
    else {
      if (!controller.frames.has(entry.key)) return { status: "unavailable", reason: "Open this view's drawer before changing its frame" };
      if (operation === "pin") { if (entry.key === center?.key) pinCentered(entry); else pinView(entry); }
      else { if (entry.key === center?.key || tiledKeys.has(entry.key)) return { status: "unavailable", reason: "Centered and tiled views already occupy their full frame" }; controller.toggle(entry.key); }
    }
    return requested();
  }, operation === "close" ? `${actionParent}:close` : actionParent);
  add("resize", "Resize spatial view", { key: { type: "string", maxLength: 8192 }, delta: { type: "number", minimum: -2000, maximum: 2000 }, edge: { type: "string", maxLength: 16, choices: ["top", "bottom"] } }, async args => {
    const key = String(args.key);
    if (!entries.some(entry => entry.key === key) || !controller.frames.has(key) || key === center?.key || tiledKeys.has(key)) return { status: "unavailable", reason: "Choose a mounted drawer view with an actual frame" };
    controller.resize(key, Number(args.delta), args.edge as "top" | "bottom"); return requested();
  });
  add("dock", "Dock spatial view", { key: { type: "string", maxLength: 8192 }, target: { type: "string", maxLength: 8192 }, edge: { type: "string", maxLength: 16, choices: ["left", "right", "top", "bottom", "center"] } }, async args => {
    const key = String(args.key), target = String(args.target);
    if (!entries.some(entry => entry.key === key && entry.tab) || !([PRIMARY, "left", "right"].includes(target) || tiledKeys.has(target))) return { status: "unavailable", reason: "Choose a current tab and actual center/tile/drawer target" };
    if ((target === "left" || target === "right") && !sides[target]) return { status: "unavailable", reason: "Enable this drawer in settings before docking" };
    dockHandler.current(key, target, args.edge as DockEdge); return requested();
  });
  add("drawer", "Set spatial drawer visibility", { side: { type: "string", maxLength: 8, choices: ["left", "right"] }, open: { type: "boolean" } }, async args => {
    const side = args.side as Side;
    if (!available[side]) return { status: "unavailable", reason: "This drawer has no available content" };
    if (!args.open && pinned[side]) return { status: "unavailable", reason: "Unpin this drawer before closing it" };
    (args.open ? showDrawer : closeDrawer)(side); return requested();
  });
  add("drawer-pin", "Toggle spatial drawer pin", { side: { type: "string", maxLength: 8, choices: ["left", "right"] } }, async args => {
    const side = args.side as Side;
    if (!available[side] || !visible[side]) return { status: "unavailable", reason: "Open an available drawer before changing its pin" };
    toggleDrawerPin(side); return requested();
  });
  add("accordion", "Set spatial automatic accordion", { enabled: { type: "boolean" } }, async args => { setAccordion(Boolean(args.enabled)); return requested(); });
  add("restore", "Restore previous spatial center", {}, async () => { restore(); return requested(); });
  add("search", "Open command search", {}, async () => { openCommandBar(); return requested(); });
  useUiCommands(commands);
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
        <button data-tip={`${pinned[side] ? "Unpin" : "Pin"} ${side} drawer`} aria-label={`${pinned[side] ? "Unpin" : "Pin"} ${side} drawer`} aria-pressed={pinned[side]} onClick={() => toggleDrawerPin(side)}><PinIcon /></button>
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
        <button role="tab" aria-label={side === "left" ? "Files" : "Right file buffer"} aria-selected={layout.focus === pane.id} className="spatial-files-title" onClick={() => props.onLayout(focus(layout, pane.id))}>Files</button><SpatialFileEdges /><div className="shell-pane__body"><UiActionsVisibility visible={visible[side] && files[side]}>{props.render(tab, pane)}</UiActionsVisibility></div>
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
            {centered ? <><button data-tip="Search (Ctrl/Cmd+P)" aria-label="Open search" onClick={() => openCommandBar()}>⌕</button><button data-tip="Restore previous center" aria-label="Restore previous center" onClick={restore}>▣</button>{filesWindows && (sides.left || sides.right) && entries.some(item => item.tab && item.key !== entry.key) && <button data-tip="Pin view in drawer" aria-label="Pin view in drawer" onClick={() => pinCentered(entry)}><PinIcon /></button>}<button data-tip="Automatic accordion" aria-label="Automatic accordion" aria-pressed={scene.auto} onClick={() => setAccordion(!scene.auto)}>↕</button></>
              : <><button data-tip={scene.views[entry.key]?.pin ? "Unpin view" : "Pin view"} aria-label={scene.views[entry.key]?.pin ? "Unpin view" : "Pin view"} aria-pressed={!!scene.views[entry.key]?.pin} onClick={() => click(() => { if (tiled) { controller.pin(entry.key); dockHandler.current(entry.key, sides[entry.side] ? entry.side : sides.left ? "left" : "right", "center"); } else pinView(entry); })}><PinIcon /></button>
                <button data-tip="Center" aria-label="Center" onClick={() => click(() => centerOn(entry))}>□</button>{!tiled && <button data-tip="Minimize" aria-label="Minimize" onClick={() => { if (controller.frames.get(entry.key)?.h !== 36) controller.toggle(entry.key); }}>—</button>}</>}
            {tab && !centered && <button data-tip={`Close ${title}`} aria-label={`Close ${title}`} onClick={() => close(entry)}>×</button>}
          </div>
        </header>
        <div id={tab ? `spatial-content-${tab.id}` : undefined} className="shell-pane__body spatial-view-body"><UiActionsVisibility visible={central || (visible[entry.side] && effective.drawers.keys.includes(entry.key) && (scene.views[entry.key]?.pin ? scene.views[entry.key]!.pin!.h > 96 : scene.views[entry.key]?.expanded === true))}>{tab && entry.pane ? props.render(tab, entry.pane) : <button className="spatial-open" onClick={() => openPort(entry)}>Open {title}</button>}</UiActionsVisibility></div>
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
</storycode:copy>

### Assemble `apps/web/src/shell/SpatialShellView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/SpatialShellView.tsx"><storycode:paste select="#f-64456f67b44e-Props"/><storycode:paste select="#f-64456f67b44e-Entry"/><storycode:paste select="#f-64456f67b44e-PinIcon"/><storycode:paste select="#f-64456f67b44e-storageKey"/><storycode:paste select="#f-64456f67b44e-load"/><storycode:paste select="#f-64456f67b44e-relative"/><storycode:paste select="#f-64456f67b44e-paneId"/><storycode:paste select="#f-64456f67b44e-SpatialShellView"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-57fb3ab47480"></a>
## `apps/web/src/shell/SpatialShellView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-57fb3ab47480-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement">
import { useEffect, useRef, useState } from "react";
import { act, cleanup, fireEvent, render, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import { history, undo } from "@codemirror/commands";
import { emptyPane, open, tab, type Layout, type Tab } from "./layout";
import type { SpatialState } from "./spatialLayout";
import { SpatialShellView } from "./SpatialShellView";
import { saveFilesWindows } from "../lib/filesWindows";
import { saveSpatialSides } from "../lib/spatialSides";
import { revealLine } from "../lib/revealLine";
import { publishReading, forgetReading, openReading } from "../lib/readingViews";
import { UiActions } from "../lib/uiActions";
import { UiActionContext, useUiCommands } from "../lib/uiActionBridge";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
beforeEach(() => saveFilesWindows(true));
</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-2">
afterEach(() => { cleanup(); localStorage.clear(); forgetReading("proposal"); });
</storycode:copy>
<a id="f-57fb3ab47480-live"></a>
### `live`

<storycode:copy id="f-57fb3ab47480-live">
const live = new Map<string, EditorView>();
</storycode:copy>
<a id="f-57fb3ab47480-Buffer"></a>
### `Buffer`

<storycode:copy id="f-57fb3ab47480-Buffer">
function Buffer({ entry }: { entry: Tab }) {
  const parent = useRef<HTMLDivElement>(null);
  useUiCommands([{ key: `probe:${entry.id}`, command: { label: entry.target, parent: "test-visible-editors", subject: entry.id, parameters: {}, prerequisites: [], read: () => ({ revision: "probe", enabled: true }), execute: async () => ({ status: "completed", value: null }) } }]);
  useEffect(() => { const view = new EditorView({ parent: parent.current!, state: EditorState.create({ doc: `${entry.target}\nsecond line`, extensions: [lineNumbers(), history()] }) }); live.set(entry.target, view); return () => { view.destroy(); live.delete(entry.target); }; }, [entry.id]);
  return <div ref={parent} />;
}
</storycode:copy>
<a id="f-57fb3ab47480-Harness"></a>
### `Harness`

<storycode:copy id="f-57fb3ab47480-Harness">
function Harness({ mixed = false, related = false, files = false, onClose }: { mixed?: boolean; related?: boolean; files?: boolean; onClose?: React.ComponentProps<typeof SpatialShellView>["onRequestCloseTab"] }) {
  const [layout, setLayout] = useState<Layout>(() => { const pane = emptyPane(); return { root: { ...pane, tabs: [tab("document", "main.md"), tab("document", "related.md"), ...(mixed ? [tab("file", "app.py"), tab("chat", "conversation", "Agent")] : []), tab("tree", "folder")], active: 0 }, focus: pane.id }; });
  return <><button onClick={() => { publishReading("proposal", { title: "Review", path: "note.md", before: "old", after: "new", status: "pending" }); setLayout(current => open(current, tab("tool", "reading:proposal", "Review"))); }}>Open review</button><SpatialShellView layout={layout} onRequestCloseTab={onClose} onLayout={setLayout} sources={[]} files={[]} hasFiles={files} links={related ? [{ key: "related", title: "Related file", family: "declared", from: { path: "main.md", lines: [1, 1] }, to: { path: "related.md" } }] : []} render={entry => <Buffer entry={entry} />} /></>;
}
</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-3">
it("uses registered center and restore commands without replacing the editor or accepting old layout handles", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const ui = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  await waitFor(() => expect(live.has("related.md")).toBe(true));
  const original = live.get("related.md")!;
  act(() => original.dispatch({ changes: { from: 0, insert: "draft " }, selection: { anchor: 4 } }));
  expect(registry.observe("test-visible-editors").actions.map(action => action.label)).toEqual(["main.md"]);
  const parent = registry.groups().groups.find(group => group.startsWith("spatial-layout:"))!;
  const center = registry.observe(parent).actions.find(action => action.label === "Center spatial view")!;
  const actor = { id: "python:turn", kind: "agent" as const };
  await act(async () => { expect(await registry.act(registry.scope, actor, { handle: center.handle, arguments: { key: "document:related.md" }, request_id: "center" })).toMatchObject({ status: "result" }); });
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:related.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
  expect(live.get("related.md")).toBe(original); expect(original.state.selection.main.anchor).toBe(4);
  expect(registry.observe("test-visible-editors").actions.map(action => action.label)).toEqual(["related.md"]);
  expect(await registry.act(registry.scope, actor, { handle: center.handle, arguments: { key: "document:main.md" }, request_id: "old-layout" })).toMatchObject({ status: "stale" });
  const restore = registry.observe(parent).actions.find(action => action.label === "Restore previous spatial center")!;
  await act(async () => { await registry.act(registry.scope, actor, { handle: restore.handle, arguments: {}, request_id: "restore" }); });
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:main.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
  expect(original.state.doc.toString()).toContain("draft ");
});
</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-4">
it("centers and restores the same editor instance with selection and undo state intact", async () => {
  const ui = render(<Harness />);
  await waitFor(() => expect(live.has("related.md")).toBe(true));
  const original = live.get("related.md")!;
  act(() => original.dispatch({ changes: { from: 0, insert: "my edit " }, selection: { anchor: 5 } }));
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  fireEvent.click(related.querySelector('[aria-label="Center"]')!);
  await waitFor(() => expect(related.dataset.spatialCenter).toBe("true"));
  expect(live.get("related.md")).toBe(original);
  expect(original.state.doc.toString()).toContain("my edit "); expect(original.state.selection.main.anchor).toBe(5);
  fireEvent.click(related.querySelector('[aria-label="Restore previous center"]')!);
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:main.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
  expect(live.get("related.md")).toBe(original);
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-5">
it("switches documents, files and agents through center tabs without remounting their buffers", async () => {
  const ui = render(<Harness mixed />);
  await waitFor(() => expect(live.has("conversation")).toBe(true));
  const document = live.get("main.md")!, agent = live.get("conversation")!, file = live.get("app.py")!;
  act(() => { document.dispatch({ changes: { from: 0, insert: "draft " }, selection: { anchor: 4 } }); agent.dispatch({ changes: { from: 0, insert: "unsent prompt " } }); });
  const tabs = () => within(ui.getByRole("tablist", { name: "Center views" }));
  expect(tabs().queryByRole("tab", { name: "folder" })).toBeNull();
  for (const [label, key] of [["main.md", "document:main.md"], ["app.py", "file:app.py"], ["Agent", "chat:conversation"], ["main.md", "document:main.md"]]) {
    fireEvent.click(tabs().getByRole("tab", { name: label }));
    await waitFor(() => expect(ui.container.querySelector(`[data-spatial-key="${key}"]`)?.getAttribute("data-spatial-center")).toBe("true"));
    expect(tabs().getByRole("tab", { name: label }).getAttribute("aria-selected")).toBe("true");
  }
  expect(live.get("main.md")).toBe(document); expect(live.get("app.py")).toBe(file); expect(live.get("conversation")).toBe(agent);
  expect(document.state.selection.main.anchor).toBe(4); expect(agent.state.doc.toString()).toContain("unsent prompt");
  act(() => { expect(undo(document)).toBe(true); });
  expect(document.state.doc.toString()).toBe("main.md\nsecond line");
  fireEvent.click(ui.container.querySelector('[data-spatial-center=true] [aria-label="Close main.md"]')!);
  await waitFor(() => expect(tabs().queryByRole("tab", { name: "main.md" })).toBeNull());
  expect(tabs().getByRole("tab", { name: "related.md" }).getAttribute("aria-selected")).toBe("true");
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-6">
it("supports arrow keys, Home and End in the center tab strip", async () => {
  const ui = render(<Harness />);
  const tabs = () => within(ui.getByRole("tablist", { name: "Center views" }));
  fireEvent.keyDown(tabs().getByRole("tab", { name: "main.md" }), { key: "ArrowRight" });
  await waitFor(() => expect(tabs().getByRole("tab", { name: "related.md" }).getAttribute("aria-selected")).toBe("true"));
  await waitFor(() => expect(document.activeElement).toBe(tabs().getByRole("tab", { name: "related.md" })));
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  await waitFor(() => expect(tabs().getByRole("tab", { name: "main.md" }).getAttribute("aria-selected")).toBe("true"));
  fireEvent.keyDown(tabs().getByRole("tab", { name: "main.md" }), { key: "End" });
  expect(tabs().getByRole("tab", { name: "related.md" }).getAttribute("aria-selected")).toBe("true");
  fireEvent.keyDown(tabs().getByRole("tab", { name: "related.md" }), { key: "Home" });
  expect(tabs().getByRole("tab", { name: "main.md" }).getAttribute("aria-selected")).toBe("true");
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-7">
it("uses the full width and shows no drawer handles when only unrelated tabs are open", async () => {
  const ui = render(<Harness />);
  const shell = ui.container.querySelector<HTMLElement>(".spatial-shell")!;
  Object.defineProperties(shell, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  await waitFor(() => expect(ui.container.querySelector<HTMLElement>('[data-spatial-center=true]')!.style.width).toBe("1200px"));
  expect(ui.queryByRole("button", { name: /Show .* drawer/ })).toBeNull();
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(true);
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!.hidden).toBe(true);
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-8">
it("opens related content as an overlay, reserves space only when pinned, and keeps drafts and selection", async () => {
  const ui = render(<Harness related />);
  const shell = ui.container.querySelector<HTMLElement>(".spatial-shell")!;
  Object.defineProperties(shell, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  const center = ui.container.querySelector<HTMLElement>('[data-spatial-center=true]')!;
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  const original = live.get("related.md")!;
  expect(related.hidden).toBe(true);
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  await waitFor(() => expect(related.hidden).toBe(false));
  await waitFor(() => expect(center.style.width).toBe("1200px"));
  act(() => original.dispatch({ changes: { from: 0, insert: "draft " }, selection: { anchor: 3 } }));
  fireEvent.click(ui.getByRole("button", { name: "Pin left drawer" }));
  await waitFor(() => expect(parseFloat(center.style.width)).toBeLessThan(1200));
  fireEvent.click(ui.getByRole("button", { name: "Unpin left drawer" }));
  await waitFor(() => expect(center.style.width).toBe("1200px"));
  fireEvent.keyDown(shell, { key: "Escape" });
  await waitFor(() => expect(related.hidden).toBe(true));
  expect(live.get("related.md")).toBe(original); expect(original.state.doc.toString()).toContain("draft "); expect(original.state.selection.main.anchor).toBe(3);
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  fireEvent.pointerDown(center);
  await waitFor(() => expect(related.hidden).toBe(true));
  ui.rerender(<Harness />);
  expect(ui.queryByRole("button", { name: /Show .* drawer/ })).toBeNull();
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-9">
it("keeps Files collapsed, restores drawer pins, and respects the independent side settings", async () => {
  const ui = render(<Harness files />);
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(true);
  expect(ui.queryByRole("button", { name: "Show right drawer" })).toBeNull();
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  fireEvent.click(ui.getByRole("button", { name: "Pin left drawer" }));
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(false);
  const editor = live.get("folder")!;
  act(() => saveSpatialSides({ left: false, right: true }));
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(true);
  expect(ui.getByRole("button", { name: "Show right drawer" })).toBeTruthy();
  act(() => saveSpatialSides({ left: true, right: false }));
  expect(live.get("folder")).toBe(editor);
  ui.unmount();
  const restored = render(<Harness files />);
  expect(restored.getByRole("button", { name: "Unpin left drawer" })).toBeTruthy();
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-10">
it("centers a revealed file when its context side is hidden", async () => {
  const ui = render(<Harness />);
  act(() => revealLine("related.md", 1));
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:related.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-11">
it("announces a pending review in a collapsed drawer without moving the editor", async () => {
  saveSpatialSides({ left: true, right: true });
  const ui = render(<Harness />);
  fireEvent.click(ui.getByRole("button", { name: "Open review" }));
  const show = ui.getByRole("button", { name: /Show .* drawer/ });
  const review = ui.container.querySelector<HTMLElement>('[data-spatial-key="tool:reading:proposal"]')!;
  expect(review.hidden).toBe(true);
  fireEvent.click(show);
  expect(review.hidden).toBe(false);
  act(() => publishReading("proposal", { title: "Review", path: "note.md", before: "old", after: "new", status: "applied" }));
  expect(review.hidden).toBe(true);
  expect(ui.queryByRole("button", { name: /Show .* drawer/ })).toBeNull();
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-12">
it("centers an explicitly requested review while background proposals remain collapsed", async () => {
  const ui = render(<Harness />);
  fireEvent.click(ui.getByRole("button", { name: "Open review" }));
  expect(ui.container.querySelector('[data-spatial-key="tool:reading:proposal"]')?.getAttribute("data-spatial-center")).toBe("false");
  act(() => openReading("proposal", "Review"));
  expect(ui.container.querySelector('[data-spatial-key="tool:reading:proposal"]')?.getAttribute("data-spatial-center")).toBe("true");
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-13">
it("closes an inactive center tab directly from its X without changing the active editor", async () => {
  const ui = render(<Harness />);
  const original = live.get("main.md")!;
  const tabs = within(ui.getByRole("tablist", { name: "Center views" }));
  fireEvent.click(tabs.getByRole("button", { name: "Close related.md" }));
  await waitFor(() => expect(live.has("related.md")).toBe(false));
  expect(live.get("main.md")).toBe(original);
  expect(tabs.getByRole("tab", { name: "main.md" }).getAttribute("aria-selected")).toBe("true");
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-14">
it("opens a ribbon destination in its drawer and uses icons for drawer pins", async () => {
  const { requestSpatialRibbonOpen } = await import("./spatialNavigation");
  const ui = render(<Harness />);
  const original = live.get("related.md")!;
  act(() => { requestSpatialRibbonOpen(ui.container, { kind: "path", path: "related.md", family: "declared" }); revealLine("related.md", 2); });
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  await waitFor(() => expect(related.hidden).toBe(false));
  expect(related.dataset.spatialCenter).toBe("false");
  expect(related.dataset.expanded).toBe("true");
  const pin = ui.getByRole("button", { name: "Pin left drawer" });
  expect(pin.textContent).toBe(""); expect(pin.querySelector("svg")).toBeTruthy();
  expect(live.get("related.md")).toBe(original);
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-15">
it("keeps a newly opened ribbon target beside the existing center", async () => {
  const { requestSpatialRibbonOpen } = await import("./spatialNavigation");
  function RibbonHarness() {
    const [layout, setLayout] = useState<Layout>(() => { const pane = emptyPane(); return { root: { ...pane, tabs: [tab("document", "main.md")], active: 0 }, focus: pane.id }; });
    return <div><button onClick={event => {
      requestSpatialRibbonOpen(event.currentTarget.parentElement, { kind: "path", path: "new.md", family: "declared" });
      setLayout(current => open(current, tab("document", "new.md")));
      revealLine("new.md", 2);
    }}>Follow ribbon</button><SpatialShellView layout={layout} onLayout={setLayout} sources={[]} files={[]} links={[]} render={entry => <Buffer entry={entry} />} /></div>;
  }
  const ui = render(<RibbonHarness />);
  fireEvent.click(ui.getByText("Follow ribbon"));
  const destination = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:new.md"]')!;
  await waitFor(() => expect(destination.hidden).toBe(false));
  expect(destination.dataset.spatialCenter).toBe("false");
  expect(ui.container.querySelector('[data-spatial-center=true]')?.getAttribute("data-shell-target")).toBe("main.md");
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-16"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-16">
it("centers a ribbon destination when both side areas are disabled", async () => {
  const { requestSpatialRibbonOpen } = await import("./spatialNavigation");
  saveSpatialSides({ left: false, right: false });
  const ui = render(<Harness />);
  act(() => { requestSpatialRibbonOpen(ui.container, { kind: "path", path: "related.md", family: "declared" }); revealLine("related.md", 1); });
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:related.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-17"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-17">
// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("restores the folder's spatial state without depending on browser storage", async () => {
  const { emptySpatialState } = await import("./spatialLayout");
  function Remembered() {
    const [layout, setLayout] = useState<Layout>(() => {
      const pane = emptyPane();
      return { root: { ...pane, tabs: [tab("document", "main.md"), tab("document", "related.md"), tab("chat", "conversation", "Agent")], active: 0 }, focus: pane.id };
    });
    const [scene, setScene] = useState<SpatialState>(() => ({ ...emptySpatialState(), center: "document:related.md", columns: [.30, .20] as [number, number], auto: true, drawerPins: { left: true, right: false } }));
    return <SpatialShellView layout={layout} onLayout={setLayout} sources={[]} files={[]} links={[]} spatialState={scene} onSpatialState={setScene} spatialHydrated render={entry => <Buffer entry={entry} />} />;
  }
  const ui = render(<Remembered />);
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:related.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
  expect(ui.getByRole("button", { name: "Automatic accordion" }).getAttribute("aria-pressed")).toBe("true");
  fireEvent.click(within(ui.getByRole("tablist", { name: "Center views" })).getByRole("tab", { name: "main.md" }));
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:main.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-18"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-18">
it("forwards right-clicks on center tabs and drawer filenames without selecting them", () => {
  const pane = emptyPane();
  const first = tab("document", "main.md"), second = tab("file", "input.txt");
  const layout: Layout = { root: { ...pane, tabs: [first, second], active: 0 }, focus: pane.id };
  const picked: string[] = [];
  const ui = render(<SpatialShellView layout={layout} onLayout={() => {}} sources={[]} files={[]} links={[]}
    render={() => <p>Editor</p>} onTabContextMenu={(event, target) => { event.preventDefault(); picked.push(target.id); }} />);
  const centerTab = ui.container.querySelector<HTMLElement>(`#spatial-tab-${second.id}`)!;
  fireEvent.contextMenu(centerTab);
  expect(picked).toEqual([second.id]);
  expect(centerTab.getAttribute("aria-selected")).toBe("false");
  const drawerTab = ui.container.querySelector<HTMLElement>('[data-spatial-key="file:input.txt"] .spatial-filename')!;
  fireEvent.contextMenu(drawerTab);
  expect(picked).toEqual([second.id, second.id]);
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-19"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-19">
it("hides Files windows immediately while preserving the file list and editor buffers", async () => {
  const ui = render(<Harness related files />);
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  const original = live.get("related.md");
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  await waitFor(() => expect(related.hidden).toBe(false));
  act(() => saveFilesWindows(false));
  await waitFor(() => expect(related.hidden).toBe(true));
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files="left"]')!.hidden).toBe(false);
  expect(live.get("related.md")).toBe(original);
  act(() => saveFilesWindows(true));
  await waitFor(() => expect(related.hidden).toBe(false));
});

</storycode:copy>
<a id="f-57fb3ab47480-expression_statement-20"></a>
### `expression_statement`

<storycode:copy id="f-57fb3ab47480-expression_statement-20">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("returns the actual pending close outcome and keeps layout reading addressable", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let finish!: (value: import("../lib/uiActions").UiOutcome) => void;
  const calls: string[][] = []; let closeSignal: AbortSignal | undefined;
  const onClose = (pane: string, tab: string, signal?: AbortSignal) => { closeSignal = signal; calls.push([pane, tab]); return new Promise<import("../lib/uiActions").UiOutcome>(resolve => { finish = resolve; }); };
  render(<UiActionContext.Provider value={registry}><Harness onClose={onClose} /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(group => group.startsWith("spatial-layout:"))!;
  const action = (label: string) => registry.observe(parent, 64).actions.find(action => action.label === label)!;
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action("Close spatial view").handle, arguments: { key: "document:main.md" }, request_id: "close" }); });
  expect(calls).toHaveLength(1);
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action("Read spatial layout bytes").handle, arguments: { offset: 0, limit: 8192 }, request_id: "read-pending" })).toMatchObject({ status: "result" }); });
  expect(registry.cancel(registry.scope, { id: "python:fixture", kind: "agent" }, "close")).toBe(true);
  expect(closeSignal?.aborted).toBe(true);
  await act(async () => { finish({ status: "completed", value: { tab_close_requested: false, cancelled: true } }); expect(await pending).toMatchObject({ status: "result", outcome: { value: { tab_close_requested: false, cancelled: true } } }); });
  expect(live.has("main.md")).toBe(true);
});
</storycode:copy>

### Assemble `apps/web/src/shell/SpatialShellView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/SpatialShellView.test.tsx"><storycode:paste select="#f-57fb3ab47480-expression_statement"/><storycode:paste select="#f-57fb3ab47480-expression_statement-2"/><storycode:paste select="#f-57fb3ab47480-live"/><storycode:paste select="#f-57fb3ab47480-Buffer"/><storycode:paste select="#f-57fb3ab47480-Harness"/><storycode:paste select="#f-57fb3ab47480-expression_statement-3"/><storycode:paste select="#f-57fb3ab47480-expression_statement-4"/><storycode:paste select="#f-57fb3ab47480-expression_statement-5"/><storycode:paste select="#f-57fb3ab47480-expression_statement-6"/><storycode:paste select="#f-57fb3ab47480-expression_statement-7"/><storycode:paste select="#f-57fb3ab47480-expression_statement-8"/><storycode:paste select="#f-57fb3ab47480-expression_statement-9"/><storycode:paste select="#f-57fb3ab47480-expression_statement-10"/><storycode:paste select="#f-57fb3ab47480-expression_statement-11"/><storycode:paste select="#f-57fb3ab47480-expression_statement-12"/><storycode:paste select="#f-57fb3ab47480-expression_statement-13"/><storycode:paste select="#f-57fb3ab47480-expression_statement-14"/><storycode:paste select="#f-57fb3ab47480-expression_statement-15"/><storycode:paste select="#f-57fb3ab47480-expression_statement-16"/><storycode:paste select="#f-57fb3ab47480-expression_statement-17"/><storycode:paste select="#f-57fb3ab47480-expression_statement-18"/><storycode:paste select="#f-57fb3ab47480-expression_statement-19"/><storycode:paste select="#f-57fb3ab47480-expression_statement-20"/></storycode:file>


</storycode:doc>
