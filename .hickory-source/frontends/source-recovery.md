<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review resource saves and recover source without replacing the editor

For maintainers adding local recovery through the existing menu on browser, macOS, iOS and Android.

Discovery names retained resource operations, including interrupted ones. It does not label receipts as current disk state or grant restoration authority. Open a floating reading from the shared View menu and leave the editor mounted underneath it. Existing saved baselines and unsaved drafts can differ; request one explicit snapshot and preserve empty text separately from absent source. All hosts now prepare a source-current inverse and preview saved baselines separately from drafts. Explicit publication uses the shared engine guard, then admits only a complete matching receipt into live editors. Its inverse is another reviewed recovery operation, not an automatic retry. Provider resource proposals share this review, adoption and ordered workspace history. Ordinary-save history and general crash/session recovery remain separate work.

## Focused evidence, 2026-11-15

Connected publication acceptance: the actual-provider browser flow passed ordered
text/create/text/move/text/delete, unopened-source acquisition/requery, complete
review, explicit saved publication and guarded workspace undo/redo. Its independent
actual-engine recovery flow passed stale-draft and post-review refusal, separate
saved/draft restoration, inverse preparation and retained editor identity/geometry.
TypeScript and 11 focused resource/history/reader cases passed. Apple passed ten
selected recovery/controller tests, including the actual-engine save/restore/redo
path and later-edit refusal, plus five tab/adoption tests and the iOS simulator
build. Android compiled the complete app and passed fourteen selected resource,
history and proposal tests, including actual-engine save/undo/redo and one lost
reply reconciled by same-request status with exactly one publication POST.
Native controller checks preserve buffer, caret, local undo, pane and directory
state; they do not establish physical renderer, IME or accessibility acceptance.
Canonical-entry moves, invalid historical construction drafts, partial-state
restoration and native canonical-equivalent duplicate wire paths still refuse.
Source recovery reads remain available when restoration cannot be admitted.

Earlier read-only acceptance:

The real-engine browser witness passes menu discovery, exact saved/draft/absent snapshot reads and preservation of the live editor identity, geometry, newer unsaved text and undo/redo. Two focused model cases pass on each of browser, Apple and Android for retired asynchronous replies, changed evidence/workspace refusal and distinct empty saved/dirty text. TypeScript checking and the iOS simulator build pass. Two shared-store discovery cases cover exact sorted pagination, read-only absence and symlink refusal. The actual resource service witness passes discovery and separate saved/draft text inspection alongside its guarded save/restore cases. Native physical input and accessibility acceptance remain unverified; ordinary-save history, crash/session restoration and frontend resource restoration are still required.

## browser 1

The reader binds selection to a workspace and exact record fingerprint. Each read retires older requests; refresh and close invalidate pending replies. A later snapshot of changed evidence refuses, so two sides cannot silently come from different records.

