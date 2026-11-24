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

The refresh case mounts a clean saved buffer and supplies a newer disk revision
through the normal file response. The File command should move the caret with
the original text, keep the buffer clean, and leave template feedback empty.

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

private final class UIKitRefreshResponse: URLProtocol {
    static let path = "Export.kt"
    static let workspaceID = String(repeating: "a", count: 32)
    static let original = "package journal\n\nfun export() = Unit\n"
    static let prefix = "// External line\n"
    static let changed = prefix + original

    override class func canInit(with request: URLRequest) -> Bool { request.url?.host == "uikit-refresh.invalid" }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        guard let url = request.url else { return }
        let body: [String: Any]
        let status: Int
        switch url.path {
        case "/api/files":
            body = ["root": "owned", "root_path": "/owned", "workspace_id": Self.workspaceID,
                    "tree": [["name": Self.path, "path": Self.path, "dir": false]]]
            status = 200
        case "/api/file":
            body = ["path": Self.path, "content": Self.changed, "hash": "new", "save_guard": "new"]
            status = 200
        default:
            body = ["error": "Optional capability unavailable in the refresh fixture"]
            status = 404
        }
        do {
            let data = try JSONSerialization.data(withJSONObject: body)
            let response = try XCTUnwrap(HTTPURLResponse(url: url, statusCode: status, httpVersion: nil, headerFields: nil))
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data)
            client?.urlProtocolDidFinishLoading(self)
        } catch {
            client?.urlProtocol(self, didFailWithError: error)
        }
    }
    override func stopLoading() {}
}

extension UIKitParityAcceptanceTests {
    func testMountedSavedRefreshKeepsCaretAndTemplateStatus() async throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [UIKitRefreshResponse.self]
        let session = URLSession(configuration: configuration)
        defer { session.invalidateAndCancel() }
        let workspace = Workspace()
        let path = UIKitRefreshResponse.path
        let file = FileNode(name: path, path: path, dir: false)
        let source = FileText(path: path, content: UIKitRefreshResponse.original, hash: "old", save_guard: "old")
        let buffer = DocumentBuffer(file: file, text: source)
        workspace.engine = try Engine(address: "http://uikit-refresh.invalid", session: session)
        workspace.sourceRootAddress = "/owned"
        workspace.sourceWorkspaceID = UIKitRefreshResponse.workspaceID
        workspace.rootFiles = [file]
        workspace.buffers[path] = buffer
        workspace.showSource(path)
        let mounted = try MountedIOSWorkspace(workspace: workspace)
        defer { mounted.close() }
        let editor = try await mounted.editor(buffer)
        let initial = NSRange(location: 0, length: 0)
        XCTAssertEqual(editor.selectedRange, initial)
        XCTAssertTrue(buffer.snippets.notice.isEmpty)

        try mounted.command("file.refresh")
        try await mounted.wait("The saved source did not reload") {
            buffer.base == UIKitRefreshResponse.changed && editor.text == UIKitRefreshResponse.changed
        }
        let expected = NSRange(location: UIKitRefreshResponse.prefix.utf16.count, length: 0)
        XCTAssertEqual(editor.selectedRange, expected)
        XCTAssertEqual(buffer.selection, expected)
        XCTAssertFalse(buffer.dirty)
        XCTAssertTrue(buffer.snippets.notice.isEmpty)
    }

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
</ui:copy>

## Publish a managed proposal through the shared resource review

Both managed cases submit the mounted editor's original context to the owned
ACP tool and wait for its sealed review. For a rename, preparing the guarded
Save must leave the old path readable and the destination absent. The confirmed
publication must retarget the same UIKit editor. Workspace Undo reviews and
publishes the inverse before the old address becomes available again.

<ui:copy id="ios-mounted-agent-resource-publication">

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
</ui:copy>

## Review a genuine clangd Quick Fix before editing the draft

The fixture starts with a missing semicolon. Open the exact authored Problem,
then select the provider's Quick Fix. Review must expose the full source change
while leaving the current draft and disk alone. Verify that Apply and
coordinated Undo and Redo update the same mounted editor. The saved text and
hash must agree before and after the draft history.

<ui:copy id="ios-mounted-clangd-quickfix">

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
</ui:copy>

## Replay the exact draft from a multi-edit TypeScript refactoring

Use the shared TypeScript refactoring fixture with CRLF and supplementary
Unicode. Retain a dirty Paste before resolving the provider's extraction.
After Apply, compare the native editor and model byte for byte against the
reviewed patch. Undo must reach the original dirty draft so the workspace
history can still authorize Redo. Compare the saved source independently.

Mount the production startup view with a requested file under a nested folder
before a controlled provider publishes anything. The provider withholds
diagnostics until it receives the first draft, then sends both its retired
version and its current version. The first open must show the file in the
existing editor beside its sibling in Files. Current positional diagnostics
attach to that same editor while native Paste Undo and Redo retain their
pre-arrival transaction. The full tree still provides workspace navigation.

<ui:copy id="ios-mounted-typescript-history">

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
</ui:copy>

## Move a declaration out of an authored TypeScript block

Open the managed Markdown source in UIKit Source mode and Paste a dirty marker
outside the literal. Ask the installed TypeScript server to move the selected
declaration. Compare the exact reviewed draft and destination before publishing,
and check the saved namespace independently at every step. Save and each
reviewed history action must retain the same native editor, undo manager and
frame. The selection follows the replaced line between declaration and import,
while the surrounding Markdown and dirty marker survive in the draft.
The navigation case opens those exact published bytes in a fresh workspace.
It queries from the Markdown call, opens the provider's workspace result and
checks that Back and Forward restore the carets in both native editors,
including a caret moved inside the destination after opening it.

<ui:copy id="ios-mounted-managed-typescript-move">

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
</ui:copy>

## Follow an immutable Agent message into Debug or patch review

Each report starts with the durable turn loaded by the real workspace engine and
waits for the production card's native editor. Debug gives the selected fence
and its sibling different breakpoint intentions. Only the selected fence may
start, pause and produce output. The patch case reads the complete seal before
applying, checks the generated consequence of the authored change, and undoes
the resulting draft batch. The host separately compares every fixture file
against its original bytes.

<ui:copy id="ios-recorded-agent-fences">

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
</ui:copy>
<ui:file path="apps/apple-simple/IOSAcceptance/UIKitParityAcceptanceTests.swift"><ui:paste select="#ios-mounted-parity-test"/><ui:paste select="#ios-mounted-agent-resource-publication"/><ui:paste select="#ios-mounted-clangd-quickfix"/><ui:paste select="#ios-mounted-typescript-history"/><ui:paste select="#ios-mounted-managed-typescript-move"/><ui:paste select="#ios-recorded-agent-fences"/></ui:file>

## Run each saved fence through the installed app

The UI test opens Agent from the View menu and taps Run within each recorded message. It checks the completed terminal through WebKit accessibility and compares the text and frame of the same recorded editor after dismissal. The runner independently checks terminal results and the original archive bytes.

The tab close test uses native keyboard input, the tab's close button and all
three alert choices. Reopening through Files distinguishes a discarded draft
from a saved one. The runner starts with an empty file and independently checks
that the only final bytes are the text submitted through Save.

The View toggle witness uses the native menu twice around an unsaved edit.
The editor widens and returns to its original frame with the same draft.

<ui:copy id="ios-recorded-run-ui-test">
import XCTest

final class RecordedRunUITests: XCTestCase {
    @MainActor func testViewTogglePreservesEditorAndItsDraft() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let path = environment["HICKORY_IOS_STARTUP_PATH"] ?? "note.txt"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = path
        app.launch()

        let editor = app.textViews[path].firstMatch
        let viewMenu = app.buttons["View"].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 20))
        let width = editor.frame.width
        editor.tap()
        editor.typeText("nested-draft")
        let draft = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(draft.contains("nested-draft"))
        func toggle() {
            viewMenu.tap()
            let item = app.buttons["Toggle Files pane"].firstMatch
            XCTAssertTrue(item.waitForExistence(timeout: 5))
            item.tap()
        }

        toggle()
        XCTAssertTrue(editor.waitForExistence(timeout: 6))
        XCTAssertGreaterThan(editor.frame.width, width + 50)
        XCTAssertEqual(editor.value as? String, draft)
        toggle()
        XCTAssertTrue(editor.waitForExistence(timeout: 6))
        XCTAssertEqual(editor.frame.width, width, accuracy: 2)
        XCTAssertEqual(editor.value as? String, draft)

    }

    @MainActor func testNativeDirtyTabCloseChoices() throws {
        continueAfterFailure = false
        let origin = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PARITY_ORIGIN"])
        let path = "note.txt"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = path
        app.launch()

        let editor = app.textViews[path].firstMatch
        let close = app.buttons["Close \(path) •"].firstMatch
        let alert = app.alerts.firstMatch
        func choose(_ choice: String) {
            XCTAssertTrue(close.waitForExistence(timeout: 15))
            close.tap()
            XCTAssertTrue(alert.waitForExistence(timeout: 10))
            for action in ["Save", "Discard Changes", "Cancel"] {
                XCTAssertTrue(alert.buttons[action].exists, action)
            }
            alert.buttons[choice].tap()
            XCTAssertTrue(alert.waitForNonExistence(timeout: 10))
        }
        func edit(_ text: String) {
            XCTAssertTrue(editor.waitForExistence(timeout: 15))
            editor.tap()
            editor.typeText(text)
            XCTAssertEqual(editor.value as? String, text)
        }

        edit("discarded draft")
        choose("Cancel")
        XCTAssertEqual(editor.value as? String, "discarded draft")
        choose("Discard Changes")
        XCTAssertTrue(editor.waitForNonExistence(timeout: 10))
        app.buttons[path].firstMatch.tap()
        edit("saved draft")
        choose("Save")
        XCTAssertTrue(editor.waitForNonExistence(timeout: 15))
        app.buttons[path].firstMatch.tap()
        XCTAssertTrue(editor.waitForExistence(timeout: 10))
        XCTAssertEqual(editor.value as? String, "saved draft")
        NSLog("HickoryUIKitParity native-dirty-tab-close-choices")
    }

    @MainActor func testRecordedUserAndAssistantRunsShowTerminalWithoutChangingSource() throws {
        continueAfterFailure = false
        let origin = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PARITY_ORIGIN"])
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launch()

        let view = app.buttons["View"].firstMatch
        XCTAssertTrue(view.waitForExistence(timeout: 20))
        view.tap()
        app.buttons["Agent"].firstMatch.tap()
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 20))

        let cases = [("user", "user", "print(value + 1)", "value = 41", "42"),
                     ("assistant", "agent", "print(value + 1)", "value = 52", "53")]
        for (side, buffer, statement, input, expected) in cases {
            let card = conversation.descendants(matching: .any)["Recorded \(side) message"].firstMatch
            let editor = card.textViews["hick-agent-message:///recorded-fence-host-\(buffer).md"].firstMatch
            let title = "Run recorded python fence on line 3"
            let run = card.buttons[title].firstMatch
            let overflow = card.buttons["More gutter icons on line 3"].firstMatch
            for _ in 0..<5 where !run.isHittable && !overflow.isHittable {
                if !editor.exists { _ = editor.waitForExistence(timeout: 3) }
                if run.isHittable || overflow.isHittable { break }
                let start = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.78))
                let end = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.28))
                start.press(forDuration: 0.04, thenDragTo: end)
            }
            XCTAssertTrue(editor.exists, "Recorded \(side) source is unavailable")
            let before = try XCTUnwrap(editor.value as? String)
            XCTAssertTrue(before.contains(statement) && before.contains(input))
            let frame = editor.frame
            if run.isHittable {
                run.tap()
            } else {
                XCTAssertTrue(overflow.isHittable, "Recorded \(side) Run gutter is unavailable")
                overflow.tap()
                let more = app.descendants(matching: .any)["More actions on line 3"].firstMatch
                let revealed = more.buttons[title].firstMatch
                XCTAssertTrue(revealed.waitForExistence(timeout: 10))
                revealed.tap()
            }

            let terminal = app.descendants(matching: .any)["Recorded fence terminal"].firstMatch
            XCTAssertTrue(terminal.waitForExistence(timeout: 20))
            let web = terminal.webViews.firstMatch
            XCTAssertTrue(web.waitForExistence(timeout: 15))
            XCTAssertFalse(web.frame.isEmpty)
            let line = "\\s*\(expected)\\s*"
            let output = web.descendants(matching: .any).matching(NSPredicate(format: "label MATCHES %@ OR value MATCHES %@", line, line)).firstMatch
            XCTAssertTrue(output.waitForExistence(timeout: 15), "The \(side) terminal did not expose output \(expected)")
            XCTAssertTrue(terminal.staticTexts["Exit code 0"].waitForExistence(timeout: 15))
            let screenshot = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
            screenshot.name = "Recorded \(side) Run output \(expected)"
            screenshot.lifetime = .keepAlways
            add(screenshot)
            terminal.buttons["Close"].tap()
            XCTAssertTrue(terminal.waitForNonExistence(timeout: 10))
            XCTAssertEqual(editor.value as? String, before)
            XCTAssertEqual(editor.frame.integral, frame.integral)
            NSLog("HickoryUIKitParity recorded-run-\(side)-terminal-kept-editor")
        }
    }

    @MainActor func testRecordedConstructionShowsOnlyTheSelectedLocalTransition() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let side = try XCTUnwrap(environment["HICKORY_IOS_CONSTRUCTION_SIDE"])
        XCTAssertTrue(side == "user" || side == "assistant")
        let buffer = side == "user" ? "user" : "agent"
        let document = side == "user" ? "request.md" : "response.md"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launch()
        let view = app.buttons["View"].firstMatch
        XCTAssertTrue(view.waitForExistence(timeout: 20))
        view.tap()
        app.buttons["Agent"].firstMatch.tap()
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 20))
        let card = conversation.descendants(matching: .any)["Recorded \(side) message"].firstMatch
        let editor = card.textViews["hick-agent-message:///declared-construction-\(buffer).md"].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 15))
        let source = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(source.contains("<hick:patch id=\"selected\">"))
        let frame = editor.frame
        let action = "Inspect recorded construction patch on line 3"
        let inspect = card.buttons[action].firstMatch
        if inspect.isHittable {
            inspect.tap()
        } else {
            let overflow = card.buttons["More gutter icons on line 3"].firstMatch
            XCTAssertTrue(overflow.waitForExistence(timeout: 15))
            overflow.tap()
            app.buttons[action].firstMatch.tap()
        }
        let reading = app.descendants(matching: .any)["Recorded construction reading"].firstMatch
        XCTAssertTrue(reading.waitForExistence(timeout: 20))
        XCTAssertTrue(reading.staticTexts["\(document) · selected"].waitForExistence(timeout: 10))
        let output = reading.descendants(matching: .any)["materialized.txt"].firstMatch
        XCTAssertTrue(output.waitForExistence(timeout: 15))
        output.tap()
        let row = { (text: String) in reading.staticTexts.matching(NSPredicate(format: "label MATCHES %@", "\\s*\(text)\\s*")).firstMatch }
        XCTAssertTrue(row("old").waitForExistence(timeout: 10))
        XCTAssertTrue(row("middle").waitForExistence(timeout: 10))
        XCTAssertFalse(row("final").exists)
        XCTAssertFalse(reading.buttons.matching(NSPredicate(format: "label MATCHES %@", "(?i).*(Apply|Save|Run).*")).firstMatch.exists)
        let screenshot = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        screenshot.name = "Recorded construction patch Before and After"
        screenshot.lifetime = .keepAlways
        add(screenshot)
        reading.buttons["Close"].tap()
        XCTAssertTrue(reading.waitForNonExistence(timeout: 10))
        XCTAssertEqual(editor.value as? String, source)
        XCTAssertEqual(editor.frame.integral, frame.integral)
        NSLog("HickoryUIKitParity recorded-construction-\(side)-read-immutable-local-transition")
    }
}
</ui:copy>
## Send a native message and replay the resulting two-source draft

