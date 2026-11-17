<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise private recovery capacity through the actual simple hosts

For maintainers checking that bounded storage remains usable after many sessions.

Seed32 owned private sessions with3 revisions each, then make one bounded record unreadable. Every host must expose all pages, select the exact old material, recover draft copies beside an independent unsaved editor, and explicitly forget a complete private session to unblock checkpointing. Stale revision and physical replacement guards refuse; exact lost-reply replay does not delete a replacement. The test owns all source and state and checks source bytes afterward. Scripted browser controls and native controller/renderer paths do not establish physical-input or accessibility acceptance.

## Drive one real-engine fixture for HTTP, browser and native acceptance

The HTTP path checks authoritative storage and failure cases; host paths use their production recovery models and existing compact surfaces. Native tests require an explicit owned marker and never accept an arbitrary workspace.

<ui:copy id="editor-recovery-management-protocol">
#!/usr/bin/env python3
"""Owned real-engine private session capacity/revision/Forget acceptance."""
import argparse,hashlib,importlib.util,json,os,re,socket,subprocess,tempfile,time,urllib.error,urllib.parse,urllib.request,uuid,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TARGET=f'{32:032x}';CORRUPT=f'{31:032x}';FILES={'note.py':'value = 1\n'+''.join(f'# readable source line {i}\n' for i in range(100)),'.hickory-native-recovery-management-fixture':'owned-recovery-management-v1\n'}
def call(origin,method,path,body=None):
 request=urllib.request.Request(origin+path,data=None if body is None else json.dumps(body).encode(),method=method,headers={'Content-Type':'application/json'})
 with urllib.request.urlopen(request,timeout=60) as response:return json.load(response)
def refused(operation):
 try:operation()
 except urllib.error.HTTPError as error:assert error.code in (400,403,404,409,422),(error.code,error.read());return
 raise AssertionError('Stale, replaced or over-capacity private operation was admitted')
def snapshot(session,revision):return {'host':'browser','buffers':[{'id':f'{session+1000:032x}','address':{'kind':'unnamed','name':f'private session {session}'},'text':f'private session {session} revision {revision}\n','baseline':None,'view':{'selection':{'anchor':0,'head':0},'scroll_x':0,'scroll_y':revision*7}}],'layout':{}}
def checkpoint(origin,meta,client,base,value):return call(origin,'POST','/api/workspace/recovery/checkpoint',{'version':1,'workspace':meta['engine_workspace_id'],'window_slot':meta['current_window_slot'],'client_id':client,'request_id':uuid.uuid4().hex,'base_revision':base,'snapshot':value})
def catalogue(origin):
 allrows=[];bad=[];after=None;scope=None
 for _ in range(33):
  page=call(origin,'GET','/api/workspace/recovery'+('?' + urllib.parse.urlencode({'after':after}) if after else ''));scope=scope or page;assert page['engine_workspace_id']==scope['engine_workspace_id'] and page['workspace_id']==scope['workspace_id'];allrows+=page['sessions'];bad+=page.get('unreadable_sessions',[]);next_cursor=page['next'];assert next_cursor is None or after is None or next_cursor>after
  if next_cursor is None:return {**scope,'sessions':allrows,'unreadable_sessions':bad,'next':None}
  after=next_cursor
 raise AssertionError('Recovery cursor did not terminate within bounded sessions')
def forget_body(meta,row):return {'version':1,'workspace':meta['engine_workspace_id'],'window_slot':row['window_slot'],'client_id':row['client_id'],'request_id':uuid.uuid4().hex,'record_guard':row['record_guard']}
def seed(origin,state):
 meta=catalogue(origin)
 for session in range(1,33):
  for revision in range(1,4):checkpoint(origin,meta,f'{session:032x}',revision-1,snapshot(session,revision))
 refused(lambda:checkpoint(origin,meta,f'{33:032x}',0,snapshot(33,1)))
 candidates=list(state.rglob(CORRUPT+'.json'));assert len(candidates)==1;candidates[0].write_bytes(b'owned corrupt private record; not a snapshot')
 result=catalogue(origin);assert len(result['sessions'])==31 and len(result['unreadable_sessions'])==1;return result
