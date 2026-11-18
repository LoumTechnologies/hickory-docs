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

The workspace case drives native command handlers and checks exact caret positions
after saved search and clangd Problems navigation. It runs only the selected
pytest declaration from the source index, then drives the bundled coverage
WebKit page to distinguish a failed case from a process exit and verify the
matching source marks. The recovery case follows the bound construction entry
through declared relationships and a selected check, then restores a private
draft copy into a new pane. The historical proposal case reopens a sealed
intention after a genuine engine restart while the original editor retains an
independent dirty draft.

<ui:copy id="ios-mounted-parity-test">
import XCTest
import UIKit
import SwiftUI
import WebKit
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
        let clock = ContinuousClock()
        var submitted: ContinuousClock.Instant?
        var preparation: Duration?, admission: Duration?
        debug.observeRequestForTest = { request in
            guard let op = request["op"] as? String else { return }
            if op == "start_test" { submitted = clock.now }
            mark("debug-request \(op)")
        }
        debug.observeEventForTest = { response in
            guard let event = response["event"] as? String else { return }
            if let submitted {
                if event == "test_preparing" { preparation = submitted.duration(to: clock.now) }
                if event == "started" { admission = submitted.duration(to: clock.now) }
            }
            mark("debug-event \(event)")
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
        if ProcessInfo.processInfo.environment["HICKORY_IOS_PARITY_SLOW_DEBUG"] == "1" {
            let early = try XCTUnwrap(preparation, "The engine must acknowledge preparation within the initial lease")
            let late = try XCTUnwrap(admission, "Debug must still bind the selected saved source")
            XCTAssertLessThan(early, .seconds(30))
            XCTAssertGreaterThan(late, .seconds(30))
            mark("slow-debug-admission-passed preparation=\(early) start=\(late)")
        }
        mark("completed")
    }
}

// Keep the actual UIKit widget, tab layout and adapter connections mounted for
// the complete workflow, including presentations and source retargeting.
@MainActor private final class MountedIOSWorkspace {
    let workspace: Workspace
    let host: UIHostingController<WorkspaceView>
    let window: UIWindow
    private weak var previous: UIWindow?

    init(marker: String, expected: String, path: String) async throws {
        let origin = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PARITY_ORIGIN"], "Use the owned iOS fixture")
        let workspace = Workspace()
        workspace.address = origin
        guard await workspace.connect() else { throw EngineError(message: workspace.status) }
        let engine = try XCTUnwrap(workspace.engine)
        let ownership = try await engine.file(FileNode(name: marker, path: marker, dir: false))
        guard ownership.content == expected else { throw EngineError(message: "Refuse an unrelated workspace") }
        let file = try XCTUnwrap(Self.file(path, in: workspace.rootFiles))
        _ = try await workspace.loadBuffer(file)
        workspace.showSource(path)
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        previous = scene.windows.first { $0.isKeyWindow }
        self.workspace = workspace
        host = UIHostingController(rootView: WorkspaceView(workspace: workspace))
        window = UIWindow(windowScene: scene)
        window.frame = scene.coordinateSpace.bounds
        window.windowLevel = UIWindow.Level(rawValue: UIWindow.Level.normal.rawValue + 1)
        window.rootViewController = host
        window.makeKeyAndVisible()
    }

    static func file(_ path: String, in nodes: [FileNode]) -> FileNode? {
        for node in nodes {
            if exactUTF8(node.path, path) { return node }
            if let child = file(path, in: node.children ?? []) { return child }
        }
        return nil
    }

    func view<V: UIView>(_ type: V.Type, matching: (V) -> Bool) -> V? {
        func visit(_ view: UIView) -> V? {
            if let typed = view as? V, matching(typed) { return typed }
            for child in view.subviews {
                if let found = visit(child) { return found }
            }
            return nil
        }
        return visit(window)
    }

    func wait(_ message: String, until: () -> Bool) async throws {
        let clock = ContinuousClock(), deadline = clock.now + .seconds(45)
        while clock.now < deadline {
            host.view.layoutIfNeeded()
            if until() { return }
            try await Task.sleep(for: .milliseconds(50))
        }
        let detail = message + " · " + workspace.status
        XCTFail(detail)
        throw EngineError(message: detail)
    }

    func editor(_ buffer: DocumentBuffer) async throws -> UITextView {
        var native: UITextView?
        try await wait("Editor did not mount for " + buffer.file.path) {
            native = view(UITextView.self) { exactUTF8($0.accessibilityLabel, buffer.file.path) }
            return native != nil && buffer.sourceEditorState?()?.attached == true
        }
        return try XCTUnwrap(native)
    }

    func command(_ id: String) throws {
        let command = try XCTUnwrap(workspace.commands().first { $0.id == id }, id)
        XCTAssertNil(command.reason)
        command.run()
    }

    func paste(_ source: String, into text: UITextView) {
        XCTAssertTrue(text.becomeFirstResponder())
        text.selectedRange = NSRange(location: text.text.utf16.count, length: 0)
        let destination: UIPasteConfigurationSupporting = text
        destination.paste?(itemProviders: [NSItemProvider(object: source as NSString)])
    }

    func close() {
        workspace.debugger?.close()
        workspace.coverage.stop()
        workspace.editorRecovery.close()
        workspace.sourceTestIndex?.stop()
        for buffer in workspace.buffers.values { buffer.language?.close() }
        window.isHidden = true
        window.rootViewController = nil
        previous?.makeKey()
    }
}

