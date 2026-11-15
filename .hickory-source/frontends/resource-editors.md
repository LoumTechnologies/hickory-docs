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

Three browser cases, two isolated Apple XCTest cases and two isolated Kotlin JUnit
cases pass (2026-11-15). The browser checks include an actual CodeMirror state with
selection and undo depth; the native cases use opaque reference handles. These
witnesses establish routing and refusal, not host renderer or device preservation.
TypeScript checking and source/output reconstruction and audit pass. Next connect
confirmed publication receipts to stable buffer/tab identities, retain deleted
renderers, rebind path-sensitive language sessions and prepare all text changes
before synchronous adoption. No resource Save or restoration menu is connected by
this chapter.

## Browser namespace routing

Reject malformed namespaces and missing or stale captures before returning any routes. Retained handles are admitted only for identities absent from the before namespace and only against their exact restoration draft; they cannot replace a live editor.

<ui:copy id="resource-editor-routes-browser">
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

<ui:file path="apps/web/src/simple/ResourceEditors.ts"><ui:paste select="#resource-editor-routes-browser"/></ui:file>

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

<ui:file path="apps/apple-simple/Sources/ResourceEditors.swift"><ui:paste select="#resource-editor-routes-apple"/></ui:file>

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

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ResourceEditors.kt"><ui:paste select="#resource-editor-routes-android"/></ui:file>

## Browser preservation and refusal witnesses

These model witnesses use mutable editor handles with selection, scroll and undo state. Routing a swap, deletion, explicit restoration and creation must return those exact objects without touching their state. Refusals produce no partial mutations. This is model evidence, not renderer adoption acceptance.

<ui:copy id="resource-editor-routes-browser-tests">
import {describe,it,expect} from 'vitest';
import {EditorState} from '@codemirror/state';
import {history,undoDepth} from '@codemirror/commands';
import {routeResourceEditors,type ResourceEditorPatch} from './ResourceEditors';
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
const source=(identity:string,text=identity)=>({identity,text});
describe('resource editor identity routing',()=>{
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