def service(origin,state,meta):
 row=next(row for row in meta['sessions'] if row['client_id']==TARGET);assert [item['revision'] for item in row['revisions']]==[1,2,3]
 for version in row['revisions']:
  value=call(origin,'GET','/api/workspace/recovery/'+TARGET+'?'+urllib.parse.urlencode({'window_slot':row['window_slot'],'revision':version['revision']}));assert value['snapshot_hash']==version['snapshot_hash'];assert hashlib.sha256(value['material'].encode()).hexdigest()==value['snapshot_hash'];assert json.loads(value['material'])['buffers'][0]['text']==snapshot(32,version['revision'])['buffers'][0]['text']
 old=forget_body(meta,row);checkpoint(origin,meta,TARGET,3,snapshot(32,4));refused(lambda:call(origin,'POST','/api/workspace/recovery/forget',old));assert call(origin,'GET','/api/workspace/recovery/'+TARGET)['revision']==4
 meta=catalogue(origin);row=next(row for row in meta['sessions'] if row['client_id']==TARGET);body=forget_body(meta,row);receipt=call(origin,'POST','/api/workspace/recovery/forget',body);assert receipt['forgotten'] and receipt['source_effects'] is False and receipt['source_saved'] is False;checkpoint(origin,meta,f'{33:032x}',0,snapshot(33,1))
 bad=next(row for row in meta['unreadable_sessions'] if row['client_id']==CORRUPT);call(origin,'POST','/api/workspace/recovery/forget',forget_body(meta,bad));checkpoint(origin,meta,TARGET,0,snapshot(32,9));replay=call(origin,'POST','/api/workspace/recovery/forget',body);assert replay['replayed'] is True;assert json.loads(call(origin,'GET','/api/workspace/recovery/'+TARGET)['material'])['buffers'][0]['text']==snapshot(32,9)['buffers'][0]['text']
 latest=catalogue(origin);row=next(row for row in latest['sessions'] if row['client_id']==TARGET);stale=forget_body(latest,row);path=next(state.rglob(TARGET+'.json'));replacement=path.with_suffix('.replacement');replacement.write_bytes(path.read_bytes());os.replace(replacement,path);refused(lambda:call(origin,'POST','/api/workspace/recovery/forget',stale));assert call(origin,'GET','/api/workspace/recovery/'+TARGET)['revision']==1
 print('PASS real recovery HTTP:32capacity refusal, all3 exact retained revisions, stale/latest/physical ABA refusal, corrupt bounded record removal without adoption, capacity reopened, exact replay preserves replacement; no source saves.',flush=True)
