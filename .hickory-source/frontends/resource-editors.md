<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Route resources by source identity, retaining editor handles

For maintainers connecting reviewed resource publication to the existing editors.

A path is an address, not an editor identity. Resolve the complete before/after
namespace before changing any buffer or tab. A swap must route both editors from
the original namespace; a deletion must return its existing handle for retention,
and restoration must explicitly supply that retained handle. Never infer restoration
identity from a reused path. This pure routing boundary does no IO and grants no
write permission. Engine publication receipts and current local editor checks are
still required at adoption. Saved baselines are not inferred from draft text.

The result describes both source sides even when text changes accompany a move.
Hosts must prepare the complete editor transactions before applying any route.
A null editor requests a new editor; an existing handle must keep its renderer,
selection, viewport and undo history. Routing itself never touches those objects.
The three implementations intentionally share this small value model; rendering,
language-service retirement and platform transaction ordering remain host adapters.

## Verification boundary

Five focused browser cases and three focused cases per native implementation pass
(2026-11-15). The browser adoption witness uses live buffers and a CodeMirror view:
move, deletion and explicit restoration retain the same state object, selection,
scroll position and undo depth, while the restored saved baseline differs from its
dirty draft. Refusals cover partial publication and changed local baselines. Native
receipt witnesses check exact scope, complete coverage and separate saved/draft
text; existing routing witnesses retain opaque editor references.

TypeScript checking, the Xcode-backed macOS build and selected XCTest cases,
Android compilation and selected JUnit cases, and the iOS simulator build pass.
The default Command Line Tools Swift build cannot load the SwiftUI macro plugin;
use the installed Xcode developer directory for these Apple checks. The iOS project
was regenerated from `project.yml` so it includes the resource routing source.

The live adoption seams update buffer namespaces and native language lifetimes,
retire obsolete opens and keep deleted dirty buffers in unsaved-change guards.
They are not yet called by resource publication menus. Tab-address adoption,
menu-driven resource Save/restoration, coordinated saved/draft workspace history
and real host/device acceptance remain required. These adapter checks do not
establish completed live apply/undo parity.

## Browser namespace routing

Reject malformed namespaces and missing or stale captures before returning any routes. Retained handles are admitted only for identities absent from the before namespace and only against their exact restoration draft; they cannot replace a live editor.

<ui:copy id="resource-editor-routes-browser">
import {Buffer,type Buffers} from './buffers';
import type {FileNode} from '../api/types';
import {editorForBuffer} from './Document';
import {EditorView} from '@codemirror/view';
import {isolateHistory} from '@codemirror/commands';
import type {Transaction} from '@codemirror/state';
export interface ResourceSource {identity:string;text:string}
export interface ResourceEditorPatch {path:string;before:ResourceSource|null;after:ResourceSource|null}
export interface ResourceEditorCapture<T> {editor:T;source:string}
export interface ResourceEditorRoute<T> {identity:string;from:string|null;to:string|null;before:string|null;after:string|null;editor:T|null}
export function routeResourceEditors<T>(patches:readonly ResourceEditorPatch[],live:ReadonlyMap<string,ResourceEditorCapture<T>>,retained:ReadonlyMap<string,ResourceEditorCapture<T>>=new Map()):ResourceEditorRoute<T>[] {
 const before=new Map<string,{path:string;source:string}>(),after=new Map<string,{path:string;source:string}>(),paths=new Set<string>();
 if(!patches.length||patches.length>1000)throw new Error('Choose a bounded complete resource namespace.');
 for(const patch of patches){
  if(!patch.path||paths.has(patch.path)||(!patch.before&&!patch.after))throw new Error('Resource paths must be unique and describe a source transition.');paths.add(patch.path);
  for(const [side,target] of [[patch.before,before],[patch.after,after]] as const)if(side){if(!side.identity||target.has(side.identity))throw new Error('A source identity must have exactly one address per namespace.');target.set(side.identity,{path:patch.path,source:side.text});}
 }
 if([...live.keys()].some(id=>!before.has(id))||[...retained.keys()].some(id=>before.has(id)||!after.has(id)))throw new Error('Editor captures must belong to this complete transition.');
 const handles=new Set<T>();
 for(const [id,capture] of [...live,...retained]){
  const expected=(before.get(id)??after.get(id))!;
  if(capture.source!==expected.source||handles.has(capture.editor))throw new Error('Source changed or one editor was assigned to multiple identities.');handles.add(capture.editor);
 }
 for(const id of before.keys())if(!live.has(id))throw new Error('Acquire every existing source editor before namespace adoption.');
 return [...new Set([...before.keys(),...after.keys()])].map(identity=>({identity,from:before.get(identity)?.path??null,to:after.get(identity)?.path??null,before:before.get(identity)?.source??null,after:after.get(identity)?.source??null,editor:(live.get(identity)??retained.get(identity))?.editor??null}));
}

</ui:copy>

Validate the whole publication before acquiring mutation authority. Match saved
baselines independently of draft text; restoration intentionally keeps them different.

