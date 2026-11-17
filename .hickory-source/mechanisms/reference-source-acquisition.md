<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Acquire referenced source before preparing a change

For maintainers expanding reviewed changes beyond visible or retained editors.

An initial reference query discovers targets; it cannot authorize edits against text loaded afterward. Acquire and synchronize every discovered target once, including retained drafts, then query again. Keep original snapshots, refuse any text or writable-state drift, deduplicate targets and acquire sequentially so one newly opened language owner cannot retire another pending owner. Obsolete intent stops subsequent dispatches after the current read settles. Four expanding query rounds or more than 10,000 targets refuse explicitly. This establishes the observed reference set, not exhaustive project analysis. Host callbacks validate workspace-local ownership and run on the owning UI context; reading source never navigates, saves or applies changes.

A proposal query carries both discovered paths and its original value. Only a query whose paths have all completed synchronization can return that value. This same acquisition boundary serves references and provider edits; a discovering edit is discarded after source acquisition, because synchronizing new source may invalidate the provider artifact that produced it.

## reference-source-acquisition-browser

The provider query and source acquisition remain replaceable callbacks. The optional `requireWritable` policy defaults to editing authority; callers may admit read-only context explicitly, while any change to an original writable state still refuses. Every discovered path must complete one acquisition before a subsequent query can return a proposal. Retained text is compared byte-for-byte and never replaced; acquiring is also the host’s provider synchronization boundary.

<ui:copy id="reference-source-acquisition-browser">
import type {SourceSnapshot} from './ChangePlan';
/** Discovery never authorizes edits. Every discovered target synchronizes before a fresh query. */
export async function acquireSourceProposal<T>(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<{paths:string[];value:T}>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean,requireWritable:(path:string)=>boolean=()=>true){
 const sources=new Map(initial),synchronized=new Set<string>();let references:string[]=[];
 const check=()=>{if(!current())throw new Error('Source, selection or provider changed. Prepare again.');};
 for(let round=0;round<4;round++){
  check();const proposal=await query();references=[...new Set(proposal.paths)];check();if(references.length>10000)throw new Error('Reference discovery exceeds 10,000 source targets. Narrow the change.');
  const pending=references.filter(path=>!synchronized.has(path));if(!pending.length)return {sources,paths:references,value:proposal.value};
  for(const path of pending){
   check();const value=await acquire(path);check();const original=sources.get(path);
   if(original&&(original.text!==value.text||original.writable!==value.writable))throw new Error(`Source changed while synchronizing: ${path}. Prepare again.`);
   if(requireWritable(path)&&!value.writable)throw new Error(`Source is generated, read-only or saving: ${path}`);
   if(!original)sources.set(path,value);synchronized.add(path);
  }
  check();
 }
 throw new Error('Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.');
}
export async function acquireReferenceSources(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<string[]>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean,requireWritable:(path:string)=>boolean=()=>true){
 const result=await acquireSourceProposal(initial,async()=>({paths:await query(),value:null}),acquire,current,requireWritable);
 return {sources:result.sources,references:result.paths};
}
</ui:copy>

<ui:file path="apps/web/src/simple/ReferenceSources.ts"><ui:paste select="#reference-source-acquisition-browser"/></ui:file>

## reference-source-acquisition-browser-witness

Hold source completions to prove sequential dispatch, duplicate removal and a fresh query after both new and retained sources synchronize. Original drafts survive unchanged; text or writable-state drift refuses. Obsolete intent stops subsequent reads and repeated expansion or excessive targets never return a proposal.

