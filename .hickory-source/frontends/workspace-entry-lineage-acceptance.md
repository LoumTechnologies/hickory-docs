<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Verify older main Move Undo and Redo without borrowing a new identity

For maintainers closing the user-visible older-history path. The fixture uses the real engine and the existing Move, workspace Undo/Redo, source-history and editor interfaces. It requires the complete A → B → C → B → A → B → C sequence, with one stable main-role identity and six new generations. No toolbar or test-only product command is introduced.

## Retain a real saved baseline and a separate dirty draft

The runner reuses the existing owned Move and native fixture helpers. Before the editor opens, it makes a genuine guarded ordinary save from an earlier source to the published baseline, retaining a source-history identity. The editor then changes only its draft. Every reviewed Move or inverse must keep that baseline on disk and the distinct draft in the same editor, without generating either declared output. The earlier ordinary save must remain reachable through Local source history after all moves.

Protocol mode checks exact entry intents, complete recovery receipts, dirty and saved namespaces, and stale intent refusal after an independent same-spelling role cycle. Every host also holds a fresh Undo review while an unrelated equal-byte inode replaces its current source. Refusal must preserve the visible draft and current address; an unavailable role proof must not bootstrap a new identity.

The native clients retain the submitted request when the server refuses a
replaced source guard. A changed physical source also prevents the server from
confirming the original namespace in a no-effect receipt. Read the original
status without retrying publication and compare saved files and editor state.

