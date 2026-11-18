<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review complete changes through the existing menu

For maintainers changing workspace services independently of the editor renderer.

The existing Edit menu requests a rename, formatting or provider action. Capture all retained source buffers and the originating selection before asking; an obsolete provider answer or edited target cannot become a preview. The preview pages every before/after line without reserving editor space. Application preflights every target, including retained editors and IME state, before synchronously committing the prepared replacements. It changes drafts only. The same exact inverse model drives bounded workspace undo/redo; later local edits refuse coordinated undo rather than being overwritten. Provider commands remain explicit refusals. Ordered resource operations enter a distinct complete saved-source review, acquire every existing target and re-query provider actions after newly synchronized sources, bind compiler consequences and publish only after explicit save. Matching receipts route existing buffer and tab identities; Source recovery supplies the guarded inverse. No text-only portion of a refused resource action is applied.

Rename discovery acquires previously unopened reference targets through the existing workspace buffer loader. Their language sessions synchronize before a fresh reference query; the discovering answer cannot authorize edits against later-loaded text. Acquisition never activates a tab. Captured source snapshots remain immutable inputs to the final provider proposal, and complete draft-plan validation still precedes application. Native preparation retains its own fixed request-validity closure so an older reply cannot borrow a newer request's authority.

Provider choices resolve through their captured provider before entering the same complete change-plan preview. A latest-choice ticket and the fixed preparation snapshot reject late answers or a replaced choice list. Disabled actions and complete edit-plus-command results refuse explicitly; resolving never applies an edit portion or saves source. The native view passes the rendered preparation identity so an old row index cannot select a newer list item.

Provider edit discovery uses the same bounded acquisition boundary. Newly loaded targets synchronize before the host re-queries actions at the captured selection and resolves a uniquely matching title and kind. Changed or ambiguous choices refuse instead of selecting an arbitrary replacement. Tokens from the discovering response never authorize the final plan. Engine-issued source hashes bind every edited preimage and the readonly contributor context to the original frozen snapshots; the acquisition policy grants write authority only to actual edit targets. Missing, stale or incomplete proofs refuse the complete action. Original draft snapshots remain fixed; only missing sources are added, and acquisition does not activate tabs.

Explicit cleanup enters before language-provider acquisition. Its fixed source, selection, buffer identity and request ticket authorize only that preparation. Returning ordinary source replacements lets the existing preview, writability checks, coordinated apply and inverse history handle it; no second editor or save path is introduced. A no-op reports the observation without creating an undo entry. The cleanup profile is literal and explicit: significant trailing spaces can change, so review the before/after text.

Reviewed capabilities can offer a complete source proposal without a language provider. Every target must have the exact supplied reviewed text, and the ordinary planner and writability checks must accept the whole proposal before it becomes a preview. Capture target buffer identities, text and selections for the approval boundary. Source or selection changes refuse; successful approval uses the same coordinated inverse history as provider changes. The proposal mode has no provider-preparation button and never saves implicitly.

File-wide diagnostic fixes use the explicit `source.fixAll` family and existing whole-action review. Request the complete source range, retain only classified complete actions, and requery that same family after source acquisition. No offered action reports unavailable, and individual quick fixes are never merged. Source revision changes retire the original review even when Undo restores equal bytes.

A pending proposal keeps its original revision lease. A fresh synchronous Undo/Redo instead binds the inverse to its original editor objects and resolves each unique current address immediately before complete preimage validation. This follows the same editor through Save As and guarded namespace moves, while refusing an independent handle that reused the old path. Ordinary Save and late capability metadata do not discard text history: history never writes saved files, and current writability and composition still preflight the whole batch. Reading the inverse and committing it contain no await.

Source cleanup and generation use the standard complete-file `source` action
family. Keep that request distinct from positional refactor/quick-fix choices
and from literal whitespace cleanup. Retain one classified choice through
acquisition and lazy resolution; a changed family or unavailable provider never
falls back to an unrelated quick fix. Existing source proofs and coordinated
history continue to authorize the result.

The browser plan boundary uses exact authored source and UTF-16 edits, mapping only its prepared editor transactions and selection queries into display coordinates. A matching normalized display alone cannot satisfy a revision or provider proof.