<ui:copy id="reference-source-acquisition-browser-witness">
import {expect,it} from 'vitest';
import {acquireReferenceSources,acquireSourceProposal} from './ReferenceSources';
function held(){let resolve!:(value:{text:string;writable:boolean})=>void;const promise=new Promise<{text:string;writable:boolean}>(done=>resolve=done);return {promise,resolve};}
const flush=async()=>{for(let i=0;i<6;i++)await Promise.resolve();};
it('synchronizes retained drafts once and returns only a fresh proposal without replacing snapshots',async()=>{
 const original={text:'original draft',writable:true};let round=0;const reads:string[]=[];
 const result=await acquireSourceProposal(new Map([['retained',original]]),async()=>({paths:['retained','retained'],value:++round}),async path=>{reads.push(path);return {...original};},()=>true);
 expect(result.value).toBe(2);expect(reads).toEqual(['retained']);expect(result.sources.get('retained')).toBe(original);
});
it('acquires new and retained targets sequentially, deduplicates and queries again',async()=>{
 let queries=0;const reads:{path:string;done:ReturnType<typeof held>}[]=[];
 const task=acquireReferenceSources(new Map([['retained',{text:'original draft',writable:true}]]),async()=>{queries++;return ['a','b','a','retained'];},path=>{const done=held();reads.push({path,done});return done.promise;},()=>true);
 await flush();expect(reads.map(row=>row.path)).toEqual(['a']);reads[0].done.resolve({text:'one',writable:true});await flush();expect(reads.map(row=>row.path)).toEqual(['a','b']);expect(queries).toBe(1);
 reads[1].done.resolve({text:'two',writable:true});await flush();expect(reads.map(row=>row.path)).toEqual(['a','b','retained']);reads[2].done.resolve({text:'original draft',writable:true});const result=await task;expect(queries).toBe(2);expect(result.references).toEqual(['a','b','retained']);
});
it('refuses retained text or writable drift and readonly new sources',async()=>{
 for(const next of [{text:'changed',writable:true},{text:'original',writable:false}])await expect(acquireReferenceSources(new Map([['a',{text:'original',writable:true}]]),async()=>['a'],async()=>next,()=>true)).rejects.toThrow('changed');
 await expect(acquireReferenceSources(new Map(),async()=>['a'],async()=>({text:'source',writable:false}),()=>true)).rejects.toThrow('read-only');
});
it('stale acquisition settles its read without dispatching another source or query',async()=>{
 let current=true,queries=0;const reads:ReturnType<typeof held>[]=[];const task=acquireReferenceSources(new Map(),async()=>{queries++;return ['a','b','c'];},()=>{const read=held();reads.push(read);return read.promise;},()=>current);const rejected=expect(task).rejects.toThrow('changed');await flush();current=false;reads[0].resolve({text:'source',writable:true});await rejected;expect(reads).toHaveLength(1);expect(queries).toBe(1);
});
it('refuses expanding and oversized discovery without dispatching past the bound',async()=>{
 let round=0;await expect(acquireReferenceSources(new Map(),async()=>[`new-${round++}`],async()=>({text:'source',writable:true}),()=>true)).rejects.toThrow('kept expanding');expect(round).toBe(4);
 let reads=0;await expect(acquireReferenceSources(new Map(),async()=>Array.from({length:10001},(_,i)=>String(i)),async()=>{reads++;return {text:'',writable:true};},()=>true)).rejects.toThrow('10,000');expect(reads).toBe(0);
});
it('admits readonly context only with explicit nonediting policy and still refuses drift',async()=>{
 const readonly={text:'context',writable:false};const result=await acquireSourceProposal(new Map([['context',readonly]]),async()=>({paths:['context'],value:1}),async()=>({...readonly}),()=>true,()=>false);expect(result.sources.get('context')).toBe(readonly);
 await expect(acquireSourceProposal(new Map([['context',readonly]]),async()=>({paths:['context'],value:1}),async()=>({text:'context',writable:true}),()=>true,()=>false)).rejects.toThrow('changed');
});
</ui:copy>

<ui:file path="apps/web/src/simple/ReferenceSources.test.ts"><ui:paste select="#reference-source-acquisition-browser-witness"/></ui:file>

## reference-source-acquisition-apple

The provider query and source acquisition remain replaceable callbacks. The optional `requireWritable` policy defaults to editing authority; callers may admit read-only context explicitly, while any change to an original writable state still refuses. Every discovered path must complete one acquisition before a subsequent query can return a proposal. Retained text is compared byte-for-byte and never replaced; acquiring is also the host’s provider synchronization boundary.

