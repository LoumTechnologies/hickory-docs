<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Acquire referenced source before preparing a change

For maintainers expanding reviewed changes beyond visible or retained editors.

An initial reference query discovers targets; it cannot authorize edits against text loaded afterward. Acquire unknown source snapshots and synchronize the provider, then query again. Keep the original retained snapshots, limit acquisition to two concurrent reads, deduplicate targets and drain started work after failure or obsolete intent. Four expanding query rounds or more than 10,000 targets refuse explicitly. This establishes the observed reference set, not exhaustive project analysis. Host callbacks validate workspace-local ownership and run on the owning UI context; reading source never navigates, saves or applies changes.

A proposal query carries both discovered paths and its original value. Only a query whose paths already have captured source can return that value. This same acquisition boundary serves references and provider edits; a discovering edit is discarded after source acquisition, because synchronizing new source may invalidate the provider artifact that produced it.

## reference-source-acquisition-browser

The provider query and source acquisition remain replaceable callbacks. A newly acquired snapshot becomes an input to a subsequent query; no proposal from the discovering query is returned for application.

<ui:copy id="reference-source-acquisition-browser">
import type {SourceSnapshot} from './ChangePlan';
/** Discovery never authorizes edits. Newly found targets synchronize before a fresh query. */
export async function acquireSourceProposal<T>(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<{paths:string[];value:T}>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean){
 const sources=new Map(initial);let references:string[]=[];
 const check=()=>{if(!current())throw new Error('Source, selection or provider changed. Prepare again.');};
 for(let round=0;round<4;round++){
  check();const proposal=await query();references=[...new Set(proposal.paths)];check();if(references.length>10000)throw new Error('Reference discovery exceeds 10,000 source targets. Narrow the change.');
  const missing=references.filter(path=>!sources.has(path));if(!missing.length)return {sources,paths:references,value:proposal.value};
  let cursor=0,failure:unknown=null;
  const worker=async()=>{while(failure===null&&cursor<missing.length){if(!current()){failure=new Error('Source, selection or provider changed. Prepare again.');break;}const path=missing[cursor++];
   try{const value=await acquire(path);check();if(!value.writable)throw new Error(`Source is generated, read-only or saving: ${path}`);sources.set(path,value);}catch(error){failure=error;}
  }};
  await Promise.all([worker(),worker()]);if(failure!==null)throw failure;check();
 }
 throw new Error('Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.');
}
export async function acquireReferenceSources(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<string[]>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean){
 const result=await acquireSourceProposal(initial,async()=>({paths:await query(),value:null}),acquire,current);
 return {sources:result.sources,references:result.paths};
}
</ui:copy>

<ui:file path="apps/web/src/simple/ReferenceSources.ts"><ui:paste select="#reference-source-acquisition-browser"/></ui:file>

## reference-source-acquisition-browser-witness

Hold source completions to prove two-worker bounds, duplicate removal and a fresh query after acquisition. Existing drafts must retain their original snapshots; obsolete intent stops remaining dispatches while started reads drain. Repeated expansion refuses without returning a change proposal.

