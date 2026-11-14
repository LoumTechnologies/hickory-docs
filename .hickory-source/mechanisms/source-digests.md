<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Bind optional facts to background source digests

For maintainers attaching source-bound capabilities without blocking readable text.

A workspace digest service serializes hashing, retains only the latest pending source per path and rejects replies after source changes or shutdown. Native hashing runs off the UI thread. A missing or failed digest grants no positional authority, while unrelated pending sources can continue. Cache lookup compares exact source bytes, including canonically equivalent Unicode. This does not authorize a save or freeze files on disk.

## source-digests-browser

Observe an available source immediately. The digest queue owns only background work and a matching-source cache; presentation and capability-specific projection remain consumers.

<ui:copy id="source-digests-browser">
/** Source hashing is optional background work: one worker, one latest job per path. */
export class SourceDigests {
  private records=new Map<string,{source:string;hash:string}>();private pending=new Map<string,{source:string;current:()=>string|null}>();private active=false;private closed=false;
  private revision=0;private listeners=new Set<()=>void>();
  constructor(private hash:(source:string)=>Promise<string>=async source=>{const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source));return [...new Uint8Array(digest)].map(byte=>byte.toString(16).padStart(2,'0')).join('');}){}
  subscribe=(listener:()=>void)=>{this.listeners.add(listener);return()=>{this.listeners.delete(listener);};};snapshot=()=>this.revision;
  current(path:string,source:string){const record=this.records.get(path);return record?.source===source?record.hash:null;}
  enqueue(path:string,source:string,current:()=>string|null){if(this.closed||this.current(path,source)!==null)return;this.pending.set(path,{source,current});void this.pump();}
  stop(){this.closed=true;this.pending.clear();}
  private async pump(){if(this.closed||this.active)return;const next=this.pending.entries().next().value;if(!next)return;const [path,job]=next;this.pending.delete(path);this.active=true;
    try {const hash=await this.hash(job.source);if(!this.closed&&job.current()===job.source&&!this.pending.has(path)){this.records.set(path,{source:job.source,hash});this.revision++;this.listeners.forEach(listener=>listener());}}
    catch{/* A failed optional digest grants no positional authority. */}
    finally{this.active=false;void this.pump();}
  }
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceDigests.ts"><ui:paste select="#source-digests-browser"/></ui:file>

## source-digests-browser-witness

Hold actual asynchronous completions to prove one-worker bounds, latest-source coalescing, exact Unicode retirement, closed-reply rejection and independent failure recovery.

