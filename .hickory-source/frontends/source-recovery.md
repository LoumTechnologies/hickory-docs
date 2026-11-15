<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read retained source recovery without replacing the editor

For maintainers adding local recovery through the existing menu on browser, macOS, iOS and Android.

Discovery names retained resource operations, including interrupted ones. It does not label receipts as current disk state or grant restoration authority. Open a floating reading from the shared View menu and leave the editor mounted underneath it. Existing saved baselines and unsaved drafts can differ; request one explicit snapshot and preserve empty text separately from absent source. This reader does not yet perform restoration or provide history of ordinary saves.

## Focused evidence, 2026-11-15

The real-engine browser witness passes menu discovery, exact saved/draft/absent snapshot reads and preservation of the live editor identity, geometry, newer unsaved text and undo/redo. Two focused model cases pass on each of browser, Apple and Android for retired asynchronous replies, changed evidence/workspace refusal and distinct empty saved/dirty text. TypeScript checking and the iOS simulator build pass. Two shared-store discovery cases cover exact sorted pagination, read-only absence and symlink refusal. The actual resource service witness passes discovery and separate saved/draft text inspection alongside its guarded save/restore cases. Native physical input and accessibility acceptance remain unverified; ordinary-save history, crash/session restoration and frontend resource restoration are still required.

## browser 1

The reader binds selection to a workspace and exact record fingerprint. Each read retires older requests; refresh and close invalidate pending replies. A later snapshot of changed evidence refuses, so two sides cannot silently come from different records.

<ui:copy id="source-recovery-browser-0">
import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
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

Render the selected retained evidence in a floating reading, reached through existing menu commands. Keep refresh, paging and snapshot controls inside this reading, with selectable abbreviated text and explicit absence. Closing returns focus to the existing source editor.

<ui:copy id="source-recovery-browser-1">
export function SourceRecoveryView({close}:{close:()=>void}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new RecoveryReader(route=>request('GET',route),paint),[]),panel=useRef<HTMLElement>(null);
 const [path,setPath]=useState(''),[snapshot,setSnapshot]=useState<RecoverySnapshot>('draft_before');
 useEffect(()=>{panel.current?.focus();void model.list();return()=>model.close();},[model]);
 const value=model.value,selected=value?.paths.find(item=>item.path===path),text=value?.[snapshot.startsWith('saved')?'saved_text':'draft_text'];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source recovery" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Source recovery</strong><button onClick={close}>Close</button></header>
 <button disabled={model.loading} onClick={()=>{setPath('');void model.list();}}>Refresh records</button><p role="status">{model.notice}</p>
 {model.page&&<><label>Retained record <select aria-label="Retained record" value={value?.id??''} disabled={model.loading} onChange={event=>{if(event.target.value){setPath('');void model.read(event.target.value);}}}><option value="">Choose record</option>{model.page.ids.map(id=><option key={id} value={id}>{id}</option>)}</select></label>{model.page.ignored>0&&<p>{model.page.ignored} unrecognized entries omitted.</p>}{model.page.next&&<button disabled={model.loading} onClick={()=>{setPath('');void model.list(model.page!.next);}}>Next 100 identities</button>}</>}
 {value&&<><h3>{value.title}{value.title_truncated?'…':''}</h3><p>{value.all_paths_recorded_applied?'Every path has an applied receipt.':'The record does not confirm every path was applied.'} These receipts describe the recorded operation, not current disk state.</p>{value.unrecognized_files.length>0&&<p>{value.unrecognized_files.length} unrecognized files in this record.</p>}
 <label>Recorded source <select aria-label="Recorded source" value={path} disabled={model.loading} onChange={event=>{setPath(event.target.value);void model.read(value.id,event.target.value,snapshot);}}><option value="" disabled>Choose source</option>{value.paths.map(item=><option key={item.path} value={item.path}>{item.path}</option>)}</select></label>
 <label>Snapshot <select aria-label="Snapshot" value={snapshot} disabled={model.loading} onChange={event=>{const next=event.target.value as RecoverySnapshot;setSnapshot(next);if(path)void model.read(value.id,path,next);}}>{(['saved_before','draft_before','saved_after','draft_after'] as RecoverySnapshot[]).map(side=><option key={side} value={side}>{side.replace('_',' ')}</option>)}</select></label>
 {selected&&<><p>{selected[snapshot].kind==='missing'?'Source absent in this snapshot.':`${selected[snapshot].bytes} UTF-8 bytes · ${selected[snapshot].sha256}`}</p>{text!==null&&text!==undefined&&<><pre style={{whiteSpace:'pre-wrap'}}>{text.slice(0,32000)}</pre>{text.length>32000&&<p>First 32,000 characters shown.</p>}</>}</>}
 </>}
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

## apple 2

Render the selected retained evidence in a floating reading, reached through existing menu commands. Keep refresh, paging and snapshot controls inside this reading, with selectable abbreviated text and explicit absence. Closing returns focus to the existing source editor.