Open both source files through the app before typing the exact prompt. Send must capture those named sources and their original write guards. Keep the Agent pane open. An acknowledged Send releases the cleared composer’s native focus so the conversation can show the recorded message in place. Read and apply the sealed intention in the presented review, then use Edit menu history to check both source texts and their viewport frames after Undo and Redo. The matching engine trace checks that no source or generated file was published.

<ui:copy id="ios-native-agent-send-ui-test">
extension RecordedRunUITests {
    @MainActor func testNativeAgentSendAppliesBothDraftsAndReplaysHistory() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let prompt = try XCTUnwrap(environment["HICKORY_IOS_PROPOSAL_PROMPT"])
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = "note.py"
        app.launch()

        func menu(_ name: String, _ action: String) {
            let menu = app.buttons[name].firstMatch
            XCTAssertTrue(menu.waitForExistence(timeout: 15), name)
            menu.tap()
            let item = app.buttons[action].firstMatch
            XCTAssertTrue(item.waitForExistence(timeout: 8), action)
            item.tap()
        }
        func source(_ path: String, expected: String? = nil) throws -> XCUIElement {
            let open = app.buttons.matching(NSPredicate(format: "identifier == %@ OR label == %@ OR label == %@", path, path, path + " •")).firstMatch
            let editor = app.textViews[path].firstMatch
            if !editor.exists || !editor.isHittable {
                XCTAssertTrue(open.waitForExistence(timeout: 12), path)
                open.tap()
            }
            XCTAssertTrue(editor.waitForExistence(timeout: 12), path)
            if let expected {
                let exact = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", expected), object: editor)
                XCTAssertEqual(XCTWaiter.wait(for: [exact], timeout: 8), .completed, path)
            }
            return editor
        }
        func reveal(_ element: XCUIElement, upward: Bool) {
            for _ in 0..<5 where !element.isHittable {
                if upward { app.swipeUp() } else { app.swipeDown() }
            }
            XCTAssertTrue(element.isHittable, element.debugDescription)
        }
        func acknowledgeChange(_ statusPrefix: String) {
            let preview = app.staticTexts["Workspace change preview"].firstMatch
            XCTAssertTrue(preview.waitForExistence(timeout: 12))
            let status = app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", statusPrefix)).firstMatch
            XCTAssertTrue(status.waitForExistence(timeout: 8), statusPrefix)
            app.buttons["Close"].firstMatch.tap()
            XCTAssertTrue(preview.waitForNonExistence(timeout: 10))
        }

        let note = "note.py", construction = "construction.md"
        let noteView = try source(note, expected: "value=1\n")
        let noteFrame = noteView.frame.integral
        let constructionView = try source(construction)
        let originalConstruction = try XCTUnwrap(constructionView.value as? String)
        XCTAssertTrue(originalConstruction.contains("generated = 1"))
        let constructionFrame = constructionView.frame.integral
        let changedConstruction = originalConstruction.replacingOccurrences(of: "generated = 1", with: "generated = 41")

        menu("View", "Agent")
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 15))
        let composer = app.textViews.matching(NSPredicate(format: "identifier == %@ OR label == %@ OR label == %@",
            "conversation/draft.md", "conversation/draft.md", "Message")).firstMatch
        XCTAssertTrue(composer.waitForExistence(timeout: 12))
        composer.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 8))
        composer.typeText(prompt)
        let entered = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", prompt), object: composer)
        XCTAssertEqual(XCTWaiter.wait(for: [entered], timeout: 10), .completed)
        let send = app.buttons["Send"].firstMatch
        XCTAssertTrue(send.isHittable && send.isEnabled)
        send.tap()
        let cleared = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == '' OR value == nil"), object: composer)
        XCTAssertEqual(XCTWaiter.wait(for: [cleared], timeout: 10), .completed)
        XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
        XCTAssertTrue(conversation.waitForExistence(timeout: 12))
        XCTAssertGreaterThan(conversation.frame.height, 0)
        let userCard = conversation.descendants(matching: .any)["Recorded user message"].firstMatch
        let archived = userCard.textViews.matching(NSPredicate(format: "value == %@", prompt)).firstMatch
        XCTAssertTrue(archived.waitForExistence(timeout: 20), "The exact native message was not visible in the recorded user card")
        NSLog("HickoryUIKitParity native-proposal-send-archived-exact-message")

        menu("View", "Agent proposal review")
        let review = app.staticTexts["Agent proposal review"].firstMatch
        XCTAssertTrue(review.waitForExistence(timeout: 12))
        let read = app.buttons["Read Explain and change the source together"].firstMatch
        XCTAssertTrue(read.waitForExistence(timeout: 15))
        read.tap()
        let digest = app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", "Complete SHA256 ")).firstMatch
        XCTAssertTrue(digest.waitForExistence(timeout: 15))
        let apply = app.buttons["Apply complete draft batch"].firstMatch
        reveal(apply, upward: true)
        XCTAssertTrue(apply.isEnabled)
        apply.tap()
        acknowledgeChange("Applied 2 source drafts. Not saved or verified.")
        if review.exists {
            let close = app.buttons["Close"].firstMatch
            reveal(close, upward: false)
            close.tap()
            XCTAssertTrue(review.waitForNonExistence(timeout: 10))
        }

        func drafts(noteText: String, constructionText: String) throws {
            XCTAssertEqual(try source(note, expected: noteText).frame.integral, noteFrame)
            XCTAssertEqual(try source(construction, expected: constructionText).frame.integral, constructionFrame)
        }
        try drafts(noteText: "value=41\n", constructionText: changedConstruction)
        menu("View", "Agent")
        menu("Edit", "Undo workspace change")
        acknowledgeChange("Workspace undo applied to 2 drafts. Not saved or verified.")
        try drafts(noteText: "value=1\n", constructionText: originalConstruction)
        menu("View", "Agent")
        menu("Edit", "Redo workspace change")
        acknowledgeChange("Workspace redo applied to 2 drafts. Not saved or verified.")
        try drafts(noteText: "value=41\n", constructionText: changedConstruction)
        NSLog("HickoryUIKitParity native-proposal-send-applied-and-replayed-two-drafts")
    }
}
</ui:copy>
## Review and stage the saved file without publishing the iPhone draft

Use separate texts for the commit, saved working tree and native draft. The review compares the commit and saved text. Stage uses the saved text. After Unstage, verify that the index matches the commit and the original editor keeps its exact draft and frame. Select each side at the measured center of its button. UIKit had previously delivered center taps without opening Working tree, even though an upper-right tap worked. The test reads the real index after each explicit button dispatch.

<ui:copy id="ios-native-git-review-ui-test">
extension RecordedRunUITests {
    @MainActor func testNativeGitReviewStagesOnlySavedBytesAndRetainsDraft() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let baseline = try XCTUnwrap(environment["HICKORY_IOS_GIT_BASELINE"])
        let saved = try XCTUnwrap(environment["HICKORY_IOS_GIT_SAVED"])
        let draftMarker = try XCTUnwrap(environment["HICKORY_IOS_GIT_DRAFT_MARKER"])
        let path = "note.py"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = path
        app.launch()

        func tap(_ title: String) {
            let button = app.buttons[title].firstMatch
            XCTAssertTrue(button.waitForExistence(timeout: 15), title)
            XCTAssertTrue(button.isHittable && button.isEnabled, title)
            button.tap()
        }
        func read(_ phrase: String) {
            let label = app.descendants(matching: .any).matching(NSPredicate(format: "label CONTAINS %@ OR value CONTAINS %@", phrase, phrase)).firstMatch
            XCTAssertTrue(label.waitForExistence(timeout: 12), phrase)
        }
        func diff(_ side: String) {
            let button = app.buttons["\(side) \(path)"].firstMatch
            XCTAssertTrue(button.waitForExistence(timeout: 15), side)
            XCTAssertTrue(button.isHittable && button.isEnabled, side)
            button.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5)).tap()
            let removed = "-" + baseline.trimmingCharacters(in: .newlines)
            let added = "+" + saved.trimmingCharacters(in: .newlines)
            let exact = app.descendants(matching: .any).matching(NSPredicate(format:
                "(label CONTAINS %@ AND label CONTAINS %@ AND NOT label CONTAINS %@) OR (value CONTAINS %@ AND value CONTAINS %@ AND NOT value CONTAINS %@)",
                removed, added, draftMarker, removed, added, draftMarker)).firstMatch
            XCTAssertTrue(exact.waitForExistence(timeout: 15), "The \(side) diff must show the exact saved hunk")
        }

        let editor = app.textViews[path].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 20))
        tap("View"); tap("Agent")
        tap(path)
        let loaded = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", saved), object: editor)
        XCTAssertEqual(XCTWaiter.wait(for: [loaded], timeout: 10), .completed)
        let frame = editor.frame.integral
        editor.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 8))
        editor.typeText(draftMarker)
        let draft = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(draft.contains(draftMarker)); XCTAssertNotEqual(draft, saved)
        tap("Agent")
        XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
        tap("View"); tap("Repository changes")
        read("Saved working tree and index. Unsaved editor drafts are not included.")
        diff("Working tree")
        tap("Stage saved file")
        read("Saved paths staged.")
        diff("Staged")
        tap("Unstage file")
        read("Saved paths unstaged.")
        tap("Close")
        tap(path)
        XCTAssertTrue(editor.waitForExistence(timeout: 12))
        XCTAssertEqual(editor.value as? String, draft)
        XCTAssertEqual(editor.frame.integral, frame)
        NSLog("HickoryUIKitParity native-git-review-staged-saved-only-and-retained-draft")
    }
}
</ui:copy>
## Follow saved attribution into its immutable commit

Keep one saved line identical to the commit and change another on disk. Type a third version as a native draft. Saved attribution should offer a commit only on the unchanged line. Follow that line’s commit, then read the same commit from the file history. Compare the exact commit address and historical blob with Git, and confirm that neither reading altered the index, saved file or draft.

<ui:copy id="ios-native-git-history-ui-test">
extension RecordedRunUITests {
    @MainActor func testNativeGitAttributionOpensImmutableCommitAndRetainsDraft() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let baseline = try XCTUnwrap(environment["HICKORY_IOS_GIT_BASELINE"])
        let saved = try XCTUnwrap(environment["HICKORY_IOS_GIT_SAVED"])
        let draftMarker = try XCTUnwrap(environment["HICKORY_IOS_GIT_DRAFT_MARKER"])
        let commit = try XCTUnwrap(environment["HICKORY_IOS_GIT_COMMIT"])
        let path = "note.py"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = path
        app.launch()

