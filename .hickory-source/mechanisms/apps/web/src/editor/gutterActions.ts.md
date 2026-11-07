<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/gutterActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d8b909a5a3f8"></a>
## `apps/web/src/editor/gutterActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d8b909a5a3f8-accessibleGutterActions"></a>
### `accessibleGutterActions`

<storycode:copy id="f-d8b909a5a3f8-accessibleGutterActions">
import { ViewPlugin } from "@codemirror/view";

/** CodeMirror hides decorative gutters from accessibility; ours contain actions. */
export const accessibleGutterActions = ViewPlugin.define(view => {
  const expose = () => view.dom.querySelector(".cm-gutters")?.removeAttribute("aria-hidden");
  view.requestMeasure({ read: () => null, write: expose });
  return { update: expose };
});
</storycode:copy>

### Assemble `apps/web/src/editor/gutterActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/gutterActions.ts"><storycode:paste select="#f-d8b909a5a3f8-accessibleGutterActions"/></storycode:file>


## Stack icons without changing text geometry

Stack icons without changing text geometry

Every editor installs this layout through editorChrome. Measure populated gutter rows together, reserve the next populated row’s center, and use a three-dot trigger when a vertical stack cannot fit. Marker nodes stay in their original gutters so native and delegated actions retain their owner; the floating grid positions those same nodes and closes on Escape, outside pointer presses, scrolling or editor updates. Blame text and line numbers remain annotations. This does not claim that every inherited marker has an accessible label.

Reserve one slot for the overflow trigger before placing any icons. Slot height follows the editor’s line height, capped at twenty pixels, so a dense editor does not make adjacent row controls collide.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts">
import { EditorView, ViewPlugin, type ViewUpdate } from "@codemirror/view";

import { gutterSlots } from "../lib/cardRail";
export { gutterSlots } from "../lib/cardRail";
const SIZE = 20;
type Icon = { node: HTMLElement; cell: HTMLElement; left?: number; top?: number; offset: number; width: number; height: number; style: string; tabindex: string | null };
type Row = { gutter: HTMLElement; top: number; left: number; icons: Icon[]; available: number };

</storycode:copy>

The layout belongs to the editor, while each marker retains its original DOM owner. Close transient state on content, viewport or geometry changes; focus alone must not immediately close a newly opened grid.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-1">
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
  constructor(readonly view: EditorView) {
    view.dom.ownerDocument.addEventListener("keydown", this.keyboard);
    for (const type of ["mousedown", "click", "dblclick"]) view.dom.addEventListener(type, this.retarget, true);
    view.dom.ownerDocument.addEventListener("pointerdown", this.outside, true);
    view.scrollDOM.addEventListener("scroll", this.scroll);
    const wrapper = view.dom.closest(".editor-cm-host")?.parentElement;
    if (wrapper && typeof MutationObserver !== "undefined") {
      this.observer = new MutationObserver(records => {
        if (records.some(record => record.type === "attributes"
          ? record.target instanceof Element && record.target.matches(".cm-debug-rail__group") && record.oldValue !== record.target.getAttribute(record.attributeName!)
          : record.target instanceof Element && record.target.closest(".cm-debug-rail__group") || Array.from(record.addedNodes).some(node => node instanceof Element && node.matches(".cm-debug-rail, .cm-debug-rail__group")))) this.measure();
      });
      this.observer.observe(wrapper, { childList: true, subtree: true, attributes: true, attributeFilter: ["style", "class"], attributeOldValue: true });
    }
    this.measure();
  }
  update(update: ViewUpdate) { if (update.docChanged || update.viewportChanged || update.geometryChanged || update.transactions.length) { this.close(false); this.measure(); } }
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
</storycode:copy>

A displaced icon still refers to its source line. Reissue mouse events at that gutter cell’s coordinates so delegated breakpoint and fold handlers cannot select a later line underneath the displayed icon.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-2">
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
</storycode:copy>

Restore original styles and keyboard state when a marker retires. Dismissing the grid hides its overflow items and optionally returns focus to the three-dot trigger.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-3">
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
</storycode:copy>