extension UIKitParityAcceptanceTests {
    func testMountedSearchProblemsAndCollectedTests() async throws {
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-ios-workspace", expected: "owned-workspace-v1\n", path: "app.py")
        defer { mounted.close() }
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let app = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(app)
        let identity = try XCTUnwrap(app.sourceEditorState?()).identity
        let source = app.text
        editor.selectedRange = NSRange(location: 2, length: 0)
        let appSelection = editor.selectedRange

        try mounted.command("workspace.find")
        workspace.search.query = "owned_problem_two"
        await workspace.search.run(engine)
        let hit = try XCTUnwrap(workspace.search.result?.hits.first)
        XCTAssertEqual(hit.path, "diagnostics.c")
        await workspace.openSearchHit(hit)
        let diagnostics = try XCTUnwrap(workspace.currentBuffer)
        XCTAssertEqual(diagnostics.file.path, "diagnostics.c")
        let diagnosticEditor = try await mounted.editor(diagnostics)
        let hitOffset = try XCTUnwrap(languageOffset(diagnostics.text, ["line": hit.line - 1, "character": try XCTUnwrap(hit.character)]))
        XCTAssertEqual(diagnosticEditor.selectedRange.location, hitOffset)
        await workspace.navigateHistory(-1)
        XCTAssertTrue(workspace.currentBuffer === app)
        let returned = try await mounted.editor(app)
        XCTAssertTrue(returned === editor)
        XCTAssertEqual(returned.selectedRange, appSelection)
        await workspace.navigateHistory(1)
        XCTAssertTrue(workspace.currentBuffer === diagnostics)

        try await mounted.wait("clangd did not report current source positions") {
            workspace.problems.filter { $0.path == "diagnostics.c" && $0.current }.count >= 2
        }
        let positions = workspace.problems.filter { $0.path == "diagnostics.c" && $0.current }
        let first = try XCTUnwrap(positions.first)
        try mounted.command("workspace.problems")
        await workspace.openProblem(first)
        let target = try XCTUnwrap(first.location)
        let offset = try XCTUnwrap(languageOffset(diagnostics.text, ["line": target.line, "character": target.character]))
        try await mounted.wait("Diagnostic did not reveal exact source") { diagnosticEditor.selectedRange.location == offset }
        await workspace.navigateProblem(1)
        let next = diagnosticEditor.selectedRange
        XCTAssertNotEqual(next.location, offset)
        await workspace.navigateProblem(-1)
        XCTAssertEqual(diagnosticEditor.selectedRange.location, offset)
        mounted.paste("\n// unsaved diagnostic context\n", into: diagnosticEditor)
        try await mounted.wait("Diagnostic draft did not change") { diagnostics.dirty }
        let changedPosition = diagnosticEditor.selectedRange
        await workspace.openProblem(first)
        XCTAssertEqual(diagnosticEditor.selectedRange, changedPosition)
        XCTAssertTrue(workspace.status.contains("changed") || workspace.status.contains("unversioned"))
        workspace.showProblems = false
        let diagnosticManager = try XCTUnwrap(diagnosticEditor.undoManager)
        diagnosticManager.undo()
        try await mounted.wait("Diagnostic Paste Undo did not restore saved text") { !diagnostics.dirty }
        NSLog("HickoryUIKitParity workspace-navigation-problems-passed")

        let testFile = try XCTUnwrap(MountedIOSWorkspace.file("test_app.py", in: workspace.rootFiles))
        let testSource = try await workspace.loadBuffer(testFile)
        await workspace.open(testFile)
        _ = try await mounted.editor(testSource)
        try await mounted.wait("Source-adjacent pytest declaration missing") {
            testSource.sourceTestRows().contains { $0.candidate.name == "test_positive" }
        }
        let row = try XCTUnwrap(testSource.sourceTestRows().first { $0.candidate.name == "test_positive" })
        await workspace.inspectTestSource(testSource, line: row.line)
        let choices = try XCTUnwrap(workspace.testSources.observation?.1, workspace.testSources.notice)
        let candidate = try XCTUnwrap(choices.candidates.first { $0.name == "test_positive" })
        await workspace.testSources.run(candidate, .nameFilter, workspace: workspace)
        let run = try XCTUnwrap(workspace.testSources.runs.first, workspace.testSources.notice)
        for _ in 0..<120 {
            await workspace.testSources.refreshRun(run.id, workspace: workspace)
            if workspace.testSources.runs.first?.exit != nil { break }
            try await Task.sleep(for: .milliseconds(100))
        }
        XCTAssertEqual(workspace.testSources.runs.first?.exit, 0)
        XCTAssertTrue(workspace.testSources.runs.first?.output.contains("1 passed") == true)
        workspace.showTests = false
        try await mounted.wait("Source-test review did not dismiss") { mounted.host.presentedViewController == nil }
        workspace.showSource(app.file.path)
        let current = try await mounted.editor(app)
        XCTAssertTrue(current === editor)
        mounted.window.endEditing(true)
        try await mounted.wait("Source keyboard did not release") { !current.isFirstResponder }

        try mounted.command("workspace.coverage")
        func coverage() async throws -> WKWebView {
            var web: WKWebView?
            try await mounted.wait("Coverage WKWebView did not show") {
                web = mounted.view(WKWebView.self) { $0.url?.lastPathComponent == "coverage.html" }
                return web != nil
            }
            return try XCTUnwrap(web)
        }
        let web = try await coverage()
        func awaitText(_ fragment: String) async throws {
            for _ in 0..<400 {
                if let text = (try? await web.evaluateJavaScript("document.body.innerText")) as? String,
                   text.contains(fragment) { return }
                try await Task.sleep(for: .milliseconds(100))
            }
            throw EngineError(message: "Coverage surface did not show " + fragment)
        }
        func click(_ label: String) async throws {
            let literal = String(decoding: try JSONSerialization.data(withJSONObject: label, options: .fragmentsAllowed), as: UTF8.self)
            _ = try await web.evaluateJavaScript("(()=>{const b=[...document.querySelectorAll('button')].find(node=>node.textContent.trim()===" + literal + ");if(!b||b.disabled)throw Error('Control unavailable');b.click();return true})()")
        }
        try await awaitText("Discover runner cases")
        try await click("Discover runner cases")
        try await awaitText("discovered · test_app.py::test_negative")
        try await click("Run saved project with coverage")
        try await awaitText("automated · failed · current source")
        try await awaitText("failed · test_app.py::test_negative")
        try await awaitText("executable lines covered")
        try await mounted.wait("Coverage did not bind app.py source") { !app.coverageRows().isEmpty }
        let destination = try await web.evaluateJavaScript("""
            (() => {
                const title = [...document.querySelectorAll('summary')].find(
                    row => row.textContent.startsWith('failed · test_app.py::test_negative'));
                const failure = title?.parentElement;
                if (!failure) throw Error('Failed case is unavailable');
                failure.open = true;
                const button = failure.querySelector('button');
                if (!button?.textContent.startsWith('Open source line ')) throw Error('No bound failure position');
                const line = Number(button.textContent.trim().split(' ').at(-1));
                button.click();
                return line;
            })()
            """)
        let failureLine = try XCTUnwrap(destination as? Int)
        try await mounted.wait("Coverage review did not dismiss") { !workspace.showCoverage && mounted.host.presentedViewController == nil }
        XCTAssertTrue(workspace.currentBuffer === testSource)
        let failureEditor = try await mounted.editor(testSource)
        let failureOffset = try XCTUnwrap(languageOffset(testSource.text, ["line": failureLine - 1, "character": 0]))
        XCTAssertEqual(failureEditor.selectedRange.location, failureOffset)
        workspace.showSource(app.file.path)
        let after = try await mounted.editor(app)
        XCTAssertTrue(after === editor)
        XCTAssertEqual(app.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(after.selectedRange, appSelection)
        XCTAssertEqual(app.text, source)
        let saved = try await engine.file(app.file)
        XCTAssertEqual(saved.content, source)
        mounted.paste("\n# current draft differs from covered source\n", into: editor)
        try await mounted.wait("Draft did not retire coverage") { app.dirty && app.coverageRows().isEmpty }
        NSLog("HickoryUIKitParity workspace-tests-coverage-passed")
    }

    func testMountedConnectedReadingAndPrivateRecovery() async throws {
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-native-behavior-fixture", expected: "owned-native-behavior-v1\n", path: "source.md")
        let workspace = mounted.workspace
        let connected = ConnectedBehaviorModel()
        defer { connected.close(); mounted.close() }
        let engine = try XCTUnwrap(workspace.engine)
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let source = buffer.text
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let manager = try XCTUnwrap(editor.undoManager)
        let keyboard = HostedKeyboardObservation()
        let keyboardEvents = [UIResponder.keyboardWillShowNotification, UIResponder.keyboardDidShowNotification,
                              UIResponder.keyboardDidHideNotification]
        let keyboardTokens = keyboardEvents.map { name in
            NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) { [weak keyboard] notification in
                MainActor.assumeIsolated { keyboard?.record(notification) }
            }
        }
        defer { keyboardTokens.forEach { NotificationCenter.default.removeObserver($0) } }

        func markViewport(_ stage: String) {
            let point = editor.contentOffset
            let layout = editor.layoutManager, container = editor.textContainer
            let glyph = layout.glyphIndex(for: CGPoint(x: point.x, y: point.y - editor.textContainerInset.top), in: container)
            let character = glyph < layout.numberOfGlyphs ? layout.characterIndexForGlyph(at: glyph) : -1
            NSLog("HickoryUIKitParity recovery-viewport %@ offset=%.2f,%.2f width=%.2f height=%.2f topUTF16=%ld keyboard=%@ first=%@",
                  stage, point.x, point.y, editor.bounds.width, editor.bounds.height, character, keyboard.event, String(editor.isFirstResponder))
        }

        try mounted.command("workspace.construction")
        try await mounted.wait("Construction review did not present") { mounted.host.presentedViewController != nil }
        let original = try await WorkspaceEntrySelections.read(load: {
            let reply: ResourceJSON = try await engine.request("GET", "/api/workspace/entry")
            return try JSONSerialization.jsonObject(with: JSONEncoder().encode(reply))
        }, workspace: try XCTUnwrap(workspace.sourceWorkspaceID),
           namespace: try XCTUnwrap(workspace.sourceRootAddress), current: { mounted.window.rootViewController === mounted.host })
        let entry = try XCTUnwrap(original.entryPath)
        await connected.read(workspace, entry: entry, focus: "a.py", entrySelection: original.proof, original: original.current)
        let reading = try XCTUnwrap(connected.value, connected.notice)
        XCTAssertEqual(reading.verification_available, true)
        XCTAssertTrue(reading.graph.nodes.values.contains { $0.path == "a.py" && $0.status == "constructed" })
        XCTAssertTrue(reading.graph.nodes.values.contains { $0.kind == "test-module" })
        let contract = try XCTUnwrap(reading.graph.nodes.values.first { $0.kind == "contract" })
        await connected.readSource(contract.id)
        XCTAssertEqual(connected.page?.text, "# Answer behavior\nReturn the declared answer.\n")
        let explanation = try XCTUnwrap(reading.graph.nodes.values.first { $0.kind == "explanation" })
        await connected.readSource(explanation.id)
        XCTAssertEqual(connected.page?.fragment_id, "answer")
        XCTAssertEqual(connected.page?.target_status, "observed-fragment-context")
        await connected.readSource(explanation.id, offset: 0)
        var full = ""
        while let page = connected.page {
            full += page.text
            guard let next = page.next else { break }
            await connected.continueSource(page, offset: next)
            guard connected.page?.offset == next else { throw EngineError(message: connected.notice) }
        }
        XCTAssertEqual(full, source)
        let supported = try XCTUnwrap(reading.graph.checks.first { $0.label == "test_selected" && $0.available })
        let unsupported = try XCTUnwrap(reading.graph.checks.first { $0.label == "Explicit unsupported check" && !$0.available })
        await connected.run(unsupported.id)
        XCTAssertTrue(connected.receipts.isEmpty)
        await connected.run(supported.id)
        let receipt = try XCTUnwrap(connected.receipts.first, connected.notice)
        for _ in 0..<150 {
            guard connected.receipts.first?.status != "process-finished" else { break }
            await connected.refresh(receipt)
            try await Task.sleep(for: .milliseconds(100))
        }
        XCTAssertEqual(connected.receipts.first?.status, "process-finished")
        XCTAssertEqual(connected.receipts.first?.outcome, "process-only")
        XCTAssertEqual(connected.receipts.first?.exit_code, 0)
        XCTAssertTrue(connected.current)
        let appended = "\nReview snapshot changed.\n"
        let insertion = NativeSourceEdit(range: NSRange(location: source.utf16.count, length: 0), text: appended)
        XCTAssertTrue(buffer.applySourceEdits?([insertion], source) == true)
        XCTAssertEqual(editor.text, source + appended)
        await connected.run(supported.id)
        XCTAssertEqual(connected.receipts.count, 1)
        XCTAssertFalse(connected.current)
        let removal = NativeSourceEdit(range: NSRange(location: source.utf16.count, length: appended.utf16.count), text: "")
        XCTAssertTrue(buffer.applySourceEdits?([removal], source + appended) == true)
        workspace.showConstruction = false
        try await mounted.wait("Construction review did not dismiss") { mounted.host.presentedViewController == nil }
        let afterReading = try await mounted.editor(buffer)
        XCTAssertTrue(afterReading === editor)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(buffer.text, source)
        NSLog("HickoryUIKitParity connected-impact-passed")

        mounted.paste("\nRetained iOS recovery draft.\n", into: editor)
        try await mounted.wait("Source was not marked dirty") { buffer.dirty }
        let draft = buffer.text

        editor.selectedRange = NSRange(location: 3, length: 4)
        let originalSelection = editor.selectedRange
        let expectedHide = keyboard.hidden + (keyboard.visible ? 1 : 0)
        mounted.window.endEditing(true)
        try await mounted.wait("Recovery keyboard did not finish hiding") {
            !editor.isFirstResponder && !keyboard.visible && keyboard.hidden >= expectedHide
        }
        mounted.host.view.layoutIfNeeded()
        mounted.window.layoutIfNeeded()
        editor.setContentOffset(CGPoint(x: 0, y: 90), animated: false)
        let originalScroll = editor.contentOffset
        XCTAssertGreaterThan(originalScroll.y, 0)
        markViewport("checkpoint")
        let recovery = workspace.editorRecovery
        let rowID = editorRecoveryToken(buffer.id)
        let before = recovery.revision
        for _ in 0..<40 {
            await recovery.checkpoint(workspace)
            if recovery.revision > before { break }
            try await Task.sleep(for: .milliseconds(50))
        }
        await recovery.open(workspace)
        let catalogue = try XCTUnwrap(recovery.catalogue, recovery.notice)
        var matching: EditorRecoverySession?
        for row in catalogue.sessions.filter({ $0.host == "ios" }).reversed() {
            await recovery.read(row, workspace)
            if recovery.snapshot?.buffers.contains(where: { $0.id == rowID && $0.text == draft }) == true {
                matching = row
                break
            }
        }
        _ = try XCTUnwrap(matching, recovery.notice)
        XCTAssertTrue(recovery.canRestore, recovery.notice)
        markViewport("before-restore")
        XCTAssertEqual(editor.contentOffset, originalScroll)
        let originalRoot = workspace.layout.rootID
        let originalGroups = workspace.layout.groups().map(\.id)
        let originalPane = workspace.focused
        let originalWidth = editor.bounds.width
        let compact = mounted.host.traitCollection.horizontalSizeClass == .compact
        XCTAssertTrue(compact, "The owned phone must exercise compact recovery")
        recovery.restore(workspace, inTabs: compact)
        recovery.close()
        let restored = try XCTUnwrap(workspace.currentBuffer)
        XCTAssertFalse(restored === buffer)
        XCTAssertEqual(restored.text, draft)
        XCTAssertTrue(restored.isUntitledDraft)
        XCTAssertNil(restored.savedGuard)
        XCTAssertEqual(restored.recoveryBaseline?.text, source)
        XCTAssertEqual(workspace.layout.rootID, originalRoot)
        XCTAssertEqual(workspace.layout.groups().map(\.id), originalGroups)
        XCTAssertEqual(workspace.focused, originalPane)
        try await mounted.wait("Recovery review did not dismiss") { mounted.host.presentedViewController == nil }
        let restoredEditor = try await mounted.editor(restored)
        try await mounted.wait("Recovered view did not restore selection and scroll") {
            restoredEditor.selectedRange == originalSelection && restoredEditor.contentOffset == originalScroll
        }
        markViewport("recovered-attached")
        workspace.showSource(buffer.file.path)
        let originalEditor = try await mounted.editor(buffer)
        markViewport("original-active")
        XCTAssertTrue(originalEditor === editor)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(originalEditor.selectedRange, originalSelection)
        XCTAssertEqual(originalEditor.contentOffset, originalScroll)
        XCTAssertEqual(originalEditor.bounds.width, originalWidth)
        XCTAssertEqual(buffer.text, draft)
        XCTAssertTrue(manager.canUndo)
        XCTAssertEqual(manager.undoActionName, "Paste")
        manager.undo()
        NSLog("HickoryUIKitParity recovery-undo restored=%@ redo=%@ draftMatchesWidget=%@ notice=%@",
              String(buffer.text == source), String(manager.canRedo), String(buffer.text == originalEditor.text), buffer.snippets.notice)
        try await mounted.wait("Original native history did not reach saved text") { buffer.text == source }
        XCTAssertTrue(manager.canRedo)
        manager.redo()
        try await mounted.wait("Original native Redo did not restore the exact draft") { buffer.text == draft }
        XCTAssertEqual(originalEditor.text, draft)
        let saved = try await engine.file(buffer.file)
        XCTAssertEqual(saved.content, source)
        NSLog("HickoryUIKitParity private-recovery-passed")
    }