<ui:copy id="source-recovery-apple-1">
struct SourceRecoveryView:View {
 var workspace:Workspace;@State private var model=RecoveryReader();@State private var path="";@State private var snapshot="draft_before"
 private func list(_ after:String?=nil)async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};path="";await model.list(after:after){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 private func read(_ id:String,_ path:String?=nil,_ snapshot:String="draft_before")async {guard let engine=workspace.engine else{return};await model.read(id,path:path,snapshot:snapshot){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 var body:some View {VStack(alignment:.leading,spacing:8){
  HStack{Text("Source recovery").font(.headline);Spacer();Button("Close"){workspace.showRecovery=false;workspace.currentBuffer?.focusSource?()}}
  Button("Refresh records"){Task{await list()}}.disabled(model.loading);Text(model.notice).font(.caption).textSelection(.enabled)
  ScrollView{VStack(alignment:.leading,spacing:8){
   if let page=model.page {Picker("Retained record",selection:Binding(get:{model.value?.id ?? ""},set:{id in if !id.isEmpty{path="";Task{await read(id)}}})){Text("Choose record").tag("");ForEach(page.ids,id:\.self){Text($0).tag($0)}}.disabled(model.loading)
    if page.ignored>0 {Text("\(page.ignored) unrecognized entries omitted.").font(.caption)}
    if let next=page.next {Button("Next 100 identities"){Task{await list(next)}}.disabled(model.loading)}
   }
   if let value=model.value {
    Text(value.title+(value.title_truncated ? "…":"")).font(.headline)
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
 }.padding().frame(idealWidth:680,idealHeight:600).task{await list()}.onDisappear{model.close()}}
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceRecovery.swift"><ui:paste select="#source-recovery-apple-0"/><ui:paste select="#source-recovery-apple-1"/></ui:file>

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
 suspend fun read(record:String,path:String?=null,snapshot:String="draft_before",load:suspend(String)->RecoveryReading){val workspace=page?.workspace;val previous=value?.takeIf{it.id==record};val id=++ticket;loading=true;value=null;notice="Reading retained source evidence…"
  try{check(workspace!=null&&page?.ids?.contains(record)==true){"Refresh recovery identities before selecting a record."};val parts=snapshot.split('_');check(parts.size==2&&parts[0] in listOf("saved","draft")&&parts[1] in listOf("before","after")){"Choose a supported recovery snapshot."}
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
 val model=remember{RecoveryReader()};val scope=rememberCoroutineScope();val engine=workspace.engine;var path by remember{mutableStateOf("")};var snapshot by remember{mutableStateOf("draft_before")};var recordMenu by remember{mutableStateOf(false)};var pathMenu by remember{mutableStateOf(false)}
 suspend fun list(after:String?=null){if(engine==null){model.notice="Connect to a workspace server first.";return};path="";model.list(after){recoveryPage(engine.request("GET",it))}}
 suspend fun read(id:String,path:String?=null,snapshot:String="draft_before"){if(engine==null)return;model.read(id,path,snapshot){recoveryReading(engine.request("GET",it))}}
 LaunchedEffect(engine){list()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Source recovery",style=MaterialTheme.typography.titleMedium);Row{TextButton(enabled=!model.loading,onClick={scope.launch{list()}}){Text("Refresh records")};TextButton(onClick={workspace.showRecovery=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  model.page?.let{page->Box{TextButton(enabled=!model.loading,onClick={recordMenu=true}){Text(model.value?.id ?: "Choose retained record")};DropdownMenu(expanded=recordMenu,onDismissRequest={recordMenu=false}){page.ids.forEach{id->DropdownMenuItem(text={Text(id)},onClick={recordMenu=false;path="";scope.launch{read(id)}})}}};if(page.ignored>0)Text("${page.ignored} unrecognized entries omitted.",fontSize=12.sp);page.next?.let{next->TextButton(enabled=!model.loading,onClick={scope.launch{list(next)}}){Text("Next 100 identities")}}}
  model.value?.let{value->Text(value.title+if(value.titleTruncated)"…" else "",style=MaterialTheme.typography.titleSmall);Text((if(value.allApplied)"Every path has an applied receipt." else "The record does not confirm every path was applied.")+" These receipts describe the recorded operation, not current disk state.",fontSize=12.sp);if(value.unrecognized>0)Text("${value.unrecognized} unrecognized files in this record.",fontSize=12.sp)
   Box{TextButton(enabled=!model.loading,onClick={pathMenu=true}){Text(path.ifEmpty{"Choose recorded source"})};DropdownMenu(expanded=pathMenu,onDismissRequest={pathMenu=false}){value.paths.forEach{item->DropdownMenuItem(text={Text(item.path)},onClick={pathMenu=false;path=item.path;scope.launch{read(value.id,path,snapshot)}})}}}
   listOf("saved_before","draft_before","saved_after","draft_after").forEach{side->TextButton(enabled=!model.loading,onClick={snapshot=side;if(path.isNotEmpty())scope.launch{read(value.id,path,side)}}){Text((if(snapshot==side)"✓ " else "")+side.replace('_',' '))}}
   value.paths.find{it.path==path}?.snapshots?.get(snapshot)?.let{selected->SelectionContainer{Text(if(selected.kind=="missing")"Source absent in this snapshot." else "${selected.bytes} UTF-8 bytes · ${selected.sha256}",fontSize=12.sp)};val text=if(snapshot.startsWith("saved"))value.savedText else value.draftText;if(text!=null){SelectionContainer{Text(text.take(32000),fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(text.length>32000)Text("First 32,000 characters shown.",fontSize=12.sp)}}
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceRecovery.kt"><ui:paste select="#source-recovery-android-0"/><ui:paste select="#source-recovery-android-1"/></ui:file>

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
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceRecoveryTests.swift"><ui:paste select="#source-recovery-apple-test-0"/></ui:file>

## android-test 1

These focused cases exercise a delayed response after refresh/close, changed evidence and workspace refusal, exact encoded source paths, and empty saved text alongside a dirty draft. They do not establish physical input or accessibility acceptance.

<ui:copy id="source-recovery-android-test-0">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
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
   print("PASS: View menu recovery discovery, exact named saved/dirty/absent snapshot reading, retained editor geometry/identity/draft/undo/redo, unchanged saved source")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>
<ui:file path="scripts/test-simple-recovery-browser.py"><ui:paste select="#source-recovery-browser-service-witness"/></ui:file>
</ui:doc>