<ui:copy id="reference-source-acquisition-apple">
import Foundation
struct ReferenceSourceAcquisition {var sources:NativeSourceMap<SourceSnapshot>;var references:[String]}
struct SourceProposalAcquisition<Value> {var sources:NativeSourceMap<SourceSnapshot>;var paths:[String];var value:Value}
@MainActor func acquireSourceProposal<Value>(_ initial:NativeSourceMap<SourceSnapshot>,query:@escaping () async throws->(paths:[String],value:Value),acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool,requireWritable:@escaping (String)->Bool={_ in true}) async throws->SourceProposalAcquisition<Value> {
    var sources=initial;var synchronized=Set<NativeSourceKey>()
    func check()throws{guard current() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}}
    for _ in 0..<4 {
        try check();let proposal=try await query(),references=Set(proposal.paths.map(NativeSourceKey.init)).map(\.string).sorted();try check()
        guard references.count<=10000 else{throw EngineError(message:"Reference discovery exceeds 10,000 source targets. Narrow the change.")}
        let pending=references.filter {!synchronized.contains(NativeSourceKey($0))};if pending.isEmpty{return SourceProposalAcquisition(sources:sources,paths:references,value:proposal.value)}
        for path in pending {
            try check();let value=try await acquire(path);try check()
            if let original=sources[path],!original.text.utf8.elementsEqual(value.text.utf8)||original.writable != value.writable{throw EngineError(message:"Source changed while synchronizing: \(path). Prepare again.")}
            guard !requireWritable(path)||value.writable else{throw EngineError(message:"Source is generated, read-only or saving: \(path)")}
            if sources[path]==nil{sources[path]=value};synchronized.insert(NativeSourceKey(path))
        }
        try check()
    }
    throw EngineError(message:"Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
@MainActor func acquireReferenceSources(_ initial:NativeSourceMap<SourceSnapshot>,query:@escaping () async throws->[String],acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool,requireWritable:@escaping (String)->Bool={_ in true}) async throws->ReferenceSourceAcquisition {
    let result=try await acquireSourceProposal(initial,query:{(paths:try await query(),value:false)},acquire:acquire,current:current,requireWritable:requireWritable)
    return ReferenceSourceAcquisition(sources:result.sources,references:result.paths)
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ReferenceSources.swift"><ui:paste select="#reference-source-acquisition-apple"/></ui:file>

## reference-source-acquisition-apple-witness

Hold source completions to prove sequential dispatch, duplicate removal and a fresh query after both new and retained sources synchronize. Original drafts survive unchanged; text or writable-state drift refuses. Obsolete intent stops subsequent reads and repeated expansion or excessive targets never return a proposal.

<ui:copy id="reference-source-acquisition-apple-witness">
import XCTest
@testable import HickorySimple
@MainActor final class ReferenceSourcesTests:XCTestCase {
    func testRetainedDraftSynchronizesOnceBeforeFreshProposal()async throws {
        var round=0;var reads:[String]=[]
        let result=try await acquireSourceProposal(["retained":SourceSnapshot(text:"original draft",writable:true)],query:{round+=1;return (paths:["retained","retained"],value:round)},acquire:{path in reads.append(path);return SourceSnapshot(text:"original draft",writable:true)},current:{true})
        XCTAssertEqual(result.value,2);XCTAssertEqual(reads,["retained"]);XCTAssertEqual(result.sources["retained"]?.text,"original draft")
    }
    struct Read {var path:String;var continuation:CheckedContinuation<SourceSnapshot,Error>}
    func settle(_ condition:@escaping ()->Bool)async{let deadline=Date().addingTimeInterval(2);while !condition()&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
    func testSequentialAcquisitionDeduplicatesAndRequeries()async throws {
        var queries=0;var reads:[Read]=[]
        let task=Task {try await acquireReferenceSources(["retained":SourceSnapshot(text:"original draft",writable:true)],query:{queries+=1;return ["a","b","a","retained"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{true})}
        await settle{reads.count==1};XCTAssertEqual(reads.map(\.path),["a"]);reads[0].continuation.resume(returning:SourceSnapshot(text:"one",writable:true));await settle{reads.count==2};XCTAssertEqual(reads.map(\.path),["a","b"]);XCTAssertEqual(queries,1)
        reads[1].continuation.resume(returning:SourceSnapshot(text:"two",writable:true));await settle{reads.count==3};XCTAssertEqual(reads[2].path,"retained");reads[2].continuation.resume(returning:SourceSnapshot(text:"original draft",writable:true));let result=try await task.value
        XCTAssertEqual(queries,2);XCTAssertEqual(result.references,["a","b","retained"])
    }
    func testTextWritableAndUnicodeByteDriftRefuse()async {
        for next in [SourceSnapshot(text:"changed",writable:true),SourceSnapshot(text:"é",writable:false),SourceSnapshot(text:"e\u{301}",writable:true)] {
            do{_ = try await acquireReferenceSources(["a":SourceSnapshot(text:"é",writable:true)],query:{["a"]},acquire:{_ in next},current:{true});XCTFail("drift accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))}
        }
        do{_ = try await acquireReferenceSources([:],query:{["a"]},acquire:{_ in SourceSnapshot(text:"source",writable:false)},current:{true});XCTFail("readonly accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("read-only"))}
    }
    func testStaleReadStopsSubsequentDispatchAndQuery()async {
        var current=true;var reads:[Read]=[];var queries=0
        let task=Task {try await acquireReferenceSources([:],query:{queries+=1;return ["a","b","c"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{current})}
        await settle{reads.count==1};current=false;reads[0].continuation.resume(returning:SourceSnapshot(text:"source",writable:true))
        do{_ = try await task.value;XCTFail("obsolete accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))};XCTAssertEqual(reads.count,1);XCTAssertEqual(queries,1)
    }
    func testExpandingAndOversizedDiscoveryRefuse()async {
        var round=0
        do{_ = try await acquireReferenceSources([:],query:{defer{round+=1};return ["new-\(round)"]},acquire:{_ in SourceSnapshot(text:"source",writable:true)},current:{true});XCTFail("expansion accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("kept expanding"))};XCTAssertEqual(round,4)
        var reads=0
        do{_ = try await acquireReferenceSources([:],query:{(0...10000).map(String.init)},acquire:{_ in reads+=1;return SourceSnapshot(text:"",writable:true)},current:{true});XCTFail("oversized accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("10,000"))};XCTAssertEqual(reads,0)
    }
    func testReadonlyContextRequiresExplicitPolicyAndStillRefusesDrift()async throws {
        let original=SourceSnapshot(text:"context",writable:false)
        let result=try await acquireSourceProposal(["context":original],query:{(paths:["context"],value:1)},acquire:{_ in original},current:{true},requireWritable:{_ in false});XCTAssertFalse(result.sources["context"]!.writable)
        do{_ = try await acquireSourceProposal(["context":original],query:{(paths:["context"],value:1)},acquire:{_ in SourceSnapshot(text:"context",writable:true)},current:{true},requireWritable:{_ in false});XCTFail("drift accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))}
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ReferenceSourcesTests.swift"><ui:paste select="#reference-source-acquisition-apple-witness"/></ui:file>

## reference-source-acquisition-android

The provider query and source acquisition remain replaceable callbacks. The optional `requireWritable` policy defaults to editing authority; callers may admit read-only context explicitly, while any change to an original writable state still refuses. Every discovered path must complete one acquisition before a subsequent query can return a proposal. Retained text is compared byte-for-byte and never replaced; acquiring is also the host’s provider synchronization boundary.

<ui:copy id="reference-source-acquisition-android">
package com.hickorydocs.simple
import kotlinx.coroutines.*
data class ReferenceSourceAcquisition(val sources:Map<String,SourceSnapshot>,val references:List<String>)
data class SourceProposal<Value>(val paths:List<String>,val value:Value)
data class SourceProposalAcquisition<Value>(val sources:Map<String,SourceSnapshot>,val paths:List<String>,val value:Value)
suspend fun <Value> acquireSourceProposal(initial:Map<String,SourceSnapshot>,query:suspend ()->SourceProposal<Value>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean,requireWritable:(String)->Boolean={true}):SourceProposalAcquisition<Value> {
 val sources=initial.toMutableMap();val synchronized=mutableSetOf<String>()
 fun checkCurrent(){check(current()){"Source, selection or provider changed. Prepare again."}}
 repeat(4){
  checkCurrent();val proposal=query();val references=proposal.paths.distinct().sorted();checkCurrent();check(references.size<=10000){"Reference discovery exceeds 10,000 source targets. Narrow the change."}
  val pending=references.filter{it !in synchronized};if(pending.isEmpty())return SourceProposalAcquisition(sources.toMap(),references,proposal.value)
  for(path in pending){
   checkCurrent();val value=acquire(path);checkCurrent();val original=sources[path]
   check(original==null||(original.text==value.text&&original.writable==value.writable)){"Source changed while synchronizing: $path. Prepare again."}
   check(!requireWritable(path)||value.writable){"Source is generated, read-only or saving: $path"}
   if(original==null)sources[path]=value;synchronized.add(path)
  }
  checkCurrent()
 }
 error("Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
suspend fun acquireReferenceSources(initial:Map<String,SourceSnapshot>,query:suspend ()->List<String>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean,requireWritable:(String)->Boolean={true}):ReferenceSourceAcquisition {
 val result=acquireSourceProposal(initial,{SourceProposal(query(),Unit)},acquire,current,requireWritable)
 return ReferenceSourceAcquisition(result.sources,result.paths)
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ReferenceSources.kt"><ui:paste select="#reference-source-acquisition-android"/></ui:file>

## reference-source-acquisition-android-witness

Hold source completions to prove sequential dispatch, duplicate removal and a fresh query after both new and retained sources synchronize. Original drafts survive unchanged; text or writable-state drift refuses. Obsolete intent stops subsequent reads and repeated expansion or excessive targets never return a proposal.

<ui:copy id="reference-source-acquisition-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class ReferenceSourcesTest {
 @Test fun retainedDraftSynchronizesOnceBeforeFreshProposal()=runBlocking {
  var round=0;val reads=mutableListOf<String>();val original=SourceSnapshot("original draft",true)
  val result=acquireSourceProposal(mapOf("retained" to original),{SourceProposal(listOf("retained","retained"),++round)},{path->reads.add(path);original.copy()},{true})
  assertEquals(2,result.value);assertEquals(listOf("retained"),reads);assertSame(original,result.sources["retained"])
 }
 data class Read(val path:String,val done:CompletableDeferred<SourceSnapshot>)
 private suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
 @Test fun sequentialAcquisitionDeduplicatesAndRequeries()=runBlocking {
  var queries=0;val reads=mutableListOf<Read>()
  val task=async{acquireReferenceSources(mapOf("retained" to SourceSnapshot("original draft",true)),{queries++;listOf("a","b","a","retained")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{true})}
  settle{reads.size==1};assertEquals(listOf("a"),reads.map{it.path});reads[0].done.complete(SourceSnapshot("one",true));settle{reads.size==2};assertEquals(listOf("a","b"),reads.map{it.path});assertEquals(1,queries)
  reads[1].done.complete(SourceSnapshot("two",true));settle{reads.size==3};assertEquals("retained",reads[2].path);reads[2].done.complete(SourceSnapshot("original draft",true));val result=task.await();assertEquals(2,queries);assertEquals(listOf("a","b","retained"),result.references)
 }
 @Test fun textAndWritableDriftAndReadonlySourcesRefuse()=runBlocking {
  for(next in listOf(SourceSnapshot("changed",true),SourceSnapshot("original",false))){try{acquireReferenceSources(mapOf("a" to SourceSnapshot("original",true)),{listOf("a")},{next},{true});fail("drift accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))}}
  try{acquireReferenceSources(emptyMap(),{listOf("a")},{SourceSnapshot("source",false)},{true});fail("readonly accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("read-only"))}
 }
 @Test fun staleReadStopsSubsequentDispatchAndQuery()=runBlocking {
  supervisorScope {
   var current=true;var queries=0;val reads=mutableListOf<Read>();val task=async{acquireReferenceSources(emptyMap(),{queries++;listOf("a","b","c")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{current})}
   settle{reads.size==1};current=false;reads[0].done.complete(SourceSnapshot("source",true));try{task.await();fail("obsolete accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))};assertEquals(1,reads.size);assertEquals(1,queries)
  }
 }
 @Test fun expandingAndOversizedDiscoveryRefuse()=runBlocking {
  var round=0;try{acquireReferenceSources(emptyMap(),{listOf("new-${round++}")},{SourceSnapshot("source",true)},{true});fail("expansion accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("kept expanding"))};assertEquals(4,round)
  var reads=0;try{acquireReferenceSources(emptyMap(),{(0..10000).map{it.toString()}},{reads++;SourceSnapshot("",true)},{true});fail("oversized accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("10,000"))};assertEquals(0,reads)
 }
 @Test fun readonlyContextRequiresExplicitPolicyAndStillRefusesDrift()=runBlocking {
  val original=SourceSnapshot("context",false);val result=acquireSourceProposal(mapOf("context" to original),{SourceProposal(listOf("context"),1)},{original.copy()},{true},{false});assertSame(original,result.sources["context"])
  try{acquireSourceProposal(mapOf("context" to original),{SourceProposal(listOf("context"),1)},{SourceSnapshot("context",true)},{true},{false});fail("drift accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ReferenceSourcesTest.kt"><ui:paste select="#reference-source-acquisition-android-witness"/></ui:file>

</ui:doc>