    func testMountedHistoricalProposalRefusesApplication() async throws {
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-proposal-archive-fixture", expected: "owned-proposal-archive-v1\n", path: "note.py")
        defer { mounted.close() }
        let token = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PROPOSAL_TOKEN"])
        let fingerprint = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PROPOSAL_FINGERPRINT"])
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let source = buffer.text
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let manager = try XCTUnwrap(editor.undoManager)
        mounted.paste("# independent current draft\n", into: editor)
        try await mounted.wait("Current source draft did not change") { buffer.dirty }
        mounted.window.endEditing(true)
        try await mounted.wait("Keyboard did not release") { !editor.isFirstResponder }
        let draft = buffer.text, selection = editor.selectedRange
        let review = workspace.agentProposals
        try mounted.command("agent.proposals")
        try await mounted.wait("Historical proposal list unavailable") { review.entries.contains { $0.token == token } && !review.busy }
        let row = try XCTUnwrap(review.entries.first { $0.token == token })
        XCTAssertEqual(row.fingerprint, fingerprint)
        await review.read(workspace, row)
        XCTAssertTrue(review.value?.historical == true, review.notice)
        XCTAssertTrue(review.value?.sections.contains { $0.title.hasPrefix("Explanation") } == true)
        XCTAssertFalse(review.canApply)
        XCTAssertFalse(review.authorityCurrent)
        await review.applyDraft(workspace)
        await review.prepareSave(workspace)
        XCTAssertEqual(buffer.text, draft)
        review.close()
        let returned = try await mounted.editor(buffer)
        XCTAssertTrue(returned === editor)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(returned.selectedRange, selection)
        XCTAssertTrue(manager.canUndo)
        manager.undo()
        try await mounted.wait("Original editable source history was lost") { buffer.text == source }
        let saved = try await engine.file(buffer.file)
        XCTAssertEqual(saved.content, source)
        NSLog("HickoryUIKitParity historical-proposal-passed")
    }