def browser(fixture,folder,root,origin,meta,port):
 driver=None;vite=None
 try:
  vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**os.environ,'HICKORY_API_ORIGIN':origin});driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='note.py');driver.act('source-tests-edit',path='note.py',text='# independent unsaved source\n');before=driver.act('construction-source',path='note.py');driver.act('remember-editor',path='note.py');position=driver.act('editor-position',path='note.py',anchor=2,head=4,y=90);driver.act('session-open')
  for _ in range(3):driver.act('session-more')
  state=driver.act('session-state');assert state['readCount']==31 and CORRUPT in state['text'] and not state['more'],state
  for revision in (1,2,3):assert snapshot(32,revision)['buffers'][0]['text'].strip() in driver.act('session-read',client=TARGET,revision=revision)
  driver.act('session-read',client=TARGET,revision=1);restored=driver.act('session-restore');assert any(row['text']==snapshot(32,1)['buffers'][0]['text'] for row in restored);assert driver.act('editor-is-retained',path='note.py');assert driver.act('construction-source',path='note.py')==before;assert driver.act('editor-position',path='note.py')['anchor']==position['anchor']
  driver.act('session-open');[driver.act('session-more') for _ in range(3)];driver.act('session-forget',client=CORRUPT);deadline=time.monotonic()+15
  while time.monotonic()<deadline:
   current=catalogue(origin)
   if len(current['sessions'])==32 and not current['unreadable_sessions']:break
   time.sleep(.1)
  else:raise AssertionError('Current readable editor did not checkpoint after capacity was freed')
  driver.act('session-refresh');[driver.act('session-more') for _ in range(3)];checkpoint(origin,meta,TARGET,3,snapshot(32,4));failed=driver.act('session-forget',client=TARGET);assert 'changed' in failed.lower(),failed;driver.act('session-stop-forget');[driver.act('session-more') for _ in range(3)];assert 'revision 4' in driver.act('session-read',client=TARGET,revision=4).lower();driver.act('session-forget',client=TARGET);assert not any(row['client_id']==TARGET for row in catalogue(origin)['sessions']);driver.act('session-close');assert driver.act('editor-is-retained',path='note.py');assert driver.act('construction-source',path='note.py')==before;driver.act('undo',path='note.py');assert driver.act('construction-source',path='note.py')==FILES['note.py'];assert not driver.act('debug-diagnostics')['errors'];print('PASS browser compact recovery: all32/corrupt slot, all3 revisions, exact old draft copy, stale removal refusal, explicit Forget frees checkpoint capacity, retained source editor/selection/Undo and unchanged saved bytes. Scripted controls, not physical/accessibility acceptance.',flush=True)
 finally:
  if driver:driver.close()
  if vite:fixture.stop(vite)
def native(host,origin,root,skip_build):
 env={**os.environ,'HICKORY_RECOVERY_MANAGEMENT_ORIGIN':origin,'HICKORY_RECOVERY_MANAGEMENT_WORKSPACE':str(root)}
 if host=='apple':name='testOwnedCapacityRevisionAndForgetFlow';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','EditorRecoveryManagementAcceptanceTests.'+name+'$']+(['--skip-build'] if skip_build else [])
 else:name='ownedCapacityRevisionAndForgetFlow';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.EditorRecoveryManagementAcceptanceTest.'+name,'--rerun']
 log=ROOT/'target'/f'recovery-management-{host}-actual.log'
 with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
 if result.returncode:raise RuntimeError(f'{host} recovery management failed; inspect {log}')
 if host=='apple':text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and re.search(r'Executed 1 test, with 0 failures',text) and 'Test skipped' not in text,str(log)
 else:
  report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.EditorRecoveryManagementAcceptanceTest.xml';cases=list(ET.parse(report).getroot().iter('testcase'));assert len(cases)==1 and cases[0].get('name','').split('(')[0]==name and not any(cases[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
 print(f'PASS {host}: exact owned capacity/revision/Forget acceptance case, no skips; no physical/accessibility claim.',flush=True)
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['service','browser','apple','android']);parser.add_argument('--skip-build',action='store_true');args=parser.parse_args();spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-recovery-management-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'private-state'
  for path,text in FILES.items():(root/path).write_text(text)
  port=None
  if args.host=='browser':
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  ui_origin=f'http://127.0.0.1:{port}' if port else 'http://127.0.0.1:1'
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui_origin],env={**os.environ,'HICKORY_STATE_DIR':str(state)})
  try:
   origin=fixture.origin(engine);meta=seed(origin,state)
   if args.host=='service':service(origin,state,meta)
   elif args.host=='browser':browser(fixture,folder,root,origin,meta,port)
   else:native(args.host,origin,root,args.skip_build)
   for path,text in FILES.items():assert (root/path).read_text()==text,path
   assert not (root/'.hick-cache').exists();print('PASS all owned source bytes retained; private recovery management does not save or publish source.',flush=True)
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>

<ui:file path="scripts/test-editor-recovery-management.py"><ui:paste select="#editor-recovery-management-protocol"/></ui:file>