<ui:copy id="simple-browser-workspace-changes">
import {authoredSource,authoredOffset,authoredChanges,authoredTransactionSource} from './SourceText';
import { useEffect, useRef, useState } from "react";
import {api,request} from "../api/client";
import {ResourceConsequencesView} from "./SourceRecovery";
import {captureResourceRoster,isResourceEdit,prepareResourceSave,prepareResourceRestore,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute} from "./ResourceEditors";
import {prepareEntryIntent,readFrozenEntrySelection,type FrozenEntrySelection} from "./WorkspaceEntrySelection";
import { createPortal } from "react-dom";
import { isolateHistory } from "@codemirror/commands";
import { type Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorForBuffer } from "./Document";
import {resolveCodeAction} from "../lsp/codeActionResolution";
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
export function scopedProviderActions(actions:CodeAction[],scope:string|null):CodeAction[]{const prefix=scope==="fixAll"?"source.fixAll":scope==="sourceActions"?"source":null;return prefix?actions.filter(action=>action&&typeof action.kind==="string"&&(action.kind===prefix||action.kind.startsWith(prefix+"."))):actions;}
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
</ui:copy>

## Retain provider choices beside their source capture

A menu command starts a fresh request identity. Retain the exact provider response and every target snapshot; source, selection or connection changes make it unavailable. Choosing an action prepares the same pure plan as rename or formatting. Applying records its inverse only after the complete preflight succeeds. Local editor undo remains independent, so coordinated undo validates every target again.

<ui:copy id="simple-browser-workspace-changes-controller">
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
    catch(error){setNotice(String(error)+" Preserve drafts and inspect Source recovery before another publication.");}finally{busy.current=false;setPublishing(false);}
  }
  function apply() {
    if (!plan) return;
    try { if (!captured.current?.current()) throw new Error("Source, selection or provider changed. Prepare the proposal again."); applyWorkspacePlan(buffers, plan); history.record(captureDraftHistory(buffers,invertChangePlan(plan))); setPlan(null); setNotice(`Applied ${plan.patches.length} source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch.`); }
    catch (error) { setNotice(String(error)); }
  }
</ui:copy>

## Page the complete preview without docking it

A fixed floating dialog uses the existing menu, restores editor focus when dismissed, and pages all before/after lines. Pagination bounds rendered content rather than treating excerpts as the entire change. The renderer can choose a provider action or ask the controller to apply; it cannot forge replacements or save files.