        func tap(_ title: String) {
            let button = app.buttons[title].firstMatch
            XCTAssertTrue(button.waitForExistence(timeout: 15), title)
            XCTAssertTrue(button.isHittable && button.isEnabled, title)
            button.tap()
        }
        func text(_ value: String) -> XCUIElement {
            app.descendants(matching: .any).matching(NSPredicate(format: "label == %@ OR value == %@", value, value)).firstMatch
        }
        func read(_ value: String) {
            XCTAssertTrue(text(value).waitForExistence(timeout: 12), value)
        }
        func readCommittedBlob() {
            read(commit)
            let lines = baseline.split(separator: "\n").map(String.init)
            XCTAssertEqual(lines.count, 2)
            let blob = app.descendants(matching: .any).matching(NSPredicate(format:
                "(label CONTAINS %@ AND label CONTAINS %@ AND NOT label CONTAINS %@) OR (value CONTAINS %@ AND value CONTAINS %@ AND NOT value CONTAINS %@)",
                lines[0], lines[1], draftMarker, lines[0], lines[1], draftMarker)).firstMatch
            XCTAssertTrue(blob.waitForExistence(timeout: 15), "The immutable reading must display both committed lines")
        }
        func leaveKeyboard() {
            tap("Agent")
            XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
            tap(path)
        }

        let editor = app.textViews[path].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 20))
        tap("View"); tap("Agent"); tap(path)
        let loaded = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", saved), object: editor)
        XCTAssertEqual(XCTWaiter.wait(for: [loaded], timeout: 10), .completed)
        let frame = editor.frame.integral
        editor.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 8))
        editor.typeText(draftMarker)
        let draft = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(draft.contains(draftMarker)); XCTAssertNotEqual(draft, saved)
        leaveKeyboard()

        tap("View"); tap("Saved file attribution")
        read("Saved attribution · \(path)")
        read("Saved-file lines only. Unsaved editor changes are not included.")
        let lines = saved.split(separator: "\n").map(String.init)
        XCTAssertEqual(lines.count, 2)
        for line in lines { read(line) }
        read("Full immutable commit reference unavailable.")
        XCTAssertFalse(app.buttons["Read commit for saved line 2"].exists)
        tap("Read commit for saved line 1")
        let attributedHistory = "Commit from saved attribution · \(path):1"
        read(attributedHistory)
        readCommittedBlob()
        tap("Close")
        XCTAssertTrue(text(attributedHistory).waitForNonExistence(timeout: 10))
        leaveKeyboard()

        tap("View"); tap("Saved file history")
        let fileHistory = "Saved history · \(path)"
        read(fileHistory)
        tap("Read commit \(commit)")
        readCommittedBlob()
        tap("Close")
        XCTAssertTrue(text(fileHistory).waitForNonExistence(timeout: 10))
        leaveKeyboard()
        XCTAssertTrue(editor.waitForExistence(timeout: 12))
        XCTAssertEqual(editor.value as? String, draft)
        XCTAssertEqual(editor.frame.integral, frame)
        NSLog("HickoryUIKitParity native-git-history-read-immutable-commit-and-retained-draft")
    }
}
</ui:copy>
## Follow a Python definition between two iPhone editors

Select the unique call to an imported function with native Find. A fresh simulator can show the keyboard’s QuickPath introduction over the editor. Dismiss that message when it appears, then submit the Find field with Return. Use Go to definition, choose the exact provider location, and walk Back and Forward through the original caller and target tabs. Check the two source contents and editor geometry after navigation. The owned transport records the real provider request and definition so the rendered result can be checked against the provider's UTF-16 coordinates.

An enabled provider result can sit below the iPhone language panel's visible
bounds. Scroll the containing panel before activating the authored declaration.
Back and Forward then verify the exact files and original editor frames.

<ui:copy id="ios-native-python-definition-ui-test">
extension RecordedRunUITests {
    @MainActor func testNativePythonDefinitionAndHistoryAcrossFiles() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let mainSource = try XCTUnwrap(environment["HICKORY_IOS_PYTHON_CALLER"])
        let helperSource = try XCTUnwrap(environment["HICKORY_IOS_PYTHON_DEFINITION"])
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = "main.py"
        app.launch()

        func tap(_ button: XCUIElement, _ description: String) {
            XCTAssertTrue(button.waitForExistence(timeout: 15), description)
            XCTAssertTrue(button.isHittable && button.isEnabled, description)
            button.tap()
        }
        func command(_ item: String, menu: String = "Edit") {
            tap(app.buttons[menu].firstMatch, menu)
            let button = app.buttons[item].firstMatch
            if menu == "Edit" && !button.isHittable {
                let list = app.collectionViews.firstMatch
                XCTAssertTrue(list.waitForExistence(timeout: 5))
                // The collection extends under the clipped menu. Start the drag inside its visible middle.
                for _ in 0..<6 where !button.isHittable {
                    let start = list.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.46))
                    let end = list.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.21))
                    start.press(forDuration: 0.05, thenDragTo: end)
                }
            }
            tap(button, item)
        }
        func tab(_ path: String) -> XCUIElement {
            let strip = app.scrollViews.containing(.button, identifier: "Close \(path)").firstMatch
            return strip.buttons[path].firstMatch
        }
        func editor(_ path: String, text: String) -> XCUIElement {
            let view = app.textViews[path].firstMatch
            XCTAssertTrue(view.waitForExistence(timeout: 15), path)
            let matches = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", text), object: view)
            XCTAssertEqual(XCTWaiter.wait(for: [matches], timeout: 8), .completed, path)
            return view
        }
        func releaseKeyboard(in path: String) {
            tap(app.buttons["Agent"].firstMatch, "Agent tab")
            XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
            tap(tab(path), path)
        }

        _ = editor("main.py", text: mainSource)
        command("Agent", menu: "View")
        tap(tab("main.py"), "main.py tab")
        let caller = editor("main.py", text: mainSource)
        let callerFrame = caller.frame.integral
        command("Find and replace")
        let find = app.textFields["Find"].firstMatch
        XCTAssertTrue(find.waitForExistence(timeout: 12))
        find.tap()
        find.typeText("greeting(")
        let entered = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", "greeting("), object: find)
        XCTAssertEqual(XCTWaiter.wait(for: [entered], timeout: 8), .completed)
        let keyboardIntro = app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", "Speed up your typing by sliding your finger")).firstMatch
        if keyboardIntro.waitForExistence(timeout: 3) {
            tap(app.buttons["Continue"].firstMatch, "Dismiss the first-use keyboard introduction")
            XCTAssertTrue(keyboardIntro.waitForNonExistence(timeout: 8))
        }
        find.tap()
        find.typeText("\n")
        tap(app.buttons["Close search"].firstMatch, "Close search with selected call")
        releaseKeyboard(in: "main.py")

        command("Go to definition")
        let target = app.buttons["helper.py:4:5"].firstMatch
        XCTAssertTrue(target.waitForExistence(timeout: 20), "Expected the real Python declaration at helper.py:4:5")
        if !target.isHittable {
            let panel = app.scrollViews.containing(.button, identifier: "helper.py:4:5").firstMatch
            XCTAssertTrue(panel.waitForExistence(timeout: 5), app.debugDescription)
            for _ in 0..<4 where !target.isHittable {
                let start = panel.coordinate(withNormalizedOffset: CGVector(dx: 0.20, dy: 0.72))
                let end = panel.coordinate(withNormalizedOffset: CGVector(dx: 0.20, dy: 0.25))
                start.press(forDuration: 0.05, thenDragTo: end)
            }
        }
        tap(target, "exact Python definition")
        _ = editor("helper.py", text: helperSource)
        releaseKeyboard(in: "helper.py")
        let helperFrame = editor("helper.py", text: helperSource).frame.integral
        XCTAssertTrue(app.textViews["main.py"].waitForNonExistence(timeout: 8))

        command("Navigate back")
        _ = editor("main.py", text: mainSource)
        releaseKeyboard(in: "main.py")
        XCTAssertEqual(editor("main.py", text: mainSource).frame.integral, callerFrame)
        XCTAssertTrue(app.textViews["helper.py"].waitForNonExistence(timeout: 8))
        command("Navigate forward")
        _ = editor("helper.py", text: helperSource)
        releaseKeyboard(in: "helper.py")
        XCTAssertEqual(editor("helper.py", text: helperSource).frame.integral, helperFrame)
        NSLog("HickoryUIKitParity native-python-definition-and-back-forward")
    }
}
</ui:copy>
The recorded debugger test presses the executable line's named breakpoint
button, checks its pending state, and verifies that the opening fence has no
breakpoint button. Both mapped stop positions precede the inspection. Tapping Expression must produce a
software keyboard and admit a typed Watch. Continue and dismissal return to
the original recorded text and frame. The same test selects each side through
the runner, which checks the saved fixture bytes after execution.

<ui:copy id="ios-recorded-debug-ui-test">
extension RecordedRunUITests {
    @MainActor func testRecordedFenceDebugThroughNativeGutter() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let side = try XCTUnwrap(environment["HICKORY_IOS_RECORDED_DEBUG_SIDE"])
        XCTAssertTrue(["user", "assistant"].contains(side))
        let buffer = side == "user" ? "user" : "agent"
        let expected = side == "user" ? "42" : "53"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launch()
        let view = app.buttons["View"].firstMatch
        XCTAssertTrue(view.waitForExistence(timeout: 20))
        view.tap()
        app.buttons["Agent"].firstMatch.tap()
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 20))

        let card = conversation.descendants(matching: .any)["Recorded \(side) message"].firstMatch
        let editor = card.textViews["hick-agent-message:///recorded-fence-host-\(buffer).md"].firstMatch
        func screenshot(_ phase: String) {
            let attachment = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
            attachment.name = "Recorded \(side) Debug \(phase)"
            attachment.lifetime = .keepAlways
            add(attachment)
        }
        func firstAction(_ line: Int) -> XCUIElement {
            let overflow = card.buttons["More gutter icons on line \(line)"].firstMatch
            let run = card.buttons["Run recorded python fence on line \(line)"].firstMatch
            return overflow.exists ? overflow : run
        }
        for _ in 0..<7 where !firstAction(3).isHittable || !firstAction(8).isHittable {
            let start = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.78))
            let end = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.37))
            if firstAction(3).isHittable && firstAction(8).isHittable { break }
            start.press(forDuration: 0.04, thenDragTo: end)
        }
        XCTAssertTrue(editor.exists)
        XCTAssertTrue(firstAction(3).isHittable && firstAction(8).isHittable, "Both real fence anchors must be on screen before tapping the line-number rail")
        let archived = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(archived.contains(side == "user" ? "value = 41" : "value = 52"))
        XCTAssertTrue(archived.contains("print(value + 1)"))
        let originalFrame = editor.frame.integral
        let breakpoint = card.buttons["Toggle breakpoint on line 4"].firstMatch
        XCTAssertTrue(breakpoint.waitForExistence(timeout: 10) && breakpoint.isHittable && breakpoint.isEnabled)
        XCTAssertEqual(breakpoint.value as? String, "No breakpoint")
        XCTAssertFalse(card.buttons["Toggle breakpoint on line 3"].exists)
        breakpoint.tap()
        let pending = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", "pending"), object: breakpoint)
        XCTAssertEqual(XCTWaiter.wait(for: [pending], timeout: 7), .completed)
        XCTAssertEqual(editor.value as? String, archived)
        XCTAssertEqual(editor.frame.integral, originalFrame)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-breakpoint-tap")
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-native-line-button-4-pending")
        screenshot("authored breakpoint")

        func tapGutter(_ label: String, line: Int) {
            let direct = card.buttons[label].firstMatch
            let more = card.buttons["More gutter icons on line \(line)"].firstMatch
            if direct.waitForExistence(timeout: 7), direct.isHittable { direct.tap(); return }
            XCTAssertTrue(more.waitForExistence(timeout: 10) && more.isHittable, "Missing native gutter action \(label)")
            more.tap()
            let overflow = app.descendants(matching: .any)["More actions on line \(line)"].firstMatch
            let revealed = overflow.buttons[label].firstMatch
            XCTAssertTrue(revealed.waitForExistence(timeout: 9) && revealed.isHittable, "Missing overflow action \(label)")
            revealed.tap()
        }
        func paused(_ line: Int) {
            let ready = card.buttons["Continue on line \(line)"].firstMatch
            XCTAssertTrue(ready.waitForExistence(timeout: 24), "No native pause controls on recorded line \(line)")
            XCTAssertTrue(ready.isHittable)
            XCTAssertEqual(editor.value as? String, archived)
            screenshot("paused on source line \(line)")
        }
        tapGutter("Debug recorded python fence on line 3", line: 3)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-launch-tap")
        paused(4)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-paused-4")
        tapGutter("Over on line 4", line: 4)
        paused(5)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-stepped-5")
        tapGutter("Inspect on line 5", line: 5)
        let atFrame = app.staticTexts["Source line 5 · paused"].firstMatch
        XCTAssertTrue(atFrame.waitForExistence(timeout: 12))
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-inspect-mapped-5")
        let expression = app.textFields["Expression"].firstMatch
        let inspector = app.scrollViews.containing(.staticText, identifier: "Source line 5 · paused").firstMatch
        XCTAssertTrue(inspector.exists, "The debugger's own scroll container is unavailable")
        let initialInspectorFrame = inspector.frame
        let keyboard = app.keyboards.firstMatch
        func screenPoint(_ point: CGPoint) -> XCUICoordinate {
            app.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: point.x - app.frame.minX, dy: point.y - app.frame.minY))
        }
        func reveal(_ element: XCUIElement, upward: Bool) {
            for _ in 0..<5 where !element.isHittable {
                let bounds = (inspector.exists ? inspector.frame : initialInspectorFrame).intersection(app.frame)
                let bottom = keyboard.exists && !keyboard.frame.isEmpty ? min(bounds.maxY, keyboard.frame.minY) : bounds.maxY
                let height = bottom - bounds.minY
                XCTAssertGreaterThan(height, 55, "The inspector has no scroll viewport above the keyboard")
                let low = CGPoint(x: bounds.midX, y: bounds.minY + height * 0.32)
                let high = CGPoint(x: bounds.midX, y: bounds.minY + height * 0.78)
                let start = screenPoint(upward ? high : low)
                let end = screenPoint(upward ? low : high)
                start.press(forDuration: 0.03, thenDragTo: end)
            }
            XCTAssertTrue(element.isHittable, element.exists ? element.debugDescription : "Debugger control is no longer mounted")
        }
        reveal(expression, upward: true)
        let inputFrame = expression.frame
        XCTAssertTrue(initialInspectorFrame.contains(CGPoint(x: inputFrame.midX, y: inputFrame.midY)))
        NSLog("HickoryUIKitParity \(side) inspector=\(initialInspectorFrame) expression-before-focus=\(inputFrame)")
        screenshot("Expression before focus")
        expression.tap()
        let keyboardOpened = keyboard.waitForExistence(timeout: 8)
        if !keyboardOpened { screenshot("Expression after focus without keyboard") }
        XCTAssertTrue(keyboardOpened, "The expression tap did not open a native keyboard")
        screenshot("native expression keyboard")
        // UIKit can retire the placeholder-labelled accessibility element as
        // the text field gains focus. Send real keys to the focused app.
        app.typeText("value + 1")
        let watch = app.buttons["Watch"].firstMatch
        reveal(watch, upward: true)
        XCTAssertTrue(watch.isEnabled, "The native expression did not enable Watch")
        watch.tap()
        let value = app.staticTexts["value + 1 = \(expected)"].firstMatch
        XCTAssertTrue(value.waitForExistence(timeout: 12), "The mapped frame did not show the expected Watch")
        reveal(value, upward: true)
        screenshot("watch \(expected)")
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-watched-\(expected)")
        let resume = app.buttons["Continue"].firstMatch
        reveal(resume, upward: false)
        resume.tap()
        let finished = app.staticTexts["Source line 5 · finished"].firstMatch
        XCTAssertTrue(finished.waitForExistence(timeout: 22))
        screenshot("continued to finished")
        // Tap outside a popover or dismiss the iPhone sheet with its ordinary
        // drag gesture before sampling the underlying recorded editor.
        if !editor.isHittable {
            let leftOfInspector = CGPoint(x: app.frame.minX + 10, y: initialInspectorFrame.midY)
            let aboveInspector = CGPoint(x: app.frame.midX, y: max(app.frame.minY + 35, initialInspectorFrame.minY - 25))
            let outside = initialInspectorFrame.minX - app.frame.minX > 25 ? leftOfInspector : aboveInspector
            XCTAssertFalse(initialInspectorFrame.contains(outside))
            screenPoint(outside).tap()
            if !editor.isHittable, finished.exists {
                let top = CGPoint(x: app.frame.midX, y: app.frame.minY + 85)
                let bottom = CGPoint(x: app.frame.midX, y: app.frame.maxY - 35)
                screenPoint(top).press(forDuration: 0.03, thenDragTo: screenPoint(bottom))
            }
        }
        XCTAssertEqual(editor.value as? String, archived)
        XCTAssertEqual(editor.frame.integral, originalFrame)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-finished-immutable")
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/IOSUIAcceptance/RecordedRunUITests.swift"><ui:paste select="#ios-recorded-run-ui-test"/><ui:paste select="#ios-native-agent-send-ui-test"/><ui:paste select="#ios-native-git-review-ui-test"/><ui:paste select="#ios-native-git-history-ui-test"/><ui:paste select="#ios-native-python-definition-ui-test"/><ui:paste select="#ios-recorded-debug-ui-test"/></ui:file>

