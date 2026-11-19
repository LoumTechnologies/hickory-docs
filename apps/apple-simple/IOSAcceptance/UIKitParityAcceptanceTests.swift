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

    convenience init(marker: String, expected: String, path: String) async throws {
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
        try self.init(workspace: workspace)
    }

    init(workspace: Workspace, launchFile: String? = nil) throws {
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        previous = scene.windows.first { $0.isKeyWindow }
        self.workspace = workspace
        host = UIHostingController(rootView: WorkspaceView(workspace: workspace, launchFile: launchFile))
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

        let sealed = try await readManagedProposal(in: mounted)
        let review = workspace.agentProposals
        XCTAssertTrue(sealed.textOnly)
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

    private func readManagedProposal(in mounted: MountedIOSWorkspace) async throws -> SealedAgentProposal {
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let _: ResourceJSON = try await engine.request("POST", "/api/docs/workspace/agent/acp", body: ["backend": "owned-proposal"])
        let _: ResourceJSON = try await engine.request("POST", "/api/docs/workspace/agent", body: [
            "backend": "owned-proposal",
            "prompt": "Prepare the owned intention for review. Do not apply, save or execute it.",
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
        XCTAssertFalse(sealed.envelope.source_effects)
        XCTAssertFalse(sealed.envelope.checks_executed)
        return sealed
    }

    func testMountedManagedProposalPublishesReviewedResource() async throws {
        let original = "note.py", destination = "moved λ%2F?#.py"
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-native-agent-proposal-fixture", expected: "owned-native-agent-proposal-v1\n", path: original)
        defer { mounted.close() }
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let buffer = try XCTUnwrap(workspace.currentBuffer), source = buffer.text
        let editor = try await mounted.editor(buffer)
        editor.selectedRange = NSRange(location: 2, length: 2)
        let selection = editor.selectedRange, frame = editor.convert(editor.bounds, to: mounted.window)
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let history = try XCTUnwrap(editor.undoManager)
        let review = workspace.agentProposals, restoration = workspace.resourceRestoration

        func checkSaved(_ path: String, absent: String) async throws {
            let tree = try await engine.files()
            XCTAssertNil(MountedIOSWorkspace.file(absent, in: tree.tree))
            let file = try XCTUnwrap(MountedIOSWorkspace.file(path, in: tree.tree))
            let saved = try await engine.file(file)
            XCTAssertEqual(saved.content, source)
        }
        func checkEditor(_ path: String) async throws {
            let retained = try await mounted.editor(buffer)
            XCTAssertTrue(workspace.buffers[path] === buffer)
            XCTAssertTrue(retained === editor)
            XCTAssertTrue(retained.undoManager === history)
            XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
            XCTAssertEqual(retained.selectedRange, selection)
            XCTAssertEqual(retained.convert(retained.bounds, to: mounted.window), frame)
            XCTAssertEqual(buffer.text, source)
        }

        let sealed = try await readManagedProposal(in: mounted)
        XCTAssertFalse(sealed.textOnly)
        XCTAssertEqual(Set(sealed.patches.map(\.path)), Set([original, destination]))
        XCTAssertTrue(review.canApply, review.notice)
        XCTAssertFalse(review.canSave)
        await review.prepareSave(workspace)
        XCTAssertTrue(review.canSave, review.notice)
        try await checkSaved(original, absent: destination)
        await review.save(workspace)
        XCTAssertEqual(restoration.confirmedReceipt?.adoption_current, true, restoration.notice)
        XCTAssertTrue(review.phase.hasPrefix("Saved confirmed namespace"), review.notice)
        XCTAssertFalse(review.hasReport, review.notice)
        try await checkSaved(destination, absent: original)
        review.close()
        try await mounted.wait("Proposal review did not dismiss") { mounted.host.presentedViewController == nil }
        try await checkEditor(destination)
        NSLog("HickoryUIKitParity managed-resource-publication-passed")

        try mounted.command("workspace.undo")
        try await mounted.wait("Workspace Undo did not prepare a resource review") { restoration.review != nil }
        try await checkSaved(destination, absent: original)
        await restoration.publish(workspace)
        XCTAssertEqual(restoration.confirmedReceipt?.adoption_current, true, restoration.notice)
        try await checkSaved(original, absent: destination)
        try await checkEditor(original)
        NSLog("HickoryUIKitParity managed-resource-inverse-passed")
    }
}

extension UIKitParityAcceptanceTests {
    func testMountedClangdQuickFixReviewsAndReplaysDraft() async throws {
        let path = "main.cpp"
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-ios-quickfix", expected: "owned-quickfix-v1\n", path: path)
        defer { mounted.close() }
        let workspace = mounted.workspace
        let engine = try XCTUnwrap(workspace.engine)
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let original = buffer.text
        let fixed = original.replacingOccurrences(of: "square(3) }", with: "square(3); }")
        XCTAssertNotEqual(original, fixed)
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        try await mounted.wait("clangd missing-semicolon diagnostic unavailable") {
            workspace.problems.contains { $0.current && $0.path == path && $0.location?.line == 1 }
        }
        let problem = try XCTUnwrap(workspace.problems.first { $0.current && $0.path == path && $0.location?.line == 1 })
        await workspace.openProblem(problem)
        let location = try XCTUnwrap(problem.location)
        let diagnosticOffset = try XCTUnwrap(languageOffset(original, ["line": location.line, "character": location.character]))
        try await mounted.wait("Diagnostic did not navigate to the mounted editor") { editor.selectedRange.location == diagnosticOffset }
        // The known clangd quick fix accepts the point inside the diagnostic range.
        let actionOffset = try XCTUnwrap(languageOffset(original, ["line": 1, "character": 27]))
        let selection = NSRange(location: actionOffset, length: 0)
        _ = buffer.selectSourceRange?(selection, original)
        try await mounted.wait("Diagnostic action selection did not attach") { buffer.selection == selection && editor.selectedRange == selection }
        try mounted.command("symbol.actions")
        await workspace.changes.prepare(workspace)
        let index = try XCTUnwrap(workspace.changes.choices.firstIndex {
            ($0["kind"] as? String)?.hasPrefix("quickfix") == true && ($0["title"] as? String)?.contains("insert ';'") == true
        }, workspace.changes.notice + " " + String(describing: workspace.changes.choices.map { $0["title"] }))
        await workspace.changes.choose(index, workspace: workspace, expectedSnapshot: workspace.changes.actionSnapshot)
        let plan = try XCTUnwrap(workspace.changes.plan, workspace.changes.notice)
        XCTAssertEqual(plan.patches.count, 1)
        XCTAssertEqual(plan.patches.first?.path, path)
        XCTAssertEqual(plan.patches.first?.before, original)
        XCTAssertEqual(plan.patches.first?.after, fixed)
        XCTAssertEqual(editor.text, original)
        XCTAssertEqual(buffer.selection, selection)
        let before = try await engine.file(buffer.file)
        XCTAssertEqual(before.content, original)
        NSLog("HickoryUIKitParity clangd-quickfix-preview-passed")

        workspace.changes.apply(workspace)
        try await mounted.wait("Quick Fix did not apply to mounted draft") { buffer.text == fixed && editor.text == fixed && buffer.dirty }
        workspace.changes.open("undo", workspace: workspace)
        try await mounted.wait("Quick Fix Undo did not restore original draft") { buffer.text == original && editor.text == original && !buffer.dirty }
        XCTAssertEqual(editor.selectedRange, selection)
        workspace.changes.open("redo", workspace: workspace)
        try await mounted.wait("Quick Fix Redo did not restore corrected draft") { buffer.text == fixed && editor.text == fixed && buffer.dirty }
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertTrue(workspace.currentBuffer === buffer)
        let after = try await engine.file(buffer.file)
        XCTAssertEqual(after.content, before.content)
        XCTAssertEqual(after.hash, before.hash)
        NSLog("HickoryUIKitParity clangd-quickfix-history-passed")
    }
}

extension UIKitParityAcceptanceTests {
    func testMountedTypeScriptExtractionReplaysExactDraft() async throws {
        let path = "crlf-extract.ts"
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-refactor-fixture", expected: "owned-refactor-v1\n", path: path)
        defer { mounted.close() }
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let saved = buffer.text
        XCTAssertTrue(saved.contains("\r\n"))
        XCTAssertTrue(saved.contains("😀"))

        let suffix = "// retained dirty\n"
        let draft = saved + suffix
        mounted.paste(suffix, into: editor)
        try await mounted.wait("Native Paste did not reach the CRLF editor") {
            exactUTF8(buffer.text, draft) && exactUTF8(editor.text, draft)
        }
        mounted.window.endEditing(true)
        try await mounted.wait("Source did not release keyboard") { !editor.isFirstResponder }
        let selection = (draft as NSString).range(of: #""Hello " + name"#)
        XCTAssertNotEqual(selection.location, NSNotFound)
        XCTAssertTrue(buffer.selectSourceRange?(selection, draft) == true)
        try await mounted.wait("Extraction range did not reach the editor") { editor.selectedRange == selection }

        try mounted.command("symbol.actions")
        let changes = workspace.changes
        let title = "Extract to inner function in function 'greetCR'"
        for _ in 0..<100 {
            await changes.prepare(workspace)
            if changes.choices.contains(where: { $0["title"] as? String == title }) { break }
            try await Task.sleep(for: .milliseconds(100))
        }
        let index = try XCTUnwrap(changes.choices.firstIndex { $0["title"] as? String == title }, changes.notice)
        await changes.choose(index, workspace: workspace, expectedSnapshot: changes.actionSnapshot)
        let plan = try XCTUnwrap(changes.plan, changes.notice)
        XCTAssertEqual(plan.patches.count, 1)
        let patch = try XCTUnwrap(plan.patches.first)
        XCTAssertEqual(patch.path, path)
        XCTAssertGreaterThan(patch.edits.count, 1)
        XCTAssertTrue(exactUTF8(patch.before, draft))
        XCTAssertTrue(patch.after.contains("function newFunction"))
        XCTAssertTrue(exactUTF8(editor.text, draft))

        changes.apply(workspace)
        XCTAssertTrue(exactUTF8(buffer.text, patch.after), changes.notice)
        XCTAssertTrue(exactUTF8(editor.text, patch.after), changes.notice)
        changes.open("undo", workspace: workspace)
        XCTAssertTrue(exactUTF8(buffer.text, draft), changes.notice)
        XCTAssertTrue(exactUTF8(editor.text, draft), changes.notice)
        changes.open("redo", workspace: workspace)
        XCTAssertTrue(exactUTF8(buffer.text, patch.after), changes.notice)
        XCTAssertTrue(exactUTF8(editor.text, patch.after), changes.notice)
        let retained = try await mounted.editor(buffer)
        XCTAssertTrue(retained === editor)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        let disk = try await engine.file(buffer.file)
        XCTAssertTrue(exactUTF8(disk.content, saved))
        NSLog("HickoryUIKitParity typescript-extraction-exact-history-passed")
    }

    func testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics() async throws {
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let path = try XCTUnwrap(environment["HICKORY_IOS_STARTUP_PATH"])
        let release = try XCTUnwrap(environment["HICKORY_STARTUP_DIAGNOSTIC_GATE"].flatMap(URL.init(string:)))
        let keyboard = HostedKeyboardObservation()
        let notifications = [UIResponder.keyboardWillShowNotification, UIResponder.keyboardDidShowNotification,
                             UIResponder.keyboardDidHideNotification]
        let observers = notifications.map { name in
            NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) { [weak keyboard] note in
                MainActor.assumeIsolated { keyboard?.record(note) }
            }
        }
        defer { observers.forEach { NotificationCenter.default.removeObserver($0) } }
        let workspace = Workspace()
        workspace.address = origin
        let mounted = try MountedIOSWorkspace(workspace: workspace, launchFile: path)
        defer { mounted.close() }
        try await mounted.wait("The launch file did not become the active source") {
            exactUTF8(workspace.currentBuffer?.file.path, path) && !workspace.connecting
        }
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let source = buffer.text
        XCTAssertTrue(source.hasPrefix("# owned startup capability fixture\n"))
        XCTAssertEqual(editor.text, source)
        XCTAssertTrue(editor.isEditable)
        XCTAssertEqual(Set(workspace.files.map(\.path)), Set([path, "nested/sibling.txt"]))
        XCTAssertNotNil(MountedIOSWorkspace.file("outside.txt", in: workspace.rootFiles))
        XCTAssertEqual(workspace.layout.groups().map(\.id), ["files-group", "document-group"])
        XCTAssertTrue(workspace.layout.nodes["files-group"]?.tabs.containsUtility(.files) == true)
        XCTAssertEqual(workspace.layout.nodes["document-group"]?.tabs.count, 1)
        XCTAssertFalse(workspace.problems.contains { $0.message.contains("startup diagnostic") })
        NSLog("HickoryUIKitParity requested-source-before-provider")

        XCTAssertTrue(editor.becomeFirstResponder())
        let insertion = "# sampled-draft\n"
        let destination: UIPasteConfigurationSupporting = editor
        editor.selectedRange = NSRange(location: 0, length: 0)
        destination.paste?(itemProviders: [NSItemProvider(object: insertion as NSString)])
        let draft = insertion + source
        try await mounted.wait("Native Paste did not stay in the draft") { exactUTF8(buffer.text, draft) }
        let expectedHide = keyboard.hidden + (keyboard.visible ? 1 : 0)
        mounted.window.endEditing(true)
        try await mounted.wait("Keyboard did not settle before sampling scroll") {
            !editor.isFirstResponder && !keyboard.visible && keyboard.hidden >= expectedHide
        }
        let clock = ContinuousClock()
        let settle = clock.now + .milliseconds(500)
        try await mounted.wait("Keyboard transition did not retire") {
            clock.now >= settle && !editor.isFirstResponder && !keyboard.visible
        }
        let selected = NSRange(location: draft.utf16.count / 3, length: 5)
        editor.selectedRange = selected
        editor.layoutIfNeeded()
        editor.setContentOffset(CGPoint(x: 0, y: 320), animated: false)
        let viewport = editor.contentOffset
        XCTAssertGreaterThan(viewport.y, 0)
        let history = try XCTUnwrap(editor.undoManager)
        XCTAssertTrue(history.canUndo)

        XCTAssertFalse(workspace.problems.contains { $0.message.contains("startup diagnostic") })
        var request = URLRequest(url: release)
        request.httpMethod = "POST"
        let (_, released) = try await URLSession.shared.data(for: request)
        XCTAssertEqual((released as? HTTPURLResponse)?.statusCode, 204)
        try await mounted.wait("Current diagnostic did not reach the source editor") {
            workspace.problems.contains { exactUTF8($0.path, path) && $0.current && $0.message == "fresh startup diagnostic" }
                && exactUTF8(buffer.inlineMarkup.source, draft)
        }
        XCTAssertFalse(workspace.problems.contains { $0.message == "stale startup diagnostic" })
        let retained = try await mounted.editor(buffer)
        XCTAssertTrue(retained === editor)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(editor.selectedRange, selected)
        XCTAssertEqual(editor.contentOffset.x, viewport.x, accuracy: 1)
        XCTAssertEqual(editor.contentOffset.y, viewport.y, accuracy: 1)
        XCTAssertTrue(exactUTF8(editor.text, draft))
        XCTAssertTrue(buffer.dirty)
        history.undo()
        try await mounted.wait("Undo lost the pre-arrival edit transaction") { exactUTF8(editor.text, source) && !buffer.dirty }
        history.redo()
        try await mounted.wait("Redo lost the pre-arrival edit transaction") { exactUTF8(editor.text, draft) && buffer.dirty }
        await workspace.refresh()
        XCTAssertEqual(Set(workspace.files.map(\.path)), Set([path, "nested/sibling.txt"]))
        XCTAssertTrue(workspace.currentBuffer === buffer)
        NSLog("HickoryUIKitParity delayed-diagnostic-kept-editor-state")
    }

    func testMountedRequestedTextPrecedesTreeAuthority() async throws {
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let path = try XCTUnwrap(environment["HICKORY_OPEN_FILE"])
        let release = try XCTUnwrap(environment["HICKORY_STARTUP_AUTHORITY_GATE"].flatMap(URL.init(string:)))
        let mode = try XCTUnwrap(environment["HICKORY_STARTUP_AUTHORITY_MODE"])
        let liveRoot = environment["HICKORY_IOS_STARTUP_REAL_ROOT"]
        let workspace = Workspace()
        workspace.address = origin
        let mounted = try MountedIOSWorkspace(workspace: workspace, launchFile: path)
        defer { mounted.close() }
        func releaseGate(_ url: URL) async throws {
            var request = URLRequest(url: url)
            request.httpMethod = "POST"
            let (_, response) = try await URLSession.shared.data(for: request)
            XCTAssertEqual((response as? HTTPURLResponse)?.statusCode, 204)
        }
        let treeFirst = mode == "tree-first-retry"
        if treeFirst {
            try await mounted.wait("Files did not arrive ahead of the held source reads") {
                workspace.engine != nil && Set(workspace.files.map(\.path)) == [path, "nested/sibling.txt"]
            }
            XCTAssertNil(workspace.currentBuffer)
            try await releaseGate(release)
        }
        try await mounted.wait("Requested source did not mount while authority was held") {
            exactUTF8(workspace.currentBuffer?.file.path, path)
        }
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let source = buffer.text, identity = try XCTUnwrap(buffer.sourceEditorState?()?.identity)
        XCTAssertTrue(workspace.connecting)
        XCTAssertEqual(workspace.engine != nil, treeFirst)
        XCTAssertTrue(buffer.readOnly)
        XCTAssertFalse(editor.isEditable)
        XCTAssertNil(buffer.savedGuard)
        let selection = NSRange(location: 42, length: 5)
        editor.selectedRange = selection
        editor.layoutIfNeeded()
        try await mounted.wait("Local markup did not attach independently of tree authority") {
            exactUTF8(buffer.inlineMarkup.source, source)
        }
        editor.setContentOffset(CGPoint(x: 0, y: 260), animated: false)
        let viewport = editor.contentOffset, bounds = editor.convert(editor.bounds, to: mounted.window)
        XCTAssertGreaterThan(viewport.y, 0)
        NSLog(treeFirst ? "HickoryUIKitParity requested-text-before-guarded-authority" : "HickoryUIKitParity requested-text-before-tree-authority")

        try await releaseGate(treeFirst ? release.appendingPathComponent("guard") : release)
        try await mounted.wait("Workspace authority reply did not settle") {
            !workspace.connecting && (mode != "confirmed" || editor.isEditable)
        }
        let retained = try await mounted.editor(buffer)
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertTrue(retained === editor)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertTrue(exactUTF8(editor.text, source))
        XCTAssertEqual(editor.selectedRange, selection)
        XCTAssertEqual(editor.contentOffset.x, viewport.x, accuracy: 1)
        XCTAssertEqual(editor.contentOffset.y, viewport.y, accuracy: 1)
        XCTAssertEqual(editor.convert(editor.bounds, to: mounted.window), bounds)
        if mode != "confirmed" {
            XCTAssertTrue(buffer.readOnly)
            XCTAssertFalse(editor.isEditable)
            XCTAssertNil(buffer.savedGuard)
            XCTAssertTrue(workspace.status.contains(treeFirst ? "Source permission is unconfirmed" : "remains readable"))
        }
        if mode == "retry" || treeFirst {
            if treeFirst {
                NSLog("HickoryUIKitParity missing-startup-guard-kept-editor-read-only")
                await workspace.refresh()
            } else {
                let connected = await workspace.connect()
                XCTAssertTrue(connected, workspace.status)
            }
            try await mounted.wait("Retry did not confirm the reading already on screen") { editor.isEditable }
            let retried = try await mounted.editor(buffer)
            XCTAssertTrue(workspace.currentBuffer === buffer)
            XCTAssertTrue(retried === editor)
            XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
            XCTAssertEqual(editor.selectedRange, selection)
            XCTAssertEqual(editor.contentOffset.x, viewport.x, accuracy: 1)
            XCTAssertEqual(editor.contentOffset.y, viewport.y, accuracy: 1)
            NSLog("HickoryUIKitParity retried-tree-authority-in-existing-editor")
        }
        if mode != "unavailable" {
            XCTAssertFalse(buffer.readOnly)
            XCTAssertNil(buffer.startupObservation)
            if let liveRoot {
                XCTAssertEqual(workspace.sourceRootAddress, liveRoot)
                let workspaceID = try XCTUnwrap(workspace.sourceWorkspaceID)
                let guardBytes = try XCTUnwrap(buffer.savedGuard)
                XCTAssertFalse(guardBytes.isEmpty)
                NSLog("HickoryUIKitParity live-startup-source-sha256=\(nativeSourceHash(source)) guard-sha256=\(nativeSourceHash(guardBytes)) workspace=\(workspaceID)")
            } else {
                XCTAssertEqual(buffer.savedGuard, "owned-startup-guard")
            }
            XCTAssertEqual(Set(workspace.files.map(\.path)), [path, "nested/sibling.txt"])
            XCTAssertTrue(editor.becomeFirstResponder())
            mounted.paste("draft", into: editor)
            try await mounted.wait("The confirmed source did not admit native Paste") {
                buffer.dirty && exactUTF8(editor.text, buffer.text)
            }
            let draft = buffer.text
            let history = try XCTUnwrap(editor.undoManager)
            history.undo()
            try await mounted.wait("Undo did not restore the identical startup bytes") {
                exactUTF8(editor.text, source) && !buffer.dirty
            }
            if treeFirst {
                XCTAssertTrue(history.canRedo)
                history.redo()
                try await mounted.wait("Redo did not restore the first authorized draft") {
                    exactUTF8(editor.text, draft) && exactUTF8(buffer.text, draft) && buffer.dirty
                }
                history.undo()
            }
        }
        NSLog("HickoryUIKitParity late-tree-authority-kept-editor-state")
    }
}

extension UIKitParityAcceptanceTests {
    func testMountedManagedTypeScriptMovePublishesExactHistory() async throws {
        let path = "notes.md", destination = "relocate.ts"
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-managed-move-fixture", expected: "owned-managed-move-v1\n", path: path)
        defer { mounted.close() }
        let workspace = mounted.workspace, engine = try XCTUnwrap(workspace.engine)
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let history = try XCTUnwrap(editor.undoManager)
        let original = buffer.text
        let declaration = #"function relocate(name: string) { return "Hello " + name; }"#
        let imported = #"import { relocate } from "./relocate";"#
        let importedSelection = imported + "\n"
        let call = #"console.log(relocate("Ada"));"#
        let targetSource = "export " + declaration + "\n"
        let block = declaration + "\n" + call
        let replacedBlock = imported + "\n\n" + call
        let marker = "// retained dirty\n"
        XCTAssertEqual(original.components(separatedBy: block).count, 2)
        XCTAssertTrue(original.contains("The authored prose after the declaration remains."))
        let draft = original + marker
        let moved = original.replacingOccurrences(of: block, with: replacedBlock) + marker
        mounted.paste(marker, into: editor)
        try await mounted.wait("Native Paste did not retain the managed draft") {
            buffer.dirty && exactUTF8(buffer.text, draft) && exactUTF8(editor.text, draft)
        }
        mounted.window.endEditing(true)
        try await mounted.wait("Source did not release the keyboard") { !editor.isFirstResponder }
        let selection = (draft as NSString).range(of: declaration)
        XCTAssertNotEqual(selection.location, NSNotFound)
        XCTAssertTrue(buffer.selectSourceRange?(selection, draft) == true)
        try await mounted.wait("Move selection did not reach the native editor") { editor.selectedRange == selection }
        let frame = editor.convert(editor.bounds, to: mounted.window)
        let restoration = workspace.resourceRestoration

        func checkEditor(_ source: String, selected: String, dirty: Bool) async throws {
            let retained = try await mounted.editor(buffer)
            let state = try XCTUnwrap(buffer.sourceEditorState?())
            let expected = (source as NSString).range(of: selected)
            XCTAssertNotEqual(expected.location, NSNotFound)
            XCTAssertTrue(workspace.currentBuffer === buffer)
            XCTAssertTrue(workspace.buffers[path] === buffer)
            XCTAssertTrue(retained === editor)
            XCTAssertTrue(retained.undoManager === history)
            XCTAssertEqual(state.identity, identity)
            XCTAssertEqual(retained.convert(retained.bounds, to: mounted.window), frame)
            XCTAssertTrue(exactUTF8(retained.text, source))
            XCTAssertTrue(exactUTF8(buffer.text, source))
            XCTAssertEqual(retained.selectedRange, expected)
            XCTAssertEqual(state.selection, expected)
            XCTAssertEqual(buffer.dirty, dirty)
        }
        func checkSaved(_ source: String, created: Bool) async throws {
            let files = try await engine.files()
            let file = try XCTUnwrap(MountedIOSWorkspace.file(path, in: files.tree))
            let note = try await engine.file(file)
            XCTAssertTrue(exactUTF8(note.content, source))
            let destinationFile = MountedIOSWorkspace.file(destination, in: files.tree)
            if created {
                let saved = try await engine.file(try XCTUnwrap(destinationFile))
                XCTAssertTrue(exactUTF8(saved.content, targetSource))
            } else {
                XCTAssertNil(destinationFile)
            }
        }
        func checkReview(from: String, to: String, creates: Bool, history direction: String?) throws {
            let review = try XCTUnwrap(restoration.review, restoration.notice)
            XCTAssertEqual(review.history?.direction, direction)
            XCTAssertEqual(review.save != nil, direction == nil)
            XCTAssertEqual(Set(review.patches.map(\.path)), Set([path, destination]))
            let note = try XCTUnwrap(review.patches.first { $0.path == path })
            let target = try XCTUnwrap(review.patches.first { $0.path == destination })
            XCTAssertTrue(exactUTF8(note.before?.text, from))
            XCTAssertTrue(exactUTF8(note.after?.text, to))
            XCTAssertEqual(target.before?.text, creates ? nil : targetSource)
            XCTAssertEqual(target.after?.text, creates ? targetSource : nil)
        }
        func publish() async throws -> String {
            await restoration.publish(workspace)
            XCTAssertEqual(restoration.confirmedReceipt?.adoption_current, true, restoration.notice)
            return try XCTUnwrap(restoration.lastRecovery, restoration.notice)
        }

        try mounted.command("symbol.actions")
        let changes = workspace.changes
        let title = "Move to a new file"
        for _ in 0..<100 {
            await changes.prepare(workspace)
            if changes.choices.contains(where: { $0["title"] as? String == title }) { break }
            try await Task.sleep(for: .milliseconds(100))
        }
        let index = try XCTUnwrap(changes.choices.firstIndex { $0["title"] as? String == title }, changes.notice)
        await changes.choose(index, workspace: workspace, expectedSnapshot: changes.actionSnapshot)
        try checkReview(from: draft, to: moved, creates: true, history: nil)
        try await checkSaved(original, created: false)
        try await checkEditor(draft, selected: declaration, dirty: true)
        NSLog("HickoryUIKitParity managed-typescript-move-preview-passed")

        let saved = try await publish()
        try await checkSaved(moved, created: true)
        try await checkEditor(moved, selected: importedSelection, dirty: false)
        NSLog("HickoryUIKitParity managed-typescript-move-save-passed")

        try mounted.command("workspace.undo")
        try await mounted.wait("Workspace Undo did not prepare a managed resource review") { restoration.review != nil }
        try checkReview(from: moved, to: draft, creates: false, history: "undo")
        try await checkSaved(moved, created: true)
        try await checkEditor(moved, selected: importedSelection, dirty: false)
        let inverse = try await publish()
        XCTAssertNotEqual(inverse, saved)
        try await checkSaved(original, created: false)
        try await checkEditor(draft, selected: declaration, dirty: true)
        NSLog("HickoryUIKitParity managed-typescript-move-inverse-passed")

        try mounted.command("workspace.redo")
        try await mounted.wait("Workspace Redo did not prepare a managed resource review") { restoration.review != nil }
        try checkReview(from: draft, to: moved, creates: true, history: "redo")
        try await checkSaved(original, created: false)
        try await checkEditor(draft, selected: declaration, dirty: true)
        let redo = try await publish()
        XCTAssertNotEqual(redo, inverse)
        try await checkSaved(moved, created: true)
        try await checkEditor(moved, selected: importedSelection, dirty: false)
        NSLog("HickoryUIKitParity managed-typescript-move-redo-passed")
    }

    func testMountedManagedTypeScriptDefinitionRestoresCarets() async throws {
        let destination = "relocate.ts"
        let mounted = try await MountedIOSWorkspace(marker: ".hickory-managed-move-fixture", expected: "owned-managed-move-definition-v1\n", path: "notes.md")
        defer { mounted.close() }
        let workspace = mounted.workspace
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try await mounted.editor(buffer)
        let source = buffer.text
        let targetSource = #"export function relocate(name: string) { return "Hello " + name; }"# + "\n"
        XCTAssertFalse(buffer.dirty)
        XCTAssertTrue(source.contains(#"import { relocate } from "./relocate";"#))
        XCTAssertTrue(source.hasSuffix("// retained dirty\n"))
        let callRange = (source as NSString).range(of: #"relocate("Ada")"#)
        guard callRange.location != NSNotFound else { throw EngineError(message: "The saved authored call is missing") }
        let callCaret = NSRange(location: callRange.location + 3, length: 0)
        XCTAssertEqual(callCaret.location, 179)
        XCTAssertTrue(buffer.selectSourceRange?(callCaret, source) == true)
        try await mounted.wait("The call caret did not reach the native editor") {
            editor.selectedRange == callCaret && buffer.selection == callCaret
        }
        let language = try XCTUnwrap(buffer.language)
        let declarationCaret = NSRange(location: "export function ".utf16.count, length: 0)
        func destinationResult() -> SemanticLocation? {
            language.locations.first {
                $0.path == destination && $0.line == 0 && $0.character == declarationCaret.location
            }
        }
        for _ in 0..<5 {
            try mounted.command("semantic.definition")
            try await Task.sleep(for: .milliseconds(150))
            try await mounted.wait("TypeScript did not finish Go to Definition") {
                !language.locationStatus.isEmpty && language.locationStatus != "Searching…"
            }
            if destinationResult() != nil { break }
        }
        let location = try XCTUnwrap(destinationResult(), language.locationStatus)
        XCTAssertTrue(exactUTF8(language.locationSource, source))
        XCTAssertEqual(language.locationSelection, callCaret)
        language.open(location)
        try await mounted.wait("Definition did not reveal the moved declaration") {
            workspace.currentBuffer?.file.path == destination
                && workspace.currentBuffer?.sourceEditorState?()?.selection == declarationCaret
        }
        let target = try XCTUnwrap(workspace.currentBuffer)
        let targetEditor = try await mounted.editor(target)
        XCTAssertTrue(exactUTF8(targetEditor.text, targetSource))
        XCTAssertEqual(targetEditor.selectedRange, declarationCaret)
        XCTAssertFalse(target.dirty)

        let returnRange = (targetSource as NSString).range(of: "return")
        guard returnRange.location != NSNotFound else { throw EngineError(message: "The moved declaration has no return statement") }
        let updatedTargetCaret = NSRange(location: returnRange.location, length: 0)
        XCTAssertEqual(updatedTargetCaret.location, 41)
        XCTAssertTrue(target.selectSourceRange?(updatedTargetCaret, targetSource) == true)
        try await mounted.wait("The updated declaration caret did not reach the native editor") {
            targetEditor.selectedRange == updatedTargetCaret && target.selection == updatedTargetCaret
        }
        try mounted.command("navigation.back")
        try await mounted.wait("Back did not restore the authored call caret") {
            workspace.currentBuffer === buffer && buffer.sourceEditorState?()?.selection == callCaret
        }
        let caller = try await mounted.editor(buffer)
        XCTAssertTrue(exactUTF8(caller.text, source))
        XCTAssertEqual(caller.selectedRange, callCaret)
        try mounted.command("navigation.forward")
        try await mounted.wait("Forward did not restore the updated declaration caret") {
            workspace.currentBuffer === target && target.sourceEditorState?()?.selection == updatedTargetCaret
        }
        let declarationEditor = try await mounted.editor(target)
        XCTAssertTrue(exactUTF8(declarationEditor.text, targetSource))
        XCTAssertEqual(declarationEditor.selectedRange, updatedTargetCaret)
        NSLog("HickoryUIKitParity managed-typescript-move-definition-history-passed")
    }
}

extension UIKitParityAcceptanceTests {
    private func mountRecordedTurn(_ id: String) async throws -> (MountedIOSWorkspace, AgentTurn) {
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let root = try XCTUnwrap(environment["HICKORY_IOS_RECORDED_ROOT"])
        let workspace = Workspace()
        workspace.address = origin
        guard await workspace.connect(), exactUTF8(workspace.sourceRootAddress, root) else {
            throw EngineError(message: "Refuse an unrelated recorded-message workspace · " + workspace.status)
        }
        workspace.showUtility(.agent)
        let mounted = try MountedIOSWorkspace(workspace: workspace)
        do {
            try await mounted.wait("The retained Agent turn did not mount") {
                workspace.agent.turns.contains { $0.id == id && $0.status == "ok" && $0.answer != nil }
            }
            return (mounted, try XCTUnwrap(workspace.agent.turns.first { $0.id == id }))
        } catch {
            mounted.close()
            throw error
        }
    }

    func testMountedRecordedCodeFencesPauseStepWatchAndContinue() async throws {
        let (mounted, turn) = try await mountRecordedTurn("recorded-fence-host")
        defer { mounted.close() }
        let workspace = mounted.workspace, agent = workspace.agent
        let debug = try XCTUnwrap(workspace.debugger)
        var sent: [[String: Any]] = []
        debug.observeRequestForTest = { sent.append($0) }
        let messages = [(RecordedMessageSide.user, turn.prompt, "42"),
                        (.assistant, try XCTUnwrap(turn.answer), "53")]
        for (side, text, expected) in messages {
            let buffer = agent.document("\(turn.id)-\(side.bufferSuffix)", text)
            let editor = try await mounted.editor(buffer)
            let identity = try XCTUnwrap(buffer.sourceEditorState?()?.identity)
            XCTAssertFalse(editor.isEditable)
            try await mounted.wait("Recorded code fence did not become available") {
                buffer.inlineMarkup.debug.recorded[2] != nil
            }
            let fence = try XCTUnwrap(buffer.inlineMarkup.debug.recorded[2])
            let sibling = try XCTUnwrap(buffer.inlineMarkup.debug.recorded[7])
            let requestStart = sent.count
            debug.toggle(buffer, line: fence.opening + 1)
            debug.toggle(buffer, line: sibling.opening + 1)
            XCTAssertEqual(sent.count, requestStart)
            let source = NativeRecordedMessageSource(turn: turn.id, side: side, workspace: workspace.sourceRootAddress) {
                agent.owns(buffer, turn: turn.id, side: side)
            }
            await debug.prepareRecordedFence(buffer, source: source, fence: fence)
            try await mounted.wait("Recorded \(side.rawValue) fence did not pause · " + debug.message) {
                debug.status == "paused" || debug.status == "failed"
            }
            guard debug.status == "paused" else { throw EngineError(message: debug.message) }
            XCTAssertEqual(debug.line(in: buffer.file.path), 3)
            XCTAssertTrue(debug.sourceMatches(buffer))
            let admission = sent.dropFirst(requestStart).first
            let breakpoints = admission?["breakpoints"] as? [[String: Any]]
            XCTAssertEqual(breakpoints?.compactMap { $0["line"] as? Int }, [3])
            debug.step("over")
            try await mounted.wait("Step did not reach the next authored line") { debug.line(in: buffer.file.path) == 4 }
            debug.evaluate("value + 1", watch: true)
            try await mounted.wait("Watch did not read the selected stopped frame") { debug.values["value + 1"] == expected }
            debug.step("continue")
            try await mounted.wait("Continue did not finish the selected fence with its output") {
                debug.status == "finished" && debug.output.map(\.text).joined().contains(expected)
            }
            XCTAssertTrue(exactUTF8(editor.text, text))
            XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
            XCTAssertFalse(buffer.dirty)
            NSLog("HickoryUIKitParity recorded-code-\(side.rawValue)-pause-step-watch-continue")
        }
        XCTAssertEqual(sent.filter { $0["op"] as? String == "start" }.count, 2)
    }

    func testMountedRecordedPatchesReviewApplyAndUndo() async throws {
        let (mounted, turn) = try await mountRecordedTurn("recorded-patch")
        defer { mounted.close() }
        let workspace = mounted.workspace, agent = workspace.agent, review = workspace.agentProposals
        let engine = try XCTUnwrap(workspace.engine)
        let messages = [
            (RecordedMessageSide.user, turn.prompt, "author.md", "value = 1", "value = 41"),
            (.assistant, try XCTUnwrap(turn.answer), "review.txt", "saved note", "assistant draft"),
        ]
        for (side, text, path, before, after) in messages {
            let buffer = agent.document("\(turn.id)-\(side.bufferSuffix)", text)
            let editor = try await mounted.editor(buffer)
            let identity = try XCTUnwrap(buffer.sourceEditorState?()?.identity)
            XCTAssertFalse(editor.isEditable)
            try await mounted.wait("Recorded \(side.rawValue) patch did not become available") {
                buffer.inlineMarkup.debug.recordedPatches.contains(2)
            }
            let source = NativeRecordedMessageSource(turn: turn.id, side: side, workspace: workspace.sourceRootAddress) {
                agent.owns(buffer, turn: turn.id, side: side)
            }
            let file = try XCTUnwrap(workspace.workspaceFile(path))
            let saved = try await engine.file(file).content
            XCTAssertTrue(saved.contains(before))
            let expected = saved.replacingOccurrences(of: before, with: after)
            await review.prepareRecordedPatch(workspace, message: buffer, source: source, openingLine: 2)
            let sealed = try XCTUnwrap(review.value, review.notice)
            XCTAssertTrue(review.showing)
            XCTAssertEqual(sealed.material.origin_authority, "recorded \(side.rawValue) fence, source SHA-256 \(nativeSourceHash(text)); claims remain unverified")
            XCTAssertEqual(review.phase, "Reviewed only")
            XCTAssertTrue(review.canApply, review.notice)
            let target = try XCTUnwrap(workspace.buffers[path])
            XCTAssertTrue(exactUTF8(target.text, saved))
            XCTAssertFalse(target.dirty)
            if side == .user {
                XCTAssertTrue(sealed.sections.contains { $0.text.contains("generated.py") })
            }
            await review.applyDraft(workspace)
            XCTAssertTrue(exactUTF8(target.text, expected), review.notice)
            XCTAssertTrue(target.dirty)
            XCTAssertFalse(review.canApply)
            let stillSaved = try await engine.file(file).content
            XCTAssertTrue(exactUTF8(stillSaved, saved))
            review.close()
            workspace.changes.close(workspace)
            await Task.yield()
            workspace.changes.open("undo", workspace: workspace)
            XCTAssertTrue(exactUTF8(target.text, saved), workspace.changes.notice)
            XCTAssertFalse(target.dirty)
            workspace.changes.close(workspace)
            XCTAssertTrue(exactUTF8(editor.text, text))
            XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
            XCTAssertFalse(buffer.dirty)
            NSLog("HickoryUIKitParity recorded-patch-\(side.rawValue)-review-apply-undo")
        }
    }
}
