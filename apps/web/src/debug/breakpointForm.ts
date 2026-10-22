import { StateEffect } from "@codemirror/state";
import { EditorView, ViewPlugin } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import type { BreakpointMark, DebugEditorOptions } from "./cmDebug";
type Patch = { condition?: string; hit_condition?: string; log_message?: string };
type Form = { panel: HTMLDivElement; line: number; current: () => boolean; state: () => unknown; set: (key: keyof Patch, value: string) => UiOutcome; apply: () => UiOutcome; close: (notify?: boolean) => UiOutcome; focus: () => void };
const forms = new WeakMap<EditorView, Form>();
const changed = StateEffect.define<null>();

/** Native Enter/focus-out and agent submission use the same captured form. */
export function showBreakpointForm(view: EditorView, mark: BreakpointMark, onSet: (line: number, patch: Patch) => void, lookup: () => BreakpointMark | undefined) {
  const previous = forms.get(view);
  if (previous?.current() && previous.line === mark.line) { previous.focus(); return; }
  previous?.close(false);
  const source = view.state.doc;
  const fingerprint = () => { const mark = lookup(); return JSON.stringify([mark?.condition ?? "", mark?.hitCondition ?? "", mark?.logMessage ?? ""]); };
  const captured = fingerprint();
  let installed = !!lookup();
  const panel = document.createElement("div"); panel.className = "cm-bp-editor"; panel.setAttribute("role", "dialog"); panel.setAttribute("aria-label", `Breakpoint on line ${mark.line + 1}`);
  const title = document.createElement("div"); title.className = "cm-bp-editor-title"; title.textContent = `Line ${mark.line + 1}`; panel.append(title);
  const fields = new Map<keyof Patch, HTMLInputElement>();
  let error: string | null = null;
  const errorNode = document.createElement("span"); errorNode.setAttribute("role", "alert");
  const notify = () => { if (view.dom.isConnected) view.dispatch({ effects: changed.of(null) }); };
  const current = () => {
    const mark = lookup();
    if (installed && !mark) return false;
    if (mark) installed = true;
    return view.dom.isConnected && panel.isConnected && forms.get(view) === form && view.state.doc === source && fingerprint() === captured;
  };
  const unavailable = (): UiOutcome => ({ status: "unavailable", reason: "Breakpoint editor source/target changed or closed; reopen the current gutter target" });
  const close = (signal = true): UiOutcome => { if (forms.get(view) === form) forms.delete(view); panel.remove(); if (signal) notify(); return { status: "completed", value: { closed: true, applied: false } }; };
  const apply = (): UiOutcome => {
    if (!current()) { close(); return unavailable(); }
    const patch: Patch = {}; for (const [key, input] of fields) patch[key] = input.value.trim();
    try { onSet(mark.line, patch); close(); return { status: "completed", value: { callback_requested: true, line: mark.line + 1, adapter_acceptance_unknown: true } }; }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); errorNode.textContent = error; notify(); return { status: "unavailable", reason: error }; }
  };
  const form: Form = { panel, line: mark.line, current, state: () => ({ line: mark.line + 1, fields: Object.fromEntries([...fields].map(([key, input]) => [key, input.value])), error }),
    set: (key, value) => {
      if (!current()) return unavailable();
      const input = fields.get(key);
      if (!input || /[\r\n]/.test(value)) return { status: "unavailable", reason: "Choose an existing single-line breakpoint field" };
      input.value = value; error = null; errorNode.textContent = ""; notify(); return { status: "completed", value: { draft_changed: true, applied: false } };
    }, apply, close, focus: () => { fields.get("condition")?.focus(); fields.get("condition")?.select(); } };
  forms.set(view, form);
  for (const [key, label, value, placeholder] of [["condition", "Condition", mark.condition, "i > 10"], ["hit_condition", "Hit count", mark.hitCondition, "> 5"], ["log_message", "Log and continue", mark.logMessage, "reached with i={i}"]] as const) {
    const row = document.createElement("label"); row.className = "cm-bp-editor-row";
    const name = document.createElement("span"); name.textContent = label;
    const input = document.createElement("input"); input.type = "text"; input.value = value ?? ""; input.placeholder = placeholder; fields.set(key, input);
    input.addEventListener("input", () => { if (current()) { error = null; errorNode.textContent = ""; notify(); } });
    row.append(name, input); panel.append(row);
  }
  const hint = document.createElement("p"); hint.className = "cm-bp-editor-hint"; hint.textContent = "Empty means no condition. A log message makes this a breakpoint that does not stop."; panel.append(hint, errorNode);
  panel.addEventListener("keydown", event => { event.stopPropagation(); if (event.key === "Escape") { event.preventDefault(); close(); } else if (event.key === "Enter") { event.preventDefault(); apply(); } });
  panel.addEventListener("focusout", () => window.setTimeout(() => { if (current() && !panel.contains(document.activeElement)) apply(); }, 0));
  const block = view.lineBlockAt(view.state.doc.line(mark.line + 1).from); panel.style.top = `${block.top}px`; view.dom.append(panel); form.focus(); notify();
}

