<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Verify a mounted UIKit editor against a real workspace

For maintainers distinguishing an iOS product build or simulator launch from feature acceptance. This focused hosted XCTest mounts the same WorkspaceView and UITextView as the application. It verifies the actual source widget subclass and initialized owned history, invokes UIKit’s native Paste item-provider path, reviews real whitespace Cleanup and exercises coordinated Undo/Redo, and follows production saved-source Rust Debug through the installed engine, Cargo and LLDB. A passing model or successful launch is insufficient.

## Interact with the production UIKit editor

The fixture checks its supplied origin and source marker before mounting the real
workspace in a UIWindow. A test-owned NSString item provider enters UIKit's
Paste path. Undo and Redo must restore the exact replacement text and selection
through the editor's UndoManager. The whitespace append is a second native
transaction so guarded Cleanup review and workspace Undo/Redo exercise a separate
history path. Capture geometry after the review and keyboard have finished
dismissing and UIKit reports a stable layout. Require a saved-source Rust
breakpoint at the authored line, continuation and a retained process exit.
This witness covers automated UIKit and controller interaction. Physical input,
device IME and assistive-technology acceptance are separate.

<ui:copy id="ios-mounted-parity-test">
import XCTest
import UIKit
import SwiftUI
@testable import HickorySimpleMobile

@MainActor private final class UIKitFactoryOwnershipProbe:UITextView {
    private let ownedHistory=UndoManager()
    override var undoManager:UndoManager? {ownedHistory}
}

@MainActor private final class HostedKeyboardObservation {
    var visible=false
    var hidden=0
    var frame=CGRect.null
    var event="none"
    func record(_ note:Notification){
        event=note.name.rawValue
        if let value=note.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue {frame=value.cgRectValue}
        if note.name==UIResponder.keyboardDidHideNotification {visible=false;hidden+=1}
        else if note.name==UIResponder.keyboardWillShowNotification || note.name==UIResponder.keyboardDidShowNotification {visible=true}
    }
}

