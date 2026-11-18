<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Complete an interrupted Move in the retained editor

For maintainers checking the Source recovery flow against the real workspace
engine. A producer checkpoint comes from the workspace fixture before the new
engine and host open its sealed evidence. The test never edits the private
journal or replaces a publication response.

## Start from the producer's observed partial state

Prepare the owned fixture and restart the service. First inspect both saved
addresses and the current interrupted record. Preview exposes the same source
and destination with separate saved and draft sides. Declining the preview has
no file effect. The live editor must retain its identity, selection, geometry and
Undo history after explicit completion. A cold browser replay also checks that
the sealed draft can be recovered without an open original editor.
A separate rendered case replaces the copied destination with equal bytes
after Preview. The review must refuse without writing either path or changing
the original live editor. A new reading must explain why completion is no
longer offered.

<ui:copy id="workspace-entry-interrupted-host-runner">
#!/usr/bin/env python3
"""Exercise reviewed interruption completion with a real engine and host."""
import argparse
from contextlib import ExitStack
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]


def load_script(name):
    path = ROOT / 'scripts' / name
    spec = importlib.util.spec_from_file_location(path.stem, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(origin, route):
    with urllib.request.urlopen(origin + '/api/' + route, timeout=20) as response:
        return json.load(response)


def authored_files(root):
    files = set()
    for path in root.rglob('*'):
        relative = path.relative_to(root)
        if path.is_file() and relative.parts[0] not in {'.hick-cache', '.git'}:
            files.add(relative.as_posix())
    return files


def replace_with_equal_bytes(root, path):
    destination = root / path
    inode = destination.stat().st_ino
    replacement = root / 'fixture-equal-byte-replacement.tmp'
    replacement.write_bytes(destination.read_bytes())
    replacement.replace(destination)
    assert destination.stat().st_ino != inode


def browser_completion(driver, root, origin, material, marker, cold, replaced_after_review):
    old = material['change']['before']['path']
    new = material['change']['after']['path']
    original = material['current_entry']
    saved = (root / material['effective_entry']).read_text()
    saved_scope = {path: (root / path).read_bytes() if (root / path).is_file() else None
                   for path in (old, new)}
    if cold:
        query = urllib.parse.urlencode({'id': material['request_id'], 'path': old, 'draft': 'before'})
        draft = read(origin, 'changes/resource-recovery?' + query)['draft_text']
        assert draft == marker['retained_text'] and draft != saved
    else:
        driver.act('open', path=old)
        driver.act('remember-editor', path=old)
        end = len(saved.encode('utf-16-le')) // 2
        driver.act('source-text-select', path=old, anchor=0, head=end)
        driver.act('type-source-selection', path=old, text=marker['live_text'])
        draft = driver.act('construction-source', path=old)
        assert draft == marker['live_text'] and draft != saved
        selection = driver.act('editor-position', path=old)
        geometry = driver.act('editor-size', path=old)
        captured = driver.act('source-text-state', path=old)

    def preview():
        driver.act('recovery-open')
        evidence = driver.act('recovery-interrupted')
        assert evidence['canComplete'] and old in evidence['text'] and new in evidence['text'], evidence
        notices = ' '.join(driver.act('recovery-prepare', completion=True))
        assert 'Review the saved source and retained draft' in notices, notices
        for path, expected in saved_scope.items():
            actual = (root / path).read_bytes() if (root / path).is_file() else None
            assert actual == expected, path
        assert read(origin, 'workspace/entry/interrupted')['interrupted']['material']['current_entry'] == original

    preview()
    if replaced_after_review:
        replace_with_equal_bytes(root, new)
        substituted = (root / new).stat().st_ino
        notice = ' '.join(driver.act('recovery-publish', completion=True))
        assert notice and 'Completed the reviewed Move' not in notice, notice
        current = read(origin, 'workspace/entry/interrupted')['interrupted']['material']
        assert current['current_entry'] == original and not current['can_complete'], current
        assert (root / new).stat().st_ino == substituted
        assert all((root / path).read_bytes() == expected for path, expected in saved_scope.items())
        assert driver.act('editor-is-retained', path=old, previous=old)
        assert driver.act('source-text-state', path=old) == captured
        assert driver.act('editor-size', path=old) == geometry
        assert not driver.act('recovery-interrupted')['canComplete']
        print('PASS browser refuses an equal-byte replacement after Preview and preserves both saved paths and the original draft')
        return
    if not cold:
        driver.act('construction-set-source', path=old, source=draft + '\n')
        driver.act('construction-history', redo=False)
        assert driver.act('construction-source', path=old) == draft
        refused = ' '.join(driver.act('recovery-publish', completion=True))
        assert 'Source changed after review' in refused, refused
        assert read(origin, 'workspace/entry/interrupted')['interrupted']['material']['current_entry'] == original
        for path, expected in saved_scope.items():
            actual = (root / path).read_bytes() if (root / path).is_file() else None
            assert actual == expected, path
        selection = driver.act('editor-position', path=old)
        captured = driver.act('source-text-state', path=old)
    driver.act('recovery-close')
    preview()
    notices = ' '.join(driver.act('recovery-publish', completion=True))
    assert 'Completed the reviewed Move' in notices, notices
    driver.act('recovery-close')
    assert read(origin, 'workspace/entry/interrupted')['interrupted'] is None
    stamp = read(origin, 'workspace/entry')['entry']
    assert stamp['identity'] == original['identity'] and stamp['path'] == new
    assert stamp['generation'] == material['reservation_floor'] + 1
    assert not (root / old).exists()
    assert (root / new).read_text() == saved
    if cold:
        driver.act('open', path=new)
    else:
        assert driver.act('editor-is-retained', path=new, previous=old)
        assert driver.act('editor-position', path=new) == selection
        assert driver.act('editor-size', path=new) == geometry
        assert driver.act('source-text-state', path=new) == captured
    assert driver.act('construction-source', path=new) == draft
    print('PASS browser recovery review and explicit completion of the sealed Move')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', choices=['browser', 'apple', 'android'], required=True)
    parser.add_argument('--test-binary', type=Path, required=True, help='Focused hickory-cli test executable')
    parser.add_argument('--stage', choices=['copied', 'old_deleted'], default='copied')
    parser.add_argument('--cold', action='store_true', help='Check a browser with no original editor')
    parser.add_argument('--replaced', action='store_true', help='Check an equal-byte replacement of the copied destination')
    parser.add_argument('--replaced-after-review', action='store_true', help='Replace the copied destination after Preview')
    parser.add_argument('--engine', default=os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',
                        str(ROOT / 'target/source-edit-focus/debug/examples/engine_client')))
    args = parser.parse_args()
    if args.cold and args.host != 'browser':
        parser.error('Use --cold with the browser case')
    if args.stage == 'old_deleted' and not args.cold:
        parser.error('The old_deleted fixture starts without a retained original editor. Use --cold.')
    if args.replaced and (args.host != 'browser' or args.stage != 'copied'):
        parser.error('Use --replaced with the copied browser fixture')
    if args.replaced_after_review and (args.host != 'browser' or args.stage != 'copied' or args.replaced or args.cold):
        parser.error('Use --replaced-after-review with the live copied browser fixture')
    simple = load_script('test-simple-browser.py')
    owned = load_script('test-agent-proposal-persistence.py')
    native = load_script('test-native-templates.py')
    with tempfile.TemporaryDirectory(prefix='interrupted-entry-host-', dir=ROOT / 'target') as temporary:
        folder = Path(temporary)
        root, state = folder / 'workspace', folder / 'state'
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(state), 'HICKORY_SIMPLE_TEST_ENGINE': args.engine}
        prepared = subprocess.run([sys.executable, str(ROOT / 'scripts/prepare-interrupted-entry.py'),
                        '--root', str(root), '--state', str(state), '--stage', args.stage,
                        '--test-binary', str(args.test_binary.resolve())], env=environment, cwd=ROOT,
                        capture_output=True, text=True, timeout=95)
        assert prepared.returncode == 0, (prepared.stdout, prepared.stderr)
        marker = json.loads((root / '.hick-cache/entry-completion-fixture.json').read_text())
        assert marker['kind'] == 'entry-interrupted-completion-fixture' and marker['version'] == 1
        assert Path(marker['source_root']).resolve() == root.resolve()
        initial_files = authored_files(root)
        expected_files = (initial_files - {marker['old_path']}) | {marker['destination']}
        if args.replaced:
            replace_with_equal_bytes(root, marker['destination'])
        with socket.socket() as reservation:
            reservation.bind(('127.0.0.1', 0))
            port = reservation.getsockname()[1]
        with ExitStack() as cleanup:
            engine = simple.start([args.engine, str(root), 'window', f'http://127.0.0.1:{port}'], env=environment)
            cleanup.callback(owned.stop_engine, simple, engine, state)
            origin = simple.origin(engine)
            material = read(origin, 'workspace/entry/interrupted')['interrupted']['material']
            assert material['can_complete'] is not args.replaced and material['source_files_modified'] is False, material
            assert material['request_id'] == marker['original_request_id']
            if args.host == 'browser':
                vite = simple.start(['node', 'node_modules/vite/bin/vite.js', '--config', 'vite.simple.config.ts',
                                     '--host', '127.0.0.1', '--port', str(port), '--strictPort'], cwd=ROOT / 'apps/web',
                                    env={**environment, 'HICKORY_API_ORIGIN': origin})
                cleanup.callback(simple.stop, vite)
                driver = simple.BrowserDriver(simple.origin(vite) + '/simple.html')
                cleanup.callback(driver.close)
                if args.replaced:
                    driver.act('recovery-open')
                    evidence = driver.act('recovery-interrupted')
                    assert not evidence['canComplete'] and material['notice'] in evidence['text'], evidence
                    assert read(origin, 'workspace/entry/interrupted')['interrupted']['material'] == material
                    assert authored_files(root) == initial_files
                    for path in (marker['old_path'], marker['destination']):
                        assert (root / path).read_text() == marker['saved_text']
                    print('PASS browser preserves an independently replaced destination and offers no completion')
                    return
                browser_completion(driver, root, origin, material, marker, args.cold, args.replaced_after_review)
                if args.replaced_after_review:
                    assert authored_files(root) == initial_files
                    return
            else:
                label = 'entry-interrupted'
                witness = (('InterruptedEntryTests', 'testReviewedEntryCompletion', label) if args.host == 'apple'
                           else ('InterruptedEntryTest', 'reviewedEntryCompletion', label))
                native.run_host(args.host, origin, root, folder, simple, acceptance=witness)
            assert read(origin, 'workspace/entry/interrupted')['interrupted'] is None
            assert authored_files(root) == expected_files, 'Recovery materialized files outside its authored source scope'