<ui:copy id="reference-source-acquisition-browser-witness">
import {expect,it} from 'vitest';
import {acquireReferenceSources,acquireSourceProposal} from './ReferenceSources';
function held(){let resolve!:(value:{text:string;writable:boolean})=>void;const promise=new Promise<{text:string;writable:boolean}>(done=>resolve=done);return {promise,resolve};}
const flush=async()=>{for(let i=0;i<6;i++)await Promise.resolve();};
it('returns only the proposal queried after all source acquisition',async()=>{
 let round=0;const result=await acquireSourceProposal(new Map(),async()=>({paths:['other'],value:++round}),async()=>({text:'loaded',writable:true}),()=>true);
 expect(result.value).toBe(2);expect(result.sources.get('other')?.text).toBe('loaded');
});
it('loads at most two unknown targets and queries again after acquisition',async()=>{
 let queries=0;const reads:{path:string;done:ReturnType<typeof held>}[]=[];
 const result=acquireReferenceSources(new Map([['retained',{text:'original draft',writable:true}]]),async()=>{queries++;return ['a','b','a','c','retained'];},path=>{const done=held();reads.push({path,done});return done.promise;},()=>true);
 await flush();expect(reads.map(item=>item.path)).toEqual(['a','b']);expect(queries).toBe(1);reads[0].done.resolve({text:'one',writable:true});await flush();expect(reads.map(item=>item.path)).toEqual(['a','b','c']);expect(queries).toBe(1);
 reads[1].done.resolve({text:'two',writable:true});reads[2].done.resolve({text:'three',writable:true});const value=await result;expect(queries).toBe(2);expect(value.references).toEqual(['a','b','c','retained']);expect(value.sources.get('retained')?.text).toBe('original draft');expect(value.sources.size).toBe(4);
});
it('stale requests drain started reads and never dispatch remaining targets',async()=>{
 let current=true;const reads:ReturnType<typeof held>[]=[];const result=acquireReferenceSources(new Map(),async()=>['a','b','c'],()=>{const read=held();reads.push(read);return read.promise;},()=>current);const rejected=expect(result).rejects.toThrow('changed');await flush();current=false;reads.forEach(read=>read.resolve({text:'source',writable:true}));await rejected;expect(reads).toHaveLength(2);
 let round=0;await expect(acquireReferenceSources(new Map(),async()=>[`new-${round++}`],async()=>({text:'source',writable:true}),()=>true)).rejects.toThrow('kept expanding');expect(round).toBe(4);
});
</ui:copy>

<ui:file path="apps/web/src/simple/ReferenceSources.test.ts"><ui:paste select="#reference-source-acquisition-browser-witness"/></ui:file>

## reference-source-acquisition-apple

The provider query and source acquisition remain replaceable callbacks. A newly acquired snapshot becomes an input to a subsequent query; no proposal from the discovering query is returned for application.