    func testMountedManagedProposalAppliesBothReviewedDrafts() async throws {
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-native-agent-proposal-fixture", expected: "owned-native-agent-proposal-v1\n", path: "note.py")
        defer { mounted.close() }
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let note = try XCTUnwrap(workspace.currentBuffer)
        let noteEditor = try await mounted.editor(note)
        let constructionFile = try XCTUnwrap(MountedIOSWorkspace.file("construction.md", in: workspace.rootFiles))
        let construction = try await workspace.loadBuffer(constructionFile)
        workspace.showSource(construction.file.path)
        let constructionEditor = try await mounted.editor(construction)
        let savedNote = note.text, savedConstruction = construction.text
        let closing = (savedConstruction as NSString).range(of: "Retained author context")
        guard closing.location != NSNotFound else { throw EngineError(message: "Owned construction viewport fixture is unavailable.") }
        constructionEditor.selectedRange = NSRange(location: closing.location, length: 3)
        let constructionSelection = constructionEditor.selectedRange
        let shiftedConstructionSelection = NSRange(location: constructionSelection.location + 1, length: constructionSelection.length)
        constructionEditor.layoutIfNeeded()
        constructionEditor.setContentOffset(CGPoint(x: 0, y: 90), animated: false)
        let constructionViewport = constructionEditor.contentOffset
        XCTAssertGreaterThan(constructionViewport.y, 0)
        workspace.showSource(note.file.path)
        let originalNote = try await mounted.editor(note)
        XCTAssertTrue(originalNote === noteEditor)
        noteEditor.selectedRange = NSRange(location: 2, length: 0)
        let noteSelection = noteEditor.selectedRange
        let noteFrame = noteEditor.convert(noteEditor.bounds, to: mounted.window)
        let identities = try [note, construction].map { try XCTUnwrap($0.sourceEditorState?()).identity }
        XCTAssertEqual(savedNote, "value=1\n")
        XCTAssertTrue(savedConstruction.contains("generated = 1"))
        let afterNote = "value=41\n"
        let afterConstruction = savedConstruction.replacingOccurrences(of: "generated = 1", with: "generated = 41")

        // The owned ACP child submits the two-file intention through the real
        // managed tool using this sender snapshot and original loaded guards.
        let _: ResourceJSON = try await engine.request("POST", "/api/docs/workspace/agent/acp", body: ["backend": "owned-proposal"])
        let _: ResourceJSON = try await engine.request("POST", "/api/docs/workspace/agent", body: [
            "backend": "owned-proposal",
            "prompt": "Prepare the owned two-source intention for review. Do not apply, save or execute it.",
            "context": try nativeAgentContext(workspace),
        ])
        struct Catalogue: Decodable { let reviews: [AgentProposalEntry] }
        var discovered: AgentProposalEntry?
        for _ in 0..<150 {
            let reply: Catalogue = try await engine.request("GET", "/api/agent-proposals")
            if let first = reply.reviews.first { discovered = first; break }
            try await Task.sleep(for: .milliseconds(100))
        }
        let entry = try XCTUnwrap(discovered, "Managed ACP tool did not prepare a proposal")
        let review = workspace.agentProposals
        try mounted.command("agent.proposals")
        try await mounted.wait("Managed proposal review did not present") {
            !review.busy && review.entries.contains(entry) && mounted.host.presentedViewController != nil
        }
        await review.read(workspace, entry)
        let sealed = try XCTUnwrap(review.value, review.notice)
        XCTAssertTrue(sealed.material.origin_authority.hasPrefix("engine-managed agent tool"))
        XCTAssertEqual(nativeSourceHash(sealed.envelope.material), entry.fingerprint)
        XCTAssertTrue(sealed.textOnly)
        XCTAssertFalse(sealed.envelope.source_effects)
        XCTAssertFalse(sealed.envelope.checks_executed)
        let patches = Dictionary(uniqueKeysWithValues: sealed.patches.map { ($0.path, $0) })
        XCTAssertEqual(Set(patches.keys), Set(["note.py", "construction.md"]))
        XCTAssertEqual(patches["note.py"]?.before?.text, savedNote)
        XCTAssertEqual(patches["note.py"]?.after?.text, afterNote)
        XCTAssertEqual(patches["construction.md"]?.before?.text, savedConstruction)
        XCTAssertEqual(patches["construction.md"]?.after?.text, afterConstruction)
        let consequences = try XCTUnwrap(sealed.sections.first { $0.title.hasPrefix("Saved and draft generated consequences") })
        XCTAssertTrue(consequences.text.contains("generated.py"))
        XCTAssertTrue(sealed.sections.contains { $0.title.hasPrefix("Explanation") })
        XCTAssertTrue(sealed.sections.contains { $0.title.hasPrefix("Claims · unverified") })
        XCTAssertTrue(sealed.sections.contains { $0.title.hasPrefix("Requested checks") })
        XCTAssertEqual(note.text, savedNote)
        XCTAssertEqual(construction.text, savedConstruction)
        XCTAssertTrue(review.canApply, review.notice)
        NSLog("HickoryUIKitParity managed-proposal-preview-passed")

        await review.applyDraft(workspace)
        try await mounted.wait("Reviewed two-source batch did not apply") {
            note.text == afterNote && construction.text == afterConstruction
        }
        XCTAssertEqual(noteEditor.selectedRange, noteSelection)
        XCTAssertEqual(constructionEditor.selectedRange, shiftedConstructionSelection)
        XCTAssertEqual(constructionEditor.contentOffset, constructionViewport)
        XCTAssertFalse(review.canApply)
        XCTAssertFalse(review.hasReport, review.notice)
        review.close()
        try await mounted.wait("Proposal review did not dismiss") { mounted.host.presentedViewController == nil }
        workspace.changes.open("undo", workspace: workspace)
        try await mounted.wait("One coordinated Undo did not restore both source drafts") {
            note.text == savedNote && construction.text == savedConstruction
        }
        XCTAssertEqual(noteEditor.selectedRange, noteSelection)
        XCTAssertEqual(constructionEditor.selectedRange, constructionSelection)
        XCTAssertEqual(constructionEditor.contentOffset, constructionViewport)
        workspace.changes.open("redo", workspace: workspace)
        try await mounted.wait("One coordinated Redo did not replay both source drafts") {
            note.text == afterNote && construction.text == afterConstruction
        }
        workspace.changes.close(workspace)
        mounted.window.endEditing(true)
        try await mounted.wait("History review did not return to the original editor frame") {
            mounted.host.presentedViewController == nil && !noteEditor.isFirstResponder &&
            noteEditor.convert(noteEditor.bounds, to: mounted.window) == noteFrame
        }

        let reflectedNote = try await mounted.editor(note)
        XCTAssertTrue(reflectedNote === noteEditor)
        XCTAssertEqual(reflectedNote.selectedRange, noteSelection)
        XCTAssertEqual(reflectedNote.convert(reflectedNote.bounds, to: mounted.window), noteFrame)
        XCTAssertEqual(reflectedNote.text, afterNote)
        workspace.showSource(construction.file.path)
        let reflectedConstruction = try await mounted.editor(construction)
        XCTAssertTrue(reflectedConstruction === constructionEditor)
        XCTAssertEqual(reflectedConstruction.selectedRange, shiftedConstructionSelection)
        XCTAssertEqual(reflectedConstruction.contentOffset, constructionViewport)
        XCTAssertEqual(reflectedConstruction.text, afterConstruction)
        XCTAssertEqual(try [note, construction].map { try XCTUnwrap($0.sourceEditorState?()).identity }, identities)
        let originalNoteOnDisk = try await engine.file(note.file)
        let originalConstructionOnDisk = try await engine.file(construction.file)
        XCTAssertEqual(originalNoteOnDisk.content, savedNote)
        XCTAssertEqual(originalConstructionOnDisk.content, savedConstruction)
        NSLog("HickoryUIKitParity managed-proposal-history-passed")
    }
}
</ui:copy>
<ui:file path="apps/apple-simple/IOSAcceptance/UIKitParityAcceptanceTests.swift"><ui:paste select="#ios-mounted-parity-test"/></ui:file>