if __name__ == '__main__':
    main()
</ui:copy>

<ui:file path="scripts/test-interrupted-entry-hosts.py"><ui:paste select="#workspace-entry-interrupted-host-runner"/></ui:file>

## Keep the mounted AppKit editor through completion

Use the same `ResourceRestoration` controller the popup calls. The test edits
through the mounted AppKit text view, confirms the draft still owns the old
address during preview and exercises its original Undo manager after adoption.

<ui:copy id="workspace-entry-interrupted-apple-witness">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple

@MainActor final class InterruptedEntryTests: XCTestCase {
 #if os(macOS)
 func testReviewedEntryCompletion() async throws {
  let environment=ProcessInfo.processInfo.environment
  guard let origin=environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],
        let directory=environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else {throw XCTSkip("Use the owned interrupted Move fixture.")}
  let root=URL(fileURLWithPath:directory).resolvingSymlinksInPath()
  func disk(_ path:String)throws->String {try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8)}
  let bytes=try Data(contentsOf:root.appendingPathComponent(".hick-cache/entry-completion-fixture.json"))
  let marker=try XCTUnwrap(JSONSerialization.jsonObject(with:bytes) as? [String:Any])
  guard marker["version"] as? Int==1,marker["kind"] as? String=="entry-interrupted-completion-fixture",
        exactUTF8(marker["source_root"] as? String,root.path) else {throw EngineError(message:"Unowned interruption fixture.")}
  let old=try XCTUnwrap(marker["old_path"] as? String),destination=try XCTUnwrap(marker["destination"] as? String)
  let draft=try XCTUnwrap(marker["live_text"] as? String),saved=try XCTUnwrap(marker["saved_text"] as? String)
  let engine=try Engine(address:origin),tree=try await engine.files(),workspace=Workspace()
  guard exactUTF8(tree.root_path,root.path) else {throw EngineError(message:"Engine serves a different fixture root.")}
  workspace.engine=engine;workspace.rootFiles=tree.tree;workspace.sourceRootAddress=tree.root_path;workspace.sourceWorkspaceID=tree.workspace_id
  let model=workspace.resourceRestoration
  let result=try await model.readInterruptedEntry(workspace)
  let reading=try XCTUnwrap(result)
  guard reading.material.can_complete,exactUTF8(reading.source,old),exactUTF8(reading.destination,destination),
        reading.material.request_id==(marker["original_request_id"] as? String) else {
   throw EngineError(message:"Engine returned a different interruption.")
  }
  let buffer=try await workspace.loadBuffer(XCTUnwrap(tree.tree.first{$0.path==old}))
  workspace.layout.activate(old,in:"document-group")
  _=NSApplication.shared
  let editor=MacEditor(buffer)
  let window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false)
  window.contentView=editor.scroll;window.makeKeyAndOrderFront(nil)
  defer{window.orderOut(nil);for source in workspace.buffers.values {source.language?.close()}}
  let range=NSRange(location:0,length:buffer.text.utf16.count)
  editor.text.setSelectedRange(range);editor.text.insertText(draft,replacementRange:range)
  let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
  let undo=try XCTUnwrap(editor.text.undoManager)
  let selection=editor.text.selectedRange(),geometry=editor.scroll.frame
  XCTAssertEqual(buffer.text,draft);XCTAssertTrue(undo.canUndo)

  workspace.showRecovery=true
  await model.prepareEntryCompletion(workspace,reading:reading,current:{workspace.showRecovery})
  let review=try XCTUnwrap(model.review,model.notice)
  XCTAssertEqual(review.entryIntent?.after_entry.path,destination)
  XCTAssertEqual(review.result.restoration.saved_before[old]?.text,saved)
  XCTAssertEqual(review.result.restoration.saved_after[destination]?.text,saved)
  XCTAssertEqual(try disk(old),saved);XCTAssertEqual(try disk(destination),saved)
  XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft)
  await model.publish(workspace)
  XCTAssertFalse(model.hasPending,model.notice)
  let stamp=try await model.readEntry(workspace,current:{true}).proof.entry
  XCTAssertEqual(stamp.path,destination);XCTAssertEqual(stamp.identity,reading.material.current_entry.identity)
  XCTAssertEqual(stamp.generation,reading.material.reservation_floor+1)
  XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent(old).path))
  XCTAssertEqual(try disk(destination),saved);XCTAssertEqual(buffer.base,saved)
  XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft)
  XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity)
  XCTAssertEqual(editor.text.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
  XCTAssertTrue(editor.text.undoManager === undo)
  undo.undo();XCTAssertEqual(buffer.text,saved)
  undo.redo();XCTAssertEqual(buffer.text,draft)
 }
 #endif
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/InterruptedEntryTests.swift"><ui:paste select="#workspace-entry-interrupted-apple-witness"/></ui:file>

