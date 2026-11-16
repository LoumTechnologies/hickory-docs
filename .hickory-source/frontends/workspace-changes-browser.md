<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review complete changes through the existing menu

For maintainers changing workspace services independently of the editor renderer.

The existing Edit menu requests a rename, formatting or provider action. Capture all retained source buffers and the originating selection before asking; an obsolete provider answer or edited target cannot become a preview. The preview pages every before/after line without reserving editor space. Application preflights every target, including retained editors and IME state, before synchronously committing the prepared replacements. It changes drafts only. The same exact inverse model drives bounded workspace undo/redo; later local edits refuse coordinated undo rather than being overwritten. Provider commands remain explicit refusals. Ordered resource operations enter a distinct complete saved-source review, acquire every existing target and re-query provider actions after newly synchronized sources, bind compiler consequences and publish only after explicit save. Matching receipts route existing buffer and tab identities; Source recovery supplies the guarded inverse. No text-only portion of a refused resource action is applied.

Rename discovery acquires previously unopened reference targets through the existing workspace buffer loader. Their language sessions synchronize before a fresh reference query; the discovering answer cannot authorize edits against later-loaded text. Acquisition never activates a tab. Captured source snapshots remain immutable inputs to the final provider proposal, and complete draft-plan validation still precedes application. Native preparation retains its own fixed request-validity closure so an older reply cannot borrow a newer request's authority.

Provider choices resolve through their captured provider before entering the same complete change-plan preview. A latest-choice ticket and the fixed preparation snapshot reject late answers or a replaced choice list. Disabled actions and complete edit-plus-command results refuse explicitly; resolving never applies an edit portion or saves source. The native view passes the rendered preparation identity so an old row index cannot select a newer list item.

Provider edit discovery uses the same bounded acquisition boundary. Newly loaded targets synchronize before the host re-queries actions at the captured selection and resolves a uniquely matching title and kind. Changed or ambiguous choices refuse instead of selecting an arbitrary replacement. Tokens from the discovering response never authorize the final plan. Original draft snapshots remain fixed; only missing sources are added, and acquisition does not activate tabs.

Explicit cleanup enters before language-provider acquisition. Its fixed source, selection, buffer identity and request ticket authorize only that preparation. Returning ordinary source replacements lets the existing preview, writability checks, coordinated apply and inverse history handle it; no second editor or save path is introduced. A no-op reports the observation without creating an undo entry. The cleanup profile is literal and explicit: significant trailing spaces can change, so review the before/after text.

Reviewed capabilities can offer a complete source proposal without a language provider. Every target must have the exact supplied reviewed text, and the ordinary planner and writability checks must accept the whole proposal before it becomes a preview. Capture target buffer identities, text and selections for the approval boundary. Source or selection changes refuse; successful approval uses the same coordinated inverse history as provider changes. The proposal mode has no provider-preparation button and never saves implicitly.

