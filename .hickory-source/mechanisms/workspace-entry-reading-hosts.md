<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Freeze the optional workspace-entry reading before compiler work

For maintainers integrating entry-dependent capabilities with the existing simple hosts. Capture the original editor and retained-source roster first; only a capability that needs the main closure awaits this reader. Source content remains available independently. An explicitly selected projection stays explicit rather than being replaced with an automatic main.

These adapters validate the versioned shape and expected runtime/namespace, retain immutable original material, and keep the caller's original current-context predicate. That predicate reports local editor/source ownership; it is not a remote-entry freshness claim. The backend must validate the original entry proof before and after compiler work, including Apply preparation. Neither a callback nor a menu command grants source publication authority.

## Browser: retain exact typed proof material

Retain the exact original scope. Reject unsupported fields, literal-type mismatches, invalid generations and incomplete configured-source observations. Returning JSON must not expose mutable storage that can retarget the retained selection; a late response cannot borrow a newer source lease.

<ui:copy id="workspace-entry-reading-browser-helper">
export type EntryStamp=Readonly<{version:1;identity:string;generation:number;path:string|null}>;
export type EntrySelection=Readonly<{version:1;entry_protocol:1;workspace:string;namespace:string;entry:EntryStamp;guard:string;source_guard?:string;source_hash?:string;movable:boolean;source_effects:false}>;
export type FrozenEntrySelection=Readonly<{entryPath:string|null;proof:EntrySelection;current:()=>boolean}>;
export type EntryReadingLease={workspace:string;namespace:string;current:()=>boolean};
function object(value:unknown):Record<string,unknown>{if(!value||typeof value!=="object"||Array.isArray(value))throw new Error("Workspace entry response is malformed.");return value as Record<string,unknown>;}
function hex(value:unknown,size:number):value is string{return typeof value==="string"&&value.length===size&&/^[0-9a-f]+$/.test(value);}
export function parseEntrySelection(value:unknown):EntrySelection{
 const body=object(value),entry=object(body.entry),path=entry.path;
 if(Object.keys(body).some(key=>!["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].includes(key))||Object.keys(entry).some(key=>!["version","identity","generation","path"].includes(key)))throw new Error("Workspace entry response has unsupported fields.");
 if(body.version!==1||body.entry_protocol!==1||!hex(body.workspace,32)||typeof body.namespace!=="string"||!body.namespace||!hex(body.guard,64)||body.source_effects!==false||typeof body.movable!=="boolean"||entry.version!==1||!hex(entry.identity,32)||typeof entry.generation!=="number"||!Number.isInteger(entry.generation)||entry.generation<0||entry.generation>0xffffffff)throw new Error("Workspace entry response has no supported exact authority.");
 if(path!==null&&(typeof path!=="string"||!path||path.includes('\\')||path.includes(':')||path.split('/').some(part=>!part||part==='.'||part==='..')||/[\u0000-\u001f\u007f]/.test(path)))throw new Error("Workspace entry path is outside the supported source namespace.");
 if(path===null){if(body.source_guard!==undefined||body.source_hash!==undefined||body.movable)throw new Error("An absent entry cannot carry source authority.");}
 else if(typeof body.source_guard!=="string"||!body.source_guard||new TextEncoder().encode(body.source_guard).length>16384||typeof body.source_hash!=="string"||!/^sha256:[0-9a-f]{64}$/.test(body.source_hash))throw new Error("Workspace entry lacks an original physical source proof.");
 const stamp=Object.freeze({version:1 as const,identity:entry.identity,generation:entry.generation,path:path as string|null});
 return Object.freeze({version:1 as const,entry_protocol:1 as const,workspace:body.workspace,namespace:body.namespace,entry:stamp,guard:body.guard,...(path===null?{}:{source_guard:body.source_guard as string,source_hash:body.source_hash as string}),movable:body.movable,source_effects:false as const});
}
export async function readFrozenEntrySelection(load:()=>Promise<unknown>,lease:EntryReadingLease):Promise<FrozenEntrySelection>{
 if(!lease.current())throw new Error("Original source context changed before entry acquisition.");
 const proof=parseEntrySelection(await load());
 if(!lease.current()||proof.workspace!==lease.workspace||proof.namespace!==lease.namespace)throw new Error("Workspace entry response belongs to an obsolete source context.");
 return Object.freeze({entryPath:proof.entry.path,proof,current:()=>lease.current()});
}
</ui:copy>

## Browser: refuse incomplete authority and obsolete replies

The witness checks mutation isolation, strict wire types, positive no-main versus incomplete source authority, and obsolete source context before or during the optional request. It exercises models, not physical input or rendered accessibility.

<ui:copy id="workspace-entry-reading-browser-witness">
import {describe,it,expect} from "vitest";
import {parseEntrySelection,readFrozenEntrySelection} from "./WorkspaceEntrySelection";
function original(){return {version:1,entry_protocol:1,workspace:"a".repeat(32),namespace:"/workspace",entry:{version:1,identity:"b".repeat(32),generation:7,path:"project.md"},guard:"c".repeat(64),source_guard:"opaque physical observation",source_hash:"sha256:"+"d".repeat(64),movable:true,source_effects:false};}
describe("workspace entry reading selection",()=>{
 it("retains an immutable exact original rather than a caller-owned response",()=>{const response=original(),proof=parseEntrySelection(response);response.entry.path="replacement.md";response.namespace="/other";expect(proof.entry.path).toBe("project.md");expect(proof.namespace).toBe("/workspace");expect(Object.isFrozen(proof)).toBe(true);expect(Object.isFrozen(proof.entry)).toBe(true);});
 it("rejects coerced booleans, incomplete source proofs and invalid generations",()=>{const proof=original();for(const change of [{unexpected:true},{source_effects:"false"},{source_effects:0},{movable:1},{workspace:"A".repeat(32)},{guard:"C".repeat(64)},{source_guard:null},{source_hash:"d".repeat(64)},{entry:{...proof.entry,generation:0x100000000}},{entry:{...proof.entry,generation:-1}},{entry:{...proof.entry,generation:1.5}},{entry:{...proof.entry,path:"../project.md"}},{entry:{...proof.entry,path:undefined}}])expect(()=>parseEntrySelection({...proof,...change})).toThrow();});
 it("distinguishes a positive no-main reading from a missing source proof",()=>{const proof=original();expect(()=>parseEntrySelection({...proof,entry:{...proof.entry,path:null}})).toThrow();const withoutSource:Record<string,unknown>={...proof};delete withoutSource.source_guard;delete withoutSource.source_hash;expect(parseEntrySelection({...withoutSource,entry:{...proof.entry,path:null},movable:false}).entry.path).toBeNull();});
 it("rejects stale asynchronous acquisition and keeps the original lease afterward",async()=>{const proof=original();let live=false,reads=0;const lease={workspace:proof.workspace,namespace:proof.namespace,current:()=>live};await expect(readFrozenEntrySelection(async()=>{reads++;return proof},lease)).rejects.toThrow();expect(reads).toBe(0);live=true;await expect(readFrozenEntrySelection(async()=>{live=false;return proof},lease)).rejects.toThrow();live=true;await expect(readFrozenEntrySelection(async()=>({...proof,namespace:"/other"}),lease)).rejects.toThrow();const frozen=await readFrozenEntrySelection(async()=>proof,lease);expect(frozen.entryPath).toBe("project.md");live=false;expect(frozen.current()).toBe(false);});
});
</ui:copy>

## Apple: isolate immutable proof material from mutable JSON

Retain the exact original scope. Reject unsupported fields, literal-type mismatches, invalid generations and incomplete configured-source observations. Returning JSON must not expose mutable storage that can retarget the retained selection; a late response cannot borrow a newer source lease.

<ui:copy id="workspace-entry-reading-apple-helper">
import Foundation
struct WorkspaceEntryStamp: Decodable, Equatable {
 let version: UInt32
 let identity: String
 let generation: UInt32
 let path: String?
}
struct WorkspaceEntrySelectionProof {
 private struct Wire: Decodable {
  let version: UInt32
  let entry_protocol: UInt32
  let workspace: String
  let namespace: String
  let entry: WorkspaceEntryStamp
  let guardValue: String
  let source_guard: String?
  let source_hash: String?
  let movable: Bool
  let source_effects: Bool
  enum CodingKeys:String,CodingKey {case version,entry_protocol,workspace,namespace,entry,source_guard,source_hash,movable,source_effects;case guardValue="guard"}
 }
 private let material: Data
 let workspace: String
 let namespace: String
 let entry: WorkspaceEntryStamp
 init(_ value:Any)throws {
  guard let object=value as? [String:Any],JSONSerialization.isValidJSONObject(object) else {throw EntrySelectionFailure("Workspace entry response is malformed.")}
  guard let entryBody=object["entry"] as? [String:Any],entryBody.keys.contains("path"),object.keys.allSatisfy({["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].contains($0)}),entryBody.keys.allSatisfy({["version","identity","generation","path"].contains($0)}) else {throw EntrySelectionFailure("Workspace entry path field is missing.")}
  let data=try JSONSerialization.data(withJSONObject:object,options:[.sortedKeys]),wire=try JSONDecoder().decode(Wire.self,from:data)
  func hex(_ text:String,_ count:Int)->Bool {text.utf8.count==count && text.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
  guard wire.version==1,wire.entry_protocol==1,hex(wire.workspace,32),!wire.namespace.isEmpty,hex(wire.guardValue,64),!wire.source_effects,wire.entry.version==1,hex(wire.entry.identity,32) else {throw EntrySelectionFailure("Workspace entry response has no supported exact authority.")}
  if let path=wire.entry.path {
   guard !path.isEmpty,!path.contains(":"),!path.contains("\\"),!path.unicodeScalars.contains(where:{$0.value<32||$0.value==127}),!path.components(separatedBy:"/").contains(where:{$0.isEmpty||$0=="."||$0==".."}),let physical=wire.source_guard,!physical.isEmpty,physical.utf8.count<=16384,let hash=wire.source_hash,hash.hasPrefix("sha256:"),hex(String(hash.dropFirst(7)),64) else {throw EntrySelectionFailure("Workspace entry lacks a confined path and original physical source proof.")}
  } else if object.keys.contains("source_guard") || object.keys.contains("source_hash") || wire.movable {throw EntrySelectionFailure("An absent entry cannot carry source or move authority.")}
  material=data;workspace=wire.workspace;namespace=wire.namespace;entry=wire.entry
 }
 func body()throws->Any {try JSONSerialization.jsonObject(with:material)}
}
struct EntrySelectionFailure: LocalizedError {let message:String;init(_ message:String){self.message=message};var errorDescription:String?{message}}
@MainActor struct FrozenWorkspaceEntrySelection {
 let entryPath: String?
 let proof: WorkspaceEntrySelectionProof
 let current: ()->Bool
}
@MainActor enum WorkspaceEntrySelections {
 static func read(load:@MainActor () async throws->Any,workspace:String,namespace:String,current:@escaping @MainActor ()->Bool) async throws->FrozenWorkspaceEntrySelection {
  guard current() else {throw EntrySelectionFailure("Original source context changed before entry acquisition.")}
  let proof=try WorkspaceEntrySelectionProof(await load())
  guard current(),proof.workspace==workspace,proof.namespace==namespace else {throw EntrySelectionFailure("Workspace entry response belongs to an obsolete source context.")}
  return FrozenWorkspaceEntrySelection(entryPath:proof.entry.path,proof:proof,current:current)
 }
}
</ui:copy>

## Apple: keep the original source lease across suspension

The witness checks mutation isolation, strict wire types, positive no-main versus incomplete source authority, and obsolete source context before or during the optional request. It exercises models, not physical input or rendered accessibility.

<ui:copy id="workspace-entry-reading-apple-witness">
import XCTest
@testable import HickorySimple
final class WorkspaceEntrySelectionTests:XCTestCase {
 private func original()->[String:Any]{["version":1,"entry_protocol":1,"workspace":String(repeating:"a",count:32),"namespace":"/workspace","entry":["version":1,"identity":String(repeating:"b",count:32),"generation":7,"path":"project.md"],"guard":String(repeating:"c",count:64),"source_guard":"opaque physical observation","source_hash":"sha256:"+String(repeating:"d",count:64),"movable":true,"source_effects":false]}
 func testOriginalProofDoesNotFollowMutableResponseOrReturnedBody()throws {var response=original();let proof=try WorkspaceEntrySelectionProof(response);response["namespace"]="/other";var body=try XCTUnwrap(proof.body() as? [String:Any]);body["namespace"]="/mutated";XCTAssertEqual(proof.namespace,"/workspace");XCTAssertEqual(proof.entry.path,"project.md");XCTAssertEqual((try proof.body() as? [String:Any])?["namespace"] as? String,"/workspace")}
 func testStrictTypesAndGenerationsRefuseIncompleteAuthority(){for (key,value) in [("unexpected",1 as Any),("source_effects","false" as Any),("source_effects",0 as Any),("movable",1 as Any),("source_guard",NSNull()),("source_hash","wrong" as Any),("guard",String(repeating:"C",count:64) as Any)]{var response=original();response[key]=value;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response))};for generation in [-1,4_294_967_296]{var response=original(),entry=response["entry"] as! [String:Any];entry["generation"]=generation;response["entry"]=entry;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response))}}
 func testPositiveAbsenceCannotBorrowSourceOrMoveAuthority()throws {var response=original(),entry=response["entry"] as! [String:Any];entry["path"]=NSNull();response["entry"]=entry;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response));response.removeValue(forKey:"source_guard");response.removeValue(forKey:"source_hash");response["movable"]=false;XCTAssertNil(try WorkspaceEntrySelectionProof(response).entry.path)}
 @MainActor func testAsynchronousAcquisitionNeverRebindsOriginalLease()async throws {let response=original();let workspace=response["workspace"] as! String;var live=false,reads=0;do{_ = try await WorkspaceEntrySelections.read(load:{reads+=1;return response},workspace:workspace,namespace:"/workspace",current:{live});XCTFail("stale source must refuse before request")}catch{};XCTAssertEqual(reads,0);live=true;do{_ = try await WorkspaceEntrySelections.read(load:{live=false;return response},workspace:workspace,namespace:"/workspace",current:{live});XCTFail("late reply must refuse")}catch{};live=true;let frozen=try await WorkspaceEntrySelections.read(load:{response},workspace:workspace,namespace:"/workspace",current:{live});XCTAssertEqual(frozen.entryPath,"project.md");live=false;XCTAssertFalse(frozen.current())}
}
</ui:copy>

## Android: check literal wire types before retaining proof

Retain the exact original scope. Reject unsupported fields, literal-type mismatches, invalid generations and incomplete configured-source observations. Returning JSON must not expose mutable storage that can retarget the retained selection; a late response cannot borrow a newer source lease.

<ui:copy id="workspace-entry-reading-android-helper">
package com.hickorydocs.simple
import org.json.JSONObject

data class WorkspaceEntryStamp(val version:Long,val identity:String,val generation:Long,val path:String?)
class WorkspaceEntrySelectionProof(value:JSONObject) {
 private val material=value.toString()
 val workspace:String
 val namespace:String
 val entry:WorkspaceEntryStamp
 init {
  val body=JSONObject(material)
  require(body.keys().asSequence().all{it in setOf("version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects")})
  fun text(value:JSONObject,key:String)=value.opt(key) as? String ?: error("Workspace entry $key must be an exact string.")
  fun integer(value:JSONObject,key:String):Long {val raw=value.opt(key);require(raw is Int||raw is Long){"Workspace entry $key must be an exact integer."};return (raw as Number).toLong()}
  fun hex(value:String,count:Int)=value.length==count&&value.all{it in '0'..'9'||it in 'a'..'f'}
  require(integer(body,"version")==1L&&integer(body,"entry_protocol")==1L)
  workspace=text(body,"workspace");namespace=text(body,"namespace")
  require(hex(workspace,32)&&namespace.isNotEmpty()&&hex(text(body,"guard"),64))
  require(body.opt("source_effects") is Boolean&&body.get("source_effects")==false)
  require(body.opt("movable") is Boolean)
  val source=body.opt("entry") as? JSONObject ?: error("Workspace entry stamp is malformed.")
  require(source.keys().asSequence().all{it in setOf("version","identity","generation","path")})
  val generation=integer(source,"generation");val identity=text(source,"identity")
  require(integer(source,"version")==1L&&hex(identity,32)&&generation in 0L..0xffffffffL&&source.has("path"))
  val rawPath=source.get("path");val path=if(rawPath===JSONObject.NULL)null else rawPath as? String ?: error("Workspace entry path is malformed.")
  if(path==null){require(!body.has("source_guard")&&!body.has("source_hash")&&body.get("movable")==false)}
  else {
   require(path.isNotEmpty()&&!path.contains(':')&&!path.contains('\\')&&path.none{it.code<32||it.code==127}&&path.split('/').none{it.isEmpty()||it=="."||it==".."})
   val guard=text(body,"source_guard");val hash=text(body,"source_hash")
   require(guard.isNotEmpty()&&guard.toByteArray(Charsets.UTF_8).size<=16384&&hash.startsWith("sha256:")&&hex(hash.removePrefix("sha256:"),64))
  }
  entry=WorkspaceEntryStamp(1,identity,generation,path)
 }
 fun body()=JSONObject(material)
}
data class FrozenWorkspaceEntrySelection(val entryPath:String?,val proof:WorkspaceEntrySelectionProof,val current:()->Boolean)
object WorkspaceEntrySelections {
 suspend fun read(load:suspend()->JSONObject,workspace:String,namespace:String,current:()->Boolean):FrozenWorkspaceEntrySelection {
  check(current()){"Original source context changed before entry acquisition."}
  val proof=WorkspaceEntrySelectionProof(load())
  check(current()&&proof.workspace==workspace&&proof.namespace==namespace){"Workspace entry response belongs to an obsolete source context."}
  return FrozenWorkspaceEntrySelection(proof.entry.path,proof,current)
 }
}
</ui:copy>

## Android: reject coercion and stale acquisition

The witness checks mutation isolation, strict wire types, positive no-main versus incomplete source authority, and obsolete source context before or during the optional request. It exercises models, not physical input or rendered accessibility.

<ui:copy id="workspace-entry-reading-android-witness">
package com.hickorydocs.simple
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
class WorkspaceEntrySelectionTest {
 private fun original()=JSONObject().put("version",1).put("entry_protocol",1).put("workspace","a".repeat(32)).put("namespace","/workspace").put("entry",JSONObject().put("version",1).put("identity","b".repeat(32)).put("generation",7).put("path","project.md")).put("guard","c".repeat(64)).put("source_guard","opaque physical observation").put("source_hash","sha256:"+"d".repeat(64)).put("movable",true).put("source_effects",false)
 private fun refuses(action:()->Unit){try{action();fail("Malformed entry authority was accepted")}catch(expected:IllegalArgumentException){}catch(expected:IllegalStateException){}}
 @Test fun originalProofDoesNotFollowMutableResponseOrReturnedBody(){val response=original();val proof=WorkspaceEntrySelectionProof(response);response.put("namespace","/other");proof.body().put("namespace","/mutated");assertEquals("/workspace",proof.namespace);assertEquals("project.md",proof.entry.path);assertEquals("/workspace",proof.body().getString("namespace"))}
 @Test fun strictTypesAndGenerationBoundsRefuseIncompleteAuthority(){for((key,value)in listOf("unexpected" to 1,"source_effects" to "false","source_effects" to 0,"movable" to 1,"source_guard" to JSONObject.NULL,"source_hash" to "wrong","guard" to "C".repeat(64))){refuses{WorkspaceEntrySelectionProof(original().put(key,value))}};for(value in listOf(-1L,0x100000000L)){val body=original();body.getJSONObject("entry").put("generation",value);refuses{WorkspaceEntrySelectionProof(body)}}}
 @Test fun positiveAbsenceCannotBorrowSourceOrMoveAuthority(){val response=original();response.getJSONObject("entry").put("path",JSONObject.NULL);refuses{WorkspaceEntrySelectionProof(response)};response.remove("source_guard");response.remove("source_hash");response.put("movable",false);assertNull(WorkspaceEntrySelectionProof(response).entry.path)}
 @Test fun asynchronousAcquisitionNeverRebindsOriginalLease()=runBlocking {val response=original();var live=false;var reads=0;try{WorkspaceEntrySelections.read({reads++;response},"a".repeat(32),"/workspace"){live};fail("stale source must refuse before request")}catch(expected:IllegalStateException){};assertEquals(0,reads);live=true;try{WorkspaceEntrySelections.read({live=false;response},"a".repeat(32),"/workspace"){live};fail("late reply must refuse")}catch(expected:IllegalStateException){};live=true;val frozen=WorkspaceEntrySelections.read({response},"a".repeat(32),"/workspace"){live};assertEquals("project.md",frozen.entryPath);live=false;assertFalse(frozen.current())}
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceEntrySelection.ts"><ui:paste select="#workspace-entry-reading-browser-helper"/></ui:file>
<ui:file path="apps/web/src/simple/WorkspaceEntrySelection.test.ts"><ui:paste select="#workspace-entry-reading-browser-witness"/></ui:file>
<ui:file path="apps/apple-simple/Sources/WorkspaceEntrySelection.swift"><ui:paste select="#workspace-entry-reading-apple-helper"/></ui:file>
<ui:file path="apps/apple-simple/Tests/WorkspaceEntrySelectionTests.swift"><ui:paste select="#workspace-entry-reading-apple-witness"/></ui:file>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/WorkspaceEntrySelection.kt"><ui:paste select="#workspace-entry-reading-android-helper"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/WorkspaceEntrySelectionTest.kt"><ui:paste select="#workspace-entry-reading-android-witness"/></ui:file>
</ui:doc>
