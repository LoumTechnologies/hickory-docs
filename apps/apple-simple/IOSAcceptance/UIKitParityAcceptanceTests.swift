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