export function breakpointFormActions(options: DebugEditorOptions, open: (view: EditorView, line: number) => boolean) {
  const id = crypto.randomUUID(), current = (view: EditorView) => { const form = forms.get(view); return form?.current() ? form : undefined; };
  const visible = (view: EditorView) => !!current(view);
  const unavailable: UiOutcome = { status: "unavailable", reason: "Breakpoint form closed or source/target changed" };
  return [ViewPlugin.fromClass(class {
    constructor(readonly view: EditorView) {}
    update() { const form = forms.get(this.view); if (form && !form.current()) form.close(false); }
    destroy() { forms.get(this.view)?.close(false); }
  }), editorLanguageActions.of({ id: `${id}:open`, label: "Open gutter breakpoint condition editor", visible: () => !!options.onSetBreakpointCondition, enabled: () => true, parameters: { line: { type: "number", integer: true, minimum: 1, maximum: 100000000 } },
    execute: async (view, args, _signal, actor) => {
      const focused = document.activeElement, existing = forms.get(view);
      if (actor?.kind === "agent" && focused instanceof HTMLElement && focused.closest("input,textarea") && !existing?.panel.contains(focused)) return { status: "requires_input", request: "breakpoint-finish-field", reason: "Finish or cancel the focused field before opening a breakpoint editor" };
      return open(view, Number(args.line) - 1) ? { status: "completed", value: { form_open_requested: true } } : { status: "unavailable", reason: "This line has no current mapped breakpoint target" };
    } }), editorLanguageActions.of({ id: `${id}:set`, label: "Set gutter breakpoint editor field", visible, enabled: () => true, parameters: { field: { type: "string", maxLength: 32, choices: ["condition", "hit_condition", "log_message"] }, text: { type: "string", maxLength: 8192 } }, revision: view => current(view)?.state(),
      execute: async (view, args) => current(view)?.set(args.field as keyof Patch, String(args.text)) ?? unavailable }),
    ...[true, false].map(apply => editorLanguageActions.of({ id: `${id}:${apply ? "apply" : "close"}`, label: apply ? "Apply gutter breakpoint editor" : "Close gutter breakpoint editor", visible, enabled: () => true, parameters: {}, revision: view => current(view)?.state(), execute: async view => (apply ? current(view)?.apply() : current(view)?.close()) ?? unavailable })),
    editorLanguageActions.of({ id: `${id}:read`, label: "Read gutter breakpoint editor bytes", visible, enabled: () => true, parameters: { offset: { type: "number", integer: true, minimum: 0, maximum: 100000000 }, limit: { type: "number", integer: true, minimum: 1, maximum: 8192 } }, revision: view => current(view)?.state(),
      execute: async (view, args) => {
        const form = current(view); if (!form) return unavailable;
        const bytes = new TextEncoder().encode(JSON.stringify(form.state())), offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current breakpoint form" };
        return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "JSON UTF-8", span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length } };
      } }),
  ];
}