## Run only on a deliberately selected simulator

Coordinate the disk/link window before building or booting. The runner defaults to an already booted installed simulator and an existing hosted test build. `--build` explicitly builds the one test scheme using the existing derived products; `--boot` starts the selected simulator and restores Shutdown afterward. With explicit `--temporary-device --boot`, the selected installed device is only a seed for its exact installed runtime and device type: the runner creates a fresh uniquely named device, rejects any returned UUID from the initial complete device list (including unavailable devices), uses only a genuinely new returned UUID, then shuts down and deletes only that owned device in finally. The seed and its data stay untouched; no clone, runtime download or signing change occurs. Device-owned storage is reclaimed on deletion, while global simulator/OS caches may remain. A default2GiB free-space floor guards the measured first-boot/keyboard-cache cost; a coordinated run may explicitly choose a different floor. It is an admission check, not a guarantee against unrelated concurrent disk writes. Simulator and build caches can exceed100MiB, so this must not be presented as a small launch-only probe.

After generation and resource coordination, use `DEVELOPER_DIR=/Applications/Xcode-27.0.0.app/Contents/Developer python3 scripts/test-ios-parity.py --device <installed-UDID> --build --boot --temporary-device`. Omit those flags to reuse an already built/booted target. The runner starts an isolated workspace engine, passes its address through a disposable XCTest configuration and runs exactly one named case. It admits only one passed case with zero failures/skips from the real xcresult summary, retains the result/log digest, checks saved source and sibling exclusion, and cleans up its owned runtime. A separate lifecycle receipt records the seed, actual returned device UUID and deletion outcome, including failed attempts. Compilation, invocation and physical-input acceptance must remain unverified until actually executed.