## Run only on a deliberately selected simulator

Coordinate the disk/link window before building or booting. The runner defaults to an already booted installed simulator and an existing hosted test build. `--build` explicitly builds the one test scheme using the existing derived products; `--boot` starts the selected simulator and restores Shutdown afterward. With explicit `--temporary-device --boot`, the selected installed device is only a seed for its exact installed runtime and device type: the runner creates a fresh uniquely named device, rejects any returned UUID from the initial complete device list (including unavailable devices), uses only a genuinely new returned UUID, then shuts down and deletes only that owned device in finally. The seed and its data stay untouched; no clone, runtime download or signing change occurs. Device-owned storage is reclaimed on deletion, while global simulator/OS caches may remain. A default2GiB free-space floor guards the measured first-boot/keyboard-cache cost; a coordinated run may explicitly choose a different floor. It is an admission check, not a guarantee against unrelated concurrent disk writes. Simulator and build caches can exceed100MiB, so this must not be presented as a small launch-only probe.

After generation and resource coordination, use `DEVELOPER_DIR=/Applications/Xcode-27.0.0.app/Contents/Developer python3 scripts/test-ios-parity.py --device <installed-UDID> --build --boot --temporary-device`. Omit those flags to reuse an already built/booted target. The runner starts an isolated workspace engine, passes its address through a disposable XCTest configuration and runs exactly one named case. It admits only one passed case with zero failures/skips from the real xcresult summary, retains the result/log digest, checks saved source and sibling exclusion, and cleans up its owned runtime. A separate lifecycle receipt records the seed, actual returned device UUID and deletion outcome, including failed attempts. Compilation, invocation and physical-input acceptance must remain unverified until actually executed.

By default the runner selects `editing`. Use repeated `--workflow` flags to
reuse one owned simulator while giving each case its own workspace and exact
XCTest result. The `workspace` case requires clangd and uses the existing pytest
and pytest-cov collector fixture. The `quickfix` case uses clangd through the
production source-action adapter. The `typescript-extraction` and
`typescript-managed-move` cases require the TypeScript fixture tools under
`target/refactor-provider/node_modules`. The Move case also requires an explicit
engine binary. Its host receipt compares the published Markdown and new module
against exact fixture bytes after the mounted Save, Undo and Redo. Run
`typescript-managed-move-definition` to load those saved bytes in a separate
owned workspace. A transparent provider relay records the staged TypeScript
call position and the result that its mounted navigation commands follow.
Both `workspace` and `recovery` use the
installed environment from `--python-env`. The `proposal` case checks engine
lifetime turnover before mounting its saved historical review. The optional
`--slow-debug` fixture deliberately delays Cargo longer than 30 seconds and
requires a staged preparation acknowledgment before that threshold.

The `proposal-apply` case uses the owned scripted ACP provider and the same
native proposal review. Supply the sender's actual editors to the managed
tool, read the sealed proposal before applying its ordinary and authored
changes together, and retain a separate generated-output consequence.
One coordinated Undo and Redo must reach both drafts.
The `proposal-save` case supplies a resource rename through the same provider.
It verifies the publication and inverse inside the mounted test and checks the
restored authored namespace on the host. Pass a new `--receipt-dir` to retain
this run without replacing an earlier result.

The `startup` case starts a controlled Python LSP child in the private fixture.
It waits for an explicit test-only loopback gate after the editor has recorded
its selection and scroll with UIKit's keyboard dismissed, then pairs the current
version with the retired one so the test can distinguish delivery from a
timeout. The host also checks that both reports were emitted and the saved
bytes remain exact.
Use `--appkit-startup --receipt-dir <new-directory>` to drive the same fixture
through the mounted macOS test without operating a simulator.

Use `--appkit-startup-authority --receipt-dir <new-directory>` for the earlier
boundary. A controlled loopback protects the Markdown source and holds the file
tree. It lets the production AppKit editor select and scroll the provisional
text before sending confirmed or failed authority. A transient failure then lets
the same connection retry and confirm the reading already on screen. AppKit and
UIKit share these HTTP responses and compare each mounted editor before and after
its authority response. Select the relevant UIKit case with
`--workflow startup-authority`, `--workflow startup-authority-unavailable`
or `--workflow startup-authority-retry`.

Run `--workflow startup-authority-tree-first-retry` for the opposite order.
Files returns while both source reads are still held. Release the startup text
after the guarded request starts, select and scroll that text, and then send a
response without a save guard. Refresh Files admits the same editor after a
confirmed response. Native editing and Undo/Redo check the first draft after
permission arrives.

For the same UIKit editor check with real engine reads, supply
`HICKORY_SIMPLE_TEST_ENGINE` and select `--workflow startup-authority-live`
or `--workflow startup-authority-live-retry`. The relay delays Files and can
return one temporary failure. The runner compares the editor's source and
adopted guard fingerprints with the successful engine responses before it
accepts the mounted Paste and Undo receipt.

The recorded cases accept an explicit fixture containing `recorded-debug` and
`recorded-patch` directories. Each directory holds the corresponding files
from the shared browser replay, including the unwrapped durable turn. The
runner copies only those expected files into a disposable workspace and
reports their original source digests. Supply the actual engine with
`HICKORY_SIMPLE_TEST_ENGINE`, pass `--recorded-fixtures <directory>` and
select `--workflow recorded-run --workflow recorded-debug --workflow recorded-patch`.
Run and Debug share the same immutable recorded-code fixture. Run uses the UI
test scheme on that simulator and saves a screenshot for each completed side.
The Debug lane uses debugpy from `--python-env`. Use
`--workflow recorded-debug-ui-user` or `--workflow recorded-debug-ui-assistant`
with the recorded Debug fixture to exercise the native inspector and software
keyboard. The recorded fixture must put the selected fence and its sibling
anchor within reach in the conversation.
Use `--workflow recorded-construction` or `--workflow recorded-construction-user`
with the explicit engine to inspect the captured Before and After in the
assistant or user message. The runner creates each fixture at its exact
workspace path and checks the recorded card that owns the verified declaration.

