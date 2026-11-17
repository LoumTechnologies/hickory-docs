<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain breakpoint wishes without granting stale coordinates

For maintainers changing the boundary between local breakpoint intentions, complete adapter receipts and stopped stack observations.

An edit does not authorize the adapter to reuse a line number. Retain its old source wishes as a readonly reading, hide obsolete positional marks, and admit a new binding only from a complete receipt for the exact source and latest intention. Explicit placement on current text archives the older wishes; it cannot change the running snapshot. The existing floating controls page retained wishes without reserving editor space. History is bounded and in memory, not crash-persistent breakpoint evidence.

## Refuse incomplete binding evidence

These tests exercise the actual native setters and browser source model. Missing, duplicated, foreign and malformed rows must not promote a partial setter response into source binding. A later complete response can still confirm the same request; earlier wishes and current drafts survive refusal.

<ui:copy id="debugger-retention-apple-tests">
import XCTest
import SwiftUI
@testable import HickorySimple
#if os(macOS)
import AppKit
#endif
@MainActor final class DebuggerRetentionTests:XCTestCase {
 func testCompleteBreakpointReceiptCoverageAndStates()throws {
  let debug=NativeDebugger(engine:try Engine(address:"http://127.0.0.1:1")),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"first = 1\nsecond = 2\n",hash:"saved"));defer{debug.close()}
  debug.owner=file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};let launch=debug.beginLaunchRequest();debug.receive(["event":"started","source_hash":nativeSourceHash(buffer.text),"request_id":launch,"doc":languageURI(file.path),"session":"owned","breakpoints":[]])
  debug.toggle(buffer,line:0,condition:"first == 1");debug.toggle(buffer,line:1,condition:"second == 2");let request=try XCTUnwrap(requests.last)
  let good:[[String:Any]]=[["line":0,"state":"bound"],["line":1,"state":"pending"]]
  let bad:[[[String:Any]]]=[[],[good[0]],[good[0],good[0]],[good[0],["line":2,"state":"bound"]],[good[0],["line":1,"state":"invented"]],[good[0],["line":1,"state":"bound","moved_to":999]],[good[0],["line":1,"state":"bound","message":["foreign":true]]],[good[0],["line":true,"state":"bound"]]]
  func receipt(_ rows:[[String:Any]])->[String:Any]{["event":"breakpoints","session":"owned","request_id":request["request_id"]!,"source_hash":request["source_hash"]!,"breakpoints":rows]}
  for rows in bad {debug.receive(receipt(rows));XCTAssertFalse(debug.breakpointMarkers(buffer).contains{$0["state"] as? String=="bound"});XCTAssertEqual(debug.breakpoints[file.path]?.count,2);XCTAssertEqual(buffer.text,"first = 1\nsecond = 2\n")}
  debug.receive(receipt(good));XCTAssertEqual(debug.breakpointMarkers(buffer).filter{$0["state"] as? String=="bound"}.count,1);XCTAssertEqual(debug.breakpoints[file.path]?.first?["condition"] as? String,"first == 1")
 }
 func testEditedSourceRetainsEarlierAndCurrentWishesWithoutRuntimeSetters()throws {
  let debug=NativeDebugger(engine:try Engine(address:"http://127.0.0.1:1")),buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value = 1\n",hash:"saved"));defer{debug.close()};debug.owner=buffer.file.path;debug.launchSource=buffer.text;debug.sourceBuffers={ [buffer] };var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};let launch=debug.beginLaunchRequest();debug.receive(["event":"started","source_hash":nativeSourceHash(buffer.text),"request_id":launch,"doc":languageURI(buffer.file.path),"session":"owned","breakpoints":[]]);debug.toggle(buffer,line:0,condition:"value == 1");let before=requests.count
  buffer.text="# changed\nvalue = 1\n";debug.sourceChanged(buffer);XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty);debug.toggle(buffer,line:1,condition:"value >= 1");XCTAssertEqual(requests.count,before);let reading=debug.retainedBreakpointReading(buffer.file.path);XCTAssertTrue(reading.contains{$0.contains("Earlier")&&$0.contains("value == 1")});XCTAssertTrue(reading.contains{$0.contains("Retained")&&$0.contains("value >= 1")});XCTAssertTrue(debug.breakpointMarkers(buffer).isEmpty);XCTAssertEqual(buffer.text,"# changed\nvalue = 1\n")
 }
 #if os(macOS)
 func testLiveOwnedDeepStackAndEditedBreakpointRetention()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_DEBUG_RETENTION_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_DEBUG_RETENTION_WORKSPACE"] else{throw XCTSkip("Use the maintained owned debugger-retention fixture.")}
  let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-debug-retention-fixture"),encoding:.utf8)=="owned-debug-retention-v1\n" else{throw EngineError(message:"Owned fixture required.")}
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:origin),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  let file=FileNode(name:"deep.py",path:"deep.py",dir:false),b=try await w.loadBuffer(file),source=b.text,debug=NativeDebugger(engine:e);w.debugger=debug;w.layout.activateSource(file.path,in:"document-group");debug.sourceBuffers={Array(w.buffers.values)}
  let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
  defer{debug.close();w.sourceTestIndex?.stop();for value in w.buffers.values{value.language?.close()};window.orderOut(nil);window.contentView=nil}
  func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
  func wait(_ reason:String,_ predicate:()->Bool)async throws{for _ in 0..<1200{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:reason)}
  try await wait("Native editor did not mount"){native(host) != nil};let text=try XCTUnwrap(native(host)),scroll=try XCTUnwrap(text.enclosingScrollView),editor=try XCTUnwrap(b.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host)
  let line=try XCTUnwrap(source.components(separatedBy:"\n").firstIndex{$0.contains("# deep-stop")});debug.toggle(b,line:line,condition:"depth == 0");await debug.prepare(b);try await wait("Deep program did not pause: \(debug.message)"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertGreaterThanOrEqual(debug.frames.count,131)
  let frame=try XCTUnwrap(debug.frames.dropFirst(100).first);debug.select(frame,snapshot:debug.frameSnapshot);debug.readFrameLocals();try await wait("Deep caller locals unavailable"){!debug.frameBusy};debug.evaluate("depth");try await wait("Deep caller evaluation unavailable: \(debug.evaluationNotice)"){debug.values["depth"] != nil};XCTAssertEqual(debug.values["depth"],"100");XCTAssertEqual(b.sourceEditorState?()?.identity,editor);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
  var requests:[[String:Any]]=[];debug.observeRequestForTest={requests.append($0)};text.setSelectedRange(NSRange(location:text.string.utf16.count,length:0));text.insertText("# retained unsaved breakpoint edit\n",replacementRange:text.selectedRange());let edited=b.text,selection=text.selectedRange();debug.sourceChanged(b);debug.toggle(b,line:line,condition:"depth <= 0");XCTAssertFalse(requests.contains{["breakpoints","source_breakpoints"].contains($0["op"] as? String ?? "")});XCTAssertTrue(debug.breakpointMarkers(b).isEmpty);XCTAssertTrue(debug.retainedBreakpointReading(file.path).contains{$0.contains("depth == 0")});XCTAssertTrue(debug.retainedBreakpointReading(file.path).contains{$0.contains("depth <= 0")});XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),source);XCTAssertEqual(text.selectedRange(),selection)
  debug.stop();try await wait("Deep session did not stop"){!debug.active};await b.save(using:e);XCTAssertFalse(b.dirty,b.error);XCTAssertEqual(b.text,edited);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),edited)
  await debug.prepare(b);try await wait("Edited deep program did not pause: \(debug.message)"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertGreaterThanOrEqual(debug.frames.count,131);XCTAssertTrue(debug.breakpoints[file.path]?.contains{$0["condition"] as? String=="depth <= 0"}==true);XCTAssertTrue(debug.breakpointMarkers(b).contains{$0["state"] as? String=="bound"});debug.stop();try await wait("Edited deep session did not stop"){!debug.active}
  XCTAssertEqual(b.sourceEditorState?()?.identity,editor);XCTAssertTrue(native(host) === text);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);try XCTUnwrap(text.undoManager).undo();XCTAssertEqual(b.text,source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(file.path),encoding:.utf8),edited)
 }
 #endif
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/DebuggerRetentionTests.swift"><ui:paste select="#debugger-retention-apple-tests"/></ui:file>