<ui:copy id="simple-browser-workspace-changes">
import { useEffect, useRef, useState } from "react";
import {api,request} from "../api/client";
import {ResourceConsequencesView} from "./SourceRecovery";
import {isResourceEdit,prepareResourceSave,prepareResourceRestore,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute} from "./ResourceEditors";
import { createPortal } from "react-dom";
import { isolateHistory } from "@codemirror/commands";
import { type Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorForBuffer } from "./Document";
import {resolveCodeAction} from "../lsp/codeActionResolution";
import {acquireReferenceSources,acquireSourceProposal} from "./ReferenceSources";
import type {FileNode} from "../api/types";
import { useWorkspaceLanguageLookup,useWorkspaceOutlineReader } from "./WorkspaceLanguages";
import { utf16ToPosition } from "../lsp/positions";
import type { CodeAction } from "../lsp/client";
import type { Buffer, Buffers } from "./buffers";
import { semanticLocations, semanticPath } from "./Intelligence";
import { WorkspaceChangeHistory,type WorkspaceHistoryToken,providerActionPaths,validateReferencedTargets, planWorkspaceEdit, invertChangePlan, validateChangePlan, type SourceSnapshot, type ChangePlan } from "./ChangePlan";
import {workspaceChangeEventName as eventName,type ChangeCommand} from "./WorkspaceChangeEvents";
export type {ChangeCommand} from "./WorkspaceChangeEvents";
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
export function WorkspaceChanges({ buffers, current, focus,files,adoptResources,history }: { files:FileNode[];buffers: Buffers; current: () => Buffer | undefined; focus: () => void;adoptResources:(routes:ResourceEditorRoute<Buffer>[])=>void;history:WorkspaceChangeHistory }) {
  const lookup = useWorkspaceLanguageLookup(),readOutline=useWorkspaceOutlineReader();
  const [mode, setMode] = useState<ChangeCommand | "proposal" | null>(null), [name, setName] = useState(""), [notice, setNotice] = useState("");
  const [resource,setResource]=useState<ResourceRestoreReview|null>(null),[publishing,setPublishing]=useState(false);
  const [plan, setPlan] = useState<ChangePlan | null>(null), [choices, setChoices] = useState<CodeAction[]>([]);
  const [file, setFile] = useState(0), [page, setPage] = useState(0), [actionPage, setActionPage] = useState(0);
  const resourceHistory=useRef<WorkspaceHistoryToken|null>(null),busy=useRef(false),ticket = useRef(0),choiceTicket=useRef(0);
  const captured = useRef<{ sources: Map<string, SourceSnapshot>; references: string[]; current: () => boolean;request:number;choices:CodeAction[];resolve:(action:CodeAction)=>Promise<CodeAction>;acquire:(path:string)=>Promise<SourceSnapshot>;query?:()=>Promise<CodeAction[]> } | null>(null);
  const input = useRef<HTMLInputElement>(null), panel = useRef<HTMLDivElement>(null);
  const close = () => { if(publishing)return;ticket.current++; setResource(null);setMode(null); focus(); };
  async function acquireResource(path:string){const existing=buffers.values.get(path);if(existing)return existing;const response=await api.files();const find=(nodes:FileNode[]):FileNode|undefined=>{for(const file of nodes){if(!file.dir&&file.path===path)return file;const nested=find(file.children??[]);if(nested)return nested;}};const file=find(response.tree);if(!file)throw new Error("Current resource source is unavailable: "+path);return await buffers.open(file);}
  async function prepareHistory(token:WorkspaceHistoryToken,id:number){
    if(token.entry.kind!=="resource")return;setNotice("Preparing reviewed resource "+token.direction+"…");
    try{const record=await request<import("./SourceRecovery").RecoveryReading>("GET","/api/changes/resource-recovery?id="+encodeURIComponent(token.entry.recovery));
      if(record.id!==token.entry.recovery)throw new Error("Recovery identity changed.");
      const review=await prepareResourceRestore(buffers,record,acquireResource,(route,body)=>request("POST",route,body),()=>ticket.current===id&&history.current(token));
      if(ticket.current!==id||!history.current(token))throw new Error("Workspace history changed. Prepare again.");resourceHistory.current=token;setResource(review);setFile(0);setPage(0);setNotice("Review complete saved and draft consequences before workspace "+token.direction+".");
    }catch(error){if(ticket.current===id)setNotice(String(error));}
  }
  const install = async (title: string, edit: unknown) => {
    const capture = captured.current;
    if (!capture?.current()) throw new Error("Source, selection or provider changed. Prepare the proposal again.");
    resourceHistory.current=null;
    if(isResourceEdit(edit)){
      setPlan(null);setResource(null);setNotice("Preparing complete source resource changes…");
      const reviewed=await prepareResourceSave(buffers,title,edit,async path=>{await capture.acquire(path);const buffer=buffers.values.get(path);if(!buffer)throw new Error("Resource source was not retained.");return buffer;},(route,body)=>request("POST",route,body),()=>captured.current===capture&&capture.current());
      if(!capture.current())throw new Error("Source or provider changed. Prepare again.");setResource(reviewed);setChoices([]);setFile(0);setPage(0);setNotice("Review saved source consequences. This explicit action saves resources; generated outputs are not materialized.");return;
    }
    setResource(null);
    const next = planWorkspaceEdit(title, edit, capture.sources, mode === "cleanup"), refusal = validateChangePlan(next, workspaceSnapshots(buffers));
    if (refusal) throw new Error(refusal); validateReferencedTargets(next, capture.references); if(!next.patches.length){setPlan(null);setNotice("No trailing ASCII spaces or tabs to remove.");return;} setPlan(next); setChoices([]); setFile(0); setPage(0); setNotice("");
  };
  useEffect(() => {
    const command = (event: Event) => {
      if(busy.current)return;
      const detail=(event as CustomEvent).detail;
      if(detail?.kind==='proposal'){
        if(isResourceEdit(detail.edit)){
          try{const targets=[...detail.expected].map(([path,text]:[string,string])=>{const buffer=buffers.values.get(path),view=buffer&&editorForBuffer(buffer)?.view;if(!buffer||buffer.text!==text||buffer.readonly||buffer.saving||buffer.transient)throw new Error('Reviewed resource source changed or is unavailable.');return {path,buffer,text,view,selection:view?.state.selection};});
            const id=++ticket.current,valid=()=>id===ticket.current&&targets.every(target=>buffers.values.get(target.path)===target.buffer&&target.buffer.text===target.text&&(!target.view||!!target.selection&&target.view.state.selection.eq(target.selection)));
            captured.current={sources:workspaceSnapshots(buffers),references:[],current:valid,request:id,choices:[],resolve:async()=>{throw new Error('This is a reviewed source proposal.');},acquire:async path=>{const target=targets.find(item=>item.path===path);if(!target||!valid())throw new Error('Every existing resource target must be captured in the proposal.');return {text:target.text,writable:true};}};
            setMode('proposal');setPlan(null);setChoices([]);detail.accepted=true;void install(detail.title,detail.edit).catch(error=>{if(id===ticket.current)setNotice(String(error));});
          }catch(error){detail.error=String(error);}return;
        }
        try{
          const sources=workspaceSnapshots(buffers),next=planWorkspaceEdit(detail.title,detail.edit,sources);
          const targets=next.patches.map(patch=>{
            const buffer=buffers.values.get(patch.path),view=buffer&&editorForBuffer(buffer)?.view;
            if(!buffer||detail.expected.get(patch.path)!==patch.before||buffer.text!==patch.before)throw new Error('Reviewed source changed or a target was not captured. Prepare the whole proposal again.');
            return {buffer,text:patch.before,view,selection:view?.state.selection};
          });
          const refusal=validateChangePlan(next,sources);if(refusal)throw new Error(refusal);
          const id=++ticket.current,valid=()=>id===ticket.current&&targets.every(target=>buffers.values.get(target.buffer.file.path)===target.buffer&&target.buffer.text===target.text&&(!target.view||!!target.selection&&target.view.state.selection.eq(target.selection)));
          captured.current={sources,references:[],current:valid,request:id,choices:[],resolve:async()=>{throw new Error('Prepare provider actions first.');},acquire:async()=>{throw new Error('Reviewed source was already captured.');}};
          setMode('proposal');setPlan(next);setChoices([]);setNotice('');setFile(0);setPage(0);setActionPage(0);detail.accepted=true;
        }catch(error){detail.error=String(error);}
        return;
      }
      const next = (event as CustomEvent).detail as ChangeCommand;
      if (!["rename", "actions", "format", "cleanup", "undo", "redo"].includes(next)) return;
      if(busy.current)return;ticket.current++;resourceHistory.current=null; setResource(null);setMode(next); setPlan(null); setChoices([]); setName(""); setNotice(""); setActionPage(0);
      if (next === "undo" || next === "redo") {
        const token=history.preview(next);
        if (!token) { setNotice(`No workspace change to ${next}.`); return; }
        if(token.entry.kind==='resource'){void prepareHistory(token,ticket.current);return;}
        try { const target=token.entry.plan;applyWorkspacePlan(buffers, target);history.commit(token,{kind:'draft',plan:invertChangePlan(target)});setNotice(`Workspace ${next} applied to ${target.patches.length} drafts. Not saved or verified.`); }
        catch (error) { setNotice(String(error)); }
      }
    };
    window.addEventListener(eventName, command); return () => { ticket.current++; window.removeEventListener(eventName, command); };
  }, [buffers]);
  useEffect(() => { if (mode === "rename") input.current?.focus(); else if (mode) panel.current?.focus(); }, [mode]);
  async function prepare() {
    if(busy.current)return;
    if (mode === "cleanup") {
      const buffer=current(),view=buffer&&editorForBuffer(buffer)?.view;
      if(!buffer||!view||buffer.readonly||buffer.saving||buffer.transient){setNotice("Choose writable source before preparing cleanup.");return;}
      const source=buffer.text,selection=view.state.selection,id=++ticket.current;
      const valid=()=>id===ticket.current&&buffers.values.get(buffer.file.path)===buffer&&buffer.text===source&&view.state.doc.toString()===source&&view.state.selection.eq(selection);
      captured.current={sources:workspaceSnapshots(buffers),references:[],current:valid,request:id,choices:[],resolve:async()=>{throw new Error("Prepare provider actions first.");},acquire:async()=>{throw new Error("Cleanup has one captured source.");}};
      setPlan(null);setChoices([]);setNotice("Preparing trailing-whitespace cleanup…");
      try{
        const edit=await request<unknown>("POST","/api/changes/cleanup",{uri:"hick:///"+buffer.file.path.split("/").map(encodeURIComponent).join("/"),source,profile:"trim_trailing_horizontal"});
        if(!valid()){if(id===ticket.current)setNotice("Source or selection changed. Prepare cleanup again.");return;}
        await install("Trim trailing whitespace",edit);
      }catch(error){if(id===ticket.current)setNotice(String(error));}
      return;
    }
    const buffer = current(), view = buffer && editorForBuffer(buffer)?.view, session = buffer && lookup(buffer);
    if (!buffer || !view || !session?.client || !session.ready || buffer.readonly) { setNotice("Choose writable source with a connected language provider, then prepare again."); return; }
    const source = buffer.text, selection = view.state.selection, client = session.client, id = ++ticket.current, sources = workspaceSnapshots(buffers);
    const valid = () => id === ticket.current && buffer.text === source && view.state.doc.toString() === source && view.state.selection.eq(selection) && lookup(buffer).client === client && lookup(buffer).ready;
        const listed=new Map<string,FileNode>();const collect=(nodes:FileNode[])=>{for(const file of nodes){if(!file.dir)listed.set(file.path,file);collect(file.children??[]);}};collect(files);
    const acquire=async (path:string)=>{
          if(!valid())throw new Error("Source, selection or provider changed. Prepare again.");
          const file=buffers.values.get(path)?.file??listed.get(path);if(!file||file.dir)throw new Error(`Referenced source is no longer listed: ${path}. Refresh files and prepare again.`);
          const target=await buffers.open(file),text=target.text;
          if(target.readonly||target.saving||target.transient)throw new Error(`Source is generated, read-only or saving: ${path}`);
          await readOutline(target,text);
          if(!valid()||target.text!==text)throw new Error(`Source changed while reading ${path}. Prepare again.`);
          return {text,writable:!target.readonly&&!target.saving&&!target.transient};
        };
    captured.current = { sources, references: [], current: valid,request:id,choices:[],resolve:action=>resolveCodeAction(client,action,session.uri),acquire }; setPlan(null); setChoices([]); setNotice("Preparing complete source changes…");
    try {
      const uri = session.uri, position = utf16ToPosition(source, selection.main.head);
      const method = mode === "rename" ? "textDocument/rename" : mode === "format" ? "textDocument/formatting" : "textDocument/codeAction";
      const params = { textDocument: { uri }, ...(mode === "rename" ? { position, newName: name } : mode === "format" ? { options: { tabSize: view.state.tabSize, insertSpaces: !/^\t/m.test(source) } } : { range: { start: utf16ToPosition(source, selection.main.from), end: utf16ToPosition(source, selection.main.to) }, context: { diagnostics: [] } }) };
      captured.current!.query=async()=>{const value=await client.request("textDocument/codeAction",params);return Array.isArray(value)?value:[];};
      if (mode === "rename") {
        const acquired=await acquireReferenceSources(sources,async()=>{
          const references=semanticLocations(await client.request("textDocument/references",{textDocument:{uri},position,context:{includeDeclaration:true}}));
          return references.map(location=>{const path=semanticPath(location.uri);if(!path)throw new Error("Reference target has no writable workspace source identity. No edits were applied.");return path;});
        },acquire,valid);
        if(!valid())return;
        captured.current!.sources=acquired.sources;captured.current!.references=acquired.references;
      }
      const result = await client.request(method, params);
      if (!valid()) { if (id === ticket.current) setNotice("Source, selection or provider changed. Prepare the proposal again."); return; }
      if (mode === "actions") { const choices:CodeAction[]=Array.isArray(result)?structuredClone(result):[];captured.current!.choices=choices;setChoices(choices); setNotice(Array.isArray(result) && result.length ? "Choose a provider action to preview." : "No provider actions here. Support depends on the installed language server."); }
      else await install(mode === "rename" ? `Rename to ${name}` : "Format source", mode === "format" ? { changes: { [uri]: result } } : result);
    } catch (error) { if (id===ticket.current) setNotice(String(error)); }
  }
  const choose = async (action: CodeAction) => {
    if(busy.current)return;
    const capture=captured.current,choice=++choiceTicket.current;
    const owned=()=>!!capture&&captured.current===capture&&capture.request===ticket.current&&choiceTicket.current===choice;
    try {
      if(!capture||!capture.current()||!capture.choices.includes(action))throw new Error("Code action choices changed. Prepare actions again.");
      if(action.disabled)throw new Error(action.disabled.reason);
      setPlan(null);setNotice("Resolving complete provider action…");
      let first:CodeAction|undefined=await capture.resolve(action);
      const acquired=await acquireSourceProposal(capture.sources,async()=>{
        let resolved=first;first=undefined;
        if(!resolved){
          const candidates=(await capture.query!()).filter(item=>item.title===action.title&&(item.kind??"")===(action.kind??""));
          if(candidates.length!==1)throw new Error("The selected provider action changed or is ambiguous. Prepare actions again.");
          resolved=await capture.resolve(candidates[0]);
        }
        if(!owned()||!capture.current())throw new Error("Source, selection or provider changed. Prepare actions again.");
        if(resolved.disabled)throw new Error(resolved.disabled.reason);
        if(resolved.command)throw new Error("This action requires a provider command; its edit has not been applied.");
        let paths:string[];
        if(isResourceEdit(resolved.edit)){
          const observed=await request<{sources:Record<string,{kind:string}>;refusals:Record<string,string>}>("POST","/api/changes/resource-inputs",{context:crypto.randomUUID().replaceAll('-',''),edit:resolved.edit});
          if(Object.keys(observed.refusals).length)throw new Error("Resource sources cannot be acquired. Refresh the workspace and prepare again.");paths=Object.entries(observed.sources).filter(([,source])=>source.kind==='file').map(([path])=>path);
        }else paths=providerActionPaths(resolved);
        return {paths,value:resolved};
      },capture.acquire,()=>owned()&&capture.current());
      if(!owned())return;
      capture.sources=acquired.sources;const resolved=acquired.value;
      await install(resolved.title,resolved.edit);
    }catch(error){if(owned())setNotice(String(error));}
  };
  async function saveResources(){if(!resource||publishing)return;const review=resource,token=resourceHistory.current;choiceTicket.current++;if(token&&!history.current(token)){setNotice("Workspace history changed. Prepare again.");return;}setResource(null);busy.current=true;setPublishing(true);setNotice("Publishing reviewed source resources…");
    try{const result=await publishResourceRestore(buffers,review,(route,body)=>request("POST",route,body));adoptResources(result.routes);if(token)history.commit(token,{kind:'resource',recovery:result.recovery});else history.record({kind:'resource',recovery:result.recovery});resourceHistory.current=null;setNotice("Saved source resources. Review Source recovery record "+result.recovery+" for coordinated inverse restoration. Generated files and checks were not run.");}
    catch(error){setNotice(String(error)+" Preserve drafts and inspect Source recovery before another publication.");}finally{busy.current=false;setPublishing(false);}
  }
  function apply() {
    if (!plan) return;
    try { if (!captured.current?.current()) throw new Error("Source, selection or provider changed. Prepare the proposal again."); applyWorkspacePlan(buffers, plan); history.record({kind:'draft',plan:invertChangePlan(plan)}); setPlan(null); setNotice(`Applied ${plan.patches.length} source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch.`); }
    catch (error) { setNotice(String(error)); }
  }
</ui:copy>

## Page the complete preview without docking it

A fixed floating dialog uses the existing menu, restores editor focus when dismissed, and pages all before/after lines. Pagination bounds rendered content rather than treating excerpts as the entire change. The renderer can choose a provider action or ask the controller to apply; it cannot forge replacements or save files.

<ui:copy id="simple-browser-workspace-changes-presentation">
  if (!mode) return null;
  const patch = plan?.patches[Math.min(file, plan.patches.length - 1)], before = patch?.before.split("\n") ?? [], after = patch?.after.split("\n") ?? [], last = Math.max(0, Math.ceil(Math.max(before.length, after.length) / 100) - 1), selected = Math.min(page, last);
  return createPortal(<div ref={panel} tabIndex={-1} className="simple-workspace-problems" role="dialog" aria-label="Workspace change preview" onKeyDown={event => { if (event.key === "Escape") { event.preventDefault(); close(); } }}>
    <button disabled={publishing} aria-label="Close change preview" onClick={close}>×</button><strong>Workspace change preview</strong><p>{resource?"Reviewed source resource publication":"Source drafts · not saved or verified"}</p>
    {mode === "rename" && <input disabled={publishing} ref={input} aria-label="New symbol name" value={name} onChange={event => { ticket.current++; setPlan(null); setChoices([]); setName(event.target.value); setNotice(""); }} />}
    {!["undo", "redo", "proposal"].includes(mode) && <button disabled={publishing||(mode === "rename" && !name.trim())} onClick={() => { void prepare(); }}>Prepare {mode === "actions" ? "provider actions" : mode === "format" ? "formatting" : mode === "cleanup" ? "cleanup" : "rename"}</button>}
    {notice && <p role="status">{notice}</p>}
    {resource&&<section aria-label="Resource change preview"><strong>{resource.plan.title}</strong><p>{resource.patches.length} reviewed paths · saved sources and drafts</p><select aria-label="Resource preview source" value={file} onChange={event=>{setFile(Number(event.target.value));setPage(0);}}>{resource.patches.map((item,index)=><option key={item.path} value={index}>{item.path}</option>)}</select>
    {resource.patches.slice(file,file+1).map(item=><div key={item.path}><strong>Current draft</strong><pre>{item.before?.text.slice(page*16000,(page+1)*16000)??'Source absent'}</pre><strong>After draft</strong><pre>{item.after?.text.slice(page*16000,(page+1)*16000)??'Source absent'}</pre><strong>Current saved baseline</strong><pre>{resource.savedBefore[item.path]?.kind==='file'?(resource.savedBefore[item.path] as {text:string}).text.slice(page*16000,(page+1)*16000):'Source absent'}</pre><strong>After saved baseline</strong><pre>{resource.saved[item.path]?.kind==='file'?(resource.saved[item.path] as {text:string}).text.slice(page*16000,(page+1)*16000):'Source absent'}</pre><p>Text page {page+1} · 16,000 characters per side</p><button disabled={!page} onClick={()=>setPage(page-1)}>Previous text</button><button disabled={Math.max(item.before?.text.length??0,item.after?.text.length??0,resource.savedBefore[item.path]?.kind==='file'?(resource.savedBefore[item.path] as {text:string}).text.length:0,resource.saved[item.path]?.kind==='file'?(resource.saved[item.path] as {text:string}).text.length:0)<=(page+1)*16000} onClick={()=>setPage(page+1)}>Next text</button></div>)}
    <ResourceConsequencesView review={resource}/><button disabled={publishing} onClick={()=>{void saveResources();}}>Save reviewed source resources</button></section>}
    {choices.slice(actionPage * 100, (actionPage + 1) * 100).map((action, index) => <div key={index}><button onClick={() => {void choose(action);}}>{action.title}</button>{action.disabled && <small>{action.disabled.reason}</small>}</div>)}
    {choices.length > 100 && <div><button disabled={!actionPage} onClick={() => setActionPage(actionPage - 1)}>Previous actions</button><button disabled={(actionPage + 1) * 100 >= choices.length} onClick={() => setActionPage(actionPage + 1)}>Next actions</button></div>}
    {plan && patch && <><p>{plan.title} · {plan.patches.length} source files</p>{mode === "rename" && <p>Reference targets: {new Set(captured.current?.references).size} files · project analysis scope not established</p>}<select aria-label="Preview source file" value={file} onChange={event => { setFile(Number(event.target.value)); setPage(0); }}>{plan.patches.map((patch, index) => <option key={patch.path} value={index}>{patch.path}</option>)}</select>
      <div><strong>Before · lines {selected * 100 + 1}–{Math.min(before.length, (selected + 1) * 100)}</strong><pre>{before.slice(selected * 100, (selected + 1) * 100).join("\n")}</pre><strong>After</strong><pre>{after.slice(selected * 100, (selected + 1) * 100).join("\n")}</pre></div>
      {last > 0 && <div><button disabled={!selected} onClick={() => setPage(selected - 1)}>Previous lines</button><span>{selected + 1}/{last + 1}</span><button disabled={selected === last} onClick={() => setPage(selected + 1)}>Next lines</button></div>}
      <button onClick={apply}>Apply to source drafts</button></>}
  </div>, document.body);
}
</ui:copy>



## Offer reviewed source changes through the existing preview

A source-aware capability can offer ordinary replacements with the exact text it reviewed. The event is handled synchronously by the existing change controller; acceptance means a preview was prepared, not that source was edited or saved. Refuse an absent controller instead of leaving a pending operation.

<ui:copy id="reviewed-source-browser-request">
export {requestWorkspaceChange,requestWorkspaceProposal} from "./WorkspaceChangeEvents";
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceChanges.tsx"><ui:paste select="#simple-browser-workspace-changes"/><ui:paste select="#simple-browser-workspace-changes-controller"/><ui:paste select="#simple-browser-workspace-changes-presentation"/><ui:paste select="#reviewed-source-browser-request"/></ui:file>
</ui:doc>