By default the runner selects `editing`. Use repeated `--workflow` flags to
reuse one owned simulator while giving each case its own workspace and exact
XCTest result. The `workspace` case requires clangd and uses the existing pytest
and pytest-cov collector fixture. Both `workspace` and `recovery` use the
installed environment from `--python-env`. The `proposal` case checks engine
lifetime turnover before mounting its saved historical review. The optional
`--slow-debug` fixture deliberately delays Cargo longer than 30 seconds and
requires a staged preparation acknowledgment before that threshold.

The `proposal-apply` case uses the owned scripted ACP provider and the same
native proposal review. Supply the sender's actual editors to the managed
tool, read the sealed proposal before applying its ordinary and authored
changes together, and retain a separate generated-output consequence.
One coordinated Undo and Redo must reach both drafts.

<ui:copy id="ios-mounted-parity-runner">
"""Owned mounted UIKit parity witness. Requires an explicitly selected simulator."""
import argparse,hashlib,importlib.util,json,os,plistlib,shutil,socket,subprocess,tempfile,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TARGET = 'HickorySimpleMobileParityTests'
WORKFLOWS = {
    'editing': ('testMountedUIKitEditingAndSelectedRustDebug', ['native-paste-history-passed', 'coordinated-cleanup-history-passed', 'debug-paused', 'completed']),
    'workspace': ('testMountedSearchProblemsAndCollectedTests', ['workspace-navigation-problems-passed', 'workspace-tests-coverage-passed']),
    'recovery': ('testMountedConnectedReadingAndPrivateRecovery', ['connected-impact-passed', 'private-recovery-passed']),
    'proposal': ('testMountedHistoricalProposalRefusesApplication', ['historical-proposal-passed']),
    'proposal-apply': ('testMountedManagedProposalAppliesBothReviewedDrafts', ['managed-proposal-preview-passed', 'managed-proposal-history-passed']),
}

def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def prepare_workflow(workflow, workspace, python_env, slow_debug):
    workspace.mkdir()
    if workflow == 'editing':
        (workspace/'src').mkdir()
        (workspace/'.git').mkdir()
        sentinel = workspace/'wrong-ios-sibling-ran'
        files = {
            'src/lib.rs': '// owned-ios-parity-source\n#[cfg(test)]\nmod cases {\n #[test]\n fn test_selected(){\n  let selected_value=41;\n  assert_eq!(selected_value,41); // owned-ios-parity-stop\n }\n #[test]\n fn test_sibling(){std::fs::write('+json.dumps(str(sentinel))+',b"wrong").unwrap();}\n}\n',
            'Cargo.toml': '[workspace]\n[package]\nname="owned-ios-parity"\nversion="0.1.0"\nedition="2024"\n',
        }
        if slow_debug:
            files['build.rs'] = 'fn main() { std::thread::sleep(std::time::Duration::from_secs(33)); }\n'
    elif workflow == 'workspace':
        collector = load_script('test-collector-hosts')
        (workspace/'.hick-cache').mkdir()
        (workspace/'.venv').symlink_to(python_env.resolve(), target_is_directory=True)
        sentinel = None
        files = {
            '.hickory-ios-workspace': 'owned-workspace-v1\n',
            'app.py': collector.APP,
            'test_app.py': collector.TEST,
            'diagnostics.c': 'int first(void) {return owned_problem_one;}\nint second(void) {return owned_problem_two;}\n',
            'pyproject.toml': '[project]\nname="ios-collector-fixture"\nversion="0.0.0"\n[tool.uv]\n[tool.pytest.ini_options]\njunit_family="xunit1"\n',
        }
    elif workflow == 'recovery':
        details = load_script('test-behavior-readings-browser').prepare(workspace)
        (workspace/'.venv').unlink()
        (workspace/'.venv').symlink_to(python_env.resolve(), target_is_directory=True)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'source.md', 'behavior.md', 'a.py', 'test_a.py', 'test_undeclared.py', 'hickory-dogfood/behavior-map.json', 'hickory-dogfood/narrative-map.json']}
        sentinel = Path(details['sibling'])
    elif workflow == 'proposal-apply':
        load_script('test-agent-proposals-browser').prepare(workspace)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'construction.md', 'note.py', 'generated.py', 'behavior.md']}
        close = '\x3c/h:doc>\n'
        files['construction.md'] = files['construction.md'].removesuffix(close) + 'Retained author context for the mounted viewport.\n' * 100 + close
        sentinel = None
    else:
        archive = load_script('test-agent-proposal-persistence')
        files = {**archive.FILES, '.hickory-proposal-archive-fixture': 'owned-proposal-archive-v1\n'}
        sentinel = None
    for name, text in files.items():
        (workspace/name).write_text(text)
    return files, sentinel