Position the existing marker within a fixed slot without increasing its gutter cell’s height. Preserve the measured symbol dimensions so a CSS breakpoint dot does not become a twenty-pixel circle.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-4">
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
</storycode:copy>

The overflow grid is an overlay, not a text widget. Temporarily raise its containing pane so a minimap or sibling pane cannot intercept its controls; restore the pane’s prior stacking when it closes. Its panel is clamped to the viewport and scrolls when necessary. Icons keep their delegated ancestors; aria-owns relates them to the floating dialog without transferring CodeMirror’s DOM ownership.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-5">
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
</storycode:copy>

Read all populated rows together before writing positions, including the separate debugger action column. Observe actual group changes without treating repeated identical placement writes as changes. Exclude width spacers and textual annotations, but include icons drawn only with CSS and described through data-tip or aria-label. Translated preview grips retain their visual anchor.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-6">
  measure() {
    this.view.requestMeasure({ key: this, read: () => {
      this.size = Math.min(SIZE, this.view.defaultLineHeight);
      const gutterColumns = [
        ...this.view.dom.querySelectorAll<HTMLElement>(".cm-gutters"),
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
          if (!row) { row = { gutter: gutters, top, left: right - this.size, icons: [], available: 0 }; rows.set(key, row); }
          icon.left = rect.left; icon.top = rect.top;
          row.icons.push(icon);
        }
      }
      }
      const result = [...rows.values()].sort((a, b) => Array.from(gutterColumns).indexOf(a.gutter) - Array.from(gutterColumns).indexOf(b.gutter) || a.top - b.top);
      result.forEach((row, index) => row.available = (result[index + 1]?.gutter === row.gutter ? result[index + 1].top : viewport.bottom) - row.top);
      return result;
</storycode:copy>

Write absolute positions only after measuring. Reserve the next populated row’s first slot; remaining icons stay hidden until their three-dot trigger opens the grid. Reserve a rightmost gutter lane only when icons exist.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-7">
    }, write: rows => {
      if (this.destroyed) return;
      this.close(false);
      this.buttons.forEach(button => button.remove()); this.buttons = [];
      const current = new Set(rows.flatMap(row => row.icons.map(icon => icon.node)));
      for (const [node, icon] of this.icons) if (!current.has(node)) { this.restore(icon); this.icons.delete(node); }
      const currentCells = new Set(rows.flatMap(row => row.icons.map(icon => icon.cell)));
      for (const [cell, position] of this.cells) if (!currentCells.has(cell)) { cell.style.position = position; this.cells.delete(cell); }
      for (const row of rows) {
        const { visible, overflow } = gutterSlots(row.icons.length, row.available, this.size);
        row.icons.forEach((icon, index) => {
          this.icons.set(icon.node, icon);
          if (!row.gutter.classList.contains("cm-debug-rail")) {
            if (!this.cells.has(icon.cell)) this.cells.set(icon.cell, icon.cell.style.position);
            if (icon.cell.style.position !== "relative") icon.cell.style.position = "relative";
          }
          // Absolute children cannot increase gutter cells or push later lines.
          this.place(icon, row.left - icon.left!, row.top - icon.top! + index * this.size);
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
          button.style.cssText = `position:absolute;left:${row.left - host.left}px;top:${row.top - host.top + visible * this.size}px;width:${this.size}px;height:${this.size}px;padding:0;line-height:${this.size}px;z-index:201;background:var(--bg-raised);color:var(--fg);border:0;border-radius:3px;cursor:pointer`;
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
</storycode:copy>

Retirement removes transient panels, triggers and listeners, then returns marker styling to its original owner.

<storycode:copy id="just-gutter-apps-web-src-editor-gutterIconLayout-ts-8">
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
</storycode:copy>

<storycode:file path="apps/web/src/editor/gutterIconLayout.ts"><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-1"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-2"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-3"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-4"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-5"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-6"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-7"/><storycode:paste select="#just-gutter-apps-web-src-editor-gutterIconLayout-ts-8"/></storycode:file>


## Discover saved recipes on folder rows

Discover saved recipes on folder rows

Ask the host to parse each saved justfile, without running recipes. Ignore responses after an editor closes or its text changes. Edits clear the markers instead of launching commands against a draft folder name. The containing folder owns the buttons; the workspace root uses the same gutter in a read-only header row.

Recipe metadata comes from the API rather than a second justfile parser in the browser. Folder rows identify saved paths independently of their visible labels.

<storycode:copy id="just-gutter-apps-web-src-editor-justRecipeGutter-ts">
import { RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import { EditorView, GutterMarker, ViewPlugin, gutter } from "@codemirror/view";
import { api } from "../api/client";

import type { JustRecipe } from "../api/types";
export type { JustRecipe } from "../api/types";

export interface RecipeFolder { line: number; path: string }

</storycode:copy>

Ask for one literal parameter value at a time, including additional variadic values. Cancellation starts nothing. Blank optional values retain the recipe’s defaults; positional defaults cannot be skipped to supply a later parameter. Keep each value separate in the terminal argument vector.

<storycode:copy id="just-gutter-apps-web-src-editor-justRecipeGutter-ts-1">
/** Pass exact argv values; a recipe argument never becomes shell syntax here. */
export async function launchJustRecipe(path: string, recipe: JustRecipe) {
  const args: string[] = [];
  for (const parameter of recipe.parameters) {
    let first = true;
    do {
      const required = first && parameter.required;
      const answer = window.prompt(`Run just ${recipe.name} — ${parameter.name}${required ? "" : " (leave empty to use defaults or finish)"}`, "");
      if (answer === null) return null;
      if (!required && answer === "") break;
      args.push(answer); first = false;
    } while (parameter.variadic);
    // Arguments are positional. Omitting a default also omits later defaults.
    if (first && !parameter.required) break;
  }
  const slash = path.lastIndexOf("/");
  return api.openTerminal({ title: `just ${recipe.name}`, cwd: slash < 0 ? "." : path.slice(0, slash), argv: ["just", "--justfile", path.slice(slash + 1), "--", recipe.name, ...args] });
}

</storycode:copy>

Each button retains the saved justfile path and recipe identity. Its accessible name identifies both recipe and folder, and its tooltip includes the recipe’s documentation.

<storycode:copy id="just-gutter-apps-web-src-editor-justRecipeGutter-ts-2">
class RecipeMarker extends GutterMarker {
  constructor(readonly path: string, readonly recipe: JustRecipe, readonly run: (path: string, recipe: JustRecipe) => void) { super(); }
  eq(other: RecipeMarker) { return other.path === this.path && JSON.stringify(other.recipe) === JSON.stringify(this.recipe); }
  toDOM() {
    const button = document.createElement("button");
    button.type = "button"; button.textContent = "▶";
    button.className = "cm-just-recipe";
    button.title = `just ${this.recipe.name}${this.recipe.doc ? ` — ${this.recipe.doc}` : ""}`;
    button.setAttribute("aria-label", `Run just ${this.recipe.name} in ${this.path.includes("/") ? this.path.slice(0, this.path.lastIndexOf("/")) : "."}`);
    button.onclick = () => this.run(this.path, this.recipe);
    return button;
  }
}

</storycode:copy>

A document edit clears recipe markers and invalidates callbacks held by old DOM references. Asynchronous discovery publishes only into the same live source; closing the editor retires pending results. Empty folders do not dispatch a discovery update.

<storycode:copy id="just-gutter-apps-web-src-editor-justRecipeGutter-ts-3">
/** Saved filesystem identity must never be inferred from an edited tree label. */
export function justRecipeGutter(folders: RecipeFolder[], current: () => boolean, run: (path: string, recipe: JustRecipe) => void, error: (reason: string) => void): Extension {
  if (!folders.length) return [];
  const replace = StateEffect.define<{ line: number; path: string; recipes: JustRecipe[] }[]>();
  const rows = StateField.define<{ line: number; path: string; recipes: JustRecipe[] }[]>({
    create: () => [], update(value, transaction) {
      if (transaction.docChanged) return [];
      for (const effect of transaction.effects) if (effect.is(replace)) value = effect.value;
      return value;
    },
  });
  return [rows, gutter({ side: "after", class: "cm-just-gutter", markers: view => { const source = view.state.doc; return RangeSet.of(view.state.field(rows).flatMap(row =>
    row.line <= view.state.doc.lines ? row.recipes.map(recipe => new RecipeMarker(row.path, recipe, (path, selected) => { if (current() && view.state.doc === source) run(path, selected); }).range(view.state.doc.line(row.line).from)) : []), true); } }),
    ViewPlugin.fromClass(class {
      alive = true;
      constructor(view: EditorView) {
        const source = view.state.doc;
        void Promise.all(folders.map(async folder => {
          try { return { ...folder, recipes: (await api.justRecipes(folder.path)).recipes }; }
          catch (reason) { if (this.alive && current()) error(reason instanceof Error ? reason.message : String(reason)); return { ...folder, recipes: [] }; }
        })).then(result => { if (this.alive && view.state.doc === source && current()) view.dispatch({ effects: replace.of(result) }); });
      }
      destroy() { this.alive = false; }
    }),
    EditorView.theme({ ".cm-just-gutter": { minWidth: "20px" }, ".cm-just-recipe": { color: "var(--ok)", background: "transparent", border: "0", cursor: "pointer", font: "inherit" } }),
  ];
}
</storycode:copy>

<storycode:file path="apps/web/src/editor/justRecipeGutter.ts"><storycode:paste select="#just-gutter-apps-web-src-editor-justRecipeGutter-ts"/><storycode:paste select="#just-gutter-apps-web-src-editor-justRecipeGutter-ts-1"/><storycode:paste select="#just-gutter-apps-web-src-editor-justRecipeGutter-ts-2"/><storycode:paste select="#just-gutter-apps-web-src-editor-justRecipeGutter-ts-3"/></storycode:file>


## Give the workspace root the same recipe gutter

Give the workspace root the same recipe gutter

The workspace root is a read-only one-line editor, not an invented editable filesystem entry. It uses the same discovery and gutter layout as nested folders. Retire asynchronous discovery and terminal callbacks when the header unmounts.

<storycode:copy id="just-gutter-apps-web-src-shell-RecipeFolderHeader-tsx">
import { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorChrome } from "../editor/chrome";
import { justRecipeGutter, launchJustRecipe } from "../editor/justRecipeGutter";

/** A root needs a real gutter row too; it isn't a file in the editable buffer. */
export function RecipeFolderHeader({ name, paths, revision, onOpenTerminal, onError }: {
  name: string; paths: string[]; revision: unknown; onOpenTerminal?: (id: string) => void; onError: (reason: string) => void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const callbacks = useRef({ onOpenTerminal, onError }); callbacks.current = { onOpenTerminal, onError };
  const version = JSON.stringify(paths);
  useEffect(() => {
    if (!host.current) return;
    let alive = true, pending = false;
    const view = new EditorView({ parent: host.current, state: EditorState.create({ doc: name, extensions: [
      editorChrome("code"), EditorState.readOnly.of(true), EditorView.editable.of(false),
      EditorView.contentAttributes.of({ "aria-label": "Workspace folder" }),
      justRecipeGutter(paths.map(path => ({ path, line: 1 })), () => alive, (path, recipe) => {
        if (pending) return;
        pending = true;
        void launchJustRecipe(path, recipe).then(session => { if (alive && session) callbacks.current.onOpenTerminal?.(session.id); })
          .catch(error => { if (alive) callbacks.current.onError(error instanceof Error ? error.message : String(error)); })
          .finally(() => { pending = false; });
      }, reason => callbacks.current.onError(reason)),
      EditorView.theme({ "&": { overflow: "visible" }, ".cm-scroller": { overflow: "visible" }, ".cm-content": { padding: "0" }, ".cm-line": { padding: "0" } }),
    ] }) });
    return () => { alive = false; view.destroy(); };
  }, [name, version, revision]);
  return <div ref={host} style={{ flex: 1, minWidth: 0 }} />;
}
</storycode:copy>

<storycode:file path="apps/web/src/shell/RecipeFolderHeader.tsx"><storycode:paste select="#just-gutter-apps-web-src-shell-RecipeFolderHeader-tsx"/></storycode:file>


## Check source identity and launch arguments

Check source identity and launch arguments

These focused witnesses retire asynchronous scans and already-held button references after tree edits. They also keep arguments as literal argv values and reserve space for overflow before the next populated row. Browser geometry and real native execution are checked separately by the Python system witness.

<storycode:copy id="just-gutter-apps-web-src-editor-justRecipeGutter-test-ts">
// Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { waitFor } from "@testing-library/react";
import { api } from "../api/client";
import { gutterSlots } from "./gutterIconLayout";
import { justRecipeGutter, launchJustRecipe, type JustRecipe } from "./justRecipeGutter";
vi.mock("../api/client", () => ({ api: { justRecipes: vi.fn(), openTerminal: vi.fn() } }));
const views: EditorView[] = [];
afterEach(() => { for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } vi.restoreAllMocks(); vi.mocked(api.justRecipes).mockReset(); vi.mocked(api.openTerminal).mockReset(); });
const recipe: JustRecipe = { name: "build", doc: "Build", parameters: [] };
function setup() {
  const run = vi.fn(), error = vi.fn();
  const view = new EditorView({ parent: document.body.appendChild(document.createElement("div")), state: EditorState.create({ doc: "src/\nnext/", extensions: justRecipeGutter([{ line: 1, path: "src/justfile" }], () => true, run, error) }) });
  views.push(view); return { view, run, error };
}
it("reserves the overflow slot before the next row rather than pushing it down", () => {
  expect(gutterSlots(4, 60)).toEqual({ visible: 2, overflow: 2 });
  expect(gutterSlots(4, 20)).toEqual({ visible: 0, overflow: 4 });
  expect(gutterSlots(3, 60)).toEqual({ visible: 3, overflow: 0 });
});
it("ignores stale discovery and invalidates previously drawn recipe buttons on an edit", async () => {
  vi.mocked(api.justRecipes).mockResolvedValue({ recipes: [recipe] });
  const { view, run } = setup();
  await waitFor(() => expect(view.dom.querySelector(".cm-just-recipe")).not.toBeNull());
  const old = view.dom.querySelector<HTMLButtonElement>(".cm-just-recipe")!;
  old.click(); expect(run).toHaveBeenCalledWith("src/justfile", recipe);
  run.mockClear(); view.dispatch({ changes: { from: 0, to: 3, insert: "elsewhere" } });
  old.click(); expect(run).not.toHaveBeenCalled();
  expect(view.dom.querySelector(".cm-just-recipe")).toBeNull();
  let resolve!: (result: { recipes: JustRecipe[] }) => void;
  vi.mocked(api.justRecipes).mockImplementation(() => new Promise(done => { resolve = done; }));
  const second = setup(); second.view.dispatch({ changes: { from: 0, insert: "draft" } });
  resolve({ recipes: [recipe] }); await Promise.resolve(); await Promise.resolve();
  expect(second.view.dom.querySelector(".cm-just-recipe")).toBeNull();
});
it("passes parameter values as exact argv and cancels without starting a terminal", async () => {
  vi.mocked(api.openTerminal).mockResolvedValue({ id: "terminal" } as never);
  const prompt = vi.spyOn(window, "prompt").mockReturnValueOnce("a b").mockReturnValueOnce("$(touch unwanted)").mockReturnValueOnce("");
  const parameterized = { ...recipe, parameters: [{ name: "input", required: true, variadic: true }] };
  await launchJustRecipe("folder with spaces/justfile", parameterized);
  expect(api.openTerminal).toHaveBeenCalledWith({ title: "just build", cwd: "folder with spaces", argv: ["just", "--justfile", "justfile", "--", "build", "a b", "$(touch unwanted)"] });
  vi.mocked(api.openTerminal).mockClear(); prompt.mockReturnValue(null);
  expect(await launchJustRecipe("justfile", parameterized)).toBeNull(); expect(api.openTerminal).not.toHaveBeenCalled();
  prompt.mockReturnValue("");
  await launchJustRecipe("justfile", { ...recipe, parameters: [{ name: "input", required: false, variadic: false }] });
  expect(api.openTerminal).toHaveBeenCalledWith({ title: "just build", cwd: ".", argv: ["just", "--justfile", "justfile", "--", "build"] });
});
</storycode:copy>

<storycode:file path="apps/web/src/editor/justRecipeGutter.test.ts"><storycode:paste select="#just-gutter-apps-web-src-editor-justRecipeGutter-test-ts"/></storycode:file>


## Drive real recipes and overflowing folder icons

Drive real recipes and overflowing folder icons

The Python witness uses only HTTP and browser gestures against the native disposable workspace. It checks non-evaluating discovery, path refusal, neighboring row geometry, a multirow grid, Escape and outside dismissal, workspace-root controls, and a recipe’s actual filesystem output in the correct folder.

<storycode:copy id="just-gutter-scripts-test-just-gutters-py">
#!/usr/bin/env python3
"""Native HTTP + browser witness for recipe gutters and shared overflow geometry.
Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
Run in the disposable workspace provided by test-spatial-ui.sh --just-gutters.
"""
import json
import sys
import time
from playwright.sync_api import sync_playwright, expect


def main(origin):
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch()
        page = browser.new_page(viewport={"width": 1200, "height": 900})
        request = page.request
        def post(path, data):
            response = request.post(origin + path, data=data)
            assert response.ok, response.text()
            return response.json()
        def file(path, content):
            post("/api/files/op", {"op": "create", "path": path})
            response = request.put(origin + "/api/file", data={"path": path, "content": content})
            assert response.ok, response.text()
        for directory in ["a-recipes", "b-recipes"]:
            post("/api/files/op", {"op": "mkdir", "path": directory})
            file(directory + "/justfile", "value := `touch discovery-ran`\n" + "\n".join(f"recipe{i}:\n    printf 'recipe-{i}-ok' > result.txt\n" for i in range(8)))
        file("justfile", "root-one:\n    printf root > root-result.txt\nroot-two:\n    true\n")
        response = request.get(origin + "/api/just-recipes?path=a-recipes%2Fjustfile")
        assert response.ok, response.text()
        assert len(response.json()["recipes"]) == 8
        assert request.get(origin + "/api/file?path=a-recipes%2Fdiscovery-ran").status == 404
        assert request.get(origin + "/api/just-recipes?path=..%2Fjustfile").status == 400
        page.goto(origin)
        page.evaluate("window.dispatchEvent(new CustomEvent('hickory-menu', {detail:'files'}))")
        files = page.locator('[data-spatial-files=left]')
        handle = page.get_by_role("button", name="Show left drawer")
        if handle.is_visible(): handle.click()
        expect(files).to_be_visible()
        tree = files.locator(".folder-tree__list[role=tree]").first
        a = tree.locator('[data-tree-path="a-recipes"]')
        b = tree.locator('[data-tree-path="b-recipes"]')
        expect(a).to_be_visible()
        expect(b).to_be_visible()
        expect(tree.locator(".cm-gutter-more").first).to_be_visible()
        # Fold controls and recipes share one column. The following directory
        # keeps its original row center while the first row opens a grid.
        before = {"a": a.bounding_box(), "b": b.bounding_box()}
        more = tree.locator(".cm-gutter-more").first
        more.click()
        expect(tree.get_by_role("dialog", name="Gutter icons")).to_be_visible()
        for key, line in [("a", a), ("b", b)]:
            after = line.bounding_box()
            for dimension in ["x", "y", "width", "height"]:
                assert abs(after[dimension] - before[key][dimension]) < 1, (key, before[key], after)
        buttons = tree.locator('.cm-just-recipe[aria-label$="in a-recipes"]')
        expect(buttons.first).to_be_visible()
        rects = buttons.evaluate_all("nodes => nodes.map(node => {const r=node.getBoundingClientRect();return {x:r.x,y:r.y}})")
        assert len({round(rect["x"]) for rect in rects}) > 1, rects
        assert len({round(rect["y"]) for rect in rects}) > 1, rects
        page.screenshot(path="/tmp/hickory-files-gutter-grid.png")
        page.keyboard.press("Escape")
        expect(tree.get_by_role("dialog", name="Gutter icons")).to_have_count(0)
        expect(more).to_be_focused()
        more.click()
        # Exactly one recipe request starts, and it runs in its owning directory.
        with page.expect_response(lambda response: response.url.endswith("/api/terminals") and response.request.method == "POST") as launch:
            tree.get_by_role("button", name="Run just recipe3 in a-recipes", exact=True).click()
        session = launch.value.json()
        assert launch.value.ok, session
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            result = request.get(origin + "/api/file?path=a-recipes%2Fresult.txt")
            if result.ok and result.json()["content"] == "recipe-3-ok": break
            page.wait_for_timeout(100)
        else: raise AssertionError("recipe terminal did not produce its file in a-recipes")
        assert request.get(origin + "/api/file?path=b-recipes%2Fresult.txt").status == 404
        # Workspace-root recipes use the same overflow mechanism in the header.
        root = files.locator(".folder-tree__header")
        expect(root.locator(".cm-gutter-more")).to_be_visible()
        root.locator(".cm-gutter-more").click()
        expect(root.get_by_role("button", name="Run just root-one in .", exact=True)).to_be_visible()
        page.mouse.click(1100, 850)
        expect(root.get_by_role("dialog", name="Gutter icons")).to_have_count(0)
        page.screenshot(path="/tmp/hickory-just-gutters.png")
        doc = post("/api/projects/local/docs", {"path": "gutter-actions.md", "source": '<hick:exec container="shell">true</hick:exec>' * 24 + '\n' + "\n".join("Space below the actions." for _ in range(30))})
        page.goto(origin + "/#/docs/" + doc["id"])
        page.get_by_role("tablist", name="Center views").get_by_role("tab", name="gutter-actions.md", exact=True).click()
        rail = page.locator('[data-spatial-center=true] .cm-card-rail')
        cell_more = rail.locator(".cm-card-rail__more").first
        expect(cell_more).to_be_visible()
        tops = rail.locator("[data-card]").evaluate_all("nodes => nodes.map(node => node.style.top)")
        cell_more.click()
        grid = page.get_by_role("dialog", name="Gutter icons")
        expect(grid).to_be_visible()
        assert grid.locator("button").count() >= 2
        assert rail.locator("[data-card]").evaluate_all("nodes => nodes.map(node => node.style.top)") == tops
        page.screenshot(path="/tmp/hickory-document-gutter-grid.png")
        page.keyboard.press("Escape")
        expect(grid).to_have_count(0)
        expect(cell_more).to_be_focused()
        browser.close()
        print("Recipe discovery, scoped launch, root controls, overflow grid and unchanged row geometry passed.")


if __name__ == "__main__":
    main(sys.argv[1].rstrip("/"))
</storycode:copy>

<storycode:file path="scripts/test-just-gutters.py"><storycode:paste select="#just-gutter-scripts-test-just-gutters-py"/></storycode:file>


## Keep document action callbacks behind the shared overflow rule

Keep document action callbacks behind the shared overflow rule

A short native editor viewport leaves no room for a three-action stack. The component must offer a three-dot trigger, preserve all three verbs in its floating grid, restore focus on Escape, and invoke the existing source-toggle callback from that grid.

<storycode:copy id="just-gutter-apps-web-src-editor-CardRail-test-tsx">
// Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { CardRail } from "./CardRail";
import { editorChrome } from "./chrome";
import { EnvRegistry, wysiwyg } from "./wysiwyg";
import { DebugLaunchRail } from "../debug/DebugRail";
import type { DocCard } from "./cards";
const views: EditorView[] = [];
afterEach(() => { cleanup(); vi.restoreAllMocks(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } });
it("opens excess document actions in a grid and invokes their existing callbacks", async () => {
  const view = new EditorView({ parent: document.body.appendChild(document.createElement("div")), state: EditorState.create({ doc: "task" }) }); views.push(view);
  Object.defineProperty(view.scrollDOM, "clientHeight", { value: 22 });
  const card: DocCard = { key: "exec-0", kind: "exec", index: 0, at: 0, from: 0, to: 4, label: "Task" };
  const action = vi.fn();
  render(<CardRail view={view} cards={[card]} stateOf={() => "idle"} openKey={null} renderedAt={[]} actionsOf={() => ["run", "source", "replay"]} replayingAt={[]} onAction={action} />);
  const more = await screen.findByRole("button", { name: "Show 3 more gutter icons" });
  fireEvent.click(more);
  const dialog = screen.getByRole("dialog", { name: "Gutter icons" });
  expect(dialog.querySelectorAll("button")).toHaveLength(3);
  fireEvent.keyDown(document, { key: "Escape" });
  expect(screen.queryByRole("dialog", { name: "Gutter icons" })).toBeNull(); expect(document.activeElement).toBe(more);
  fireEvent.click(more);
  const source = screen.getByRole("button", { name: /showing its source; click to render it/ });
  await act(async () => fireEvent.click(source));
  await waitFor(() => expect(action).toHaveBeenCalledWith(card, "source", Number.parseFloat(more.style.top)));
  expect(screen.queryByRole("dialog", { name: "Gutter icons" })).toBeNull();
});

it("keeps debugger actions owned by their original groups inside the same overflow grid", async () => {
  const wrapper = document.body.appendChild(document.createElement("div"));
  const host = wrapper.appendChild(document.createElement("div")); host.className = "editor-cm-host";
  const view = new EditorView({ parent: host, state: EditorState.create({
    doc: '<hick:file path="a.py" language="python">print(1)</hick:file><hick:file path="b.py" language="python">print(2)</hick:file>',
    extensions: [editorChrome("code"), wysiwyg(new EnvRegistry())],
  }) }); views.push(view);
  const rect = (left: number, width: number) => ({ left, right: left + width, top: 0, bottom: 22, x: left, y: 0, width, height: 22, toJSON: () => ({}) });
  vi.spyOn(view.scrollDOM, "getBoundingClientRect").mockReturnValue(rect(0, 100));
  Object.defineProperty(view.scrollDOM, "clientHeight", { value: 22 });
  const start = vi.fn();
  render(<DebugLaunchRail view={view} onStart={start} onProfiles={vi.fn()} />, { container: wrapper.appendChild(document.createElement("div")) });
  const rail = wrapper.querySelector<HTMLElement>(".cm-debug-rail")!;
  vi.spyOn(rail, "getBoundingClientRect").mockReturnValue(rect(100, 24));
  Object.defineProperty(rail, "clientHeight", { value: 22 });
  for (const group of rail.querySelectorAll<HTMLElement>(".cm-debug-rail__group")) vi.spyOn(group, "getBoundingClientRect").mockReturnValue(rect(100, 24));
  const more = await screen.findByRole("button", { name: "Show 4 more gutter icons" });
  fireEvent.click(more);
  expect(screen.getByRole("dialog", { name: "Gutter icons" })).toBeTruthy();
  const button = screen.getByRole("button", { name: "Debug a.py" });
  expect(button.closest(".cm-debug-rail__group")).not.toBeNull();
  await act(async () => fireEvent.click(button));
  expect(start).toHaveBeenCalledWith("a.py", 0);
  fireEvent.keyDown(document, { key: "Escape" });
  expect(screen.queryByRole("dialog", { name: "Gutter icons" })).toBeNull();
});
</storycode:copy>

<storycode:file path="apps/web/src/editor/CardRail.test.tsx"><storycode:paste select="#just-gutter-apps-web-src-editor-CardRail-test-tsx"/></storycode:file>

</storycode:doc>