## Exercise the retained catalogue in real native hosts

These opted-in cases connect production host controllers and editor buffers to the owned engine. They prove pagination, complete revision selection, guarded removal and preservation of live drafts. Automation is not physical-input or accessibility acceptance.

<ui:copy id="editor-recovery-management-apple-actual">
import XCTest
import SwiftUI
@testable import HickorySimple
#if os(macOS)
import AppKit
@MainActor final class EditorRecoveryManagementAcceptanceTests:XCTestCase {
 func testOwnedCapacityRevisionAndForgetFlow() async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_RECOVERY_MANAGEMENT_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_RECOVERY_MANAGEMENT_WORKSPACE"] else{throw XCTSkip("Use the owned recovery management fixture.")}
  XCTAssertEqual(try String(contentsOfFile:root+"/.hickory-native-recovery-management-fixture",encoding:.utf8),"owned-recovery-management-v1\n")
  _=NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let listing=try await engine.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  func find(_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path=="note.py"{return node};if let found=find(node.children ?? []){return found}};return nil}
  let b=try await w.loadBuffer(try XCTUnwrap(find(listing.tree)));w.showSource(b.file.path);let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:980,height:700),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
  func settle(_ message:String,_ predicate:()->Bool)async throws{for _ in 0..<600{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:message)}
  func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
  try await settle("Original code editor did not attach"){b.sourceEditorState?()?.attached==true};let editor=try XCTUnwrap(native(host)),manager=try XCTUnwrap(editor.undoManager),source=b.text,editorIdentity=try XCTUnwrap(b.sourceEditorState?()).identity,scroll=try XCTUnwrap(editor.enclosingScrollView)
  editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("# independent dirty draft\n",replacementRange:editor.selectedRange());editor.setSelectedRange(NSRange(location:2,length:3));b.selection=editor.selectedRange();scroll.contentView.scroll(to:CGPoint(x:0,y:90));scroll.reflectScrolledClipView(scroll.contentView)
  let text=b.text,selection=b.selection,identity=ObjectIdentifier(b),viewport=scroll.contentView.bounds.origin,m=w.editorRecovery
  defer{m.close();w.sourceTestIndex?.stop();for buffer in w.buffers.values{buffer.language?.close()};window.orderOut(nil);window.contentView=nil}
  func allPages()async throws{for _ in 0..<32{guard let previous=m.catalogue?.next else{return};await m.more(w);guard m.catalogue?.next != previous else{throw EngineError(message:"Recovery page did not advance: "+m.notice)}};guard m.catalogue?.next==nil else{throw EngineError(message:"Recovery exceeded bounded page count.")}}
  m.showing=true;host.layoutSubtreeIfNeeded();try await settle("Mounted recovery popover did not load"){!m.busy&&m.catalogue != nil};try await allPages()
  XCTAssertEqual(m.catalogue?.sessions.count,31,m.notice);XCTAssertEqual(m.catalogue?.unreadable_sessions?.count,1)
  let row=try XCTUnwrap(m.catalogue?.sessions.first{$0.client_id==String(format:"%032x",32)})
  for revision in [UInt32(1),2,3]{await m.read(row,w,revision:revision);XCTAssertEqual(m.snapshot?.buffers.first?.text,"private session 32 revision \(revision)\n",m.notice);XCTAssertTrue(m.canRestore,m.notice);XCTAssertEqual(b.sourceEditorState?()?.identity,editorIdentity);XCTAssertEqual(scroll.contentView.bounds.origin,viewport);XCTAssertEqual(b.selection,selection)}
  await m.read(row,w,revision:1);m.restore(w);XCTAssertTrue(w.buffers.values.contains{$0.isUntitledDraft&&$0.text=="private session 32 revision 1\n"},m.notice)
  await m.open(w);try await allPages()
  let corrupt=try XCTUnwrap(m.catalogue?.unreadable_sessions?.first);await m.forget(corrupt.removal,w);XCTAssertFalse(m.hasPendingForget,m.notice)
  let chosen=try XCTUnwrap(m.catalogue?.sessions.first{$0.client_id==row.client_id});await m.forget(chosen.removal,w);XCTAssertFalse(m.hasPendingForget,m.notice)
  await m.open(w);try await allPages();XCTAssertFalse(m.catalogue!.sessions.contains{$0.client_id==row.client_id})
  XCTAssertEqual(ObjectIdentifier(b),identity);XCTAssertEqual(b.sourceEditorState?()?.identity,editorIdentity);XCTAssertEqual(b.text,text);XCTAssertEqual(b.selection,selection);XCTAssertTrue(manager.canUndo);m.close();manager.undo();XCTAssertEqual(b.text,source);XCTAssertFalse(try String(contentsOfFile:root+"/note.py",encoding:.utf8).contains("independent dirty"))
 }
}
#endif
</ui:copy>
<ui:file path="apps/apple-simple/Tests/EditorRecoveryManagementAcceptanceTests.swift"><ui:paste select="#editor-recovery-management-apple-actual"/></ui:file>

