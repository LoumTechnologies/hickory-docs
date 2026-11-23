<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep buffer revisions independent of pane arrangement

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Each buffer owns its text and save baseline. Subscribers to text changes are separate from the workspace shell so typing does not rerender every pane. One in-flight read per path prevents late duplicate opens from replacing edits.

Managed documents use their document ID to read the current source directly, without rendering or analysis. The plain-file endpoint deliberately refuses them. Their save baseline is that source text; the hash is unused for document saves.

Closing a clean saved plain tab keeps its editor in the cache and releases
its language authority. If a workspace edit reaches that buffer while it is
closed, the draft again owns its language source. Reopening the unchanged
saved bytes can reuse the editor. A different saved revision needs a fresh
clean buffer. The closed cache stays available if the read fails or loses
its authority.

Startup can put an independent text observation in the normal buffer map. Its
read-only state survives missing or stale ownership. Once the authoritative
tree and guarded source identify the same path and workspace, update the
same buffer to the latest saved bytes and promote it when authority is
available. A guarded read in flight cannot recreate a closed observation.

A disk refresh checks the same buffer and original baseline after the read. A
changed clean source updates the model in place. A divergent draft keeps the
original guard so an ordinary Save cannot overwrite unseen disk changes.

An ordinary Save uses a guard captured with the original saved bytes, plus a
retained request identity. It never refreshes that guard from a later history
read or silently falls back to a legacy route that ignores identity fields.
Only a fully echoed confirmed receipt advances the saved baseline; unsaved text
can continue changing independently while the submitted snapshot is saved.

