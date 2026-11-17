<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Validate provider coordinates against frozen source

For maintainers admitting provider actions across browser, Apple and Android hosts.

A provider action’s edit positions are useful only for the exact authored bytes seen by the engine. The backend-held opaque action and engine-issued `data.hickory_sources` rows travel together. Discover these context paths before a fresh query, freeze their snapshots, and reject a missing, malformed or mismatched proof before preview or application. Existing edited preimages must all be covered; the caller determines ordered resource preimages and excludes only destinations created by that action. Context hashes grant no write permission. Frontend-local proposals and direct rename/format responses need their own authority path; this mechanism admits engine-stamped provider CodeAction values only.

## Browser coordinate authority

Parse before acquiring. Proof rows are engine evidence, so malformed or missing stamps refuse rather than fall back to live editor bytes. The existing local semantic URI rules keep external and generated identities out. Hash checks are asynchronous and grant no writable authority.

<ui:copy id="provider-source-proofs-browser">
import type {SourceSnapshot} from './ChangePlan';
import {semanticPath} from './Intelligence';
import {sha256} from '../lib/reviews';
export interface ProviderSourceProof {uri:string;path:string;sourceHash:string}
const record=(value:unknown):value is Record<string,unknown>=>!!value&&typeof value==='object'&&!Array.isArray(value);
export function providerSourceProofs(action:unknown):ProviderSourceProof[]{
 const data=record(action)&&record(action.data)?action.data:null;
 if(!data||typeof data.hickory_action!=='string'||! /^[\x21-\x7e]{1,1024}$/.test(data.hickory_action)||!Array.isArray(data.hickory_sources)||!data.hickory_sources.length||data.hickory_sources.length>10000)throw new Error('Missing or malformed engine source proof. Prepare the provider action again.');
 const seen=new Set<string>();let previous='';return data.hickory_sources.map(raw=>{
  if(!record(raw)||Object.keys(raw).sort().join(',')!=='source_hash,uri'||typeof raw.uri!=='string'||! /^[\x21-\x7e]+$/.test(raw.uri)||typeof raw.source_hash!=='string'||! /^sha256:[a-f0-9]{64}$/.test(raw.source_hash))throw new Error('Malformed engine source proof row.');
  const path=semanticPath(raw.uri);if(!path||raw.uri<=previous||seen.has(path))throw new Error('Engine source proof identities are unsupported, duplicated or unsorted.');
  previous=raw.uri;seen.add(path);return {uri:raw.uri,path,sourceHash:raw.source_hash};
 });
}
export function providerSourceProofPaths(action:unknown):string[]{return providerSourceProofs(action).map(row=>row.path);}
export async function validateProviderSourceProofs(action:unknown,sources:ReadonlyMap<string,SourceSnapshot>,requiredExisting:ReadonlySet<string>):Promise<void>{
 const rows=providerSourceProofs(action),covered=new Set(rows.map(row=>row.path));
 for(const path of requiredExisting)if(!covered.has(path))throw new Error(`Edited source lacks engine coordinate proof: ${path}. Prepare again.`);
 for(const row of rows){const source=sources.get(row.path);if(!source)throw new Error(`Acquire and synchronize provider proof source before querying again: ${row.path}`);if('sha256:'+await sha256(source.text)!==row.sourceHash)throw new Error(`Provider source proof changed: ${row.path}. Prepare again.`);}
}
</ui:copy>

<ui:file path="apps/web/src/simple/ProviderSourceProofs.ts"><ui:paste select="#provider-source-proofs-browser"/></ui:file>

## Browser refusal witnesses

The tests distinguish context from edit permission, demand coverage for edited preimages, reject alias/sort/schema errors, and compare hashes with frozen text rather than a replaced draft.

