<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Move the current main construction through a reviewed source change

For maintainers verifying the current-entry move and its inverse through the
existing Edit menu. Each run owns its workspace and private state. It opens the
main source, makes a dirty draft, previews the exact saved and draft states, and
publishes through the resource publisher. Undo and Redo each require a new review
of the retained role and advance its generation. Editor identity and history
follow the same authored source as its path changes.

The browser also puts a foreign file at the destination after preview. The
publication must refuse before changing either file or the main role. The next
review starts from current authority. A literal space, hash and non-Markdown
extension exercise independent source addressing. The default construction
reader follows the new role and retains separate saved and draft outputs.
Compiler output remains unmaterialized. The mounted Mac and Android controller witnesses have separate
receipts. Physical input, accessibility and iOS runtime require their own runs.

The optional foreign-room case starts a read-only collaboration peer on a
temporary destination, then removes the disk file before preparing the move.
The same visible command and native controllers must retain the original draft,
selection, editor history and role after the engine confirms zero writes. A
fresh Preview remains available in the same workspace.

<ui:copy id="workspace-entry-move-host-runner">
#!/usr/bin/env python3
"""Exercise a reviewed main-construction move against an owned real engine."""
import argparse
from contextlib import ExitStack
import importlib.util
import json
import os
import select
import socket
import subprocess
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL = 'project.md'
DESTINATION = 'literal #main.spec'
PUBLISHED_SOURCE = ('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">'
                    '<h:project sources="part.md"/>'
                    '<h:file path="saved.txt"><h:paste select="#part"/></h:file>'
                    '</h:doc>\n')
DRAFT_SOURCE = PUBLISHED_SOURCE.replace('saved.txt', 'draft.txt')


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def entry(origin):
    with urllib.request.urlopen(origin + '/api/workspace/entry', timeout=20) as reply:
        value = json.load(reply)
    assert value['version'] == 1 and value['source_effects'] is False, value
    return value['entry']