<ui:copy id="simple-964718fb75-0">
import { api,request } from "../api/client";
import {historySourceHash} from "./SourceHistory";
import type { FileNode } from "../api/types";
import type {GitComparison} from "./GitHistory";
export class Buffer {
  gitComparison?: GitComparison;
  setGitComparison(value: GitComparison | undefined) { this.gitComparison=value; this.publish(); }
  text: string;
  base: string;
  hash: string;
  saveGuard?:string;
  saveGuardReason?:string;
  private saveAttempt?:{id:string;text:string;base:string;hash:string;guard:string;file:FileNode;namespace:string};
  saving = false;
  error = "";
  completionNotice="";
  setCompletionNotice(value:string){if(this.completionNotice===value)return;this.completionNotice=value;this.publish();this.shellChanged();}
  inlineRenderNotice = "";
  setInlineRenderNotice(value: string) {
    if (this.inlineRenderNotice === value) return;
    this.inlineRenderNotice = value;
    this.shellChanged();
  }
  caretOffset = 0;
  version = 0;
  sourceRevision=0;
  private listeners = new Set<() => void>();
  resourceDeleted = false;
  isUntitledDraft = false;
  startupWorkspace?: string | null;
  closedSavedPlainSource = false;
  // Recovery creates new drafts; these observations never grant a save guard.
  recoveryView?:{selection:{anchor:number;head:number};scroll_x:number;scroll_y:number};
  recoveryBaseline?:{text:string;save_guard:string|null}|null;
  private identity?:string;
  get sourceId(){return this.identity??=crypto.randomUUID().replaceAll('-','');}
  constructor(public file: FileNode, content: string, hash: string, private shellChanged: () => void, readonly transient = false,public namespace = "") {
    this.text = this.base = content; this.hash = hash;
  }
  get dirty() { return this.text !== this.base; }
  get readonly() { return this.startupWorkspace !== undefined || !!this.file.generated_by; }
  get cleanSavedPlainSource() {
    return !this.readonly && !this.transient && !this.file.doc_id && !this.resourceDeleted
      && !this.isUntitledDraft && !this.dirty && !this.saving;
  }
  get languageSuspended() { return this.closedSavedPlainSource && this.cleanSavedPlainSource; }
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  edit(text: string) {
    if (this.readonly) return;
    const dirty = this.dirty;if(this.text!==text)this.sourceRevision++; this.text = text; this.publish();
    if (dirty !== this.dirty) this.shellChanged();
  }
</ui:copy>

Capture text and revision once when Save is pressed. Never fetch a new baseline before writing, force an overwrite, or mark later keystrokes as saved. Literate writes use the existing room-and-disk check.

<ui:copy id="simple-964718fb75-1">
  async save() {
    if (this.resourceDeleted || this.transient || this.readonly || this.saving || !this.dirty) return;
    const text = this.text, base = this.base, hash = this.hash,guard=this.saveGuard,file=this.file,namespace=this.namespace;
    this.saving = true; this.error = ""; this.shellChanged(); this.publish();
    try {
      if(!guard)throw Error(this.saveGuardReason||"The loaded saved baseline has no source identity guard. Preserve your draft and reload the saved source in a separate editor before reconciling it.");
      const prior=this.saveAttempt,attempt=prior&&prior.text===text&&prior.base===base&&prior.hash===hash&&prior.guard===guard&&prior.file===file&&prior.namespace===namespace?prior:{id:crypto.randomUUID().replaceAll('-',''),text,base,hash,guard,file,namespace};
      this.saveAttempt=attempt;
      const sourceHash=await historySourceHash(text);
      const current=()=>this.file===file&&this.namespace===namespace&&!this.resourceDeleted&&this.base===base&&this.hash===hash&&this.saveGuard===guard;
      if(!current())throw Error('Source address or saved baseline changed before save admission.');
      const body={version:1,request_id:attempt.id,save_guard:guard,...(file.doc_id?{source:text,expected:{source:base,disk:base}}:{path:file.path,content:text,base_hash:hash})};
      const value=await request<{version:number;history_protocol:number;request_id:string;path:string;source_hash:string;save_guard:string;hash?:string;history:{available:boolean;outcome:string;reason?:string}}>('PUT',file.doc_id?'/api/source-history/docs/'+encodeURIComponent(file.doc_id):'/api/source-history/file',body);
      if(!current()||value.version!==1||value.history_protocol!==1||value.request_id!==attempt.id||value.path!==file.path||value.source_hash!==sourceHash||typeof value.save_guard!=='string'||!value.save_guard||typeof value.history?.available!=='boolean'||value.history.outcome!=='published'||!file.doc_id&&typeof value.hash!=='string')throw Error('Save outcome or source identity was not confirmed. Inspect retained history before another save.');
      if(!file.doc_id)this.hash=value.hash!;
      this.saveGuard=value.save_guard;this.base=text;this.sourceRevision++;this.saveAttempt=undefined;
      if(!value.history.available)this.error='Saved; local history is unavailable: '+(value.history.reason??'This source has no authorized history store.');
    } catch (error) {
      this.error = `${String(error)} Your edits are retained. Copy them before closing; compare with the file on disk before retrying.`;
    } finally { this.saving = false; this.publish(); this.shellChanged(); }
  }
}
export class Buffers {
  workspaceId="";
  private namespace="";
  get workspace(){return this.namespace;}
  set workspace(value:string){const initial=!this.namespace;this.namespace=value;if(initial&&value)for(const buffer of this.values.values())if(!buffer.namespace){buffer.namespace=value;buffer.sourceRevision++;buffer.publish();}}

