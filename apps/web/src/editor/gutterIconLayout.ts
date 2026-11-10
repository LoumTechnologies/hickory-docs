import { EditorView, ViewPlugin, type ViewUpdate } from "@codemirror/view";
import { Facet } from "@codemirror/state";

// Dedicated number/fold/breakpoint columns can retain their own layout.
export const layoutEditorGutters = Facet.define<boolean, boolean>({ combine: values => values.every(Boolean) });

import { gutterSlots } from "../lib/cardRail";
export { gutterSlots } from "../lib/cardRail";
const SIZE = 20;
type Icon = { node: HTMLElement; cell: HTMLElement; left?: number; top?: number; offset: number; width: number; height: number; style: string; tabindex: string | null };
type Row = { gutter: HTMLElement; top: number; left: number; icons: Icon[]; available: number };

/** Keep marker DOM in its owning gutter: delegated handlers and drag identity survive. */
export const gutterIconLayout = ViewPlugin.fromClass(class {
  size = SIZE;
  icons = new Map<HTMLElement, Icon>();
  cells = new Map<HTMLElement, string>();
  buttons: HTMLButtonElement[] = [];
  buttonGutters = new WeakMap<HTMLButtonElement, HTMLElement>();
  panel: HTMLDivElement | null = null;
  raised: { node: HTMLElement; zIndex: string } | null = null;
  trigger: HTMLButtonElement | null = null;
  popupIcons: Icon[] = [];
  destroyed = false;
  observer: MutationObserver | null = null;
  observedWrapper: Element | null = null;
  constructor(readonly view: EditorView) {
    view.dom.ownerDocument.addEventListener("keydown", this.keyboard);
    for (const type of ["mousedown", "click", "dblclick"]) view.dom.addEventListener(type, this.retarget, true);
    view.dom.ownerDocument.addEventListener("pointerdown", this.outside, true);
    view.scrollDOM.addEventListener("scroll", this.scroll);
    this.observeRail();
    this.measure();
  }
  observeRail() {
    const view = this.view;
    const wrapper = view.dom.closest(".editor-cm-host")?.parentElement;
    if (this.observer && wrapper !== this.observedWrapper) { this.observer.disconnect(); this.observer = null; }
    if (!this.observer && wrapper && typeof MutationObserver !== "undefined") {
      this.observedWrapper = wrapper;
      this.observer = new MutationObserver(records => {
        if (records.some(record => record.type === "attributes"
          ? record.target instanceof Element && record.target.matches(".cm-debug-rail__group") && record.oldValue !== record.target.getAttribute(record.attributeName!)
          : record.target instanceof Element && record.target.closest(".cm-debug-rail__group") || Array.from(record.addedNodes).some(node => node instanceof Element && node.matches(".cm-debug-rail, .cm-debug-rail__group")))) this.measure();
      });
      this.observer.observe(wrapper, { childList: true, subtree: true, attributes: true, attributeFilter: ["style", "class"], attributeOldValue: true });
    }
  }
  update(update: ViewUpdate) { this.observeRail(); if (update.docChanged || update.viewportChanged || update.geometryChanged || update.transactions.length) { this.close(false); this.measure(); } }
  scroll = () => { this.close(false); this.measure(); };
  keyboard = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.panel) { event.preventDefault(); this.close(true); }
    const node = event.target instanceof HTMLElement ? event.target : null;
    if (node && this.icons.has(node) && !node.matches("button, a, input") && ["Enter", " "].includes(event.key)) {
      event.preventDefault();
      node.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
      node.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
    }
  };
  outside = (event: PointerEvent) => {
    if (this.panel && event.target instanceof Node && !this.panel.contains(event.target) && !this.popupIcons.some(icon => icon.node.contains(event.target as Node)) && !this.trigger?.contains(event.target)) this.close(false);
  };
  forwarded = new WeakSet<Event>();
  retarget = (event: Event) => {
    if (!(event instanceof MouseEvent) || this.forwarded.has(event)) return;
    const node = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-gutter-icon]") : null;
    const icon = node && this.icons.get(node);
    if (!icon) return;
    const rect = icon.cell.getBoundingClientRect();
    const forwarded = new MouseEvent(event.type, { bubbles: true, cancelable: true, clientX: rect.left + rect.width / 2, clientY: rect.top + this.view.defaultLineHeight / 2, button: event.button, buttons: event.buttons, detail: event.detail, ctrlKey: event.ctrlKey, metaKey: event.metaKey, altKey: event.altKey, shiftKey: event.shiftKey });
    this.forwarded.add(forwarded); event.stopImmediatePropagation(); event.preventDefault();
    (event.target as Element).dispatchEvent(forwarded);
  };
  restore(icon: Icon) {
    icon.node.style.cssText = icon.style;
    if (icon.tabindex === null) icon.node.removeAttribute("tabindex"); else icon.node.setAttribute("tabindex", icon.tabindex);
    icon.node.removeAttribute("data-gutter-icon");
  }
  close(focus: boolean) {
    if (!this.panel) return;
    this.panel.remove(); this.panel = null;
    if (this.raised) { this.raised.node.style.zIndex = this.raised.zIndex; this.raised = null; }
    for (const icon of this.popupIcons) { icon.node.style.visibility = "hidden"; icon.node.tabIndex = -1; }
    this.popupIcons = [];
    this.trigger?.setAttribute("aria-expanded", "false");
    if (focus) this.trigger?.focus();
    this.trigger = null;
  }
  place(icon: Icon, left: number, top: number, popup = false) {
    const node = icon.node;
    node.dataset.gutterIcon = "true";
    Object.assign(node.style, {
      position: popup ? "fixed" : "absolute", left: `${left + (this.size - icon.width) / 2}px`, top: `${top + (this.size - icon.height) / 2}px`,
      width: `${icon.width}px`, height: `${icon.height}px`, lineHeight: `${icon.height}px`,
      minHeight: "0", margin: "0", padding: "0", transform: "none", boxSizing: "border-box",
      textAlign: "center", display: "inline-block", zIndex: popup ? "10002" : "2", visibility: "visible",
    });
    if (node.matches("button, [role=button], a")) node.tabIndex = 0;
  }
  open(button: HTMLButtonElement, icons: Icon[]) {
    this.close(false);
    const panel = document.createElement("div");
    panel.className = "cm-gutter-overflow";
    panel.setAttribute("role", "dialog"); panel.setAttribute("aria-label", "Gutter icons");
    const bounds = button.getBoundingClientRect();
    const columns = Math.max(1, Math.min(4, Math.floor((window.innerWidth - 16) / this.size), icons.length));
    const width = columns * this.size + 12, height = Math.min(Math.ceil(icons.length / columns) * this.size + 12, window.innerHeight - 16);
    const left = Math.max(8, Math.min(bounds.right, window.innerWidth - width - 8));
    const top = Math.max(8, Math.min(bounds.top, window.innerHeight - height - 8));
    panel.style.cssText = `position:fixed;left:${left}px;top:${top}px;width:${width}px;height:${height}px;z-index:10001;background:var(--bg-raised);border:1px solid var(--border);border-radius:6px;box-shadow:0 4px 18px #0005;box-sizing:border-box;overflow:auto`;
    this.buttonGutters.get(button)!.append(panel);
    this.panel = panel; this.trigger = button; this.popupIcons = icons;
    const pane = this.view.dom.closest<HTMLElement>(".spatial-view, .spatial-files");
    if (pane) { this.raised = { node: pane, zIndex: pane.style.zIndex }; pane.style.zIndex = "20"; }
    panel.setAttribute("aria-owns", icons.map(icon => { if (!icon.node.id) icon.node.id = `gutter-icon-${crypto.randomUUID()}`; return icon.node.id; }).join(" "));
    button.setAttribute("aria-expanded", "true");
    // Marker nodes retain their ancestors. Their fixed positions overlay this
    // scrollable panel rather than transferring ownership away from CodeMirror.
    const draw = () => icons.forEach((icon, index) => {
      const y = top + 6 + Math.floor(index / columns) * this.size - panel.scrollTop;
      this.place(icon, left + 6 + index % columns * this.size, y, true);
      const visible = y >= top + 5 && y + this.size <= top + height - 5;
      icon.node.style.visibility = visible ? "visible" : "hidden";
      icon.node.tabIndex = visible ? 0 : -1;
    });
    const space = document.createElement("div"); space.style.height = `${Math.ceil(icons.length / columns) * this.size + 12}px`; panel.append(space);
    panel.onscroll = draw; draw();
    icons[0]?.node.focus();
  }
  measure() {
    this.view.requestMeasure({ key: this, read: () => {
      this.size = Math.min(SIZE, this.view.defaultLineHeight);
      const gutterColumns = [
        ...(this.view.state.facet(layoutEditorGutters) ? this.view.dom.querySelectorAll<HTMLElement>(".cm-gutters") : []),
        ...this.view.dom.closest(".editor-cm-host")?.parentElement?.querySelectorAll<HTMLElement>(".cm-debug-rail") ?? [],
      ];
      const viewport = this.view.scrollDOM.getBoundingClientRect();
      const rows = new Map<string, Row>();
      for (const [column, gutters] of Array.from(gutterColumns).entries()) {
        const right = gutters.getBoundingClientRect().right;
      const external = gutters.classList.contains("cm-debug-rail");
      for (const cell of gutters.querySelectorAll<HTMLElement>(external ? ".cm-debug-rail__group" : ".cm-gutter:not(.cm-lineNumbers):not(.cm-blame-gutter) > .cm-gutterElement")) {
        if (cell.style.visibility === "hidden" || getComputedStyle(cell).display === "none") continue; // CodeMirror's width spacer
        const rect = cell.getBoundingClientRect();
        for (const node of (external ? Array.from(cell.querySelectorAll<HTMLElement>("button.debug-icon, .more-actions > summary")).filter(node => !node.closest(".more-actions__items")) : Array.from(cell.children) as HTMLElement[])) {
          if (!node.textContent?.trim() && !node.querySelector("svg") && !node.matches("svg") && !node.dataset.tip && !node.getAttribute("aria-label")) continue;
          if (node.getAttribute("class")?.includes("spacer")) continue;
          let icon = this.icons.get(node);
          if (!icon) {
            const bounds = node.getBoundingClientRect();
            icon = { node, cell, offset: node.dataset.spatialDrag ? Number(node.style.transform.match(/translateY\(([-.\d]+)px\)/)?.[1] ?? 0) : 0, width: Math.min(this.size, bounds.width || this.size), height: Math.min(this.size, bounds.height || this.size), style: node.style.cssText, tabindex: node.getAttribute("tabindex") };
          }
          const top = rect.top + icon.offset;
          if (top + this.size < viewport.top || top >= viewport.bottom) continue;
          const key = `${column}:${Math.round(top * 2) / 2}`;
          let row = rows.get(key);
          if (!row) { row = { gutter: gutters, top, left: external ? gutters.getBoundingClientRect().left : right - this.size, icons: [], available: 0 }; rows.set(key, row); }
          icon.left = rect.left; icon.top = rect.top;
          row.icons.push(icon);
        }
      }
      }
      const result = [...rows.values()].sort((a, b) => Array.from(gutterColumns).indexOf(a.gutter) - Array.from(gutterColumns).indexOf(b.gutter) || a.top - b.top);
      result.forEach((row, index) => row.available = (result[index + 1]?.gutter === row.gutter ? result[index + 1].top : viewport.bottom) - row.top);
      return result;
    }, write: rows => {
      if (this.destroyed) return;
      this.close(false);
      this.buttons.forEach(button => button.remove()); this.buttons = [];
      const current = new Set(rows.flatMap(row => row.icons.map(icon => icon.node)));
      for (const [node, icon] of this.icons) if (!current.has(node)) { this.restore(icon); this.icons.delete(node); }
      const currentCells = new Set(rows.flatMap(row => row.icons.map(icon => icon.cell)));
      for (const [cell, position] of this.cells) if (!currentCells.has(cell)) { cell.style.position = position; this.cells.delete(cell); }
      for (const row of rows) {
        const columns = row.gutter.classList.contains("cm-debug-rail") ? Math.max(1, Math.floor(row.gutter.clientWidth / this.size)) : 1;
        const { visible, overflow } = gutterSlots(row.icons.length, row.available, this.size, columns);
        row.icons.forEach((icon, index) => {
          this.icons.set(icon.node, icon);
          if (!row.gutter.classList.contains("cm-debug-rail")) {
            if (!this.cells.has(icon.cell)) this.cells.set(icon.cell, icon.cell.style.position);
            if (icon.cell.style.position !== "relative") icon.cell.style.position = "relative";
          }
          // Absolute children cannot increase gutter cells or push later lines.
          this.place(icon, row.left - icon.left! + (index % columns) * this.size, row.top - icon.top! + Math.floor(index / columns) * this.size);
          if (index >= visible) { icon.node.style.visibility = "hidden"; icon.node.tabIndex = -1; }
        });
        if (overflow) {
          const button = document.createElement("button");
          button.type = "button"; button.textContent = "⋯";
          button.className = "cm-gutter-more";
          button.setAttribute("aria-label", `Show ${overflow} more gutter icons`);
          button.setAttribute("aria-haspopup", "dialog"); button.setAttribute("aria-expanded", "false");
          const external = row.gutter.classList.contains("cm-debug-rail");
          const parent = external ? row.gutter : this.view.dom;
          const host = parent.getBoundingClientRect();
          button.style.cssText = `position:absolute;left:${row.left - host.left + (visible % columns) * this.size}px;top:${row.top - host.top + Math.floor(visible / columns) * this.size}px;width:${this.size}px;height:${this.size}px;padding:0;line-height:${this.size}px;z-index:201;background:var(--bg-raised);color:var(--fg);border:0;border-radius:3px;cursor:pointer`;
          button.onclick = () => this.panel && this.trigger === button ? this.close(true) : this.open(button, row.icons.slice(visible));
          parent.append(button); this.buttons.push(button); this.buttonGutters.set(button, row.gutter);
        }
      }
      for (const gutters of this.view.dom.querySelectorAll<HTMLElement>(".cm-gutters")) {
        gutters.removeAttribute("aria-hidden");
        const padding = rows.some(row => row.gutter === gutters) ? `${this.size}px` : "";
        if (gutters.style.paddingRight !== padding) { gutters.style.paddingRight = padding; this.measure(); }
      }
    } });
  }
  destroy() {
    this.destroyed = true; this.observer?.disconnect(); this.close(false);
    this.buttons.forEach(button => button.remove());
    for (const icon of this.icons.values()) this.restore(icon);
    for (const [cell, position] of this.cells) cell.style.position = position;
    this.view.dom.ownerDocument.removeEventListener("keydown", this.keyboard);
    for (const type of ["mousedown", "click", "dblclick"]) this.view.dom.removeEventListener(type, this.retarget, true);
    this.view.dom.ownerDocument.removeEventListener("pointerdown", this.outside, true);
    this.view.scrollDOM.removeEventListener("scroll", this.scroll);
  }
});