<ui:copy id="editor-recovery-management-android-actual">
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import androidx.compose.ui.text.TextRange
class EditorRecoveryManagementAcceptanceTest {
 @Test(timeout=120000) fun ownedCapacityRevisionAndForgetFlow()=runBlocking {
  val origin=System.getenv("HICKORY_RECOVERY_MANAGEMENT_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_RECOVERY_MANAGEMENT_WORKSPACE") ?: error("Use owned recovery fixture"));check(root.resolve(".hickory-native-recovery-management-fixture").readText()=="owned-recovery-management-v1\n")
  val scope=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(scope);val engine=Engine(origin!!);w.engine=engine;val (_,tree)=engine.files();w.rootFiles=tree
  fun find(nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path=="note.py")return node;find(node.children)?.let{return it}};return null}
  val b=w.loadBuffer(find(tree) ?: error("Missing owned source"));w.show(sourceTab(b.file.path));val state=b.state;assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# independent dirty draft\n")),b.text));state.edit{selection=TextRange(2,5)};val text=b.text;val selection=state.selection;val m=w.editorRecovery
  suspend fun allPages(){repeat(32){val previous=m.catalogue?.next ?: return;m.more(w);check(m.catalogue?.next!=previous){"Recovery page did not advance: "+m.notice}};check(m.catalogue?.next==null){"Recovery exceeded bounded page count."}}
  try{
   m.open(w);allPages();assertEquals(m.notice,31,m.catalogue!!.sessions.size);assertEquals(1,m.catalogue!!.unreadable.size)
   val row=m.catalogue!!.sessions.first{it.client=="%032x".format(32)}
   for(revision in 1L..3L){m.read(row,w,revision);assertEquals(m.notice,"private session 32 revision $revision\n",m.snapshot?.buffers?.first()?.text);assertTrue(m.notice,m.canRestore)}
   m.read(row,w,1);assertTrue(m.notice,m.restore(w));assertTrue(w.buffers.values.any{it.isUntitledDraft&&it.text=="private session 32 revision 1\n"})
   m.open(w);allPages();m.forget(m.catalogue!!.unreadable.first().removal,w);assertFalse(m.notice,m.hasPendingForget);m.forget(m.catalogue!!.sessions.first{it.client==row.client}.removal,w);assertFalse(m.notice,m.hasPendingForget)
   m.open(w);allPages();assertFalse(m.catalogue!!.sessions.any{it.client==row.client});assertSame(state,b.state);assertEquals(text,b.text);assertEquals(selection,state.selection);assertFalse(root.resolve("note.py").readText().contains("independent dirty"))
  }finally{m.close();w.buffers.values.forEach{it.language?.close()};scope.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/EditorRecoveryManagementAcceptanceTest.kt"><ui:paste select="#editor-recovery-management-android-actual"/></ui:file>

</ui:doc>