<ui:copy id="ios-mounted-parity-runner">
"""Owned mounted UIKit parity witness. Requires an explicitly selected simulator."""
import argparse,hashlib,importlib.util,json,os,plistlib,shutil,socket,subprocess,sys,tempfile,urllib.error,urllib.parse,urllib.request,uuid
from contextlib import ExitStack, contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Event, Lock, Thread
ROOT=Path(__file__).resolve().parents[1]
TARGET = 'HickorySimpleMobileParityTests'
RECORDED_RUN_TARGET = 'HickorySimpleMobileRecordedRunTests'
SCHEMES = {TARGET: 'HickorySimpleMobileParity', RECORDED_RUN_TARGET: 'HickorySimpleMobileRecordedRun'}
PROPOSAL_DESTINATION = 'moved λ%2F?#.py'
MANAGED_MOVE_WORKFLOW = 'typescript-managed-move'
MANAGED_MOVE_DEFINITION_WORKFLOW = 'typescript-managed-move-definition'
MANAGED_MOVE_WORKFLOWS = {MANAGED_MOVE_WORKFLOW, MANAGED_MOVE_DEFINITION_WORKFLOW}
MANAGED_MOVE_DECLARATION = 'function relocate(name: string) { return "Hello " + name; }\n'
MANAGED_MOVE_LITERAL = MANAGED_MOVE_DECLARATION + 'console.log(relocate("Ada"));\n'
MANAGED_MOVE_IMPORT = 'import { relocate } from "./relocate";\n\n'
MANAGED_MOVE_MARKER = '// retained dirty\n'
TYPESCRIPT_PROVIDER_VERSIONS = {'typescript-language-server': '6.1.0', 'typescript': '6.0.3'}
STARTUP_PATH = 'nested/entry # %.py'
STARTUP_SOURCE = '# owned startup capability fixture\n' + ''.join(f'value_{i} = {i}\n' for i in range(180))
STARTUP_AUTHORITY_PATH = 'nested/entry # %.md'
GIT_REVIEW_BASELINE = 'print("committed baseline")\n'
GIT_REVIEW_SAVED = 'print("saved working tree")\n'
GIT_REVIEW_DRAFT_MARKER = '# retained local editor draft'
GIT_HISTORY_STABLE_LINE = '# stable committed source\n'
GIT_WORKFLOWS = {'git-review', 'git-history'}
PYTHON_CALLER = 'from helper import greeting\n\ngreeting("reader")\n'
PYTHON_DEFINITION = '# The provider should open the declaration below.\n\n\ndef greeting(name: str) -> str:\n    return name.upper()\n'
STARTUP_AUTHORITY_MODES = {
    'startup-authority': 'confirmed', 'startup-authority-unavailable': 'unavailable', 'startup-authority-retry': 'retry',
    'startup-authority-live': 'confirmed', 'startup-authority-live-retry': 'retry',
    'startup-authority-tree-first-retry': 'tree-first-retry',
}
LIVE_STARTUP_AUTHORITY = {'startup-authority-live', 'startup-authority-live-retry'}
RECORDED_CONSTRUCTION_SIDES = {'recorded-construction': 'assistant', 'recorded-construction-user': 'user'}
RECORDED_DEBUG_UI_SIDES = {'recorded-debug-ui-user': 'user', 'recorded-debug-ui-assistant': 'assistant'}
WORKFLOWS = {
    'editing': ('testMountedUIKitEditingAndSelectedRustDebug', ['native-paste-history-passed', 'coordinated-cleanup-history-passed', 'debug-paused', 'completed']),
    'workspace': ('testMountedSearchProblemsAndCollectedTests', ['workspace-navigation-problems-passed', 'workspace-tests-coverage-passed']),
    'quickfix': ('testMountedClangdQuickFixReviewsAndReplaysDraft', ['clangd-quickfix-preview-passed', 'clangd-quickfix-history-passed']),
    'typescript-extraction': ('testMountedTypeScriptExtractionReplaysExactDraft', ['typescript-extraction-exact-history-passed']),
    MANAGED_MOVE_WORKFLOW: ('testMountedManagedTypeScriptMovePublishesExactHistory', [
        'managed-typescript-move-preview-passed', 'managed-typescript-move-save-passed',
        'managed-typescript-move-inverse-passed', 'managed-typescript-move-redo-passed',
    ]),
    MANAGED_MOVE_DEFINITION_WORKFLOW: ('testMountedManagedTypeScriptDefinitionRestoresCarets', [
        'managed-typescript-move-definition-history-passed',
    ]),
    'recovery': ('testMountedConnectedReadingAndPrivateRecovery', ['connected-impact-passed', 'private-recovery-passed']),
    'proposal': ('testMountedHistoricalProposalRefusesApplication', ['historical-proposal-passed']),
    'proposal-apply': ('testMountedManagedProposalAppliesBothReviewedDrafts', ['managed-proposal-preview-passed', 'managed-proposal-history-passed']),
    'proposal-native-send': ('testNativeAgentSendAppliesBothDraftsAndReplaysHistory',
                             ['native-proposal-send-archived-exact-message', 'native-proposal-send-applied-and-replayed-two-drafts']),
    'proposal-save': ('testMountedManagedProposalPublishesReviewedResource', ['managed-resource-publication-passed', 'managed-resource-inverse-passed']),
    'git-review': ('testNativeGitReviewStagesOnlySavedBytesAndRetainsDraft', ['native-git-review-staged-saved-only-and-retained-draft']),
    'git-history': ('testNativeGitAttributionOpensImmutableCommitAndRetainsDraft',
                    ['native-git-history-read-immutable-commit-and-retained-draft']),
    'dirty-tab-close': ('testNativeDirtyTabCloseChoices', ['native-dirty-tab-close-choices']),
    'python-definition': ('testNativePythonDefinitionAndHistoryAcrossFiles', ['native-python-definition-and-back-forward']),
    'startup': ('testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics', ['requested-source-before-provider', 'delayed-diagnostic-kept-editor-state']),
    'startup-authority': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'late-tree-authority-kept-editor-state']),
    'startup-authority-unavailable': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'late-tree-authority-kept-editor-state']),
    'startup-authority-retry': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'retried-tree-authority-in-existing-editor', 'late-tree-authority-kept-editor-state']),
    'startup-authority-live': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'late-tree-authority-kept-editor-state']),
    'startup-authority-live-retry': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-tree-authority', 'retried-tree-authority-in-existing-editor', 'late-tree-authority-kept-editor-state']),
    'startup-authority-tree-first-retry': ('testMountedRequestedTextPrecedesTreeAuthority', ['requested-text-before-guarded-authority', 'missing-startup-guard-kept-editor-read-only', 'retried-tree-authority-in-existing-editor', 'late-tree-authority-kept-editor-state']),
    'recorded-debug': ('testMountedRecordedCodeFencesPauseStepWatchAndContinue',
                       ['recorded-code-user-pause-step-watch-continue', 'recorded-code-assistant-pause-step-watch-continue']),
    'recorded-run': ('testRecordedUserAndAssistantRunsShowTerminalWithoutChangingSource',
                     ['recorded-run-user-terminal-kept-editor', 'recorded-run-assistant-terminal-kept-editor']),
    'recorded-construction': ('testRecordedConstructionShowsOnlyTheSelectedLocalTransition',
                              ['recorded-construction-assistant-read-immutable-local-transition']),
    'recorded-construction-user': ('testRecordedConstructionShowsOnlyTheSelectedLocalTransition',
                                   ['recorded-construction-user-read-immutable-local-transition']),
    'recorded-patch': ('testMountedRecordedPatchesReviewApplyAndUndo',
                       ['recorded-patch-user-review-apply-undo', 'recorded-patch-assistant-review-apply-undo']),
}

RECORDED_FILES = {
    'recorded-debug': ('sessions/agent.md',),
    'recorded-run': ('sessions/agent.md',),
    'recorded-patch': ('project.md', 'author.md', 'generated.py', 'review.txt', 'sessions/recorded-patch.md'),
}

for workflow, side in RECORDED_DEBUG_UI_SIDES.items():
    value = {'user': '42', 'assistant': '53'}[side]
    WORKFLOWS[workflow] = ('testRecordedFenceDebugThroughNativeGutter', [
        f'recorded-debug-ui-{side}-{phase}' for phase in
        ('breakpoint-tap', 'native-line-button-4-pending', 'launch-tap', 'paused-4', 'stepped-5', f'watched-{value}', 'finished-immutable')
    ])
    RECORDED_FILES[workflow] = RECORDED_FILES['recorded-debug']

def recorded_fixture_path(folder, workflow, path):
    fixture = 'recorded-debug' if workflow == 'recorded-run' or workflow in RECORDED_DEBUG_UI_SIDES else workflow
    return folder/fixture/path

def workflow_target(workflow):
    ui_workflows = {'recorded-run', 'proposal-native-send', 'python-definition', 'dirty-tab-close'} | RECORDED_CONSTRUCTION_SIDES.keys() | RECORDED_DEBUG_UI_SIDES.keys() | GIT_WORKFLOWS
    return RECORDED_RUN_TARGET if workflow in ui_workflows else TARGET

DEFINITION_TRACE = r'''
import json, signal, subprocess, sys
from pathlib import Path
from threading import Lock, Thread

event_file = Path(sys.argv[1])
provider = subprocess.Popen(sys.argv[2:], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
definitions = set()
guard = Lock()

def terminate(*_):
    raise SystemExit

signal.signal(signal.SIGTERM, terminate)

def forward(source, target, outgoing):
    try:
        while True:
            headers = []
            length = None
            while line := source.readline():
                headers.append(line)
                if not line.strip():
                    break
                key, value = line.split(b':', 1)
                if key.strip().lower() == b'content-length':
                    length = int(value.strip())
            if length is None:
                return
            body = source.read(length)
            if len(body) != length:
                return
            message = json.loads(body)
            with guard:
                identifier = message.get('id')
                if outgoing and message.get('method') == 'textDocument/definition':
                    definitions.add(identifier)
                    event = 'request'
                elif not outgoing and identifier in definitions and 'method' not in message:
                    definitions.remove(identifier)
                    event = 'response'
                else:
                    event = None
                if event:
                    with event_file.open('a') as receipt:
                        receipt.write(json.dumps({'event': event, 'message': message})+'\n')
            target.write(b''.join(headers)+body)
            target.flush()
    except BrokenPipeError:
        return

reader = Thread(target=forward, args=(provider.stdout, sys.stdout.buffer, False), daemon=True)
reader.start()
try:
    forward(sys.stdin.buffer, provider.stdin, True)
finally:
    provider.stdin.close()
    if provider.poll() is None:
        provider.terminate()
    try:
        provider.wait(timeout=4)
    except subprocess.TimeoutExpired:
        provider.kill()
        provider.wait()
'''

STARTUP_PROVIDER = r'''
import json
from pathlib import Path
import sys
import time

def receive():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        if not line.strip():
            break
        key, value = line.decode().split(':', 1)
        headers[key.lower()] = value.strip()
    return json.loads(sys.stdin.buffer.read(int(headers['content-length'])))

def send(**message):
    body = json.dumps({'jsonrpc': '2.0', **message}).encode()
    sys.stdout.buffer.write(f'Content-Length: {len(body)}\r\n\r\n'.encode() + body)
    sys.stdout.buffer.flush()

def diagnostic(uri, version, report):
    send(method='textDocument/publishDiagnostics', params={
        'uri': uri, 'version': version, 'diagnostics': [{
            'message': report + ' startup diagnostic', 'severity': 2,
            'range': {'start': {'line': 1, 'character': 0}, 'end': {'line': 1, 'character': 4}},
        }],
    })

versions = {}
while message := receive():
    method, params = message.get('method'), message.get('params', {})
    if method == 'textDocument/didOpen':
        document = params['textDocument']
        versions[document['uri']] = document['version']
    if method == 'textDocument/didChange':
        document = params['textDocument']
        uri, version = document['uri'], document['version']
        if uri in versions and params['contentChanges'][-1]['text'].startswith('# sampled-draft'):
            old = versions.pop(uri)
            gate = Path(sys.argv[2])
            deadline = time.monotonic() + 45
            while not gate.exists():
                if time.monotonic() >= deadline:
                    raise TimeoutError('Startup test did not release diagnostics')
                time.sleep(0.025)
            diagnostic(uri, old, 'stale')
            diagnostic(uri, version, 'fresh')
            Path(sys.argv[1]).write_text(json.dumps({'old': old, 'current': version}))
    if method == 'exit':
        break
    if 'id' in message:
        send(id=message['id'], result={'capabilities': {'textDocumentSync': 1}} if method == 'initialize' else None)
'''

@contextmanager
def startup_gate(folder):
    release_path = '/' + uuid.uuid4().hex
    marker = folder/'startup-diagnostic-release'

    class Release(BaseHTTPRequestHandler):
        def do_POST(self):
            if self.path == release_path:
                marker.touch()
                self.send_response(204)
                self.end_headers()
            else:
                self.send_error(404)

        def log_message(self, *_detail):
            pass

    with ThreadingHTTPServer(('127.0.0.1', 0), Release) as server:
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            yield f'http://127.0.0.1:{server.server_port}{release_path}'
        finally:
            server.shutdown()
            thread.join()

def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def engine_command(workspace):
    with socket.socket() as reserve:
        reserve.bind(('127.0.0.1', 0))
        port = reserve.getsockname()[1]
    binary = os.environ.get('HICKORY_SIMPLE_TEST_ENGINE', str(ROOT/'target/source-edit-focus/debug/examples/engine_client'))
    return [binary, str(workspace), 'window', f'http://127.0.0.1:{port}']

def git_output(workspace, *arguments):
    return subprocess.check_output(['git', '-C', str(workspace), *arguments])

def observe_git_fixture(workspace, method, address, request, status, response):
    path = urllib.parse.urlsplit(address).path
    if status != 200:
        return {}
    if method == 'GET' and path == '/api/git/diff':
        return {'diff': json.loads(response)['diff']}
    if method == 'GET' and path in {'/api/blame', '/api/git/log', '/api/git/reading'}:
        return {'reading': json.loads(response)}
    if method == 'POST' and path in {'/api/git/stage', '/api/git/unstage'}:
        return {'paths': json.loads(request)['paths'],
                'index_sha256': hashlib.sha256(git_output(workspace, 'show', ':note.py')).hexdigest()}
    return {}

def check_git_review(workspace, trace, head):
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    mutations = [row for row in rows if row['method'] == 'POST' and row['path'] in {'/api/git/stage', '/api/git/unstage'}]
    assert [row['path'] for row in mutations] == ['/api/git/stage', '/api/git/unstage'], mutations
    for row, expected in zip(mutations, (GIT_REVIEW_SAVED, GIT_REVIEW_BASELINE)):
        assert row['status'] == 200 and row['paths'] == ['note.py'], row
        assert row['index_sha256'] == hashlib.sha256(expected.encode()).hexdigest(), row
    diffs = [row for row in rows if row['method'] == 'GET' and urllib.parse.urlsplit(row['path']).path == '/api/git/diff']
    sides = set()
    for row in diffs:
        query = urllib.parse.parse_qs(urllib.parse.urlsplit(row['path']).query)
        assert row['status'] == 200 and query['path'] == ['note.py'], row
        assert '-'+GIT_REVIEW_BASELINE in row['diff'] and '+'+GIT_REVIEW_SAVED in row['diff'], row
        assert GIT_REVIEW_DRAFT_MARKER not in row['diff'], row
        sides.update(query['staged'])
    assert sides == {'false', 'true'}, diffs
    assert git_output(workspace, 'rev-parse', 'HEAD') == head
    assert git_output(workspace, 'diff', '--cached', '--', 'note.py') == b''
    assert (workspace/'note.py').read_bytes() == GIT_REVIEW_SAVED.encode()
    assert not any(row['method'] in {'PUT', 'PATCH', 'DELETE'} for row in rows), rows
    return mutations

def check_git_history(workspace, trace, head, baseline, saved):
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    sha = head.decode().strip()
    def reads(path, field, value):
        selected = [row for row in rows if row['method'] == 'GET' and urllib.parse.urlsplit(row['path']).path == path]
        for row in selected:
            query = urllib.parse.parse_qs(urllib.parse.urlsplit(row['path']).query)
            assert row['status'] == 200 and query[field] == [value], row
        return [row['reading'] for row in selected]
    blame, = reads('/api/blame', 'path', 'note.py')
    assert blame['path'] == 'note.py'
    committed, uncommitted = blame['lines']
    first, second = saved.splitlines()
    assert committed['line'] == 1 and committed['text'] == first and not committed['uncommitted'] and committed['full_commit'] == sha, committed
    assert uncommitted['line'] == 2 and uncommitted['text'] == second and uncommitted['uncommitted'] and not uncommitted.get('full_commit'), uncommitted
    history, = reads('/api/git/log', 'path', 'note.py')
    assert history['repository'] and [commit['sha'] for commit in history['commits']] == [sha], history
    immutable = reads('/api/git/reading', 'sha', sha)
    assert len(immutable) == 2, immutable
    for reading in immutable:
        file, = reading['files']
        assert reading['sha'] == sha and file['path'] == 'note.py' and file['after'] == baseline and file['before'] == '' and not file['binary'], reading
    assert git_output(workspace, 'rev-parse', 'HEAD') == head
    assert git_output(workspace, 'diff', '--cached', '--', 'note.py') == b''
    assert (workspace/'note.py').read_bytes() == saved.encode()
    assert not any(row['method'] in {'PUT', 'PATCH', 'DELETE'} or row['path'] in {'/api/git/stage', '/api/git/unstage'} for row in rows), rows
    return sha

