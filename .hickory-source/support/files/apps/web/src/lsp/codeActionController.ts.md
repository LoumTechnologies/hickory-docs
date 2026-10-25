<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lsp/codeActionController.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f4333bfafe29212f9f310fbf8fd274b73ceb0d55bf45292ec10604e9ba1f236a`.

<book:entry path="apps/web/src/lsp/codeActionController.ts">
import type { EditorSelection, Extension, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import type { CodeAction, WorkspaceEdit } from "./client";
import type { LspFeatureOptions } from "./cmLspFeatures";

function freeze<T>(value: T, seen = new WeakSet<object>()): T {
  if (value && typeof value === "object" && !seen.has(value)) {
    seen.add(value);
    for (const child of Object.values(value)) freeze(child, seen);
    Object.freeze(value);
  }
  return value;
}

type Capture = { id: string; source: Text; selection: EditorSelection; choices: { id: string; action: CodeAction }[]; bytes: Uint8Array };
/** Retained provider choices; model titles or IDs cannot fabricate a workspace edit. */
export function codeActionController(opts: LspFeatureOptions, apply: (view: EditorView, edit: WorkspaceEdit | null, opts: LspFeatureOptions) => boolean): { extensions: Extension[]; human: (view: EditorView) => boolean } {
  let captured: Capture | undefined;
  const writable = (view: EditorView) => !view.state.readOnly && view.state.facet(EditorView.editable);
  const matches = (view: EditorView, capture: Capture) => writable(view) && view.state.doc === capture.source && view.state.selection.eq(capture.selection);
  const refuse = (reason: string): UiOutcome => { opts.onMessage?.(reason); return { status: "unavailable", reason }; };
  const collect = async (view: EditorView, signal?: AbortSignal): Promise<Capture | UiOutcome> => {
    const { client, uri } = opts, source = view.state.doc, selection = view.state.selection;
    const start = opts.positionAt(selection.main.from, view), end = opts.positionAt(selection.main.to, view) ?? start;
    if (!client || !writable(view) || !start || !end) return refuse("Code actions need a writable mapped source selection.");
    try {
      const actions = await client.codeActions(uri, { start, end });
      if (signal?.aborted || view.state.doc !== source || !view.state.selection.eq(selection) || !writable(view)) return refuse("The source or selection changed. Request code actions again.");
      const choices = structuredClone(actions).map(action => ({ id: crypto.randomUUID(), action: freeze(action) }));
      const id = crypto.randomUUID();
      const bytes = new TextEncoder().encode(JSON.stringify({ snapshot: id, choices: choices.map(({ id, action }) => ({ id, title: action.title, kind: action.kind, has_edit: !!action.edit, needs_command: !!action.command, command_supported: false })) }));
      captured = { id, source, selection, choices, bytes };
      return captured;
    } catch (e) { return refuse(e instanceof Error ? e.message : String(e)); }
  };
  const choose = (view: EditorView, capture: Capture, id: string, signal?: AbortSignal): UiOutcome => {
    if (captured !== capture || !matches(view, capture) || signal?.aborted) return refuse("Code action choices changed. Request them again.");
    const chosen = capture.choices.find(choice => choice.id === id)?.action;
    if (!chosen) return refuse("Choose an action from this retained provider snapshot.");
    // The human editor also cannot execute arbitrary provider commands. Do not
    // silently apply only the edit portion of an edit+command action.
    if (chosen.command) return refuse(`"${chosen.title}" needs a language-server command this editor cannot execute yet.`);
    if (chosen.edit?.documentChanges?.some(entry => !entry.textDocument?.uri)) return refuse("This code action needs resource operations this editor cannot apply yet.");
    captured = undefined;
    const changed = chosen.edit ? apply(view, chosen.edit, opts) : false;
    const other = Object.keys(chosen.edit?.changes ?? {});
    for (const entry of chosen.edit?.documentChanges ?? []) if (entry.textDocument?.uri) other.push(entry.textDocument.uri);
    return { status: "completed", value: { title: chosen.title, source_changed: changed, other_documents: [...new Set(other)].filter(uri => uri !== opts.uri), saved: false, verified: false, scope: "Actual current-editor edit result; other document edits are reported, not applied" } };
  };
  const common = { enabled: (view: EditorView) => !!opts.client && !!opts.onCodeActions && writable(view) && !!opts.positionAt(view.state.selection.main.from, view) };
  const extensions = !opts.client || !opts.onCodeActions ? [] : [
    editorLanguageActions.of({ ...common, id: `${opts.uri}:actions`, label: `Inspect editor code actions (${opts.uri.slice(0, 200)})`, parameters: {}, execute: async (view, _args, signal) => {
      const reply = await collect(view, signal);
      return "status" in reply ? reply : { status: "completed", value: { snapshot: reply.id, count: reply.choices.length, total_bytes: reply.bytes.length } };
    } }),
    editorLanguageActions.of({ id: `${opts.uri}:actions-read`, label: `Read editor code action choices (${opts.uri.slice(0, 200)})`, revision: () => captured?.id,
      parameters: { snapshot: { type: "string", maxLength: 128 }, offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } },
      enabled: view => !!captured && matches(view, captured), execute: async (view, args) => {
        const capture = captured;
        if (!capture || capture.id !== args.snapshot || !matches(view, capture)) return refuse("Code action snapshot changed. Inspect current choices again.");
        const offset = args.offset as number, end = Math.min(offset + (args.limit as number), capture.bytes.length);
        if (offset > capture.bytes.length) return refuse("Offset outside captured code action choices.");
        return { status: "completed", value: { snapshot: capture.id, bytes: Array.from(capture.bytes.slice(offset, end)), offset, encoding: "JSON UTF-8", total_bytes: capture.bytes.length, next_offset: end < capture.bytes.length ? end : null } };
      },
    }),
    editorLanguageActions.of({ id: `${opts.uri}:actions-choose`, label: `Apply captured editor code action (${opts.uri.slice(0, 200)})`, revision: () => captured?.id,
      parameters: { snapshot: { type: "string", maxLength: 128 }, choice: { type: "string", maxLength: 128 } }, enabled: view => !!captured && matches(view, captured),
      execute: async (view, args, signal) => !captured || captured.id !== args.snapshot ? refuse("Code action snapshot changed. Inspect current choices again.") : choose(view, captured, args.choice as string, signal),
    }),
  ];
  return { extensions, human: view => {
    if (!common.enabled(view)) return false;
    void (async () => {
      const capture = await collect(view);
      if ("status" in capture) return;
      if (!capture.choices.length) { opts.onMessage?.("No code actions here."); return; }
      const actions = capture.choices.map(choice => choice.action);
      const selected = await opts.onCodeActions!(actions);
      if (!selected) return;
      const choice = capture.choices.find(choice => choice.action === selected);
      if (choice) choose(view, capture, choice.id);
      else refuse("The selected action is not from the retained provider choices.");
    })().catch(e => opts.onMessage?.(e instanceof Error ? e.message : String(e)));
    return true;
  } };
}
</book:entry>

</book:doc>
