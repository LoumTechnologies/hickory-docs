import { EditorView } from "@codemirror/view";
import type { EditorSelection } from "@codemirror/state";
import { showSpatialExcerpt } from "../editor/spatialExcerpt";
import { reserveSpatialFileRows } from "../editor/spatialFiles";
import { samePath } from "../lib/paths";
import { workspaceRegions } from "./spatialDrawers";
import { bandBetween } from "./Ribbons";
import { relationBand, type SpatialRelation } from "./spatialRelations";
import { treeEndpoint } from "./spatialTerminals";
import { clearSpatialSlot, intersects, parallaxAnchor, resizeSpatialPin, type Rect, type Side, type SpatialState } from "./spatialLayout";

export interface SpatialItem { key: string; path: string; side: Side; port?: boolean }
interface Route { animation: Animation; from: Rect; target: Rect; side: Side }
interface Gesture { key: string; pointer: number; startX: number; startY: number; start: Rect; side: Side; kind: "drag" | "top" | "bottom"; lastY: number; moved: boolean; preview?: Rect; neighbor?: string; initialViews: SpatialState["views"]; selections: [EditorView, EditorSelection][] }
export interface SpatialAdapter {
  root: () => HTMLElement | null;
  state: () => SpatialState;
  items: () => readonly SpatialItem[];
  relations: () => readonly SpatialRelation[];
  update: (make: (state: SpatialState) => SpatialState) => void;
}
const setRect = (el: HTMLElement, r: Rect) => Object.assign(el.style, { left: `${r.x}px`, top: `${r.y}px`, width: `${Math.max(20, r.w)}px`, height: `${Math.max(20, r.h)}px` });
const rectOf = (el: HTMLElement, root: HTMLElement): Rect => { const r = el.getBoundingClientRect(), base = root.getBoundingClientRect(); return { x: r.left - base.left, y: r.top - base.top, w: r.width, h: r.height }; };
const editorOf = (el?: HTMLElement | null) => { const dom = el?.querySelector<HTMLElement>(".cm-editor"); return dom ? EditorView.findFromDOM(dom) ?? undefined : undefined; };