## Keep the Android controller's existing text state

The same explicit review and publication verify its source ownership and saved
bytes. The witness drives the production controller and `TextFieldState`.
Rendered Android, physical gestures, IME and assistive technology have their own
acceptance requirements.

<ui:copy id="workspace-entry-interrupted-android-witness">
package com.hickorydocs.simple

import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.*
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeNotNull
import org.junit.Test
import java.io.File

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
class InterruptedEntryTest {
 @Test(timeout=90000) fun reviewedEntryCompletion()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN")
  assumeNotNull(origin)
  val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use the owned interrupted Move fixture.")).canonicalFile
  val marker=JSONObject(root.resolve(".hick-cache/entry-completion-fixture.json").readText())
  check(marker.getInt("version")==1&&marker.getString("kind")=="entry-interrupted-completion-fixture"&&
   marker.getString("source_root")==root.path){"Unowned interruption fixture."}
  val old=marker.getString("old_path")
  val destination=marker.getString("destination")
  val draft=marker.getString("live_text")
  val saved=marker.getString("saved_text")
  val engine=Engine(origin!!)
  val (_,tree)=engine.files()
  check(engine.sourceRootAddress==root.path){"Engine serves a different fixture root."}
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined)
  val workspace=Workspace(background)
  workspace.engine=engine;workspace.rootFiles=tree
  try {
   val model=workspace.resourceChanges
   val reading=checkNotNull(model.readInterruptedEntry(workspace))
   check(reading.canComplete&&reading.source==old&&reading.destination==destination&&
    reading.request==marker.getString("original_request_id")){"Unexpected interruption."}
   val buffer=workspace.loadBuffer(tree.first{it.path==old})
   workspace.show(old)
   val state=buffer.state
   state.edit{replace(0,length,draft);selection=TextRange(draft.length)}
   buffer.advanceSourceRevision()
   val selection=state.selection
   val scroll=buffer.sourceScroll
   assertEquals(draft,buffer.text);assertTrue(state.undoState.canUndo)

   workspace.showRecovery=true
   model.prepareEntryCompletion(workspace,reading){workspace.showRecovery}
   val review=checkNotNull(model.review){model.notice}
   assertEquals(destination,review.entryIntent?.after?.path)
   assertEquals(saved,review.saved[destination]?.text)
   assertEquals(saved,root.resolve(old).readText());assertEquals(saved,root.resolve(destination).readText())
   assertSame(buffer,workspace.currentBuffer);assertEquals(draft,buffer.text)
   model.apply(workspace,review)
   assertFalse(model.notice,model.hasPending)
   val stamp=model.readEntry(workspace){true}.proof.entry
   assertEquals(destination,stamp.path);assertEquals(reading.currentEntry.identity,stamp.identity)
   assertEquals(reading.reservationFloor+1,stamp.generation)
   assertFalse(root.resolve(old).exists());assertEquals(saved,root.resolve(destination).readText())
   assertEquals(saved,buffer.base);assertEquals(draft,buffer.text)
   assertSame(buffer,workspace.currentBuffer);assertSame(state,buffer.state);assertSame(scroll,buffer.sourceScroll)
   assertEquals(selection,state.selection)
   state.undoState.undo();assertEquals(saved,buffer.text)
   state.undoState.redo();assertEquals(draft,buffer.text)
  } finally {
   workspace.buffers.values.forEach{it.language?.close()}
   background.cancel()
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/InterruptedEntryTest.kt"><ui:paste select="#workspace-entry-interrupted-android-witness"/></ui:file>
</ui:doc>