def browser_flow(driver, root, origin, destination=DESTINATION, foreign_room=False):
    first = entry(origin)
    assert first['path'] == ORIGINAL, first
    driver.act('open', path=ORIGINAL)
    driver.act('remember-editor', path=ORIGINAL)
    location = PUBLISHED_SOURCE.index('saved.txt')
    driver.act('source-text-select', path=ORIGINAL, anchor=location, head=location + len('saved.txt'))
    driver.act('type-source-selection', path=ORIGINAL, text='draft.txt')
    assert driver.act('construction-source', path=ORIGINAL) == DRAFT_SOURCE
    selection = driver.act('editor-position', path=ORIGINAL)
    size = driver.act('editor-size', path=ORIGINAL)

    def prepare():
        driver.act('change-open', command='Move main construction…')
        driver.act('change-prepare', name=destination, input='New workspace-relative path', command='Prepare move')
        return driver.act('change-resource-ready')

    def assert_source(path, absent, generation):
        stamp = entry(origin)
        assert stamp['identity'] == first['identity'] and stamp['generation'] == first['generation'] + generation, stamp
        assert stamp['path'] == path, stamp
        assert not (root / absent).exists()
        assert (root / path).read_text() == PUBLISHED_SOURCE
        assert driver.act('construction-source', path=path) == DRAFT_SOURCE
        assert driver.act('editor-is-retained', path=path, previous=ORIGINAL)
        assert driver.act('editor-position', path=path) == selection
        assert driver.act('editor-size', path=path) == size
        assert not any((root / path).exists() for path in ('saved.txt', 'draft.txt'))

    preview = prepare()
    assert destination in preview and 'saved.txt' in preview, preview
    assert_source(ORIGINAL, destination, 0)
    if foreign_room:
        before = driver.act('source-text-state', path=ORIGINAL)
        message = driver.act('change-resource-save')
        assert 'Publication ended before any source write' in message, message
        assert_source(ORIGINAL, destination, 0)
        assert driver.act('source-text-state', path=ORIGINAL) == before
        driver.act('change-close')
        prepare()
        assert_source(ORIGINAL, destination, 0)
        print('PASS browser: real foreign room Move refusal retained the original editor, saved source and undo depth; fresh Preview available')
        return

    (root / DESTINATION).write_text('independent destination\n')
    refusal = driver.act('change-resource-save')
    assert 'Publish' in refusal or 'publication' in refusal.lower(), refusal
    assert (root / DESTINATION).read_text() == 'independent destination\n'
    assert (root / ORIGINAL).read_text() == PUBLISHED_SOURCE
    assert entry(origin) == first
    (root / DESTINATION).unlink()
    driver.act('change-close')

    prepare()
    result = driver.act('change-resource-save')
    assert 'Saved source resources' in result, result
    assert_source(DESTINATION, ORIGINAL, 1)
    for direction, path, absent, generation in [('Undo', ORIGINAL, DESTINATION, 2), ('Redo', DESTINATION, ORIGINAL, 3)]:
        driver.act('change-close')
        driver.act('change-open', command=direction + ' workspace change')
        driver.act('change-resource-ready')
        assert entry(origin)['generation'] == first['generation'] + generation - 1
        result = driver.act('change-resource-save')
        assert 'Saved source resources' in result, result
        assert_source(path, absent, generation)
    driver.act('change-close')
    driver.act('construction-open')
    reading = driver.act('construction-read', entry='')
    assert 'Captured entry: ' + DESTINATION in reading and 'saved.txt' in reading and 'draft.txt' in reading, reading
    connected = driver.act('behavior-ready')
    assert DESTINATION in connected, connected
    driver.act('construction-close')
    assert_source(DESTINATION, ORIGINAL, 3)
    driver.act('undo', path=DESTINATION)
    assert driver.act('construction-source', path=DESTINATION) == PUBLISHED_SOURCE
    driver.act('redo', path=DESTINATION)
    assert driver.act('construction-source', path=DESTINATION) == DRAFT_SOURCE
    print('PASS browser: main construction Move, stale destination refusal, reviewed Undo and Redo, unchanged saved baseline and retained editor Undo')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--host', choices=['browser', 'apple', 'android'], required=True)
    parser.add_argument('--foreign-room', action='store_true')
    parser.add_argument('--engine', default=os.environ.get('HICKORY_SIMPLE_TEST_ENGINE', str(ROOT / 'target/source-edit-focus/debug/examples/engine_client')))
    args = parser.parse_args()
    fixture = module('scripts/test-simple-browser.py', 'simple_fixture')
    native = module('scripts/test-native-templates.py', 'native_fixture')
    owned = module('scripts/test-agent-proposal-persistence.py', 'owned_engine')
    with tempfile.TemporaryDirectory(prefix='entry-move-host-', dir=ROOT / 'target') as temporary:
        directory = Path(temporary)
        root = directory / 'workspace'
        root.mkdir()
        (root / '.git').mkdir()
        (root / '.hickory-entry-move-fixture').write_text('owned-entry-move-v1\n')
        (root / ORIGINAL).write_text(PUBLISHED_SOURCE)
        (root / 'part.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:copy id="part">source</h:copy></h:doc>\n')
        destination = 'live destination.md' if args.foreign_room else DESTINATION
        foreign = '# Independent room\n'
        if args.foreign_room:
            (root / destination).write_text(foreign)
            os.environ['HICKORY_ENTRY_MOVE_FOREIGN_ROOM'] = destination
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            port = reserve.getsockname()[1]
        ui = f'http://127.0.0.1:{port}'
        env = {**os.environ, 'HICKORY_STATE_DIR': str(directory / 'state')}
        with ExitStack() as cleanup:
            engine = fixture.start([args.engine, str(root), 'window', ui], env=env)
            cleanup.callback(owned.stop_engine, fixture, engine, directory / 'state')
            origin = fixture.origin(engine)
            if args.foreign_room:
                with urllib.request.urlopen(origin + '/api/files', timeout=20) as response:
                    tree = json.load(response)['tree']
                identity = next(node['doc_id'] for node in tree if node['path'] == destination)
                room = subprocess.Popen(['node', str(ROOT / 'scripts/control-entry-room.mjs'), origin, identity, foreign],
                                        cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                def close_room():
                    if room.poll() is None:
                        room.stdin.write('close\n')
                        room.stdin.flush()
                    try:
                        assert room.wait(timeout=10) == 0, room.stderr.read()
                    except subprocess.TimeoutExpired:
                        room.kill()
                        room.wait(timeout=5)
                        raise
                cleanup.callback(close_room)
                assert select.select([room.stdout], [], [], 20)[0], 'Foreign room did not synchronize'
                assert json.loads(room.stdout.readline()) == {'ready': True, 'source': foreign, 'writes_sent': 0}
                (root / destination).unlink()
            if args.host == 'browser':
                vite = fixture.start(['node', 'node_modules/vite/bin/vite.js', '--config', 'vite.simple.config.ts', '--host', '127.0.0.1', '--port', str(port), '--strictPort'], cwd=ROOT / 'apps/web', env={**env, 'HICKORY_API_ORIGIN': origin})
                cleanup.callback(fixture.stop, vite)
                driver = fixture.BrowserDriver(fixture.origin(vite) + '/simple.html')
                cleanup.callback(driver.close)
                browser_flow(driver, root, origin, destination, args.foreign_room)
            else:
                label = 'entry-move-no-effect' if args.foreign_room else 'entry-move'
                case = ('WorkspaceEntryMoveTests', 'testLiveMainMoveUndoRedo', label) if args.host == 'apple' else ('WorkspaceEntryMoveTest', 'liveMainMoveUndoRedo', label)
                native.run_host(args.host, origin, root, directory, fixture, acceptance=case)
            saved, absent = (ORIGINAL, destination) if args.foreign_room else (destination, ORIGINAL)
            assert (root / saved).read_text() == PUBLISHED_SOURCE
            assert not (root / absent).exists()


if __name__ == '__main__':
    main()
</ui:copy>

<ui:file path="scripts/test-workspace-entry-move-hosts.py"><ui:paste select="#workspace-entry-move-host-runner"/></ui:file>

## Follow the original AppKit editor through both workspace inverses

The mounted editor makes a real dirty edit and retains its undo manager. Each
menu command prepares a separate saved-source review. The witness compares the
physical destination, exact role generation, current selection and original
editor before exercising local Undo at the final address.

<ui:copy id="workspace-entry-move-apple-witness">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple

@MainActor final class WorkspaceEntryMoveTests:XCTestCase {
 #if os(macOS)
 func testLiveMainMoveUndoRedo()async throws {
  guard let address=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],
        let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else {
   throw XCTSkip("Use the owned workspace entry Move fixture.")
  }
  let foreign=ProcessInfo.processInfo.environment["HICKORY_ENTRY_MOVE_FOREIGN_ROOM"]
  let root=URL(fileURLWithPath:directory),old="project.md",new=foreign ?? "literal #main.spec"
  func disk(_ path:String)throws->String {try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8)}
  XCTAssertEqual(try disk(".hickory-entry-move-fixture"),"owned-entry-move-v1\n")
  let engine=try Engine(address:address),tree=try await engine.files(),space=Workspace()
  space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id
  let buffer=try await space.loadBuffer(XCTUnwrap(tree.tree.first{$0.path==old}))
  let saved=buffer.text,draft=saved.replacingOccurrences(of:"saved.txt",with:"draft.txt")
  space.layout.activate(old,in:"document-group")
  let original=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
  XCTAssertEqual(original.path,old)

  _=NSApplication.shared
  let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false)
  window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil)
  defer{window.orderOut(nil);for value in space.buffers.values{value.language?.close()}}
  let native=editor.text,view=try XCTUnwrap(buffer.sourceEditorState?()),manager=try XCTUnwrap(native.undoManager)
  let range=(saved as NSString).range(of:"saved.txt")
  native.setSelectedRange(range);native.insertText("draft.txt",replacementRange:range)
  XCTAssertEqual(buffer.text,draft)
  let selection=native.selectedRange(),geometry=editor.scroll.frame

  func command(_ id:String)throws {
   let selected=try XCTUnwrap(space.commands().first{$0.id==id})
   XCTAssertNil(selected.reason);selected.run()
  }
  func wait(_ ready:()->Bool)async throws {
   for _ in 0..<400 {if ready(){return};try await Task.sleep(for:.milliseconds(25))}
   throw EngineError(message:space.changes.notice+" "+space.resourceRestoration.notice)
  }
  func assertState(_ path:String,absent:String,advance:UInt32)async throws {
   let stamp=try await space.resourceRestoration.readEntry(space,current:{true}).proof.entry
   XCTAssertEqual(stamp.path,path);XCTAssertEqual(stamp.identity,original.identity);XCTAssertEqual(stamp.generation,original.generation+advance)
   XCTAssertEqual(try disk(path),saved);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(absent).path))
   XCTAssertTrue(space.currentBuffer===buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved)
   let retained=try XCTUnwrap(buffer.sourceEditorState?())
   XCTAssertEqual(retained.identity,view.identity);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
  }
  try command("workspace.moveEntry")
  try await wait{space.changes.entrySelection?.entryPath==old}
  space.changes.name=new
  await space.changes.prepare(space)
  let move=try XCTUnwrap(space.resourceRestoration.review,space.resourceRestoration.notice)
  XCTAssertEqual(move.entryIntent?.after_entry.path,new)
  try await assertState(old,absent:new,advance:0)
  await space.resourceRestoration.publish(space)
  XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
  if foreign != nil {
   XCTAssertTrue(space.resourceRestoration.notice.contains("Publication ended before any source write"),space.resourceRestoration.notice)
   try await assertState(old,absent:new,advance:0)
   XCTAssertTrue(native.undoManager===manager);XCTAssertTrue(manager.canUndo)
   manager.undo();XCTAssertEqual(buffer.text,saved)
   manager.redo();XCTAssertEqual(buffer.text,draft)
   try command("workspace.moveEntry")
   try await wait{space.changes.entrySelection?.entryPath==old}
   space.changes.name=new;await space.changes.prepare(space)
   XCTAssertNotNil(space.resourceRestoration.review,space.resourceRestoration.notice)
   return
  }
  try await assertState(new,absent:old,advance:1)

  for (id,path,absent,advance) in [("workspace.undo",old,new,UInt32(2)),("workspace.redo",new,old,UInt32(3))] {
   try command(id)
   try await wait{space.resourceRestoration.review != nil}
   XCTAssertEqual(space.resourceRestoration.review?.entryIntent?.after_entry.path,path)
   await space.resourceRestoration.publish(space)
   XCTAssertFalse(space.resourceRestoration.hasPending,space.resourceRestoration.notice)
   try await assertState(path,absent:absent,advance:advance)
  }
  XCTAssertTrue(native.undoManager===manager);XCTAssertTrue(manager.canUndo)
  manager.undo();XCTAssertEqual(buffer.text,saved)
  manager.redo();XCTAssertEqual(buffer.text,draft)
  XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("saved.txt").path))
  XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("draft.txt").path))
 }
 #endif
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/WorkspaceEntryMoveTests.swift"><ui:paste select="#workspace-entry-move-apple-witness"/></ui:file>

