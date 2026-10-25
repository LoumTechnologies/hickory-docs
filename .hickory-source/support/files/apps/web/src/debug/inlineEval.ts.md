<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/inlineEval.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b19ab81da9696ec71a269a2a59b0590397ac37bfbc8bb8612e6c597a6c21a1e2`.

<book:entry path="apps/web/src/debug/inlineEval.ts">
import { StateEffect, StateField, type Text, type StateEffectType } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import type { DebugSession } from "./useDebugger";
import type { DebugEditorOptions } from "./cmDebug";

const frameOwners = new WeakMap<object, string>();
/** Values/watch replies do not change this owner; a new stop/frame/session does. */
export function inlineEvaluationOwner(debug: DebugSession): string | null {
  if (debug.status !== "paused") return null;
  let frames = frameOwners.get(debug.frames);
  if (!frames) { frames = crypto.randomUUID(); frameOwners.set(debug.frames, frames); }
  return JSON.stringify([debug.status, debug.program, debug.reading?.scope, debug.reading?.source.revision, debug.selectedFrame, frames]);
}

/** The actual paused-line form, shared by native presses and language actions. */
export function inlineEval(options: DebugEditorOptions, paused: StateField<number | null>, stopped: StateEffectType<number | null>) {
  const changed = StateEffect.define<null>();
  const widgets = new WeakMap<EditorView, EvalWidget>();
  const tick = StateField.define({ create: () => 0, update: (value, tr) => tr.effects.some(effect => effect.is(stopped)) ? value + 1 : value });
  class EvalWidget extends WidgetType {
    private view: EditorView | null = null;
    private root: HTMLSpanElement | null = null;
    private input: HTMLInputElement | null = null;
    private answer: HTMLSpanElement | null = null;
    private open = false;
    private draft = "";
    private result = "";
    private requested: string | null = null;
    private error: string | null = null;
    private pending = false;
    private generation = 0;
    constructor(readonly line: number, readonly source: Text, readonly owner: unknown) { super(); }
    eq(other: EvalWidget) { return other.line === this.line && other.source === this.source && other.owner === this.owner; }
    current() { return !!this.view?.dom.isConnected && widgets.get(this.view) === this && this.owner != null && this.view.state.doc === this.source && this.view.state.field(paused, false) === this.line && (options.evaluationOwner ? options.evaluationOwner() : options) === this.owner; }
    shown() { return this.current() && this.open; }
    state() { return { open: this.open, pending: this.pending, draft: this.draft, requested_expression: this.requested, result: this.result, error: this.error, line: this.line + 1, evaluation_is_not_verification: true }; }
    signal() { if (this.current()) this.view!.dispatch({ effects: changed.of(null) }); }
    paint() {
      if (!this.root) return;
      this.root.textContent = ""; this.input = null; this.answer = null;
      if (!this.open) {
        const toggle = document.createElement("button"); toggle.type = "button"; toggle.className = "cm-eval-toggle"; toggle.textContent = "eval…";
        toggle.dataset.tip = "Evaluate an expression in the paused frame";
        toggle.addEventListener("mousedown", event => { event.preventDefault(); event.stopPropagation(); if (toggle.isConnected && this.root?.contains(toggle)) this.disclose(true); });
        this.root.append(toggle); return;
      }
      const input = document.createElement("input"); input.className = "cm-eval-input"; input.value = this.draft;
      input.placeholder = "expression — Enter evaluates, Shift-Enter watches";
      input.setAttribute("aria-label", "Evaluate an expression in the paused frame");
      const answer = document.createElement("span"); answer.className = "cm-eval-result"; answer.textContent = this.error ?? this.result;
      input.addEventListener("input", () => { if (this.shown() && this.input === input) this.setDraft(input.value); });
      input.addEventListener("keydown", event => {
        event.stopPropagation();
        if (!this.shown() || this.input !== input) return;
        if (event.key === "Escape") { event.preventDefault(); this.disclose(false); }
        else if (event.key === "Enter") { event.preventDefault(); void this.submit(event.shiftKey); }
      });
      this.input = input; this.answer = answer; this.root.append(input, answer);
    }
    disclose(open: boolean, agent = false): UiOutcome {
      if (!this.current()) return { status: "unavailable", reason: "Inline evaluation source/stop/frame retired" };
      const focused = document.activeElement;
      if (agent && open && focused instanceof HTMLElement && !this.root?.contains(focused) && focused.closest("input,textarea")) return { status: "requires_input", request: "inline-eval-finish-field", reason: "Finish or cancel the focused field before opening inline evaluation" };
      if (this.open !== open) { this.open = open; this.generation++; this.draft = ""; this.requested = null; this.result = ""; this.error = null; this.paint(); this.signal(); }
      if (open) this.input?.focus();
      return { status: "completed", value: { open, evaluation_cancelled: false, possible_pending_request: this.pending } };
    }
    setDraft(text: string): UiOutcome {
      if (!this.shown()) return { status: "unavailable", reason: "Expand the current inline evaluation form before editing" };
      if (/[\r\n]/.test(text)) return { status: "unavailable", reason: "Inline evaluation uses a single-line expression field" };
      this.draft = text; if (this.input) this.input.value = text; this.signal();
      return { status: "completed", value: { draft_changed: true, evaluated: false } };
    }
    async submit(watch: boolean, signal?: AbortSignal): Promise<UiOutcome> {
      if (signal?.aborted || !this.shown() || this.pending || !this.draft.trim()) return { status: "unavailable", reason: "A nonempty expression and idle current inline form are required" };
      const expression = this.draft.trim(), generation = this.generation;
      this.requested = expression;
      if (watch) {
        if (!options.onAddWatch) return { status: "unavailable", reason: "This host has no watch callback" };
        try {
          options.onAddWatch(expression); this.result = "→ watching"; this.error = null;
          if (this.answer) this.answer.textContent = this.result; this.signal();
          return { status: "completed", value: { watch_requested: true } };
        } catch (cause) {
          this.error = cause instanceof Error ? cause.message : String(cause);
          if (this.answer) this.answer.textContent = this.error; this.signal();
          return { status: "unavailable", reason: this.error };
        }
      }
      if (!options.onEvaluate) return { status: "unavailable", reason: "This host has no evaluation callback" };
      this.pending = true; this.result = "…"; this.error = null;
      if (this.answer) this.answer.textContent = this.result; this.signal();
      try {
        const value = await options.onEvaluate(expression);
        if (signal?.aborted) {
          if (this.shown() && generation === this.generation) { this.error = "Evaluation cancelled after dispatch; the request may have executed, but its value was not adopted"; if (this.answer) this.answer.textContent = this.error; }
          return { status: "unavailable", reason: "Evaluation cancelled after dispatch; inspect its actual debugger session before retrying" };
        }
        if (!this.shown() || generation !== this.generation) return { status: "unavailable", reason: "Inline evaluation closed or source/stop/frame changed after dispatch; the request may have executed, but its value was not adopted" };
        this.result = value === null ? "(no value)" : `= ${value}`;
        if (this.answer) this.answer.textContent = this.result;
        return { status: "completed", value: { evaluation_callback_completed: true, result_available: true, verified: false } };
      } catch (cause) {
        if (!this.shown() || generation !== this.generation) return { status: "unavailable", reason: "Retired inline evaluation failed after dispatch" };
        this.error = cause instanceof Error ? cause.message : String(cause); if (this.answer) this.answer.textContent = this.error;
        return { status: "unavailable", reason: this.error };
      } finally { this.pending = false; this.signal(); }
    }
    toDOM(view: EditorView) {
      this.view = view; widgets.set(view, this); this.root = document.createElement("span"); this.root.className = "cm-eval"; this.paint();
      queueMicrotask(() => this.signal()); return this.root;
    }
    destroy() { if (this.view && widgets.get(this.view) === this) widgets.delete(this.view); this.generation++; this.view = null; this.root = null; this.input = null; this.answer = null; }
    ignoreEvent() { return true; }
  }
  const current = (view: EditorView) => { const widget = widgets.get(view); return widget?.current() ? widget : undefined; };
  const id = crypto.randomUUID(), visible = (view: EditorView) => !!current(view), expanded = (view: EditorView) => !!current(view)?.shown();
  const unavailable: UiOutcome = { status: "unavailable", reason: "Inline evaluation is closed or its source/stop/frame retired" };
  return [tick, EditorView.decorations.compute(["doc", paused, tick], state => {
    if (!options.onEvaluate) return Decoration.none;
    const line = state.field(paused);
    if (line === null || line < 0 || line >= state.doc.lines) return Decoration.none;
    const owner = options.evaluationOwner ? options.evaluationOwner() : options;
    if (owner == null) return Decoration.none;
    return Decoration.set([Decoration.widget({ widget: new EvalWidget(line, state.doc, owner), side: 3 }).range(state.doc.line(line + 1).to)]);
  }), editorLanguageActions.of({ id: `${id}:disclosure`, subject: `${id}:disclosure`, label: "Set paused-line evaluation disclosure", visible, enabled: () => true, parameters: { open: { type: "boolean" } },
    revision: view => current(view)?.state(), execute: async (view, args, _signal, actor) => current(view)?.disclose(Boolean(args.open), actor?.kind === "agent") ?? unavailable }),
    editorLanguageActions.of({ id: `${id}:draft`, subject: `${id}:draft`, label: "Set paused-line evaluation draft", visible: expanded, enabled: () => true, parameters: { text: { type: "string", maxLength: 8192 } },
      revision: view => current(view)?.state(), execute: async (view, args) => current(view)?.setDraft(String(args.text)) ?? unavailable }),
    ...[false, true].map(watch => editorLanguageActions.of({ id: `${id}:${watch ? "watch" : "evaluate"}`, label: watch ? "Watch paused-line evaluation draft" : "Evaluate paused-line draft", visible: expanded, enabled: view => { const state = current(view)?.state(); return !!state && !state.pending && !!state.draft.trim() && (!watch || !!options.onAddWatch); }, parameters: {}, revision: view => current(view)?.state(),
      execute: async (view, _args, signal) => current(view)?.submit(watch, signal) ?? unavailable })),
    editorLanguageActions.of({ id: `${id}:read`, subject: `${id}:reading`, label: "Read paused-line evaluation form bytes", visible: expanded, enabled: () => true,
      parameters: { offset: { type: "number", integer: true, minimum: 0, maximum: 100000000 }, limit: { type: "number", integer: true, minimum: 1, maximum: 8192 } }, revision: view => current(view)?.state(),
      execute: async (view, args) => {
        const widget = current(view); if (!widget?.shown()) return unavailable;
        const bytes = new TextEncoder().encode(JSON.stringify(widget.state())), offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds the current inline form" };
        return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "JSON UTF-8", span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length } };
      } }),
  ];
}
</book:entry>

</book:doc>