<ui:copy id="resource-editor-confirmation-browser">
export type ResourceSaved = {kind:'missing'}|{kind:'file';text:string;sha256:string;readonly:boolean;executable:boolean;mode:number|null};
export interface ResourceReceipt {version:number;request_id:string;workspace:string;context:string;outcome:{complete:boolean;recovery:string;applied:string[]};saved_sources:Record<string,ResourceSaved>|null;source_files:{file:FileNode;saved:ResourceSaved;hash:string}[]}
export interface ResourceExpectation {request:string;workspace:string;context:string;saved:ReadonlyMap<string,ResourceSaved>}
function sameSaved(a:ResourceSaved|undefined,b:ResourceSaved|undefined):boolean {
 if(!a||!b||a.kind!==b.kind)return false;
 return a.kind==='missing'||b.kind==='file'&&a.text===b.text&&a.sha256===b.sha256&&a.readonly===b.readonly&&a.executable===b.executable&&a.mode===b.mode;
}
export function confirmedResourceFiles(patches:readonly ResourceEditorPatch[],receipt:ResourceReceipt,expected:ResourceExpectation):Map<string,{file:FileNode;saved:Extract<ResourceSaved,{kind:'file'}>;hash:string}> {
 const refuse=()=>{throw new Error('Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors.');};
 if(receipt.version!==1||receipt.request_id!==expected.request||receipt.workspace!==expected.workspace||receipt.context!==expected.context||!receipt.outcome?.complete||!receipt.saved_sources||!receipt.outcome.recovery)refuse();
 const paths=new Set(patches.map(p=>p.path));
 if(paths.size!==patches.length||paths.size!==expected.saved.size||Object.keys(receipt.saved_sources!).length!==paths.size||receipt.outcome.applied.length!==paths.size||new Set(receipt.outcome.applied).size!==paths.size||receipt.outcome.applied.some(path=>!paths.has(path)))refuse();
 const files=new Map<string,{file:FileNode;saved:Extract<ResourceSaved,{kind:'file'}>;hash:string}>();
 for(const patch of patches){const saved=receipt.saved_sources![patch.path];if(!sameSaved(saved,expected.saved.get(patch.path))||(saved?.kind==='file')!==(patch.after!==null))refuse();}
 for(const entry of receipt.source_files){const saved=receipt.saved_sources![entry.file.path];if(!paths.has(entry.file.path)||files.has(entry.file.path)||entry.file.dir||entry.file.generated_by||saved?.kind!=='file'||!sameSaved(saved,entry.saved)||typeof entry.hash!=='string')refuse();else files.set(entry.file.path,{file:entry.file,saved,hash:entry.hash});}
 if(files.size!==patches.filter(p=>p.after!==null).length)refuse();return files;
}
/** Adopt only an already confirmed publication; this function never writes files. */
</ui:copy>

Capture current buffer authority before network publication. Adoption preflights
every editor before removing addresses, retires delayed reads and retains deleted
objects outside the writable namespace. Ordinary Save cannot write those objects.

<ui:copy id="resource-editor-adoption-browser">
export function captureResourceEditors(buffers:Buffers,patches:readonly ResourceEditorPatch[],restoring:ReadonlyMap<string,Buffer>=new Map()):()=>boolean {
 const paths=patches.map(patch=>({path:patch.path,buffer:buffers.values.get(patch.path)}));
 const captures=[...new Set([...paths.flatMap(row=>row.buffer?[row.buffer]:[]),...restoring.values()])].map(buffer=>({buffer,text:buffer.text,base:buffer.base,hash:buffer.hash,version:buffer.version,file:buffer.file}));
 return ()=>paths.every(row=>buffers.values.get(row.path)===row.buffer)&&captures.every(row=>row.buffer.text===row.text&&row.buffer.base===row.base&&row.buffer.hash===row.hash&&row.buffer.version===row.version&&row.buffer.file===row.file&&!row.buffer.saving);
}
export function adoptResourceReceipt(buffers:Buffers,patches:readonly ResourceEditorPatch[],receipt:ResourceReceipt,expected:ResourceExpectation,current:()=>boolean,restoring:ReadonlyMap<string,Buffer>=new Map()):ResourceEditorRoute<Buffer>[] {
 if(!current())throw new Error('Editor drafts or baselines changed after review. Preserve them and inspect Source recovery.');
 const retained=buffers.retainedResources;
 const files=confirmedResourceFiles(patches,receipt,expected),live=new Map<string,ResourceEditorCapture<Buffer>>();
 for(const patch of patches)if(patch.before){const buffer=buffers.values.get(patch.path);if(!buffer||buffer.saving||buffer.readonly||buffer.transient)throw new Error('Source editor is unavailable or saving. Preserve drafts and inspect recovery.');live.set(patch.before.identity,{editor:buffer,source:buffer.text});}
 for(const buffer of restoring.values())if(!buffer.resourceDeleted||![...retained.values()].includes(buffer)||[...buffers.values.values()].includes(buffer))throw new Error('Restoration must select a retained deleted editor. Inspect Source recovery.');
 const held=new Map([...restoring].map(([id,editor])=>[id,{editor,source:editor.text}]));
 const routes=routeResourceEditors(patches,live,held),prepared:{route:ResourceEditorRoute<Buffer>;buffer:Buffer;view?:EditorView;transaction?:Transaction}[]=[];
 for(const route of routes){
  const entry=route.to===null?undefined:files.get(route.to)!;
  const buffer=route.editor??new Buffer(entry!.file,route.after!,entry!.hash,()=>buffers.changed());
  const view=editorForBuffer(buffer)?.view;
  if(view&&(view.composing||view.state.doc.toString()!==buffer.text||view.state.readOnly||!view.state.facet(EditorView.editable)))throw new Error('Finish composition and inspect recovery before adopting source editors.');
  if(route.to!==null&&!patches.some(p=>p.path===route.to&&p.before)&&buffers.values.has(route.to))throw new Error('A destination editor appeared after review. Preserve it and inspect recovery.');
  const transaction=view&&route.after!==null&&buffer.text!==route.after?view.state.update({changes:{from:0,to:view.state.doc.length,insert:route.after},annotations:isolateHistory.of('full'),userEvent:'input.workspace',filter:false}):undefined;
  if(transaction&&transaction.newDoc.toString()!==route.after)throw new Error('Editor refused the complete resource change. Inspect recovery.');
  prepared.push({route,buffer,view,transaction});
 }
 // All failures above precede mutation. Stable buffer objects keep cached renderers.
 buffers.retirePendingReads();
 for(const {route} of prepared)if(route.from!==null)buffers.values.delete(route.from);
 for(const {route,buffer,view,transaction} of prepared){
  if(route.to===null){buffer.resourceDeleted=true;retained.set(route.identity,buffer);continue;}
  const entry=files.get(route.to)!;
  if(view&&transaction)view.update([transaction]);
  buffer.file=entry.file;buffer.text=route.after!;buffer.base=entry.saved.text;buffer.hash=entry.hash;buffer.resourceDeleted=false;buffer.error='';
  buffers.values.set(route.to,buffer);for(const [id,held] of retained)if(held===buffer)retained.delete(id);
 }
 for(const {buffer} of prepared)buffer.publish();buffers.changed();return routes;
}
</ui:copy>