<ui:copy id="workspace-entry-lineage-host-runner">
#!/usr/bin/env python3
"""Owned older main-Move Undo/Redo through real review, producer and editor paths."""
import argparse,hashlib,importlib.util,json,os,socket,tempfile,urllib.parse,urllib.error,uuid
from contextlib import ExitStack
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def load(path,name):
 spec=importlib.util.spec_from_file_location(name,ROOT/path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
move=load('scripts/test-workspace-entry-move-hosts.py','lineage_move_helpers')
A=move.ORIGINAL;B=move.DESTINATION;C='second #main.md';SAVED=move.PUBLISHED_SOURCE;DRAFT=move.DRAFT_SOURCE;EARLIER=SAVED.replace('saved.txt','before.txt')
def call(origin,method,path,body=None):return load('scripts/test-agent-proposal-persistence.py','lineage_http').call(origin,method,path,body)
def proof(origin):return call(origin,'GET','/api/workspace/entry')
def history(origin,path):return call(origin,'GET','/api/source-history?'+urllib.parse.urlencode({'path':path}))
def seed_history(origin):
 tree=call(origin,'GET','/api/files')['tree'];identifier=next(node['doc_id'] for node in tree if node['path']==A)
 document=call(origin,'GET','/api/docs/'+urllib.parse.quote(identifier,safe=''));assert document['source']==EARLIER
 identifier_save=uuid.uuid4().hex
 saved=call(origin,'PUT','/api/source-history/docs/'+urllib.parse.quote(identifier,safe=''),{'version':1,'request_id':identifier_save,'save_guard':document['save_guard'],'source':SAVED,'expected':{'source':EARLIER,'disk':EARLIER}})
 assert saved['history']['available'] is True and saved['history']['outcome']=='published' and saved['history']['id']==identifier_save,saved
 return identifier_save

def browser(driver,root,origin,seed):
 first=proof(origin)['entry'];driver.act('open',path=A);driver.act('remember-editor',path=A)
 location=SAVED.index('saved.txt');driver.act('source-text-select',path=A,anchor=location,head=location+len('saved.txt'));driver.act('type-source-selection',path=A,text='draft.txt')
 selection=driver.act('editor-position',path=A);size=driver.act('editor-size',path=A)
 def check(path,generation,position=None):
  stamp=move.entry(origin);assert stamp=={**first,'path':path,'generation':first['generation']+generation},stamp
  assert(root/path).read_text()==SAVED;assert all(not(root/other).exists() for other in [A,B,C] if other!=path)
  assert driver.act('construction-source',path=path)==DRAFT
  assert driver.act('editor-is-retained',path=path,previous=A)
  actual_position=driver.act('editor-position',path=path)
  expected_position=selection if position is None else position
  assert actual_position==expected_position,(actual_position,expected_position)
  assert driver.act('editor-size',path=path)==size
  assert not(root/'saved.txt').exists() and not(root/'draft.txt').exists()
 def publish(destination):
  driver.act('change-open',command='Move main construction…');driver.act('change-prepare',name=destination,input='New workspace-relative path',command='Prepare move');preview=driver.act('change-resource-ready');assert destination in preview,preview
  result=driver.act('change-resource-save');assert 'Saved source resources' in result,result;driver.act('change-close')
 check(A,0);publish(B);check(B,1);publish(C);check(C,2)
 for direction,path,generation in [('Undo',B,3),('Undo',A,4),('Redo',B,5),('Redo',C,6)]:
  driver.act('change-open',command=direction+' workspace change');preview=driver.act('change-resource-ready');assert path in preview,preview
  result=driver.act('change-resource-save');assert 'Saved source resources' in result,result;driver.act('change-close');check(path,generation)
 value=driver.act('local-history-open');assert 'unavailable' not in value.lower(),value
 selected=driver.act('local-history-select',id=seed,side='before');assert selected['id']==seed and selected['text']==EARLIER and selected['enabled'],selected;driver.act('local-history-close')
 before_history=driver.act('source-text-state',path=C)
 driver.act('undo',path=C)
 assert driver.act('construction-source',path=C)==SAVED
 driver.act('redo',path=C)
 # CodeMirror restores the replacement range on Redo. The publications above
 # preserve the original post-typing caret independently of this native edit.
 check(C,6,{**selection,'anchor':location,'head':location+len('draft.txt')})
 after_history=driver.act('source-text-state',path=C)
 assert (after_history['undo'],after_history['redo'])==(before_history['undo'],before_history['redo'])
 # A fresh review is held before an unrelated equal-byte inode replaces its source.
 driver.act('change-open',command='Undo workspace change');driver.act('change-resource-ready');state=driver.act('source-text-state',path=C)
 previous=(root/C).stat().st_ino;replacement=root/'independent-replacement';replacement.write_text(SAVED);os.replace(replacement,root/C);assert(root/C).stat().st_ino!=previous
 result=driver.act('change-resource-save');assert 'Saved source resources' not in result and any(word in result.lower() for word in ['changed','replac','physical','stale','inspect','publish','publication']),result
 assert driver.act('source-text-state',path=C)==state;assert(root/C).read_text()==SAVED;assert all(not(root/path).exists() for path in [A,B]);driver.act('change-close')
 print('PASS browser: Move A→B→C, reviewed Undo twice/Redo twice, exact role generations, saved/draft separation, original editor/selection/geometry/local history, native text Undo retained, equal-byte foreign inode refusal and no generated output materialization')

def protocol(root,origin,seed):
 first=proof(origin)['entry'];outcomes=[]
 def prepared(operation):
  current=proof(origin);review=call(origin,'POST','/api/workspace/entry/intent',{'version':1,'entry_selection':current,'operation':operation})
  path=current['entry']['path'];guard=current['source_guard'];capture={'context':review['context'],'edit':review['edit'],'drafts':{path:{'text':DRAFT,'base_sha256':hashlib.sha256(SAVED.encode()).hexdigest(),'version':99,'writable':True,'save_guard':guard}},'ownership_sources':{path:DRAFT},'saved_guards':{path:guard}}
  for _ in range(2):inputs=call(origin,'POST','/api/workspace/entry/inputs',{'version':1,'token':review['token'],'capture':capture});assert not inputs['refusals'],inputs
  proposal={'inputs':inputs,'capture':capture}
  if operation['kind']=='move':
   proposal['plan']=call(origin,'POST','/api/changes/plan-resources',{'context':review['context'],'title':review['title'],'edit':review['edit'],'sources':inputs['sources']})['plan'];facts=call(origin,'POST','/api/changes/resource-consequences',proposal);route='/api/changes/resource-save';extra={}
  else:
   proposal['recovery_id']=operation['recovery_id'];restoration=call(origin,'POST','/api/changes/prepare-restoration',proposal);facts=restoration['consequences'];route='/api/changes/restore-resources';extra={'reviewed_recovery_fingerprint':restoration['restoration']['recovery_fingerprint']}
  return route,{'request_id':uuid.uuid4().hex,'reviewed_fingerprint':facts['fingerprint'],'proposal':proposal,**extra},review,capture
 def run(operation,expected,generation):
  route,body,review,_=prepared(operation);result=call(origin,'POST',route,body);assert result['outcome']['complete'] and result['entry_outcome']['state']=='confirmed',result
  assert result['entry_outcome']['current_entry']==review['after_entry']=={**first,'generation':first['generation']+generation,'path':expected};assert(root/expected).read_text()==SAVED
  assert all(not(root/path).exists() for path in [A,B,C] if path!=expected)
  record=call(origin,'GET','/api/changes/resource-recovery?id='+Path(result['outcome']['recovery']).parent.name);outcomes.append(record);return record
 def inverse(record):return {'kind':'restoration','recovery_id':record['id'],'recovery_fingerprint':record['fingerprint']}
 m1=run({'kind':'move','destination':B},B,1);m2=run({'kind':'move','destination':C},C,2);u2=run(inverse(m2),B,3);u1=run(inverse(m1),A,4);run(inverse(u1),B,5);run(inverse(u2),C,6)
 assert any(row['id']==seed and row['available'] for row in history(origin,C)['entries'])
 def refused(route,body):
  try:result=call(origin,'POST',route,body)
  except urllib.error.HTTPError as error:
   assert error.code==409,(error.code,error.read());reason=error.read(4096).decode();assert any(word in reason.lower() for word in ['source','entry','changed','physical','producer','generation']),reason
  else:
   assert result['outcome']['complete'] is False and result['outcome']['source_files_modified'] is False and result['outcome']['source_files_uncertain'] is False,result
 held_route,held_body,_,_=prepared(inverse(outcomes[-1]));extra='unrelated main.md';run({'kind':'move','destination':extra},extra,7);run({'kind':'move','destination':C},C,8);refused(held_route,held_body);assert not(root/extra).exists()
 route,body,_,_=prepared(inverse(outcomes[-1]));previous=(root/C).stat().st_ino;replacement=root/'independent-replacement';replacement.write_text(SAVED);os.replace(replacement,root/C);assert(root/C).stat().st_ino!=previous;refused(route,body)
 assert(root/C).read_text()==SAVED and all(not(root/path).exists() for path in [A,B]);print('PASS protocol: exact complete older producer Undo/Redo, saved/dirty plans, retained local history, independent same-path role ABA and equal-byte inode refusals; no UI or physical-input claim')

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--host',choices=['protocol','browser','apple','android'],required=True);parser.add_argument('--engine',default=os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')));args=parser.parse_args()
 fixture=move.module('scripts/test-simple-browser.py','lineage_browser_fixture');native=move.module('scripts/test-native-templates.py','lineage_native_fixture');owned=move.module('scripts/test-agent-proposal-persistence.py','lineage_owned_engine')
 with tempfile.TemporaryDirectory(prefix='entry-lineage-host-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory-entry-move-fixture').write_text('owned-entry-move-v1\n');(root/A).write_text(EARLIER);(root/'part.md').write_text('\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:copy id="part">source\x3c/h:copy>\x3c/h:doc>\n')
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  ui=f'http://127.0.0.1:{port}';env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  with ExitStack() as cleanup:
   engine=fixture.start([args.engine,str(root),'window',ui],env=env);cleanup.callback(owned.stop_engine,fixture,engine,folder/'state');origin=fixture.origin(engine);seed=seed_history(origin);baseline=proof(origin)['entry'];os.environ['HICKORY_ENTRY_LINEAGE_HISTORY']=seed
   if args.host=='protocol':protocol(root,origin,seed)
   elif args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});cleanup.callback(fixture.stop,vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');cleanup.callback(driver.close);browser(driver,root,origin,seed)
   else:
    case=('WorkspaceEntryLineageTests','testLiveOlderMainMoveUndoRedo','entry-lineage') if args.host=='apple' else ('WorkspaceEntryLineageTest','liveOlderMainMoveUndoRedo','entry-lineage');native.run_host(args.host,origin,root,folder,fixture,acceptance=case)
   assert(root/C).read_text()==SAVED and all(not(root/path).exists() for path in [A,B]);assert not(root/'saved.txt').exists() and not(root/'draft.txt').exists()
   try:retained=proof(origin)['entry']
   except urllib.error.HTTPError as error:assert error.code==409,(error.code,error.read())
   else:assert retained['identity']==baseline['identity'] and retained['path']==C and retained['generation']>=baseline['generation']+6,retained
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-workspace-entry-lineage.py"><ui:paste select="#workspace-entry-lineage-host-runner"/></ui:file>

## Exercise the mounted macOS editor

Use the production command descriptors, compact resource review and publication controller. Preserve the actual NSTextView, native undo manager, selection and geometry while checking the main role and authored file namespace at each generation. Reading local history must preserve its original source record, and a foreign inode must retire the held Undo before any source writer.

<ui:copy id="workspace-entry-lineage-apple-live">
import XCTest
#if os(macOS)
import AppKit
import Darwin
#endif
@testable import HickorySimple
@MainActor final class WorkspaceEntryLineageTests:XCTestCase {
 #if os(macOS)
 func testLiveOlderMainMoveUndoRedo()async throws {
  guard let address=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"],let historyID=ProcessInfo.processInfo.environment["HICKORY_ENTRY_LINEAGE_HISTORY"] else{throw XCTSkip("Use test-workspace-entry-lineage.py --host apple")}
  let root=URL(fileURLWithPath:directory),a="project.md",b="literal #main.spec",c="second #main.md"
  func disk(_ path:String)throws->String{try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8)}
  XCTAssertEqual(try disk(".hickory-entry-move-fixture"),"owned-entry-move-v1\n")
  let engine=try Engine(address:address),tree=try await engine.files(),space=Workspace();space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id
  let buffer=try await space.loadBuffer(XCTUnwrap(tree.tree.first{$0.path==a}));space.layout.activate(a,in:"document-group")
  let saved=buffer.text,draft=saved.replacingOccurrences(of:"saved.txt",with:"draft.txt"),earlier=saved.replacingOccurrences(of:"saved.txt",with:"before.txt")
  let original=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
  _=NSApplication.shared
  let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil)
  defer{window.orderOut(nil);for value in space.buffers.values{value.language?.close()}}
  let native=editor.text,view=try XCTUnwrap(buffer.sourceEditorState?()),manager=try XCTUnwrap(native.undoManager),range=(saved as NSString).range(of:"saved.txt")
  native.setSelectedRange(range);native.insertText("draft.txt",replacementRange:range)
  let selection=native.selectedRange(),geometry=editor.scroll.frame
  func command(_ id:String)throws{let selected=try XCTUnwrap(space.commands().first{$0.id==id});XCTAssertNil(selected.reason);selected.run()}
  func wait(_ ready:()->Bool)async throws{for _ in 0..<400{if ready(){return};try await Task.sleep(for:.milliseconds(25))};throw EngineError(message:space.changes.notice+" "+space.resourceRestoration.notice)}
  func check(_ path:String,_ generation:UInt32)async throws{
   let stamp=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
   XCTAssertEqual(stamp.path,path);XCTAssertEqual(stamp.identity,original.identity);XCTAssertEqual(stamp.generation,original.generation+generation)
   XCTAssertEqual(try disk(path),saved);for other in [a,b,c] where other != path{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(other).path))}
   XCTAssertTrue(space.currentBuffer===buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved);XCTAssertEqual(buffer.sourceEditorState?()?.identity,view.identity);XCTAssertTrue(native.undoManager===manager);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
   for output in ["saved.txt","draft.txt"]{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(output).path))}
  }
  func move(_ destination:String)async throws{
   try command("workspace.moveEntry");try await wait{space.changes.entrySelection?.entryPath==buffer.file.path};space.changes.name=destination;await space.changes.prepare(space)
   XCTAssertEqual(space.resourceRestoration.review?.entryIntent?.after_entry.path,destination,space.resourceRestoration.notice)
   await space.resourceRestoration.publish(space);XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  }
  try await check(a,0);try await move(b);try await check(b,1);try await move(c);try await check(c,2)
  for(id,path,generation) in [("workspace.undo",b,UInt32(3)),("workspace.undo",a,UInt32(4)),("workspace.redo",b,UInt32(5)),("workspace.redo",c,UInt32(6))]{
   try command(id);try await wait{space.resourceRestoration.review != nil};XCTAssertEqual(space.resourceRestoration.review?.entryIntent?.after_entry.path,path,space.resourceRestoration.notice)
   await space.resourceRestoration.publish(space);XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice);try await check(path,generation)
  }
  await space.sourceHistory.open(space);let historical=try XCTUnwrap(space.sourceHistory.reading?.entries.first{$0.id==historyID},space.sourceHistory.notice);XCTAssertTrue(historical.available);XCTAssertEqual(historical.before.text,earlier);XCTAssertEqual(historical.after.text,saved);space.sourceHistory.close()
  manager.undo();XCTAssertEqual(buffer.text,saved);manager.redo();try await check(c,6)
  try command("workspace.undo");try await wait{space.resourceRestoration.review != nil}
  let originalInode=try FileManager.default.attributesOfItem(atPath:root.appendingPathComponent(c).path)[.systemFileNumber] as? NSNumber
  let replacement=root.appendingPathComponent("independent-replacement");try saved.write(to:replacement,atomically:false,encoding:.utf8)
  let result=replacement.path.withCString{source in root.appendingPathComponent(c).path.withCString{destination in Darwin.rename(source,destination)}};XCTAssertEqual(result,0)
  let replacedInode=try FileManager.default.attributesOfItem(atPath:root.appendingPathComponent(c).path)[.systemFileNumber] as? NSNumber;XCTAssertNotEqual(originalInode,replacedInode)
  await space.resourceRestoration.publish(space)
  XCTAssertTrue(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  XCTAssertTrue(space.resourceRestoration.notice.contains("physical source guard"),space.resourceRestoration.notice)
  await space.resourceRestoration.status(space)
  XCTAssertTrue(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  XCTAssertTrue(space.resourceRestoration.notice.contains("physical source guard"),space.resourceRestoration.notice)
  XCTAssertTrue(space.currentBuffer===buffer);XCTAssertEqual(buffer.file.path,c);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry);XCTAssertEqual(try disk(c),saved)
  for absent in [a,b,"saved.txt","draft.txt"]{XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(absent).path))}
 }
 #endif
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/WorkspaceEntryLineageTests.swift"><ui:paste select="#workspace-entry-lineage-apple-live"/></ui:file>