<ui:copy id="source-recovery-browser-0">
import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import {prepareResourceRestore,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute} from './ResourceEditors';
export interface RecoveryPage {version:number;workspace:string;ids:string[];next:string|null;ignored:number}
export interface RecoverySummary {kind:'file'|'missing';bytes?:number;sha256?:string}
export interface RecoveryReading {version:number;workspace:string;id:string;fingerprint:string;title:string;title_truncated:boolean;paths:{path:string;saved_before:RecoverySummary;saved_after:RecoverySummary;draft_before:RecoverySummary;draft_after:RecoverySummary}[];all_paths_recorded_applied:boolean;unrecognized_files:string[];draft_text:string|null;saved_text:string|null}
export type RecoverySnapshot='saved_before'|'saved_after'|'draft_before'|'draft_after';
export class RecoveryReader {
 page:RecoveryPage|null=null;value:RecoveryReading|null=null;notice='';loading=false;private ticket=0;
 constructor(private load:<T>(route:string)=>Promise<T>,private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.value=null;this.page=null;}
 async list(after:string|null=null){const id=++this.ticket;this.loading=true;this.value=null;this.notice='Reading retained recovery identities…';this.changed();
 try{const page=await this.load<RecoveryPage>('/api/changes/resource-recoveries'+(after?'?after='+encodeURIComponent(after):''));if(id!==this.ticket)return;
 if(page.version!==1||!page.workspace||page.ids.length>100||page.ids.some((value,index)=>!/^([0-9a-f]{32})$/.test(value)||(index>0&&page.ids[index-1]>=value)||(after!==null&&value<=after))||(page.next!==null&&page.next!==page.ids.at(-1))||(after!==null&&page.workspace!==this.page?.workspace))throw new Error('Recovery page identity changed. Refresh from the beginning.');
 this.page=page;this.notice=page.ids.length?'Choose a retained record. Identities are sorted, not chronological.':'No retained resource recovery records.';
 }catch(error){if(id===this.ticket){this.page=null;this.notice=String(error);}}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
 async read(record:string,path?:string,snapshot:RecoverySnapshot='draft_before'){
 const workspace=this.page?.workspace,previous=this.value?.id===record?this.value:null,id=++this.ticket;this.loading=true;this.value=null;this.notice='Reading retained source evidence…';this.changed();
 try{if(!workspace||!this.page?.ids.includes(record))throw new Error('Refresh recovery identities before selecting a record.');const [kind,side]=snapshot.split('_');
 const value=await this.load<RecoveryReading>('/api/changes/resource-recovery?id='+encodeURIComponent(record)+(path!==undefined?'&path='+encodeURIComponent(path)+'&'+kind+'='+side:''));if(id!==this.ticket)return;
 if(value.version!==1||value.workspace!==workspace||value.id!==record||!/^([0-9a-f]{64})$/.test(value.fingerprint)||(previous&&previous.fingerprint!==value.fingerprint)||(path!==undefined&&!value.paths.some(item=>item.path===path)))throw new Error('Recovery evidence changed. Refresh identities and read again.');
 this.value=value;this.notice='Retained evidence only. Current source and editor drafts are unchanged.';
 }catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
</ui:copy>

## browser 2

Render the selected retained evidence in a floating reading, reached through existing menu commands. Keep refresh, paging and snapshot controls inside this reading, with selectable abbreviated text and explicit absence. Closing returns focus to the existing source editor. Android's workspace-owned review captures every retained authored buffer and supplies Markdown drafts as ownership inputs, so opening or editing another construction retires the review. Preparation reads current authority; its separate Save action repeats server guards, and uncertain publication admits only the original request's read-only status. A complete receipt adopts saved baselines independently of dirty draft text. Review one source at a time to bound rendering cost; generated consequence paths describe constructions, never execution or materialization.

<ui:copy id="source-recovery-browser-1">
export function ResourceConsequencesView({review}:{review:ResourceRestoreReview}){
 const [page,setPage]=useState(0),[layer,setLayer]=useState<'saved'|'draft'>('saved');const value=review.facts[layer],rows=[...value.entries.map(path=>'Construction entry: '+path),...value.removed_entries.map(path=>'Removed entry: '+path),...value.output_paths.map(path=>'Generated output: '+path)],last=Math.max(0,Math.ceil(rows.length/100)-1),selected=Math.min(page,last);
 return <section aria-label="Construction consequences"><label>Construction layer <select aria-label="Construction layer" value={layer} onChange={event=>{setLayer(event.target.value as 'saved'|'draft');setPage(0);}}><option value="saved">Saved construction</option><option value="draft">Draft construction</option></select></label><p>{value.output_paths.length} generated output paths · relationships observed by compiler analysis; no checks or coverage queried.</p><pre>{rows.slice(selected*100,(selected+1)*100).join('\n')||'No construction entries or outputs reported.'}</pre>{last>0&&<><button disabled={!selected} onClick={()=>setPage(selected-1)}>Previous consequences</button><span>{selected+1}/{last+1}</span><button disabled={selected===last} onClick={()=>setPage(selected+1)}>Next consequences</button></>}</section>;
}
export function SourceRecoveryView({close,buffers,acquire,adopt,alive,epoch,editorSessions}:{editorSessions?:()=>void;close:()=>void;buffers:Buffers;acquire:(path:string)=>Promise<Buffer>;adopt:(routes:ResourceEditorRoute<Buffer>[],recovery:string)=>void;alive:()=>boolean;epoch:()=>number}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new RecoveryReader(route=>request('GET',route),paint),[]),panel=useRef<HTMLElement>(null);
 const [path,setPath]=useState(''),[snapshot,setSnapshot]=useState<RecoverySnapshot>('draft_before');
 const [review,setReview]=useState<ResourceRestoreReview|null>(null),[busy,setBusy]=useState(false),[message,setMessage]=useState(''),[inverse,setInverse]=useState<string|null>(null);
 const [reviewPage,setReviewPage]=useState(0);
 const lifetime=useRef(true);
 async function prepare(record=model.value){
  if(!record||busy)return;const stamp=epoch();setBusy(true);setReview(null);setMessage('Preparing current saved and draft restoration…');
  try{const next=await prepareResourceRestore(buffers,record,acquire,(route,body)=>request('POST',route,body),()=>lifetime.current&&alive()&&epoch()===stamp&&model.value?.id===record.id&&model.value.fingerprint===record.fingerprint);setReview(next);setReviewPage(0);setPath(next.patches[0].path);setMessage('Review every saved baseline and source draft before restoring.');}
  catch(error){setMessage(String(error));}finally{setBusy(false);}
 }
 async function publish(){if(!review||busy)return;const selected=review;setReview(null);setBusy(true);setMessage('Publishing reviewed source restoration…');
  try{const result=await publishResourceRestore(buffers,selected,(route,body)=>request('POST',route,body));adopt(result.routes,result.recovery);setInverse(result.recovery);setMessage('Restored saved baselines and source drafts. Prepare inverse to undo this restoration. No checks were run.');}
  catch(error){setMessage(String(error)+' Inspect Source recovery and current files before preparing another operation.');}finally{setBusy(false);}
 }
 async function prepareInverse(){if(!inverse||busy)return;setBusy(true);setReview(null);try{const record=await request<RecoveryReading>('GET','/api/changes/resource-recovery?id='+encodeURIComponent(inverse));if(record.id!==inverse||!alive())throw new Error('Inverse recovery identity changed.');model.value=record;setBusy(false);await prepare(record);}catch(error){setMessage(String(error));setBusy(false);}}
 useEffect(()=>{lifetime.current=true;panel.current?.focus();void model.list();return()=>{lifetime.current=false;model.close();};},[model]);
 const value=model.value,selected=value?.paths.find(item=>item.path===path),text=value?.[snapshot.startsWith('saved')?'saved_text':'draft_text'];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source recovery" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();if(!busy)close();}}}>
 <header><strong>Source recovery</strong>{editorSessions&&<button disabled={busy} onClick={editorSessions}>Editor sessions…</button>}<button disabled={busy} onClick={close}>Close</button></header>
 <button disabled={model.loading||busy} onClick={()=>{setReview(null);setPath('');void model.list();}}>Refresh records</button><p role="status">{model.notice}</p>
 {model.page&&<><label>Retained record <select aria-label="Retained record" value={value?.id??''} disabled={model.loading||busy} onChange={event=>{if(event.target.value){setReview(null);setPath('');void model.read(event.target.value);}}}><option value="">Choose record</option>{model.page.ids.map(id=><option key={id} value={id}>{id}</option>)}</select></label>{model.page.ignored>0&&<p>{model.page.ignored} unrecognized entries omitted.</p>}{model.page.next&&<button disabled={model.loading||busy} onClick={()=>{setPath('');void model.list(model.page!.next);}}>Next 100 identities</button>}</>}
 {message&&<p role="status">{message}</p>}{inverse&&<button disabled={busy||model.loading} onClick={()=>{void prepareInverse();}}>Prepare inverse restoration</button>}
 {value&&<><h3>{value.title}{value.title_truncated?'…':''}</h3><p>{value.all_paths_recorded_applied?'Every path has an applied receipt.':'The record does not confirm every path was applied.'} These receipts describe the recorded operation, not current disk state.</p>{value.unrecognized_files.length>0&&<p>{value.unrecognized_files.length} unrecognized files in this record.</p>}
 <button disabled={busy||model.loading} onClick={()=>{void prepare();}}>Prepare restoration</button>
 <label>Recorded source <select aria-label="Recorded source" value={path} disabled={model.loading||busy} onChange={event=>{setPath(event.target.value);void model.read(value.id,event.target.value,snapshot);}}><option value="" disabled>Choose source</option>{value.paths.map(item=><option key={item.path} value={item.path}>{item.path}</option>)}</select></label>
 <label>Snapshot <select aria-label="Snapshot" value={snapshot} disabled={model.loading||busy} onChange={event=>{const next=event.target.value as RecoverySnapshot;setSnapshot(next);if(path)void model.read(value.id,path,next);}}>{(['saved_before','draft_before','saved_after','draft_after'] as RecoverySnapshot[]).map(side=><option key={side} value={side}>{side.replace('_',' ')}</option>)}</select></label>
 {selected&&<><p>{selected[snapshot].kind==='missing'?'Source absent in this snapshot.':`${selected[snapshot].bytes} UTF-8 bytes · ${selected[snapshot].sha256}`}</p>{text!==null&&text!==undefined&&<><pre style={{whiteSpace:'pre-wrap'}}>{text.slice(0,32000)}</pre>{text.length>32000&&<p>First 32,000 characters shown.</p>}</>}</>}
 </>}
 {review&&<section aria-label="Restoration preview"><h3>Restore saved sources and drafts</h3><p>{review.patches.length} paths · source and construction revisions are checked again on publication.</p>
 <label>Restoration source <select aria-label="Restoration source" value={path} onChange={event=>{setPath(event.target.value);setReviewPage(0);}}>{review.patches.map(item=><option key={item.path} value={item.path}>{item.path}</option>)}</select></label>
 {review.patches.filter(item=>item.path===path).map(item=><div key={item.path}><strong>Current draft</strong><pre>{item.before?.text.slice(reviewPage*16000,(reviewPage+1)*16000)??'Source absent'}</pre><strong>Restored draft</strong><pre>{item.after?.text.slice(reviewPage*16000,(reviewPage+1)*16000)??'Source absent'}</pre><strong>Current saved baseline</strong><pre>{review.savedBefore[path]?.kind==='file'?(review.savedBefore[path] as {text:string}).text.slice(reviewPage*16000,(reviewPage+1)*16000):'Source absent'}</pre><strong>Restored saved baseline</strong><pre>{review.saved[path]?.kind==='file'?(review.saved[path] as {text:string}).text.slice(reviewPage*16000,(reviewPage+1)*16000):'Source absent'}</pre><p>Text page {reviewPage+1} · 16,000 characters per side. Draft text and saved text may differ.</p><button disabled={!reviewPage} onClick={()=>setReviewPage(reviewPage-1)}>Previous restoration text</button><button disabled={Math.max(item.before?.text.length??0,item.after?.text.length??0,review.savedBefore[path]?.kind==='file'?(review.savedBefore[path] as {text:string}).text.length:0,review.saved[path]?.kind==='file'?(review.saved[path] as {text:string}).text.length:0)<=(reviewPage+1)*16000} onClick={()=>setReviewPage(reviewPage+1)}>Next restoration text</button></div>)}
 <ResourceConsequencesView review={review}/><button disabled={busy} onClick={()=>{void publish();}}>Restore reviewed saved sources and drafts</button></section>}
 </section>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceRecovery.tsx"><ui:paste select="#source-recovery-browser-0"/><ui:paste select="#source-recovery-browser-1"/></ui:file>

## apple 1

The reader binds selection to a workspace and exact record fingerprint. Each read retires older requests; refresh and close invalidate pending replies. A later snapshot of changed evidence refuses, so two sides cannot silently come from different records.

<ui:copy id="source-recovery-apple-0">
import SwiftUI
import Observation
private func recoveryQuery(_ text:String)->String {text.addingPercentEncoding(withAllowedCharacters:CharacterSet(charactersIn:"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"))!}
struct RecoveryPage:Decodable {var version:Int;var workspace:String;var ids:[String];var next:String?;var ignored:Int}
struct RecoverySummary:Decodable {var kind:String;var bytes:Int?;var sha256:String?}
struct RecoveryPath:Decodable {var path:String;var saved_before:RecoverySummary;var saved_after:RecoverySummary;var draft_before:RecoverySummary;var draft_after:RecoverySummary
 func summary(_ snapshot:String)->RecoverySummary {switch snapshot {case "saved_before":return saved_before;case "saved_after":return saved_after;case "draft_after":return draft_after;default:return draft_before}}
}
struct RecoveryReading:Decodable {var version:Int;var workspace:String;var id:String;var fingerprint:String;var title:String;var title_truncated:Bool;var paths:[RecoveryPath];var all_paths_recorded_applied:Bool;var unrecognized_files:[String];var draft_text:String?;var saved_text:String?}
@MainActor @Observable final class RecoveryReader {
 var page:RecoveryPage?;var value:RecoveryReading?;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();page=nil;value=nil;loading=false}
 func list(after:String?=nil,load:(String)async throws->RecoveryPage)async {
  let id=UUID();ticket=id;loading=true;value=nil;notice="Reading retained recovery identities…";defer{if ticket==id{loading=false}}
  do{let next=try await load("/api/changes/resource-recoveries"+(after.map{"?after="+recoveryQuery($0)} ?? ""));guard ticket==id else{return}
   guard next.version==1,!next.workspace.isEmpty,next.ids.count<=100,next.ids.allSatisfy({$0.range(of:"^[0-9a-f]{32}$",options:.regularExpression) != nil}),zip(next.ids,next.ids.dropFirst()).allSatisfy({$0.0<$0.1}),after.map({cursor in next.ids.allSatisfy{$0>cursor}&&next.workspace==page?.workspace}) ?? true,next.next==nil||next.next==next.ids.last else{throw EngineError(message:"Recovery page identity changed. Refresh from the beginning.")}
   page=next;notice=next.ids.isEmpty ? "No retained resource recovery records.":"Choose a retained record. Identities are sorted, not chronological."
  }catch{if ticket==id{page=nil;notice=error.localizedDescription}}
 }
 func read(_ record:String,path:String?=nil,snapshot:String="draft_before",load:(String)async throws->RecoveryReading)async {
  let workspace=page?.workspace,previous=value?.id==record ? value:nil,id=UUID();ticket=id;loading=true;value=nil;notice="Reading retained source evidence…";defer{if ticket==id{loading=false}}
  do{guard let workspace,page?.ids.contains(record)==true else{throw EngineError(message:"Refresh recovery identities before selecting a record.")}
   let parts=snapshot.split(separator:"_");guard parts.count==2,["saved","draft"].contains(String(parts[0])),["before","after"].contains(String(parts[1])) else{throw EngineError(message:"Choose a supported recovery snapshot.")}
   let route="/api/changes/resource-recovery?id="+recoveryQuery(record)+(path.map{"&path="+recoveryQuery($0)+"&"+parts[0]+"="+parts[1]} ?? "")
   let reply=try await load(route);guard ticket==id else{return}
   guard reply.version==1,reply.workspace==workspace,reply.id==record,reply.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,previous.map({$0.fingerprint==reply.fingerprint}) ?? true,path.map({path in reply.paths.contains{exactUTF8($0.path,path)}}) ?? true else{throw EngineError(message:"Recovery evidence changed. Refresh identities and read again.")}
   value=reply;notice="Retained evidence only. Current source and editor drafts are unchanged."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
}
</ui:copy>

### Apple focused verification, 2026-11-15

Ten selected `SourceRecoveryTests` pass with full Xcode. They include the actual engine path: publish a dirty source move, restore its original saved baseline while retaining a separate dirty draft, redo, then refuse a later local edit. The same buffer, caret and pane geometry survive. Controlled controller cases cover complete provider resource actions, mixed draft/resource history, changed ownership, lost replies/status without replay, malformed recovery metadata with retained confirmed publication evidence, and whole-action command refusal. The iOS simulator application build passes separately. These are native controller/protocol and compilation witnesses; physical input, mirrored IME and assistive-technology acceptance remain unexecuted.

### Decode state separately from provider metadata

Only provider edits and annotations carry open JSON. Exact draft text, source identities, saved metadata and construction facts have typed fields. Preserve integer metadata without rounding; require a complete bounded plan before routing editors. The native protocol currently refuses canonically equivalent duplicate path spellings because Swift string-keyed wire maps cannot retain both identities; this is an explicit admission limit, not proof of byte-distinct namespace parity.

<ui:copy id="source-recovery-apple-wire">
// Only annotations and provider edits are structurally open JSON. State,
// baselines and publication admission use the typed values below.
indirect enum ResourceJSON:Codable {
 case object([String:ResourceJSON]),array([ResourceJSON]),string(String),integer(Int64),unsigned(UInt64),number(Double),bool(Bool),null
 init(from decoder:Decoder)throws {let value=try decoder.singleValueContainer();if value.decodeNil(){self = .null}else if let item=try? value.decode(Bool.self){self = .bool(item)}else if let item=try? value.decode(String.self){self = .string(item)}else if let item=try? value.decode(Int64.self){self = .integer(item)}else if let item=try? value.decode(UInt64.self){self = .unsigned(item)}else if let item=try? value.decode(Double.self){self = .number(item)}else if let item=try? value.decode([String:ResourceJSON].self){self = .object(item)}else{self = .array(try value.decode([ResourceJSON].self))}}
 func encode(to encoder:Encoder)throws {var value=encoder.singleValueContainer();switch self {case .object(let item):try value.encode(item);case .array(let item):try value.encode(item);case .string(let item):try value.encode(item);case .integer(let item):try value.encode(item);case .unsigned(let item):try value.encode(item);case .number(let item):try value.encode(item);case .bool(let item):try value.encode(item);case .null:try value.encodeNil()}}
}
private func resourceBody<T:Encodable>(_ value:T)throws->[String:Any] {guard let object=try JSONSerialization.jsonObject(with:JSONEncoder().encode(value)) as? [String:Any] else{throw EngineError(message:"Resource request must be a typed object.")};return object}
func resourceRecoveryIdentity(_ path:String)throws->String {
 let parts=path.split(separator:"/",omittingEmptySubsequences:false)
 guard parts.count==4,parts[0]==".hick-cache",parts[1]=="resource-changes",parts[3]=="manifest.json",parts[2].range(of:"^[0-9a-f]{32}$",options:.regularExpression) != nil else{throw EngineError(message:"Saved publication returned an invalid recovery record address. Retain its request identity and inspect Source recovery before adopting editors.")}
 return String(parts[2])
}
struct ResourceWireSource:Codable {
 var kind:String;var identity:String?;var text:String?;var writable:Bool?;var removable:Bool?;var version:Int?;var creatable:Bool?
 func source()throws->ResourceSource? {
  if kind=="missing",creatable != nil{return nil}
  guard kind=="file",let identity,!identity.isEmpty,let text,writable != nil,removable != nil else{throw EngineError(message:"Resource source state is incomplete. Prepare the complete proposal again.")}
  return ResourceSource(identity:identity,text:text)
 }
}
struct ResourceWirePatch:Codable {var path:String;var before:ResourceWireSource;var after:ResourceWireSource}
struct ResourceWirePlan:Codable {var version:Int;var context:String;var title:String;var patches:[ResourceWirePatch];var annotations:[ResourceJSON]
 func editors()throws->[ResourceEditorPatch] {
  guard version==1,!context.isEmpty,!patches.isEmpty,patches.count<=1000 else{throw EngineError(message:"Resource plan identity or size is invalid.")}
  guard Set(patches.map{$0.path}).count==patches.count else{throw EngineError(message:"The native wire maps cannot distinguish duplicate or canonically equivalent path spellings. No resource change was published; use a source scope with distinct names.")}
  return try patches.map{ResourceEditorPatch(path:$0.path,before:try $0.before.source(),after:try $0.after.source())}
 }
}
struct ResourceWireOwnership:Codable {var mode:String;var entry:String?;var protected_entries:[String];var saved_sources:[String:String];var draft_sources:[String:String]}
struct ResourceWireInputs:Codable {var version:Int;var context:String;var workspace:String;var ownership:ResourceWireOwnership;var sources:[String:ResourceWireSource];var saved_sources:[String:ResourceSaved];var refusals:[String:String];var source_files_modified:Bool;var saved_guards:[String:String]?=nil}
struct ResourceWireDraft:Codable {var text:String;var base_sha256:String;var version:Int?;var writable:Bool;var save_guard:String?=nil}
struct ResourceWireCapture:Codable {var context:String;var edit:ResourceJSON;var drafts:[String:ResourceWireDraft];var project_entry:String?;var ownership_sources:[String:String];var unaddressed_sources:[String:String]=[:];var saved_guards:[String:String]=[:]}
struct ResourceWireLayer:Decodable {var entries:[String];var removed_entries:[String];var source_hashes:[String:String];var output_paths:[String]}
struct ResourceWireFacts:Decodable {var version:Int;var workspace:String;var context:String;var fingerprint:String;var saved:ResourceWireLayer;var draft:ResourceWireLayer;var protected_targets:[String];var source_files_modified:Bool;var saved_sources:[String:ResourceSaved]?}
struct ResourceWireTargets:Decodable {var version:Int;var paths:[String]}
struct ResourceWirePlanned:Decodable {var plan:ResourceWirePlan}
struct ResourceWireProposal:Codable {var inputs:ResourceWireInputs;var capture:ResourceWireCapture;var plan:ResourceWirePlan}
@MainActor func resourceOwnershipSnapshot(_ workspace:Workspace)throws->(named:[String:String],unaddressed:[String:String],guards:[String:String]) {
 let all=Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values),unique=Dictionary(grouping:all,by:{ObjectIdentifier($0)}).values.compactMap{$0.first};var named:[String:String]=[:],unaddressed:[String:String]=[:],guards:[String:String]=[:]
 guard unique.count<=4096 else{throw EngineError(message:"Retained source roster exceeds the bounded resource review. Preserve sources and choose a smaller workspace scope.")};var cost=0
 for buffer in unique where buffer.file.generated_by==nil {
  let bytes=buffer.text.utf8.count;guard bytes<=8*1024*1024,bytes<=16*1024*1024-cost else{throw EngineError(message:"Retained sources exceed the bounded resource review text capacity. Preserve them and choose a smaller scope.")};cost+=bytes
  if buffer.resourceDeleted||buffer.isUntitledDraft||buffer.file.path.hasPrefix("/"){unaddressed[buffer.id.uuidString]=buffer.text;continue}
  guard named[buffer.file.path]==nil else{throw EngineError(message:"Retained source names have ambiguous Unicode spellings; no source ownership was merged.")};named[buffer.file.path]=buffer.text;if let guardToken=buffer.savedGuard{guards[buffer.file.path]=guardToken}
 }
 return(named,unaddressed,guards)
}
@MainActor func captureRetainedSourceNamespace(_ workspace:Workspace)->()->Bool {
 let epoch=workspace.sourceNavigationEpoch,engine=workspace.engine,all=Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values),unique=Dictionary(grouping:all,by:{ObjectIdentifier($0)}).values.compactMap{$0.first}
 let snapshots=unique.map{($0,ObjectIdentifier($0),$0.sourceEditIdentity,ExactSourceText($0.file.path),$0.sourceRevision,ExactSourceText($0.base),$0.savedGuard.map{ExactSourceText($0)},$0.readOnly,$0.resourceDeleted,$0.file.generated_by.map{ExactSourceText($0)})}
 return{[weak workspace] in guard let workspace,workspace.sourceNavigationEpoch==epoch,workspace.engine?.origin==engine?.origin,workspace.engine?.session === engine?.session else{return false};let now=Set((Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values)).map{ObjectIdentifier($0)});guard now==Set(snapshots.map{$0.1}) else{return false};return snapshots.allSatisfy{buffer,_,revision,path,text,base,guardToken,readonly,deleted,generated in buffer.sourceEditIdentity==revision&&ExactSourceText(buffer.file.path)==path&&buffer.sourceRevision==text&&ExactSourceText(buffer.base)==base&&buffer.savedGuard.map{ExactSourceText($0)}==guardToken&&buffer.readOnly==readonly&&buffer.resourceDeleted==deleted&&buffer.file.generated_by.map{ExactSourceText($0)}==generated && !buffer.saving}}
}
func isResourceWorkspaceEdit(_ raw:Any)->Bool {guard let value=raw as? [String:Any],let operations=value["documentChanges"] as? [[String:Any]] else{return false};return operations.contains{$0["kind"] != nil}}
struct ResourceWireRestoreRequest:Codable {var recovery_id:String;var inputs:ResourceWireInputs;var capture:ResourceWireCapture}
struct ResourceWireRestoration:Decodable {var version:Int;var recovery_id:String;var recovery_fingerprint:String;var workspace:String;var context:String;var plan:ResourceWirePlan;var saved_before:[String:ResourceSaved];var saved_after:[String:ResourceSaved]}
struct ResourceWirePrepared:Decodable {var version:Int;var restoration:ResourceWireRestoration;var consequences:ResourceWireFacts}
struct ResourceRestorationReview {
 let identity=UUID();var preflight:(()async throws->Void)?=nil;var published:((String)->Void)?=nil
 var request:ResourceWireRestoreRequest;var result:ResourceWirePrepared;var patches:[ResourceEditorPatch];var retained:[String:DocumentBuffer];var current:()->Bool;var save:ResourceWireProposal?=nil;var history:ResourceHistoryIntent?=nil
}

</ui:copy>

### Keep review, publication and editor adoption distinct

Acquire saved baselines and retained drafts independently. Freeze every retained buffer and ownership draft, so adding a construction source retires an earlier review. Newly opened provider sources synchronize before querying the complete action again. Construction previews protect outputs but never stand in for executed checks.

Retain a unique pending request before sending a write. A lost or partial result cannot enable a fresh publication; read-only status may recover that original result. Only complete, matching receipts adopt the simultaneous buffer and tab transition. Source recovery uses the same path with a distinct saved projection, preserving restored dirty text. Workspace history orders resource recovery IDs with draft inverses; resource undo/redo is a review followed by explicit publication, guarded by that history revision.

<ui:copy id="source-recovery-apple-publication">
// A pending write outlives the popover. Never replay it automatically: status is
// read-only, and local admission is repeated before adopting a confirmed result.
@MainActor @Observable final class ResourceRestoration {
 private(set) var confirmedReceipt:ResourcePublication<FileNode>?
 var review:ResourceRestorationReview?;var notice="";var busy=false;var lastRecovery:String?;var selectedPath=0
 private var admitting=false
 private var ticket=UUID();private var pending:(id:String,review:ResourceRestorationReview)?;private weak var publishingWorkspace:Workspace?
 var hasPending:Bool {pending != nil}
 var pendingRequest:String? {pending?.id}
 var transport:((String,String,[String:Any]?) async throws->ResourceJSON)?
 private func call<T:Decodable>(_ workspace:Workspace,_ method:String,_ route:String,_ body:[String:Any]?=nil)async throws->T {
  if let transport {return try JSONDecoder().decode(T.self,from:JSONEncoder().encode(try await transport(method,route,body)))}
  guard let engine=workspace.engine else{throw EngineError(message:"Connect to a workspace server first.")}
  let address=URLComponents(string:route)!
  let value:ResourceJSON=try await engine.request(method,address.path,query:address.queryItems ?? [],body:body)
  return try JSONDecoder().decode(T.self,from:JSONEncoder().encode(value))
 }
 func cancel(){guard !hasPending else{return};ticket=UUID();review=nil;busy=false;notice=""}
 private func currentWorkspace(_ workspace:Workspace)->()->Bool {captureRetainedSourceNamespace(workspace)}
 func prepare(_ workspace:Workspace,record:RecoveryReading,history:ResourceHistoryIntent?=nil)async {
  guard !busy,!hasPending else{notice="Inspect the pending publication before preparing another restoration.";return}
  let id=UUID();ticket=id;busy=true;review=nil;notice="Acquiring current sources and construction ownership…";defer{if ticket==id{busy=false}}
  let initial=workspace.sourceNavigationEpoch
  let historyCurrent:()->Bool={history.map{workspace.changes.historySnapshot==$0.token && workspace.changes.resourceHistoryRecord($0.direction)==$0.record} ?? true}
  do {
   guard historyCurrent(),record.paths.count>0,record.paths.count<=1000 else{throw EngineError(message:"Choose a bounded recovery record.")}
   func node(_ path:String,_ nodes:[FileNode])->FileNode? {for item in nodes {if exactUTF8(item.path,path){return item};if let child=node(path,item.children ?? []){return child}};return nil}
   for item in record.paths where item.saved_after.kind=="file" && workspace.buffers[item.path]==nil {
    let file=node(item.path,workspace.rootFiles) ?? FileNode(name:(item.path as NSString).lastPathComponent,path:item.path,dir:false)
    _ = try await workspace.loadBuffer(file)
    guard ticket==id,workspace.sourceNavigationEpoch==initial else{throw EngineError(message:"Workspace changed while acquiring recovery sources.")}
   }
   let captured=currentWorkspace(workspace),context=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased()
   var edits:[String:ResourceJSON]=[:],drafts:[String:ResourceWireDraft]=[:]
   for item in record.paths {
    edits[languageURI(item.path)] = .array([])
    if let buffer=workspace.buffers[item.path] {drafts[item.path]=ResourceWireDraft(text:buffer.text,base_sha256:nativeSourceHash(buffer.base),version:nil,writable:!buffer.readOnly && !buffer.saving,save_guard:buffer.savedGuard)}
   }
   let ownership=try resourceOwnershipSnapshot(workspace)
   let capture=ResourceWireCapture(context:context,edit:.object(["changes":.object(edits)]),drafts:drafts,project_entry:nil,ownership_sources:ownership.named,unaddressed_sources:ownership.unaddressed,saved_guards:ownership.guards)
   let inputs:ResourceWireInputs=try await call(workspace,"POST","/api/changes/resource-inputs",resourceBody(capture))
   guard ticket==id,captured(),inputs.version==1,inputs.saved_guards != nil,inputs.context==context,inputs.workspace==record.workspace,inputs.refusals.isEmpty else{throw EngineError(message:"Current source, ownership or workspace changed. Refresh and prepare restoration again.")}
   let request=ResourceWireRestoreRequest(recovery_id:record.id,inputs:inputs,capture:capture)
   let result:ResourceWirePrepared=try await call(workspace,"POST","/api/changes/prepare-restoration",resourceBody(request))
   guard ticket==id,captured(),historyCurrent(),result.version==1,result.restoration.version==1,result.restoration.recovery_id==record.id,result.restoration.recovery_fingerprint==record.fingerprint,result.restoration.workspace==record.workspace,result.restoration.context==context,result.restoration.plan.context==context,result.consequences.version==1,result.consequences.context==context,result.consequences.workspace==record.workspace,result.consequences.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,result.consequences.protected_targets.isEmpty else{throw EngineError(message:"Recovery evidence, construction consequences or current source changed. Prepare again.")}
   let patches=try result.restoration.plan.editors()
   guard Set(patches.map{$0.path})==Set(record.paths.map{$0.path}),patches.allSatisfy({patch in patch.before.map{exactUTF8(workspace.buffers[patch.path]?.text,$0.text)} ?? (workspace.buffers[patch.path]==nil)}) else{throw EngineError(message:"Every current source must match the complete restoration plan.")}
   var retained:[String:DocumentBuffer]=[:]
   for patch in patches {if let after=patch.after,let buffer=workspace.retainedResourceEditors[after.identity] {retained[after.identity]=buffer}}
   let local=workspace.captureResourceEditors(patches,restoring:retained)
   review=ResourceRestorationReview(request:request,result:result,patches:patches,retained:retained,current:{captured() && local() && historyCurrent()},history:history);selectedPath=0
   notice="Review saved baselines and dirty drafts separately. Publication changes saved source; generated files are only analyzed."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 func prepareEdit(_ workspace:Workspace,title:String,edit:Any,current:@escaping()->Bool={true},requery:(() async throws->Any)?=nil)async {
  guard !busy,!hasPending else{notice="Inspect the pending publication before preparing another resource proposal.";return}
  let id=UUID();ticket=id;busy=true;review=nil;notice="Acquiring the complete resource proposal…";defer{if ticket==id{busy=false}}
  do {
   var selectedEdit=edit
   var raw=try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:edit))
   let discovered:ResourceWireTargets=try await call(workspace,"POST","/api/changes/resource-targets",["edit":edit])
   guard ticket==id,current(),discovered.version==1,!discovered.paths.isEmpty,discovered.paths.count<=1000,Set(discovered.paths).count==discovered.paths.count else{throw EngineError(message:"Resource targets or provider changed. Prepare the complete proposal again.")}
   let acquisition=try await acquireResourceInputs(workspace,paths:discovered.paths,edit:raw,current:{self.ticket==id&&current()},readInputs:{capture in try await self.call(workspace,"POST","/api/changes/resource-inputs",resourceBody(capture))})
   var sourceCapture=acquisition.capture,inputs=acquisition.inputs;let captured=acquisition.current,context=sourceCapture.context
   if acquisition.acquired,let requery {
    for path in discovered.paths {if let buffer=workspace.buffers[path],let service=buffer.language {let source=buffer.text;_ = try await service.readOutline(source:source);guard exactUTF8(buffer.text,source),ticket==id,current(),captured() else{throw EngineError(message:"Original source changed while synchronizing the resource proposal.")}}}
    selectedEdit=try await requery()
    let rediscovered:ResourceWireTargets=try await call(workspace,"POST","/api/changes/resource-targets",["edit":selectedEdit])
    guard rediscovered.version==1,Set(rediscovered.paths.map{Data($0.utf8)})==Set(discovered.paths.map{Data($0.utf8)}),ticket==id,current(),captured() else{throw EngineError(message:"Provider resource targets or original source owners changed after acquisition. Prepare again.")}
    raw=try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:selectedEdit));sourceCapture.edit=raw
    inputs=try await call(workspace,"POST","/api/changes/resource-inputs",resourceBody(sourceCapture))
   }
   guard ticket==id,current(),captured(),inputs.version==1,inputs.saved_guards != nil,inputs.context==context,inputs.refusals.isEmpty,Set(inputs.sources.keys)==Set(discovered.paths) else{throw EngineError(message:"Complete resource sources changed. Prepare again.")}
   let planned:ResourceWirePlanned=try await call(workspace,"POST","/api/changes/plan-resources",["context":context,"title":title,"edit":selectedEdit,"sources":try resourceBody(inputs)["sources"]!])
   let patches=try planned.plan.editors()
   let proposal=ResourceWireProposal(inputs:inputs,capture:sourceCapture,plan:planned.plan)
   let facts:ResourceWireFacts=try await call(workspace,"POST","/api/changes/resource-consequences",resourceBody(proposal))
   guard ticket==id,current(),captured(),planned.plan.context==context,facts.version==1,facts.context==context,facts.workspace==inputs.workspace,facts.protected_targets.isEmpty,facts.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,let saved=facts.saved_sources,Set(saved.keys)==Set(patches.map{$0.path}) else{throw EngineError(message:"Resource consequences changed or complete saved baselines are unavailable. Prepare again.")}
   guard patches.allSatisfy({patch in patch.before.map{exactUTF8(workspace.buffers[patch.path]?.text,$0.text)} ?? (workspace.buffers[patch.path]==nil)}) else{throw EngineError(message:"Every current source must match the complete resource plan.")}
   let local=workspace.captureResourceEditors(patches)
   let restoration=ResourceWireRestoration(version:1,recovery_id:"",recovery_fingerprint:"",workspace:inputs.workspace,context:context,plan:planned.plan,saved_before:inputs.saved_sources,saved_after:saved)
   review=ResourceRestorationReview(request:ResourceWireRestoreRequest(recovery_id:"",inputs:inputs,capture:sourceCapture),result:ResourceWirePrepared(version:1,restoration:restoration,consequences:facts),patches:patches,retained:[:],current:{captured() && local() && current()},save:proposal);selectedPath=0
   notice="Review every text and namespace consequence together. Publish explicitly after reviewing saved source and generated consequences."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 func prepareHistory(_ workspace:Workspace,intent:ResourceHistoryIntent)async {
  guard !busy,!hasPending,workspace.changes.historySnapshot==intent.token,workspace.changes.resourceHistoryRecord(intent.direction)==intent.record else{notice="Workspace history changed. Prepare its current undo/redo entry.";return}
  let id=UUID(),epoch=workspace.sourceNavigationEpoch;ticket=id;busy=true
  do {
   let record:RecoveryReading=try await call(workspace,"GET","/api/changes/resource-recovery?id="+recoveryQuery(intent.record))
   guard ticket==id,workspace.sourceNavigationEpoch==epoch,record.version==1,record.id==intent.record,workspace.changes.historySnapshot==intent.token,workspace.changes.resourceHistoryRecord(intent.direction)==intent.record else{throw EngineError(message:"Workspace history or recovery changed. Prepare again.")}
   busy=false;await prepare(workspace,record:record,history:intent)
  }catch{if ticket==id{busy=false;notice=error.localizedDescription}}
 }
 func prepareInverse(_ workspace:Workspace)async {
  guard let record=lastRecovery,!busy,!hasPending else{return}
  let id=UUID(),epoch=workspace.sourceNavigationEpoch;ticket=id;busy=true
  do {let value:RecoveryReading=try await call(workspace,"GET","/api/changes/resource-recovery?id="+recoveryQuery(record));guard ticket==id,workspace.sourceNavigationEpoch==epoch,value.version==1,value.id==record,value.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil else{throw EngineError(message:"Recovery identity or workspace changed.")};busy=false;await prepare(workspace,record:value)}catch{if ticket==id{busy=false;notice=error.localizedDescription}}
 }
 func acceptSealed(_ workspace:Workspace,proposal:ResourceWireProposal,facts:ResourceWireFacts,current:@escaping()->Bool,preflight:@escaping()async throws->Void,published:@escaping(String)->Void)throws->UUID {
  guard !busy,!hasPending,review==nil,current(),let saved=facts.saved_sources else{throw EngineError(message:"Finish the existing resource review before staging the complete intention.")}
  let patches=try proposal.plan.editors();let local=workspace.captureResourceEditors(patches)
  guard facts.context==proposal.plan.context,facts.workspace==proposal.inputs.workspace,Set(saved.keys.map(NativeSourceKey.init))==Set(patches.map{NativeSourceKey($0.path)}),patches.allSatisfy({p in p.before.map{exactUTF8(workspace.buffers[p.path]?.text,$0.text)} ?? (workspace.buffers[p.path]==nil)}) else{throw EngineError(message:"Complete sealed resource sources are not current.")}
  let restoration=ResourceWireRestoration(version:1,recovery_id:"",recovery_fingerprint:"",workspace:proposal.inputs.workspace,context:proposal.plan.context,plan:proposal.plan,saved_before:proposal.inputs.saved_sources,saved_after:saved)
  var next=ResourceRestorationReview(request:ResourceWireRestoreRequest(recovery_id:"",inputs:proposal.inputs,capture:proposal.capture),result:ResourceWirePrepared(version:1,restoration:restoration,consequences:facts),patches:patches,retained:[:],current:{current()&&local()},save:proposal)
  next.preflight=preflight;next.published=published;review=next;return next.identity
 }
 func publish(_ workspace:Workspace)async {
  guard !busy,!admitting,pending==nil,let value=review else{return}
  guard value.current() else{review=nil;notice="Source changed after review. Prepare restoration again.";return}
  admitting=true;defer{admitting=false}
  do{try await value.preflight?()}catch{notice=error.localizedDescription;return}
  guard review?.identity==value.identity,value.current(),pending==nil,!busy else{notice="Complete review changed during final admission.";return}
  let id=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased();pending=(id,value);confirmedReceipt=nil;publishingWorkspace=workspace;review=nil;busy=true
  defer{busy=false}
  do {
   var body:[String:Any]=["request_id":id,"reviewed_fingerprint":value.result.consequences.fingerprint]
   if let save=value.save {body["proposal"]=try resourceBody(save)}
   else {body["proposal"]=try resourceBody(value.request);body["reviewed_recovery_fingerprint"]=value.result.restoration.recovery_fingerprint}
   let receipt:ResourcePublication<FileNode>=try await call(workspace,"POST",value.save==nil ? "/api/changes/restore-resources":"/api/changes/resource-save",body)
   try adopt(workspace,id:id,value:value,receipt:receipt)
  }catch{notice=error.localizedDescription+(confirmedReceipt?.request_id==id ? " Saved publication is confirmed; editor adoption is incomplete. Preserve local drafts and inspect Source recovery. This action will not be resent.":" Publication is unconfirmed. Inspect status or retained recovery; this action will not be retried.")}
 }
 func status(_ workspace:Workspace)async {
  guard !busy,let (id,value)=pending else{return};busy=true;defer{busy=false}
  do {guard publishingWorkspace === workspace else{throw EngineError(message:"Inspect recovery in the original workspace.")};let receipt:ResourcePublication<FileNode>=try await call(workspace,"GET","/api/changes/resource-save?request_id="+id);try adopt(workspace,id:id,value:value,receipt:receipt)}catch{notice=error.localizedDescription+(confirmedReceipt?.request_id==id ? " Saved publication is confirmed; preserve drafts and inspect recovery because editor adoption is incomplete.":" Preserve drafts and inspect retained recovery before another publication.")}
 }
 private func adopt(_ workspace:Workspace,id:String,value:ResourceRestorationReview,receipt:ResourcePublication<FileNode>)throws {
  guard publishingWorkspace === workspace else{throw EngineError(message:"Workspace changed during publication. Inspect recovery in its original workspace.")}
  _ = try confirmedResourceFiles(value.patches,receipt:receipt,request:id,workspace:value.result.restoration.workspace,context:value.request.capture.context,expected:value.result.restoration.saved_after){($0.path,!$0.dir && $0.generated_by==nil)}
  confirmedReceipt=receipt
  let recovery=try resourceRecoveryIdentity(receipt.outcome.recovery)
  _ = try workspace.adoptResourceReceipt(value.patches,receipt:receipt,request:id,workspace:value.result.restoration.workspace,context:value.request.capture.context,expected:value.result.restoration.saved_after,current:value.current,restoring:value.retained)
  workspace.changes.recordResourcePublication(recovery,intent:value.history)
  pending=nil;lastRecovery=recovery;value.published?(recovery);notice=value.save==nil ? "Restored confirmed saved baselines and retained dirty drafts. Prepare the inverse restoration to redo this operation.":"Saved the reviewed resource namespace. Use Undo workspace change to review its inverse. No tests were executed by this save."
  Task {await workspace.refresh()}
 }
}
</ui:copy>

## apple 2

Render the selected retained evidence in a floating reading, reached through existing menu commands. Keep refresh, paging and snapshot controls inside this reading, with selectable abbreviated text and explicit absence. Closing returns focus to the existing source editor.

<ui:copy id="source-recovery-apple-1">
struct SourceRecoveryView:View {
 var workspace:Workspace;@State private var model=RecoveryReader();@State private var path="";@State private var snapshot="draft_before"
 private func list(_ after:String?=nil)async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};workspace.resourceRestoration.cancel();path="";await model.list(after:after){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 private func read(_ id:String,_ path:String?=nil,_ snapshot:String="draft_before")async {guard let engine=workspace.engine else{return};workspace.resourceRestoration.cancel();await model.read(id,path:path,snapshot:snapshot){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 var body:some View {VStack(alignment:.leading,spacing:8){
  HStack{Text("Source recovery").font(.headline);Spacer();Button("Editor sessions…"){workspace.showRecovery=false;workspace.editorRecovery.showing=true}.disabled(workspace.resourceRestoration.busy || workspace.resourceRestoration.hasPending);Button("Close"){workspace.showRecovery=false;workspace.currentBuffer?.focusSource?()}}
  Button("Refresh records"){Task{await list()}}.disabled(model.loading);Text(model.notice).font(.caption).textSelection(.enabled)
  ResourceRestorationPreview(workspace:workspace,model:workspace.resourceRestoration)
  ScrollView{VStack(alignment:.leading,spacing:8){
   if let page=model.page {Picker("Retained record",selection:Binding(get:{model.value?.id ?? ""},set:{id in if !id.isEmpty{path="";Task{await read(id)}}})){Text("Choose record").tag("");ForEach(page.ids,id:\.self){Text($0).tag($0)}}.disabled(model.loading)
    if page.ignored>0 {Text("\(page.ignored) unrecognized entries omitted.").font(.caption)}
    if let next=page.next {Button("Next 100 identities"){Task{await list(next)}}.disabled(model.loading)}
   }
   if let value=model.value {
    Text(value.title+(value.title_truncated ? "…":"")).font(.headline)
    Button("Prepare restoration"){Task{await workspace.resourceRestoration.prepare(workspace,record:value)}}.disabled(model.loading || workspace.resourceRestoration.busy || workspace.resourceRestoration.hasPending)
    Text((value.all_paths_recorded_applied ? "Every path has an applied receipt.":"The record does not confirm every path was applied.")+" These receipts describe the recorded operation, not current disk state.").font(.caption)
    if !value.unrecognized_files.isEmpty {Text("\(value.unrecognized_files.count) unrecognized files in this record.").font(.caption)}
    Picker("Recorded source",selection:Binding(get:{path},set:{next in path=next;Task{await read(value.id,next,snapshot)}})){Text("Choose source").tag("");ForEach(Array(value.paths.enumerated()),id:\.offset){_,item in Text(item.path).tag(item.path)}}.disabled(model.loading)
    Picker("Snapshot",selection:Binding(get:{snapshot},set:{next in snapshot=next;if !path.isEmpty{Task{await read(value.id,path,next)}}})){ForEach(["saved_before","draft_before","saved_after","draft_after"],id:\.self){Text($0.replacingOccurrences(of:"_",with:" ")).tag($0)}}.disabled(model.loading)
    if let selected=value.paths.first(where:{exactUTF8($0.path,path)}) {let summary=selected.summary(snapshot)
     Text(summary.kind=="missing" ? "Source absent in this snapshot.":"\(summary.bytes ?? 0) UTF-8 bytes · \(summary.sha256 ?? "")").font(.caption).textSelection(.enabled)
     if let text=snapshot.hasPrefix("saved") ? value.saved_text:value.draft_text {Text(String(text.prefix(32000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled);if text.count>32000 {Text("First 32,000 characters shown.").font(.caption)}}
    }
   }
  }.frame(maxWidth:.infinity,alignment:.leading)}
 }.padding().frame(idealWidth:680,idealHeight:600).task{await list()}.onDisappear{model.close();workspace.resourceRestoration.cancel()}}
}

struct ResourceRestorationPreview:View {
 var workspace:Workspace;var model:ResourceRestoration
 @ViewBuilder private func saved(_ name:String,_ value:ResourceSaved?) -> some View {
  Text(name).font(.headline)
  if let value {Text(value.kind=="missing" ? "Source absent.":"Saved file · mode \(value.mode.map(String.init) ?? "provider default") · \(value.readonly==true ? "read-only":"writable")").font(.caption)
   if let text=value.text {Text(String(text.prefix(32000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled);if text.count>32000{Text("First 32,000 characters shown.").font(.caption)}}
  }else{Text("Saved baseline missing from review. Publication will refuse.").font(.caption)}
 }
 var body:some View {VStack(alignment:.leading,spacing:8){
  if !model.notice.isEmpty {Text(model.notice).font(.caption).textSelection(.enabled)}
  if let request=model.pendingRequest {Text("Resource request: "+request).font(.caption).textSelection(.enabled);Text(model.confirmedReceipt==nil ? "Publication result unknown. Preserve local editors until status or recovery confirms it.":"Saved publication confirmed; editor adoption is incomplete.").font(.caption);Button("Inspect publication status"){Task{await model.status(workspace)}}.disabled(model.busy)}
  if model.lastRecovery != nil {Button("Prepare inverse restoration (undo/redo)"){Task{await model.prepareInverse(workspace)}}.disabled(model.busy || model.hasPending)}
  if let review=model.review {
   Text("Review source change: \(review.result.restoration.plan.title)").font(.headline)
   Picker("Restored source",selection:Binding(get:{model.selectedPath},set:{model.selectedPath=$0})){ForEach(Array(review.patches.enumerated()),id:\.offset){index,patch in Text(patch.path).tag(index)}}
   ScrollView([.horizontal,.vertical]){VStack(alignment:.leading,spacing:8){
    if review.patches.indices.contains(model.selectedPath){let patch=review.patches[model.selectedPath]
     saved("Saved before",review.result.restoration.saved_before[patch.path]);saved("Saved after",review.result.restoration.saved_after[patch.path])
     Text("Draft before").font(.headline);Text(patch.before.map{String($0.text.prefix(32000))} ?? "Source absent.").font(.system(.caption,design:.monospaced)).textSelection(.enabled)
     Text("Draft after").font(.headline);Text(patch.after.map{String($0.text.prefix(32000))} ?? "Source absent.").font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    }
    Text("Saved construction outputs: "+review.result.consequences.saved.output_paths.joined(separator:", ")).font(.caption).textSelection(.enabled)
    Text("Draft construction outputs: "+review.result.consequences.draft.output_paths.joined(separator:", ")).font(.caption).textSelection(.enabled)
    Text("Construction consequences are observed relationships; no tests or coverage were executed by this preview.").font(.caption)
   }.frame(maxWidth:.infinity,alignment:.leading)}.frame(maxHeight:240)
   HStack{Button(review.save==nil ? "Publish reviewed restoration":"Save reviewed resource change"){Task{await model.publish(workspace)}}.disabled(model.busy || model.hasPending);Button("Discard preview"){model.cancel()}.disabled(model.busy)}
  }
 }}
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceRecovery.swift"><ui:paste select="#source-recovery-apple-0"/><ui:paste select="#source-recovery-apple-wire"/><ui:paste select="#source-recovery-apple-publication"/><ui:paste select="#source-recovery-apple-1"/></ui:file>

## android 1

The reader binds selection to a workspace and exact record fingerprint. Each read retires older requests; refresh and close invalidate pending replies. A later snapshot of changed evidence refuses, so two sides cannot silently come from different records.

<ui:copy id="source-recovery-android-0">
package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import java.net.URLEncoder
import org.json.JSONObject

// Preparation and publication are separate states. A failed POST is never
// retried: only the original request's read-only status may be consulted.
class ResourceChangeReview(private val transport:(suspend(String,String,JSONObject?)->JSONObject)?=null) {
 data class Review(val proposal:JSONObject,val facts:JSONObject,val plan:JSONObject,val saved:Map<String,ResourceSaved>,val restoration:Boolean,val fingerprint:String?,val current:()->Boolean,val restoring:Map<String,DocumentBuffer>,val published:((String,String)->Unit)?=null,val preflight:(suspend ()->Unit)?=null)
 var review by mutableStateOf<Review?>(null);private set
 var notice by mutableStateOf("");private set
 var busy by mutableStateOf(false);private set
 var lastRecovery by mutableStateOf<String?>(null);private set
 var lastWorkspace by mutableStateOf<String?>(null);private set
 private var ticket=0L
 private var pending:Pair<String,Review>?=null
 private var publishedWorkspace:Workspace?=null
 private fun nonce()=java.util.UUID.randomUUID().toString().replace("-","")
 private fun hash(text:String)=java.security.MessageDigest.getInstance("SHA-256").digest(text.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
 private suspend fun call(workspace:Workspace,method:String,path:String,body:JSONObject?=null)=transport?.invoke(method,path,body) ?: (workspace.engine ?: error("Connect to a workspace server first.")).request(method,path,body)
 internal fun acceptSealed(workspace:Workspace,proposal:JSONObject,facts:JSONObject,plan:JSONObject,current:()->Boolean,preflight:suspend ()->Unit,published:((String,String)->Unit)?=null):Review {
  check(!busy&&pending==null&&current()){ "Finish the current publication and refresh the complete proposal." }
  val copied=JSONObject(proposal.toString());val reviewedFacts=JSONObject(facts.toString());val reviewedPlan=JSONObject(plan.toString());val paths=resourcePatches(reviewedPlan).map{it.path}
  check(paths.isNotEmpty()&&paths.distinct().size==paths.size&&reviewedPlan.get("version")==1&&copied.getJSONObject("inputs").getString("context")==reviewedPlan.getString("context")&&reviewedFacts.getString("context")==reviewedPlan.getString("context")&&reviewedFacts.getString("workspace")==workspace.engine?.sourceWorkspaceID&&reviewedFacts.getJSONArray("protected_targets").length()==0){"Sealed resource consequences do not match this workspace."}
  val saved=resourceSavedMap(reviewedFacts.getJSONObject("saved_sources"));check(saved.keys==paths.toSet()){ "Sealed proposal does not contain the complete saved namespace." }
  ticket++;return Review(copied,reviewedFacts,reviewedPlan,saved,false,null,current,emptyMap(),published,preflight).also{review=it;notice="Complete sealed proposal staged for guarded Save. No file was written."}
 }
 fun dismiss(){if(pending!=null)return;ticket++;review=null;busy=false}
 fun reset(){ticket++;review=null;pending=null;publishedWorkspace=null;lastRecovery=null;lastWorkspace=null;busy=false;notice=""}
</ui:copy>

### Capture the retained authored namespace

Resource coordinates and ownership depend on more than the selected paths. Keep
every retained authored buffer's identity, draft, baseline and file metadata in
the admission closure, and reject newly retained authored sources. Supply every named authored draft, regardless of suffix, as an ownership input.
Retained deleted, unnamed and outside-address drafts carry opaque identities in
unaddressed_sources; the provider decides whether their declarations have a proven
construction context. Complete live and retained address maps, physical baseline
guards, selections and revisions remain part of the lease. Generated editors
never enter writable draft capture; unfinished composition or Save
refuses preparation before any publication request.

<ui:copy id="source-resource-capture-android">
 private suspend fun capture(workspace:Workspace,paths:List<String>,edit:JSONObject,valid:()->Boolean):Pair<JSONObject,()->Boolean> {
  check(paths.isNotEmpty()&&paths.size<=1000&&paths.distinct().size==paths.size){"Choose a bounded complete source proposal."}
  val epoch=workspace.sourceNavigationEpoch;val server=workspace.engine
  fun find(nodes:List<FileNode>,path:String):FileNode?=nodes.firstNotNullOfOrNull{if(it.path==path)it else find(it.children,path)}
  for(path in paths){check(valid()&&workspace.sourceNavigationEpoch==epoch){"Source or workspace changed. Prepare again."};find(workspace.rootFiles,path)?.takeUnless{it.dir}?.let{workspace.loadBuffer(it)}}
  val addresses=workspace.buffers.toMap();val retained=workspace.retainedResourceEditors.toMap();val tabs=workspace.retainedResourceTabs.toMap()
  fun roster()=(workspace.buffers.values+workspace.retainedResourceEditors.values+workspace.retainedResourceTabs.values).distinct().filter{it.file.generatedBy==null}
  data class Snapshot(val buffer:DocumentBuffer,val text:String,val base:String,val hash:String,val guard:String?,val file:FileNode,val revision:Long,val deleted:Boolean,val readonly:Boolean,val untitled:Boolean,val selection:androidx.compose.ui.text.TextRange)
  val snapshots=roster().map{check(!it.saving&&it.state.composition==null){"Finish source input and saves before preparing resources."};Snapshot(it,it.text,it.base,it.hash,it.saveGuard,it.file,it.sourceRevision,it.resourceDeleted,it.readOnly,it.isUntitledDraft,it.state.selection)}
  check(snapshots.size<=4096){"Retained source roster exceeds its bounded capture."}
  fun same(live:Map<String,DocumentBuffer>,before:Map<String,DocumentBuffer>)=live.size==before.size&&before.all{(path,b)->live[path]===b}
  val current={valid()&&workspace.engine===server&&workspace.sourceNavigationEpoch==epoch&&same(workspace.buffers,addresses)&&same(workspace.retainedResourceEditors,retained)&&same(workspace.retainedResourceTabs,tabs)&&roster().toSet()==snapshots.map{it.buffer}.toSet()&&snapshots.all{val b=it.buffer;b.sourceRevision==it.revision&&b.text==it.text&&b.base==it.base&&b.hash==it.hash&&b.saveGuard==it.guard&&b.file===it.file&&b.resourceDeleted==it.deleted&&b.readOnly==it.readonly&&b.isUntitledDraft==it.untitled&&b.state.selection==it.selection&&!b.saving&&b.state.composition==null}}
  check(current()){"Source changed. Prepare again."}
  val drafts=JSONObject();val ownership=JSONObject();val unaddressed=JSONObject();val guards=JSONObject();var bytes=0L
  for(s in snapshots){val named=!s.deleted&&!s.untitled&&!s.file.path.startsWith('/')&&addresses[s.file.path]===s.buffer
   val key=if(named)s.file.path else s.buffer.debugIdentity;check(s.text.length<=8*1024*1024&&key.length<=8192){"Retained source input exceeds its bounded capture."};bytes+=key.toByteArray(Charsets.UTF_8).size.toLong()+s.text.toByteArray(Charsets.UTF_8).size;check(bytes<=32L*1024*1024){"Retained resource ownership capture exceeds 32 MiB."}
   if(named){check(!ownership.has(key)){"Retained authored source addresses are ambiguous."};ownership.put(key,s.text);s.guard?.takeIf{it.isNotEmpty()}?.let{guards.put(key,it)}}else unaddressed.put(key,s.text)
   if(named&&key in paths){check(!s.readonly){"A selected source is not writable."};check(!s.guard.isNullOrEmpty()){"A selected source has no captured physical save guard. Reopen its saved baseline before preparing resources."};drafts.put(key,JSONObject().put("save_guard",s.guard).put("text",s.text).put("base_sha256",hash(s.base)).put("version",JSONObject.NULL).put("writable",true))}
  }
  val capture=JSONObject().put("context",nonce()).put("edit",edit).put("drafts",drafts).put("project_entry",JSONObject.NULL).put("ownership_sources",ownership).put("unaddressed_sources",unaddressed).put("saved_guards",guards)
  return capture to current
 }
</ui:copy>

### Replace discovery facts with acquired editor authority

An engine observation can identify an existing source that the local tree has
not listed yet. Refresh that index, load its available text without activating a
pane, and then replace the entire discovery capture. Never pass the discovering
source identities or saved facts into the final reviewed plan. A missing source
or a changed original editor refuses the whole acquisition.

<ui:copy id="source-resource-acquisition-android">
 internal suspend fun acquire(workspace:Workspace,paths:List<String>,edit:JSONObject,valid:()->Boolean):Triple<JSONObject,JSONObject,()->Boolean> {
  var (captured,current)=capture(workspace,paths,edit,valid)
  var inputs=call(workspace,"POST","/api/changes/resource-inputs",captured)
  check(current()){"Source changed during acquisition. Prepare again."}
  fun requireSavedGuards(raw:JSONObject){val guards=raw.getJSONObject("saved_guards");check(guards.keys().asSequence().all{guards.get(it) is String&&guards.getString(it).isNotEmpty()}){"Resource inputs contain malformed physical save guards."}}
  requireSavedGuards(inputs)
  val sources=inputs.getJSONObject("sources")
  val missing=paths.filter{sources.getJSONObject(it).getString("kind")=="file"&&workspace.buffers[it]==null}
  if(missing.isNotEmpty()){
   // Discovery observes saved existence; it never grants a write. Refresh the
   // file index and acquire those editors without changing the active pane,
   // then discard the discovery authority and capture all sources again.
   workspace.refresh();check(current()){"Workspace changed during source discovery. Prepare again."}
   val next=capture(workspace,paths,edit,valid);captured=next.first;current=next.second
   check(missing.all{workspace.buffers[it]!=null}){"Current source is not available in the file index: ${missing.joinToString()}. Refresh files and prepare again."}
   inputs=call(workspace,"POST","/api/changes/resource-inputs",captured);requireSavedGuards(inputs)
   check(current()){"Source changed after discovery. Prepare again."}
  }
  return Triple(captured,inputs,current)
 }
</ui:copy>

The Android sealed-proposal adapter reuses this same publisher rather than teaching an agent view how to adopt files. It installs immutable engine inputs, plan and consequences only after host admission. An optional read-only final preflight revalidates the whole seal before consuming the save action; the same editor lease and review object must still be current after that await. Ordinary reviews keep the default empty preflight.

### Review all ordered resource consequences

Keep the provider's ordered edit intact. The shared engine discovers targets,
plans against acquired drafts and computes both saved and draft construction
layers. Its exact final saved states carry permissions through source identity;
the Android host must not infer filesystem mode from its own platform. Generated
destinations refuse before a review can become writable.

<ui:copy id="source-resource-proposal-android">
 suspend fun prepare(workspace:Workspace,title:String,edit:JSONObject,valid:()->Boolean={true}) {
  if(busy||pending!=null){notice="Inspect the pending publication status before preparing another resource change.";return}
  val id=++ticket;busy=true;review=null;notice="Reading current resource authority…"
  try {
   val targets=call(workspace,"POST","/api/changes/resource-targets",JSONObject().put("edit",edit)).getJSONArray("paths");val paths=(0 until targets.length()).map{targets.getString(it)}
   val (capture,inputs,current)=acquire(workspace,paths,edit,valid)
   check(current()&&id==ticket){"Source changed during acquisition. Prepare again."}
   val plan=call(workspace,"POST","/api/changes/plan-resources",JSONObject().put("context",inputs.getString("context")).put("title",title).put("edit",edit).put("sources",inputs.getJSONObject("sources"))).getJSONObject("plan")
   val proposal=JSONObject().put("inputs",inputs).put("capture",capture).put("plan",plan)
   val facts=call(workspace,"POST","/api/changes/resource-consequences",proposal)
   check(current()&&id==ticket){"Source changed during analysis. Prepare again."}
   val patches=resourcePatches(plan);check(patches.all{it.before==null||workspace.buffers[it.path]?.text==it.before.text}){"Refresh files to acquire every existing source editor, then prepare again."}
   check(facts.getJSONArray("protected_targets").length()==0){"A proposed path becomes generated. Change its authored construction instead."}
   val expected=resourceSavedMap(facts.getJSONObject("saved_sources"))
   review=Review(proposal,facts,plan,expected,false,null,current,emptyMap());workspace.showRecovery=true;notice="Review source drafts, saved publication and generated consequences before Save resources."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
</ui:copy>

### Prepare inverse history without moving its cursor

A resource history entry names retained evidence, not permission to replay it.
Read that exact identity, compare its workspace and fingerprint, then ask the
engine to prepare current saved/draft restoration. Keep explicitly retained
deleted editor handles separate from ordinary path lookup. A workspace-history
revision closure guards this review; only confirmed publication transfers the
inverse recovery identity to the opposite stack.

<ui:copy id="source-resource-restoration-android">
 suspend fun openHistory(workspace:Workspace,record:String,expectedWorkspace:String,current:()->Boolean,published:(String,String)->Unit){
  if(busy||pending!=null){notice="Settle the pending resource change before workspace undo or redo.";return}
  val id=++ticket;busy=true
  try{
   check(current()){"Workspace history changed. Choose Undo or Redo again."}
   val reading=recoveryReading(call(workspace,"GET","/api/changes/resource-recovery?id="+URLEncoder.encode(record,"UTF-8")))
   check(id==ticket&&current()&&reading.id==record&&reading.workspace==expectedWorkspace){"Workspace history or recovery identity changed. Refresh recovery and choose Undo or Redo again."}
   busy=false;restore(workspace,reading,current,published)
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 suspend fun restore(workspace:Workspace,reading:RecoveryReading,valid:()->Boolean={true},published:((String,String)->Unit)?=null) {
  if(busy||pending!=null){notice="Inspect the pending publication status before preparing restoration.";return}
  val id=++ticket;busy=true;review=null;notice="Preparing saved baselines and drafts independently…"
  try {
   val paths=reading.paths.map{it.path};val changes=JSONObject();paths.forEach{changes.put(languageURI(it),org.json.JSONArray())}
   val (capture,inputs,current)=acquire(workspace,paths,JSONObject().put("changes",changes),valid)
   check(current()&&id==ticket){"Source changed during acquisition. Prepare restoration again."}
   val proposal=JSONObject().put("recovery_id",reading.id).put("inputs",inputs).put("capture",capture)
   val prepared=call(workspace,"POST","/api/changes/prepare-restoration",proposal);val restoration=prepared.getJSONObject("restoration");val facts=prepared.getJSONObject("consequences");val plan=restoration.getJSONObject("plan")
   check(current()&&id==ticket&&restoration.getString("recovery_fingerprint")==reading.fingerprint&&restoration.getString("workspace")==reading.workspace){"Recovery evidence or workspace changed. Refresh and prepare again."}
   check(facts.getJSONArray("protected_targets").length()==0){"Restored source becomes generated. Reconcile its authored construction first."}
   val patches=resourcePatches(plan);check(patches.all{it.before==null||workspace.buffers[it.path]?.text==it.before.text}){"Refresh files and acquire every current source before restoring."}
   val retained=patches.mapNotNull{it.after?.let{after->workspace.retainedResourceEditors[after.identity]?.let{buffer->after.identity to buffer}}}.toMap()
   val held=retained.mapValues{it.value.text};val guard={current()&&retained.all{(identity,buffer)->workspace.retainedResourceEditors[identity]===buffer&&buffer.text==held[identity]}}
   review=Review(proposal,facts,plan,resourceSavedMap(restoration.getJSONObject("saved_after")),true,reading.fingerprint,guard,retained,published);notice="Review the complete restoration. Saved baselines and unsaved drafts remain separate."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
</ui:copy>

### Publish once and admit only the original outcome

The Save button carries the displayed review identity. Consume that review as
soon as its POST begins; errors, cancellation and lost responses retain the same
request identity and never manufacture a retry. A later status read is GET only.
Before adopting tabs or baselines, validate the complete receipt and recheck the
captured editors. Partial results and independent edits keep local buffers intact
and direct the user to retained recovery evidence.

<ui:copy id="source-resource-publication-android">
 suspend fun apply(workspace:Workspace,expected:Review?=review) {
  val value=review ?: return;if(busy||pending!=null)return
  if(value!==expected){notice="Resource review changed. Read the current proposal before saving.";return}
  busy=true
  try{check(value.current()){"Source changed after review. Prepare again."};value.preflight?.invoke();check(review===value&&pending==null&&value.current()){ "Proposal or source changed during final admission. Nothing was sent." }}catch(error:CancellationException){throw error}catch(error:Exception){notice=error.message.orEmpty();return}finally{busy=false}
  val request=nonce();pending=request to value;publishedWorkspace=workspace;review=null;busy=true
  val body=JSONObject().put("request_id",request).put("reviewed_fingerprint",value.facts.getString("fingerprint")).put("proposal",value.proposal)
  if(value.restoration)body.put("reviewed_recovery_fingerprint",value.fingerprint)
  try{adopt(workspace,request,value,call(workspace,"POST",if(value.restoration)"/api/changes/restore-resources" else "/api/changes/resource-save",body))}
  catch(error:CancellationException){notice="Publication may have started. Inspect its status or Source recovery; do not repeat Save.";throw error}
  catch(error:Exception){notice="${error.message} Publication is unconfirmed. Inspect status or Source recovery; this action will not be retried."}
  finally{busy=false}
 }
 suspend fun status(workspace:Workspace){val (request,value)=pending ?: return;if(busy)return;busy=true
  try{check(publishedWorkspace===workspace){"Workspace changed. Inspect recovery in the original workspace."};adopt(workspace,request,value,call(workspace,"GET","/api/changes/resource-save?request_id=$request"))}
  catch(error:CancellationException){throw error}catch(error:Exception){notice=error.message.orEmpty()+" Inspect retained recovery before another publication."}finally{busy=false}
 }
 private fun adopt(workspace:Workspace,request:String,value:Review,raw:JSONObject){
  val receipt=resourcePublication(raw);val patches=resourcePatches(value.plan)
  workspace.adoptResourceReceipt(patches,receipt,request,value.facts.getString("workspace"),value.facts.getString("context"),value.saved,value.current,value.restoring)
  val recovery=receipt.recovery.substringBeforeLast('/').substringAfterLast('/')
  value.published?.invoke(recovery,receipt.workspace) ?: workspace.changes.recordResource(recovery,receipt.workspace)
  lastRecovery=recovery;lastWorkspace=receipt.workspace;pending=null;publishedWorkspace=null
  notice="Saved resources confirmed. Drafts and editor state retained. Restore this new recovery record to reverse the change; no checks were executed."
 }
 val hasPending get()=pending!=null
 val pendingRecovery get()=pending?.first
 val pendingWorkspace get()=pending?.second?.facts?.getString("workspace")
}
</ui:copy>

### Decode distinct namespaces and saved snapshots

Missing source is separate from an empty file. Decode draft source identities
from the reviewed plan and exact saved text/metadata from the engine; never use
the restored dirty draft as its saved baseline. Recovery reading retains its own
request lifetime, independent of publication and editor rendering.

<ui:copy id="source-resource-wire-android">
fun resourcePatches(plan:JSONObject):List<ResourceEditorPatch>{
 val rows=plan.getJSONArray("patches")
 fun side(value:JSONObject):ResourceSource? {require(value.getString("kind") in listOf("file","missing")){"Unsupported resource state. Prepare again."};return if(value.getString("kind")=="file")ResourceSource(value.getString("identity"),value.getString("text")) else null}
 return (0 until rows.length()).map{val row=rows.getJSONObject(it);ResourceEditorPatch(row.getString("path"),side(row.getJSONObject("before")),side(row.getJSONObject("after")))}
}
fun resourceSavedMap(raw:JSONObject)=raw.keys().asSequence().associateWith{key->val value=raw.getJSONObject(key);if(value.getString("kind")=="missing")ResourceSaved("missing") else ResourceSaved("file",value.getString("text"),value.getString("sha256"),value.getBoolean("readonly"),value.getBoolean("executable"),if(value.isNull("mode"))null else value.getInt("mode"))}
fun resourcePublication(raw:JSONObject):ResourcePublication<FileNode>{check(raw.get("version")==2){"Resource publication protocol2 is required for current editor adoption."};val outcome=raw.getJSONObject("outcome");val rows=raw.getJSONArray("source_files");val applied=outcome.getJSONArray("applied");return ResourcePublication(raw.getInt("version"),raw.getString("request_id"),raw.getString("workspace"),raw.getString("context"),outcome.getBoolean("complete"),outcome.getString("recovery"),(0 until applied.length()).map{applied.getString(it)},if(raw.isNull("saved_sources"))null else resourceSavedMap(raw.getJSONObject("saved_sources")),(0 until rows.length()).map{val row=rows.getJSONObject(it);val file=row.getJSONObject("file");ResourcePublishedFile(FileNode(file.getString("name"),file.getString("path"),file.getBoolean("dir"),emptyList(),if(file.isNull("doc_id"))null else file.getString("doc_id"),if(file.isNull("generated_by"))null else file.getString("generated_by"),if(file.isNull("language"))null else file.getString("language")),resourceSavedMap(JSONObject().put("entry",row.getJSONObject("saved"))).getValue("entry"),row.getString("hash"),if(row.isNull("save_guard"))null else row.getString("save_guard").also{check(it.isNotEmpty()){"Malformed source save guard."}})},raw.get("adoption_current") as? Boolean ?: error("Resource publication adoption authority is unavailable or malformed."),if(raw.isNull("adoption_reason"))null else raw.getString("adoption_reason"))}

data class RecoveryPage(val version:Int,val workspace:String,val ids:List<String>,val next:String?,val ignored:Int)
data class RecoverySummary(val kind:String,val bytes:Int?,val sha256:String?)
data class RecoveryPath(val path:String,val snapshots:Map<String,RecoverySummary>)
data class RecoveryReading(val version:Int,val workspace:String,val id:String,val fingerprint:String,val title:String,val titleTruncated:Boolean,val paths:List<RecoveryPath>,val allApplied:Boolean,val unrecognized:Int,val draftText:String?,val savedText:String?)
private fun JSONObject.recoveryText(key:String)=if(isNull(key))null else getString(key)
fun recoveryPage(raw:JSONObject):RecoveryPage {val ids=raw.getJSONArray("ids");return RecoveryPage(raw.getInt("version"),raw.getString("workspace"),(0 until ids.length()).map{ids.getString(it)},raw.recoveryText("next"),raw.getInt("ignored"))}
fun recoveryReading(raw:JSONObject):RecoveryReading {val paths=raw.getJSONArray("paths");return RecoveryReading(raw.getInt("version"),raw.getString("workspace"),raw.getString("id"),raw.getString("fingerprint"),raw.getString("title"),raw.getBoolean("title_truncated"),(0 until paths.length()).map{val row=paths.getJSONObject(it);RecoveryPath(row.getString("path"),listOf("saved_before","saved_after","draft_before","draft_after").associateWith{key->val value=row.getJSONObject(key);RecoverySummary(value.getString("kind"),if(value.has("bytes"))value.getInt("bytes") else null,value.recoveryText("sha256"))})},raw.getBoolean("all_paths_recorded_applied"),raw.getJSONArray("unrecognized_files").length(),raw.recoveryText("draft_text"),raw.recoveryText("saved_text"))}
class RecoveryReader {
 var page by mutableStateOf<RecoveryPage?>(null);var value by mutableStateOf<RecoveryReading?>(null);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0L
 fun close(){ticket++;loading=false;page=null;value=null}
 suspend fun list(after:String?=null,load:suspend(String)->RecoveryPage){val id=++ticket;loading=true;value=null;notice="Reading retained recovery identities…"
  try{val next=load("/api/changes/resource-recoveries"+(after?.let{"?after="+URLEncoder.encode(it,"UTF-8")} ?: ""));if(id!=ticket)return
   check(next.version==1&&next.workspace.isNotEmpty()&&next.ids.size<=100&&next.ids.all{it.matches(Regex("[0-9a-f]{32}"))}&&next.ids.zipWithNext().all{it.first<it.second}&&(after==null||next.ids.all{it>after}&&next.workspace==page?.workspace)&&(next.next==null||next.next==next.ids.lastOrNull())){"Recovery page identity changed. Refresh from the beginning."}
   page=next;notice=if(next.ids.isEmpty())"No retained resource recovery records." else "Choose a retained record. Identities are sorted, not chronological."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket){page=null;notice=error.message.orEmpty()}}finally{if(id==ticket)loading=false}
 }
 suspend fun read(record:String,path:String?=null,snapshot:String="draft_before",expectedWorkspace:String?=null,load:suspend(String)->RecoveryReading){val workspace=expectedWorkspace ?: page?.workspace;val previous=value?.takeIf{it.id==record};val id=++ticket;loading=true;value=null;notice="Reading retained source evidence…"
  try{check(workspace!=null&&record.matches(Regex("[0-9a-f]{32}"))&&(expectedWorkspace!=null||page?.ids?.contains(record)==true||previous?.workspace==workspace)){"Refresh recovery identities before selecting a record."};val parts=snapshot.split('_');check(parts.size==2&&parts[0] in listOf("saved","draft")&&parts[1] in listOf("before","after")){"Choose a supported recovery snapshot."}
   val reply=load("/api/changes/resource-recovery?id="+URLEncoder.encode(record,"UTF-8")+(path?.let{"&path="+URLEncoder.encode(it,"UTF-8")+"&"+parts[0]+"="+parts[1]} ?: ""));if(id!=ticket)return
   check(reply.version==1&&reply.workspace==workspace&&reply.id==record&&reply.fingerprint.matches(Regex("[0-9a-f]{64}"))&&(previous==null||previous.fingerprint==reply.fingerprint)&&(path==null||reply.paths.any{it.path==path})){"Recovery evidence changed. Refresh identities and read again."}
   value=reply;notice="Retained evidence only. Current source and editor drafts are unchanged."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)loading=false}
 }
}
</ui:copy>

## android 2

Render the selected retained evidence in a floating reading, reached through existing menu commands. Keep refresh, paging and snapshot controls inside this reading, with selectable abbreviated text and explicit absence. Closing returns focus to the existing source editor.

<ui:copy id="source-recovery-android-1">
@Composable fun SourceRecoveryView(workspace:Workspace){
 val model=remember{RecoveryReader()};val resources=workspace.resourceChanges;val scope=rememberCoroutineScope();val engine=workspace.engine;var path by remember{mutableStateOf("")};var snapshot by remember{mutableStateOf("draft_before")};var recordMenu by remember{mutableStateOf(false)};var pathMenu by remember{mutableStateOf(false)}
 var reviewFile by remember(resources.review){mutableStateOf(0)}
 suspend fun list(after:String?=null){if(engine==null){model.notice="Connect to a workspace server first.";return};path="";model.list(after){recoveryPage(engine.request("GET",it))}}
 suspend fun read(id:String,path:String?=null,snapshot:String="draft_before",expectedWorkspace:String?=null){if(engine==null)return;model.read(id,path,snapshot,expectedWorkspace ?: model.value?.takeIf{it.id==id}?.workspace){recoveryReading(engine.request("GET",it))}}
 LaunchedEffect(engine){list()};DisposableEffect(model){onDispose{model.close();resources.dismiss()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Source recovery",style=MaterialTheme.typography.titleMedium);Row{TextButton(enabled=!workspace.resourceChanges.busy&&!workspace.resourceChanges.hasPending,onClick={workspace.showRecovery=false;workspace.editorRecovery.showing=true}){Text("Editor sessions…")};TextButton(enabled=!model.loading,onClick={scope.launch{list()}}){Text("Refresh records")};TextButton(onClick={workspace.showRecovery=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  if(resources.notice.isNotEmpty())Text(resources.notice,fontSize=12.sp)
  if(resources.hasPending)TextButton(enabled=!resources.busy,onClick={scope.launch{resources.status(workspace)}}){Text("Inspect original publication status")}
  resources.pendingRecovery?.let{record->SelectionContainer{Text("Original publication: $record",fontSize=12.sp)};TextButton(enabled=!resources.busy,onClick={scope.launch{read(record,expectedWorkspace=resources.pendingWorkspace)}}){Text("Inspect interrupted request recovery")}}
  resources.lastRecovery?.let{record->SelectionContainer{Text("Latest recovery: $record",fontSize=12.sp)};TextButton(enabled=!resources.busy,onClick={scope.launch{read(record,expectedWorkspace=resources.lastWorkspace)}}){Text("Review inverse / redo from latest recovery")}}
  resources.review?.let{review->
   Text(review.plan.getString("title"),style=MaterialTheme.typography.titleSmall)
   Text("Save publishes the reviewed saved states. Draft text can differ from its saved baseline. Generated output is analyzed, never materialized by this action.",fontSize=12.sp)
   val patches=resourcePatches(review.plan)
   Row{TextButton(enabled=reviewFile>0,onClick={reviewFile--}){Text("Previous source")};Text("${reviewFile+1} / ${patches.size}",fontSize=12.sp);TextButton(enabled=reviewFile+1<patches.size,onClick={reviewFile++}){Text("Next source")}}
   patches.getOrNull(reviewFile)?.let{patch->
    Text(patch.path,style=MaterialTheme.typography.titleSmall)
    listOf("Current draft" to patch.before?.text,"Reviewed draft" to patch.after?.text,"Reviewed saved baseline" to review.saved[patch.path]?.text).forEach{(label,text)->Text(label,fontSize=12.sp);SelectionContainer{Text(text?.take(16000) ?: "Source absent",fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(text!=null&&text.length>16000)Text("First 16,000 characters shown.",fontSize=12.sp)}
   }
   listOf("saved","draft").forEach{layer->val outputs=review.facts.getJSONObject(layer).getJSONArray("output_paths");Text("$layer construction consequences: ${outputs.length()} outputs; execution evidence not queried.",fontSize=12.sp);SelectionContainer{Text((0 until minOf(outputs.length(),300)).joinToString("\n"){outputs.getString(it)},fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(outputs.length()>300)Text("First 300 output paths shown.",fontSize=12.sp)}
   Row{TextButton(enabled=!resources.busy,onClick={scope.launch{resources.apply(workspace,review)}}){Text(if(review.restoration)"Save reviewed restoration" else "Save resources")};TextButton(enabled=!resources.busy,onClick={resources.dismiss()}){Text("Discard review")}}
  }
  model.page?.let{page->Box{TextButton(enabled=!model.loading,onClick={recordMenu=true}){Text(model.value?.id ?: "Choose retained record")};DropdownMenu(expanded=recordMenu,onDismissRequest={recordMenu=false}){page.ids.forEach{id->DropdownMenuItem(text={Text(id)},onClick={recordMenu=false;path="";scope.launch{read(id)}})}}};if(page.ignored>0)Text("${page.ignored} unrecognized entries omitted.",fontSize=12.sp);page.next?.let{next->TextButton(enabled=!model.loading,onClick={scope.launch{list(next)}}){Text("Next 100 identities")}}}
  model.value?.let{value->Text(value.title+if(value.titleTruncated)"…" else "",style=MaterialTheme.typography.titleSmall);Text((if(value.allApplied)"Every path has an applied receipt." else "The record does not confirm every path was applied.")+" These receipts describe the recorded operation, not current disk state.",fontSize=12.sp);if(value.unrecognized>0)Text("${value.unrecognized} unrecognized files in this record.",fontSize=12.sp)
   TextButton(enabled=!model.loading&&!resources.busy&&!resources.hasPending,onClick={scope.launch{resources.restore(workspace,value)}}){Text("Prepare complete saved / draft restoration")}
   Box{TextButton(enabled=!model.loading,onClick={pathMenu=true}){Text(path.ifEmpty{"Choose recorded source"})};DropdownMenu(expanded=pathMenu,onDismissRequest={pathMenu=false}){value.paths.forEach{item->DropdownMenuItem(text={Text(item.path)},onClick={pathMenu=false;path=item.path;scope.launch{read(value.id,path,snapshot)}})}}}
   listOf("saved_before","draft_before","saved_after","draft_after").forEach{side->TextButton(enabled=!model.loading,onClick={snapshot=side;if(path.isNotEmpty())scope.launch{read(value.id,path,side)}}){Text((if(snapshot==side)"✓ " else "")+side.replace('_',' '))}}
   value.paths.find{it.path==path}?.snapshots?.get(snapshot)?.let{selected->SelectionContainer{Text(if(selected.kind=="missing")"Source absent in this snapshot." else "${selected.bytes} UTF-8 bytes · ${selected.sha256}",fontSize=12.sp)};val text=if(snapshot.startsWith("saved"))value.savedText else value.draftText;if(text!=null){SelectionContainer{Text(text.take(32000),fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(text.length>32000)Text("First 32,000 characters shown.",fontSize=12.sp)}}
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceRecovery.kt"><ui:paste select="#source-recovery-android-0"/><ui:paste select="#source-resource-capture-android"/><ui:paste select="#source-resource-acquisition-android"/><ui:paste select="#source-resource-proposal-android"/><ui:paste select="#source-resource-restoration-android"/><ui:paste select="#source-resource-publication-android"/><ui:paste select="#source-resource-wire-android"/><ui:paste select="#source-recovery-android-1"/></ui:file>

## browser-test 1

These focused cases exercise a delayed response after refresh/close, changed evidence and workspace refusal, exact encoded source paths, and empty saved text alongside a dirty draft. They do not establish physical input or accessibility acceptance.

<ui:copy id="source-recovery-browser-test-0">
import {it,expect} from 'vitest';
import {RecoveryReader,type RecoveryPage,type RecoveryReading} from './SourceRecovery';
const id='a'.repeat(32),workspace='b'.repeat(32);
const page=():RecoveryPage=>({version:1,workspace,ids:[id],next:null,ignored:0});
const reading=(fingerprint='c'.repeat(64)):RecoveryReading=>({version:1,workspace,id,fingerprint,title:'Move draft',title_truncated:false,paths:[{path:'a #😀.md',saved_before:{kind:'file',bytes:0},draft_before:{kind:'file',bytes:2},saved_after:{kind:'missing'},draft_after:{kind:'missing'}}],all_paths_recorded_applied:false,unrecognized_files:[],draft_text:'😀',saved_text:''});
it('recovery reads retain empty saved text separately from dirty drafts and encode the exact path',async()=>{
 const routes:string[]=[];const model=new RecoveryReader(async<T>(route:string)=>{routes.push(route);return (route.includes('resource-recoveries')?page():reading()) as T;});await model.list();await model.read(id,'a #😀.md','saved_before');expect(model.value?.saved_text).toBe('');expect(model.value?.draft_text).toBe('😀');expect(routes.at(-1)).toContain('path=a%20%23%F0%9F%98%80.md&saved=before');
});
it('changed record evidence and workspace pages refuse; closed and superseded replies cannot replace selection',async()=>{
 let response:unknown=page();const model=new RecoveryReader(async<T>()=>response as T);await model.list();response=reading();await model.read(id);response=reading('d'.repeat(64));await model.read(id,'a #😀.md');expect(model.value).toBeNull();expect(model.notice).toContain('evidence changed');
 response=page();await model.list();response={...page(),workspace:'other'};await model.list(id);expect(model.page).toBeNull();
 let finish!:(value:unknown)=>void;let defer=false;const delayed=new RecoveryReader(async<T>(route:string)=>(defer?await new Promise<unknown>(resolve=>finish=resolve):route.includes('resource-recoveries')?page():reading()) as T);await delayed.list();defer=true;const old=delayed.read(id);defer=false;await delayed.list();finish(reading());await old;expect(delayed.value).toBeNull();defer=true;const closed=delayed.read(id);delayed.close();finish(reading());await closed;expect(delayed.value).toBeNull();expect(delayed.page).toBeNull();
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceRecovery.test.ts"><ui:paste select="#source-recovery-browser-test-0"/></ui:file>

## apple-test 1

These focused cases exercise a delayed response after refresh/close, changed evidence and workspace refusal, exact encoded source paths, and empty saved text alongside a dirty draft. They do not establish physical input or accessibility acceptance.

<ui:copy id="source-recovery-apple-test-0">
import XCTest
@testable import HickorySimple
@MainActor final class SourceRecoveryTests:XCTestCase {
 let record=String(repeating:"a",count:32),workspace=String(repeating:"b",count:32)
 func page()->RecoveryPage{RecoveryPage(version:1,workspace:workspace,ids:[record],ignored:0)}
 func reading(_ hash:String=String(repeating:"c",count:64))->RecoveryReading{RecoveryReading(version:1,workspace:workspace,id:record,fingerprint:hash,title:"Move",title_truncated:false,paths:[RecoveryPath(path:"a #😀.md",saved_before:RecoverySummary(kind:"file",bytes:0),saved_after:RecoverySummary(kind:"missing"),draft_before:RecoverySummary(kind:"file",bytes:4),draft_after:RecoverySummary(kind:"missing"))],all_paths_recorded_applied:false,unrecognized_files:[],draft_text:"😀",saved_text:"")}
 func testExactSnapshotRequestsAndChangedEvidence()async{
  let model=RecoveryReader();await model.list{_ in self.page()};var route="";await model.read(record,path:"a #😀.md",snapshot:"saved_before"){route=$0;return self.reading()};XCTAssertTrue(route.contains("path=a%20%23%F0%9F%98%80.md&saved=before"));XCTAssertEqual(model.value?.saved_text,"");XCTAssertEqual(model.value?.draft_text,"😀")
  await model.read(record,path:"a #😀.md"){_ in self.reading(String(repeating:"d",count:64))};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("evidence changed"))
  await model.list{_ in self.page()};await model.list(after:record){_ in var page=self.page();page.workspace="other";page.ids=[];return page};XCTAssertNil(model.page)
 }
 func testClosedAndSupersededReadsCannotReplaceSelection()async{
  let model=RecoveryReader();await model.list{_ in self.page()};var pending:CheckedContinuation<RecoveryReading,Never>?
  let old=Task{await model.read(record){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.list{_ in self.page()};pending?.resume(returning:reading());await old.value;XCTAssertNil(model.value)
  pending=nil;let closed=Task{await model.read(record){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:reading());await closed.value;XCTAssertNil(model.value);XCTAssertNil(model.page)
 }
 func testRecoveryIdentityUsesExactManifestAddress()throws {
  XCTAssertEqual(try resourceRecoveryIdentity(".hick-cache/resource-changes/"+record+"/manifest.json"),record)
  for value in [record,"../.hick-cache/resource-changes/"+record+"/manifest.json",".hick-cache/resource-changes/"+record+"/../manifest.json",".hick-cache/resource-changes/"+record.uppercased()+"/manifest.json"] {XCTAssertThrowsError(try resourceRecoveryIdentity(value))}
 }
 private func json(_ value:Any)throws->ResourceJSON {try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:value))}
 private func publicationFixture(failWrite:Bool=false)throws->(Workspace,ResourceRestoration,RecoveryReading,()->Int) {
  let space=Workspace();space.engine=try Engine(address:"http://127.0.0.1:1")
  let buffer=DocumentBuffer(file:FileNode(name:"b.py",path:"b.py",dir:false),text:FileText(path:"b.py",content:"dirty",hash:"old",save_guard:"original-b"));buffer.selection=NSRange(location:2,length:0);space.buffers["b.py"]=buffer;space.layout.activate("b.py",in:"document-group")
  let model=space.resourceRestoration,missing:[String:Any]=["kind":"missing"],source:[String:Any]=["kind":"file","identity":"source","text":"dirty","writable":true,"removable":true,"version":NSNull()],after:[String:Any]=["kind":"file","identity":"source","text":"prior dirty","writable":true,"removable":true,"version":NSNull()],absent:[String:Any]=["kind":"missing","creatable":true]
  func saved(_ text:String)->[String:Any] {["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":NSNull()]}
  let summary=RecoverySummary(kind:"file",bytes:5),recordValue=RecoveryReading(version:1,workspace:workspace,id:record,fingerprint:String(repeating:"c",count:64),title:"Move",title_truncated:false,paths:[RecoveryPath(path:"a.py",saved_before:summary,saved_after:RecoverySummary(kind:"missing"),draft_before:summary,draft_after:RecoverySummary(kind:"missing")),RecoveryPath(path:"b.py",saved_before:RecoverySummary(kind:"missing"),saved_after:summary,draft_before:RecoverySummary(kind:"missing"),draft_after:summary)],all_paths_recorded_applied:true,unrecognized_files:[])
  var context="",writes=0,request=""
  model.transport={method,route,body in
   if route=="/api/changes/resource-inputs" {
    context=body!["context"] as! String
    let drafts=body!["drafts"] as! [String:[String:Any]];XCTAssertEqual(drafts["b.py"]?["base_sha256"] as? String,nativeSourceHash("dirty"))
    return try self.json(["version":1,"context":context,"workspace":self.workspace,"ownership":["mode":"ordinary","entry":NSNull(),"protected_entries":[],"saved_sources":[:],"draft_sources":[:]],"sources":["a.py":absent,"b.py":source],"saved_sources":["a.py":missing,"b.py":saved("dirty")],"refusals":[:],"source_files_modified":false,"saved_guards":["a.py":"original-absence","b.py":"original-b"]])
   }
   if route=="/api/changes/prepare-restoration" {
    let plan:[String:Any]=["version":1,"context":context,"title":"Restore Move","patches":[["path":"a.py","before":absent,"after":after],["path":"b.py","before":source,"after":absent]],"annotations":[]]
    let layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]]
    return try self.json(["version":1,"restoration":["version":1,"recovery_id":self.record,"recovery_fingerprint":String(repeating:"c",count:64),"workspace":self.workspace,"context":context,"plan":plan,"saved_before":["a.py":missing,"b.py":saved("dirty")],"saved_after":["a.py":saved("saved"),"b.py":missing]],"consequences":["version":1,"workspace":self.workspace,"context":context,"fingerprint":String(repeating:"d",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false]])
   }
   if method=="POST",route=="/api/changes/restore-resources" {writes+=1;request=body!["request_id"] as! String;if failWrite{throw EngineError(message:"Lost response")}}
   guard route=="/api/changes/restore-resources" || route=="/api/changes/resource-save?request_id="+request else{throw EngineError(message:"Unexpected resource request")}
   return try self.json(["version":2,"adoption_current":true,"request_id":request,"workspace":self.workspace,"context":context,"outcome":["complete":true,"recovery":".hick-cache/resource-changes/"+String(repeating:"e",count:32)+"/manifest.json","applied":["a.py","b.py"]],"saved_sources":["a.py":saved("saved"),"b.py":missing],"source_files":[["file":["name":"a.py","path":"a.py","dir":false],"saved":saved("saved"),"hash":"new"]]])
  }
  return (space,model,recordValue,{writes})
 }
 func testLegacyInputsWithoutPhysicalGuardsCannotAuthorizePublication()async throws {
  let (space,model,value,writes)=try publicationFixture(),original=model.transport!
  model.transport={method,route,body in let result=try await original(method,route,body);if route=="/api/changes/resource-inputs",case .object(var raw)=result{raw["saved_guards"]=nil;return .object(raw)};return result}
  await model.prepare(space,record:value);XCTAssertNil(model.review);await model.publish(space);XCTAssertEqual(writes(),0);XCTAssertEqual(space.currentBuffer?.text,"dirty");XCTAssertTrue(model.notice.contains("changed"))
 }
 func testRestorationPublishesSeparateSavedAndDirtyDraftThroughLiveNamespace()async throws {
  let (space,model,value,writes)=try publicationFixture();let buffer=space.currentBuffer!
  await model.prepare(space,record:value);XCTAssertNotNil(model.review,model.notice);XCTAssertEqual(writes(),0)
  await model.publish(space);XCTAssertEqual(writes(),1);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(space.active,"a.py")
  XCTAssertEqual(buffer.text,"prior dirty");XCTAssertEqual(buffer.base,"saved");XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertEqual(model.lastRecovery,String(repeating:"e",count:32));buffer.language?.close()
 }
 func testNewOwnershipDraftRetiresReviewAndLostPublicationCannotReplay()async throws {
  let (space,model,value,writes)=try publicationFixture();await model.prepare(space,record:value);XCTAssertNotNil(model.review,model.notice)
  space.buffers["new.md"]=DocumentBuffer(file:FileNode(name:"new.md",path:"new.md",dir:false),text:FileText(path:"new.md",content:"",hash:""));space.buffers["new.md"]!.text="new ownership"
  await model.publish(space);XCTAssertEqual(writes(),0);XCTAssertNil(model.review);XCTAssertTrue(model.notice.contains("Source changed"))
  let (other,pending,reading,attempts)=try publicationFixture(failWrite:true);await pending.prepare(other,record:reading);await pending.publish(other);XCTAssertTrue(pending.hasPending);XCTAssertEqual(attempts(),1)
  await pending.publish(other);await pending.prepare(other,record:reading);XCTAssertEqual(attempts(),1);XCTAssertNil(pending.review)
  await pending.status(other);XCTAssertFalse(pending.hasPending,pending.notice);XCTAssertEqual(attempts(),1);XCTAssertEqual(other.active,"a.py");other.currentBuffer?.language?.close()
 }

 private func resourceSaveFixture()throws->(Workspace,ResourceRestoration,()->Int) {
  let (space,model,_,_)=try publicationFixture();let initial=model.transport!;var writes=0
  func saved(_ text:String)->[String:Any] {["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":NSNull()]}
  model.transport={method,route,body in
   if route=="/api/changes/resource-targets" {return try self.json(["version":1,"paths":["a.py","b.py"],"acquired":false,"saved":false])}
   if route=="/api/changes/plan-resources" {
    let sources=body!["sources"] as! [String:Any]
    return try self.json(["plan":["version":1,"context":body!["context"]!,"title":body!["title"]!,"patches":[["path":"a.py","before":sources["a.py"]!,"after":sources["b.py"]!],["path":"b.py","before":sources["b.py"]!,"after":sources["a.py"]!]],"annotations":[]]])
   }
   if route=="/api/changes/resource-consequences" {
    let capture=body!["capture"] as! [String:Any],layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]]
    return try self.json(["version":1,"workspace":self.workspace,"context":capture["context"]!,"fingerprint":String(repeating:"d",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false,"saved_sources":["a.py":saved("dirty"),"b.py":["kind":"missing"]]])
   }
   if route=="/api/changes/resource-save",method=="POST" {
    writes+=1;let proposal=body!["proposal"] as! [String:Any],capture=proposal["capture"] as! [String:Any]
    XCTAssertNil(body!["reviewed_recovery_fingerprint"])
    return try self.json(["version":2,"adoption_current":true,"request_id":body!["request_id"]!,"workspace":self.workspace,"context":capture["context"]!,"outcome":["complete":true,"recovery":".hick-cache/resource-changes/"+String(repeating:"f",count:32)+"/manifest.json","applied":["a.py","b.py"]],"saved_sources":["a.py":saved("dirty"),"b.py":["kind":"missing"]],"source_files":[["file":["name":"a.py","path":"a.py","dir":false],"saved":saved("dirty"),"hash":"new"]]])
   }
   return try await initial(method,route,body)
  }
  return (space,model,{writes})
 }
 func testProviderResourceActionUsesWholeReviewAndConfirmedSave()async throws {
  let (space,model,writes)=try resourceSaveFixture();let buffer=space.currentBuffer!,engine=space.engine!;buffer.language=NativeLanguage(engine:engine,buffer:buffer)
  let edit:[String:Any]=["documentChanges":[["kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///a.py"]]]
  space.changes.open("actions",workspace:space)
  await space.changes.prepare(space,proposal:{_,_ in [["title":"Move source","kind":"refactor.move"]]},resolution:{_ in ["title":"Move source","kind":"refactor.move","edit":edit]})
  await space.changes.choose(0,workspace:space)
  XCTAssertNotNil(model.review,model.notice);XCTAssertNotNil(model.review?.save);XCTAssertNil(space.changes.plan);XCTAssertEqual(writes(),0);XCTAssertEqual(space.active,"b.py")
  await model.publish(space);XCTAssertEqual(writes(),1);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(space.active,"a.py");XCTAssertEqual(buffer.text,"dirty");XCTAssertEqual(buffer.base,"dirty");XCTAssertFalse(buffer.dirty)
  XCTAssertEqual(space.changes.resourceHistoryRecord("undo"),String(repeating:"f",count:32));buffer.language?.close()
 }
 func testMixedDraftAndResourceHistoryKeepsOrderingAndRetiresStaleIntent()async throws {
  let (space,model,_,_)=try publicationFixture();let buffer=space.currentBuffer!
  let edit:[String:Any]=["changes":["hick:///b.py":[["range":["start":["line":0,"character":0],"end":["line":0,"character":0]],"newText":"X"]]]]
  try space.changes.offer("Prefix",edit:edit,expected:["b.py":"dirty"],workspace:space);space.changes.apply(space);XCTAssertEqual(buffer.text,"Xdirty")
  space.changes.recordResourcePublication(record,intent:nil)
  let intent=ResourceHistoryIntent(direction:"undo",record:record,token:space.changes.historySnapshot)
  space.changes.recordResourcePublication(String(repeating:"e",count:32),intent:intent)
  XCTAssertNil(space.changes.resourceHistoryRecord("undo"));XCTAssertEqual(space.changes.resourceHistoryRecord("redo"),String(repeating:"e",count:32))
  space.changes.open("undo",workspace:space);XCTAssertEqual(buffer.text,"dirty");XCTAssertNil(space.changes.resourceHistoryRecord("redo"))
  space.changes.open("redo",workspace:space);XCTAssertEqual(buffer.text,"Xdirty");XCTAssertEqual(space.changes.resourceHistoryRecord("redo"),String(repeating:"e",count:32))
  await model.prepareHistory(space,intent:intent);XCTAssertNil(model.review);XCTAssertTrue(model.notice.contains("history changed"))
 }

 func testActualEngineResourceSaveRestorationAndRedoPreserveDirtyBaseline()async throws {
  #if os(macOS)
  let repository=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
  let executable=ProcessInfo.processInfo.environment["HICKORY_SIMPLE_TEST_ENGINE"] ?? repository.appendingPathComponent("target/source-edit-focus/debug/examples/engine_client").path
  guard FileManager.default.isExecutableFile(atPath:executable) else{throw XCTSkip("Build the focused engine_client fixture before native resource protocol acceptance.")}
  let temporary=FileManager.default.temporaryDirectory.appendingPathComponent("apple-resource-"+UUID().uuidString),root=temporary.appendingPathComponent("workspace")
  try FileManager.default.createDirectory(at:root,withIntermediateDirectories:true);defer{try? FileManager.default.removeItem(at:temporary)}
  let saved="saved baseline\n",draft=saved+"# retained draft\n";try Data(saved.utf8).write(to:root.appendingPathComponent("a.py"))
  let process=Process(),output=Pipe();process.executableURL=URL(fileURLWithPath:executable);process.arguments=[root.path,"window","http://127.0.0.1:1"];process.standardOutput=output;process.standardError=FileHandle.nullDevice
  var environment=ProcessInfo.processInfo.environment;environment["HICKORY_STATE_DIR"]=temporary.appendingPathComponent("state").path;process.environment=environment
  try process.run();defer{if process.isRunning{process.terminate()};process.waitUntilExit()}
  let greeting=try XCTUnwrap(String(data:output.fileHandleForReading.availableData,encoding:.utf8)),range=try XCTUnwrap(greeting.range(of:"http://127\\.0\\.0\\.1:[0-9]+",options:.regularExpression))
  let engine=try Engine(address:String(greeting[range])),space=Workspace();space.engine=engine;space.rootFiles=try await engine.files().tree
  let buffer=try await space.loadBuffer(FileNode(name:"a.py",path:"a.py",dir:false));buffer.text=draft;buffer.selection=NSRange(location:2,length:0);space.layout.activate("a.py",in:"document-group")
  defer{buffer.language?.close()}
  let model=space.resourceRestoration,geometry=space.layout.nodes["root"]
  await model.prepareEdit(space,title:"Move retained source",edit:["documentChanges":[["kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"]]])
  XCTAssertNotNil(model.review,model.notice);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("a.py"),encoding:.utf8),saved);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("b.py").path))
  await model.publish(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(space.active,"b.py");XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,draft)
  XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("b.py"),encoding:.utf8),draft);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("a.py").path))
  await model.prepareInverse(space);XCTAssertNotNil(model.review,model.notice);await model.publish(space)
  XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(space.active,"a.py");XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved);XCTAssertTrue(buffer.dirty)
  XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("a.py"),encoding:.utf8),saved);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("b.py").path))
  await model.prepareInverse(space);XCTAssertNotNil(model.review,model.notice);await model.publish(space)
  XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(space.active,"b.py");XCTAssertEqual(buffer.base,draft);XCTAssertFalse(buffer.dirty)
  XCTAssertEqual(space.layout.nodes["root"],geometry);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0))
  await model.prepareInverse(space);XCTAssertNotNil(model.review,model.notice);buffer.text += "# newer local draft\n";await model.publish(space)
  XCTAssertFalse(model.hasPending);XCTAssertNil(model.review);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("b.py"),encoding:.utf8),draft);XCTAssertTrue(buffer.text.hasSuffix("# newer local draft\n"))
  #endif
 }

 func testMalformedRecoveryAddressRetainsConfirmedReceiptWithoutAdoptionOrRetry()async throws {
  let (space,model,reading,writes)=try publicationFixture(),original=model.transport!
  model.transport={method,route,body in
   let result=try await original(method,route,body)
   if method=="POST",route=="/api/changes/restore-resources",case .object(var value)=result,case .object(var outcome)?=value["outcome"] {outcome["recovery"] = .string("unexpected-record-address");value["outcome"] = .object(outcome);return .object(value)}
   return result
  }
  let buffer=space.currentBuffer!
  await model.prepare(space,record:reading);await model.publish(space)
  XCTAssertTrue(model.hasPending);XCTAssertNotNil(model.confirmedReceipt);XCTAssertEqual(writes(),1);XCTAssertEqual(space.active,"b.py");XCTAssertTrue(space.currentBuffer === buffer);XCTAssertTrue(model.notice.contains("Saved publication is confirmed"))
  await model.publish(space);XCTAssertEqual(writes(),1)
  await model.status(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(writes(),1);XCTAssertEqual(space.active,"a.py");buffer.language?.close()
 }
 func testResourceActionWithProviderCommandRefusesWholeAction()async throws {
  let (space,model,writes)=try resourceSaveFixture();let buffer=space.currentBuffer!;buffer.language=NativeLanguage(engine:space.engine!,buffer:buffer)
  let edit:[String:Any]=["documentChanges":[["kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///a.py"]]]
  space.changes.open("actions",workspace:space)
  await space.changes.prepare(space,proposal:{_,_ in [["title":"Move and command","kind":"refactor.move"]]},resolution:{_ in ["title":"Move and command","edit":edit,"command":["command":"unsupported"]]})
  await space.changes.choose(0,workspace:space)
  XCTAssertNil(model.review);XCTAssertNil(space.changes.plan);XCTAssertEqual(writes(),0);XCTAssertEqual(space.active,"b.py");XCTAssertEqual(buffer.text,"dirty");XCTAssertTrue(space.changes.notice.contains("no edit portion"));buffer.language?.close()
 }

 func testDetachedSaveAsRetainsFailureAndRetiresOnlyCopiedDirtyHandle()async throws {
  let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
  let space=Workspace();space.engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config))
  let source=DocumentBuffer(file:FileNode(name:"reused.py",path:"reused.py",dir:false),text:FileText(path:"reused.py",content:"saved",hash:"old"));source.text="dirty retained text";source.resourceDeleted=true
  let occupant=DocumentBuffer(file:source.file,text:FileText(path:"reused.py",content:"independent source",hash:"independent"))
  space.buffers["reused.py"]=occupant;space.retainedResourceEditors["deleted"]=source
  let retainedTab=space.retainedResourceTab(source);space.layout.activate("reused.py",in:"document-group");space.layout.activate(retainedTab,in:"document-group")
  let submitted=source.text
  space.sourceRootAddress="/fixture";space.sourceWorkspaceID="runtime-workspace"
  var refuse=true
  ResponseProtocol.beforeResponse={request in
   ResponseProtocol.status=refuse ? 409:200
   let value:[String:Any]
   if request.url?.path=="/api/files" {ResponseProtocol.status=200;value=["root":"fixture","root_path":"/fixture","workspace_id":"runtime-workspace","tree":[]]}
   else if request.url?.path != "/api/source-history/create" {ResponseProtocol.status=200;value=[:]}
   else if refuse{value=["error":"Destination unavailable"]}
   else{value=["version":1,"history_protocol":1,"workspace":"runtime-workspace","namespace":"/fixture","path":"copy.py","source":submitted,"source_hash":nativeSourceHash(submitted),"hash":"saved","request_id":source.creationAttempt?.id ?? "missing-attempt","save_guard":"confirmed-copy","history":["available":true,"outcome":"published"]]}
   ResponseProtocol.response=try! JSONSerialization.data(withJSONObject:value)
  }
  defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];space.currentBuffer?.language?.close()}
  space.fileCommand="Save As";space.filePath="copy.py";await space.performFileCommand()
  XCTAssertTrue(space.retainedResourceEditors["deleted"] === source);XCTAssertTrue(source.dirty);XCTAssertEqual(space.active,retainedTab);XCTAssertTrue(space.buffers["reused.py"] === occupant);XCTAssertNil(space.buffers["copy.py"])
  refuse=false;space.fileCommand="Save As";space.filePath="copy.py";await space.performFileCommand()
  XCTAssertTrue(space.retainedResourceEditors.isEmpty,space.status);XCTAssertFalse(space.layout.groups().contains{$0.tabs.contains(retainedTab)});XCTAssertTrue(space.buffers["reused.py"] === occupant)
  XCTAssertEqual(space.currentBuffer?.file.path,"copy.py");XCTAssertEqual(space.currentBuffer?.text,"dirty retained text");XCTAssertFalse(space.currentBuffer?.dirty ?? true);XCTAssertFalse(space.hasUnsavedChanges)
 }

}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceRecoveryTests.swift"><ui:paste select="#source-recovery-apple-test-0"/></ui:file>

## android-test 1

The focused reader cases exercise a delayed response after refresh/close,
changed evidence/workspace refusal, exact encoded source paths, and empty saved
text alongside a dirty draft. Resource model cases admit complete receipts,
retain moved/deleted editors and refuse partial namespaces before tab mutation.

Executed on 2026-11-15: the full Android Kotlin app compiler and 14 selected
resource/recovery/provider safety cases pass. The actual-engine controller case
uses real HTTP and the rebuilt `engine_client` with an isolated workspace and
state directory. It saves a dirty source move, exercises existing-menu workspace
Undo/Redo with separate saved baseline and draft text, refuses a newly retained
construction after review, checks mixed draft/resource history order, and admits
a deliberately lost publication reply only through the original request's GET
status without another POST. It retains buffer/TextFieldState/scroll identity,
selection, local undo, tab addresses and the selected file-directory scope.
This is a live engine-to-controller witness, not Android renderer, physical
input, IME or accessibility acceptance. Ordinary-save history, crash/session
recovery, canonical entry transitions and partial-publication recovery remain
outside these checks.

<ui:copy id="source-recovery-android-test-0">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
class SourceRecoveryTest {
 val id="a".repeat(32);val workspace="b".repeat(32)
 fun page()=RecoveryPage(1,workspace,listOf(id),null,0)
 fun reading(hash:String="c".repeat(64))=RecoveryReading(1,workspace,id,hash,"Move",false,listOf(RecoveryPath("a #😀.md",emptyMap())),false,0,"😀","")
 @Test fun exactSnapshotRequestsAndChangedEvidence()=runBlocking {
  val model=RecoveryReader();model.list{page()};var route="";model.read(id,"a #😀.md","saved_before"){route=it;reading()};assertTrue(route.contains("path=a+%23%F0%9F%98%80.md&saved=before"));assertEquals("",model.value?.savedText);assertEquals("😀",model.value?.draftText)
  model.read(id,"a #😀.md"){reading("d".repeat(64))};assertNull(model.value);assertTrue(model.notice.contains("evidence changed"));model.list{page()};model.list(id){page().copy(workspace="other",ids=emptyList())};assertNull(model.page)
 }
 @Test fun closedAndSupersededReadsCannotReplaceSelection()=runBlocking{
  val model=RecoveryReader();model.list{page()};val pending=CompletableDeferred<RecoveryReading>();val old=async(start=CoroutineStart.UNDISPATCHED){model.read(id){pending.await()}};model.list{page()};pending.complete(reading());old.await();assertNull(model.value)
  val late=CompletableDeferred<RecoveryReading>();val closed=async(start=CoroutineStart.UNDISPATCHED){model.read(id){late.await()}};model.close();late.complete(reading());closed.await();assertNull(model.page);assertNull(model.value)
 }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class ResourceReviewTest {
 private fun saved(text:String)=JSONObject().put("kind","file").put("text",text).put("sha256",digest(text)).put("readonly",false).put("executable",true).put("mode",493)
 private fun digest(text:String)=java.security.MessageDigest.getInstance("SHA-256").digest(text.toByteArray()).joinToString(""){"%02x".format(it)}
 private fun missing()=JSONObject().put("kind","missing")
 private fun source(text:String)=JSONObject().put("kind","file").put("identity","source-id").put("text",text).put("writable",true).put("removable",true).put("version",JSONObject.NULL)
 private fun plan(context:String,from:String,to:String,text:String)=JSONObject().put("context",context).put("title","Move source").put("version",1).put("annotations",JSONArray()).put("patches",JSONArray().put(JSONObject().put("path",from).put("before",source(text)).put("after",missing())).put(JSONObject().put("path",to).put("before",missing()).put("after",source(text))))
 @Test fun publicationRetainsLiveEditorTabsSelectionUndoAndSavedMetadata()=runBlocking {
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  try{
   val workspace=Workspace(background);workspace.engine=Engine("http://127.0.0.1:1")
   val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("saved","old-hash","old-guard"));buffer.state.edit{replace(0,length,"dirty");selection=androidx.compose.ui.text.TextRange(2)};workspace.buffers[file.path]=buffer;workspace.show(file.path)
   var context="";var post=0;var published:JSONObject?=null
   val review=ResourceChangeReview{method,path,body->when(path){
    "/api/changes/resource-targets"->JSONObject().put("paths",JSONArray().put("a.py").put("b.py"))
    "/api/changes/resource-inputs"->{context=body!!.getString("context");assertEquals("dirty",body.getJSONObject("drafts").getJSONObject("a.py").getString("text"));assertEquals("old-guard",body.getJSONObject("drafts").getJSONObject("a.py").getString("save_guard"));assertEquals("old-guard",body.getJSONObject("saved_guards").getString("a.py"));assertEquals(digest("saved"),body.getJSONObject("drafts").getJSONObject("a.py").getString("base_sha256"));JSONObject().put("context",context).put("sources",JSONObject().put("a.py",source("dirty")).put("b.py",missing())).put("saved_guards",JSONObject().put("a.py","old-guard")).put("saved_sources",JSONObject())}
    "/api/changes/plan-resources"->JSONObject().put("plan",plan(context,"a.py","b.py","dirty"))
    "/api/changes/resource-consequences"->JSONObject().put("context",context).put("workspace","b".repeat(32)).put("fingerprint","c".repeat(64)).put("protected_targets",JSONArray()).put("saved_sources",JSONObject().put("a.py",missing()).put("b.py",saved("dirty")))
    "/api/changes/resource-save"->{assertEquals("POST",method);post++;val request=body!!.getString("request_id");JSONObject().put("version",2).put("request_id",request).put("adoption_current",true).put("workspace","b".repeat(32)).put("context",context).put("outcome",JSONObject().put("complete",true).put("applied",JSONArray().put("a.py").put("b.py")).put("recovery",".hick-cache/resource-changes/$request/manifest.json")).put("saved_sources",JSONObject().put("a.py",missing()).put("b.py",saved("dirty"))).put("source_files",JSONArray().put(JSONObject().put("file",JSONObject().put("name","b.py").put("path","b.py").put("dir",false)).put("saved",saved("dirty")).put("hash","fresh"))).also{published=it}}
    else->error("Unexpected $method $path")
   }}
   review.prepare(workspace,"Move",JSONObject().put("documentChanges",JSONArray()))
   assertNotNull(review.review);assertEquals("a.py",workspace.active);assertEquals("saved",buffer.base)
   review.apply(workspace);assertEquals(1,post);assertFalse(review.hasPending);assertSame(buffer,workspace.currentBuffer);assertSame(buffer,workspace.buffers["b.py"]);assertNull(workspace.buffers["a.py"]);assertEquals("b.py",workspace.active);assertEquals(2,buffer.state.selection.start);assertEquals("dirty",buffer.base);assertTrue(buffer.state.undoState.canUndo);assertNotNull(published)
   buffer.state.undoState.undo();assertEquals("saved",buffer.text)
  }finally{background.cancel()}
 }
 @Test fun fullRetainedRosterUsesTypedUnaddressedInputsAndRetiresOnChanges()=runBlocking{
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:1")
  fun buffer(path:String,text:String)=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText(text,"old","guard"))
  val named=buffer("owner.txt","named construction draft");val note=buffer("Untitled-note","ordinary unnamed note");note.isUntitledDraft=true;val deleted=buffer("gone.md","retained deleted draft");deleted.resourceDeleted=true;w.buffers[named.file.path]=named;w.buffers[note.file.path]=note;w.retainedResourceEditors["deleted"]=deleted
  val review=ResourceChangeReview{_,_,body->assertEquals("named construction draft",body!!.getJSONObject("ownership_sources").getString("owner.txt"));assertEquals(1,body.getJSONObject("ownership_sources").length());assertEquals(note.text,body.getJSONObject("unaddressed_sources").getString(note.debugIdentity));assertEquals(deleted.text,body.getJSONObject("unaddressed_sources").getString(deleted.debugIdentity));JSONObject().put("saved_guards",JSONObject().put("owner.txt",named.saveGuard)).put("sources",JSONObject().put("owner.txt",source(named.text)))}
  val first=review.acquire(w,listOf("owner.txt"),JSONObject().put("changes",JSONObject())){true};assertTrue(first.third());deleted.advanceSourceRevision();assertFalse(first.third())
  val second=review.acquire(w,listOf("owner.txt"),JSONObject().put("changes",JSONObject())){true};assertTrue(second.third());named.saveGuard="replacement";assertFalse(second.third())
  val third=review.acquire(w,listOf("owner.txt"),JSONObject().put("changes",JSONObject())){true};assertTrue(third.third());w.retainedResourceEditors["added"]=buffer("Untitled-other","new retained text");assertFalse(third.third())
 }
 @Test fun missingPhysicalDraftOrProviderGuardMapRefusesBeforePublication()=runBlocking{
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("source","old"));w.buffers[file.path]=buffer;var calls=0
  val review=ResourceChangeReview{_,_,_->calls++;JSONObject().put("sources",JSONObject().put("a.py",source("source")))}
  assertTrue(runCatching{review.acquire(w,listOf("a.py"),JSONObject()){true}}.isFailure);assertEquals(0,calls)
  buffer.saveGuard="original-guard";assertTrue(runCatching{review.acquire(w,listOf("a.py"),JSONObject()){true}}.isFailure);assertEquals(1,calls);assertSame(buffer,w.buffers[file.path]);assertEquals("source",buffer.text)
 }
 @Test fun completedHistoricalReceiptIsReadableButCannotAdoptAndTypesAreStrict(){
  val raw=JSONObject().put("version",2).put("request_id","request").put("workspace","workspace").put("context","context").put("adoption_current",false).put("adoption_reason","Source was replaced after publication").put("outcome",JSONObject().put("complete",true).put("recovery","record").put("applied",JSONArray().put("a.py"))).put("saved_sources",JSONObject().put("a.py",missing())).put("source_files",JSONArray())
  val receipt=resourcePublication(raw);assertTrue(receipt.complete);assertFalse(receipt.adoptionCurrent);assertNotNull(receipt.adoptionReason)
  val patch=ResourceEditorPatch("a.py",ResourceSource("source","old"),null);assertTrue(runCatching{confirmedResourceFiles(listOf(patch),receipt,"request","workspace","context",mapOf("a.py" to ResourceSaved("missing"))){it.path to true}}.isFailure)
  for(version in listOf<Any>(1,"2",true)){raw.put("version",version);assertTrue(runCatching{resourcePublication(raw)}.isFailure)};raw.put("version",2)
  for(value in listOf<Any>("true",JSONObject.NULL)){raw.put("adoption_current",value);assertTrue(runCatching{resourcePublication(raw)}.isFailure)};raw.remove("adoption_current");assertTrue(runCatching{resourcePublication(raw)}.isFailure)
 }
 @Test fun deletionRetainsReadableEditorAndPathReuseNeverClaimsItsIdentity()=runBlocking{
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("dirty","old"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val patches=listOf(ResourceEditorPatch("a.py",ResourceSource("identity","dirty"),null));val state=ResourceSaved("missing");val receipt=ResourcePublication<FileNode>(2,"request","workspace","context",true,"record",listOf("a.py"),mapOf("a.py" to state),emptyList(),adoptionCurrent=true);val capture=workspace.captureResourceEditors(patches)
  workspace.adoptResourceReceipt(patches,receipt,"request","workspace","context",mapOf("a.py" to state),capture)
  assertSame(buffer,workspace.currentBuffer);assertTrue(buffer.resourceDeleted);assertTrue(buffer.readOnly);assertTrue(workspace.active!!.startsWith("deleted-resource:"));assertNull(workspace.buffers["a.py"])
  val reused=DocumentBuffer(file,FileText("different","new"));workspace.buffers["a.py"]=reused;assertSame(buffer,workspace.currentBuffer);assertNotSame(workspace.buffers["a.py"],workspace.currentBuffer)
 }
 @Test fun partialReceiptRefusesBeforeTabOrEditorMutation()=runBlocking{
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:1");val file=FileNode("a.py","a.py",false,emptyList(),null,null);val buffer=DocumentBuffer(file,FileText("text","old"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  val patches=listOf(ResourceEditorPatch("a.py",ResourceSource("id","text"),null));val state=ResourceSaved("missing");val receipt=ResourcePublication<FileNode>(2,"request","workspace","context",false,"record",emptyList(),null,emptyList());val layout=workspace.layout
  assertTrue(runCatching{workspace.adoptResourceReceipt(patches,receipt,"request","workspace","context",mapOf("a.py" to state),workspace.captureResourceEditors(patches))}.isFailure)
  assertEquals(layout,workspace.layout);assertSame(buffer,workspace.currentBuffer);assertFalse(buffer.resourceDeleted);assertEquals("text",buffer.text)
 }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class ResourceEngineIntegrationTest {
 @Test(timeout=180000) fun actualEngineSaveMenuUndoRedoOwnershipGuardAndLostReplyStatus()=runBlocking{
  val repo=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("scripts/test-resource-proposal-service.py").isFile}
  val binary=System.getenv("HICKORY_SIMPLE_TEST_ENGINE")?.let{java.io.File(it)} ?: repo.resolve("target/source-edit-focus/debug/examples/engine_client")
  check(binary.canExecute()){ "Build the focused engine_client example before running this actual-engine controller witness." }
  val temporary=java.nio.file.Files.createTempDirectory("android-resource-controller-").toFile();val root=temporary.resolve("workspace").also{it.mkdirs()};val folder=root.resolve("src").also{it.mkdir()}
  val saved="saved\n";val draft="saved\n# retained draft\n";folder.resolve("a.py").writeText(saved)
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  val process=ProcessBuilder(binary.absolutePath,root.absolutePath,"window","http://127.0.0.1:1").directory(repo).redirectErrorStream(true).apply{environment()["HICKORY_STATE_DIR"]=temporary.resolve("state").absolutePath}.start()
  val lines=java.util.concurrent.LinkedBlockingQueue<String>(256)
  val reader=Thread{runCatching{process.inputStream.bufferedReader().useLines{stream->stream.forEach{lines.offer(it)}}}}.also{it.isDaemon=true;it.start()}
  var workspace:Workspace?=null
  try{
   val origin=withContext(Dispatchers.IO){val deadline=System.nanoTime()+45_000_000_000L;var result:String?=null;val seen=mutableListOf<String>();while(result==null&&System.nanoTime()<deadline){val line=lines.poll(200,java.util.concurrent.TimeUnit.MILLISECONDS);if(line!=null){seen.add(line);result=Regex("http://127\\.0\\.0\\.1:\\d+").find(line)?.value};if(!process.isAlive&&line==null)break};result ?: error("Engine fixture did not report its origin: ${seen.takeLast(5)}")}
   val engine=Engine(origin);val current=Workspace(background);workspace=current;current.engine=engine
   val tree=engine.files().second;current.rootFiles=tree;current.directoryScope="src";current.files=tree.first{it.path=="src"}.children
   val buffer=current.loadBuffer(current.files.first{it.path=="src/a.py"});buffer.language?.close();buffer.language=null;current.show(buffer.file.path)
   buffer.state.edit{replace(0,length,draft);selection=androidx.compose.ui.text.TextRange(2)};val state=buffer.state;val scroll=buffer.sourceScroll
   fun rename(from:String,to:String)=JSONObject().put("documentChanges",JSONArray().put(JSONObject().put("kind","rename").put("oldUri",languageURI(from)).put("newUri",languageURI(to))))
   val resources=current.resourceChanges
   resources.prepare(current,"Move dirty source",rename("src/a.py","src/b.py"));assertNotNull(resources.notice,resources.review);val reviewed=resources.review!!;assertEquals(saved,folder.resolve("a.py").readText());assertEquals(draft,buffer.text)
   resources.apply(current,reviewed);assertFalse(resources.notice,resources.hasPending);assertNotNull(resources.notice,resources.lastRecovery);assertFalse(folder.resolve("a.py").exists());assertEquals(draft,folder.resolve("b.py").readText());assertSame(buffer,current.currentBuffer);assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll);assertEquals(2,buffer.state.selection.start);assertEquals("src",current.directoryScope);assertEquals(listOf("src/b.py"),current.files.map{it.path})
   suspend fun history(kind:String){current.changes.open(kind,current);withTimeout(45000){while(resources.busy||resources.review==null){delay(25);if(!resources.busy&&resources.review==null&&resources.notice.isNotEmpty()&&!resources.notice.contains("Preparing"))error(resources.notice)}}}
   history("undo")
   val added=DocumentBuffer(FileNode("new.md","new.md",false,emptyList(),null,null),FileText("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"/>",""));current.buffers["new.md"]=added
   resources.apply(current);assertTrue(resources.notice.contains("changed",true));assertTrue(folder.resolve("b.py").exists());assertFalse(resources.hasPending);current.buffers.remove("new.md")
   history("undo");resources.apply(current);assertFalse(resources.notice,resources.hasPending);assertEquals(saved,folder.resolve("a.py").readText());assertFalse(folder.resolve("b.py").exists());assertSame(buffer,current.currentBuffer);assertEquals(draft,buffer.text);assertEquals(saved,buffer.base);assertTrue(buffer.dirty);assertSame(state,buffer.state);assertEquals(2,buffer.state.selection.start)
   history("redo");resources.apply(current);assertFalse(resources.notice,resources.hasPending);assertEquals(draft,folder.resolve("b.py").readText());assertSame(buffer,current.currentBuffer);assertFalse(folder.resolve("a.py").exists())
   // The same menu history must now see a later draft inverse before the
   // resource inverse; a new draft branch retires resource redo normally.
   val other=DocumentBuffer(FileNode("other.txt","other.txt",false,emptyList(),null,null),FileText("before",""));current.buffers["other.txt"]=other
   val edit=JSONObject().put("changes",JSONObject().put(languageURI("other.txt"),JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",6))).put("newText","after"))))
   current.changes.offer("Later draft",edit,mapOf("other.txt" to "before"),current);current.changes.apply(current);assertEquals("after",other.text);current.changes.open("undo",current);assertEquals("before",other.text);assertTrue(folder.resolve("b.py").exists())
   current.buffers.remove("other.txt")
   var publications=0
   val lost=ResourceChangeReview{method,path,body->val reply=engine.request(method,path,body);if(method=="POST"&&path=="/api/changes/resource-save"){publications++;error("Deliberately lost confirmed reply")};reply}
   lost.prepare(current,"Move with lost response",rename("src/b.py","src/c.py"));assertNotNull(lost.notice,lost.review);lost.apply(current);assertEquals(1,publications);assertTrue(lost.hasPending);assertTrue(folder.resolve("c.py").exists());assertEquals("src/b.py",buffer.file.path)
   lost.apply(current);assertEquals(1,publications);lost.status(current);assertFalse(lost.notice,lost.hasPending);assertEquals(1,publications);assertSame(buffer,current.buffers["src/c.py"]);assertEquals("src/c.py",current.active)
   buffer.state.undoState.undo();assertEquals(saved,buffer.text)
   val deletion=JSONObject().put("documentChanges",JSONArray().put(JSONObject().put("kind","delete").put("uri",languageURI("src/c.py"))))
   resources.prepare(current,"Delete while retaining draft",deletion);assertNotNull(resources.notice,resources.review);resources.apply(current);assertFalse(resources.notice,resources.hasPending);assertSame(buffer,current.currentBuffer);assertTrue(buffer.resourceDeleted);assertTrue(buffer.dirty);assertTrue(current.hasUnsavedChanges)
   engine.request("POST","/api/files/op",JSONObject().put("op","create").put("path","src/c.py"))
   val reusedFile=FileNode("c.py","src/c.py",false,emptyList(),null,null);val empty=engine.file(reusedFile);engine.save(reusedFile,"independent",empty.content,empty.hash);val reused=DocumentBuffer(reusedFile,engine.file(reusedFile));current.buffers["src/c.py"]=reused;current.refresh()
   val deletedTab=current.active
   current.fileCommand="Save As";current.filePath="src/c.py";current.performFileCommand()
   assertTrue(current.status.contains("different unused path"));assertEquals(deletedTab,current.active);assertSame(buffer,current.currentBuffer);assertSame(reused,current.buffers["src/c.py"]);assertTrue(current.retainedResourceEditors.values.any{it===buffer});assertEquals("independent",folder.resolve("c.py").readText())
   engine.request("POST","/api/files/op",JSONObject().put("op","create").put("path","src/occupied.py"));current.fileCommand="Save As";current.filePath="src/occupied.py";current.performFileCommand()
   assertTrue(current.status.contains("Original edits are retained"));assertFalse(buffer.saving);assertEquals(deletedTab,current.active);assertSame(buffer,current.currentBuffer);assertTrue(current.retainedResourceEditors.values.any{it===buffer});assertEquals("",folder.resolve("occupied.py").readText())
   current.fileCommand="Save As";current.filePath="src/recovered.py";current.performFileCommand()
   assertEquals(saved,folder.resolve("recovered.py").readText());assertEquals("independent",folder.resolve("c.py").readText());assertSame(reused,current.buffers["src/c.py"]);assertFalse(current.retainedResourceEditors.values.any{it===buffer});assertFalse(current.retainedResourceTabs.values.any{it===buffer});assertFalse(current.hasUnsavedChanges)
  }finally{
   workspace?.buffers?.values?.forEach{it.language?.close()};background.cancel()
   process.destroy();if(!process.waitFor(5,java.util.concurrent.TimeUnit.SECONDS))process.destroyForcibly();reader.join(1000);temporary.deleteRecursively()
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceRecoveryTest.kt"><ui:paste select="#source-recovery-android-test-0"/></ui:file>

## Exercise the existing menu with real retained evidence

Create an isolated reviewed dirty-draft move through the real engine, then read its older saved baseline, dirty draft and recorded absence through the View menu. Keep a newer unsaved editor underneath the reading and verify its identity, dimensions and undo/redo after closing. This establishes the browser flow; native physical input remains unverified.

<ui:copy id="source-recovery-browser-service-witness">
"""Drive source recovery from the View menu without replacing the live editor."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-recovery-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();saved="old saved 😀\n";draft="retained draft 😀\n";(root/"a #😀.txt").write_text(saved)
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   def call(route,body):
    request=urllib.request.Request(backend+"/api/changes/"+route,data=json.dumps(body).encode(),headers={"Content-Type":"application/json"},method="POST")
    with urllib.request.urlopen(request,timeout=30) as reply:return json.load(reply)
   edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///a%20%23%F0%9F%98%80.txt","newUri":"hick:///b.txt"}]}
   capture={"context":"browser-recovery","edit":edit,"drafts":{"a #😀.txt":{"text":draft,"base_sha256":hashlib.sha256(saved.encode()).hexdigest(),"version":None,"writable":True}}}
   inputs=call("resource-inputs",capture);plan=call("plan-resources",{"context":capture["context"],"title":"Move retained draft","edit":edit,"sources":inputs["sources"]})["plan"];proposal={"inputs":inputs,"capture":capture,"plan":plan};facts=call("resource-consequences",proposal)
   result=call("resource-save",{"request_id":"a"*32,"reviewed_fingerprint":facts["fingerprint"],"proposal":proposal});assert result["outcome"]["complete"];record=Path(result["outcome"]["recovery"]).parent.name
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="b.txt");driver.act("remember-editor",path="b.txt");size=driver.act("editor-size",path="b.txt");current=draft+"local editor change\n";driver.act("construction-set-source",path="b.txt",source=current)
   driver.act("recovery-open");assert driver.act("recovery-read",record=record,path="a #😀.txt",snapshot="saved_before")==saved;assert driver.act("recovery-read",snapshot="draft_before")==draft;assert driver.act("recovery-read",snapshot="saved_after") is None
   driver.act("recovery-close");assert driver.act("editor-is-retained",path="b.txt");assert driver.act("editor-size",path="b.txt")==size;assert driver.act("construction-source",path="b.txt")==current
   driver.act("construction-history",redo=False);assert driver.act("construction-source",path="b.txt")==draft;driver.act("construction-history",redo=True);assert driver.act("construction-source",path="b.txt")==current
   assert (root/"b.txt").read_text()==draft and not (root/"a #😀.txt").exists()
   # The newer local draft must refuse restoration without touching saved files.
   driver.act("recovery-open");driver.act("recovery-read",record=record,path="a #😀.txt",snapshot="saved_before")
   refused=" ".join(driver.act("recovery-prepare"));assert "refus" in refused.lower() or "match" in refused.lower() or "changed" in refused.lower(),refused
   assert (root/"b.txt").read_text()==draft and not (root/"a #😀.txt").exists()
   driver.act("recovery-close");driver.act("construction-history",redo=False);assert driver.act("construction-source",path="b.txt")==draft
   driver.act("recovery-open");driver.act("recovery-read",record=record,path="a #😀.txt",snapshot="saved_before");prepared=" ".join(driver.act("recovery-prepare"));assert "Review every" in prepared,prepared
   # A change after review refuses before the save route is used.
   driver.act("construction-set-source",path="b.txt",source=current)
   refused=" ".join(driver.act("recovery-publish"));assert "changed" in refused.lower(),refused
   assert (root/"b.txt").read_text()==draft and not (root/"a #😀.txt").exists()
   driver.act("recovery-close");driver.act("construction-history",redo=False)
   driver.act("recovery-open");driver.act("recovery-read",record=record,path="a #😀.txt",snapshot="saved_before");driver.act("recovery-prepare");published=" ".join(driver.act("recovery-publish"));assert "Restored saved baselines" in published,published
   assert (root/"a #😀.txt").read_text()==saved and not (root/"b.txt").exists()
   assert driver.act("construction-source",path="a #😀.txt")==draft
   assert driver.act("editor-is-retained",path="a #😀.txt",previous="b.txt");assert driver.act("editor-size",path="a #😀.txt")==size
   inverse=" ".join(driver.act("recovery-prepare",inverse=True));assert "Review every" in inverse,inverse
   published=" ".join(driver.act("recovery-publish"));assert "Restored saved baselines" in published,published
   driver.act("recovery-close");assert driver.act("editor-is-retained",path="b.txt");assert driver.act("editor-size",path="b.txt")==size
   assert driver.act("construction-source",path="b.txt")==draft and (root/"b.txt").read_text()==draft and not (root/"a #😀.txt").exists()
   print("PASS: menu recovery reading, stale draft/review refusal, guarded saved/draft restore and inverse, retained editor identity/geometry and explicit saved baseline")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>
<ui:file path="scripts/test-simple-recovery-browser.py"><ui:paste select="#source-recovery-browser-service-witness"/></ui:file>
</ui:doc>