<ui:copy id="provider-source-proofs-browser-witness">
import {expect,it} from 'vitest';
import {sha256} from '../lib/reviews';
import {providerSourceProofPaths,providerSourceProofs,validateProviderSourceProofs} from './ProviderSourceProofs';
const hash='sha256:'+'0'.repeat(64);
const action=(rows:unknown)=>({data:{hickory_action:'opaque:1',hickory_sources:rows}});
const row=(uri='hick:///a',source_hash=hash)=>({uri,source_hash});
it('discovers sorted proof context without granting edit authority',()=>{expect(providerSourceProofPaths(action([row(),row('hick:///b')]))).toEqual(['a','b']);});
it('refuses missing stamps and malformed proof schema, hashes, order and identities',()=>{
 for(const value of [{},action([]),action([row('file:///a')]),action([row('hick:///a?x')]),action([row('hick:///../a')]),action([row('hick:///a','SHA256:'+'0'.repeat(64))]),action([row('hick:///a','sha256:'+'A'.repeat(64))]),action([{...row(),extra:true}]),action([row(),row()]),action([row('hick:///b'),row()]),action([row('hick:///%61'),row()])])expect(()=>providerSourceProofs(value)).toThrow();
});
it('validates readonly context and edited preimages against frozen UTF8 bytes',async()=>{
 const original={text:'é😀',writable:false},proof=action([row('hick:///a','sha256:'+await sha256(original.text))]);await validateProviderSourceProofs(proof,new Map([['a',original]]),new Set(['a']));
 await expect(validateProviderSourceProofs(proof,new Map([['a',{text:'e\u0301😀',writable:false}]]),new Set())).rejects.toThrow('changed');
 await expect(validateProviderSourceProofs(proof,new Map(),new Set())).rejects.toThrow('Acquire');
 await expect(validateProviderSourceProofs(proof,new Map([['a',original]]),new Set(['created-only-if-misclassified']))).rejects.toThrow('lacks');
});
it('does not require proof for explicitly excluded created destinations',async()=>{const proof=action([row('hick:///a','sha256:'+await sha256('before'))]);await validateProviderSourceProofs(proof,new Map([['a',{text:'before',writable:true}]]),new Set(['a']));});
</ui:copy>

<ui:file path="apps/web/src/simple/ProviderSourceProofs.test.ts"><ui:paste select="#provider-source-proofs-browser-witness"/></ui:file>

## Apple coordinate authority

Use byte identities for path uniqueness and source hashing. Swift canonical string equivalence cannot merge distinct authored names or bless a normalization-only source change. The caller passes existing edit preimages as an array, interpreted as an exact byte set.