/// Simulator UIKit integration: actual text input/history plus the production Debug route.
@MainActor final class UIKitParityAcceptanceTests: XCTestCase {
    func testMountedUIKitEditingAndSelectedRustDebug() async throws {
        let origin = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PARITY_ORIGIN"], "Use the maintained owned simulator fixture")
        func mark(_ label:String){NSLog("HickoryUIKitParity %@",label)}
        mark("start")
        let keyboard=HostedKeyboardObservation()
        let keyboardEvents=[UIResponder.keyboardWillShowNotification,UIResponder.keyboardDidShowNotification,UIResponder.keyboardWillHideNotification,UIResponder.keyboardDidHideNotification,UIResponder.keyboardDidChangeFrameNotification]
        let keyboardTokens=keyboardEvents.map {name in NotificationCenter.default.addObserver(forName:name,object:nil,queue:.main){[weak keyboard] note in MainActor.assumeIsolated{keyboard?.record(note)}}}
        defer {for token in keyboardTokens{NotificationCenter.default.removeObserver(token)}}
        let workspace = Workspace()
        workspace.address = origin
        let connected = await workspace.connect()
        XCTAssertTrue(connected, workspace.status)
        mark("connected")
        let factoryProbe=UIKitFactoryOwnershipProbe(usingTextLayoutManager:false)
        mark("factory-probe="+String(reflecting:type(of:factoryProbe))+" owned-undo=\(factoryProbe.undoManager != nil)")
        func find(_ path: String, _ nodes: [FileNode]) -> FileNode? {
            for node in nodes {
                if node.path == path { return node }
                if let found = find(path, node.children ?? []) { return found }
            }
            return nil
        }
        let file = try XCTUnwrap(find("src/lib.rs", workspace.rootFiles))
        let buffer = try await workspace.loadBuffer(file)
        let source = buffer.text
        XCTAssertTrue(source.contains("owned-ios-parity-source"), "Refuse unrelated workspaces")
        let originalHash = buffer.hash
        workspace.layout.activateSource(file.path, in: "document-group")
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let previous = scene.windows.first { $0.isKeyWindow }
        let window = UIWindow(windowScene: scene)
        window.frame = scene.coordinateSpace.bounds
        window.windowLevel=UIWindow.Level(rawValue:UIWindow.Level.normal.rawValue+1)
        let host = UIHostingController(rootView: WorkspaceView(workspace: workspace))
        window.rootViewController = host
        window.makeKeyAndVisible()
        let debug = try XCTUnwrap(workspace.debugger)
        defer {
            if debug.active { debug.stop() }
            debug.close()
            workspace.sourceTestIndex?.stop()
            for value in workspace.buffers.values { value.language?.close() }
            window.isHidden = true
            window.rootViewController = nil
            previous?.makeKey()
        }
        func editor(_ view: UIView) -> UITextView? {
            if let text = view as? UITextView { return text }
            for child in view.subviews { if let found = editor(child) { return found } }
            return nil
        }
        func wait(_ label: String, attempts:Int = 6000, _ ready: () -> Bool) async throws {
            for _ in 0..<attempts {
                host.view.layoutIfNeeded()
                if ready() { return }
                try await Task.sleep(for: .milliseconds(20))
            }
            XCTFail(label + ": " + debug.status + " · " + debug.message + " · modelUTF16=\(buffer.text.utf16.count), dirty=\(buffer.dirty)")
            throw EngineError(message: label)
        }
        try await wait("UIKit editor did not mount") { editor(host.view) != nil && buffer.sourceEditorState?()?.attached == true }
        let text = try XCTUnwrap(editor(host.view))
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        XCTAssertEqual(text.text, source)
        let runtimeType=String(reflecting:type(of:text))
        mark("editor-type="+runtimeType+" owned-undo=\(text.undoManager != nil)")
        XCTAssertTrue(runtimeType.hasSuffix(".SimpleTextView"),"UIKit must retain the source subclass and its native input/history behavior")
        window.makeKeyAndVisible()
        XCTAssertTrue(window.isKeyWindow)
        mark("mounted")
        XCTAssertTrue(text.becomeFirstResponder())
        let sourceText = source as NSString
        let manager = try XCTUnwrap(text.undoManager)
        let pasteTarget:UIPasteConfigurationSupporting=text
        XCTAssertTrue(text.responds(to:#selector(UIPasteConfigurationSupporting.paste(itemProviders:))))
        func paste(_ value: String) {
            pasteTarget.paste?(itemProviders: [NSItemProvider(object: value as NSString)])
        }

        let replacementRange = sourceText.range(of: "owned-ios-parity-source")
        let replacement = "UIKit 🧪"
        let replacedSource = sourceText.replacingCharacters(in: replacementRange, with: replacement)
        let replacementCaret = NSRange(location: replacementRange.location + replacement.utf16.count, length: 0)
        text.selectedRange = replacementRange
        paste(replacement)
        try await wait("UIKit selection Paste did not replace the selected source", attempts: 500) {
            buffer.text == replacedSource && text.text == replacedSource && text.selectedRange == replacementCaret
        }
        XCTAssertTrue(manager.canUndo)
        XCTAssertEqual(manager.undoActionName, "Paste")
        manager.undo()
        try await wait("UIKit selection Paste Undo did not restore the original selection", attempts: 500) {
            buffer.text == source && text.text == source && text.selectedRange == replacementRange
        }
        XCTAssertTrue(manager.canRedo)
        manager.redo()
        try await wait("UIKit selection Paste Redo did not restore the replacement", attempts: 500) {
            buffer.text == replacedSource && text.text == replacedSource && text.selectedRange == replacementCaret
        }
        manager.undo()
        try await wait("UIKit selection Paste Undo did not restore saved text", attempts: 500) {
            buffer.text == source && text.text == source
        }

        let secondRange = sourceText.range(of: "selected_value")
        let ranges = [replacementRange, secondRange]
        XCTAssertTrue(buffer.selectMultiple(ranges, source: source))
        let multipleSource = NSMutableString(string: source)
        for range in ranges.reversed() { multipleSource.replaceCharacters(in: range, with: replacement) }
        let carets = [
            replacementCaret,
            NSRange(location: secondRange.location + replacement.utf16.count * 2 - replacementRange.length, length: 0)
        ]
        paste(replacement)
        try await wait("UIKit Paste did not edit both carets", attempts: 500) {
            buffer.text == multipleSource as String && text.text == multipleSource as String && buffer.multiRanges == carets
        }
        manager.undo()
        try await wait("UIKit multi-caret Paste Undo did not restore both selections", attempts: 500) {
            buffer.text == source && text.text == source && text.selectedRange == replacementRange && buffer.multiRanges == ranges
        }
        manager.redo()
        try await wait("UIKit multi-caret Paste Redo did not restore both carets", attempts: 500) {
            buffer.text == multipleSource as String && text.text == multipleSource as String && buffer.multiRanges == carets
        }
        manager.undo()
        try await wait("UIKit multi-caret Paste Undo did not restore saved text", attempts: 500) {
            buffer.text == source && text.text == source
        }

        let appendCaret = NSRange(location: sourceText.length, length: 0)
        text.selectedRange = appendCaret
        let insertion = "   \t"
        paste(insertion)
        try await wait("UIKit input did not reach the original source model",attempts:500) { buffer.text == source + insertion && buffer.dirty }
        XCTAssertEqual(text.text, source + insertion)
        XCTAssertTrue(manager.canUndo)
        manager.undo()
        try await wait("UIKit Paste Undo did not restore source", attempts: 500) {
            buffer.text == source && text.text == source && text.selectedRange == appendCaret
        }
        XCTAssertTrue(manager.canRedo)
        manager.redo()
        try await wait("UIKit Paste Redo did not restore input", attempts: 500) {
            buffer.text == source + insertion && text.text == source + insertion &&
            text.selectedRange.location == sourceText.length + insertion.utf16.count
        }
        mark("native-paste-history-passed")
        workspace.changes.open("cleanup",workspace:workspace)
        await workspace.changes.prepare(workspace)
        let plan=try XCTUnwrap(workspace.changes.plan,workspace.changes.notice)
        XCTAssertEqual(plan.patches.count,1)
        XCTAssertEqual(plan.patches.first?.before,source+insertion)
        XCTAssertEqual(plan.patches.first?.after,source)
        XCTAssertEqual(buffer.text,source+insertion,"Review must not apply or save source")
        try await wait("Cleanup review did not finish presenting",attempts:1500){host.presentedViewController?.viewIfLoaded?.window != nil && host.presentedViewController?.isBeingPresented == false}
        mark("cleanup-reviewed")
        workspace.changes.apply(workspace)
        try await wait("Guarded cleanup did not reach UIKit",attempts:500) { buffer.text==source && text.text==source && !buffer.dirty }
        workspace.changes.open("undo",workspace:workspace)
        try await wait("Coordinated cleanup Undo did not restore dirty input",attempts:500) { buffer.text==source+insertion && text.text==source+insertion && buffer.dirty }
        workspace.changes.open("redo",workspace:workspace)
        try await wait("Coordinated cleanup Redo did not restore source",attempts:500) { buffer.text==source && text.text==source && !buffer.dirty }
        workspace.changes.close(workspace)
        mark("coordinated-cleanup-history-passed")
        try await wait("Cleanup review did not finish dismissing",attempts:1500){host.presentedViewController == nil && host.transitionCoordinator == nil}
        let expectedHide=keyboard.hidden+(keyboard.visible ? 1:0)
        window.endEditing(true)
        text.resignFirstResponder()
        try await wait("UIKit keyboard did not finish hiding",attempts:1000){!text.isFirstResponder && !keyboard.visible && keyboard.hidden>=expectedHide}
        mark("keyboard-hidden event=\(keyboard.event) frame=\(String(describing: keyboard.frame))")
        text.selectedRange = NSRange(location: 0, length: 0)
        text.setContentOffset(.zero, animated: false)
        var stableFrame=CGRect.null
        var stableSafe=UIEdgeInsets.zero
        var stableHostSafe=UIEdgeInsets.zero
        var stableSamples=0
        for _ in 0..<250 {
            if text.isFirstResponder {window.endEditing(true);text.resignFirstResponder();stableSamples=0}
            host.view.layoutIfNeeded();window.layoutIfNeeded()
            let candidate=text.convert(text.bounds,to:window)
            if candidate==stableFrame && window.safeAreaInsets==stableSafe && host.view.safeAreaInsets==stableHostSafe && !text.isFirstResponder && !keyboard.visible && host.transitionCoordinator == nil {stableSamples+=1}else{stableSamples=0;stableFrame=candidate;stableSafe=window.safeAreaInsets;stableHostSafe=host.view.safeAreaInsets}
            if stableSamples>=20 {break}
            try await Task.sleep(for:.milliseconds(20))
        }
        mark("layout-stability samples=\(stableSamples) frame=\(String(describing: stableFrame)) safe=\(String(describing: window.safeAreaInsets)) hostSafe=\(String(describing: host.view.safeAreaInsets)) textSafe=\(String(describing: text.safeAreaInsets)) first=\(text.isFirstResponder) keyboard=\(keyboard.visible) event=\(keyboard.event)")
        XCTAssertGreaterThanOrEqual(stableSamples,20,"Capture geometry only after keyboard dismissal and stable UIKit layout")
        let selection = text.selectedRange
        let offset = text.contentOffset
        let geometry = text.convert(text.bounds, to: window)
        mark("geometry-before frame=\(String(describing: geometry)) bounds=\(String(describing: text.bounds)) rawFrame=\(String(describing: text.frame)) safe=\(String(describing: window.safeAreaInsets)) hostSafe=\(String(describing: host.view.safeAreaInsets)) textSafe=\(String(describing: text.safeAreaInsets)) scale=\(window.screen.scale) first=\(text.isFirstResponder) keyboard=\(keyboard.visible) event=\(keyboard.event)")
        let canUndo = manager.canUndo
        let canRedo = manager.canRedo
        let line = try XCTUnwrap(source.components(separatedBy: "\n").firstIndex { $0.contains("owned-ios-parity-stop") })
        debug.observeRequestForTest = { request in
            if let op = request["op"] as? String { mark("debug-request \(op)") }
        }
        debug.observeEventForTest = { response in
            if let event = response["event"] as? String { mark("debug-event \(event)") }
        }
        mark("debug-start")
        debug.toggle(buffer, line: line)
        await workspace.testSources.refresh(workspace)
        let facts = try XCTUnwrap(workspace.testSources.observation?.1)
        let candidate = try XCTUnwrap(facts.candidates.first { $0.name == "test_selected" })
        XCTAssertEqual(candidate.language, "rust")
        await workspace.testSources.debug(candidate, .nameFilter, workspace: workspace)
        try await wait("Selected Rust test did not pause") { debug.status == "paused" || debug.status == "failed" }
        let stoppedLine = try XCTUnwrap(debug.status == "paused" ? debug.pausedLine : nil, debug.message)
        XCTAssertEqual(stoppedLine, line)
        mark("debug-paused")
        debug.toggle(buffer, line: line)
        debug.step("continue")
        try await wait("Selected Rust test did not finish") { debug.status == "finished" || debug.status == "failed" }
        XCTAssertEqual(debug.status, "finished", debug.message)
        XCTAssertEqual(buffer.text, source)
        XCTAssertEqual(buffer.hash, originalHash)
        XCTAssertFalse(buffer.dirty)
        XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity, identity)
        XCTAssertTrue(editor(host.view) === text)
        XCTAssertEqual(text.selectedRange, selection)
        XCTAssertEqual(text.contentOffset, offset)
        let finalGeometry=text.convert(text.bounds,to:window)
        let scale=window.screen.scale
        func pixels(_ frame:CGRect)->CGRect{CGRect(x:(frame.origin.x*scale).rounded(),y:(frame.origin.y*scale).rounded(),width:(frame.width*scale).rounded(),height:(frame.height*scale).rounded())}
        mark("geometry-after frame=\(String(describing: finalGeometry)) beforePixels=\(String(describing: pixels(geometry))) afterPixels=\(String(describing: pixels(finalGeometry))) bounds=\(String(describing: text.bounds)) rawFrame=\(String(describing: text.frame)) safe=\(String(describing: window.safeAreaInsets)) hostSafe=\(String(describing: host.view.safeAreaInsets)) textSafe=\(String(describing: text.safeAreaInsets)) first=\(text.isFirstResponder) keyboard=\(keyboard.visible) event=\(keyboard.event) exit=\(String(describing:workspace.testSources.debugRuns.first?.exit))")
        XCTAssertEqual(finalGeometry, geometry)
        XCTAssertEqual(manager.canUndo, canUndo)
        XCTAssertEqual(manager.canRedo, canRedo)
        let saved = try await workspace.engine!.file(file)
        XCTAssertEqual(saved.content, source)
        let listing = try await workspace.engine!.files()
        XCTAssertNil(find("wrong-ios-sibling-ran", listing.tree))
        XCTAssertEqual(workspace.testSources.debugRuns.count, 1)
        XCTAssertEqual(workspace.testSources.debugRuns.first?.exit,0,"A finished debugger must retain the actual successful process exit")
        mark("completed")
    }
}
</ui:copy>
<ui:file path="apps/apple-simple/IOSAcceptance/UIKitParityAcceptanceTests.swift"><ui:paste select="#ios-mounted-parity-test"/></ui:file>

## Run only on a deliberately selected simulator

Coordinate the disk/link window before building or booting. The runner defaults to an already booted installed simulator and an existing hosted test build. `--build` explicitly builds the one test scheme using the existing derived products; `--boot` starts the selected simulator and restores Shutdown afterward. With explicit `--temporary-device --boot`, the selected installed device is only a seed for its exact installed runtime and device type: the runner creates a fresh uniquely named device, rejects any returned UUID from the initial complete device list (including unavailable devices), uses only a genuinely new returned UUID, then shuts down and deletes only that owned device in finally. The seed and its data stay untouched; no clone, runtime download or signing change occurs. Device-owned storage is reclaimed on deletion, while global simulator/OS caches may remain. A default2GiB free-space floor guards the measured first-boot/keyboard-cache cost; a coordinated run may explicitly choose a different floor. It is an admission check, not a guarantee against unrelated concurrent disk writes. Simulator and build caches can exceed100MiB, so this must not be presented as a small launch-only probe.

After generation and resource coordination, use `DEVELOPER_DIR=/Applications/Xcode-27.0.0.app/Contents/Developer python3 scripts/test-ios-parity.py --device <installed-UDID> --build --boot --temporary-device`. Omit those flags to reuse an already built/booted target. The runner starts an isolated workspace engine, passes its address through a disposable XCTest configuration and runs exactly one named case. It admits only one passed case with zero failures/skips from the real xcresult summary, retains the result/log digest, checks saved source and sibling exclusion, and cleans up its owned runtime. A separate lifecycle receipt records the seed, actual returned device UUID and deletion outcome, including failed attempts. Compilation, invocation and physical-input acceptance must remain unverified until actually executed.

<ui:copy id="ios-mounted-parity-runner">
"""Owned mounted UIKit parity witness. Requires an explicitly selected simulator."""
import argparse,hashlib,importlib.util,json,os,plistlib,shutil,socket,subprocess,tempfile,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TARGET='HickorySimpleMobileParityTests'
CASE='UIKitParityAcceptanceTests/testMountedUIKitEditingAndSelectedRustDebug'
def run(args,**kwargs):
 return subprocess.run(args,check=True,text=True,**kwargs)
def require_space(minimum):
 available=shutil.disk_usage(ROOT).free
 if available<minimum*1024*1024:raise RuntimeError(f'Need {minimum}MiB free before simulator/build work; available {available//(1024*1024)}MiB')
def test_targets(value):
 if isinstance(value,dict):
  if value.get('BlueprintName')==TARGET or value.get('TestableName')==TARGET:yield value
  else:
   for child in value.values():yield from test_targets(child)
 elif isinstance(value,list):
  for child in value:yield from test_targets(child)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--device',required=True);parser.add_argument('--build',action='store_true');parser.add_argument('--boot',action='store_true');parser.add_argument('--temporary-device',action='store_true');parser.add_argument('--minimum-free-mib',type=int,default=2048);args=parser.parse_args()
 if args.minimum_free_mib<128:parser.error('Minimum free space must remain at least128MiB')
 require_space(args.minimum_free_mib)
 devices=json.loads(subprocess.check_output(['xcrun','simctl','list','devices','-j']))
 existing_ids={d['udid'].lower()for group in devices['devices'].values()for d in group}
 selected=[(runtime,d)for runtime,group in devices['devices'].items()for d in group if d['udid']==args.device and d.get('isAvailable')is True]
 if len(selected)!=1:raise RuntimeError('Choose exactly one installed available simulator UDID')
 runtime,seed=selected[0]
 if not runtime.startswith('com.apple.CoreSimulator.SimRuntime.iOS-'):raise RuntimeError('Choose an installed iOS simulator')
 if args.temporary_device and not args.boot:parser.error('--temporary-device requires explicit --boot after resource coordination')
 device=args.device;owned_device=None;booted_here=False;passed=False
 try:
  state=seed['state']
  if args.temporary_device:
   returned=subprocess.check_output(['xcrun','simctl','create','HickoryParity-'+uuid.uuid4().hex,seed['deviceTypeIdentifier'],runtime],text=True).strip()
   uuid.UUID(returned)
   if returned.lower()in existing_ids:raise RuntimeError('Create returned a preexisting simulator identity; no boot or deletion is authorized')
   device=returned;owned_device=device;state='Shutdown'
  if state!='Booted':
   if not args.boot:raise RuntimeError('Selected simulator is shutdown; coordinate resources before explicitly using --boot')
   run(['xcrun','simctl','boot',device]);booted_here=True
  run(['xcrun','simctl','bootstatus',device,'-b'])
  derived=ROOT/'target/apple-simple-derived';destination='platform=iOS Simulator,id='+device
  if args.build:
   require_space(args.minimum_free_mib);run(['xcodegen','generate','--spec',str(ROOT/'apps/apple-simple/project.yml')])
   run(['xcodebuild','-project',str(ROOT/'apps/apple-simple/HickorySimple.xcodeproj'),'-scheme','HickorySimpleMobileParity','-configuration','Debug','-destination',destination,'-derivedDataPath',str(derived),'-jobs','2','build-for-testing','CODE_SIGNING_ALLOWED=NO'])
  candidates=[]
  for path in (derived/'Build/Products').glob('*.xctestrun'):
   value=plistlib.loads(path.read_bytes());targets=list(test_targets(value))
   if len(targets)==1:candidates.append((path,value,targets[0]))
  if len(candidates)!=1:raise RuntimeError('Build exactly one hosted parity test configuration with --build; no launch-only acceptance is inferred')
  spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
  with tempfile.TemporaryDirectory(prefix='ios-parity-')as temporary:
   folder=Path(temporary);workspace=folder/'workspace';(workspace/'src').mkdir(parents=True);(workspace/'.git').mkdir();sentinel=workspace/'wrong-ios-sibling-ran';source='// owned-ios-parity-source\n#[cfg(test)]\nmod cases {\n #[test]\n fn test_selected(){\n  let selected_value=41;\n  assert_eq!(selected_value,41); // owned-ios-parity-stop\n }\n #[test]\n fn test_sibling(){std::fs::write('+json.dumps(str(sentinel))+',b"wrong").unwrap();}\n}\n';(workspace/'src/lib.rs').write_text(source);(workspace/'Cargo.toml').write_text('[workspace]\n[package]\nname="owned-ios-parity"\nversion="0.1.0"\nedition="2024"\n')
   with socket.socket()as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(workspace),'window',f'http://127.0.0.1:{port}'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
   try:
    origin=fixture.origin(engine);original,value,target=candidates[0]
    for key in ['EnvironmentVariables','TestHostEnvironmentVariables']:
     target.setdefault(key,{})['HICKORY_IOS_PARITY_ORIGIN']=origin
     target[key]['HICKORY_API_ORIGIN']=origin
    configured=derived/'Build/Products/owned-ios-parity.xctestrun';configured.write_bytes(plistlib.dumps(value));receipt=ROOT/'target/ios-parity';receipt.mkdir(parents=True,exist_ok=True);result=receipt/'last.xcresult'
    if result.exists():shutil.rmtree(result)
    log=receipt/'last.log'
    try:
     with log.open('w')as output:run(['xcodebuild','test-without-building','-xctestrun',str(configured),'-destination',destination,'-parallel-testing-enabled','NO','-maximum-concurrent-test-simulator-destinations','1','-only-testing:'+TARGET+'/'+CASE,'-resultBundlePath',str(result),'-test-timeouts-enabled','YES','-maximum-test-execution-time-allowance','180'],stdout=output,stderr=subprocess.STDOUT)
     summary=json.loads(subprocess.check_output(['xcrun','xcresulttool','get','test-results','summary','--path',str(result)]));(receipt/'last-summary.json').write_text(json.dumps(summary,indent=2))
     assert summary['result']=='Passed' and summary['totalTestCount']==1 and summary['passedTests']==1 and summary['failedTests']==0 and summary['skippedTests']==0,summary
     assert(workspace/'src/lib.rs').read_text()==source and not sentinel.exists()
     observed=log.read_text()
     for marker in ['native-paste-history-passed','coordinated-cleanup-history-passed','debug-paused','completed']:
      assert observed.count('HickoryUIKitParity '+marker)==1,marker
     (receipt/'last-receipt.json').write_text(json.dumps({'device':device,'seed_device':args.device,'temporary_device':owned_device is not None,'test':TARGET+'/'+CASE,'summary':summary,'log_sha256':hashlib.sha256(log.read_bytes()).hexdigest(),'source_unchanged':True,'sibling_excluded':True,'native_paste_undo_redo_verified':True,'coordinated_cleanup_undo_redo_verified':True,'keyboard_input_verified':False,'physical_input_verified':False,'accessibility_verified':False},indent=2))
     passed=True
    finally:configured.unlink(missing_ok=True)
   finally:fixture.stop(engine)
 finally:
  lifecycle={'seed_device':args.device,'device':device,'temporary_device':owned_device is not None,'temporary_device_requested':args.temporary_device,'booted_here':booted_here,'seed_initial_state':seed['state']}
  if owned_device is not None:
   shutdown=subprocess.run(['xcrun','simctl','shutdown',owned_device],capture_output=True,text=True)
   deleted=subprocess.run(['xcrun','simctl','delete',owned_device],capture_output=True,text=True)
   lifecycle.update({'shutdown_exit':shutdown.returncode,'delete_exit':deleted.returncode,'owned_device_deleted':deleted.returncode==0})
  elif booted_here:
   shutdown=subprocess.run(['xcrun','simctl','shutdown',device],capture_output=True,text=True)
   lifecycle.update({'shutdown_exit':shutdown.returncode})
  receipt=ROOT/'target/ios-parity';receipt.mkdir(parents=True,exist_ok=True);(receipt/'last-device-lifecycle.json').write_text(json.dumps(lifecycle,indent=2))
  if owned_device is not None and lifecycle['delete_exit']!=0:raise RuntimeError('Owned simulator cleanup failed; inspect last-device-lifecycle.json and delete only its recorded owned UDID')
  if owned_device is None and booted_here and lifecycle['shutdown_exit']!=0:raise RuntimeError('Selected simulator did not return to Shutdown')
 if passed:print('PASS mounted UIKit Paste Undo/Redo, guarded Cleanup Undo/Redo and saved Rust Debug')
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-ios-parity.py"><ui:paste select="#ios-mounted-parity-runner"/></ui:file>

## Trace saved Rust Debug from the mounted host

The witness records the submitted operation and the debugger's event kinds.
The earlier mounted run reached the 30-second admission deadline. The same
physical Cargo fixture also ran through the workspace proxy and real engine
from a direct protocol client, reached its selected breakpoint and exited with
status zero. In the first mounted Paste history run, the UIKit host received
`started` 4.77 seconds after `start_test`, then received `stopped` and
`finished`. That run used the existing engine fixture binary and debugger
deadline. The earlier timeout's original cause remains unknown.

The final hosted run on 2026-11-18 built the iOS app and test bundle with
Xcode 27 and used a fresh engine on an owned iPhone 18 Pro simulator with
iOS 27.0. XCTest counted one pass, zero failures and zero skips. UIKit Paste
and native Undo/Redo retained exact text and selections across one Unicode
replacement, multiple selections and an append. Reviewed Cleanup and
coordinated Undo/Redo restored the saved bytes before Debug. The saved Rust
test's Cargo and LLDB admission took 6.83 seconds. Its breakpoint stopped at
the selected line, Continue retained process exit zero, and the sibling
sentinel stayed absent. The same editor, selection, viewport, history
availability and pixel frame survived the session. The runner shut down and
deleted only the temporary simulator. Its log SHA-256 is
`dd8a205f78839888576abff2db414aa59291181b34804096b18724ffd31bc3ef`.
Four runner ownership cases passed separately. Device hardware input, IME
and assistive technology remain unexercised by this receipt.

Apple and Android's source-Test Debug admission lease is 30 seconds, while
individual Rust preparation operations can take up to 120 seconds and build
events are delivered after startup completes. Longer cold admissions can
exceed the client deadline. Both mounted passes kept the original lease.

## Refuse unsafe simulator ownership during failures

These focused control-policy checks replace the tool boundary and deliberately fail before building. They establish that a disposable device is deleted using only its returned UUID, an existing seed is restored without deletion, and missing explicit boot authority cannot create a device, and a create result that names any preexisting device never grants boot/delete authority. They do not establish UIKit or simulator acceptance. Run `python3 scripts/test-ios-parity-runner.py` independently of the real hosted witness.

<ui:copy id="ios-mounted-parity-runner-safety">
"""Focused ownership checks; substitutes tool calls and never operates a simulator."""
from pathlib import Path
import importlib.util,io,json,subprocess,sys,tempfile,unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
SEED='22222222-2222-2222-2222-222222222222'
OWNED='11111111-1111-1111-1111-111111111111'
RUNTIME='com.apple.CoreSimulator.SimRuntime.iOS-27-0'
class SimulatorOwnershipTests(unittest.TestCase):
 def load(self):
  spec=importlib.util.spec_from_file_location('owned_ios_runner',ROOT/'scripts/test-ios-parity.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
 def failure(self,temporary):
  runner=self.load();calls=[]
  def output(args,**kwargs):
   calls.append(args)
   if args[:4]==['xcrun','simctl','list','devices']:return json.dumps({'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True,'deviceTypeIdentifier':'owned-type'}]}}).encode()
   self.assertEqual(args[:3],['xcrun','simctl','create']);self.assertEqual(args[-2:],['owned-type',RUNTIME]);return OWNED+'\n'
  def execute(args,**kwargs):
   calls.append(args)
   if args[0]=='xcodegen':raise RuntimeError('owned failure before build')
   return subprocess.CompletedProcess(args,0)
  with tempfile.TemporaryDirectory()as folder,patch.object(runner,'ROOT',Path(folder)),patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',side_effect=output),patch.object(runner.subprocess,'run',side_effect=execute),patch.object(sys,'argv',['runner','--device',SEED,'--boot','--build']+(['--temporary-device']if temporary else[])):
   with self.assertRaisesRegex(RuntimeError,'owned failure before build'):runner.main()
   receipt=json.loads((Path(folder)/'target/ios-parity/last-device-lifecycle.json').read_text());self.assertEqual(receipt['temporary_device'],temporary)
   return calls,receipt
 def test_owned_device_deleted_after_prebuild_failure_without_touching_seed(self):
  calls,receipt=self.failure(True);self.assertIn(['xcrun','simctl','delete',OWNED],calls);self.assertIn(['xcrun','simctl','shutdown',OWNED],calls);self.assertTrue(receipt['owned_device_deleted'])
  for call in calls:
   if call[:2]==['xcrun','simctl']and call[2]in ['boot','shutdown','delete']:self.assertNotIn(SEED,call[2:])
 def test_existing_seed_restored_without_any_delete(self):
  calls,_=self.failure(False);self.assertIn(['xcrun','simctl','shutdown',SEED],calls);self.assertFalse(any(call[:3]==['xcrun','simctl','delete']for call in calls))
 def test_a_preexisting_create_result_never_authorizes_boot_or_delete(self):
  runner=self.load();devices={'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True,'deviceTypeIdentifier':'owned-type'}]}}
  with tempfile.TemporaryDirectory()as folder,patch.object(runner,'ROOT',Path(folder)),patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',side_effect=[json.dumps(devices).encode(),SEED+'\n']),patch.object(runner.subprocess,'run')as mutate,patch.object(sys,'argv',['runner','--device',SEED,'--temporary-device','--boot','--build']):
   with self.assertRaisesRegex(RuntimeError,'preexisting simulator identity'):runner.main()
   mutate.assert_not_called();receipt=json.loads((Path(folder)/'target/ios-parity/last-device-lifecycle.json').read_text());self.assertFalse(receipt['temporary_device']);self.assertFalse(receipt['booted_here'])
 def test_temporary_mode_without_explicit_boot_cannot_create_a_device(self):
  runner=self.load();devices={'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True,'deviceTypeIdentifier':'owned-type'}]}}
  with patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',return_value=json.dumps(devices).encode())as read,patch.object(runner.subprocess,'run')as mutate,patch.object(sys,'argv',['runner','--device',SEED,'--temporary-device']),patch('sys.stderr',new=io.StringIO()):
   with self.assertRaises(SystemExit)as error:runner.main()
   self.assertEqual(error.exception.code,2);self.assertEqual(read.call_count,1);mutate.assert_not_called()
if __name__=='__main__':unittest.main()
</ui:copy>
<ui:file path="scripts/test-ios-parity-runner.py"><ui:paste select="#ios-mounted-parity-runner-safety"/></ui:file>
</ui:doc>
