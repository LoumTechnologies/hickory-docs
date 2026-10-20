import { acceptCompletion, closeCompletion, completionStatus, currentCompletions, selectedCompletionIndex, setSelectedCompletion, startCompletion, type Completion } from "@codemirror/autocomplete";
import { EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import { prepareEditorChanges } from "../editor/textInput";
import type { UiValue } from "../lib/uiActions";

const infoText = new WeakMap<Completion, string>();
export function completionReading<T extends Completion>(completion: T, text: string): T { infoText.set(completion, text); return completion; }

/** Use the native popup: no second completion query or fabricated apply range. */
export function completionActions(): Extension {
  let retained: { signature: string; id: string; bytes: Uint8Array; count: number } | undefined;
  let job: string | null = null;
  const identities = new WeakMap<object, string>();
  const identity = (object: object) => { let id = identities.get(object); if (!id) { id = crypto.randomUUID(); identities.set(object, id); } return id; };
  const capture = (view: EditorView) => {
    const status = completionStatus(view.state), options = currentCompletions(view.state);
    const value = { source_identity: identity(view.state.doc), selection: view.state.selection.toJSON(), status, selected: selectedCompletionIndex(view.state), choices: options.map((item, index) => ({ index, id: identity(item), label: item.label, insert_text: typeof item.apply === "function" ? null : item.apply ?? item.label, custom_apply: typeof item.apply === "function", display_label: item.displayLabel, detail: item.detail, type: item.type, section: typeof item.section === "string" ? item.section : item.section?.name, info: infoText.get(item) ?? (typeof item.info === "string" ? item.info : null) })) };
    const signature = JSON.stringify(value);
    if (!retained || retained.signature !== signature) retained = { signature, id: crypto.randomUUID(), bytes: new TextEncoder().encode(signature), count: options.length };
    return { ...retained, status, selected: value.selected };
  };
  const state = (view: EditorView): UiValue => { const current = capture(view); return { job, status: current.status ?? "closed", snapshot: current.id, count: current.count, selected: current.selected, total_bytes: current.bytes.length }; };
  const writable = (view: EditorView) => !view.state.readOnly && view.state.facet(EditorView.editable);
  const number = { type: "number" as const, minimum: 0, maximum: 1000000000, integer: true };
  const snapshot = { type: "string" as const, maxLength: 128 };
  let previousStatus: ReturnType<typeof completionStatus> = null;
  return [
    EditorView.updateListener.of(update => {
      const next = completionStatus(update.state);
      if (next !== null && previousStatus === null) job = crypto.randomUUID();
      previousStatus = next;
    }),
    // Provider text goes through the same whole-batch input guard for humans
    // and agents. Native completion filters and save/provenance listeners remain.
    EditorState.transactionFilter.of(transaction => {
      if (!transaction.docChanged || !transaction.isUserEvent("input.complete")) return transaction;
      const changes: { from: number; to: number; insert: string }[] = [];
      transaction.changes.iterChanges((from, to, _newFrom, _newTo, insert) => changes.push({ from, to, insert: insert.toString() }));
      return "reason" in prepareEditorChanges(transaction.startState.doc.toString(), changes) || transaction.startState.readOnly || !transaction.startState.facet(EditorView.editable) ? [] : transaction;
    }),
    editorLanguageActions.of({ id: "completion:open", label: "Open editor completions", parameters: {}, value: state, enabled: writable,
      execute: async (view, _args, signal) => {
        if (signal.aborted) return { status: "unavailable", reason: "Completion request cancelled" };
        const requested = startCompletion(view);
        if (!requested) return { status: "unavailable", reason: "The editor could not open completions at this position" };
        job ??= crypto.randomUUID();
        return { status: "queued", job, value: { ...state(view) as Record<string, UiValue>, scope: "Native completion query requested; observe popup status and choices before accepting" } };
      },
    }),
    editorLanguageActions.of({ id: "completion:state", label: "Observe editor completion popup", parameters: {}, value: state, revision: view => capture(view).id, enabled: () => true,
      execute: async view => ({ status: "completed", value: state(view) }),
    }),
    editorLanguageActions.of({ id: "completion:read", label: "Read editor completion choices", parameters: { snapshot, offset: number, limit: { ...number, minimum: 1, maximum: 8192 } }, value: state, revision: view => capture(view).id,
      enabled: view => completionStatus(view.state) === "active",
      execute: async (view, args) => {
        const current = capture(view);
        if (current.id !== args.snapshot) return { status: "unavailable", reason: "Completion choices changed; observe and read the current snapshot" };
        const offset = args.offset as number, end = Math.min(offset + (args.limit as number), current.bytes.length);
        if (offset > current.bytes.length) return { status: "unavailable", reason: "Offset outside current completion choices" };
        return { status: "completed", value: { snapshot: current.id, bytes: Array.from(current.bytes.slice(offset, end)), offset, total_bytes: current.bytes.length, next_offset: end < current.bytes.length ? end : null, encoding: "JSON UTF-8" } };
      },
    }),
    editorLanguageActions.of({ id: "completion:select", label: "Select editor completion", parameters: { snapshot, index: number }, value: state, revision: view => capture(view).id,
      enabled: view => completionStatus(view.state) === "active",
      execute: async (view, args) => {
        const current = capture(view), index = args.index as number;
        if (current.id !== args.snapshot || index >= current.count) return { status: "unavailable", reason: "Completion choice is outside the current snapshot" };
        view.dispatch({ effects: setSelectedCompletion(index) });
        return { status: "completed", value: state(view) };
      },
    }),
    editorLanguageActions.of({ id: "completion:accept", label: "Accept selected editor completion", parameters: { snapshot }, value: state, revision: view => capture(view).id,
      enabled: view => writable(view) && completionStatus(view.state) === "active" && selectedCompletionIndex(view.state) !== null,
      execute: async (view, args, signal) => {
        if (signal.aborted || capture(view).id !== args.snapshot) return { status: "unavailable", reason: "Completion selection changed or request cancelled" };
        const original = view.state.doc, accepted = acceptCompletion(view);
        return { status: "completed", value: { accepted, source_changed: view.state.doc !== original, saved: false, verified: false, popup: state(view), scope: "Actual native completion acceptance; source filters may refuse changes" } };
      },
    }),
    editorLanguageActions.of({ id: "completion:close", label: "Close editor completions", parameters: {}, value: state, enabled: view => completionStatus(view.state) !== null,
      execute: async view => ({ status: "completed", value: { closed: closeCompletion(view), job, scope: "Closes the native popup and cancels its outstanding query; no source rollback" } }),
    }),
  ];
}