## Retain the Android source state through the same reviewed history

The production command adapter and resource review controller carry the
transition. The witness retains one Compose text state, follows the current
source and verifies the separate saved baseline at each address. Local Undo
and Redo still belong to that same text state.

<ui:copy id="workspace-entry-move-android-witness">
package com.hickorydocs.simple

import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Assume.assumeNotNull
import org.junit.Test
import java.io.File

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class WorkspaceEntryMoveTest {
 @Test(timeout=90000) fun liveMainMoveUndoRedo()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN")
  assumeNotNull(origin)
  val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the owned entry Move fixture")).canonicalFile
  assertEquals("owned-entry-move-v1\n",root.resolve(".hickory-entry-move-fixture").readText())
  val old="project.md"
  val foreign=System.getenv("HICKORY_ENTRY_MOVE_FOREIGN_ROOM")
  val new=foreign ?: "literal #main.spec"
  val engine=Engine(origin!!)
  val (_,tree)=engine.files()
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  val space=Workspace(background)
  space.engine=engine;space.rootFiles=tree
  try {
   val file=tree.first{it.path==old}
   val buffer=space.loadBuffer(file)
   space.show(old)
   val saved=buffer.text
   val draft=saved.replace("saved.txt","draft.txt")
   val state=buffer.state
   val at=saved.indexOf("saved.txt")
   state.edit{replace(at,at+"saved.txt".length,"draft.txt");selection=TextRange(at+"draft.txt".length)}
   buffer.advanceSourceRevision()
   assertEquals(draft,buffer.text)
   val selection=state.selection
   val scroll=buffer.sourceScroll
   val original=space.resourceChanges.readEntry(space){true}.proof.entry
   assertEquals(old,original.path)

   val raw=javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()}
   val descriptors=commandDescriptors(raw)
   fun command(id:String){
    val selected=space.commands(descriptors).single{it.id==id}
    assertNull(selected.reason)
    selected.run()
   }
   suspend fun wait(ready:()->Boolean){
    repeat(400){if(ready())return;delay(25)}
    error(space.changes.notice+" "+space.resourceChanges.notice)
   }
   suspend fun assertState(path:String,absent:String,advance:Long){
    val stamp=space.resourceChanges.readEntry(space){true}.proof.entry
    assertEquals(path,stamp.path)
    assertEquals(original.identity,stamp.identity)
    assertEquals(original.generation+advance,stamp.generation)
    assertEquals(saved,root.resolve(path).readText())
    assertFalse(root.resolve(absent).exists())
    assertSame(buffer,space.currentBuffer)
    assertSame(state,buffer.state)
    assertSame(scroll,buffer.sourceScroll)
    assertEquals(selection,state.selection)
    assertEquals(saved,buffer.base)
    assertEquals(draft,buffer.text)
   }
   suspend fun publish(){
    space.resourceChanges.apply(space)
    assertFalse(space.resourceChanges.notice,space.resourceChanges.hasPending)
   }
   command("workspace.moveEntry")
   wait{space.changes.entrySelection?.entryPath==old}
   space.changes.updateName(new)
   space.changes.prepare(space)
   assertEquals(space.resourceChanges.notice,new,space.resourceChanges.review?.entryIntent?.after?.path)
   assertState(old,new,0)
   publish()
   if(foreign!=null){
    assertTrue(space.resourceChanges.notice,space.resourceChanges.notice.contains("Publication ended before any source write"))
    assertState(old,new,0)
    assertTrue(state.undoState.canUndo)
    state.undoState.undo();assertEquals(saved,buffer.text)
    state.undoState.redo();assertEquals(draft,buffer.text)
    command("workspace.moveEntry");wait{space.changes.entrySelection?.entryPath==old}
    space.changes.updateName(new);space.changes.prepare(space)
    assertNotNull(space.resourceChanges.notice,space.resourceChanges.review)
    return@runBlocking
   }
   assertState(new,old,1)

   command("workspace.undo")
   wait{space.resourceChanges.review!=null}
   assertEquals(old,space.resourceChanges.review?.entryIntent?.after?.path)
   publish()
   assertState(old,new,2)
   command("workspace.redo")
   wait{space.resourceChanges.review!=null}
   assertEquals(new,space.resourceChanges.review?.entryIntent?.after?.path)
   publish()
   assertState(new,old,3)

   assertTrue(state.undoState.canUndo)
   state.undoState.undo();assertEquals(saved,buffer.text)
   state.undoState.redo();assertEquals(draft,buffer.text)
   assertFalse(root.resolve("saved.txt").exists())
   assertFalse(root.resolve("draft.txt").exists())
  }finally{background.cancel()}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/WorkspaceEntryMoveTest.kt"><ui:paste select="#workspace-entry-move-android-witness"/></ui:file>

</ui:doc>