def run_workflow(workflow, device, seed, temporary_device, candidate, derived, destination, python_env, slow_debug):
    fixture = load_script('test-simple-browser')
    method, markers = WORKFLOWS[workflow]
    name = TARGET+'/UIKitParityAcceptanceTests/'+method
    with tempfile.TemporaryDirectory(prefix='ios-parity-') as temporary:
        folder = Path(temporary)
        workspace, state = folder/'workspace', folder/'state'
        files, sentinel = prepare_workflow(workflow, workspace, python_env, slow_debug)
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(state)}
        managed = load_script('test-agent-proposals-browser') if workflow == 'proposal-apply' else None
        if managed:
            settings, calls, tool_result = managed.provider_fixture(folder)
            environment['HICKORY_AGENT_TEST_UI_SETTINGS'] = str(settings)
            def replacement(line, start, end):
                return {'range': {'start': {'line': line, 'character': start}, 'end': {'line': line, 'character': end}}, 'newText': '41'}
            edit = {'changes': {
                'hick:///note.py': [replacement(0, 6, 7)],
                'hick:///construction.md': [replacement(2, 12, 13)],
            }}
            calls.write_text(json.dumps({'intention': managed.intention(edit)}))
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            port = reserve.getsockname()[1]
        command = [os.environ.get('HICKORY_SIMPLE_TEST_ENGINE', str(ROOT/'target/source-edit-focus/debug/examples/engine_client')), str(workspace), 'window', f'http://127.0.0.1:{port}']
        archive = load_script('test-agent-proposal-persistence') if workflow == 'proposal' else None
        metadata = {}
        if workflow == 'editing' and slow_debug:
            metadata['HICKORY_IOS_PARITY_SLOW_DEBUG'] = '1'
            markers = [*markers, 'slow-debug-admission-passed']
        if archive:
            first = fixture.start(command, env=environment)
            try:
                original = archive.call(fixture.origin(first), 'POST', '/api/agent-proposals/prepare', archive.intention())
                assert original['durability']['retained'] and not original['historical']
            finally:
                archive.stop_engine(fixture, first, state)
            metadata = {'HICKORY_IOS_PROPOSAL_TOKEN': original['token'], 'HICKORY_IOS_PROPOSAL_FINGERPRINT': original['fingerprint']}
        engine = fixture.start(command, env=environment)
        try:
            origin = fixture.origin(engine)
            if archive:
                catalogue = archive.call(origin, 'GET', '/api/agent-proposals')
                assert catalogue['workspace'] != original['workspace'], 'Expected a new owned engine lifetime'
            value = plistlib.loads(candidate.read_bytes())
            target, = test_targets(value)
            for key in ['EnvironmentVariables', 'TestHostEnvironmentVariables']:
                target.setdefault(key, {}).update(HICKORY_IOS_PARITY_ORIGIN=origin, HICKORY_API_ORIGIN=origin, **metadata)
            receipt = ROOT/'target/ios-parity'
            receipt.mkdir(parents=True, exist_ok=True)
            prefix = 'last' if workflow == 'editing' else workflow
            result = receipt/f'{prefix}.xcresult'
            if result.exists():
                shutil.rmtree(result)
            log = receipt/f'{prefix}.log'
            configured = derived/'Build/Products/owned-ios-parity.xctestrun'
            configured.write_bytes(plistlib.dumps(value))
            try:
                with log.open('w') as output:
                    run(['xcodebuild', 'test-without-building', '-xctestrun', str(configured), '-destination', destination, '-parallel-testing-enabled', 'NO', '-maximum-concurrent-test-simulator-destinations', '1', '-only-testing:'+name, '-resultBundlePath', str(result), '-test-timeouts-enabled', 'YES', '-maximum-test-execution-time-allowance', '180'], stdout=output, stderr=subprocess.STDOUT)
                summary = json.loads(subprocess.check_output(['xcrun', 'xcresulttool', 'get', 'test-results', 'summary', '--path', str(result)]))
                (receipt/f'{prefix}-summary.json').write_text(json.dumps(summary, indent=2))
                assert summary['result'] == 'Passed' and summary['totalTestCount'] == 1 and summary['passedTests'] == 1 and summary['failedTests'] == summary['skippedTests'] == 0, summary
                for path, text in files.items():
                    assert (workspace/path).read_bytes() == text.encode(), path
                if sentinel:
                    assert not sentinel.exists(), sentinel.name
                if workflow == 'workspace':
                    events = (workspace/'.hick-cache/executions').read_text().splitlines()
                    assert sorted(events) == ['negative', 'positive', 'positive'], events
                if managed:
                    response = json.loads(tool_result.read_text())['result']
                    assert not response['isError'], response
                    prepared = json.loads(response['content'][0]['text'])
                    assert prepared['pending_review'] and not any(prepared[key] for key in ['applied', 'saved', 'checks_executed'])
                    retained = managed.api(origin, 'GET', '/api/agent-proposals/'+prepared['token'])
                    assert retained['fingerprint'] == prepared['fingerprint']
                    report = retained['host_application']
                    assert report['status'] == 'host-reported draft application' and not report['saved'] and not report['checks_executed']
                    assert report['after_hashes'] == {
                        'note.py': hashlib.sha256(b'value=41\n').hexdigest(),
                        'construction.md': hashlib.sha256(files['construction.md'].replace('generated = 1', 'generated = 41').encode()).hexdigest(),
                    }
                observed = log.read_text()
                for marker in markers:
                    assert observed.count('HickoryUIKitParity '+marker) == 1, marker
                payload = {
                    'workflow': workflow, 'device': device, 'seed_device': seed, 'temporary_device': temporary_device,
                    'test': name, 'summary': summary, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
                    'source_unchanged': True, 'asserted_milestones': markers, 'keyboard_input_verified': False,
                    'physical_input_verified': False, 'accessibility_verified': False,
                }
                if sentinel:
                    payload['sibling_excluded'] = True
                if workflow == 'editing':
                    payload.update(native_paste_undo_redo_verified=True, coordinated_cleanup_undo_redo_verified=True)
                (receipt/f'{prefix}-receipt.json').write_text(json.dumps(payload, indent=2))
            finally:
                configured.unlink(missing_ok=True)
        finally:
            if archive:
                archive.stop_engine(fixture, engine, state)
            else:
                fixture.stop(engine)
    print(f'PASS mounted UIKit {workflow}', flush=True)

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
 parser=argparse.ArgumentParser();parser.add_argument('--device',required=True);parser.add_argument('--build',action='store_true');parser.add_argument('--boot',action='store_true');parser.add_argument('--temporary-device',action='store_true');parser.add_argument('--minimum-free-mib',type=int,default=2048)
 parser.add_argument('--workflow', choices=WORKFLOWS, action='append', help='Repeat to use one simulator for several focused cases')
 parser.add_argument('--python-env', type=Path, default=ROOT/'target/thread-debug-env')
 parser.add_argument('--slow-debug', action='store_true', help='Delay the owned Rust fixture to exercise staged admission')
 args=parser.parse_args()
 if args.slow_debug and args.workflow and 'editing' not in args.workflow:
  parser.error('--slow-debug requires the editing workflow')
 if args.minimum_free_mib<128:parser.error('Minimum free space must remain at least128MiB')
 require_space(args.minimum_free_mib)
 devices=json.loads(subprocess.check_output(['xcrun','simctl','list','devices','-j']))
 existing_ids={d['udid'].lower()for group in devices['devices'].values()for d in group}
 selected=[(runtime,d)for runtime,group in devices['devices'].items()for d in group if d['udid']==args.device and d.get('isAvailable')is True]
 if len(selected)!=1:raise RuntimeError('Choose exactly one installed available simulator UDID')
 runtime,seed=selected[0]
 if not runtime.startswith('com.apple.CoreSimulator.SimRuntime.iOS-'):raise RuntimeError('Choose an installed iOS simulator')
 if args.temporary_device and not args.boot:parser.error('--temporary-device requires explicit --boot after resource coordination')
 device=args.device;owned_device=None;booted_here=False;failures=[]
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
   if len(targets)==1:candidates.append(path)
  if len(candidates)!=1:raise RuntimeError('Build exactly one hosted parity test configuration with --build; no launch-only acceptance is inferred')
  for workflow in dict.fromkeys(args.workflow or ['editing']):
   try:
    require_space(args.minimum_free_mib)
    run_workflow(workflow, device, args.device, owned_device is not None, candidates[0], derived, destination, args.python_env, args.slow_debug)
   except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError) as error:
    print(f'FAIL mounted UIKit {workflow}: {error}', flush=True)
    failures.append(workflow)
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
 if failures:raise RuntimeError('Unaccepted iOS workflows: '+', '.join(failures))
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