<ui:copy id="simple-browser-workspace-changes-presentation">
  if (!mode) return null;
  const patch = plan?.patches[Math.min(file, plan.patches.length - 1)], before = patch?.before.split("\n") ?? [], after = patch?.after.split("\n") ?? [], last = Math.max(0, Math.ceil(Math.max(before.length, after.length) / 100) - 1), selected = Math.min(page, last);
  return createPortal(<div ref={panel} tabIndex={-1} className="simple-workspace-problems" role="dialog" aria-label="Workspace change preview" onKeyDown={event => { if (event.key === "Escape") { event.preventDefault(); close(); } }}>
    <button disabled={publishing} aria-label="Close change preview" onClick={close}>×</button><strong>Workspace change preview</strong>
    {mode!=='entryMove'&&<p>{resource?"Reviewed source resource publication":"Source drafts · not saved or verified"}</p>}
    {mode === "rename" && <input disabled={publishing} ref={input} aria-label="New symbol name" value={name} onChange={event => { ticket.current++; setPlan(null); setChoices([]); setName(event.target.value); setNotice(""); }} />}
    {mode==='entryMove'&&entrySelection&&<>
      <p>Current main construction <code>{entrySelection.entryPath}</code></p>
      <label>New workspace-relative path <input disabled={publishing} value={name} onChange={event=>{ticket.current++;setResource(null);setName(event.target.value);setNotice('');}} /></label>
      <button disabled={publishing||!name||name===entrySelection.entryPath} onClick={()=>void prepare()}>Prepare move</button>
    </>}
    {!["undo", "redo", "proposal", "entryMove"].includes(mode) && <button disabled={publishing||(mode === "rename" && !name.trim())} onClick={() => { void prepare(); }}>Prepare {mode === "fixAll" ? "file-wide diagnostic fixes" : mode === "sourceActions" ? "source actions" : mode === "actions" ? "provider actions" : mode === "format" ? "formatting" : mode === "cleanup" ? "cleanup" : "rename"}</button>}
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

## Verify complete-file fixes through the existing floating preview

These rendered checks start from the actual command event and retain the real CodeMirror editor. Preview never edits or saves; one complete action applies to drafts and enters coordinated Undo/Redo. Equal-byte edit/Undo retires a held list. Wrong action families and a changed resolved kind cannot acquire approval. The owned provider flow separately checks language-bridge routing and unopened target reacquisition.

<ui:copy id="bulk-fix-browser-tests">
import {sourceText,authoredSource} from './SourceText';
import {history,undo,redo} from '@codemirror/commands';
import {act,cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {Buffer,Buffers} from './buffers';
import {WorkspaceChanges,applyWorkspacePlan} from './WorkspaceChanges';
import {WorkspaceChangeHistory} from './ChangePlan';
import {requestWorkspaceChange,requestWorkspaceProposal} from './WorkspaceChangeEvents';
const mocks=vi.hoisted(()=>({editor:vi.fn(),lookup:vi.fn(),outline:vi.fn(),preparation:vi.fn(),request:vi.fn()}));
vi.mock('../api/client',()=>({api:{},request:(...args:unknown[])=>mocks.request(...args)}));
vi.mock('./Document',()=>({editorForBuffer:mocks.editor}));
vi.mock('./WorkspaceLanguages',()=>({useWorkspaceLanguageLookup:()=>mocks.lookup,useWorkspaceOutlineReader:()=>mocks.outline,useWorkspaceSourcePreparation:()=>mocks.preparation}));
let view:EditorView|undefined;
afterEach(()=>{cleanup();view?.destroy();view=undefined;vi.clearAllMocks();});
const replacement={range:{start:{line:0,character:0},end:{line:0,character:3}},newText:'new'};
function action(kind='source.fixAll.provider'){return {title:'Complete file fix',kind,data:{hickory_action:'fixture-held',hickory_sources:[{uri:'hick:///a.py',source_hash:'sha256:01d09d19c2139a46aebfb577780d123d7396e97201bc7ead210a2ebff8239dee'}]},edit:{changes:{'hick:///a.py':[replacement]}}};}
function fixture(result:unknown){
 mocks.request.mockReset();const buffers=new Buffers(()=>{}),file={path:'a.py',name:'a.py',dir:false};const buffer=new Buffer(file,'old\n','base',()=>{});buffers.values.set(file.path,buffer);
 view=new EditorView({state:EditorState.create({doc:buffer.text,selection:{anchor:2},extensions:[EditorView.updateListener.of(update=>{if(update.docChanged)buffer.edit(update.state.doc.toString());})]}),parent:document.body});mocks.editor.mockImplementation(target=>target===buffer?{view}:undefined);
 const request=vi.fn(async(method:string,params?:unknown)=>method==='codeAction/resolve'?params:result),client={request};mocks.lookup.mockReturnValue({uri:'hick:///a.py',client,ready:true});mocks.preparation.mockResolvedValue({client,uri:'hick:///a.py',expected:{source_hash:action().data.hickory_sources[0].source_hash,source_revision:'f'.repeat(64)}});
 const files=[file];render(<WorkspaceChanges buffers={buffers} files={files} current={()=>buffer} focus={()=>{}} adoptResources={()=>{throw new Error('No resource publication expected');}} history={new WorkspaceChangeHistory()}/>);
 act(()=>requestWorkspaceChange('fixAll'));return {buffer,buffers,files,request};
}
async function prepare(){fireEvent.click(screen.getByRole('button',{name:'Prepare file-wide diagnostic fixes'}));}
it('applies source-bound authored offsets and retains exact mixed bytes through native UndoRedo',()=>{
 const before='// first\r\nold\n// last\r\n',after='// first\r\nnew\r\nextra\n// last\r\n',buffers=new Buffers(()=>{}),buffer=new Buffer({path:'mixed.py',name:'mixed.py',dir:false},before,'base',()=>{});buffers.values.set(buffer.file.path,buffer);
 view=new EditorView({state:EditorState.create({doc:before,extensions:[sourceText(before),history(),EditorView.updateListener.of(update=>{if(update.docChanged)buffer.edit(authoredSource(update.state));})]}),parent:document.body});mocks.editor.mockImplementation(target=>target===buffer?{view}:undefined);const held=view,at=before.indexOf('old');
 applyWorkspacePlan(buffers,{title:'Complete authored edit',patches:[{path:buffer.file.path,before,after,edits:[{from:at,to:at+3,text:'new\r\nextra'}]}]});expect(buffer.text).toBe(after);expect(authoredSource(view.state)).toBe(after);expect(undo(view)).toBe(true);expect(buffer.text).toBe(before);expect(redo(view)).toBe(true);expect(buffer.text).toBe(after);expect(view).toBe(held);
});
it('requests the entire source family, reviews one complete action and coordinates draft Undo/Redo',async()=>{
 const {buffer,request}=fixture([action(),{title:'Unrelated quick fix',kind:'quickfix'}, {title:'Wrong prefix',kind:'source.fixAllWrong'}]);await prepare();await screen.findByRole('button',{name:'Complete file fix'});expect(screen.queryByRole('button',{name:'Unrelated quick fix'})).toBeNull();expect(screen.queryByRole('button',{name:'Wrong prefix'})).toBeNull();expect(request).toHaveBeenCalledWith('textDocument/codeAction',expect.objectContaining({range:{start:{line:0,character:0},end:{line:1,character:0}},context:{diagnostics:[],only:['source.fixAll']}}));expect(buffer.text).toBe('old\n');expect(view!.state.selection.main.head).toBe(2);
 fireEvent.click(screen.getByRole('button',{name:'Complete file fix'}));await screen.findByRole('button',{name:'Apply to source drafts'});expect(buffer.text).toBe('old\n');fireEvent.click(screen.getByRole('button',{name:'Apply to source drafts'}));expect(buffer.text).toBe('new\n');act(()=>requestWorkspaceChange('undo'));expect(buffer.text).toBe('old\n');act(()=>requestWorkspaceChange('redo'));expect(buffer.text).toBe('new\n');expect(request.mock.calls.filter(([method])=>method==='textDocument/codeAction')).toHaveLength(2); // Discovery is discarded after retained-source synchronization.
});
it.each(['missing','hash','uncovered'])('refuses provider source proof %s before applying any draft',async kind=>{
 const offered=action();if(kind==='missing')offered.data={} as typeof offered.data;if(kind==='hash')offered.data.hickory_sources[0].source_hash='sha256:'+'0'.repeat(64);
 if(kind==='uncovered')offered.edit={changes:{'hick:///other.py':[replacement]}} as unknown as typeof offered.edit;
 const {buffer,buffers,files}=fixture([offered]);if(kind==='uncovered'){const file={path:'other.py',name:'other.py',dir:false};files.push(file);buffers.values.set(file.path,new Buffer(file,'old\n','base',()=>{}));}
 await prepare();fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));await screen.findByText(kind==='missing'?/Missing or malformed engine source proof/:kind==='hash'?/Provider source proof changed/:/Edited source lacks engine coordinate proof/);expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();expect(buffer.text).toBe('old\n');
});
it('synchronizes readonly contributor context without granting it edit permission',async()=>{
 const offered=action();offered.data.hickory_sources.push({...offered.data.hickory_sources[0],uri:'hick:///z.md'});const {buffers,files}=fixture([offered]),file={path:'z.md',name:'z.md',dir:false,generated_by:'construction.md'};files.push(file);buffers.values.set(file.path,new Buffer(file,'old\n','base',()=>{}));
 await prepare();fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));await screen.findByRole('button',{name:'Apply to source drafts'});expect(buffers.values.get('z.md')?.readonly).toBe(true);expect(buffers.values.get('z.md')?.text).toBe('old\n');
});
it.each(['rename','format'] as const)('reviews the complete bound %s envelope and exact preparation stamp',async mode=>{
 const envelope={...action(),source_files_modified:false},value=fixture([]);value.request.mockImplementation(async(method:string)=>method==='textDocument/references'?[]:method==='hick/'+(mode==='rename'?'rename':'formatting')?envelope:[]);act(()=>requestWorkspaceChange(mode));if(mode==='rename')fireEvent.change(screen.getByRole('textbox',{name:'New symbol name'}),{target:{value:'new'}});fireEvent.click(screen.getByRole('button',{name:mode==='rename'?'Prepare rename':'Prepare formatting'}));await screen.findByRole('button',{name:'Apply to source drafts'});
 const bound=value.request.mock.calls.filter(([method])=>method.startsWith('hick/'));expect(bound).toHaveLength(2);for(const [,params]of bound)expect(params).toMatchObject({textDocument:{uri:'hick:///a.py'},expected:{source_hash:action().data.hickory_sources[0].source_hash,source_revision:'f'.repeat(64)}});expect(value.buffer.text).toBe('old\n');fireEvent.click(screen.getByRole('button',{name:'Apply to source drafts'}));expect(value.buffer.text).toBe('new\n');
});
it('reports unavailable when the provider returns no complete-file family',async()=>{fixture([action('quickfix')]);await prepare();await screen.findByText(/offers no file-wide diagnostic fixes/);expect(screen.queryByRole('button',{name:'Complete file fix'})).toBeNull();expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();});
it('retires an equal-byte edit/Undo before a held bulk list arrives',async()=>{const {buffer,request}=fixture([]);let release!:(value:unknown)=>void;request.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));await prepare();act(()=>{buffer.edit('later');buffer.edit('old\n');});await act(async()=>release([action()]));await screen.findByText(/Source, selection or provider changed/);expect(screen.queryByRole('button',{name:'Complete file fix'})).toBeNull();expect(buffer.text).toBe('old\n');});
it('refuses a resolved action that changes from complete-file fixing to a quick fix',async()=>{const original={...action(),edit:undefined,data:{hickory_action:'held'}};const {buffer,request}=fixture([original]);request.mockImplementation(async(method:string)=>method==='codeAction/resolve'?action('quickfix'):[original]);await prepare();fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));await screen.findByText(/changed the complete file-wide action kind/);expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();expect(buffer.text).toBe('old\n');});
it.each(['undo','guard','baseline','handle','unknown'])('retains original owner authority through a held query: %s',async change=>{
 const {buffer,buffers,request}=fixture([]);const file={path:'other.py',name:'other.py',dir:false},target=new Buffer(file,'old\n','base',()=>{});buffers.values.set(file.path,target);
 // Reopen the command before preparing so this target belongs to the original roster.
 act(()=>requestWorkspaceChange('fixAll'));request.mockImplementationOnce(async()=>{if(change==='undo'){target.edit('later');target.edit('old\n');}else if(change==='guard')target.saveGuard='foreign';else if(change==='baseline')target.base='foreign';else if(change==='handle')buffers.values.set(file.path,new Buffer(file,target.text,target.hash,()=>{}));else buffers.values.set('unknown.md',new Buffer({path:'unknown.md',name:'unknown.md',dir:false},'unmodeled construction','new',()=>{}));return [action()];});await prepare();await screen.findByText(/Source, selection or provider changed/);expect(screen.queryByRole('button',{name:'Complete file fix'})).toBeNull();expect(buffer.text).toBe('old\n');
});
it('refuses a newly loaded target edited and undone before acquisition returns',async()=>{
 const {buffer,buffers,files,request}=fixture([]),file={path:'other.py',name:'other.py',dir:false};files.push(file);const original={...action(),edit:undefined,data:{hickory_action:'held'}},discovery={title:original.title,kind:original.kind,data:{...action().data,hickory_acquire:[{uri:'hick:///other.py'}]}};
 request.mockImplementation(async(method:string)=>method==='codeAction/resolve'?discovery:[original]);vi.spyOn(buffers,'open').mockImplementation(async()=>{const target=new Buffer(file,'old\n','base',()=>{});buffers.values.set(file.path,target);target.edit('later');target.edit('old\n');return target;});await prepare();fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));await screen.findByText(/newly acquired source was edited/);expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();expect(buffer.text).toBe('old\n');
});
it('keeps reviewed drafts bound to the original equal-byte revision through approval',async()=>{const {buffer}=fixture([]);act(()=>requestWorkspaceProposal('Reviewed',action().edit,new Map([['a.py',buffer.text]])));await screen.findByRole('button',{name:'Apply to source drafts'});buffer.edit('later');buffer.edit('old\n');fireEvent.click(screen.getByRole('button',{name:'Apply to source drafts'}));await screen.findByText(/Source, selection or provider changed/);expect(buffer.text).toBe('old\n');});
it('does not renew cleanup authority after an edit and equal-byte Undo',async()=>{const {buffer}=fixture([]);mocks.request.mockImplementationOnce(async()=>{buffer.edit('later');buffer.edit('old\n');return action().edit;});act(()=>requestWorkspaceChange('cleanup'));fireEvent.click(screen.getByRole('button',{name:'Prepare cleanup'}));await screen.findByText(/Source or selection changed/);expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();expect(buffer.text).toBe('old\n');});
it('follows the original editor through Save metadata and a literal utility-like source name',async()=>{const {buffer,buffers}=fixture([action()]);await prepare();fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));fireEvent.click(await screen.findByRole('button',{name:'Apply to source drafts'}));expect(buffer.text).toBe('new\n');buffer.base=buffer.text;buffer.hash='saved';buffer.saveGuard='confirmed-save';buffer.sourceRevision++;buffers.values.delete('a.py');buffer.file={path:'files',name:'files',dir:false};buffers.values.set('files',buffer);const replacement=new Buffer({path:'a.py',name:'a.py',dir:false},'new\n','base',()=>{});buffers.values.set('a.py',replacement);act(()=>requestWorkspaceChange('undo'));expect(buffer.text).toBe('old\n');expect(buffer.base).toBe('new\n');expect(replacement.text).toBe('new\n');act(()=>requestWorkspaceChange('redo'));expect(buffer.text).toBe('new\n');
 buffers.values.set('alias.py',buffer);act(()=>requestWorkspaceChange('undo'));await screen.findByText(/history no longer owns the original source editors/);expect(buffer.text).toBe('new\n');buffers.values.delete('alias.py');const foreign=new Buffer(buffer.file,buffer.text,buffer.hash,()=>{});buffers.values.set('files',foreign);act(()=>requestWorkspaceChange('undo'));expect(foreign.text).toBe('new\n');expect(buffer.text).toBe('new\n');buffers.values.set('files',buffer);act(()=>requestWorkspaceChange('undo'));expect(buffer.text).toBe('old\n');expect(foreign.text).toBe('new\n');expect(replacement.text).toBe('new\n');});