## Exercise the Android controller

Drive the same existing command and review path against the real engine, retaining the original text state, scroll and local undo history. This is host/controller acceptance, not physical touch, device IME or accessibility acceptance.

<ui:copy id="workspace-entry-lineage-android-live">
package com.hickorydocs.simple
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Assume.assumeNotNull
import org.junit.Test
import java.io.File
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class WorkspaceEntryLineageTest {
 @Test(timeout=120000) fun liveOlderMainMoveUndoRedo()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");assumeNotNull(origin)
  val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use test-workspace-entry-lineage.py --host android")).canonicalFile
  val historyID=System.getenv("HICKORY_ENTRY_LINEAGE_HISTORY") ?: error("Missing owned ordinary source history seed")
  assertEquals("owned-entry-move-v1\n",root.resolve(".hickory-entry-move-fixture").readText())
  val a="project.md";val b="literal #main.spec";val c="second #main.md";val engine=Engine(origin!!);val (_,tree)=engine.files();val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val space=Workspace(background);space.engine=engine;space.rootFiles=tree
  try{
   val buffer=space.loadBuffer(tree.first{it.path==a});space.show(a);val saved=buffer.text;val draft=saved.replace("saved.txt","draft.txt");val earlier=saved.replace("saved.txt","before.txt");val state=buffer.state;val at=saved.indexOf("saved.txt")
   state.edit{replace(at,at+"saved.txt".length,"draft.txt");selection=TextRange(at+"draft.txt".length)};buffer.advanceSourceRevision();val selection=state.selection;val scroll=buffer.sourceScroll;val original=space.resourceChanges.readEntry(space){true}.proof.entry
   val descriptors=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()})
   fun command(id:String){val selected=space.commands(descriptors).single{it.id==id};assertNull(selected.reason);selected.run()}
   suspend fun wait(ready:()->Boolean){repeat(400){if(ready())return;delay(25)};error(space.changes.notice+" "+space.resourceChanges.notice)}
   suspend fun check(path:String,generation:Long){
    val stamp=space.resourceChanges.readEntry(space){true}.proof.entry;assertEquals(path,stamp.path);assertEquals(original.identity,stamp.identity);assertEquals(original.generation+generation,stamp.generation)
    assertEquals(saved,root.resolve(path).readText());for(other in listOf(a,b,c).filter{it!=path})assertFalse(root.resolve(other).exists())
    assertSame(buffer,space.currentBuffer);assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll);assertEquals(selection,state.selection);assertEquals(saved,buffer.base);assertEquals(draft,buffer.text);for(output in listOf("saved.txt","draft.txt"))assertFalse(root.resolve(output).exists())
   }
   suspend fun publish(){space.resourceChanges.apply(space);assertFalse(space.resourceChanges.notice,space.resourceChanges.hasPending)}
   suspend fun move(destination:String){command("workspace.moveEntry");wait{space.changes.entrySelection?.entryPath==buffer.file.path};space.changes.updateName(destination);space.changes.prepare(space);assertEquals(space.resourceChanges.notice,destination,space.resourceChanges.review?.entryIntent?.after?.path);publish()}
   check(a,0);move(b);check(b,1);move(c);check(c,2)
   for((id,path,generation) in listOf(Triple("workspace.undo",b,3L),Triple("workspace.undo",a,4L),Triple("workspace.redo",b,5L),Triple("workspace.redo",c,6L))){command(id);wait{space.resourceChanges.review!=null};assertEquals(space.resourceChanges.notice,path,space.resourceChanges.review?.entryIntent?.after?.path);publish();check(path,generation)}
   space.sourceHistory.open(space);val historical=space.sourceHistory.reading?.entries?.singleOrNull{it.id==historyID} ?: error(space.sourceHistory.notice);assertTrue(historical.available);assertEquals(earlier,historical.before.text);assertEquals(saved,historical.after.text);space.sourceHistory.close()
   assertTrue(state.undoState.canUndo);state.undoState.undo();assertEquals(saved,buffer.text);state.undoState.redo();check(c,6)
   command("workspace.undo");wait{space.resourceChanges.review!=null}
   val path=root.resolve(c).toPath();val before=java.nio.file.Files.readAttributes(path,java.nio.file.attribute.BasicFileAttributes::class.java).fileKey();val replacement=root.resolve("independent-replacement");replacement.writeText(saved)
   java.nio.file.Files.move(replacement.toPath(),path,java.nio.file.StandardCopyOption.ATOMIC_MOVE,java.nio.file.StandardCopyOption.REPLACE_EXISTING);assertNotEquals(before,java.nio.file.Files.readAttributes(path,java.nio.file.attribute.BasicFileAttributes::class.java).fileKey())
   space.resourceChanges.apply(space)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.hasPending)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.notice.contains("physical source guard"))
   space.resourceChanges.status(space)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.hasPending)
   assertTrue(space.resourceChanges.notice,space.resourceChanges.notice.contains("physical source guard"))
   assertSame(buffer,space.currentBuffer);assertEquals(c,buffer.file.path);assertEquals(draft,buffer.text);assertEquals(saved,buffer.base);assertEquals(selection,state.selection);assertSame(scroll,buffer.sourceScroll);assertEquals(saved,root.resolve(c).readText());for(absent in listOf(a,b,"saved.txt","draft.txt"))assertFalse(root.resolve(absent).exists())
  }finally{background.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/WorkspaceEntryLineageTest.kt"><ui:paste select="#workspace-entry-lineage-android-live"/></ui:file>

## Execution status

The maintained runner and both native witnesses are source-only pending the complete producer-lineage and guarded publisher bridges. Target Python syntax and Swift frontend parsing passed; no real Undo/Redo, physical-input, accessibility or iOS runtime completion is claimed by this fixture before its exact live modes execute. Each native live mode must produce exactly one passing case without skips through the existing retained receipt runner.

</ui:doc>
