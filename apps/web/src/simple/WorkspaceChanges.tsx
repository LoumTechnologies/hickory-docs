import {authoredSource,authoredOffset,authoredChanges,authoredTransactionSource} from './SourceText';
import { useEffect, useRef, useState } from "react";
import {api,request} from "../api/client";
import {ResourceConsequencesView} from "./SourceRecovery";
import {captureResourceRoster,EntryPublicationNoEffect,isResourceEdit,prepareResourceSave,prepareResourceRestore,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute} from "./ResourceEditors";
import {prepareEntryIntent,readFrozenEntrySelection,type FrozenEntrySelection} from "./WorkspaceEntrySelection";
import { createPortal } from "react-dom";
import { isolateHistory } from "@codemirror/commands";
import { type Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorForBuffer } from "./Document";
import {codeActionMatchesKind,resolveCodeAction} from "../lsp/codeActionResolution";
import {acquireReferenceSources,acquireSourceProposal} from "./ReferenceSources";
import {providerSourceProofPaths,validateProviderSourceProofs} from "./ProviderSourceProofs";
import type {FileNode} from "../api/types";
import { useWorkspaceLanguageLookup,useWorkspaceOutlineReader,useWorkspaceSourcePreparation } from "./WorkspaceLanguages";
import { utf16ToPosition } from "../lsp/positions";
import type { CodeAction } from "../lsp/client";
import type { Buffer, Buffers } from "./buffers";
import { semanticLocations, semanticPath } from "./Intelligence";
import { WorkspaceChangeHistory,type WorkspaceHistoryToken,providerActionPaths,validateReferencedTargets, planWorkspaceEdit, invertChangePlan, validateChangePlan, type SourceSnapshot, type ChangePlan, type WorkspaceHistoryEntry } from "./ChangePlan";
import {workspaceChangeEventName as eventName,type ChangeCommand} from "./WorkspaceChangeEvents";
export type {ChangeCommand} from "./WorkspaceChangeEvents";
export function scopedProviderActions(actions:CodeAction[],scope:string|null):CodeAction[]{const prefix=scope==="fixAll"?"source.fixAll":scope==="sourceActions"?"source":null;return prefix?actions.filter(action=>codeActionMatchesKind(action,prefix)):actions;}
export function fileWideFixActions(actions:CodeAction[]):CodeAction[]{return scopedProviderActions(actions,"fixAll");}
export function workspaceSnapshots(buffers: Buffers): Map<string, SourceSnapshot> { return new Map([...buffers.values].map(([path, buffer]) => [path, { text: buffer.text, writable: !buffer.readonly && !buffer.saving && !buffer.transient }])); }
export function captureDraftHistory(buffers:Buffers,plan:ChangePlan):WorkspaceHistoryEntry {
 const sealed={...plan,patches:plan.patches.map(patch=>({...patch,edits:patch.edits.map(edit=>({...edit}))}))},namespace=buffers.workspace,workspace=buffers.workspaceId,targets=sealed.patches.map(patch=>({patch,buffer:buffers.values.get(patch.path)}));
 return {kind:'draft',plan:sealed,read:()=>{
  const refuse=()=>{throw new Error('Workspace history no longer owns the original source editors. Drafts were retained.');};
  if(buffers.workspace!==namespace||buffers.workspaceId!==workspace||!targets.length)refuse();
  const patches=targets.map(({patch,buffer})=>{if(!buffer||(!!buffer.namespace&&buffer.namespace!==namespace)||buffer.readonly||buffer.saving||buffer.transient)refuse();const addresses=[...buffers.values].filter(([,owner])=>owner===buffer);if(addresses.length!==1||addresses[0][0]!==buffer!.file.path)refuse();return {...patch,path:addresses[0][0]};});
  if(new Set(patches.map(patch=>patch.path)).size!==patches.length)refuse();
  const current={...sealed,patches},failure=validateChangePlan(current,workspaceSnapshots(buffers));if(failure)throw new Error(failure);return current;
 }};
}
export function applyWorkspacePlan(buffers: Buffers, plan: ChangePlan): void {
  const refusal = validateChangePlan(plan, workspaceSnapshots(buffers)); if (refusal) throw new Error(refusal);
  const prepared: { buffer: Buffer; after: string; view?: EditorView; transaction?: Transaction }[] = [];
  for (const patch of plan.patches) {
    const buffer = buffers.values.get(patch.path)!, view = editorForBuffer(buffer)?.view;
    if (view && (view.composing || authoredSource(view.state) !== patch.before || view.state.readOnly || !view.state.facet(EditorView.editable))) throw new Error(`Finish composition or restore current source before applying: ${patch.path}`);
    const transaction = view?.state.update({ ...authoredChanges(view.state,patch.edits.map(edit => ({ from: edit.from, to: edit.to, insert: edit.text }))), annotations: isolateHistory.of("full"), userEvent: "input.workspace", filter: false });
    if (transaction && authoredTransactionSource(transaction) !== patch.after) throw new Error(`Editor could not prepare the complete change: ${patch.path}`);
    prepared.push({ buffer, after: patch.after, view, transaction });
  }
  // Preparation above is read-only. No await separates these trusted local commits.
  for (const target of prepared) {
    if (target.view && target.transaction) target.view.update([target.transaction]);
    if (target.buffer.text !== target.after) target.buffer.edit(target.after);
  }
}
export function WorkspaceChanges({ buffers, current, focus,files,adoptResources,history }: { files:FileNode[];buffers: Buffers; current: () => Buffer | undefined; focus: () => void;adoptResources:(routes:ResourceEditorRoute<Buffer>[])=>void;history:WorkspaceChangeHistory }) {
  const lookup = useWorkspaceLanguageLookup(),readOutline=useWorkspaceOutlineReader(),prepareSource=useWorkspaceSourcePreparation();
  const [mode, setMode] = useState<ChangeCommand | "proposal" | null>(null), [name, setName] = useState(""), [notice, setNotice] = useState("");
  const [resource,setResource]=useState<ResourceRestoreReview|null>(null),[publishing,setPublishing]=useState(false);
  const [entrySelection,setEntrySelection]=useState<FrozenEntrySelection|null>(null);
  const entryScope=useRef(0);
  const [plan, setPlan] = useState<ChangePlan | null>(null), [choices, setChoices] = useState<CodeAction[]>([]);
  const [file, setFile] = useState(0), [page, setPage] = useState(0), [actionPage, setActionPage] = useState(0);
  const resourceHistory=useRef<WorkspaceHistoryToken|null>(null),busy=useRef(false),ticket = useRef(0),choiceTicket=useRef(0);
  const captured = useRef<{ sources: Map<string, SourceSnapshot>; references: string[]; current: () => boolean;request:number;choices:CodeAction[];resolve:(action:CodeAction)=>Promise<CodeAction>;acquire:(path:string,requireWritable?:boolean)=>Promise<SourceSnapshot>;query?:()=>Promise<CodeAction[]> } | null>(null);
  const input = useRef<HTMLInputElement>(null), panel = useRef<HTMLDivElement>(null);
  const close = () => { if(publishing)return;ticket.current++;entryScope.current++;setEntrySelection(null);setResource(null);setMode(null);focus(); };
  async function acquireResource(path:string){const existing=buffers.values.get(path);if(existing)return existing;const response=await api.files();const find=(nodes:FileNode[]):FileNode|undefined=>{for(const file of nodes){if(!file.dir&&file.path===path)return file;const nested=find(file.children??[]);if(nested)return nested;}};const file=find(response.tree);if(!file)throw new Error("Current resource source is unavailable: "+path);return await buffers.open(file);}
  async function prepareHistory(token:WorkspaceHistoryToken,id:number){
    if(token.entry.kind!=="resource")return;setNotice("Preparing reviewed resource "+token.direction+"…");
    try{const record=await request<import("./SourceRecovery").RecoveryReading>("GET","/api/changes/resource-recovery?id="+encodeURIComponent(token.entry.recovery));
      if(record.id!==token.entry.recovery)throw new Error("Recovery identity changed.");
      const review=await prepareResourceRestore(buffers,record,acquireResource,(route,body)=>request("POST",route,body),
        ()=>ticket.current===id&&history.current(token),undefined,()=>request("GET","/api/workspace/entry"));
      if(ticket.current!==id||!history.current(token))throw new Error("Workspace history changed. Prepare again.");resourceHistory.current=token;setResource(review);setFile(0);setPage(0);setNotice("Review complete saved and draft consequences before workspace "+token.direction+".");
    }catch(error){if(ticket.current===id)setNotice(String(error));}
  }
  async function openEntryMove(scope:number) {
    const workspace=buffers.workspaceId,namespace=buffers.workspace;
    const current=()=>entryScope.current===scope&&buffers.workspaceId===workspace&&buffers.workspace===namespace;
    setNotice('Reading the current main construction…');
    try {
      if(!workspace||!namespace)throw new Error('Refresh Files to identify the current workspace.');
      const selection=await readFrozenEntrySelection(()=>request('GET','/api/workspace/entry'),{workspace,namespace,current});
      if(!selection.entryPath)throw new Error('This workspace has no configured main construction.');
      if(!selection.proof.movable)throw new Error('The current main construction cannot be moved. Check its saved source.');
      setEntrySelection(selection);
      setNotice('');
    }catch(error){if(current())setNotice(String(error));}
  }
  async function prepareEntryMove(selection:FrozenEntrySelection,destination:string) {
    const id=++ticket.current;
    setResource(null);
    setNotice('Preparing the main construction move…');
    try {
      if(!destination||destination===selection.entryPath)throw new Error('Enter a different workspace-relative destination.');
      const roster=captureResourceRoster(buffers);
      const valid=()=>id===ticket.current&&selection.current()&&roster.current();
      const intent=await prepareEntryIntent(selection,{kind:'move',destination},body=>request('POST','/api/workspace/entry/intent',body),destination,valid);
      const review=await prepareResourceSave(buffers,intent.title,intent.edit,async path=>{
        if(path!==selection.entryPath||!valid())throw new Error('Original main construction changed. Open the move again.');
        const wasRetained=buffers.values.has(path),buffer=await acquireResource(path);
        if(!wasRetained&&(buffer.sourceRevision!==0||buffer.text!==buffer.base))throw new Error('The acquired main construction changed. Open the move again.');
        roster.retain(path,buffer);
        return buffer;
      },(route,body)=>request('POST',route,body),valid,undefined,intent);
      if(!valid())throw new Error('Original source or destination changed. Open the move again.');
      setResource(review);setFile(0);setPage(0);
      setNotice('Review the move, saved source and retained draft before publishing.');
    }catch(error){if(id===ticket.current)setNotice(String(error));}
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
        entryScope.current++;setEntrySelection(null);
        if(isResourceEdit(detail.edit)){
          try{const targets=[...detail.expected].map(([path,text]:[string,string])=>{const buffer=buffers.values.get(path),view=buffer&&editorForBuffer(buffer)?.view;if(!buffer||buffer.text!==text||buffer.readonly||buffer.saving||buffer.transient)throw new Error('Reviewed resource source changed or is unavailable.');return {path,buffer,text,view,selection:view?.state.selection};});
            const roster=captureResourceRoster(buffers),id=++ticket.current,valid=()=>id===ticket.current&&roster.current()&&targets.every(target=>buffers.values.get(target.path)===target.buffer&&target.buffer.text===target.text&&(!target.view||!!target.selection&&target.view.state.selection.eq(target.selection)));
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
          const roster=captureResourceRoster(buffers),id=++ticket.current,valid=()=>id===ticket.current&&roster.current()&&targets.every(target=>buffers.values.get(target.buffer.file.path)===target.buffer&&target.buffer.text===target.text&&(!target.view||!!target.selection&&target.view.state.selection.eq(target.selection)));
          captured.current={sources,references:[],current:valid,request:id,choices:[],resolve:async()=>{throw new Error('Prepare provider actions first.');},acquire:async()=>{throw new Error('Reviewed source was already captured.');}};
          setMode('proposal');setPlan(next);setChoices([]);setNotice('');setFile(0);setPage(0);setActionPage(0);detail.accepted=true;
        }catch(error){detail.error=String(error);}
        return;
      }
      const next = (event as CustomEvent).detail as ChangeCommand;
      if (!["rename", "fixAll", "sourceActions", "actions", "format", "cleanup", "entryMove", "undo", "redo"].includes(next)) return;
      if(busy.current)return;ticket.current++;const scope=++entryScope.current;setEntrySelection(null);resourceHistory.current=null; setResource(null);setMode(next); setPlan(null); setChoices([]); setName(""); setNotice(""); setActionPage(0);
      if(next==='entryMove'){void openEntryMove(scope);return;}
      if (next === "undo" || next === "redo") {
        const token=history.preview(next);
        if (!token) { setNotice(`No workspace change to ${next}.`); return; }
        if(token.entry.kind==='resource'){void prepareHistory(token,ticket.current);return;}
        try { const target=history.readDraft(token);applyWorkspacePlan(buffers, target);history.commit(token,captureDraftHistory(buffers,invertChangePlan(target)));setNotice(`Workspace ${next} applied to ${target.patches.length} drafts. Not saved or verified.`); }
        catch (error) { setNotice(String(error)); }
      }
    };
    window.addEventListener(eventName, command); return () => { ticket.current++;entryScope.current++;window.removeEventListener(eventName, command); };
  }, [buffers]);
  useEffect(() => { if (mode === "rename") input.current?.focus(); else if (mode) panel.current?.focus(); }, [mode]);
  async function prepare() {
    if(busy.current)return;
    if(mode==='entryMove'){
      if(entrySelection)await prepareEntryMove(entrySelection,name);
      return;
    }
    if (mode === "cleanup") {
      const buffer=current(),view=buffer&&editorForBuffer(buffer)?.view;
      if(!buffer||!view||buffer.readonly||buffer.saving||buffer.transient){setNotice("Choose writable source before preparing cleanup.");return;}
      const source=buffer.text,selection=view.state.selection,id=++ticket.current,roster=captureResourceRoster(buffers);
      const valid=()=>id===ticket.current&&roster.current()&&buffers.values.get(buffer.file.path)===buffer&&buffer.text===source&&authoredSource(view.state)===source&&view.state.selection.eq(selection);
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
    const sourceRevision=buffer.sourceRevision;const source = buffer.text, selection = view.state.selection, client = session.client, id = ++ticket.current, sources = workspaceSnapshots(buffers);
    const roster=captureResourceRoster(buffers);
    const originCurrent = () => id === ticket.current && buffers.values.get(buffer.file.path)===buffer && buffer.sourceRevision===sourceRevision && buffer.text === source && authoredSource(view.state) === source && view.state.selection.eq(selection) && lookup(buffer).client === client && lookup(buffer).ready;
    const valid=()=>originCurrent()&&roster.current();
        const listed=new Map<string,FileNode>();const collect=(nodes:FileNode[])=>{for(const file of nodes){if(!file.dir)listed.set(file.path,file);collect(file.children??[]);}};collect(files);
    const acquire=async (path:string,requireWritable=true)=>{
          if(!valid())throw new Error("Source, selection or provider changed. Prepare again.");
          const file=buffers.values.get(path)?.file??listed.get(path);if(!file||file.dir)throw new Error(`Referenced source is no longer listed: ${path}. Refresh files and prepare again.`);
          const wasRetained=buffers.values.has(path),target=await buffers.open(file),text=target.text;
          if(!wasRetained&&(target.sourceRevision!==0||target.text!==target.base))throw new Error("The newly acquired source was edited before original ownership could be sealed.");
          if(!originCurrent())throw new Error("Original source changed during acquisition. Prepare again.");roster.retain(path,target);
          if(target.saving||requireWritable&&(target.readonly||target.transient))throw new Error(`Source is generated, read-only or saving: ${path}`);
          await readOutline(target,text);
          if(!valid()||target.text!==text)throw new Error(`Source changed while reading ${path}. Prepare again.`);
          return {text,writable:!target.readonly&&!target.saving&&!target.transient};
        };
    captured.current = { sources, references: [], current: valid,request:id,choices:[],resolve:action=>resolveCodeAction(client,action,session.uri),acquire }; setPlan(null); setChoices([]); setNotice("Preparing complete source changes…");
    try {
      const uri = session.uri, position = utf16ToPosition(source, authoredOffset(view.state,selection.main.head));
      const method = mode === "rename" ? "hick/rename" : mode === "format" ? "hick/formatting" : "textDocument/codeAction";
      const params = { textDocument: { uri }, ...(mode === "rename" ? { position, newName: name } : mode === "format" ? { options: { tabSize: view.state.tabSize, insertSpaces: !/^\t/m.test(source) } } : { range: { start: utf16ToPosition(source, (mode==="fixAll"||mode==="sourceActions") ? 0:authoredOffset(view.state,selection.main.from)), end: utf16ToPosition(source, (mode==="fixAll"||mode==="sourceActions") ? source.length:authoredOffset(view.state,selection.main.to)) }, context: { diagnostics: [],...((mode==="fixAll"||mode==="sourceActions") ? {only:[mode==="fixAll"?"source.fixAll":"source"]}:{}) } }) };
      captured.current!.query=async()=>{const value=await client.request("textDocument/codeAction",params);return Array.isArray(value)?scopedProviderActions(value,mode):[];};
      await prepareSource(buffer,source);if(!valid())throw new Error("Original source changed during provider preparation. Prepare again.");
      if (mode === "rename") {
        const acquired=await acquireReferenceSources(sources,async()=>{
          const references=semanticLocations(await client.request("textDocument/references",{textDocument:{uri},position,context:{includeDeclaration:true}}));
          return references.map(location=>{const path=semanticPath(location.uri);if(!path)throw new Error("Reference target has no writable workspace source identity. No edits were applied.");return path;});
        },acquire,valid);
        if(!valid())return;
        captured.current!.sources=acquired.sources;captured.current!.references=acquired.references;
      }
      if(mode==="rename"||mode==="format"){
        let editTargets=new Set<string>();const queryBound=async()=>{
          const receipt=await prepareSource(buffer,source);if(!valid()||receipt.client!==client||receipt.uri!==uri)throw new Error("Original source changed before the bound provider query. Prepare again.");
          const value=await client.request(method,{...params,expected:receipt.expected}) as CodeAction & {source_files_modified?:unknown};
          if(!valid()||!value||value.source_files_modified!==false||value.command||value.disabled)throw new Error(value?.disabled?.reason??"Provider did not confirm one complete no-effect source proposal.");
          let paths:string[];
          if(isResourceEdit(value.edit)){const observed=await request<{sources:Record<string,{kind:string}>;refusals:Record<string,string>}>("POST","/api/changes/resource-inputs",{context:crypto.randomUUID().replaceAll('-',''),edit:value.edit});if(Object.keys(observed.refusals).length)throw new Error("Resource preimages are unavailable. Prepare again.");paths=Object.entries(observed.sources).filter(([,source])=>source.kind==='file').map(([path])=>path);}
          else paths=providerActionPaths(value);
          editTargets=new Set(paths);return {paths:[...new Set([...paths,...providerSourceProofPaths(value)])],value};
        };
        const capture=captured.current!,acquired=await acquireSourceProposal(capture.sources,queryBound,path=>capture.acquire(path,editTargets.has(path)),valid,path=>editTargets.has(path));
        await validateProviderSourceProofs(acquired.value,acquired.sources,editTargets);if(!valid())throw new Error("Original sources changed during proof validation. Prepare again.");capture.sources=acquired.sources;await install(acquired.value.title,acquired.value.edit);return;
      }
      const result = await client.request(method, params);
      if (!valid()) { if (id === ticket.current) setNotice("Source, selection or provider changed. Prepare the proposal again."); return; }
      if (mode === "actions" || mode === "fixAll" || mode === "sourceActions") { const actions:CodeAction[]=Array.isArray(result)?structuredClone(result):[];const choices=scopedProviderActions(actions,mode);captured.current!.choices=choices;setChoices(choices); setNotice(choices.length ? "Choose one complete provider action to preview. Nothing is saved or verified." : mode==="fixAll" ? "The current provider offers no file-wide diagnostic fixes. No edits were applied.":mode==="sourceActions"?"No source cleanup or generation actions are available here. No edits were applied.":"No provider actions here. Support depends on the installed language server."); }
      else throw new Error("Prepare a supported source action.");
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
      if((mode==="fixAll"||mode==="sourceActions")&&!scopedProviderActions([first],mode).length)throw new Error("The provider changed the complete file-wide action kind. Prepare again.");
      let editTargets=new Set<string>();
      const acquired=await acquireSourceProposal(capture.sources,async()=>{
        let resolved=first;first=undefined;
        if(!resolved){
          const candidates=(await capture.query!()).filter(item=>item.title===action.title&&(item.kind??"")===(action.kind??""));
          if(candidates.length!==1)throw new Error("The selected provider action changed or is ambiguous. Prepare actions again.");
          resolved=await capture.resolve(candidates[0]);
        }
        if(!owned()||!capture.current())throw new Error("Source, selection or provider changed. Prepare actions again.");
        if((mode==="fixAll"||mode==="sourceActions")&&!scopedProviderActions([resolved],mode).length)throw new Error("The provider changed the complete file-wide action kind. Prepare again.");
        if(resolved.disabled)throw new Error(resolved.disabled.reason);
        if(resolved.command)throw new Error("This action requires a provider command; its edit has not been applied.");
        let paths:string[];
        if(isResourceEdit(resolved.edit)){
          const observed=await request<{sources:Record<string,{kind:string}>;refusals:Record<string,string>}>("POST","/api/changes/resource-inputs",{context:crypto.randomUUID().replaceAll('-',''),edit:resolved.edit});
          if(Object.keys(observed.refusals).length)throw new Error("Resource sources cannot be acquired. Refresh the workspace and prepare again.");paths=Object.entries(observed.sources).filter(([,source])=>source.kind==='file').map(([path])=>path);
        }else paths=providerActionPaths(resolved);
        editTargets=new Set(paths);return {paths:[...new Set([...paths,...providerSourceProofPaths(resolved)])],value:resolved};
      },path=>capture.acquire(path,editTargets.has(path)),()=>owned()&&capture.current(),path=>editTargets.has(path));
      if(!owned())return;
      capture.sources=acquired.sources;const resolved=acquired.value;
      await validateProviderSourceProofs(resolved,acquired.sources,editTargets);if(!owned()||!capture.current())throw new Error("Original sources changed during proof validation. Prepare again.");
      await install(resolved.title,resolved.edit);
    }catch(error){if(owned())setNotice(String(error));}
  };
  async function saveResources(){if(!resource||publishing)return;const review=resource,token=resourceHistory.current;choiceTicket.current++;if(token&&!history.current(token)){setNotice("Workspace history changed. Prepare again.");return;}setResource(null);busy.current=true;setPublishing(true);setNotice("Publishing reviewed source resources…");
    try{const result=await publishResourceRestore(buffers,review,(route,body)=>request("POST",route,body));adoptResources(result.routes);if(token)history.commit(token,{kind:'resource',recovery:result.recovery});else history.record({kind:'resource',recovery:result.recovery});resourceHistory.current=null;
      if(review.entryIntent){entryScope.current++;setEntrySelection(null);}
      setNotice("Saved source resources. Review Source recovery record "+result.recovery+" for coordinated inverse restoration. Generated files and checks were not run.");}
    catch(error){
      if(error instanceof EntryPublicationNoEffect){resourceHistory.current=null;entryScope.current++;setEntrySelection(null);setNotice(error.message);}
      else setNotice(String(error)+" Preserve drafts and inspect Source recovery before another publication.");
    }finally{busy.current=false;setPublishing(false);}
  }
  function apply() {
    if (!plan) return;
    try { if (!captured.current?.current()) throw new Error("Source, selection or provider changed. Prepare the proposal again."); applyWorkspacePlan(buffers, plan); history.record(captureDraftHistory(buffers,invertChangePlan(plan))); setPlan(null); setNotice(`Applied ${plan.patches.length} source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch.`); }
    catch (error) { setNotice(String(error)); }
  }
  if (!mode) return null;
  const patch = plan?.patches[Math.min(file, plan.patches.length - 1)], before = patch?.before.split("\n") ?? [], after = patch?.after.split("\n") ?? [], last = Math.max(0, Math.ceil(Math.max(before.length, after.length) / 100) - 1), selected = Math.min(page, last);
  return createPortal(<div ref={panel} tabIndex={-1} className="simple-workspace-problems" role="dialog" aria-label="Workspace change preview" onKeyDown={event => { if (event.key === "Escape") { event.preventDefault(); close(); } }}>
    <button disabled={publishing} title={publishing ? "A reviewed resource change is being published." : undefined} aria-label="Close change preview" onClick={close}>×</button><strong>Workspace change preview</strong>
    {mode!=='entryMove'&&<p>{resource?"Reviewed source resource publication":"Source drafts · not saved or verified"}</p>}
    {mode === "rename" && <input disabled={publishing} title={publishing ? "A reviewed resource change is being published." : undefined} ref={input} aria-label="New symbol name" value={name} onChange={event => { ticket.current++; setPlan(null); setChoices([]); setName(event.target.value); setNotice(""); }} />}
    {mode==='entryMove'&&entrySelection&&<>
      <p>Current main construction <code>{entrySelection.entryPath}</code></p>
      <label>New workspace-relative path <input disabled={publishing} title={publishing ? "A reviewed resource change is being published." : undefined} value={name} onChange={event=>{ticket.current++;setResource(null);setName(event.target.value);setNotice('');}} /></label>
      <button disabled={publishing||!name||name===entrySelection.entryPath} title={publishing ? "A reviewed resource change is being published." : !name ? "Enter a destination path." : name===entrySelection.entryPath ? "Enter a different destination path." : undefined} onClick={()=>void prepare()}>Prepare move</button>
    </>}
    {!["undo", "redo", "proposal", "entryMove"].includes(mode) && <button disabled={publishing||(mode === "rename" && !name.trim())} title={publishing ? "A reviewed resource change is being published." : (mode === "rename" && !name.trim()) ? "Enter a new symbol name." : undefined} onClick={() => { void prepare(); }}>Prepare {mode === "fixAll" ? "file-wide diagnostic fixes" : mode === "sourceActions" ? "source actions" : mode === "actions" ? "provider actions" : mode === "format" ? "formatting" : mode === "cleanup" ? "cleanup" : "rename"}</button>}
    {notice && <p role="status">{notice}</p>}
    {resource&&<section aria-label="Resource change preview"><strong>{resource.plan.title}</strong><p>{resource.patches.length} reviewed paths · saved sources and drafts</p><select aria-label="Resource preview source" value={file} onChange={event=>{setFile(Number(event.target.value));setPage(0);}}>{resource.patches.map((item,index)=><option key={item.path} value={index}>{item.path}</option>)}</select>
    {resource.patches.slice(file,file+1).map(item=><div key={item.path}><strong>Current draft</strong><pre>{item.before?.text.slice(page*16000,(page+1)*16000)??'Source absent'}</pre><strong>After draft</strong><pre>{item.after?.text.slice(page*16000,(page+1)*16000)??'Source absent'}</pre><strong>Current saved baseline</strong><pre>{resource.savedBefore[item.path]?.kind==='file'?(resource.savedBefore[item.path] as {text:string}).text.slice(page*16000,(page+1)*16000):'Source absent'}</pre><strong>After saved baseline</strong><pre>{resource.saved[item.path]?.kind==='file'?(resource.saved[item.path] as {text:string}).text.slice(page*16000,(page+1)*16000):'Source absent'}</pre><p>Text page {page+1} · 16,000 characters per side</p><button disabled={!page} title={!page ? "You’re on the first page." : undefined} onClick={()=>setPage(page-1)}>Previous text</button><button disabled={Math.max(item.before?.text.length??0,item.after?.text.length??0,resource.savedBefore[item.path]?.kind==='file'?(resource.savedBefore[item.path] as {text:string}).text.length:0,resource.saved[item.path]?.kind==='file'?(resource.saved[item.path] as {text:string}).text.length:0)<=(page+1)*16000} title={Math.max(item.before?.text.length??0,item.after?.text.length??0,resource.savedBefore[item.path]?.kind==='file'?(resource.savedBefore[item.path] as {text:string}).text.length:0,resource.saved[item.path]?.kind==='file'?(resource.saved[item.path] as {text:string}).text.length:0)<=(page+1)*16000 ? "You’re on the last page." : undefined} onClick={()=>setPage(page+1)}>Next text</button></div>)}
    <ResourceConsequencesView review={resource}/><button disabled={publishing} title={publishing ? "A reviewed resource change is being published." : undefined} onClick={()=>{void saveResources();}}>Save reviewed source resources</button></section>}
    {choices.slice(actionPage * 100, (actionPage + 1) * 100).map((action, index) => <div key={index}><button onClick={() => {void choose(action);}}>{action.title}</button>{action.disabled && <small>{action.disabled.reason}</small>}</div>)}
    {choices.length > 100 && <div><button disabled={!actionPage} title={!actionPage ? "You’re on the first page." : undefined} onClick={() => setActionPage(actionPage - 1)}>Previous actions</button><button disabled={(actionPage + 1) * 100 >= choices.length} title={(actionPage + 1) * 100 >= choices.length ? "You’re on the last page." : undefined} onClick={() => setActionPage(actionPage + 1)}>Next actions</button></div>}
    {plan && patch && <><p>{plan.title} · {plan.patches.length} source files</p>{mode === "rename" && <p>Reference targets: {new Set(captured.current?.references).size} files · project analysis scope not established</p>}<select aria-label="Preview source file" value={file} onChange={event => { setFile(Number(event.target.value)); setPage(0); }}>{plan.patches.map((patch, index) => <option key={patch.path} value={index}>{patch.path}</option>)}</select>
      <div><strong>Before · lines {selected * 100 + 1}–{Math.min(before.length, (selected + 1) * 100)}</strong><pre>{before.slice(selected * 100, (selected + 1) * 100).join("\n")}</pre><strong>After</strong><pre>{after.slice(selected * 100, (selected + 1) * 100).join("\n")}</pre></div>
      {last > 0 && <div><button disabled={!selected} title={!selected ? "You’re on the first page." : undefined} onClick={() => setPage(selected - 1)}>Previous lines</button><span>{selected + 1}/{last + 1}</span><button disabled={selected === last} title={selected === last ? "You’re on the last page." : undefined} onClick={() => setPage(selected + 1)}>Next lines</button></div>}
      <button onClick={apply}>Apply to source drafts</button></>}
  </div>, document.body);
}
export {requestWorkspaceChange,requestWorkspaceProposal} from "./WorkspaceChangeEvents";