<ui:file path="apps/web/src/simple/ResourceEditors.ts"><ui:paste select="#resource-editor-routes-browser"/><ui:paste select="#resource-editor-confirmation-browser"/><ui:paste select="#resource-editor-adoption-browser"/></ui:file>

## Apple namespace routing

Use UTF-8 keys for both paths and identities: Swift canonical string equality must not merge byte-distinct source addresses. Keep the generic editor handle opaque so the same model serves AppKit and UIKit.

<ui:copy id="resource-editor-routes-apple">
import Foundation
struct ResourceSource {var identity:String;var text:String}
struct ResourceEditorPatch {var path:String;var before:ResourceSource?;var after:ResourceSource?}
struct ResourceEditorCapture<Editor:AnyObject> {var editor:Editor;var source:String}
struct ResourceEditorRoute<Editor:AnyObject> {var identity:String;var from:String?;var to:String?;var before:String?;var after:String?;var editor:Editor?}
private struct ResourceEditorAddress {var identity:String;var path:String;var source:String}
func routeResourceEditors<Editor:AnyObject>(_ patches:[ResourceEditorPatch],live:[String:ResourceEditorCapture<Editor>],retained:[String:ResourceEditorCapture<Editor>]=[:]) throws->[ResourceEditorRoute<Editor>] {
 func key(_ text:String)->Data {Data(text.utf8)}
 func refuse(_ message:String)->NSError {NSError(domain:"ResourceEditors",code:1,userInfo:[NSLocalizedDescriptionKey:message])}
 guard !patches.isEmpty,patches.count<=1000 else{throw refuse("Choose a bounded complete resource namespace.")}
 var before:[Data:ResourceEditorAddress]=[:],after:[Data:ResourceEditorAddress]=[:],paths=Set<Data>(),order:[Data]=[]
 for patch in patches {
  guard !patch.path.isEmpty,paths.insert(key(patch.path)).inserted,patch.before != nil || patch.after != nil else{throw refuse("Resource paths must be unique and describe a source transition.")}
  if let side=patch.before {let id=key(side.identity);guard !id.isEmpty,before[id]==nil else{throw refuse("A source identity must have exactly one address per namespace.")};before[id]=ResourceEditorAddress(identity:side.identity,path:patch.path,source:side.text);order.append(id)}
  if let side=patch.after {let id=key(side.identity);guard !id.isEmpty,after[id]==nil else{throw refuse("A source identity must have exactly one address per namespace.")};after[id]=ResourceEditorAddress(identity:side.identity,path:patch.path,source:side.text)}
 }
 let captured=Dictionary(uniqueKeysWithValues:live.map{(key($0.key),$0.value)}),held=Dictionary(uniqueKeysWithValues:retained.map{(key($0.key),$0.value)})
 guard captured.keys.allSatisfy({before[$0] != nil}),held.keys.allSatisfy({before[$0]==nil && after[$0] != nil}) else{throw refuse("Editor captures must belong to this complete transition.")}
 var handles=Set<ObjectIdentifier>()
 for (id,capture) in Array(captured)+Array(held) {guard let expected=before[id] ?? after[id],key(capture.source)==key(expected.source),handles.insert(ObjectIdentifier(capture.editor)).inserted else{throw refuse("Source changed or one editor was assigned to multiple identities.")}}
 guard before.keys.allSatisfy({captured[$0] != nil}) else{throw refuse("Acquire every existing source editor before namespace adoption.")}
 for patch in patches {if let side=patch.after {let id=key(side.identity);if before[id]==nil{order.append(id)}}}
 return order.map{id in let a=before[id],b=after[id];return ResourceEditorRoute(identity:(a ?? b)!.identity,from:a?.path,to:b?.path,before:a?.source,after:b?.source,editor:(captured[id] ?? held[id])?.editor)}
}

</ui:copy>

Decode complete receipts separately from renderer state. Compare source text,
addresses and scope as UTF-8 bytes; saved restoration text need not equal its draft.