<ui:copy id="source-digests-browser-witness">
import {expect,it} from 'vitest';
import {SourceDigests} from './SourceDigests';
function deferred(){let resolve!:(value:string)=>void;const promise=new Promise<string>(done=>resolve=done);return {promise,resolve};}
const flush=async()=>{await Promise.resolve();await Promise.resolve();};
it('bounds hashing, coalesces queued drafts and rejects obsolete bytes',async()=>{
 const jobs:{source:string;done:ReturnType<typeof deferred>}[]=[];const index=new SourceDigests(source=>{const done=deferred();jobs.push({source,done});return done.promise;});
 let source='é';index.enqueue('a',source,()=>source);source='e\u0301';index.enqueue('a','intermediate',()=>source);index.enqueue('a',source,()=>source);index.enqueue('b','other',()=> 'other');expect(jobs).toHaveLength(1);
 jobs[0].done.resolve('old');await flush();expect(index.current('a','é')).toBeNull();expect(jobs[1].source).toBe(source);jobs[1].done.resolve('new');await flush();expect(index.current('a',source)).toBe('new');expect(index.current('a','é')).toBeNull();expect(jobs[2].source).toBe('other');
 index.stop();jobs[2].done.resolve('late');await flush();expect(index.current('b','other')).toBeNull();
});
it('an optional digest failure does not prevent another source from completing',async()=>{
 const index=new SourceDigests(async source=>{if(source==='bad')throw new Error('unavailable');return 'good';});index.enqueue('a','bad',()=> 'bad');index.enqueue('b','ok',()=> 'ok');await flush();await flush();expect(index.current('a','bad')).toBeNull();expect(index.current('b','ok')).toBe('good');index.stop();
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceDigests.test.ts"><ui:paste select="#source-digests-browser-witness"/></ui:file>

## source-digests-apple

Observe an available source immediately. The digest queue owns only background work and a matching-source cache; presentation and capability-specific projection remain consumers.

<ui:copy id="source-digests-apple">
import Foundation
import Observation
@MainActor @Observable final class SourceDigests {
    private struct Record {var source:ExactSourceText;var hash:String}
    private struct Job {var source:String;var current:()->String?}
    private var records:[String:Record]=[:]
    @ObservationIgnored private var pending:[String:Job]=[:]
    @ObservationIgnored private var order:[String]=[]
    @ObservationIgnored private var active=false
    @ObservationIgnored private var closed=false
    private let hash:(String) async throws->String
    init(hash:@escaping (String) async throws->String={source in await Task.detached {nativeSourceHash(source)}.value}){self.hash=hash}
    func current(_ path:String,source:String)->String? {guard let record=records[path],exactUTF8(record.source.text,source) else{return nil};return record.hash}
    func enqueue(_ path:String,source:String,current:@escaping ()->String?){guard !closed,self.current(path,source:source)==nil else{return};if pending[path]==nil{order.append(path)};pending[path]=Job(source:source,current:current);pump()}
    func stop(){closed=true;pending=[:];order=[]}
    private func pump(){guard !closed,!active,!order.isEmpty else{return};let path=order.removeFirst();guard let job=pending.removeValue(forKey:path) else{return};active=true
        Task {do{let value=try await hash(job.source)
            if !closed,exactUTF8(job.current(),job.source),pending[path]==nil{records[path]=Record(source:ExactSourceText(job.source),hash:value)}
        }catch{};active=false;pump()}
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceDigests.swift"><ui:paste select="#source-digests-apple"/></ui:file>

## source-digests-apple-witness

Hold actual asynchronous completions to prove one-worker bounds, latest-source coalescing, exact Unicode retirement, closed-reply rejection and independent failure recovery.

<ui:copy id="source-digests-apple-witness">
import XCTest
@testable import HickorySimple
@MainActor final class SourceDigestsTests:XCTestCase {
    func testCoalescedHashingRejectsObsoleteUnicodeBytesAndClosedReplies() async {
        var jobs:[(String,CheckedContinuation<String,Error>)]=[]
        let index=SourceDigests {source in try await withCheckedThrowingContinuation {jobs.append((source,$0))}}
        var source="é";index.enqueue("a",source:source,current:{source});try? await Task.sleep(for:.milliseconds(1))
        source="e\u{301}";index.enqueue("a",source:"intermediate",current:{source});index.enqueue("a",source:source,current:{source});index.enqueue("b",source:"other",current:{"other"})
        XCTAssertEqual(jobs.count,1);jobs[0].1.resume(returning:"old")
        for _ in 0..<10 {try? await Task.sleep(for:.milliseconds(1))}
        XCTAssertNil(index.current("a",source:"é"));XCTAssertEqual(jobs.count,2);XCTAssertTrue(exactUTF8(jobs[1].0,source))
        jobs[1].1.resume(returning:"new");for _ in 0..<10 {try? await Task.sleep(for:.milliseconds(1))}
        XCTAssertEqual(index.current("a",source:source),"new");XCTAssertNil(index.current("a",source:"é"));XCTAssertEqual(jobs.count,3)
        index.stop();jobs[2].1.resume(returning:"late");for _ in 0..<10 {try? await Task.sleep(for:.milliseconds(1))};XCTAssertNil(index.current("b",source:"other"))
    }
    func testFailureDoesNotBlockAnotherSource() async {
        let index=SourceDigests {source in if source=="bad"{throw EngineError(message:"unavailable")};return "good"}
        index.enqueue("a",source:"bad",current:{"bad"});index.enqueue("b",source:"ok",current:{"ok"});for _ in 0..<20 {try? await Task.sleep(for:.milliseconds(1))}
        XCTAssertNil(index.current("a",source:"bad"));XCTAssertEqual(index.current("b",source:"ok"),"good");index.stop()
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceDigestsTests.swift"><ui:paste select="#source-digests-apple-witness"/></ui:file>

## source-digests-android

Observe an available source immediately. The digest queue owns only background work and a matching-source cache; presentation and capability-specific projection remain consumers.

<ui:copy id="source-digests-android">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import kotlinx.coroutines.*
class SourceDigests(private val scope:CoroutineScope,private val hash:suspend (String)->String={source->withContext(Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}}) {
    private data class Record(val source:String,val hash:String)
    private data class Request(val source:String,val current:()->String?)
    private var records by mutableStateOf<Map<String,Record>>(emptyMap());private val pending=linkedMapOf<String,Request>();private var active=false;private var closed=false
    fun current(path:String,source:String)=records[path]?.takeIf{it.source==source}?.hash
    fun enqueue(path:String,source:String,current:()->String?){if(closed||this.current(path,source)!=null)return;pending[path]=Request(source,current);pump()}
    fun stop(){closed=true;pending.clear()}
    private fun pump(){if(closed||active)return;val next=pending.entries.firstOrNull() ?: return;val path=next.key;val request=next.value;pending.remove(path);active=true
        scope.launch{try{val value=hash(request.source);if(!closed&&request.current()==request.source&&path !in pending)records=records+(path to Record(request.source,value))}
        catch(cancelled:CancellationException){throw cancelled}catch(_:Exception){}finally{active=false;pump()}}
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SourceDigests.kt"><ui:paste select="#source-digests-android"/></ui:file>

## source-digests-android-witness

Hold actual asynchronous completions to prove one-worker bounds, latest-source coalescing, exact Unicode retirement, closed-reply rejection and independent failure recovery.

<ui:copy id="source-digests-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class SourceDigestsTest {
 @Test fun coalescingRejectsObsoleteBytesAndClosedReplies()=runBlocking {
  val jobs=mutableListOf<Pair<String,CompletableDeferred<String>>>()
  val index=SourceDigests(this){source->val done=CompletableDeferred<String>();jobs.add(source to done);done.await()}
  var source="é";index.enqueue("a",source){source};yield();source="e\u0301";index.enqueue("a","intermediate"){source};index.enqueue("a",source){source};index.enqueue("b","other"){"other"};assertEquals(1,jobs.size)
  jobs[0].second.complete("old");repeat(10){yield()};assertNull(index.current("a","é"));assertEquals(source,jobs[1].first)
  jobs[1].second.complete("new");repeat(10){yield()};assertEquals("new",index.current("a",source));assertNull(index.current("a","é"));assertEquals("other",jobs[2].first)
  index.stop();jobs[2].second.complete("late");repeat(10){yield()};assertNull(index.current("b","other"))
 }
 @Test fun failureDoesNotBlockAnotherSource()=runBlocking {
  val index=SourceDigests(this){source->if(source=="bad")error("unavailable");"good"};index.enqueue("a","bad"){"bad"};index.enqueue("b","ok"){"ok"};repeat(10){yield()};assertNull(index.current("a","bad"));assertEquals("good",index.current("b","ok"));index.stop()
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourceDigestsTest.kt"><ui:paste select="#source-digests-android-witness"/></ui:file>

</ui:doc>