<ui:copy id="provider-source-proofs-apple">
import Foundation
struct ProviderSourceProof {var uri:String;var path:String;var sourceHash:String}
func providerSourceProofs(_ action:Any)throws->[ProviderSourceProof] {
 guard let action=action as? [String:Any],let data=action["data"] as? [String:Any],let token=data["hickory_action"] as? String,!token.isEmpty,token.utf8.count<=1024,token.utf8.allSatisfy({(33...126).contains($0)}),let rows=data["hickory_sources"] as? [[String:Any]],!rows.isEmpty,rows.count<=10000 else{throw EngineError(message:"Missing or malformed engine source proof. Prepare the provider action again.")}
 var seen=Set<NativeSourceKey>();var previous:[UInt8]=[];var result:[ProviderSourceProof]=[]
 for row in rows {
  guard Set(row.keys)==Set(["uri","source_hash"]),let uri=row["uri"] as? String,!uri.isEmpty,uri.utf8.allSatisfy({(33...126).contains($0)}),let hash=row["source_hash"] as? String,hash.utf8.count==71,hash.range(of:"^sha256:[a-f0-9]{64}$",options:.regularExpression) != nil else{throw EngineError(message:"Malformed engine source proof row.")}
  let bytes=Array(uri.utf8)
  guard let path=SemanticLocation(uri:uri,line:0,character:0).path,previous.lexicographicallyPrecedes(bytes),seen.insert(NativeSourceKey(path)).inserted else{throw EngineError(message:"Engine source proof identities are unsupported, duplicated or unsorted.")}
  previous=bytes;result.append(ProviderSourceProof(uri:uri,path:path,sourceHash:hash))
 }
 return result
}
func providerSourceProofPaths(_ action:Any)throws->[String] {try providerSourceProofs(action).map(\.path)}
func validateProviderSourceProofs(_ action:Any,sources:NativeSourceMap<SourceSnapshot>,requiredExisting:[String])throws {
 let rows=try providerSourceProofs(action),covered=Set(rows.map{NativeSourceKey($0.path)})
 for path in requiredExisting where !covered.contains(NativeSourceKey(path)){throw EngineError(message:"Edited source lacks engine coordinate proof: \(path). Prepare again.")}
 for row in rows {
  guard let source=sources[row.path] else{throw EngineError(message:"Acquire and synchronize provider proof source before querying again: \(row.path)")}
  guard exactUTF8("sha256:"+nativeSourceHash(source.text),row.sourceHash) else{throw EngineError(message:"Provider source proof changed: \(row.path). Prepare again.")}
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ProviderSourceProofs.swift"><ui:paste select="#provider-source-proofs-apple"/></ui:file>

## Apple refusal witnesses

Exercise exact source bytes, read-only context, missing edited preimages and malformed evidence independently of UI adapters.

<ui:copy id="provider-source-proofs-apple-witness">
import XCTest
@testable import HickorySimple
final class ProviderSourceProofsTests:XCTestCase {
 func row(_ uri:String="hick:///a",_ hash:String="sha256:"+String(repeating:"0",count:64))->[String:Any]{["uri":uri,"source_hash":hash]}
 func action(_ rows:Any)->[String:Any]{["data":["hickory_action":"opaque:1","hickory_sources":rows]]}
 func testDiscoversContextAndRejectsMalformedEvidence()throws {
  XCTAssertEqual(try providerSourceProofPaths(action([row(),row("hick:///b")])),["a","b"])
  let bad:[Any]=[[:],action([]),action([row("file:///a")]),action([row("hick:///a?x")]),action([row("hick:///../a")]),action([row("hick:///a","sha256:"+String(repeating:"A",count:64))]),action([["uri":"hick:///a","source_hash":"bad","extra":true]]),action([row(),row()]),action([row("hick:///b"),row()]),action([row("hick:///%61"),row()])]
  for value in bad{XCTAssertThrowsError(try providerSourceProofs(value))}
 }
 func testFrozenUTF8ContextHashAndExistingEditCoverage()throws {
  let original=SourceSnapshot(text:"é😀",writable:false),proof=action([row("hick:///a","sha256:"+nativeSourceHash("é😀"))])
  try validateProviderSourceProofs(proof,sources:["a":original],requiredExisting:["a"])
  XCTAssertThrowsError(try validateProviderSourceProofs(proof,sources:["a":SourceSnapshot(text:"e\u{301}😀",writable:false)],requiredExisting:[]))
  XCTAssertThrowsError(try validateProviderSourceProofs(proof,sources:[:],requiredExisting:[]))
  XCTAssertThrowsError(try validateProviderSourceProofs(proof,sources:["a":original],requiredExisting:["unproved-existing"]))
  try validateProviderSourceProofs(proof,sources:["a":original],requiredExisting:[]) // A caller excludes newly created destinations.
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ProviderSourceProofsTests.swift"><ui:paste select="#provider-source-proofs-apple-witness"/></ui:file>

## Android coordinate authority

The Android parser shares local semantic URI policy and preserves exact decoded path identity. Hash evidence validates captured UTF-8 bytes and never changes a snapshot’s write permission.

<ui:copy id="provider-source-proofs-android">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
data class ProviderSourceProof(val uri:String,val path:String,val sourceHash:String)
fun providerSourceProofs(action:Any?):List<ProviderSourceProof> {
 val data=(action as? JSONObject)?.optJSONObject("data") ?: error("Missing or malformed engine source proof. Prepare the provider action again.")
 val token=data.opt("hickory_action") as? String
 val rows=data.opt("hickory_sources") as? JSONArray
 require(token!=null&&token.matches(Regex("[\\x21-\\x7e]{1,1024}"))&&rows!=null&&rows.length() in 1..10000){"Missing or malformed engine source proof. Prepare the provider action again."}
 val seen=mutableSetOf<String>();var previous="";val result=mutableListOf<ProviderSourceProof>()
 for(index in 0 until rows.length()){
  val row=rows.opt(index) as? JSONObject ?: error("Malformed engine source proof row.")
  val uri=row.opt("uri") as? String;val hash=row.opt("source_hash") as? String
  require(row.keys().asSequence().toSet()==setOf("uri","source_hash")&&uri!=null&&uri.matches(Regex("[\\x21-\\x7e]+"))&&hash!=null&&hash.matches(Regex("sha256:[a-f0-9]{64}"))){"Malformed engine source proof row."}
  val path=SemanticLocation(uri,0,0).path ?: error("Unsupported engine source proof identity.")
  require(uri>previous&&seen.add(path)){"Engine source proof identities are duplicated or unsorted."};previous=uri;result.add(ProviderSourceProof(uri,path,hash))
 }
 return result
}
fun providerSourceProofPaths(action:Any?):List<String> = providerSourceProofs(action).map{it.path}
fun validateProviderSourceProofs(action:Any?,sources:Map<String,SourceSnapshot>,requiredExisting:Set<String>) {
 val rows=providerSourceProofs(action);val covered=rows.map{it.path}.toSet()
 for(path in requiredExisting)require(path in covered){"Edited source lacks engine coordinate proof: $path. Prepare again."}
 for(row in rows){val source=sources[row.path] ?: error("Acquire and synchronize provider proof source before querying again: ${row.path}");require("sha256:"+debugHash(source.text)==row.sourceHash){"Provider source proof changed: ${row.path}. Prepare again."}}
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ProviderSourceProofs.kt"><ui:paste select="#provider-source-proofs-android"/></ui:file>

## Android refusal witnesses

Provider coordinate proof has the same boundaries across hosts: context may be read-only, but every existing edited source needs matching frozen bytes.

<ui:copy id="provider-source-proofs-android-witness">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
import org.junit.Test
import org.junit.Assert.*
class ProviderSourceProofsTest {
 fun row(uri:String="hick:///a",hash:String="sha256:"+"0".repeat(64))=JSONObject().put("uri",uri).put("source_hash",hash)
 fun action(rows:Any)=JSONObject().put("data",JSONObject().put("hickory_action","opaque:1").put("hickory_sources",rows))
 fun rejects(block:()->Unit){try{block();fail("malformed or unproved action accepted")}catch(expected:IllegalArgumentException){}catch(expected:IllegalStateException){}}
 @Test fun discoversContextAndRejectsMalformedEvidence(){
  assertEquals(listOf("a","b"),providerSourceProofPaths(action(JSONArray().put(row()).put(row("hick:///b")))))
  val bad=listOf(JSONObject(),action(JSONArray()),action(JSONArray().put(row("file:///a"))),action(JSONArray().put(row("hick:///a?x"))),action(JSONArray().put(row("hick:///../a"))),action(JSONArray().put(row("hick:///a","sha256:"+"A".repeat(64)))),action(JSONArray().put(row().put("extra",true))),action(JSONArray().put(row()).put(row())),action(JSONArray().put(row("hick:///b")).put(row())),action(JSONArray().put(row("hick:///%61")).put(row())))
  for(value in bad)rejects{providerSourceProofs(value)}
 }
 @Test fun frozenUTF8ContextHashAndExistingEditCoverage(){
  val original=SourceSnapshot("é😀",false);val proof=action(JSONArray().put(row("hick:///a","sha256:"+debugHash(original.text))))
  validateProviderSourceProofs(proof,mapOf("a" to original),setOf("a"))
  rejects{validateProviderSourceProofs(proof,mapOf("a" to SourceSnapshot("e\u0301😀",false)),emptySet())}
  rejects{validateProviderSourceProofs(proof,emptyMap(),emptySet())}
  rejects{validateProviderSourceProofs(proof,mapOf("a" to original),setOf("unproved-existing"))}
  validateProviderSourceProofs(proof,mapOf("a" to original),emptySet()) // Caller excludes created destinations.
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ProviderSourceProofsTest.kt"><ui:paste select="#provider-source-proofs-android-witness"/></ui:file>

</ui:doc>