Before staged admission, Apple and Android's 30-second client lease covered
the complete startup. Individual Rust preparation operations could take up
to 120 seconds, and build events arrived after startup completed. Both early
mounted passes used that original lease.

## Exercise the staged Rust admission and other mounted workflows

The staged admission follow-up built the hosted bundle on Xcode 27 and used an
owned iPhone 18 Pro with iOS 27.0. The deliberate Cargo delay produced
`test_preparing` at 0.09188 seconds and `started` at 42.04880 seconds after
submission. The selected saved Rust test paused at its source breakpoint,
continued to retained exit 0 and excluded its sibling. Paste and guarded
Cleanup Undo/Redo, editor identity, selection, viewport, frame and saved bytes
also passed. This bundle used the staged Swift model before the later
malformed-JSON guard edit. The exact editing case passed with zero skips.
Its log SHA-256 is `4b313eb7430cd255443b2e0db56603afb737412ad7782e33e34f6caa2eb0556f`.

The separate workspace case passed with zero skips. Search opened the exact
saved position and Back and Forward returned to the mounted editors.
Apple clangd 21.0.0 supplied two current Problems locations. Next and Previous
reached them, then the original location refused after an unsaved edit. The
source-adjacent pytest 9.2.0 Run selected one case and retained exit 0. The local
WKWebView collected coverage with pytest-cov 7.1.0, displayed the deliberate
failure, opened its bound source position and retired coverage marks after
source changed. The host runner checked retained saved bytes and exact test
execution counts. Its log SHA-256 is
`a3cde19504acdc528dd25ed92ee10a6b835c03ddacd10501821269a052d3d211`.

The separate historical proposal case passed with zero skips. An owned engine
restart preserved the original proposal seal, and the mounted editor held an
independent dirty draft while reading the historical explanation. Draft Apply
and Save remained unavailable. Original editor identity, selection, Undo and
the saved source were checked. Its log SHA-256 is
`fc356bfb554ed070e460c4f59c1ba92db8cccb2e99d0096c52b04511dbeae9bf`.

The first mounted recovery attempt read the bound construction, checked its
related test and process-only receipt, rejected an obsolete reading and
restored a separate draft with the captured selection and scroll. The case
failed when the original widget's vertical offset changed from 90 to 850.33
after restoring the split. The final original Undo assertion had no accepted
result. A diagnostic replay waited for UIKit keyboard dismissal. The original
editor still had y=90 and width 181.67 after the review appeared. Attaching the
horizontal split narrowed it to 23 points and moved it to y=845.67. Its Paste
undo attempt retired history despite the same displayed draft.

The compact follow-up opens recovered copies as tabs in the existing code
pane and applies plain Paste history through its guarded text storage
replacements. The mounted recovery case passed with zero skips. The pane
topology and original editor identity stayed intact. The original widget
kept its 181.67-point width, y=90 scroll and first visible UTF-16 offset 57
from checkpoint to review, copy attachment and original tab activation.
The independently addressed copy mounted its captured selection, scroll and
dirty text. It held the saved baseline separately and lacked a save guard.
Original Paste Undo restored the exact source, Redo restored the exact draft
and the saved file stayed unchanged. The bound graph read, stale check
refusal and selected pytest process-only result passed again. Its log SHA-256
is `6150203073c2e026e10a829231aec5b95ac6cab4946c18e832d519c595e2cfc0`.

The same final build reran slow editing with the strict Swift preparation
parser and exact text-storage Paste path. Its one selected XCTest passed with
zero skips. Preparation was acknowledged at 0.21530 seconds and the saved
Rust session started at 40.98233 seconds. The original line breakpoint,
Continue, retained exit 0, sibling exclusion, native Paste and workspace
Cleanup history, saved bytes and editor state all passed. Its log SHA-256 is
`7baa1fac9d5168b9cdd0354ba82fbe90884a79957ecd73d4bfd031af6965f65f`.
The runner shut down and deleted the temporary simulator after the two cases.

The first managed Apply case reached a real sealed two-source review through the
owned ACP child. It applied the reviewed drafts, then one coordinated Undo and
Redo changed both drafts together. The original UIKit instances, frame and
engine reads of both saved sources passed. XCTest failed the original selections at offset
2 when UIKit moved them to offsets 8 and 92 after reflecting the batch. The
runner did not reach its independent post-pass file/report checks. It shut down
and deleted the temporary simulator. The failed run log SHA-256 is
`b9ea79967cee640d8801367b71312332745655ca777caa095c59f34cf9ea42d7`.

The UIKit draft reflection now maps the current UTF-16 selection through the
guarded replacements and keeps its scroll offset. The replay passed one mounted
XCTest with zero skips. Its sealed review showed the exact before and after bytes
for both drafts, their generated consequence, the unverified claim and declared
check. Apply, coordinated Undo and Redo preserved both UIKit editor instances
and the original code frame. The note caret stayed at UTF-16 offset 2. The
three-code-unit construction selection shifted by one on Apply and Redo and
returned to its original position on Undo. The construction viewport kept its
nonzero scroll offset through Apply, Undo and reattachment. Independent runner
reads verified both saved sources and the generated file byte for byte. The
retained acknowledgement reported the exact two resulting draft hashes and
reported that it neither saved files nor ran checks. Its log SHA-256 is
`8d47d9ca8a53cf225a89db3e20169f243ad3ed80799ecebc119ed759dd20da44`.

The same final build reran the mounted editing case, one pass and zero skips.
Native Paste and coordinated Cleanup each passed Undo and Redo. The saved Rust
Debug case acknowledged preparation, stopped at the selected line, continued and
retained exit 0. The sibling remained excluded, the saved bytes were unchanged
and the code frame retained the exact same pixel coordinates. Its log SHA-256
is `63a8e9528cff5d066f6adae29668d4bc9dd313f62a38502cdf6ffe28fe897679`.
The runner shut down and deleted its owned simulator after both cases.

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