def traced_definition_command(folder, workflow, command):
    bridge = folder/f'{workflow}-trace.py'
    bridge.write_text(DEFINITION_TRACE)
    return [sys.executable, str(bridge), str(folder/f'{workflow}-provider.jsonl'), *command]

def provider_uri_path(uri):
    address = urllib.parse.urlsplit(uri)
    assert address.scheme == 'file' and not address.netloc, uri
    return Path(urllib.parse.unquote(address.path)).resolve()

def check_python_definition(workspace):
    trace = workspace.parent/'python-definition-provider.jsonl'
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    request, response = rows
    assert (request['event'], response['event']) == ('request', 'response'), rows
    sent, received = request['message'], response['message']
    assert sent['method'] == 'textDocument/definition' and sent['id'] == received['id'], rows
    params = sent['params']
    assert provider_uri_path(params['textDocument']['uri']) == workspace/'main.py', params
    position = params['position']
    assert position == {'line': 2, 'character': 0}, position
    result = received['result']
    location, = result if isinstance(result, list) else [result]
    uri = location.get('targetUri', location.get('uri'))
    target = location.get('targetSelectionRange', location.get('range'))['start']
    assert provider_uri_path(uri) == workspace/'helper.py' and target == {'line': 3, 'character': 4}, location
    return position, target

def check_managed_move_definition(folder):
    trace = folder/f'{MANAGED_MOVE_DEFINITION_WORKFLOW}-provider.jsonl'
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    requests = [row['message'] for row in rows if row['event'] == 'request']
    assert requests, rows
    for request in requests:
        params = request['params']
        assert request['method'] == 'textDocument/definition' and params['position'] == {'line': 2, 'character': 15}, request
        staged = provider_uri_path(params['textDocument']['uri'])
        assert staged.name == 'move.ts' and staged.parent.parent.name.startswith('hick-lsp-vfiles-'), request
    result, = [row['message']['result'] for row in rows
               if row['event'] == 'response' and row['message']['id'] == requests[-1]['id']]
    location, = result if isinstance(result, list) else [result]
    target = provider_uri_path(location.get('targetUri', location.get('uri')))
    position = location.get('targetSelectionRange', location.get('range'))['start']
    assert target == staged.with_name('relocate.ts') and position == {'line': 0, 'character': 16}, location
    return {'staged_source': staged.name, 'request_position': params['position'],
            'staged_definition': target.name, 'definition_position': position}

def managed_move_publication(files):
    saved = files['notes.md'].replace(MANAGED_MOVE_DECLARATION, MANAGED_MOVE_IMPORT, 1) + MANAGED_MOVE_MARKER
    return {**files, 'notes.md': saved, 'relocate.ts': 'export ' + MANAGED_MOVE_DECLARATION}

def prepare_workflow(workflow, workspace, python_env, slow_debug, recorded_fixtures=None):
    workspace.mkdir()
    if workflow == 'dirty-tab-close':
        (workspace/'note.txt').write_bytes(b'')
        return {'note.txt': 'saved draft'}, None
    if workflow in RECORDED_CONSTRUCTION_SIDES:
        side = RECORDED_CONSTRUCTION_SIDES[workflow]
        return load_script('test-simple-browser').prepare_recorded_construction(workspace, side), None
    if workflow == 'python-definition':
        installed = ROOT/'target/fenced-lsp-tools/basedpyright'
        provider = installed/'dist/pyright-langserver.js'
        node = shutil.which('node')
        assert node and provider.is_file(), 'The installed basedpyright fixture and local Node runtime are required'
        command = traced_definition_command(workspace.parent, workflow, [node, str(provider), '--stdio'])
        configuration = {'servers': {'python': {'command': command}}}
        files = {'main.py': PYTHON_CALLER, 'helper.py': PYTHON_DEFINITION, '.hick-lsp.json': json.dumps(configuration)}
        for path, source in files.items():
            (workspace/path).write_text(source)
        return files, None
    if workflow in GIT_WORKFLOWS:
        stable = GIT_HISTORY_STABLE_LINE if workflow == 'git-history' else ''
        note = workspace/'note.py'
        note.write_text(stable+GIT_REVIEW_BASELINE)
        git_output(workspace, 'init', '-q')
        git_output(workspace, 'add', '--', 'note.py')
        git_output(workspace, '-c', 'user.name=Owned fixture', '-c', 'user.email=fixture@example.invalid',
                   'commit', '-qm', 'Commit owned baseline')
        saved = stable+GIT_REVIEW_SAVED
        note.write_text(saved)
        return {'note.py': saved}, None
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
    elif workflow in RECORDED_FILES:
        files = {path: recorded_fixture_path(recorded_fixtures, workflow, path).read_bytes().decode('utf-8')
                 for path in RECORDED_FILES[workflow]}
        (workspace/'sessions').mkdir()
        sentinel = None
    elif workflow in STARTUP_AUTHORITY_MODES:
        (workspace/'nested').mkdir()
        files = {STARTUP_AUTHORITY_PATH: STARTUP_SOURCE, 'nested/sibling.txt': 'sibling source\n'}
        if workflow in LIVE_STARTUP_AUTHORITY:
            files['outside.txt'] = 'source outside the containing folder\n'
        sentinel = None
    elif workflow == 'startup':
        (workspace/'nested').mkdir()
        provider = workspace.parent/'startup-provider.py'
        provider.write_text(STARTUP_PROVIDER)
        events = workspace.parent/'startup-provider-events.json'
        files = {
            STARTUP_PATH: STARTUP_SOURCE,
            'nested/sibling.txt': 'sibling source\n',
            'outside.txt': 'outside the containing folder\n',
            '.hick-lsp.json': json.dumps({'servers': {'python': {'command': ['python3', str(provider), str(events), str(workspace.parent/'startup-diagnostic-release')]}}}),
        }
        sentinel = None
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
    elif workflow == 'quickfix':
        files = {
            'main.cpp': 'int square(int value) { return value * value; }\nint main() { return square(3) }\n',
            '.hickory-ios-quickfix': 'owned-quickfix-v1\n',
        }
        sentinel = None
    elif workflow in {'typescript-extraction', *MANAGED_MOVE_WORKFLOWS}:
        tools = ROOT/'target/refactor-provider/node_modules'
        server = tools/'typescript-language-server/lib/cli.mjs'
        for name, version in TYPESCRIPT_PROVIDER_VERSIONS.items():
            assert json.loads((tools/name/'package.json').read_text())['version'] == version
        (workspace/'node_modules').mkdir()
        (workspace/'node_modules/typescript').symlink_to(tools/'typescript', target_is_directory=True)
        command = ['node', str(server), '--stdio']
        if workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
            command = traced_definition_command(workspace.parent, workflow, command)
        files = {
            '.hick-lsp.json': json.dumps({'servers': {'typescript': {'command': command}}}),
            'package.json': '{"type":"module"}',
            'tsconfig.json': '{"compilerOptions":{"strict":true,"target":"ES2022","ignoreDeprecations":"6.0"}}',
        }
        if workflow in MANAGED_MOVE_WORKFLOWS:
            namespace = '\x3chick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
            close = '\x3c/hick:doc>\n'
            files.update({
                '.hickory-managed-move-fixture': 'owned-managed-move-v1\n',
                'project.md': namespace + '\x3chick:project sources="notes.md"/>\n' + close,
                'notes.md': namespace + '\n# Original managed prose 🧪\n\n'
                            '\x3chick:file id="source" path="move.ts">\n' + MANAGED_MOVE_LITERAL
                            + '\x3c/hick:file>\n\nThe authored prose after the declaration remains.\n\n' + close,
                'move.ts': MANAGED_MOVE_LITERAL,
            })
            if workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
                files = managed_move_publication(files)
                files['.hickory-managed-move-fixture'] = 'owned-managed-move-definition-v1\n'
            subprocess.run(['git', '-C', str(workspace), 'init', '-q'], check=True)
        else:
            case = load_script('test-provider-refactor-host').CASES[0]
            files.update({case['path']: case['source'], '.hickory-refactor-fixture': 'owned-refactor-v1\n'})
        sentinel = None
    elif workflow == 'recovery':
        details = load_script('test-behavior-readings-browser').prepare(workspace)
        (workspace/'.venv').unlink()
        (workspace/'.venv').symlink_to(python_env.resolve(), target_is_directory=True)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'source.md', 'behavior.md', 'a.py', 'test_a.py', 'test_undeclared.py', 'hickory-dogfood/behavior-map.json', 'hickory-dogfood/narrative-map.json']}
        sentinel = Path(details['sibling'])
    elif workflow in {'proposal-apply', 'proposal-save', 'proposal-native-send'}:
        load_script('test-agent-proposals-browser').prepare(workspace)
        files = {name: (workspace/name).read_text() for name in ['project.md', 'construction.md', 'note.py', 'generated.py', 'behavior.md']}
        if workflow != 'proposal-native-send':
            close = '\x3c/h:doc>\n'
            files['construction.md'] = files['construction.md'].removesuffix(close) + 'Retained author context for the mounted viewport.\n' * 100 + close
        sentinel = None
    else:
        archive = load_script('test-agent-proposal-persistence')
        files = {**archive.FILES, '.hickory-proposal-archive-fixture': 'owned-proposal-archive-v1\n'}
        sentinel = None
    for name, text in files.items():
        (workspace/name).write_text(text, encoding='utf-8')
    return files, sentinel