<ui:copy id="resource-editor-confirmation-apple">
struct ResourceSaved:Codable {
 var kind:String;var text:String?;var sha256:String?;var readonly:Bool?;var executable:Bool?;var mode:Int?
 func matches(_ other:ResourceSaved)->Bool {
  func exact(_ a:String?,_ b:String?)->Bool {a.map{Data($0.utf8)}==b.map{Data($0.utf8)}}
  return exact(kind,other.kind) && (kind=="missing" || kind=="file" && exact(text,other.text) && exact(sha256,other.sha256) && readonly==other.readonly && executable==other.executable && mode==other.mode)
 }
}
struct ResourcePublication<File:Decodable>:Decodable {
 struct Outcome:Decodable {var complete:Bool;var recovery:String;var applied:[String]}
 struct Entry:Decodable {var file:File;var saved:ResourceSaved;var hash:String}
 var version:Int;var request_id:String;var workspace:String;var context:String;var outcome:Outcome;var saved_sources:[String:ResourceSaved]?;var source_files:[Entry]
}
func confirmedResourceFiles<File>(_ patches:[ResourceEditorPatch],receipt:ResourcePublication<File>,request:String,workspace:String,context:String,expected:[String:ResourceSaved],address:(File)->(path:String,authored:Bool)) throws->[Data:ResourcePublication<File>.Entry] {
 func key(_ value:String)->Data {Data(value.utf8)}
 func refuse()->NSError {NSError(domain:"ResourceEditors",code:2,userInfo:[NSLocalizedDescriptionKey:"Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors."])}
 guard receipt.version==1,key(receipt.request_id)==key(request),key(receipt.workspace)==key(workspace),key(receipt.context)==key(context),receipt.outcome.complete,!receipt.outcome.recovery.isEmpty,let saved=receipt.saved_sources else{throw refuse()}
 let paths=Set(patches.map{key($0.path)}),savedByKey=Dictionary(uniqueKeysWithValues:saved.map{(key($0.key),$0.value)}),expectedByKey=Dictionary(uniqueKeysWithValues:expected.map{(key($0.key),$0.value)})
 guard paths.count==patches.count,Set(savedByKey.keys)==paths,Set(expectedByKey.keys)==paths,Set(receipt.outcome.applied.map{key($0)})==paths,receipt.outcome.applied.count==paths.count else{throw refuse()}
 for patch in patches {guard let state=savedByKey[key(patch.path)],let reviewed=expectedByKey[key(patch.path)],state.matches(reviewed),(state.kind=="file")==((patch.after) != nil) else{throw refuse()}}
 var files:[Data:ResourcePublication<File>.Entry]=[:]
 for entry in receipt.source_files {let target=address(entry.file),path=key(target.path);guard paths.contains(path),files[path]==nil,target.authored,let state=savedByKey[path],state.kind=="file",state.matches(entry.saved),state.text != nil else{throw refuse()};files[path]=entry}
 guard files.count==patches.filter({$0.after != nil}).count else{throw refuse()};return files
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ResourceEditors.swift"><ui:paste select="#resource-editor-routes-apple"/><ui:paste select="#resource-editor-confirmation-apple"/></ui:file>

## Android namespace routing

Compute the whole transition without mutating Compose state or TextFieldState. Returning the original editor references lets an adoption adapter preserve the input connection, selection and undo manager instead of replacing the text-field state.

<ui:copy id="resource-editor-routes-android">
package com.hickorydocs.simple
import java.util.Collections
import java.util.IdentityHashMap
data class ResourceSource(val identity:String,val text:String)
data class ResourceEditorPatch(val path:String,val before:ResourceSource?,val after:ResourceSource?)
data class ResourceEditorCapture<T:Any>(val editor:T,val source:String)
data class ResourceEditorRoute<T:Any>(val identity:String,val from:String?,val to:String?,val before:String?,val after:String?,val editor:T?)
fun <T:Any> routeResourceEditors(patches:List<ResourceEditorPatch>,live:Map<String,ResourceEditorCapture<T>>,retained:Map<String,ResourceEditorCapture<T>> = emptyMap()):List<ResourceEditorRoute<T>> {
 data class Address(val path:String,val source:String)
 val before=linkedMapOf<String,Address>();val after=linkedMapOf<String,Address>();val paths=mutableSetOf<String>()
 require(patches.isNotEmpty()&&patches.size<=1000){"Choose a bounded complete resource namespace."}
 for(patch in patches){
  require(patch.path.isNotEmpty()&&paths.add(patch.path)&&(patch.before!=null||patch.after!=null)){"Resource paths must be unique and describe a source transition."}
  for((side,target) in listOf(patch.before to before,patch.after to after))if(side!=null){require(side.identity.isNotEmpty()&&!target.containsKey(side.identity)){"A source identity must have exactly one address per namespace."};target[side.identity]=Address(patch.path,side.text)}
 }
 require(live.keys.all{it in before}&&retained.keys.all{it !in before&&it in after}){"Editor captures must belong to this complete transition."}
 val handles=Collections.newSetFromMap(IdentityHashMap<T,Boolean>())
 for((id,capture) in live.toList()+retained.toList()){
  val expected=before[id] ?: after.getValue(id)
  require(capture.source==expected.source&&handles.add(capture.editor)){"Source changed or one editor was assigned to multiple identities."}
 }
 require(before.keys.all{it in live}){"Acquire every existing source editor before namespace adoption."}
 return (before.keys+after.keys).map{id->ResourceEditorRoute(id,before[id]?.path,after[id]?.path,before[id]?.source,after[id]?.source,(live[id] ?: retained[id])?.editor)}
}

</ui:copy>

Require exact scope, path coverage and reviewed saved states before returning
file metadata. The caller receives every surviving file together or no authority.

<ui:copy id="resource-editor-confirmation-android">
data class ResourceSaved(val kind:String,val text:String?=null,val sha256:String?=null,val readonly:Boolean?=null,val executable:Boolean?=null,val mode:Int?=null)
data class ResourcePublishedFile<File>(val file:File,val saved:ResourceSaved,val hash:String)
data class ResourcePublication<File>(val version:Int,val request:String,val workspace:String,val context:String,val complete:Boolean,val recovery:String,val applied:List<String>,val saved:Map<String,ResourceSaved>?,val files:List<ResourcePublishedFile<File>>)
fun <File> confirmedResourceFiles(patches:List<ResourceEditorPatch>,receipt:ResourcePublication<File>,request:String,workspace:String,context:String,expected:Map<String,ResourceSaved>,address:(File)->Pair<String,Boolean>):Map<String,ResourcePublishedFile<File>> {
 fun admit(ok:Boolean){require(ok){"Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors."}}
 admit(receipt.version==1&&receipt.request==request&&receipt.workspace==workspace&&receipt.context==context&&receipt.complete&&receipt.recovery.isNotEmpty())
 val paths=patches.map{it.path}.toSet();val saved=receipt.saved
 admit(paths.size==patches.size&&saved!=null&&saved.keys==paths&&expected.keys==paths&&receipt.applied.size==paths.size&&receipt.applied.toSet()==paths)
 for(patch in patches){val state=saved!!.getValue(patch.path);admit(state==expected[patch.path]&&(state.kind=="file")== (patch.after!=null)&&state.kind in listOf("file","missing"))}
 val files=linkedMapOf<String,ResourcePublishedFile<File>>()
 for(entry in receipt.files){val (path,authored)=address(entry.file);val state=saved!![path];admit(path in paths&&path !in files&&authored&&state?.kind=="file"&&state==entry.saved&&state.text!=null);files[path]=entry}
 admit(files.size==patches.count{it.after!=null});return files
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ResourceEditors.kt"><ui:paste select="#resource-editor-routes-android"/><ui:paste select="#resource-editor-confirmation-android"/></ui:file>

## Adopt a confirmed browser publication into live buffers

The receipt must bind the exact request, workspace, context and reviewed saved
states, cover every applied path, and provide one fresh file descriptor for every
surviving source. Partial publication never supplies editor authority. Prepare all
CodeMirror transactions before removing any address, then publish the namespace
synchronously. A move only replaces address metadata; it leaves the cached view
and buffer intact. Deletion keeps the buffer in a separate identity map and blocks
ordinary Save. Restoration supplies that handle explicitly. Saved text comes from
the receipt independently of the restored draft, so dirty state survives undo.

This adapter does not itself provide resource menu admission or restoration IO.
Those callers must retain the request and review and use current local captures.

## Browser preservation and refusal witnesses

These model witnesses use mutable editor handles with selection, scroll and undo state. Routing a swap, deletion, explicit restoration and creation must return those exact objects without touching their state. Refusals produce no partial mutations. This is model evidence, not renderer adoption acceptance.

<ui:copy id="resource-editor-routes-browser-tests">
import {describe,it,expect,vi} from 'vitest';
import {Buffer,Buffers} from './buffers';
import {api} from '../api/client';
import {EditorView} from '@codemirror/view';
const views=vi.hoisted(()=>new WeakMap());
vi.mock('./Document',()=>({editorForBuffer:(buffer:Buffer)=>views.get(buffer)}));
import {EditorState} from '@codemirror/state';
import {history,undoDepth} from '@codemirror/commands';
import {routeResourceEditors,adoptResourceReceipt,captureResourceEditors,type ResourceEditorPatch,type ResourceReceipt,type ResourceSaved} from './ResourceEditors';
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
const source=(identity:string,text=identity)=>({identity,text});
describe('resource editor identity routing',()=>{
 it('adopts a confirmed move, deletion and restoration into live buffers with separate saved baselines',async()=>{
  const buffers=new Buffers(()=>{}),a=new Buffer({path:'old.py',name:'old.py',dir:false},'saved','old-hash',()=>{});a.edit('dirty');buffers.values.set('old.py',a);
  const state=EditorState.create({doc:'saved',extensions:[history()]}).update({changes:{from:0,to:5,insert:'dirty'},selection:{anchor:2}}).state;
  const view=new EditorView({state});views.set(a,{view});view.scrollDOM.scrollTop=73;
  const retained=buffers.retainedResources,saved:ResourceSaved={kind:'file',text:'dirty',sha256:'a'.repeat(64),readonly:false,executable:false,mode:null};
  function adopt(patches:ResourceEditorPatch[],baseline:ResourceSaved,restoring=new Map<string,Buffer>()){
   const expected={request:'request',workspace:'workspace',context:'context',saved:new Map(patches.map(p=>[p.path,p.after?baseline:{kind:'missing'} as ResourceSaved]))};
   const receipt:ResourceReceipt={version:1,request_id:'request',workspace:'workspace',context:'context',outcome:{complete:true,recovery:'recovery',applied:patches.map(p=>p.path)},saved_sources:Object.fromEntries(expected.saved),source_files:patches.filter(p=>p.after).map(p=>({file:{path:p.path,name:p.path,dir:false},saved:baseline,hash:'fresh'}))};
   return adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches,restoring),restoring);
  }
  const original=view.state;
  adopt([{path:'old.py',before:source('a','dirty'),after:null},{path:'new.py',before:null,after:source('a','dirty')}],saved);
  expect(buffers.values.get('new.py')).toBe(a);expect(view.state).toBe(original);expect(a.base).toBe('dirty');expect(a.dirty).toBe(false);
  a.base='saved';
  adopt([{path:'new.py',before:source('a','dirty'),after:null}],saved);expect(retained.get('a')).toBe(a);expect(a.resourceDeleted).toBe(true);expect(buffers.hasUnsavedChanges).toBe(true);
  const write=vi.spyOn(api,'saveFile').mockRejectedValue(new Error('Deleted editors cannot write.'));await a.save();expect(write).not.toHaveBeenCalled();write.mockRestore();
  adopt([{path:'old.py',before:null,after:source('restored','dirty')}],{...saved,text:'saved'},new Map([['restored',a]]));
  expect(buffers.values.get('old.py')).toBe(a);expect(a.base).toBe('saved');expect(a.text).toBe('dirty');expect(a.dirty).toBe(true);expect(a.resourceDeleted).toBe(false);
  expect(view.state).toBe(original);expect(view.state.selection.main.anchor).toBe(2);expect(undoDepth(view.state)).toBe(1);expect(view.scrollDOM.scrollTop).toBe(73);view.destroy();
 });
 it('refuses partial receipts and changed local baselines without changing any live address',()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'a','hash',()=>{});buffers.values.set('a.py',buffer);
  const patches=[{path:'a.py',before:source('a'),after:null}],expected={request:'r',workspace:'w',context:'c',saved:new Map<string,ResourceSaved>([['a.py',{kind:'missing'}]])};
  const receipt:ResourceReceipt={version:1,request_id:'r',workspace:'w',context:'c',outcome:{complete:false,recovery:'recovery',applied:[]},saved_sources:null,source_files:[]},retained=buffers.retainedResources;
  expect(()=>adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches))).toThrow();
  const current=captureResourceEditors(buffers,patches);buffer.base='independent';
  expect(()=>adoptResourceReceipt(buffers,patches,{...receipt,outcome:{complete:true,recovery:'recovery',applied:['a.py']},saved_sources:{'a.py':{kind:'missing'}}},expected,current)).toThrow();
  expect(buffers.values.get('a.py')).toBe(buffer);expect(buffer.base).toBe('independent');expect(retained.size).toBe(0);
 });

 it('retains editor state through swaps, deletion and explicit restoration',()=>{
  const a={selection:[2,4],scroll:108,undo:['old'],draft:'dirty'},b={selection:[0,0],scroll:0,undo:[],draft:'B'};
  const live=new Map([['a',{editor:a,source:'dirty'}],['b',{editor:b,source:'B'}]]);
  const patches:ResourceEditorPatch[]=[{path:'a.py',before:source('a','dirty'),after:source('b','B')},{path:'b.py',before:source('b','B'),after:source('a','dirty')}];
  const swap=routeResourceEditors(patches,live);expect(swap.find(x=>x.to==='b.py')?.editor).toBe(a);expect(swap.find(x=>x.to==='a.py')?.editor).toBe(b);
  const deleted=routeResourceEditors([{path:'b.py',before:source('a','dirty'),after:null}],new Map([['a',live.get('a')!]]));expect(deleted[0].editor).toBe(a);expect(deleted[0].to).toBeNull();
  const restored=routeResourceEditors([{path:'b.py',before:null,after:source('restored','dirty')}],new Map(),new Map([['restored',live.get('a')!]]));expect(restored[0].editor).toBe(a);
  const created=routeResourceEditors([{path:'new.py',before:null,after:source('new','')}],new Map());expect(created[0].editor).toBeNull();expect(created[0].after).toBe('');
  expect(a).toEqual({selection:[2,4],scroll:108,undo:['old'],draft:'dirty'});
 });
 it('retains actual CodeMirror selection and undo history through a move and inverse',()=>{
  let state=EditorState.create({doc:'saved',extensions:[history()]});state=state.update({changes:{from:5,insert:' dirty'},selection:{anchor:3}}).state;
  const editor={state,scrollTop:201},capture={editor,source:state.doc.toString()},a=source('a',capture.source);
  const moved=routeResourceEditors([{path:'old.py',before:a,after:null},{path:'new.py',before:null,after:a}],new Map([['a',capture]]));
  const inverse=routeResourceEditors([{path:'new.py',before:a,after:null},{path:'old.py',before:null,after:a}],new Map([['a',capture]]));
  expect(moved[0].editor).toBe(editor);expect(inverse[0].editor?.state).toBe(state);expect(editor.state.selection.main.anchor).toBe(3);expect(undoDepth(editor.state)).toBe(1);expect(editor.scrollTop).toBe(201);
 });
 it('refuses duplicate, stale, missing and aliased captures before mutation',()=>{
  const editor={undo:['retained']},capture={editor,source:'a'},live=new Map([['a',capture]]),patch={path:'a.py',before:source('a'),after:source('a','changed')};
  expect(()=>routeResourceEditors([patch,patch],live)).toThrow();expect(()=>routeResourceEditors([patch],new Map())).toThrow();expect(()=>routeResourceEditors([patch],new Map([['a',{editor,source:'later'}]]))).toThrow();
  expect(()=>routeResourceEditors([patch],live,new Map([['a',capture]]))).toThrow();expect(()=>routeResourceEditors([patch,{path:'b.py',before:source('b'),after:null}],new Map([...live,['b',{editor,source:'b'}]]))).toThrow();
  expect(editor.undo).toEqual(['retained']);
  const exact=routeResourceEditors([{path:'é.py',before:null,after:source('é')},{path:'e\u0301.py',before:null,after:source('e\u0301')}],new Map());expect(exact).toHaveLength(2);
 });
});
</ui:copy>

