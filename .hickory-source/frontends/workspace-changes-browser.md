<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review complete changes through the existing menu

For maintainers changing workspace services independently of the editor renderer.

The existing Edit menu requests a rename, formatting or provider action. Capture all retained source buffers and the originating selection before asking; an obsolete provider answer or edited target cannot become a preview. The preview pages every before/after line without reserving editor space. Application preflights every target, including retained editors and IME state, before synchronously committing the prepared replacements. It changes drafts only. The same exact inverse model drives bounded workspace undo/redo; later local edits refuse coordinated undo rather than being overwritten. Provider commands and resource operations remain explicit unsupported results, not partially applied edits.

Rename discovery acquires previously unopened reference targets through the existing workspace buffer loader. Their language sessions synchronize before a fresh reference query; the discovering answer cannot authorize edits against later-loaded text. Acquisition never activates a tab. Captured source snapshots remain immutable inputs to the final provider proposal, and complete draft-plan validation still precedes application. Native preparation retains its own fixed request-validity closure so an older reply cannot borrow a newer request's authority.

Provider choices resolve through their captured provider before entering the same complete change-plan preview. A latest-choice ticket and the fixed preparation snapshot reject late answers or a replaced choice list. Disabled actions and complete edit-plus-command results refuse explicitly; resolving never applies an edit portion or saves source. The native view passes the rendered preparation identity so an old row index cannot select a newer list item.

<ui:copy id="simple-browser-workspace-changes">
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { isolateHistory } from "@codemirror/commands";
import { type Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorForBuffer } from "./Document";
import {resolveCodeAction} from "../lsp/codeActionResolution";
import {acquireReferenceSources} from "./ReferenceSources";
import type {FileNode} from "../api/types";
import { useWorkspaceLanguageLookup,useWorkspaceOutlineReader } from "./WorkspaceLanguages";
import { utf16ToPosition } from "../lsp/positions";
import type { CodeAction } from "../lsp/client";
import type { Buffer, Buffers } from "./buffers";
import { semanticLocations, semanticPath } from "./Intelligence";
import { validateReferencedTargets, planWorkspaceEdit, invertChangePlan, validateChangePlan, type SourceSnapshot, type ChangePlan } from "./ChangePlan";
export type ChangeCommand = "rename" | "actions" | "format" | "undo" | "redo";
const eventName = "hickory:simple:workspace-change";
export function requestWorkspaceChange(command: ChangeCommand) { window.dispatchEvent(new CustomEvent(eventName, { detail: command })); }
export function workspaceSnapshots(buffers: Buffers): Map<string, SourceSnapshot> { return new Map([...buffers.values].map(([path, buffer]) => [path, { text: buffer.text, writable: !buffer.readonly && !buffer.saving && !buffer.transient }])); }
export function applyWorkspacePlan(buffers: Buffers, plan: ChangePlan): void {
  const refusal = validateChangePlan(plan, workspaceSnapshots(buffers)); if (refusal) throw new Error(refusal);
  const prepared: { buffer: Buffer; after: string; view?: EditorView; transaction?: Transaction }[] = [];
  for (const patch of plan.patches) {
    const buffer = buffers.values.get(patch.path)!, view = editorForBuffer(buffer)?.view;
    if (view && (view.composing || view.state.doc.toString() !== patch.before || view.state.readOnly || !view.state.facet(EditorView.editable))) throw new Error(`Finish composition or restore current source before applying: ${patch.path}`);
    const transaction = view?.state.update({ changes: patch.edits.map(edit => ({ from: edit.from, to: edit.to, insert: edit.text })), annotations: isolateHistory.of("full"), userEvent: "input.workspace", filter: false });
    if (transaction && transaction.newDoc.toString() !== patch.after) throw new Error(`Editor could not prepare the complete change: ${patch.path}`);
    prepared.push({ buffer, after: patch.after, view, transaction });
  }
  // Preparation above is read-only. No await separates these trusted local commits.
  for (const target of prepared) {
    if (target.view && target.transaction) target.view.update([target.transaction]);
    if (target.buffer.text !== target.after) target.buffer.edit(target.after);
  }
}
</ui:copy>