<ui:copy id="reference-source-acquisition-apple">
import Foundation
struct ReferenceSourceAcquisition {var sources:[String:SourceSnapshot];var references:[String]}
struct SourceProposalAcquisition<Value> {var sources:[String:SourceSnapshot];var paths:[String];var value:Value}
@MainActor func acquireSourceProposal<Value>(_ initial:[String:SourceSnapshot],query:@escaping () async throws->(paths:[String],value:Value),acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool) async throws->SourceProposalAcquisition<Value> {
    var sources=initial
    func check()throws{guard current() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}}
    for _ in 0..<4 {
        try check();let proposal=try await query(),references=Array(Set(proposal.paths)).sorted();try check()
        guard references.count<=10000 else{throw EngineError(message:"Reference discovery exceeds 10,000 source targets. Narrow the change.")}
        let missing=references.filter {sources[$0]==nil};if missing.isEmpty{return SourceProposalAcquisition(sources:sources,paths:references,value:proposal.value)}
        var cursor=0;var failure:Error?
        @MainActor func worker()async {
            while failure==nil && cursor<missing.count {
                do{try check();let path=missing[cursor];cursor+=1;let value=try await acquire(path);try check()
                    guard value.writable else{throw EngineError(message:"Source is generated, read-only or saving: \(path)")};sources[path]=value
                }catch{failure=error}
            }
        }
        let first=Task {await worker()},second=Task {await worker()};_ = await(first.value,second.value)
        if let failure{throw failure};try check()
    }
    throw EngineError(message:"Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
@MainActor func acquireReferenceSources(_ initial:[String:SourceSnapshot],query:@escaping () async throws->[String],acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool) async throws->ReferenceSourceAcquisition {
    let result=try await acquireSourceProposal(initial,query:{(paths:try await query(),value:false)},acquire:acquire,current:current)
    return ReferenceSourceAcquisition(sources:result.sources,references:result.paths)
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ReferenceSources.swift"><ui:paste select="#reference-source-acquisition-apple"/></ui:file>

## reference-source-acquisition-apple-witness

Hold source completions to prove two-worker bounds, duplicate removal and a fresh query after acquisition. Existing drafts must retain their original snapshots; obsolete intent stops remaining dispatches while started reads drain. Repeated expansion refuses without returning a change proposal.

<ui:copy id="reference-source-acquisition-apple-witness">
import XCTest
@testable import HickorySimple
@MainActor final class ReferenceSourcesTests:XCTestCase {
    func testReturnsOnlyProposalQueriedAfterAcquisition()async throws {
        var round=0
        let result=try await acquireSourceProposal([:],query:{round+=1;return (paths:["other"],value:round)},acquire:{_ in SourceSnapshot(text:"loaded",writable:true)},current:{true})
        XCTAssertEqual(result.value,2);XCTAssertEqual(result.sources["other"]?.text,"loaded")
    }
    struct Read {var path:String;var continuation:CheckedContinuation<SourceSnapshot,Error>}
    func settle(_ condition:@escaping ()->Bool)async{let deadline=Date().addingTimeInterval(2);while !condition()&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
    func testBoundedAcquisitionRequeriesWithoutReplacingRetainedDrafts()async throws {
        var queries=0;var reads:[Read]=[]
        let task=Task {try await acquireReferenceSources(["retained":SourceSnapshot(text:"original draft",writable:true)],query:{queries+=1;return ["a","b","a","c","retained"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{true})}
        await settle{reads.count==2};XCTAssertEqual(reads.map(\.path),["a","b"]);XCTAssertEqual(queries,1)
        reads[0].continuation.resume(returning:SourceSnapshot(text:"one",writable:true));await settle{reads.count==3};XCTAssertEqual(reads[2].path,"c");XCTAssertEqual(queries,1)
        reads[1].continuation.resume(returning:SourceSnapshot(text:"two",writable:true));reads[2].continuation.resume(returning:SourceSnapshot(text:"three",writable:true));let value=try await task.value
        XCTAssertEqual(queries,2);XCTAssertEqual(value.references,["a","b","c","retained"]);XCTAssertEqual(value.sources["retained"]?.text,"original draft");XCTAssertEqual(value.sources.count,4)
    }
    func testStaleAcquisitionDrainsAndExpandingTargetsRefuse()async {
        var current=true;var reads:[Read]=[]
        let task=Task {try await acquireReferenceSources([:],query:{["a","b","c"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{current})}
        await settle{reads.count==2};current=false;reads.forEach{$0.continuation.resume(returning:SourceSnapshot(text:"source",writable:true))}
        do{_ = try await task.value;XCTFail("obsolete request accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))};XCTAssertEqual(reads.count,2)
        var round=0
        do{_ = try await acquireReferenceSources([:],query:{defer{round+=1};return ["new-\(round)"]},acquire:{_ in SourceSnapshot(text:"source",writable:true)},current:{true});XCTFail("expanding discovery accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("kept expanding"))};XCTAssertEqual(round,4)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ReferenceSourcesTests.swift"><ui:paste select="#reference-source-acquisition-apple-witness"/></ui:file>

## reference-source-acquisition-android

The provider query and source acquisition remain replaceable callbacks. A newly acquired snapshot becomes an input to a subsequent query; no proposal from the discovering query is returned for application.

<ui:copy id="reference-source-acquisition-android">
package com.hickorydocs.simple
import kotlinx.coroutines.*
data class ReferenceSourceAcquisition(val sources:Map<String,SourceSnapshot>,val references:List<String>)
data class SourceProposal<Value>(val paths:List<String>,val value:Value)
data class SourceProposalAcquisition<Value>(val sources:Map<String,SourceSnapshot>,val paths:List<String>,val value:Value)
suspend fun <Value> acquireSourceProposal(initial:Map<String,SourceSnapshot>,query:suspend ()->SourceProposal<Value>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean):SourceProposalAcquisition<Value> = coroutineScope {
 val sources=initial.toMutableMap()
 fun checkCurrent(){check(current()){"Source, selection or provider changed. Prepare again."}}
 repeat(4){
  checkCurrent();val proposal=query();val references=proposal.paths.distinct().sorted();checkCurrent();check(references.size<=10000){"Reference discovery exceeds 10,000 source targets. Narrow the change."}
  val missing=references.filter{it !in sources};if(missing.isEmpty())return@coroutineScope SourceProposalAcquisition(sources.toMap(),references,proposal.value)
  var cursor=0;var failure:Exception?=null
  suspend fun worker(){while(failure==null&&cursor<missing.size){try{checkCurrent();val path=missing[cursor++];val value=acquire(path);checkCurrent();check(value.writable){"Source is generated, read-only or saving: $path"};sources[path]=value}
   catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){failure=error}
  }}
  val first=async{worker()};val second=async{worker()};awaitAll(first,second);failure?.let{throw it};checkCurrent()
 }
 error("Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
suspend fun acquireReferenceSources(initial:Map<String,SourceSnapshot>,query:suspend ()->List<String>,acquire:suspend (String)->SourceSnapshot,current:()->Boolean):ReferenceSourceAcquisition {
 val result=acquireSourceProposal(initial,{SourceProposal(query(),Unit)},acquire,current)
 return ReferenceSourceAcquisition(result.sources,result.paths)
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ReferenceSources.kt"><ui:paste select="#reference-source-acquisition-android"/></ui:file>

## reference-source-acquisition-android-witness

Hold source completions to prove two-worker bounds, duplicate removal and a fresh query after acquisition. Existing drafts must retain their original snapshots; obsolete intent stops remaining dispatches while started reads drain. Repeated expansion refuses without returning a change proposal.

<ui:copy id="reference-source-acquisition-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class ReferenceSourcesTest {
 @Test fun returnsOnlyProposalQueriedAfterAcquisition()=runBlocking {
  var round=0;val result=acquireSourceProposal(emptyMap(),{SourceProposal(listOf("other"),++round)},{SourceSnapshot("loaded",true)},{true})
  assertEquals(2,result.value);assertEquals("loaded",result.sources["other"]?.text)
 }
 data class Read(val path:String,val done:CompletableDeferred<SourceSnapshot>)
 private suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
 @Test fun boundedAcquisitionRequeriesWithoutReplacingRetainedDrafts()=runBlocking {
  var queries=0;val reads=mutableListOf<Read>()
  val task=async{acquireReferenceSources(mapOf("retained" to SourceSnapshot("original draft",true)),{queries++;listOf("a","b","a","c","retained")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{true})}
  settle{reads.size==2};assertEquals(listOf("a","b"),reads.map{it.path});assertEquals(1,queries);reads[0].done.complete(SourceSnapshot("one",true));settle{reads.size==3};assertEquals("c",reads[2].path);assertEquals(1,queries)
  reads[1].done.complete(SourceSnapshot("two",true));reads[2].done.complete(SourceSnapshot("three",true));val value=task.await();assertEquals(2,queries);assertEquals(listOf("a","b","c","retained"),value.references);assertEquals("original draft",value.sources["retained"]?.text);assertEquals(4,value.sources.size)
 }
 @Test fun staleAcquisitionDrainsAndExpandingTargetsRefuse()=runBlocking {
  supervisorScope {
   var current=true;val reads=mutableListOf<Read>();val task=async{acquireReferenceSources(emptyMap(),{listOf("a","b","c")},{path->val done=CompletableDeferred<SourceSnapshot>();reads.add(Read(path,done));done.await()},{current})}
   settle{reads.size==2};current=false;reads.forEach{it.done.complete(SourceSnapshot("source",true))};try{task.await();fail("obsolete accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("changed"))};assertEquals(2,reads.size)
  }
  var round=0;try{acquireReferenceSources(emptyMap(),{listOf("new-${round++}")},{SourceSnapshot("source",true)},{true});fail("expansion accepted")}catch(error:IllegalStateException){assertTrue(error.message.orEmpty().contains("kept expanding"))};assertEquals(4,round)
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ReferenceSourcesTest.kt"><ui:paste select="#reference-source-acquisition-android-witness"/></ui:file>

</ui:doc>