<ui:file path="apps/web/src/simple/ResourceEditors.test.ts"><ui:paste select="#resource-editor-routes-browser-tests"/></ui:file>

## Apple preservation and refusal witnesses

The same lifecycle retains reference identity on both Apple hosts. Byte-distinct paths remain separate; duplicate addresses, missing captures, stale source and aliasing one editor across identities refuse.

<ui:copy id="resource-editor-routes-apple-tests">
import XCTest
@testable import HickorySimple
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
final class ResourceEditorsTests:XCTestCase {
 final class Editor {var selection=2;var scroll=108;var undo=["old"]}
 func testMoveDeleteRestoreAndCreateKeepOpaqueEditorState() throws {
  let editor=Editor(),a=ResourceSource(identity:"a",text:"dirty"),capture=ResourceEditorCapture(editor:editor,source:"dirty")
  let move=try routeResourceEditors([ResourceEditorPatch(path:"a.py",before:a,after:nil),ResourceEditorPatch(path:"b.py",before:nil,after:a)],live:["a":capture]);XCTAssertTrue(move[0].editor===editor);XCTAssertEqual(move[0].to,"b.py")
  let other=Editor(),b=ResourceSource(identity:"b",text:"B"),otherCapture=ResourceEditorCapture(editor:other,source:"B")
  let swapped=try routeResourceEditors([ResourceEditorPatch(path:"a.py",before:a,after:b),ResourceEditorPatch(path:"b.py",before:b,after:a)],live:["a":capture,"b":otherCapture]);XCTAssertTrue(swapped.first(where:{$0.to=="b.py"})?.editor===editor);XCTAssertTrue(swapped.first(where:{$0.to=="a.py"})?.editor===other)
  let deleted=try routeResourceEditors([ResourceEditorPatch(path:"b.py",before:a,after:nil)],live:["a":capture]);XCTAssertNil(deleted[0].to);XCTAssertTrue(deleted[0].editor===editor)
  let restored=try routeResourceEditors([ResourceEditorPatch(path:"b.py",before:nil,after:ResourceSource(identity:"restored",text:"dirty"))],live:[:],retained:["restored":capture]);XCTAssertTrue(restored[0].editor===editor)
  let created:[ResourceEditorRoute<Editor>]=try routeResourceEditors([ResourceEditorPatch(path:"new.py",before:nil,after:ResourceSource(identity:"new",text:""))],live:[:]);XCTAssertNil(created[0].editor);XCTAssertEqual(created[0].after,"")
  XCTAssertEqual(editor.selection,2);XCTAssertEqual(editor.scroll,108);XCTAssertEqual(editor.undo,["old"])
 }
 func testConfirmedReceiptRequiresWholeNamespaceAndSeparateSavedBaseline() throws {
  let saved=ResourceSaved(kind:"file",text:"saved",sha256:String(repeating:"a",count:64),readonly:false,executable:false,mode:nil)
  let absent=ResourceSaved(kind:"missing"),patches=[ResourceEditorPatch(path:"old.py",before:ResourceSource(identity:"a",text:"dirty"),after:nil),ResourceEditorPatch(path:"new.py",before:nil,after:ResourceSource(identity:"a",text:"dirty"))]
  var receipt=ResourcePublication<String>(version:1,request_id:"request",workspace:"workspace",context:"context",outcome:.init(complete:true,recovery:"record",applied:["old.py","new.py"]),saved_sources:["old.py":absent,"new.py":saved],source_files:[.init(file:"new.py",saved:saved,hash:"fresh")])
  func confirm() throws->[Data:ResourcePublication<String>.Entry] {try confirmedResourceFiles(patches,receipt:receipt,request:"request",workspace:"workspace",context:"context",expected:["old.py":absent,"new.py":saved]){($0,true)}}
  XCTAssertEqual(try confirm()[Data("new.py".utf8)]?.saved.text,"saved")
  receipt.outcome.complete=false;XCTAssertThrowsError(try confirm());receipt.outcome.complete=true
  receipt.request_id="other";XCTAssertThrowsError(try confirm());receipt.request_id="request"
  receipt.source_files.append(receipt.source_files[0]);XCTAssertThrowsError(try confirm())
 }
 func testCompletePreflightAndByteExactNamespaces() throws {
  let editor=Editor(),capture=ResourceEditorCapture(editor:editor,source:"a"),a=ResourceSource(identity:"a",text:"a"),patch=ResourceEditorPatch(path:"a.py",before:a,after:nil)
  XCTAssertThrowsError(try routeResourceEditors([patch,patch],live:["a":capture]));XCTAssertThrowsError(try routeResourceEditors([patch],live:[String:ResourceEditorCapture<Editor>]()))
  XCTAssertThrowsError(try routeResourceEditors([patch],live:["a":ResourceEditorCapture(editor:editor,source:"later")]))
  XCTAssertThrowsError(try routeResourceEditors([patch],live:["a":capture],retained:["a":capture]))
  let routes:[ResourceEditorRoute<Editor>]=try routeResourceEditors([ResourceEditorPatch(path:"é.py",before:nil,after:ResourceSource(identity:"first",text:"")),ResourceEditorPatch(path:"e\u{301}.py",before:nil,after:ResourceSource(identity:"second",text:""))],live:[:]);XCTAssertEqual(routes.count,2)
  XCTAssertEqual(editor.undo,["old"])
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ResourceEditorsTests.swift"><ui:paste select="#resource-editor-routes-apple-tests"/></ui:file>

## Android preservation and refusal witnesses

Reference equality matters even for editor wrappers that define value equality. The identity set uses JVM reference identity, preserving distinct editor instances while rejecting a single instance assigned to two sources.

<ui:copy id="resource-editor-routes-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
class ResourceEditorsTest {
 class Editor {var selection=2;var scroll=108;val undo=mutableListOf("old")}
 @Test fun moveDeleteRestoreAndCreateKeepOpaqueEditorState(){
  val editor=Editor();val a=ResourceSource("a","dirty");val capture=ResourceEditorCapture(editor,"dirty")
  val move=routeResourceEditors(listOf(ResourceEditorPatch("a.py",a,null),ResourceEditorPatch("b.py",null,a)),mapOf("a" to capture));assertSame(editor,move.single().editor);assertEquals("b.py",move.single().to)
  val other=Editor();val b=ResourceSource("b","B");val swapped=routeResourceEditors(listOf(ResourceEditorPatch("a.py",a,b),ResourceEditorPatch("b.py",b,a)),mapOf("a" to capture,"b" to ResourceEditorCapture(other,"B")));assertSame(editor,swapped.first{it.to=="b.py"}.editor);assertSame(other,swapped.first{it.to=="a.py"}.editor)
  val deleted=routeResourceEditors(listOf(ResourceEditorPatch("b.py",a,null)),mapOf("a" to capture));assertNull(deleted.single().to);assertSame(editor,deleted.single().editor)
  val restored=routeResourceEditors(listOf(ResourceEditorPatch("b.py",null,ResourceSource("restored","dirty"))),emptyMap(),mapOf("restored" to capture));assertSame(editor,restored.single().editor)
  val created=routeResourceEditors<Editor>(listOf(ResourceEditorPatch("new.py",null,ResourceSource("new",""))),emptyMap());assertNull(created.single().editor);assertEquals("",created.single().after)
  assertEquals(2,editor.selection);assertEquals(108,editor.scroll);assertEquals(listOf("old"),editor.undo)
 }
 @Test fun confirmedReceiptRequiresWholeNamespaceAndSeparateSavedBaseline(){
  val saved=ResourceSaved("file","saved","a".repeat(64),false,false,null);val absent=ResourceSaved("missing")
  val patches=listOf(ResourceEditorPatch("old.py",ResourceSource("a","dirty"),null),ResourceEditorPatch("new.py",null,ResourceSource("a","dirty")))
  val receipt=ResourcePublication(1,"request","workspace","context",true,"record",listOf("old.py","new.py"),mapOf("old.py" to absent,"new.py" to saved),listOf(ResourcePublishedFile("new.py",saved,"fresh")))
  fun confirm(value:ResourcePublication<String>)=confirmedResourceFiles(patches,value,"request","workspace","context",mapOf("old.py" to absent,"new.py" to saved)){it to true}
  assertEquals("saved",confirm(receipt).getValue("new.py").saved.text)
  for(value in listOf(receipt.copy(complete=false),receipt.copy(request="other"),receipt.copy(files=receipt.files+receipt.files),receipt.copy(applied=listOf("new.py")))){try{confirm(value);fail("Expected refusal")}catch(_:IllegalArgumentException){}}
 }
 @Test fun completePreflightAndByteExactNamespaces(){
  val editor=Editor();val capture=ResourceEditorCapture(editor,"a");val a=ResourceSource("a","a");val patch=ResourceEditorPatch("a.py",a,null)
  fun refuses(block:()->Unit){try{block();fail("Expected refusal")}catch(_:IllegalArgumentException){}}
  refuses{routeResourceEditors(listOf(patch,patch),mapOf("a" to capture))};refuses{routeResourceEditors<Editor>(listOf(patch),emptyMap())}
  refuses{routeResourceEditors(listOf(patch),mapOf("a" to ResourceEditorCapture(editor,"later")))};refuses{routeResourceEditors(listOf(patch),mapOf("a" to capture),mapOf("a" to capture))}
  refuses{routeResourceEditors(listOf(patch,ResourceEditorPatch("b.py",ResourceSource("b","b"),null)),mapOf("a" to capture,"b" to ResourceEditorCapture(editor,"b")))}
  assertEquals(2,routeResourceEditors<Editor>(listOf(ResourceEditorPatch("é.py",null,ResourceSource("é","")),ResourceEditorPatch("e\u0301.py",null,ResourceSource("e\u0301",""))),emptyMap()).size)
  assertEquals(listOf("old"),editor.undo)
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ResourceEditorsTest.kt"><ui:paste select="#resource-editor-routes-android-tests"/></ui:file>

</ui:doc>