it('reviews complete source cleanup without admitting positional or malformed families',async()=>{
 const {buffer,request}=fixture([action('source.organizeImports'),{title:'Single fix',kind:'quickfix'},{title:'Wrong family',kind:'sourceWrong'}]);act(()=>requestWorkspaceChange('sourceActions'));fireEvent.click(screen.getByRole('button',{name:'Prepare source actions'}));await screen.findByRole('button',{name:'Complete file fix'});expect(screen.queryByRole('button',{name:'Single fix'})).toBeNull();expect(screen.queryByRole('button',{name:'Wrong family'})).toBeNull();expect(request).toHaveBeenCalledWith('textDocument/codeAction',expect.objectContaining({range:{start:{line:0,character:0},end:{line:1,character:0}},context:{diagnostics:[],only:['source']}}));fireEvent.click(screen.getByRole('button',{name:'Complete file fix'}));await screen.findByRole('button',{name:'Apply to source drafts'});fireEvent.click(screen.getByRole('button',{name:'Apply to source drafts'}));expect(buffer.text).toBe('new\n');act(()=>requestWorkspaceChange('undo'));expect(buffer.text).toBe('old\n');
});
it('refuses a source cleanup whose resolved action becomes a positional fix',async()=>{
 const {buffer,request}=fixture([action('source.organizeImports')]);request.mockImplementation(async(method:string,params?:unknown)=>method==='codeAction/resolve'?{...(params as object),kind:'quickfix'}:[action('source.organizeImports')]);act(()=>requestWorkspaceChange('sourceActions'));fireEvent.click(screen.getByRole('button',{name:'Prepare source actions'}));fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));await screen.findByText(/changed the complete file-wide action kind/);expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();expect(buffer.text).toBe('old\n');
});
it('reports source cleanup unavailable without reclassifying ordinary fixes',async()=>{
 const {buffer}=fixture([{title:'Single fix',kind:'quickfix'}]);act(()=>requestWorkspaceChange('sourceActions'));fireEvent.click(screen.getByRole('button',{name:'Prepare source actions'}));await screen.findByText('No source cleanup or generation actions are available here. No edits were applied.');expect(screen.queryByRole('button',{name:'Single fix'})).toBeNull();expect(buffer.text).toBe('old\n');
});
</ui:copy>
<ui:file path="apps/web/src/simple/WorkspaceChanges.test.tsx"><ui:paste select="#bulk-fix-browser-tests"/></ui:file>
</ui:doc>