/** Pixels are presentation. Editors, files, and provenance remain owned by the workspace. */
export class SpatialController {
  frames = new Map<string, Rect>();
  gesture: Gesture | null = null;
  suppressClick = false;
  private routes = new Map<string, Route>();
  private gates = new Map<string, Map<string, boolean>>();
  private hold: { scroll: number; frames: Map<string, Rect> } | null = null;
  private released = new Map<string, { r: Rect; scroll: number }>();
  private raf = 0;
  private timer = 0;
  private disposed = false;
  private size = "";
  private centerKey: string | null = null;
  constructor(readonly adapter: SpatialAdapter) {}
  start() {
    const root = this.adapter.root(); if (!root) return () => {};
    this.disposed = false;
    const observer = new ResizeObserver(() => this.schedule()); observer.observe(root);
    const scroll = (event: Event) => { if ((event.target as HTMLElement).closest("[data-spatial-center=true]")) this.hold = null; this.schedule(); };
    root.addEventListener("scroll", scroll, true);
    root.addEventListener("input", this.schedule, true);
    root.addEventListener("click", this.schedule, true);
    const move = (event: PointerEvent) => this.move(event), finish = () => this.finish(), cancel = () => this.finish(true);
    window.addEventListener("pointermove", move); window.addEventListener("pointerup", finish); window.addEventListener("pointercancel", cancel);
    this.timer = window.setInterval(this.schedule, 300); this.schedule();
    return () => { this.disposed = true; observer.disconnect(); root.removeEventListener("scroll", scroll, true); root.removeEventListener("input", this.schedule, true); root.removeEventListener("click", this.schedule, true); window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", finish); window.removeEventListener("pointercancel", cancel); window.clearInterval(this.timer); cancelAnimationFrame(this.raf); this.raf = 0; this.stopRoutes(); };
  }
  schedule = () => { if (this.raf || this.disposed) return; this.raf = requestAnimationFrame(() => { this.raf = 0; this.paint(); if (this.routes.size) this.schedule(); }); };
  private elements() { const root = this.adapter.root(); return new Map([...root?.querySelectorAll<HTMLElement>("[data-spatial-key]") ?? []].map(el => [el.dataset.spatialKey!, el])); }
  private centerEditor() { const root = this.adapter.root(), center = this.elements().get(this.adapter.state().center ?? ""); return editorOf(center) ?? editorOf(root?.querySelector<HTMLElement>("[data-spatial-center=true]")); }
  private scroll() { return this.centerEditor()?.scrollDOM.scrollTop ?? 0; }
  private selections(): [EditorView, EditorSelection][] { return [...this.elements().values()].flatMap(el => { const view = editorOf(el); return view ? [[view, view.state.selection] as [EditorView, EditorSelection]] : []; }); }
  preserveSelections() { const saved = this.selections(); requestAnimationFrame(() => this.restoreSelections(saved)); }
  private restoreSelections(saved: [EditorView, EditorSelection][]) { for (const [view, selection] of saved) if (view.dom.isConnected && !view.state.selection.eq(selection) && selection.main.to <= view.state.doc.length) view.dispatch({ selection }); }
  private stopRoutes() { for (const [key, run] of this.routes) { const el = this.elements().get(key); if (el) { setRect(el, rectOf(el, this.adapter.root()!)); el.classList.remove("spatial-view--crossing"); el.style.visibility = ""; } run.animation.cancel(); } this.routes.clear(); }
  private snapshot() { const root = this.adapter.root()!; this.hold = { scroll: this.scroll(), frames: new Map([...this.elements()].filter(([, el]) => !el.hidden).map(([key, el]) => [key, rectOf(el, root)])) }; this.stopRoutes(); }
  pin(key: string) {
    this.snapshot(); const state = this.adapter.state(), r = this.hold!.frames.get(key); if (!r) return;
    const side = state.views[key]?.side ?? this.adapter.items().find(item => item.key === key)?.side ?? "left";
    if (state.views[key]?.pin) {
      this.released.set(key, { r, scroll: this.scroll() });
      this.adapter.update(current => ({ ...current, views: { ...current.views, [key]: { ...current.views[key], side, pin: undefined } } }));
      return;
    }
    this.released.delete(key);
    // A pin must land where it can be seen: the frame a parallax view was
    // flowing through can sit far off the lane while its source is scrolled
    // out, and freezing it there would pin the document out of sight.
    const root = this.adapter.root(); if (!root) return;
    const lane = workspaceRegions(root.clientWidth, root.clientHeight, state)[side];
    const occupied = Object.entries(state.views).filter(([id, view]) => id !== key && view.side === side && view.pin).map(([, view]) => view.pin!);
    const pinned = clearSpatialSlot(r, occupied, lane.y, lane.y + lane.h)
      ?? { ...r, y: Math.max(lane.y, Math.min(lane.y + lane.h - Math.min(r.h, lane.h), r.y)), h: Math.min(r.h, lane.h) };
    this.adapter.update(current => ({ ...current, views: { ...current.views, [key]: { ...current.views[key], side, pin: pinned } } }));
  }
  toggle(key: string) {
    const state = this.adapter.state(), placement = state.views[key], current = this.frames.get(key), expanded = placement?.pin ? placement.pin.h > 36 : placement?.expanded === true;
    this.hold = null; this.released.delete(key);
    if (placement?.pin) this.resize(key, (expanded ? 36 : Math.max(190, placement.height ?? 253)) - placement.pin.h, "bottom");
    this.adapter.update(old => ({ ...old, views: { ...old.views, [key]: { ...old.views[key], side: placement?.side ?? this.adapter.items().find(item => item.key === key)?.side ?? "left", expanded: !expanded, height: expanded ? current?.h ?? 253 : placement?.height } } }));
  }
  resize(key: string, delta: number, edge: "top" | "bottom") {
    const root = this.adapter.root(); if (!root) return;
    this.stopRoutes(); this.hold = null;
    const current = this.adapter.state(), side = current.views[key]?.side ?? this.adapter.items().find(item => item.key === key)?.side ?? "left", r = this.frames.get(key);
    if (!r) return;
    const regions = workspaceRegions(root.clientWidth, root.clientHeight, current);
    this.adapter.update(state => ({ ...state, views: resizeSpatialPin({ ...state.views, [key]: { ...state.views[key], side, pin: state.views[key]?.pin ?? r } }, key, delta, edge, regions[side]) }));
  }
  begin(key: string, event: React.PointerEvent, kind: Gesture["kind"] = "drag") {
    const root = this.adapter.root(), el = this.elements().get(key); if (!root || !el || event.button !== 0) return;
    event.preventDefault(); (event.target as HTMLElement).setPointerCapture(event.pointerId);
    this.snapshot(); const r = rectOf(el, root), side = this.adapter.state().views[key]?.side ?? el.dataset.spatialSide as Side;
    this.gesture = { key, pointer: event.pointerId, startX: event.clientX, startY: event.clientY, lastY: event.clientY, start: r, side, kind, moved: false, neighbor: kind === "bottom" ? (event.currentTarget as HTMLElement).dataset.resizeNeighbor : undefined, initialViews: this.adapter.state().views, selections: this.selections() };
  }
  move(event: Pick<PointerEvent, "pointerId" | "clientX" | "clientY">) {
    const g = this.gesture, root = this.adapter.root(); if (!g || event.pointerId !== g.pointer || !root) return;
    const dy = event.clientY - g.startY; if (!g.moved && Math.hypot(event.clientX - g.startX, dy) < 4) return;
    if (!g.moved && g.neighbor && dy < 0) { g.key = g.neighbor; g.kind = "top"; }
    g.moved = true;
    if (g.kind !== "drag") { this.resize(g.key, event.clientY - g.lastY, g.kind); g.lastY = event.clientY; return; }
    const regions = workspaceRegions(root.clientWidth, root.clientHeight, this.adapter.state()), base = root.getBoundingClientRect();
    g.side = event.clientX - base.left < regions.center.x + regions.center.w / 2 ? "left" : "right";
    if (this.adapter.state().sides?.[g.side] === false) g.side = g.side === "left" ? "right" : "left";
    const lane = regions[g.side], occupied = Object.entries(this.adapter.state().views).filter(([key, view]) => key !== g.key && view.pin).map(([, view]) => view.pin!);
    g.preview = clearSpatialSlot({ x: lane.x, y: g.start.y + dy, w: lane.w, h: Math.min(g.start.h, lane.h) }, occupied, lane.y, lane.y + lane.h)
      ?? clearSpatialSlot({ x: lane.x, y: lane.y + lane.h, w: lane.w, h: 36 }, occupied, lane.y + lane.h, 5000) ?? undefined;
    this.hold = null; this.paint();
  }
  finish(cancel = false) {
    const g = this.gesture; if (!g) return; this.gesture = null;
    if (cancel && g.kind !== "drag" && g.moved) this.adapter.update(state => ({ ...state, views: g.initialViews }));
    if (g.kind === "drag" && g.moved && g.preview && !cancel) {
      this.snapshot(); const r = g.preview;
      this.adapter.update(state => ({ ...state, views: { ...state.views, [g.key]: { ...state.views[g.key], side: g.side, pin: r } } }));
    }
    if (g.moved) { this.suppressClick = true; window.setTimeout(() => { this.suppressClick = false; }, 0); }
    if (g.moved) { this.restoreSelections(g.selections); requestAnimationFrame(() => this.restoreSelections(g.selections)); }
    this.schedule();
  }
  private related(item: SpatialItem, center: HTMLElement | undefined, pivot: number) {
    const candidates = this.adapter.relations().flatMap(relation => {
      if (center && ((relation.from && center.contains(relation.from.dom)) || (relation.anchor && center.contains(relation.anchor))) && samePath(relation.toPath, item.path)) {
        const band = relationBand(relation); return band ? [{ y: (band[0] + band[1]) / 2, range: relation.toRange }] : [];
      }
      if (center && relation.to && center.contains(relation.to.dom) && samePath(relation.fromPath, item.path)) { const band = relationBand(relation, true); return band ? [{ y: (band[0] + band[1]) / 2, range: relation.fromRange }] : []; }
      return [];
    }).sort((a, b) => Math.abs(a.y - pivot) - Math.abs(b.y - pivot));
    return candidates[0];
  }
  private endpoint(el: HTMLElement, range: [number, number] | undefined, expanded: boolean) {
    const view = editorOf(el); if (!view) return 18;
    showSpatialExcerpt(view, expanded && range ? range[0] : null, expanded);
    if (!expanded || !range) return 18;
    const n = Math.min(range[0], view.state.doc.length);
    if (!view.dom.getBoundingClientRect().height) return 18;
    const wanted = bandBetween(view, n, Math.min(range[1], view.state.doc.length)), clip = view.scrollDOM.getBoundingClientRect();
    if (!wanted) return 18;
    if (wanted[1] <= clip.top || wanted[0] >= clip.bottom) { view.scrollDOM.scrollTop += wanted[0] - clip.top - view.scrollDOM.clientHeight * .25; view.requestMeasure(); return 18; }
    return (Math.max(wanted[0], clip.top) + Math.min(wanted[1], clip.bottom)) / 2 - el.getBoundingClientRect().top;
  }
  paint() {
    const root = this.adapter.root(); if (!root || !root.clientWidth || !root.clientHeight) return;
    const state = this.adapter.state(), items = this.adapter.items().filter(item => item.key === state.center || (state.sides?.[state.views[item.key]?.side ?? item.side] !== false && (!state.drawers || state.drawers.keys.includes(item.key)))), elements = this.elements(), regions = workspaceRegions(root.clientWidth, root.clientHeight, state), base = root.getBoundingClientRect(), center = elements.get(state.center ?? "");
    const visible = new Set(items.map(item => item.key));
    for (const [key, element] of elements) if (!visible.has(key)) element.hidden = true;
    for (const [key, route] of this.routes) if (!elements.has(key) || !visible.has(key)) { route.animation.cancel(); this.routes.delete(key); const el = elements.get(key); if (el) { el.classList.remove("spatial-view--crossing"); el.style.visibility = ""; } }
    if (this.centerKey !== state.center) { this.centerKey = state.center; this.hold = null; this.released.clear(); this.gates.clear(); this.stopRoutes(); }
    const size = `${root.clientWidth}:${root.clientHeight}:${state.columns}:${state.sides?.left}:${state.sides?.right}:${JSON.stringify(state.drawers)}`;
    if (this.size !== size) {
      this.size = size; this.hold = null; this.released.clear(); this.stopRoutes();
      const occupied: Rect[] = [], views = { ...state.views }; let changed = false;
      for (const item of items) { const view = views[item.key]; if (!view?.pin || item.key === state.center) continue; const lane = regions[view.side], p = view.pin;
        const r = clearSpatialSlot({ x: lane.x, y: Math.max(lane.y, Math.min(lane.y + lane.h - p.h, p.y)), w: lane.w, h: Math.min(p.h, lane.h) }, occupied, lane.y, lane.y + lane.h)
          ?? clearSpatialSlot({ x: lane.x, y: lane.y + lane.h, w: lane.w, h: 36 }, occupied, lane.y + lane.h, 5000);
        if (r) { occupied.push(r); if (r.x !== p.x || r.y !== p.y || r.w !== p.w || r.h !== p.h) { views[item.key] = { ...view, pin: r }; changed = true; } }
      }
      if (changed) { this.adapter.update(current => ({ ...current, views })); this.schedule(); return; }
    }
    if (center) { center.hidden = false; center.dataset.spatialCenter = "true"; center.dataset.expanded = "true"; setRect(center, regions.center); const view = editorOf(center); if (view) showSpatialExcerpt(view, null); }
    for (const side of ["left", "right"] as const) { const rail = root.querySelector<HTMLElement>(`[data-spatial-files=${side}]`); if (rail) { rail.hidden = state.sides?.[side] === false || state.drawers?.files[side] === false; setRect(rail, regions[`${side}Rail`]); } }
    if ("drawers" in regions) for (const side of ["left", "right"] as const) {
      const box = regions.drawers[side];
      const background = root.querySelector<HTMLElement>(`[data-drawer-backdrop=${side}]`), toolbar = root.querySelector<HTMLElement>(`[data-drawer-toolbar=${side}]`);
      if (background) setRect(background, box);
      if (toolbar) setRect(toolbar, { ...box, h: 36 });
    }
    const minimap = root.querySelector<HTMLElement>(".spatial-minimap"); if (minimap) minimap.style.left = `${regions.center.x + regions.center.w - 7}px`;
    for (const side of ["left", "right"] as const) { const rule = root.querySelector<HTMLElement>(`[data-spatial-column=${side}]`); if (rule) { rule.hidden = state.sides?.[side] === false || (state.drawers ? !state.drawers.pinned[side] : regions.small); setRect(rule, { x: side === "left" ? regions.center.x - regions.gap / 2 - 4 : regions.center.x + regions.center.w + regions.gap / 2 - 4, y: regions.center.y, w: 8, h: regions.center.h }); rule.style.width = "8px"; } }
    const editor = this.centerEditor(), sourceBox = editor?.scrollDOM.getBoundingClientRect(), pivot = sourceBox ? (sourceBox.top + sourceBox.bottom) / 2 : base.top + regions.center.h / 2, scroll = this.scroll(), planned = new Map<string, { r: Rect; side: Side; from?: Rect }>(), occupied: Rect[] = [];
    if (this.hold?.scroll !== scroll) this.hold = null;
    for (const item of items) { const p = state.views[item.key]; if (item.key === state.center || !p?.pin || this.gesture?.key === item.key && this.gesture.moved) continue; planned.set(item.key, { r: p.pin, side: p.side }); occupied.push(p.pin); }
    for (const [key, run] of this.routes) { occupied.push(run.from, run.target); planned.set(key, { r: run.target, side: run.side }); }
    if (this.gesture?.preview) { planned.set(this.gesture.key, { r: this.gesture.preview, side: this.gesture.side }); occupied.push(this.gesture.preview); }
    const flows = items.filter(item => item.key !== state.center && !planned.has(item.key)).map((item, index) => ({ item, index, relation: this.related(item, center, pivot) })).sort((a, b) => Math.abs((a.relation?.y ?? pivot + a.index * 60) - pivot) - Math.abs((b.relation?.y ?? pivot + b.index * 60) - pivot));
    for (const { item, index, relation } of flows) {
      const el = elements.get(item.key); if (!el) continue;
      const rail = root.querySelector<HTMLElement>(`[data-spatial-files=${state.views[item.key]?.side ?? item.side}]`);
      if (rail && treeEndpoint(rail, item.path)?.folder) { el.hidden = true; continue; }
      const side = state.views[item.key]?.side ?? item.side, lane = regions[side], release = this.released.get(item.key), held = this.hold?.frames.get(item.key), natural = parallaxAnchor((relation?.y ?? pivot + index * 70 - scroll) - base.top, pivot - base.top, sourceBox?.height ?? lane.h), c = release ? release.r.y + release.r.h / 2 - (scroll - release.scroll) * .75 : natural;
      const distance = Math.min(c - lane.y, lane.y + lane.h - c), expanded = state.views[item.key]?.expanded ?? (state.auto && distance > 100), h = held?.h ?? release?.r.h ?? (expanded && !item.port ? Math.min(253, lane.h) : 36);
      el.hidden = false; el.dataset.spatialCenter = "false"; el.dataset.expanded = String(h > 36); el.style.width = `${lane.w}px`; el.style.height = `${h}px`;
      const offset = release || held ? h / 2 : this.endpoint(el, relation?.range, h > 36), requested = held ?? { x: lane.x, y: c - offset, w: lane.w, h }, pins = Object.entries(state.views).filter(([key, p]) => key !== item.key && p.side === side && p.pin), map = this.gates.get(item.key) ?? new Map<string, boolean>(); this.gates.set(item.key, map);
      // Expanded views are a reading/editing surface: they stay inside the
      // lane so a reveal or pin always lands where it can be seen. Collapsed
      // filename views may still flow off the lane as surplus passages do.
      let low = regions.small || h > 36 ? lane.y : -5000, high = regions.small || h > 36 ? lane.y + lane.h : 5000, crossed = false;
      for (const [key, p] of pins) { const above = c < p.pin!.y, old = map.get(key); if (old !== undefined && old !== above) crossed = true; map.set(key, above); if (above) high = Math.min(high, p.pin!.y - 12); else low = Math.max(low, p.pin!.y + p.pin!.h + 12); }
      const r = held ?? clearSpatialSlot(requested, occupied, low, high);
      if (!r) { el.hidden = true; continue; }
      const previous = this.frames.get(item.key), from = crossed && !regions.small && ![...this.routes.values()].some(route => route.side === side) && previous && !occupied.some(rect => intersects(previous, rect, 6)) ? previous : undefined;
      planned.set(item.key, { r, side, from }); occupied.push(r); if (from) occupied.push(from);
    }
    for (const [key, { r, side, from }] of planned) {
      const el = elements.get(key); if (!el) continue;
      el.hidden = false; el.dataset.spatialCenter = "false"; el.dataset.spatialSide = side; el.dataset.expanded = String(r.h > 36); el.classList.toggle("spatial-view--pinned", !!state.views[key]?.pin); el.classList.toggle("spatial-view--tentative", this.gesture?.key === key && this.gesture.moved); el.style.visibility = "";
      // A pinned view is the full editable buffer, never a provenance excerpt:
      // the excerpt hides lines behind reveal controls, which is right for a
      // flowing context preview and wrong for a document a person pinned to
      // read and edit in place.
      if (state.views[key]?.pin) { const view = editorOf(el); if (view) showSpatialExcerpt(view, null); }
      if (!this.routes.has(key)) {
        setRect(el, r);
        const lane = regions[side], x = side === "left" ? lane.x - 26 : lane.x + lane.w + 2, corridor = from && { x, y: Math.min(from.y, r.y), w: 24, h: Math.max(from.y + from.h, r.y + r.h) - Math.min(from.y, r.y) };
        if (from && corridor && !occupied.some(rect => rect !== from && rect !== r && intersects(corridor, rect)) && !matchMedia("(prefers-reduced-motion: reduce)").matches && typeof el.animate === "function") {
          const animation = el.animate([{ left: `${from.x}px`, top: `${from.y}px`, width: `${from.w}px`, height: `${from.h}px` }, { left: `${x}px`, top: `${from.y}px`, width: "24px", height: "36px", offset: .25 }, { left: `${x}px`, top: `${r.y}px`, width: "24px", height: "36px", offset: .75 }, { left: `${r.x}px`, top: `${r.y}px`, width: `${r.w}px`, height: `${r.h}px` }], { duration: 480, easing: "ease-in-out" });
          this.routes.set(key, { animation, from, target: r, side }); el.classList.add("spatial-view--crossing"); animation.onfinish = () => { this.routes.delete(key); el.classList.remove("spatial-view--crossing"); this.schedule(); };
        }
      }
      this.frames.set(key, r);
    }
    this.paintHeaders(root, planned, regions, base);
    root.dispatchEvent(new CustomEvent("hickory-spatial-frame", { bubbles: true }));
  }
  private paintHeaders(root: HTMLElement, planned: Map<string, { r: Rect; side: Side }>, regions: ReturnType<typeof workspaceRegions>, base: DOMRect) {
    const elements = this.elements(), headers: { key: string; r: Rect; el: HTMLElement; strip: HTMLElement }[] = [];
    for (const strip of root.querySelectorAll<HTMLElement>("[data-spatial-strip]")) {
      const key = strip.dataset.spatialStrip!, item = planned.get(key), el = elements.get(key); strip.hidden = !item || !el || el.hidden;
      if (strip.hidden || !item || !el) continue;
      const r = rectOf(el, root), rail = regions[`${item.side}Rail`], x = item.side === "left" ? rail.x : r.x, right = item.side === "left" ? r.x + r.w : rail.x + rail.w;
      setRect(strip, { x, y: r.y, w: right - x, h: 36 }); strip.dataset.spatialSide = item.side; strip.classList.toggle("spatial-header--pinned", !!this.adapter.state().views[key]?.pin); strip.classList.toggle("spatial-header--tentative", this.gesture?.key === key && this.gesture.moved);
      strip.style.setProperty("--spatial-rail-width", `${rail.w}px`); strip.style.visibility = ""; headers.push({ key, r, el, strip });
    }
    for (const header of headers) if (this.routes.has(header.key) && headers.some(other => other.key !== header.key && other.strip.dataset.spatialSide === header.strip.dataset.spatialSide && header.r.y < other.r.y + 36 && header.r.y + header.r.h > other.r.y)) { header.el.style.visibility = "hidden"; header.strip.style.visibility = "hidden"; }
    for (const side of ["left", "right"] as const) {
      const rail = root.querySelector<HTMLElement>(`[data-spatial-files=${side}]`), view = editorOf(rail); if (!view) continue;
      const reserved = headers.filter(header => header.strip.dataset.spatialSide === side && header.strip.style.visibility !== "hidden").map(header => ({ top: header.r.y + base.top, bottom: header.r.y + base.top + 36 })).sort((a, b) => a.top - b.top);
      reserveSpatialFileRows(view, reserved);
    }
    for (const handle of root.querySelectorAll<HTMLElement>("[data-spatial-resize]")) {
      delete handle.dataset.resizeNeighbor;
      const key = handle.dataset.spatialResize!, item = planned.get(key), r = item && this.frames.get(key); handle.hidden = !r || this.routes.has(key);
      if (!r || !item || handle.hidden) continue;
      const top = handle.dataset.resizeEdge === "top", slot = { x: r.x, y: top ? r.y - 8 : r.y + r.h, w: r.w, h: 8 }, lane = regions[item.side];
      handle.hidden = slot.y < lane.y || slot.y + 8 > lane.y + lane.h || [...planned].some(([id, other]) => id !== key && intersects(slot, other.r)); setRect(handle, slot); handle.style.height = "8px";
      const shared = [...root.querySelectorAll<HTMLElement>("[data-spatial-resize]:not([hidden])")].find(other => other !== handle && other.dataset.resizeEdge === "bottom" && other.dataset.spatialResize !== key && Math.abs(Number.parseFloat(other.style.top) - slot.y) < .5 && Math.abs(Number.parseFloat(other.style.left) - slot.x) < .5);
      if (top && shared) { handle.hidden = true; shared.dataset.resizeNeighbor = key; }
    }
  }
}