def run_workflow(workflow, device, seed, temporary_device, candidate, derived, destination,
                 python_env, slow_debug, receipt, recorded_fixtures=None):
    fixture = load_script('test-simple-browser')
    method, markers = WORKFLOWS[workflow]
    taps = workflow_target(workflow) == RECORDED_RUN_TARGET
    test_class = 'RecordedRunUITests' if taps else 'UIKitParityAcceptanceTests'
    name = workflow_target(workflow)+'/'+test_class+'/'+method
    with tempfile.TemporaryDirectory(prefix='ios-parity-') as temporary, ExitStack() as cleanup:
        folder = Path(temporary)
        workspace, state = folder/'workspace', folder/'state'
        files, sentinel = prepare_workflow(workflow, workspace, python_env, slow_debug, recorded_fixtures)
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(state)}
        recorded = workflow in RECORDED_FILES or workflow in RECORDED_CONSTRUCTION_SIDES
        if workflow in ('recorded-debug', 'recorded-run') or workflow in RECORDED_DEBUG_UI_SIDES:
            environment['PATH'] = str(python_env.resolve()/'bin') + os.pathsep + environment['PATH']
        native_send = workflow == 'proposal-native-send'
        git_workflow = workflow in GIT_WORKFLOWS
        python_definition = workflow == 'python-definition'
        managed = load_script('test-agent-proposals-browser') if workflow in {'proposal-apply', 'proposal-save', 'proposal-native-send'} else None
        if managed:
            settings, calls, tool_result = managed.provider_fixture(folder)
            environment['HICKORY_AGENT_TEST_UI_SETTINGS'] = str(settings)
            if native_send:
                environment['HICKORY_EXECUTOR'] = 'local'
            if workflow == 'proposal-save':
                edit = {'documentChanges': [{
                    'kind': 'rename', 'oldUri': 'hick:///note.py',
                    'newUri': 'hick:///' + urllib.parse.quote(PROPOSAL_DESTINATION, safe=''),
                }]}
            else:
                def replacement(line, start, end):
                    return {'range': {'start': {'line': line, 'character': start}, 'end': {'line': line, 'character': end}}, 'newText': '41'}
                edit = {'changes': {
                    'hick:///note.py': [replacement(0, 6, 7)],
                    'hick:///construction.md': [replacement(2, 12, 13)],
                }}
            calls.write_text(json.dumps({'intention': managed.intention(edit)}))
        command = engine_command(workspace)
        archive = load_script('test-agent-proposal-persistence') if workflow == 'proposal' else None
        metadata = {}
        if recorded:
            metadata['HICKORY_IOS_RECORDED_ROOT'] = str(workspace.resolve())
        if workflow in RECORDED_DEBUG_UI_SIDES:
            metadata['HICKORY_IOS_RECORDED_DEBUG_SIDE'] = RECORDED_DEBUG_UI_SIDES[workflow]
        if workflow in RECORDED_CONSTRUCTION_SIDES:
            metadata['HICKORY_IOS_CONSTRUCTION_SIDE'] = RECORDED_CONSTRUCTION_SIDES[workflow]
        if workflow == 'startup':
            metadata['HICKORY_IOS_STARTUP_PATH'] = STARTUP_PATH
            metadata['HICKORY_STARTUP_DIAGNOSTIC_GATE'] = cleanup.enter_context(startup_gate(folder))
        if python_definition:
            metadata.update(HICKORY_IOS_PYTHON_CALLER=PYTHON_CALLER, HICKORY_IOS_PYTHON_DEFINITION=PYTHON_DEFINITION)
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
        authority_mode = STARTUP_AUTHORITY_MODES.get(workflow)
        live_authority = workflow in LIVE_STARTUP_AUTHORITY
        engine = fixture.start(command, env=environment) if live_authority or not authority_mode else None
        try:
            upstream = fixture.origin(engine) if engine else None
            if authority_mode:
                origin, controls, authority_calls, authority_wire = cleanup.enter_context(
                    startup_authority_fixture(workspace.resolve(), authority_mode, upstream))
                metadata.update(controls)
            else:
                origin = upstream
            if native_send:
                managed.api(upstream, 'POST', '/api/docs/workspace/agent/acp', {'backend': 'owned-proposal'})
                send_wire = receipt/'proposal-native-send-http.jsonl'
                origin = cleanup.enter_context(managed.trace_appkit_send(upstream, send_wire))
                metadata['HICKORY_IOS_PROPOSAL_PROMPT'] = managed.PROMPT
            if git_workflow:
                git_head = git_output(workspace, 'rev-parse', 'HEAD')
                baseline = git_output(workspace, 'show', 'HEAD:note.py').decode()
                git_wire = receipt/f'{workflow}-http.jsonl'
                transport = load_script('test-agent-proposals-browser')
                origin = cleanup.enter_context(transport.trace_appkit_send(
                    upstream, git_wire, observe=lambda *event: observe_git_fixture(workspace, *event)))
                metadata.update(HICKORY_IOS_GIT_BASELINE=baseline, HICKORY_IOS_GIT_SAVED=files['note.py'],
                                HICKORY_IOS_GIT_DRAFT_MARKER=GIT_REVIEW_DRAFT_MARKER,
                                HICKORY_IOS_GIT_COMMIT=git_head.decode().strip())
            if archive:
                catalogue = archive.call(origin, 'GET', '/api/agent-proposals')
                assert catalogue['workspace'] != original['workspace'], 'Expected a new owned engine lifetime'
            value = plistlib.loads(candidate.read_bytes())
            target, = test_targets(value, workflow_target(workflow))
            # The controlled startup and recorded cases mount an explicit workspace.
            host_origin = '' if authority_mode or recorded else origin
            for key in ['EnvironmentVariables', 'TestHostEnvironmentVariables']:
                target.setdefault(key, {}).update(HICKORY_IOS_PARITY_ORIGIN=origin, HICKORY_API_ORIGIN=host_origin, **metadata)
            prefix = 'last' if workflow == 'editing' else workflow
            result = receipt/f'{prefix}.xcresult'
            if result.exists():
                shutil.rmtree(result)
            log = receipt/f'{prefix}.log'
            configured = derived/'Build/Products/owned-ios-parity.xctestrun'
            configured.write_bytes(plistlib.dumps(value))
            try:
                allowance = '280' if workflow in {'python-definition', 'dirty-tab-close'} else '180'
                with log.open('w') as output:
                    run(['xcodebuild', 'test-without-building', '-xctestrun', str(configured), '-destination', destination, '-parallel-testing-enabled', 'NO', '-maximum-concurrent-test-simulator-destinations', '1', '-only-testing:'+name, '-resultBundlePath', str(result), '-test-timeouts-enabled', 'YES', '-maximum-test-execution-time-allowance', allowance], stdout=output, stderr=subprocess.STDOUT)
                summary = json.loads(subprocess.check_output(['xcrun', 'xcresulttool', 'get', 'test-results', 'summary', '--path', str(result)]))
                (receipt/f'{prefix}-summary.json').write_text(json.dumps(summary, indent=2))
                assert summary['result'] == 'Passed' and summary['totalTestCount'] == 1 and summary['passedTests'] == 1 and summary['failedTests'] == summary['skippedTests'] == 0, summary
                expected = managed_move_publication(files) if workflow == MANAGED_MOVE_WORKFLOW else files
                for path, text in expected.items():
                    assert (workspace/path).read_bytes() == text.encode(), path
                if sentinel:
                    assert not sentinel.exists(), sentinel.name
                if workflow == 'workspace':
                    events = (workspace/'.hick-cache/executions').read_text().splitlines()
                    assert sorted(events) == ['negative', 'positive', 'positive'], events
                if workflow == 'startup':
                    events = json.loads((folder/'startup-provider-events.json').read_text())
                    assert events['old'] < events['current'], events
                if authority_mode:
                    check_startup_authority_calls(authority_mode, authority_calls)
                if workflow == 'recorded-run':
                    sessions = load_script('test-agent-proposals-browser').api(origin, 'GET', '/api/terminals')['sessions']
                    assert len(sessions) == len({item['id'] for item in sessions}) == 2
                    assert all(item['state'] == 'finished' and item['exit_code'] == 0
                               and item['title'] == 'Run recorded fence' for item in sessions), sessions
                    assert sorted(item['preview'] for item in sessions) == ['42', '53'], sessions
                if managed:
                    response = json.loads(tool_result.read_text())['result']
                    assert not response['isError'], response
                    prepared = json.loads(response['content'][0]['text'])
                    assert prepared['pending_review'] and not any(prepared[key] for key in ['applied', 'saved', 'checks_executed'])
                    retained = managed.api(origin, 'GET', '/api/agent-proposals/'+prepared['token'])
                    assert retained['fingerprint'] == prepared['fingerprint']
                    report = retained['host_application']
                    assert report['status'] == 'host-reported draft application' and not report['saved'] and not report['checks_executed']
                    if workflow == 'proposal-save':
                        assert not (workspace/PROPOSAL_DESTINATION).exists()
                        assert report['after_hashes'] == {
                            'note.py': 'missing',
                            PROPOSAL_DESTINATION: hashlib.sha256(files['note.py'].encode()).hexdigest(),
                        }
                    else:
                        assert report['after_hashes'] == {
                            'note.py': hashlib.sha256(b'value=41\n').hexdigest(),
                            'construction.md': hashlib.sha256(files['construction.md'].replace('generated = 1', 'generated = 41').encode()).hexdigest(),
                        }
                if native_send:
                    turns = managed.api(upstream, 'GET', '/api/docs/workspace/agent/turns')['turns']
                    turn, = turns
                    assert turn['prompt'] == managed.PROMPT and turn['status'] == 'ok', turn
                    managed.verify_native_send(Path(command[0]).resolve(), {path: text.encode() for path, text in files.items()},
                                               workspace, upstream, tool_result, wire=send_wire, log=log,
                                               receipt=receipt/'proposal-native-send-wire-receipt.json',
                                               scope='iOS simulator native keyboard, Send, sealed review, draft Apply and coordinated Edit menu Undo/Redo')
                if workflow == 'git-review':
                    index_receipts = check_git_review(workspace, git_wire, git_head)
                if workflow == 'git-history':
                    commit = check_git_history(workspace, git_wire, git_head, baseline, files['note.py'])
                if python_definition:
                    call_site, declaration = check_python_definition(workspace.resolve())
                observed = log.read_text()
                if live_authority:
                    check_live_startup_authority(workspace.resolve(), authority_wire, observed)
                for marker in markers:
                    assert observed.count('HickoryUIKitParity '+marker) == 1, marker
                payload = {
                    'workflow': workflow, 'device': device, 'seed_device': seed, 'temporary_device': temporary_device,
                    'test': name, 'summary': summary, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
                    'asserted_milestones': markers, 'keyboard_input_verified': False,
                    'physical_input_verified': False, 'accessibility_verified': False,
                }
                if workflow == 'proposal-save':
                    payload['saved_namespace_restored'] = True
                elif workflow in MANAGED_MOVE_WORKFLOWS:
                    payload['provider_versions'] = TYPESCRIPT_PROVIDER_VERSIONS
                    if workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
                        payload['definition'] = check_managed_move_definition(folder)
                        payload['source_unchanged'] = True
                    payload['saved_source_sha256'] = {
                        path: hashlib.sha256((workspace/path).read_bytes()).hexdigest()
                        for path in ['notes.md', 'relocate.ts', 'move.ts']
                    }
                    payload['generated_output_unchanged'] = True
                else:
                    payload['source_unchanged'] = True
                if sentinel:
                    payload['sibling_excluded'] = True
                if workflow == 'editing':
                    payload.update(native_paste_undo_redo_verified=True, coordinated_cleanup_undo_redo_verified=True)
                if taps:
                    payload['simulator_ui_taps_verified'] = True
                if native_send:
                    payload['keyboard_input_verified'] = True
                    payload['native_send_wire_receipt'] = 'proposal-native-send-wire-receipt.json'
                if git_workflow:
                    payload['keyboard_input_verified'] = True
                    payload['fixture_sha256'] = {'note.py': hashlib.sha256(files['note.py'].encode()).hexdigest()}
                if workflow == 'git-review':
                    payload['git_index_receipts'] = index_receipts
                if workflow == 'git-history':
                    payload['git_immutable_commit'] = commit
                if python_definition:
                    payload['keyboard_input_verified'] = True
                    installed = ROOT/'target/fenced-lsp-tools/basedpyright'
                    payload['provider'] = {'name': 'basedpyright', 'version': json.loads((installed/'package.json').read_text())['version'],
                                           'request_position': call_site, 'definition_position': declaration}
                if workflow == 'recorded-run':
                    payload['rendered_terminal_accessibility_verified'] = True
                if workflow in RECORDED_DEBUG_UI_SIDES:
                    payload['simulator_keyboard_input_verified'] = True
                    payload['recorded_side'] = RECORDED_DEBUG_UI_SIDES[workflow]
                if authority_mode:
                    payload['authority_responses'] = authority_calls
                if live_authority:
                    payload['engine_authority'] = authority_wire
                if recorded:
                    payload['fixture_sha256'] = {path: hashlib.sha256(text.encode()).hexdigest() for path, text in files.items()}
                (receipt/f'{prefix}-receipt.json').write_text(json.dumps(payload, indent=2))
            finally:
                configured.unlink(missing_ok=True)
        finally:
            if python_definition or workflow == MANAGED_MOVE_DEFINITION_WORKFLOW:
                trace = folder/f'{workflow}-provider.jsonl'
                if trace.exists():
                    shutil.copyfile(trace, receipt/trace.name)
            if archive:
                archive.stop_engine(fixture, engine, state)
            elif recorded or live_authority or native_send or git_workflow or python_definition or workflow in MANAGED_MOVE_WORKFLOWS:
                load_script('test-agent-proposal-persistence').stop_engine(fixture, engine, state)
            elif engine:
                fixture.stop(engine)
    print(f'PASS mounted UIKit {workflow}', flush=True)

def appkit_startup(receipt):
    fixture = load_script('test-simple-browser')
    name = 'NativeSourceIdentityTests/testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics'
    receipt.mkdir(parents=True)
    with tempfile.TemporaryDirectory(prefix='appkit-startup-') as temporary, ExitStack() as cleanup:
        folder = Path(temporary)
        workspace = folder/'workspace'
        files, _ = prepare_workflow('startup', workspace, None, False)
        environment = {**os.environ, 'HICKORY_STATE_DIR': str(folder/'state')}
        engine = fixture.start(engine_command(workspace), env=environment)
        try:
            environment.update(HICKORY_APPKIT_STARTUP_ORIGIN=fixture.origin(engine), HICKORY_OPEN_FILE=STARTUP_PATH)
            environment['HICKORY_STARTUP_DIAGNOSTIC_GATE'] = cleanup.enter_context(startup_gate(folder))
            log = receipt/'appkit-startup.log'
            with log.open('w') as output:
                run(['swift', 'test', '--package-path', 'apps/apple-simple', '--scratch-path',
                     'target/apple-simple-spm', '--jobs', '2', '--filter', name],
                    env=environment, stdout=output, stderr=subprocess.STDOUT, timeout=600)
            observed = log.read_text()
            assert " testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics]' passed" in observed
            assert 'Executed 1 test, with 0 failures' in observed
            for path, text in files.items():
                assert (workspace/path).read_bytes() == text.encode(), path
            events = json.loads((folder/'startup-provider-events.json').read_text())
            assert events['old'] < events['current'], events
            (receipt/'appkit-startup-receipt.json').write_text(json.dumps({
                'test': name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
                'provider_versions': events, 'source_unchanged': True,
            }, indent=2))
        finally:
            fixture.stop(engine)
    print('PASS mounted AppKit startup', flush=True)

