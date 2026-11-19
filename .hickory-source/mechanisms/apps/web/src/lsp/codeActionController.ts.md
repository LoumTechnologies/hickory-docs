<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/codeActionController.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-67a81de633f2"></a>
## `apps/web/src/lsp/codeActionController.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-67a81de633f2-freeze"></a>
### `freeze`

<storycode:copy id="f-67a81de633f2-freeze">
import {resolveCodeAction,actionNeedsResolution,queryCodeActions,type SourceActionKind} from "./codeActionResolution";
import { sameDocumentUri } from "./documentUri";
import type { EditorSelection, Extension, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import type { CodeAction, WorkspaceEdit } from "./client";
import { utf16ToPosition, type LspRange } from "./positions";
import { authoredSource } from "../simple/SourceText";
import type { LspFeatureOptions } from "./cmLspFeatures";

function freeze<T>(value: T, seen = new WeakSet<object>()): T {
  if (value && typeof value === "object" && !seen.has(value)) {
    seen.add(value);
    for (const child of Object.values(value)) freeze(child, seen);
    Object.freeze(value);
  }
  return value;
}

</storycode:copy>
<a id="f-67a81de633f2-Capture"></a>
### `Capture`

<storycode:copy id="f-67a81de633f2-Capture">
type Capture = {
  id: string; source: Text; selection: EditorSelection; range: LspRange; scope?: SourceActionKind;
  authored?: string; context: () => boolean; choices: { id: string; action: CodeAction }[]; bytes: Uint8Array;
};
</storycode:copy>
<a id="f-67a81de633f2-codeActionController"></a>
### `codeActionController`

A disabled provider action is readable information, never an edit authority. Retain its reason in the captured choices and refuse it before considering any accompanying edit or command. This also preserves a bridge refusal when one target in a multi-file action cannot be mapped.

Retained provider choices; model titles or IDs cannot fabricate a workspace edit.

In Main, selecting a retained choice delegates to the workspace preview with its original mapped range, editor snapshot and visibility guard. The workspace may acquire closed source targets, so it queries and checks the action against the acquired sources before presenting its edits. The editor controller keeps the guard live until Apply, a newer choice or a new query.

The scoped Commands entries span the authored document, even from an unmapped
prose caret. Retain the source action family inside the same snapshot as the
choice. After acquiring a contributor the workspace repeats the scoped query
and checks the resolved action still belongs to that family.

For editors with the local apply path, resolve selected retained values before admitting their complete edits. Recheck source, selection, composition, attachment, retained snapshot and latest choice after the await. A superseded response returns an unavailable outcome without replacing a newer message. Multi-document edits still require a workspace preview, and an edit-plus-command action never applies its edit portion alone.

<storycode:copy id="f-67a81de633f2-codeActionController">
/** Retained provider choices; model titles or IDs cannot fabricate a workspace edit. */
export function codeActionController(opts: LspFeatureOptions, apply: (view: EditorView, edit: WorkspaceEdit | null, opts: LspFeatureOptions) => boolean): { extensions: Extension[]; human: (view: EditorView) => boolean } {
  let captured: Capture | undefined;let query=0,choiceEpoch=0;
  const writable = (view: EditorView) => view.dom.isConnected && !view.composing && !view.state.readOnly && view.state.facet(EditorView.editable);
  const sameSelection = (view: EditorView, selection: EditorSelection) => opts.onProviderPreview
    ? view.state.selection === selection : view.state.selection.eq(selection);
  const matches = (view: EditorView, capture: Capture) => writable(view) && view.state.doc === capture.source &&
    sameSelection(view, capture.selection) && capture.context() &&
    (capture.authored === undefined || authoredSource(view.state) === capture.authored);
  const refuse = (reason: string): UiOutcome => { opts.onMessage?.(reason); return { status: "unavailable", reason }; };
  const collect = async (view: EditorView, signal?: AbortSignal, scope?: SourceActionKind, paletteCurrent?: () => boolean): Promise<Capture | UiOutcome> => {
    const request=++query;choiceEpoch++;captured=undefined;
    const { client, uri } = opts, source = view.state.doc, selection = view.state.selection;
    if (!client || !writable(view)) return refuse("Code actions need a writable source.");
    const authored = scope ? authoredSource(view.state) : undefined;
    let range: LspRange;
    if (authored !== undefined) {
      range = {start: {line: 0, character: 0}, end: utf16ToPosition(authored, authored.length)};
    } else {
      const start = opts.positionAt(selection.main.from, view), end = opts.positionAt(selection.main.to, view) ?? start;
      if (!start || !end) return refuse("Code actions need a writable mapped source selection.");
      range = {start, end};
    }
    const editorContext = opts.onProviderPreview ? opts.captureChangeContext?.() : undefined;
    const context = () => (!editorContext || editorContext()) && (!paletteCurrent || paletteCurrent());
    try {
      const actions = await queryCodeActions(client, uri, range, scope);
      if(request!==query||signal?.aborted||!view.dom.isConnected)return {status:"unavailable",reason:"Code action query superseded."};
      if (!writable(view) || view.state.doc !== source || !sameSelection(view, selection) || !context() ||
          (authored !== undefined && authoredSource(view.state) !== authored)) {
        return refuse("The source or selection changed. Request code actions again.");
      }
      const choices = structuredClone(actions).map(action => ({ id: crypto.randomUUID(), action: freeze(action) }));
      const id = crypto.randomUUID();
      const bytes = new TextEncoder().encode(JSON.stringify({ snapshot: id, choices: choices.map(({ id, action }) => ({ id, title: action.title, kind: action.kind, disabled_reason: action.disabled?.reason, has_edit: !!action.edit, needs_resolution:actionNeedsResolution(action), needs_command: !!action.command, command_supported: false })) }));
      captured = {id, source, selection, range, scope, authored, context, choices, bytes};
      return captured;
    } catch (e) { if(request!==query||signal?.aborted||!view.dom.isConnected)return {status:"unavailable",reason:"Code action query superseded."};return refuse(e instanceof Error ? e.message : String(e)); }
  };
  const choose = async (view: EditorView, capture: Capture, id: string, signal?: AbortSignal): Promise<UiOutcome> => {
    if (captured !== capture || !matches(view, capture) || signal?.aborted) return refuse("Code action choices changed. Request them again.");
    const selected = capture.choices.find(choice => choice.id === id)?.action;
    if (!selected) return refuse("Choose an action from this retained provider snapshot.");
    if (selected.disabled) return refuse(selected.disabled.reason);
    const choice = ++choiceEpoch;
    if (opts.onProviderPreview) {
      const current = () => choice === choiceEpoch && captured === capture && !signal?.aborted && matches(view, capture);
      if (capture.choices.filter(({action}) => action.title === selected.title && action.kind === selected.kind).length !== 1) {
        return refuse("The provider returned indistinguishable choices. Select a more specific source range and try again.");
      }
      try {
        return await opts.onProviderPreview({kind: "code action", client: opts.client!, uri: opts.uri, view, range: capture.range, scope: capture.scope, action: selected, current});
      } catch (error) {
        return current() ? refuse(error instanceof Error ? error.message : String(error)) :
          {status: "unavailable", reason: "Code action choice superseded."};
      }
    }
    let chosen: CodeAction;
    try {chosen=await resolveCodeAction(opts.client!,selected,opts.uri);}catch(error){if(choice!==choiceEpoch||captured!==capture||signal?.aborted||!view.dom.isConnected)return {status:"unavailable",reason:"Code action choice superseded."};return refuse(error instanceof Error?error.message:String(error));}
    if(choice!==choiceEpoch||captured!==capture||signal?.aborted||!view.dom.isConnected)return {status:"unavailable",reason:"Code action choice superseded."};
    if(!matches(view,capture))return refuse("Code action source or selection changed. Request actions again.");
    if(chosen.disabled)return refuse(chosen.disabled.reason);
    if(!chosen.edit&&!chosen.command)return refuse("This provider action has no source edit to apply. Request actions again.");
    // The human editor also cannot execute arbitrary provider commands. Do not
    // silently apply only the edit portion of an edit+command action.
    if (chosen.command) return refuse(`"${chosen.title}" needs a language-server command this editor cannot execute yet.`);
    if (chosen.edit?.documentChanges?.some(entry => !entry.textDocument?.uri)) return refuse("This code action needs resource operations this editor cannot apply yet.");
    const targets = [...Object.keys(chosen.edit?.changes ?? {}), ...(chosen.edit?.documentChanges ?? []).flatMap(entry => entry.textDocument?.uri ? [entry.textDocument.uri] : [])];
    if (targets.some(target => !sameDocumentUri(target, opts.uri))) return refuse("A workspace preview is required before applying this multi-document code action. No edits were applied.");
    captured = undefined;
    const changed = chosen.edit ? apply(view, chosen.edit, opts) : false;
    const other = Object.keys(chosen.edit?.changes ?? {});
    for (const entry of chosen.edit?.documentChanges ?? []) if (entry.textDocument?.uri) other.push(entry.textDocument.uri);
    return { status: "completed", value: { title: chosen.title, source_changed: changed, other_documents: [...new Set(other)].filter(uri => uri !== opts.uri), saved: false, verified: false, scope: "Actual current-editor edit result; other document edits are reported, not applied" } };
  };
  const enabled = (view: EditorView, scope?: SourceActionKind) =>
    !!opts.client && !!opts.onCodeActions && writable(view) && (!!scope || !!opts.positionAt(view.state.selection.main.from, view));
  const inspect = async (view: EditorView, signal: AbortSignal, scope?: SourceActionKind): Promise<UiOutcome> => {
    const reply = await collect(view, signal, scope);
    return "status" in reply ? reply : {status: "completed", value: {snapshot: reply.id, count: reply.choices.length, total_bytes: reply.bytes.length}};
  };
  const human = (view: EditorView, scope?: SourceActionKind, paletteCurrent?: () => boolean) => {
    if (!enabled(view, scope)) return false;
    void (async () => {
      const capture = await collect(view, undefined, scope, paletteCurrent);
      if ("status" in capture) return;
      if (!capture.choices.length) {
        opts.onMessage?.(scope === "source.fixAll" ? "The provider offers no file-wide diagnostic fixes."
          : scope === "source" ? "The provider offers no source actions." : "No code actions here.");
        return;
      }
      const actions = capture.choices.map(choice => choice.action);
      const selected = await opts.onCodeActions!(actions);
      if (!selected || captured !== capture) return;
      const choice = capture.choices.find(choice => choice.action === selected);
      if (choice) await choose(view, capture, choice.id);
      else refuse("The selected action is not from the retained provider choices.");
    })().catch(e => opts.onMessage?.(e instanceof Error ? e.message : String(e)));
    return true;
  };
  const scopeCommands = [
    {id: "fixAll", label: "File-wide diagnostic fixes", scope: "source.fixAll"},
    {id: "sourceActions", label: "Source actions", scope: "source"},
  ] as const;
  const extensions = !opts.client || !opts.onCodeActions ? [] : [
    editorLanguageActions.of({id: `${opts.uri}:actions`, label: `Inspect editor code actions (${opts.uri.slice(0, 200)})`,
      parameters: {}, enabled: view => enabled(view), execute: (view, _args, signal) => inspect(view, signal),
    }),
    ...(opts.onProviderPreview ? scopeCommands.map(({id, label, scope}) => editorLanguageActions.of({
      id: `${opts.uri}:actions:${id}`, label: `Inspect ${label.toLowerCase()} (${opts.uri.slice(0, 200)})`,
      parameters: {}, enabled: view => enabled(view, scope),
      command: {id, label, run: (view, current) => human(view, scope, current)},
      execute: (view, _args, signal) => inspect(view, signal, scope),
    })) : []),
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
    editorLanguageActions.of({ id: `${opts.uri}:actions-choose`, label: `${opts.onProviderPreview ? "Preview" : "Apply"} captured editor code action (${opts.uri.slice(0, 200)})`, revision: () => captured?.id,
      parameters: { snapshot: { type: "string", maxLength: 128 }, choice: { type: "string", maxLength: 128 } }, enabled: view => !!captured && matches(view, captured),
      execute: async (view, args, signal) => !captured || captured.id !== args.snapshot ? refuse("Code action snapshot changed. Inspect current choices again.") : choose(view, captured, args.choice as string, signal),
    }),
  ];
  return {extensions, human: view => human(view)};
}
</storycode:copy>

### Assemble `apps/web/src/lsp/codeActionController.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/codeActionController.ts"><storycode:paste select="#f-67a81de633f2-freeze"/><storycode:paste select="#f-67a81de633f2-Capture"/><storycode:paste select="#f-67a81de633f2-codeActionController"/></storycode:file>

</storycode:doc>