  values = new Map<string, Buffer>();
  retainedResources = new Map<string, Buffer>();
  private namespaceRevision = 0;
  retirePendingReads() { this.namespaceRevision++; this.pending.clear(); }
  get hasUnsavedChanges() { return [...this.values.values(),...this.retainedResources.values()].some(buffer=>buffer.dirty||buffer.saving); }
  private pending = new Map<string, Promise<Buffer>>();
  constructor(public changed: () => void) {}
  closeSavedPlainSource(buffer: Buffer) {
    if (!buffer.cleanSavedPlainSource || buffer.closedSavedPlainSource || this.values.get(buffer.file.path) !== buffer) return;
    buffer.closedSavedPlainSource = true;
    buffer.sourceRevision++;
    buffer.publish();
  }
  observeStartup(source: {version:1;path:string;content:string;root_path:string;workspace_id:string|null}): Buffer | undefined {
    if (this.values.has(source.path) || this.workspace && this.workspace !== source.root_path
      || this.workspaceId && source.workspace_id && this.workspaceId !== source.workspace_id) return;
    const buffer = new Buffer({path:source.path, name:source.path.split("/").at(-1)!, dir:false}, source.content, "", this.changed, false, source.root_path);
    buffer.startupWorkspace = source.workspace_id;
    this.values.set(source.path, buffer); this.changed();
    return buffer;
  }
  private sourceCheck = 0;
  async refreshSavedSource(buffer: Buffer, currentEditor: () => boolean): Promise<string | undefined> {
    const file = buffer.file, path = file.path;
    if (!this.workspace || !this.workspaceId || buffer.transient || buffer.isUntitledDraft ||
        buffer.resourceDeleted || buffer.gitComparison || buffer.startupWorkspace !== undefined ||
        buffer.saving || this.values.get(path) !== buffer) return;
    const ticket = ++this.sourceCheck, epoch = this.namespaceRevision;
    const namespace = this.workspace, workspace = this.workspaceId;
    const {base, saveGuard, sourceRevision} = buffer;
    const current = () => ticket === this.sourceCheck && epoch === this.namespaceRevision &&
      namespace === this.workspace && workspace === this.workspaceId && this.values.get(path) === buffer &&
      buffer.file === file && base === buffer.base && saveGuard === buffer.saveGuard &&
      sourceRevision === buffer.sourceRevision && !buffer.saving && !buffer.resourceDeleted && currentEditor();
    try {
      const saved = await this.readSavedSource(file);
      if (!current() || saved.path !== path) return;
      if (buffer.dirty) {
        if (saved.content !== base || saved.save_guard !== saveGuard) return `${path} changed on disk. Your unsaved draft is still open.`;
        return;
      }
      const changed = saved.content !== base;
      if (!changed && buffer.hash === saved.hash && saveGuard === saved.save_guard && buffer.saveGuardReason === saved.save_guard_reason) return;
      buffer.text = buffer.base = saved.content;
      buffer.hash = saved.hash;
      buffer.saveGuard = saved.save_guard;
      buffer.saveGuardReason = saved.save_guard_reason;
      buffer.sourceRevision++;
      buffer.publish(); this.changed();
      if (changed) return `Reloaded ${path} from disk.`;
    } catch (error) {
      if (current()) return `Could not check ${path} on disk. ${String(error)}`;
    }
  }
  private readSavedSource(file: FileNode) {
    return file.doc_id
      ? api.doc(file.doc_id).then(doc => ({ path: doc.path, content: doc.source, hash: "",save_guard:doc.save_guard,save_guard_reason:doc.save_guard_reason }))
      : api.file(file.path);
  }
  open(file: FileNode, forStartup = false): Promise<Buffer> {
    const path = file.path;
    const current = this.values.get(path);
    if (current && current.startupWorkspace === undefined && !current.languageSuspended) {
      if (current.closedSavedPlainSource) {
        current.closedSavedPlainSource = false;
        current.sourceRevision++;
        current.publish();
      }
      return Promise.resolve(current);
    }
    const pending = this.pending.get(path);
    if (pending) return pending;
    const revision = this.namespaceRevision, workspace = this.workspaceId, namespace = this.workspace;
    const closed = current?.languageSuspended ? { buffer: current, file: current.file, revision: current.sourceRevision } : null;
    const read = this.readSavedSource(file);
    const request = read.then(data => {
      if (revision !== this.namespaceRevision) throw new Error("Source addresses changed while opening. Choose the file again.");
      if (forStartup && (this.workspace !== namespace || this.workspaceId !== workspace || data.path !== file.path)) {
        throw Error("The first file reply belongs to a different workspace or source. Refresh Files to read the current source.");
      }
      const available = this.values.get(path);
      if (closed) {
        const { buffer } = closed;
        if (this.workspace !== namespace || this.workspaceId !== workspace || buffer.namespace !== namespace
          || file.path !== path || data.path !== path || available !== buffer || buffer.file !== closed.file
          || buffer.file.path !== path || !buffer.languageSuspended || buffer.sourceRevision !== closed.revision) {
          throw Error("The cached source changed while confirming the saved file. Choose the file again.");
        }
        if (!file.doc_id && !file.generated_by && data.content === buffer.text) {
          buffer.file = file;
          buffer.hash = data.hash;
          buffer.saveGuard = data.save_guard;
          buffer.saveGuardReason = data.save_guard_reason;
          buffer.closedSavedPlainSource = false;
          buffer.sourceRevision++;
          buffer.publish(); this.changed();
          return buffer;
        }
      }
      if (current?.startupWorkspace !== undefined && available !== current) {
        throw Error("The startup editor closed while confirming ownership. Open the file again.");
      }
      if (available?.startupWorkspace !== undefined) {
        if (!namespace || !/^[0-9a-f]{32}$/.test(workspace) || this.workspace !== namespace || this.workspaceId !== workspace
          || available.namespace !== namespace || available.startupWorkspace && available.startupWorkspace !== workspace
          || data.path !== file.path || available.dirty) {
          throw Error("The startup observation differs from the current workspace source. Close this read-only tab and open the file again.");
        }
        if (available.text !== data.content) {
          available.text = available.base = data.content;
          available.sourceRevision++;
          available.publish();
        }
        if (!file.generated_by && !data.save_guard) {
          throw Error(data.save_guard_reason || "Source authority is still unavailable. Refresh files and open this file again.");
        }
        available.file = file;
        available.hash = data.hash;
        available.saveGuard = data.save_guard;
        available.saveGuardReason = data.save_guard_reason;
        available.sourceRevision++;
        available.startupWorkspace = undefined;
        available.publish(); this.changed();
        return available;
      }
      if (available && !closed) return available;
      const buffer = new Buffer(file, data.content, data.hash, this.changed, false, this.workspace);
      if (forStartup && !file.generated_by && (!namespace || !/^[0-9a-f]{32}$/.test(workspace) || !data.save_guard)) {
        // A guarded endpoint can return readable bytes even when save
        // permission is absent. Let a later exact read confirm this editor.
        buffer.startupWorkspace = /^[0-9a-f]{32}$/.test(workspace) ? workspace : null;
        buffer.saveGuardReason = data.save_guard_reason || "Source authority is still unavailable.";
      } else {
        buffer.saveGuard = data.save_guard;
        buffer.saveGuardReason = data.save_guard_reason;
      }
      this.values.set(path, buffer); this.changed(); return buffer;
    }).finally(() => { if(this.pending.get(path)===request)this.pending.delete(path); });
    this.pending.set(path, request); return request;
  }
}
/** A copy has its own saved identity; it never authorizes replacing the original editor. */
export interface CreatedSource {version:number;history_protocol:number;workspace:string;namespace:string;path:string;source:string;source_hash:string;hash:string;request_id:string;save_guard:string;doc_id?:string|null;history:{available:boolean;outcome:string;id:string}}
type CreateOwner={buffer:Buffer;file:FileNode;path:string;text:string;base:string;hash:string;guard:string|undefined;revision:number;deleted:boolean;readonly:boolean;namespace:string;docId:string|undefined;saving:boolean};
type CreateAttempt={path:string;source:Buffer;text:string;namespace:string;workspace:string;owners:CreateOwner[];values:[string,Buffer][];retained:[string,Buffer][];body:unknown;id:string;hostCurrent:()=>boolean};
const createAttempts=new WeakMap<Buffer,CreateAttempt>();
const createDraftIds=new WeakMap<Buffer,string>();
function createDraftId(buffer:Buffer){let id=createDraftIds.get(buffer);if(!id){id=crypto.randomUUID().replaceAll('-','');createDraftIds.set(buffer,id);}return id;}
function createdContentHash(text:string){let value=0xcbf29ce484222325n;for(const byte of new TextEncoder().encode(text))value=BigInt.asUintN(64,(value^BigInt(byte))*0x100000001b3n);return value.toString(16).padStart(16,'0');}
function createRoster(buffers:Buffers){return [...new Set([...buffers.values.values(),...buffers.retainedResources.values()])];}
function createCurrent(buffers:Buffers,a:CreateAttempt):boolean {
 const equal=(map:Map<string,Buffer>,entries:[string,Buffer][])=>map.size===entries.length&&entries.every(([key,value])=>map.get(key)===value);
 return a.hostCurrent()&&buffers.workspace===a.namespace&&buffers.workspaceId===a.workspace&&equal(buffers.values,a.values)&&equal(buffers.retainedResources,a.retained)&&a.owners.every(o=>o.buffer.file===o.file&&o.file.path===o.path&&o.buffer.text===o.text&&o.buffer.base===o.base&&o.buffer.hash===o.hash&&o.buffer.saveGuard===o.guard&&o.buffer.sourceRevision===o.revision&&o.buffer.resourceDeleted===o.deleted&&o.buffer.readonly===o.readonly&&o.buffer.namespace===o.namespace&&o.file.doc_id===o.docId&&o.buffer.saving===o.saving);
}
// A confirmed recovery copy changes the address of the original editor, preserving its history identity.
export function adoptCreatedDraft(buffers:Buffers,source:Buffer,copy:Buffer,current:()=>boolean):void {
 if(!current()||source===copy||source.readonly||source.transient||copy.readonly||copy.transient||(!!source.namespace&&source.namespace!==buffers.workspace)||!(source.resourceDeleted||source.isUntitledDraft)||source.text!==copy.text||copy.text!==copy.base||!copy.saveGuard||copy.namespace!==buffers.workspace||buffers.values.get(copy.file.path)!==copy||[...buffers.values].filter(([,owner])=>owner===source).length>1||![...buffers.values.values(),...buffers.retainedResources.values()].includes(source))throw Error('Confirmed copy cannot adopt this original draft editor. Both texts are retained.');
 for(const [path,owner] of buffers.values)if(owner===source)buffers.values.delete(path);
 for(const [id,owner] of buffers.retainedResources)if(owner===source)buffers.retainedResources.delete(id);
 source.file=copy.file;source.base=copy.base;source.hash=copy.hash;source.saveGuard=copy.saveGuard;source.saveGuardReason=undefined;source.namespace=copy.namespace;source.resourceDeleted=false;source.isUntitledDraft=false;source.sourceRevision++;source.error='';
 buffers.retirePendingReads();buffers.values.set(source.file.path,source);source.publish();buffers.changed();
}
export async function createSourceCopy(buffers:Buffers,source:Buffer,path:string,send:typeof request=request,hostCurrent:()=>boolean=()=>true):Promise<Buffer>{
 let attempt=createAttempts.get(source);
 if(attempt&&(attempt.path!==path||!createCurrent(buffers,attempt)))throw Error('A previous copy outcome is uncertain and its editor context changed. Preserve the draft and inspect local history before another publication.');
 if(!attempt){
  if(!buffers.workspace||!/^[0-9a-f]{32}$/.test(buffers.workspaceId)||source.readonly||source.transient||!createRoster(buffers).includes(source))throw Error('The source or workspace has no current writable creation authority.');
  const owners=createRoster(buffers).map(buffer=>({buffer,file:buffer.file,path:buffer.file.path,text:buffer.text,base:buffer.base,hash:buffer.hash,guard:buffer.saveGuard,revision:buffer.sourceRevision,deleted:buffer.resourceDeleted,readonly:buffer.readonly,namespace:buffer.namespace,docId:buffer.file.doc_id,saving:buffer.saving}));
  const ownership_sources:Record<string,string>={},saved_guards:Record<string,string>={},unaddressed_sources:Record<string,string>={};
  for(const o of owners){if(o.buffer!==source&&o.saving)throw Error('Wait for retained source saves before preparing Save As.');if(o.buffer.transient||o.readonly)continue;if(o.deleted||o.buffer.isUntitledDraft){if(o.buffer!==source)unaddressed_sources[createDraftId(o.buffer)]=o.text;continue;}if(o.namespace!==buffers.workspace||!o.guard||Object.hasOwn(ownership_sources,o.path))throw Error('Every retained authored source needs an unambiguous guarded saved baseline before Save As.');ownership_sources[o.path]=o.text;saved_guards[o.path]=o.guard;}
  const id=crypto.randomUUID().replaceAll('-','');
  attempt={path,source,text:source.text,namespace:buffers.workspace,workspace:buffers.workspaceId,owners,values:[...buffers.values],retained:[...buffers.retainedResources],id,hostCurrent,body:{version:1,workspace:buffers.workspaceId,path,source:source.text,request_id:id,ownership:{context:id,edit:{changes:{}},drafts:{},project_entry:null,ownership_sources,saved_guards,unaddressed_sources},saved_guards,unaddressed_sources}};
  createAttempts.set(source,attempt);
 }
 const a=attempt,hash=await historySourceHash(a.text);
 if(!createCurrent(buffers,a))throw Error('Editors changed before copy admission.');
 const value=await send<CreatedSource>('POST','/api/source-history/create',a.body);
 if(!createCurrent(buffers,a)||value.version!==1||value.history_protocol!==1||value.workspace!==a.workspace||value.namespace!==a.namespace||value.path!==path||value.source!==a.text||value.source_hash!==hash||value.request_id!==a.id||value.hash!==createdContentHash(a.text)||typeof value.save_guard!=='string'||!value.save_guard||value.doc_id!=null&&typeof value.doc_id!=='string'||value.history?.available!==true||value.history.outcome!=='published'||!/^[0-9a-f]{32}$/.test(value.history.id)||buffers.values.has(path))throw Error('Copy publication or editor adoption was not confirmed. Preserve the draft and inspect the retained request '+a.id+'.');
 const copy=new Buffer({name:path.split('/').at(-1)!,path,dir:false,...(value.doc_id?{doc_id:value.doc_id}:{})},a.text,value.hash,buffers.changed,false,a.namespace);copy.saveGuard=value.save_guard;
 buffers.retirePendingReads();buffers.values.set(path,copy);createAttempts.delete(source);buffers.changed();return copy;
}
</ui:copy>

A Save As copy captures every retained editor and its saved identity before sending
one no-clobber request. Lost replies retain that exact request; changed rosters
retire adoption. Only the direct confirmed receipt constructs the new buffer,
leaving the original editor and undo history intact. No empty file or later read
can stand in for a confirmed publication.

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/buffers.ts"><ui:paste select="#simple-964718fb75-0"/><ui:paste select="#simple-964718fb75-1"/></ui:file>

## Confirm copies without borrowing later disk bytes

The maintained browser witness passed strict Save As through the File menu for
a literal Unicode path against the real engine. Thirteen focused copy and
File-menu cases cover exact consumed-request replay, complete guarded authored
rosters, other unmaterialized notes, changed editor/namespace/roster admission,
held caller retirement and mismatched publication receipts. A copy retains the
original buffer and its undo history; an uncertain publication keeps the exact
request and requires inspection instead of silently choosing a fresh request.
This evidence does not establish physical keyboard or native acceptance.

</ui:doc>