@contextmanager
def startup_authority_fixture(root, mode, engine=None):
    path = STARTUP_AUTHORITY_PATH
    released, lock, calls, wire = Event(), Lock(), [], []
    tree_first = mode == 'tree-first-retry'
    guard_requested, guard_released = Event(), Event()
    gate = '/' + uuid.uuid4().hex
    class Authority(BaseHTTPRequestHandler):
        def bytes_reply(self, status, body, content_type='application/json'):
            self.send_response(status)
            self.send_header('Content-Type', content_type)
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            if body:
                self.wfile.write(body)

        def reply(self, status, value=None):
            self.bytes_reply(status, json.dumps(value).encode() if value is not None else b'')

        def forward(self, request_path):
            if self.headers.get('Upgrade'):
                self.reply(426)
                return
            try:
                incoming = urllib.request.urlopen(engine + self.path, timeout=28)
            except urllib.error.HTTPError as error:
                incoming = error
            except urllib.error.URLError:
                self.reply(502, {'error': 'Owned authority relay could not reach its engine'})
                return
            with incoming:
                body, status = incoming.read(), incoming.status
                content_type = incoming.headers.get('Content-Type', 'application/octet-stream')
            event = None
            if status == 200 and content_type.startswith('application/json'):
                reading = json.loads(body)
                if request_path == '/api/startup-source':
                    event = {'event': 'startup-source', 'path': reading['path'], 'root_path': reading['root_path'],
                             'workspace': reading['workspace_id'], 'source_sha256': hashlib.sha256(reading['content'].encode()).hexdigest()}
                elif request_path == '/api/files':
                    event = {'event': 'tree-confirmed', 'root_path': reading['root_path'], 'workspace': reading['workspace_id']}
                elif request_path.startswith('/api/docs/') and isinstance(reading, dict) and reading.get('path') == path and reading.get('save_guard'):
                    event = {'event': 'guarded-source', 'route': request_path, 'path': reading['path'],
                             'source_sha256': hashlib.sha256(reading['source'].encode()).hexdigest(),
                             'guard_sha256': hashlib.sha256(reading['save_guard'].encode()).hexdigest()}
            if event:
                with lock:
                    wire.append({**event, 'status': status, 'engine': True})
                    calls.append(event['event'])
            self.bytes_reply(status, body, content_type)

        def do_GET(self):
            request = urllib.parse.urlsplit(self.path)
            if request.path == '/api/files':
                if not tree_first and not released.wait(20):
                    self.reply(504, {'error': 'Owned authority fixture timed out'})
                    return
                with lock:
                    unavailable = mode == 'unavailable' or (mode == 'retry' and 'tree-unavailable' not in calls)
                    if unavailable:
                        calls.append('tree-unavailable')
                        if engine:
                            wire.append({'event': 'tree-unavailable', 'status': 503, 'engine': False})
                if unavailable:
                    self.reply(503, {'error': 'Owned file authority unavailable'})
                    return
            if engine:
                self.forward(request.path)
            elif request.path == '/api/startup-source' and urllib.parse.parse_qs(request.query) == {'path': [path]}:
                if tree_first and not released.wait(20):
                    self.reply(504, {'error': 'Owned startup text was not released'})
                    return
                calls.append('startup-source')
                self.reply(200, {'version': 1, 'path': path, 'content': STARTUP_SOURCE,
                                 'root_path': str(root), 'workspace_id': 'owned-workspace'})
            elif request.path == '/api/files':
                calls.append('tree-confirmed')
                self.reply(200, {'root': str(root), 'root_path': str(root), 'workspace_id': 'owned-workspace', 'tree': [
                    {'name': 'nested', 'path': 'nested', 'dir': True, 'children': [
                        {'name': path.split('/')[-1], 'path': path, 'dir': False, 'doc_id': 'owned-md'},
                        {'name': 'sibling.txt', 'path': 'nested/sibling.txt', 'dir': False},
                    ]},
                ]})
            elif request.path == '/api/docs/owned-md':
                if tree_first and 'guard-missing' not in calls:
                    guard_requested.set()
                    if not guard_released.wait(20):
                        self.reply(504, {'error': 'Owned guarded read was not released'})
                        return
                    calls.append('guard-missing')
                    self.reply(200, {'source': STARTUP_SOURCE, 'save_guard_reason': 'Source permission is unconfirmed'})
                else:
                    calls.append('guarded-source')
                    self.reply(200, {'source': STARTUP_SOURCE, 'save_guard': 'owned-startup-guard'})
            else:
                self.reply(404, {'error': 'Optional capabilities are unavailable in this fixture'})

        def do_POST(self):
            if self.path == gate:
                if tree_first and not guard_requested.wait(10):
                    self.reply(504, {'error': 'The guarded source was not requested before startup text'})
                    return
                released.set()
                self.reply(204)
            elif tree_first and self.path == gate + '/guard':
                guard_released.set()
                self.reply(204)
            else:
                self.reply(404, {'error': 'Unexpected mutation'})

        def log_message(self, *_detail):
            pass

    with ThreadingHTTPServer(('127.0.0.1', 0), Authority) as server:
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        origin = f'http://127.0.0.1:{server.server_port}'
        try:
            controls = {
                'HICKORY_STARTUP_AUTHORITY_GATE': origin + gate,
                'HICKORY_STARTUP_AUTHORITY_MODE': mode,
                'HICKORY_OPEN_FILE': path,
            }
            if engine:
                controls['HICKORY_IOS_STARTUP_REAL_ROOT'] = str(root)
            yield origin, controls, calls, wire
        finally:
            released.set()
            guard_released.set()
            server.shutdown()
            thread.join()

def appkit_startup_authority(receipt):
    receipt.mkdir(parents=True)
    name = 'NativeSourceIdentityTests/testMountedRequestedTextPrecedesTreeAuthority'
    with tempfile.TemporaryDirectory(prefix='appkit-authority-') as temporary:
        root = Path(temporary).resolve()
        source = root/STARTUP_AUTHORITY_PATH
        source.parent.mkdir()
        source.write_text(STARTUP_SOURCE)
        for mode in ('confirmed', 'unavailable', 'retry', 'tree-first-retry'):
            with startup_authority_fixture(root, mode) as (origin, controls, calls, _):
                env = {**os.environ, **controls, 'HICKORY_APPKIT_STARTUP_AUTHORITY_ORIGIN': origin}
                log = receipt/f'{mode}.log'
                with log.open('w') as output:
                    run(['swift', 'test', '--package-path', 'apps/apple-simple', '--scratch-path',
                         'target/apple-simple-spm', '--jobs', '2', '--filter', name],
                        env=env, stdout=output, stderr=subprocess.STDOUT, timeout=600)
                assert " testMountedRequestedTextPrecedesTreeAuthority]' passed" in log.read_text()
                check_startup_authority_calls(mode, calls)
                assert source.read_bytes() == STARTUP_SOURCE.encode()
                (receipt/f'{mode}-receipt.json').write_text(json.dumps({
                    'test': name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(), 'responses': calls,
                    'source_unchanged': True,
                }, indent=2))
    print('PASS mounted AppKit provisional source and later authority', flush=True)

def check_startup_authority_calls(mode, calls):
    expected = {
        'confirmed': ['startup-source', 'tree-confirmed', 'guarded-source'],
        'unavailable': ['startup-source', 'tree-unavailable'],
        'retry': ['startup-source', 'tree-unavailable', 'tree-confirmed', 'guarded-source'],
        'tree-first-retry': ['tree-confirmed', 'startup-source', 'guard-missing', 'tree-confirmed', 'guarded-source'],
    }[mode]
    assert calls == expected, calls

def check_live_startup_authority(root, wire, log):
    raw, tree, guarded = [row for row in wire if row['engine']]
    source = hashlib.sha256(STARTUP_SOURCE.encode()).hexdigest()
    assert raw['path'] == guarded['path'] == STARTUP_AUTHORITY_PATH
    assert raw['root_path'] == tree['root_path'] == str(root)
    assert raw['workspace'] == tree['workspace'] and raw['workspace']
    assert raw['source_sha256'] == guarded['source_sha256'] == source
    assert guarded['route'].startswith('/api/docs/')
    mounted = f"live-startup-source-sha256={source} guard-sha256={guarded['guard_sha256']} workspace={raw['workspace']}"
    assert log.count(mounted) == 1, 'The mounted editor did not adopt the engine guard for these exact startup bytes'

def run(args,**kwargs):
 return subprocess.run(args,check=True,text=True,**kwargs)
def require_space(minimum):
 available=shutil.disk_usage(ROOT).free
 if available<minimum*1024*1024:raise RuntimeError(f'Need {minimum}MiB free before simulator/build work; available {available//(1024*1024)}MiB')
def test_targets(value, name=TARGET):
 if isinstance(value,dict):
  if value.get('BlueprintName')==name or value.get('TestableName')==name:yield value
  else:
   for child in value.values():yield from test_targets(child,name)
 elif isinstance(value,list):
  for child in value:yield from test_targets(child,name)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--device');parser.add_argument('--build',action='store_true');parser.add_argument('--boot',action='store_true');parser.add_argument('--temporary-device',action='store_true');parser.add_argument('--minimum-free-mib',type=int,default=2048)
 parser.add_argument('--appkit-startup', action='store_true', help='Run only the mounted macOS startup workflow')
 parser.add_argument('--appkit-startup-authority', action='store_true', help='Hold file tree authority while mounting macOS source')
 parser.add_argument('--workflow', choices=WORKFLOWS, action='append', help='Repeat to use one simulator for several focused cases')
 parser.add_argument('--python-env', type=Path, default=ROOT/'target/thread-debug-env')
 parser.add_argument('--recorded-fixtures', type=Path, help='Explicit shared fixture directory for recorded Agent journeys')
 parser.add_argument('--receipt-dir', type=Path, help='Create a new directory for this run and its lifecycle receipt')
 parser.add_argument('--slow-debug', action='store_true', help='Delay the owned Rust fixture to exercise staged admission')
 args=parser.parse_args()
 if args.appkit_startup or args.appkit_startup_authority:
  if args.device or args.build or args.boot or args.temporary_device or args.workflow or args.slow_debug or not args.receipt_dir:
   parser.error('AppKit startup cases require --receipt-dir and no simulator options')
  if args.appkit_startup and args.appkit_startup_authority:
   parser.error('Choose one AppKit startup case')
  (appkit_startup if args.appkit_startup else appkit_startup_authority)(args.receipt_dir)
  return
 if not args.device:parser.error('--device selects the simulator for the hosted iOS workflow')
 recorded_workflows = RECORDED_FILES.keys() & set(args.workflow or [])
 if RECORDED_CONSTRUCTION_SIDES.keys() & set(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Recorded construction requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if 'proposal-native-send' in (args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Native proposal Send requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if GIT_WORKFLOWS.intersection(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Native Git review requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if 'python-definition' in (args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Native Python definition requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if MANAGED_MOVE_WORKFLOWS.intersection(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Managed TypeScript Move requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if LIVE_STARTUP_AUTHORITY.intersection(args.workflow or []) and not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
  parser.error('Live startup authority requires an explicit HICKORY_SIMPLE_TEST_ENGINE')
 if recorded_workflows:
  if not args.recorded_fixtures or not os.environ.get('HICKORY_SIMPLE_TEST_ENGINE'):
   parser.error('Recorded workflows require --recorded-fixtures and an explicit HICKORY_SIMPLE_TEST_ENGINE')
  if any(not recorded_fixture_path(args.recorded_fixtures, workflow, path).is_file()
         for workflow in recorded_workflows for path in RECORDED_FILES[workflow]):
   parser.error('The recorded fixture directory is missing an expected source')
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
 receipt=args.receipt_dir or ROOT/'target/ios-parity'
 receipt.mkdir(parents=True,exist_ok=args.receipt_dir is None)
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
  workflows=list(dict.fromkeys(args.workflow or ['editing']))
  names=list(dict.fromkeys(workflow_target(workflow) for workflow in workflows))
  if args.build:
   require_space(args.minimum_free_mib);run(['xcodegen','generate','--spec',str(ROOT/'apps/apple-simple/project.yml')])
   for name in names:
    run(['xcodebuild','-project',str(ROOT/'apps/apple-simple/HickorySimple.xcodeproj'),'-scheme',SCHEMES[name],'-configuration','Debug','-destination',destination,'-derivedDataPath',str(derived),'-jobs','2','build-for-testing','CODE_SIGNING_ALLOWED=NO'])
  candidates={name:[] for name in names}
  for path in (derived/'Build/Products').glob('*.xctestrun'):
   value=plistlib.loads(path.read_bytes())
   for name in names:
    if len(list(test_targets(value,name)))==1:candidates[name].append(path)
  if any(len(paths)!=1 for paths in candidates.values()):raise RuntimeError('Build one configuration per selected test scheme with --build')
  for workflow in workflows:
   try:
    require_space(args.minimum_free_mib)
    run_workflow(workflow, device, args.device, owned_device is not None, candidates[workflow_target(workflow)][0], derived,
                 destination, args.python_env, args.slow_debug, receipt, args.recorded_fixtures)
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
  (receipt/'last-device-lifecycle.json').write_text(json.dumps(lifecycle,indent=2))
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

These focused control-policy checks replace the tool boundary and deliberately fail before building. They establish that a disposable device is deleted using only its returned UUID, an existing seed is restored without deletion, and a create result that names any preexisting device never grants boot/delete authority. They also check that a new run cannot reuse an explicit receipt directory or create a simulator without explicit boot authority. Run `python3 scripts/test-ios-parity-runner.py` independently of the real hosted witness.

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
 def test_explicit_receipt_directory_cannot_replace_existing_evidence(self):
  runner=self.load();devices={'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True}]}}
  with tempfile.TemporaryDirectory() as folder:
   receipt=Path(folder)/'passed.xcresult'
   receipt.write_bytes(b'retained evidence')
   with patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',return_value=json.dumps(devices).encode()) as read,patch.object(runner.subprocess,'run') as mutate,patch.object(sys,'argv',['runner','--device',SEED,'--boot','--receipt-dir',folder]):
    with self.assertRaises(FileExistsError):runner.main()
    self.assertEqual(receipt.read_bytes(),b'retained evidence')
    self.assertEqual(read.call_count,1)
    mutate.assert_not_called()
if __name__=='__main__':unittest.main()
</ui:copy>
<ui:file path="scripts/test-ios-parity-runner.py"><ui:paste select="#ios-mounted-parity-runner-safety"/></ui:file>
</ui:doc>