## Retain provider choices beside their source capture

A menu command starts a fresh request identity. Retain the exact provider response and every target snapshot; source, selection or connection changes make it unavailable. Choosing an action prepares the same pure plan as rename or formatting. Applying records its inverse only after the complete preflight succeeds. Local editor undo remains independent, so coordinated undo validates every target again.

<ui:copy id="simple-browser-workspace-changes-controller">
export function WorkspaceChanges({ buffers, current, focus,files }: { files:FileNode[];buffers: Buffers; current: () => Buffer | undefined; focus: () => void }) {
  const lookup = useWorkspaceLanguageLookup(),readOutline=useWorkspaceOutlineReader();
  const [mode, setMode] = useState<ChangeCommand | null>(null), [name, setName] = useState(""), [notice, setNotice] = useState("");
  const [plan, setPlan] = useState<ChangePlan | null>(null), [choices, setChoices] = useState<CodeAction[]>([]);
  const [file, setFile] = useState(0), [page, setPage] = useState(0), [actionPage, setActionPage] = useState(0);
  const undo = useRef<ChangePlan[]>([]), redo = useRef<ChangePlan[]>([]), ticket = useRef(0),choiceTicket=useRef(0);
  const captured = useRef<{ sources: Map<string, SourceSnapshot>; references: string[]; current: () => boolean;request:number;choices:CodeAction[];resolve:(action:CodeAction)=>Promise<CodeAction> } | null>(null);
  const input = useRef<HTMLInputElement>(null), panel = useRef<HTMLDivElement>(null);
  const close = () => { ticket.current++; setMode(null); focus(); };
  const install = (title: string, edit: unknown) => {
    const capture = captured.current;
    if (!capture?.current()) throw new Error("Source, selection or provider changed. Prepare the proposal again.");
    const next = planWorkspaceEdit(title, edit, capture.sources), refusal = validateChangePlan(next, workspaceSnapshots(buffers));
    if (refusal) throw new Error(refusal); validateReferencedTargets(next, capture.references); setPlan(next); setChoices([]); setFile(0); setPage(0); setNotice("");
  };
  useEffect(() => {
    const command = (event: Event) => {
      const next = (event as CustomEvent).detail as ChangeCommand;
      if (!["rename", "actions", "format", "undo", "redo"].includes(next)) return;
      ticket.current++; setMode(next); setPlan(null); setChoices([]); setName(""); setNotice(""); setActionPage(0);
      if (next === "undo" || next === "redo") {
        const history = next === "undo" ? undo.current : redo.current, target = history.at(-1);
        if (!target) { setNotice(`No workspace change to ${next}.`); return; }
        try { applyWorkspacePlan(buffers, target); history.pop(); const inverse = invertChangePlan(target); (next === "undo" ? redo.current : undo.current).push(inverse); setNotice(`Workspace ${next} applied to ${target.patches.length} drafts. Not saved or verified.`); }
        catch (error) { setNotice(String(error)); }
      }
    };
    window.addEventListener(eventName, command); return () => { ticket.current++; window.removeEventListener(eventName, command); };
  }, [buffers]);
  useEffect(() => { if (mode === "rename") input.current?.focus(); else if (mode) panel.current?.focus(); }, [mode]);
  async function prepare() {
    const buffer = current(), view = buffer && editorForBuffer(buffer)?.view, session = buffer && lookup(buffer);
    if (!buffer || !view || !session?.client || !session.ready || buffer.readonly) { setNotice("Choose writable source with a connected language provider, then prepare again."); return; }
    const source = buffer.text, selection = view.state.selection, client = session.client, id = ++ticket.current, sources = workspaceSnapshots(buffers);
    const valid = () => id === ticket.current && buffer.text === source && view.state.doc.toString() === source && view.state.selection.eq(selection) && lookup(buffer).client === client && lookup(buffer).ready;
    captured.current = { sources, references: [], current: valid,request:id,choices:[],resolve:action=>resolveCodeAction(client,action,session.uri) }; setPlan(null); setChoices([]); setNotice("Preparing complete source changes…");
    try {
      const uri = session.uri, position = utf16ToPosition(source, selection.main.head);
      const method = mode === "rename" ? "textDocument/rename" : mode === "format" ? "textDocument/formatting" : "textDocument/codeAction";
      const params = { textDocument: { uri }, ...(mode === "rename" ? { position, newName: name } : mode === "format" ? { options: { tabSize: view.state.tabSize, insertSpaces: !/^\t/m.test(source) } } : { range: { start: utf16ToPosition(source, selection.main.from), end: utf16ToPosition(source, selection.main.to) }, context: { diagnostics: [] } }) };
      if (mode === "rename") {
        const listed=new Map<string,FileNode>();const collect=(nodes:FileNode[])=>{for(const file of nodes){if(!file.dir)listed.set(file.path,file);collect(file.children??[]);}};collect(files);
        const acquired=await acquireReferenceSources(sources,async()=>{
          const references=semanticLocations(await client.request("textDocument/references",{textDocument:{uri},position,context:{includeDeclaration:true}}));
          return references.map(location=>{const path=semanticPath(location.uri);if(!path)throw new Error("Reference target has no writable workspace source identity. No edits were applied.");return path;});
        },async path=>{
          if(!valid())throw new Error("Source, selection or provider changed. Prepare again.");
          const file=buffers.values.get(path)?.file??listed.get(path);if(!file||file.dir)throw new Error(`Referenced source is no longer listed: ${path}. Refresh files and prepare again.`);
          const target=await buffers.open(file),text=target.text;
          if(target.readonly||target.saving||target.transient)throw new Error(`Source is generated, read-only or saving: ${path}`);
          await readOutline(target,text);
          if(!valid()||target.text!==text)throw new Error(`Source changed while reading ${path}. Prepare again.`);
          return {text,writable:!target.readonly&&!target.saving&&!target.transient};
        },valid);
        if(!valid())return;
        captured.current!.sources=acquired.sources;captured.current!.references=acquired.references;
      }
      const result = await client.request(method, params);
      if (!valid()) { if (id === ticket.current) setNotice("Source, selection or provider changed. Prepare the proposal again."); return; }
      if (mode === "actions") { const choices:CodeAction[]=Array.isArray(result)?structuredClone(result):[];captured.current!.choices=choices;setChoices(choices); setNotice(Array.isArray(result) && result.length ? "Choose a provider action to preview." : "No provider actions here. Support depends on the installed language server."); }
      else install(mode === "rename" ? `Rename to ${name}` : "Format source", mode === "format" ? { changes: { [uri]: result } } : result);
    } catch (error) { if (id===ticket.current) setNotice(String(error)); }
  }
  const choose = async (action: CodeAction) => {
    const capture=captured.current,choice=++choiceTicket.current;
    const owned=()=>!!capture&&captured.current===capture&&capture.request===ticket.current&&choiceTicket.current===choice;
    try {
      if(!capture||!capture.current()||!capture.choices.includes(action))throw new Error("Code action choices changed. Prepare actions again.");
      if(action.disabled)throw new Error(action.disabled.reason);
      setPlan(null);setNotice("Resolving complete provider action…");
      const resolved=await capture.resolve(action);
      if(!owned())return;
      if(!capture.current())throw new Error("Source, selection or provider changed. Prepare actions again.");
      if(resolved.disabled)throw new Error(resolved.disabled.reason);
      if(resolved.command)throw new Error("This action requires a provider command; its edit has not been applied.");
      if(!resolved.edit)throw new Error("This provider action has no source edit to preview. Prepare actions again.");
      install(resolved.title,resolved.edit);
    }catch(error){if(owned())setNotice(String(error));}
  };
  function apply() {
    if (!plan) return;
    try { if (!captured.current?.current()) throw new Error("Source, selection or provider changed. Prepare the proposal again."); applyWorkspacePlan(buffers, plan); undo.current = [...undo.current.slice(-31), invertChangePlan(plan)]; redo.current = []; setPlan(null); setNotice(`Applied ${plan.patches.length} source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch.`); }
    catch (error) { setNotice(String(error)); }
  }
</ui:copy>

## Page the complete preview without docking it

A fixed floating dialog uses the existing menu, restores editor focus when dismissed, and pages all before/after lines. Pagination bounds rendered content rather than treating excerpts as the entire change. The renderer can choose a provider action or ask the controller to apply; it cannot forge replacements or save files.

<ui:copy id="simple-browser-workspace-changes-presentation">
  if (!mode) return null;
  const patch = plan?.patches[Math.min(file, plan.patches.length - 1)], before = patch?.before.split("\n") ?? [], after = patch?.after.split("\n") ?? [], last = Math.max(0, Math.ceil(Math.max(before.length, after.length) / 100) - 1), selected = Math.min(page, last);
  return createPortal(<div ref={panel} tabIndex={-1} className="simple-workspace-problems" role="dialog" aria-label="Workspace change preview" onKeyDown={event => { if (event.key === "Escape") { event.preventDefault(); close(); } }}>
    <button aria-label="Close change preview" onClick={close}>×</button><strong>Workspace change preview</strong><p>Source drafts · not saved or verified</p>
    {mode === "rename" && <input ref={input} aria-label="New symbol name" value={name} onChange={event => { ticket.current++; setPlan(null); setChoices([]); setName(event.target.value); setNotice(""); }} />}
    {!["undo", "redo"].includes(mode) && <button disabled={mode === "rename" && !name.trim()} onClick={() => { void prepare(); }}>Prepare {mode === "actions" ? "provider actions" : mode === "format" ? "formatting" : "rename"}</button>}
    {notice && <p role="status">{notice}</p>}
    {choices.slice(actionPage * 100, (actionPage + 1) * 100).map((action, index) => <div key={index}><button onClick={() => {void choose(action);}}>{action.title}</button>{action.disabled && <small>{action.disabled.reason}</small>}</div>)}
    {choices.length > 100 && <div><button disabled={!actionPage} onClick={() => setActionPage(actionPage - 1)}>Previous actions</button><button disabled={(actionPage + 1) * 100 >= choices.length} onClick={() => setActionPage(actionPage + 1)}>Next actions</button></div>}
    {plan && patch && <><p>{plan.title} · {plan.patches.length} source files</p>{mode === "rename" && <p>Reference targets: {new Set(captured.current?.references).size} files · project analysis scope not established</p>}<select aria-label="Preview source file" value={file} onChange={event => { setFile(Number(event.target.value)); setPage(0); }}>{plan.patches.map((patch, index) => <option key={patch.path} value={index}>{patch.path}</option>)}</select>
      <div><strong>Before · lines {selected * 100 + 1}–{Math.min(before.length, (selected + 1) * 100)}</strong><pre>{before.slice(selected * 100, (selected + 1) * 100).join("\n")}</pre><strong>After</strong><pre>{after.slice(selected * 100, (selected + 1) * 100).join("\n")}</pre></div>
      {last > 0 && <div><button disabled={!selected} onClick={() => setPage(selected - 1)}>Previous lines</button><span>{selected + 1}/{last + 1}</span><button disabled={selected === last} onClick={() => setPage(selected + 1)}>Next lines</button></div>}
      <button onClick={apply}>Apply to source drafts</button></>}
  </div>, document.body);
}
</ui:copy>



<ui:file path="apps/web/src/simple/WorkspaceChanges.tsx"><ui:paste select="#simple-browser-workspace-changes"/><ui:paste select="#simple-browser-workspace-changes-controller"/><ui:paste select="#simple-browser-workspace-changes-presentation"/></ui:file>
</ui:doc>