<ui:copy id="debugger-retention-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class DebuggerRetentionTest {
 @Test fun completeBreakpointReceiptsAndArchiveReadings(){
  val model=NativeBreakpointModel();val source=NativeSourceSnapshot("owned","main.py","first = 1\nsecond = 2\n",true,true);val hash="a".repeat(64);model.toggle(source,0,"first == 1");val intent=model.toggle(source,1,"second == 2");model.stamp(source.identity,intent.revision,hash);model.request(source,"session","current")
  fun row(line:Any,state:Any="bound")=JSONObject().put("line",line).put("state",state)
  val good=listOf(row(0),row(1,"pending"));val bad=listOf(emptyList(),listOf(good[0]),listOf(good[0],good[0]),listOf(good[0],row(2)),listOf(good[0],row(1,"invented")),listOf(good[0],row(1).put("moved_to",999)),listOf(good[0],row(1).put("message",JSONObject().put("foreign",true))),listOf(good[0],row(true)))
  for(rows in bad){assertFalse(model.admit(source,JSONObject().put("breakpoints",JSONArray(rows)),"session","current",hash));assertTrue(model.receipts.isEmpty());assertEquals(2,model.intents.getValue(source.identity).marks.size)}
  assertTrue(model.admit(source,JSONObject().put("breakpoints",JSONArray(good)),"session","current",hash));assertEquals("first == 1",model.rows(source,"session",hash).first().getString("condition"))
  val changed=source.copy(text="# changed\n"+source.text,saved=false);assertTrue(model.rows(changed,"session",hash).isEmpty());model.toggle(changed,1,"first >= 1");val reading=model.readingPage(changed,0);assertTrue(reading.rows.any{it.historical&&it.condition=="first == 1"});assertTrue(reading.rows.any{!it.historical&&it.condition=="first >= 1"});assertTrue(model.rows(changed,"session",hash).isEmpty());model.discard(changed.identity);assertTrue(model.readingPage(changed,0).rows.any{it.condition=="first >= 1"})
 }
 @Test fun retainedIntentionPagesDoNotExposeMutableModelRows(){val model=NativeBreakpointModel();val source=NativeSourceSnapshot("owned","main.py",(0..200).joinToString("\n"){"line $it"},true,true);for(line in 0..129)model.toggle(source,line,"test $line");val first=model.readingPage(source,0);val second=model.readingPage(source,1);val third=model.readingPage(source,999);assertEquals(130,first.total);assertEquals(64,first.rows.size);assertEquals(64,second.rows.size);assertEquals(2,third.page);assertEquals(listOf(128,129),third.rows.map{it.line});assertEquals(0,model.readingPage(source,-1).page)}
 @Test fun liveOwnedDeepStackAndEditedBreakpointRetention()=runBlocking {
  val origin=System.getenv("HICKORY_DEBUG_RETENTION_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_DEBUG_RETENTION_WORKSPACE") ?: error("Owned workspace required"));check(java.io.File(root,".hickory-debug-retention-fixture").readText()=="owned-debug-retention-v1\n");val ownedScope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val engine=Engine(origin!!);val w=Workspace(ownedScope);w.engine=engine
  try{val (_,tree)=engine.files();w.rootFiles=tree;val file=tree.first{it.path=="deep.py"};val b=w.loadBuffer(file);w.show(file.path);val source=b.text;val state=b.state;val scroll=b.sourceScroll;val debug=NativeDebugger(engine,ownedScope);w.debugger=debug
   try{
    suspend fun wait(reason:String,predicate:()->Boolean){withTimeout(30000){while(!predicate())delay(20)}}
    val line=source.lines().indexOfFirst{it.contains("# deep-stop")};check(line>=0);debug.toggle(b,line,"depth == 0");debug.prepare(b);wait("Deep program pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.size>=131)
    val frame=debug.frames[100];debug.select(frame);debug.readFrameVariables();wait("Deep caller locals"){debug.variables.any{it.optString("name")=="depth"&&it.optString("value")=="100"}};debug.evaluate("depth");wait("Deep caller evaluation"){debug.values["depth"]!=null};assertEquals("100",debug.values["depth"]);assertSame(state,b.state);assertSame(scroll,b.sourceScroll)
    val requests=java.util.concurrent.CopyOnWriteArrayList<JSONObject>();debug.observeRequestForTest={requests.add(it)};assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# retained unsaved breakpoint edit\n")),b.text));val edited=b.text;val selection=b.state.selection;debug.toggle(b,line,"depth <= 0");delay(100);assertFalse(requests.any{it.optString("op") in listOf("breakpoints","source_breakpoints")});assertTrue(debug.breakpointRows(b).isEmpty());val reading=debug.breakpointReadingPage(b,0);assertTrue(reading.rows.any{it.condition=="depth == 0"});assertTrue(reading.rows.any{it.condition=="depth <= 0"});assertEquals(selection,b.state.selection);assertEquals(source,java.io.File(root,file.path).readText())
    debug.stop();wait("Stopped deep session"){!debug.active};b.save(engine);assertFalse(b.error,b.dirty);assertEquals(edited,java.io.File(root,file.path).readText());debug.prepare(b);wait("Edited deep program pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertTrue(debug.frames.size>=131);assertTrue(debug.breakpointRows(b).any{it.optString("state")=="bound"&&it.optString("condition")=="depth <= 0"});debug.stop();wait("Stopped edited session"){!debug.active};assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertTrue(b.snippets.undoSource());assertEquals(source,b.text);assertEquals(edited,java.io.File(root,file.path).readText())
   }finally{debug.close()}
  }finally{w.buffers.values.forEach{it.language?.close()};ownedScope.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/DebuggerRetentionTest.kt"><ui:paste select="#debugger-retention-android-tests"/></ui:file>

<ui:copy id="debugger-retention-browser-tests">
import {act,cleanup,fireEvent,render,renderHook,screen,waitFor} from '@testing-library/react';
import {afterEach,beforeAll,expect,it} from 'vitest';
import {DebugStrip,breakpointReadingPage,type DebugStripProps} from './DebugStrip';
import {DebugBreakpointModel,type DebugSourceReading} from './SourceGuards';
import {useDebuggerOver} from './useDebugger';
afterEach(cleanup);beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const base=(extra:Partial<DebugStripProps>={}):DebugStripProps=>({status:'paused',program:'main.py',message:null,capabilities:null,exceptionFilters:[],onToggleExceptionFilter:()=>{},frames:[],selectedFrame:null,watches:[],onSelectFrame:()=>{},onStep:()=>{},onJumpHere:()=>{},onStart:()=>{},onStop:()=>{},onAddWatch:()=>{},onRemoveWatch:()=>{},...extra});
it('keeps old conditions readable after explicit placement on current edited source',()=>{const model=new DebugBreakpointModel(),identity={},old:DebugSourceReading={uri:'hick:///main.py',workspace:'owned',namespace:'owned',identity,text:'old\n',saved:true,current:()=>true};model.change(old.uri,old,()=>[{line:0,condition:'old condition'}]);model.change(old.uri,{...old,text:'# changed\nold\n',saved:false},()=>[{line:1,condition:'new condition'}]);expect(model.archived[0].marks[0].condition).toBe('old condition');expect(model.desired(old.uri)[0].condition).toBe('new condition');expect(model.rows(old.uri,{...old,text:'# changed\nold\n',saved:false},'session','a'.repeat(64))).toEqual([]);const groups=[...model.archived.map(row=>({...row,historical:true})),{source:old.uri,namespace:old.namespace,marks:model.desired(old.uri)}];render(<DebugStrip {...base({breakpointIntentions:groups})}/>);fireEvent.click(screen.getByLabelText('Debug options'));fireEvent.click(screen.getByText('Retained breakpoint intentions',{exact:true}));expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('old condition');expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('new condition');});
it('pages every retained wish without silently dropping later rows',()=>{const groups=[{source:'hick:///main.py',namespace:'owned',historical:true,marks:Array.from({length:130},(_,line)=>({line,condition:'condition '+line}))}];expect(breakpointReadingPage(groups,0).rows).toHaveLength(64);expect(breakpointReadingPage(groups,1).rows).toHaveLength(64);expect(breakpointReadingPage(groups,999).page).toBe(2);expect(breakpointReadingPage(groups,999).rows.join(' ')).toContain('condition 129');render(<DebugStrip {...base({breakpointIntentions:groups})}/>);fireEvent.click(screen.getByLabelText('Debug options'));fireEvent.click(screen.getByText('Retained breakpoint intentions',{exact:true}));fireEvent.click(screen.getByRole('button',{name:'Next breakpoint intentions'}));expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('condition 64');fireEvent.click(screen.getByRole('button',{name:'Next breakpoint intentions'}));expect(screen.getByLabelText('Retained breakpoint reading').textContent).toContain('condition 129');expect((screen.getByRole('button',{name:'Next breakpoint intentions'}) as HTMLButtonElement).disabled).toBe(true);});
it('refreshes a saved current reading after source capability changes without rebinding runtime coordinates',async()=>{let version=0,notify=()=>{};const identity={},capability={read:(uri:string)=>{const captured=version;return {uri,workspace:'owned',namespace:'owned',identity,text:'one\ntwo\n',saved:true,current:()=>version===captured};},subscribe:(listener:()=>void)=>{notify=listener;return ()=>{};}};const hook=renderHook(()=>useDebuggerOver(null,'main.py',[],'new',undefined,undefined,capability));act(()=>hook.result.current.toggleBreakpoint(0));expect(hook.result.current.breakpoints).toHaveLength(1);act(()=>{version++;notify();});await waitFor(()=>expect(hook.result.current.breakpoints).toHaveLength(1));expect(hook.result.current.breakpoints[0].state).toBe('pending');hook.unmount();});
it('rejects an obsolete asynchronous capability reply instead of replacing a newer reading',async()=>{let version=0,notify=()=>{},defer=false;const identity={},pending:{reading:DebugSourceReading;resolve:(value:DebugSourceReading)=>void}[]=[];const capability={read:(uri:string):DebugSourceReading|Promise<DebugSourceReading>=>{const captured=version,reading={uri,workspace:'owned',namespace:'owned',identity,text:'one\ntwo\n',saved:true,current:()=>version===captured};if(!defer)return reading;return new Promise(resolve=>pending.push({reading,resolve}));},subscribe:(listener:()=>void)=>{notify=listener;return ()=>{};}};const hook=renderHook(()=>useDebuggerOver(null,'main.py',[],'new',undefined,undefined,capability));act(()=>hook.result.current.toggleBreakpoint(0));defer=true;act(()=>{version++;notify();});act(()=>{version++;notify();});expect(pending).toHaveLength(2);await act(async()=>pending[1].resolve(pending[1].reading));await waitFor(()=>expect(hook.result.current.breakpoints).toHaveLength(1));await act(async()=>pending[0].resolve(pending[0].reading));expect(hook.result.current.breakpoints).toHaveLength(1);expect(hook.result.current.breakpoints[0].state).toBe('pending');hook.unmount();});
</ui:copy>
<ui:file path="apps/web/src/debug/BreakpointRetention.test.tsx"><ui:paste select="#debugger-retention-browser-tests"/></ui:file>

## Follow a real stack beyond the first two pages

The owned debugpy fixture recurses 130 times. Each host must select frame 100 and read its own `depth`, then edit the same retained source editor. New local breakpoint wishes preserve earlier conditions without emitting a setter into the obsolete runtime source. Explicit Save and a fresh launch bind only the current wishes. Source Undo after stopping changes the draft, never the saved file. The browser drives existing gutter, condition, stack selector, Save and debug controls; Apple mounts the actual text editor, and Android retains its real TextFieldState and scroll objects. Native pointer, device IME and assistive-technology acceptance remain unverified.

Focused execution on 2026-11-17 passed the browser’s 39 debugger/receipt/reading cases, three Apple and three Android retention/preparation cases, and one exact owned real deep-stack/retention flow per host with no skips. Browser acceptance drove the rendered gutter condition editor and floating stack/reading controls. Apple mounted its actual NSTextView; Android retained its actual TextFieldState and scroll objects while invoking the native controller. These executions establish their stated usable paths and failure guards, while native physical pointer, device input and assistive-technology interaction remain unexecuted. The iOS shared-source build is recorded separately by the integration owner.

<ui:copy id="debugger-retention-owned-runner">
#!/usr/bin/env python3
"""Owned deep-stack and source-edit breakpoint retention acceptance for simple hosts."""
import argparse,importlib.util,json,os,re,signal,socket,subprocess,tempfile,time,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE="def descend(depth):\n    token = depth\n    if depth == 0:\n        marker = 41  # deep-stop\n        return marker\n    return descend(depth - 1)\n\ndescend(130)\n"
EDIT="# retained unsaved breakpoint edit\n"
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['browser','apple','android']);parser.add_argument('--skip-build',action='store_true');parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');parser.add_argument('--venv',type=Path,default=ROOT/'target/thread-debug-env');args=parser.parse_args()
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 python=args.venv.resolve()/'bin/python';assert python.exists(),'An owned prepared debugpy environment is required; use the native-debug fixture setup.'
 def wait(read,reason):
  deadline=time.monotonic()+40
  while time.monotonic()<deadline:
   if read():return
   time.sleep(.03)
  raise RuntimeError(reason)
 with tempfile.TemporaryDirectory(prefix='hickory-debug-retention-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.venv').symlink_to(args.venv.resolve(),target_is_directory=True);(root/'.hickory-debug-retention-fixture').write_text('owned-debug-retention-v1\n');(root/'deep.py').write_text(SOURCE);(root/'project.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>');(root/'.hick-launch.json').write_text(json.dumps({'version':1,'profiles':[{'id':'owned-deep','name':'Owned deep stack','program':'deep.py','default':True}]}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   engine=fixture.start([str(args.engine.resolve()),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);origin=fixture.origin(engine)
   env.update(HICKORY_DEBUG_RETENTION_ORIGIN=origin,HICKORY_DEBUG_RETENTION_WORKSPACE=str(root))
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='deep.py');driver.act('remember-editor',path='deep.py');geometry=driver.act('editor-size',path='deep.py')
    line=next(index+1 for index,text in enumerate(SOURCE.splitlines()) if '# deep-stop' in text);driver.act('debug-breakpoint',line=line);driver.act('debug-breakpoint-condition',path='deep.py',line=line,condition='depth == 0');driver.act('debug-start',path='deep.py');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'Deep program did not pause')
    selected=driver.act('debug-frame-index',index=100);assert selected['count']>=131,selected;assert driver.act('debug-watch',expression='depth',expected='100')=='100';assert driver.act('editor-is-retained',path='deep.py') and driver.act('editor-size',path='deep.py')==geometry
    before=[row for row in driver.act('debug-wire') if row.get('op') in ['breakpoints','source_breakpoints']];driver.act('source-tests-edit',path='deep.py',text=EDIT);selection=driver.act('construction-source',path='deep.py',selection=True);driver.act('debug-breakpoint',line=line);reading=driver.act('debug-breakpoint-reading');assert 'Earlier source' in reading and 'depth == 0' in reading,reading;after=[row for row in driver.act('debug-wire') if row.get('op') in ['breakpoints','source_breakpoints']];assert len(after)==len(before),(before,after);assert (root/'deep.py').read_text()==SOURCE;assert driver.act('construction-source',path='deep.py',selection=True)==selection
    driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'Deep session did not stop');driver.act('file-command',command='Save');wait(lambda:(root/'deep.py').read_text()==SOURCE+EDIT,'Edited source Save was not confirmed');driver.act('debug-breakpoint-condition',path='deep.py',line=line,condition='depth <= 0');driver.act('debug-start',path='deep.py');wait(lambda:any('paused' in text.lower() for text in driver.act('debug-state')),'Edited deep program did not pause');assert driver.act('debug-frame-index',index=100)['count']>=131;assert driver.act('debug-watch',expression='depth',expected='100')=='100';driver.act('debug-stop');wait(lambda:not any(any(word in text.lower() for word in ['paused','running','starting','unconfirmed','stopping']) for text in driver.act('debug-state')),'Edited deep session did not stop');driver.act('undo',path='deep.py');assert driver.act('construction-source',path='deep.py')==SOURCE;assert (root/'deep.py').read_text()==SOURCE+EDIT;assert driver.act('editor-is-retained',path='deep.py') and driver.act('editor-size',path='deep.py')==geometry;assert not driver.act('debug-diagnostics')['errors']
   else:
    if args.host=='apple':
     name='testLiveOwnedDeepStackAndEditedBreakpointRetention';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','DebuggerRetentionTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
    else:
     name='liveOwnedDeepStackAndEditedBreakpointRetention';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.DebuggerRetentionTest.'+name,'--rerun']
    log=ROOT/'target'/f'debugger-retention-{args.host}.log'
    with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
    if result.returncode:raise RuntimeError(f'{args.host} actual acceptance failed; inspect {log}')
    if args.host=='apple':
     text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
    else:
     report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.DebuggerRetentionTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
    assert (root/'deep.py').read_text()==SOURCE+EDIT
   print(f'PASS {args.host}: actual debugpy stack beyond 100 callers, scoped deep-frame reading, retained edited-source wishes without stale setters, explicit Save and fresh launch, retained editor/Undo, unchanged saved source during draft Undo. Physical input/accessibility acceptance remains unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-debugger-retention.py"><ui:paste select="#debugger-retention-owned-runner"/></ui:file>
</ui:doc>
