import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import WebKit
#endif
@testable import HickorySimple
final class NativeSourceIdentityTests:XCTestCase {
    let first="caf\u{e9}.py",second="cafe\u{301}.py"
    @MainActor func testRefreshAdoptsCurrentDiskSourceAndRetainsDivergentDraft() async throws {
        let session = URLSessionConfiguration.ephemeral
        session.protocolClasses = [ResponseProtocol.self]
        let engine = try Engine(address: "http://refresh.invalid", session: URLSession(configuration: session))
        let workspace = Workspace()
        let path = "notes.txt", root = "/owned", identity = String(repeating: "a", count: 32)
        let file = FileNode(name: path, path: path, dir: false)
        workspace.engine = engine
        workspace.sourceRootAddress = root
        workspace.sourceWorkspaceID = identity
        let reading = StartupSourceObservation(origin: engine.origin, root: root, workspace: identity)
        let buffer = DocumentBuffer(file: file, text: FileText(path: path, content: "Early text", hash: ""),
                                    startupObservation: reading)
        workspace.buffers[path] = buffer
        workspace.showSource(path)
        defer {
            ResponseProtocol.responsesByPath = [:]
            buffer.language?.close()
            workspace.sourceTestIndex?.stop()
        }
        let tree = try JSONSerialization.data(withJSONObject: [
            "root": "owned", "root_path": root, "workspace_id": identity,
            "tree": [["name": path, "path": path, "dir": false]],
        ])
        func setDisk(_ source: String, guardToken: String) throws {
            ResponseProtocol.status = 200
            ResponseProtocol.responsesByPath = [
                "/api/files": tree,
                "/api/file": try JSONEncoder().encode(["path": path, "content": source, "hash": source, "save_guard": guardToken]),
            ]
        }

        try setDisk("Current text", guardToken: "current")
        await workspace.refresh()
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertEqual(buffer.text, "Current text")
        XCTAssertFalse(buffer.readOnly)
        XCTAssertFalse(buffer.dirty)
        XCTAssertEqual(buffer.savedGuard, "current")

        try setDisk("External update", guardToken: "external")
        await workspace.refresh()
        XCTAssertEqual(buffer.text, "External update")
        XCTAssertFalse(buffer.dirty)
        XCTAssertEqual(buffer.savedGuard, "external")

        buffer.text = "My draft"
        try setDisk("Another update", guardToken: "another")
        await workspace.refresh()
        XCTAssertEqual(buffer.text, "My draft")
        XCTAssertEqual(buffer.base, "External update")
        XCTAssertEqual(buffer.savedGuard, "external")
        XCTAssertTrue(workspace.status.contains("changed on disk"))
    }

    #if os(macOS)
    @MainActor func testMountedMacRefreshKeepsMappedSelectionAndViewport() async throws {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [ResponseProtocol.self]
        let session = URLSession(configuration: configuration)
        let engine = try Engine(address: "http://mounted-refresh.invalid", session: session)
        let workspace = Workspace()
        let file = FileNode(name: "HtmlExporter.kt", path: "HtmlExporter.kt", dir: false)
        let source: String
        if let fixture = ProcessInfo.processInfo.environment["HICKORY_NATIVE_REFRESH_FIXTURE"] {
            source = try String(contentsOfFile: fixture, encoding: .utf8)
        } else {
            source = (1...390).map { "val attachment\($0) = \($0)\n" }.joined()
        }
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "old", save_guard: "old"))
        workspace.engine = engine
        workspace.sourceRootAddress = "/owned"
        workspace.sourceWorkspaceID = String(repeating: "a", count: 32)
        workspace.rootFiles = [file]
        workspace.buffers[file.path] = buffer
        workspace.showSource(file.path)
        _ = NSApplication.shared
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 680, height: 480), styleMask: [.titled], backing: .buffered, defer: false)
        let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
        window.contentView = host
        window.makeKeyAndOrderFront(nil)
        func findEditor(_ view: NSView) -> SimpleMacTextView? {
            (view as? SimpleMacTextView) ?? view.subviews.lazy.compactMap { findEditor($0) }.first
        }
        defer {
            window.orderOut(nil)
            session.invalidateAndCancel()
            ResponseProtocol.responsesByPath = [:]
            ResponseProtocol.beforeResponse = nil
        }
        for _ in 0..<100 {
            host.layoutSubtreeIfNeeded()
            if findEditor(host) != nil { break }
            try await Task.sleep(for: .milliseconds(20))
        }
        let editor = try XCTUnwrap(findEditor(host))
        let scroll = try XCTUnwrap(editor.enclosingScrollView)
        XCTAssertTrue(window.makeFirstResponder(editor))
        let oldSelection = NSRange(location: 0, length: 0)
        XCTAssertEqual(buffer.selection, oldSelection)
        XCTAssertEqual(editor.selectedRange(), oldSelection)
        let beforeScroll = scroll.contentView.bounds.origin
        let prefix = "// External line\n"
        let changed = prefix + source
        ResponseProtocol.status = 200
        ResponseProtocol.responsesByPath = [
            "/api/files": try JSONSerialization.data(withJSONObject: [
                "root": "owned", "root_path": "/owned", "workspace_id": workspace.sourceWorkspaceID!,
                "tree": [["name": file.name, "path": file.path, "dir": false]]
            ]),
            "/api/file": try JSONEncoder().encode([
                "path": file.path, "content": changed, "hash": "fresh", "save_guard": "fresh"
            ])
        ]
        await workspace.refresh()
        try await Task.sleep(for: .milliseconds(150))

        let expected = NSRange(location: oldSelection.location + prefix.utf16.count, length: oldSelection.length)
        XCTAssertEqual(editor.string, changed)
        XCTAssertEqual(editor.selectedRange(), expected)
        XCTAssertEqual(buffer.selection, expected)
        XCTAssertEqual(scroll.contentView.bounds.origin, beforeScroll)
        XCTAssertFalse(buffer.dirty)
        XCTAssertTrue(buffer.snippets.notice.isEmpty)

        // While the server is reading disk, the reviewer can still move the
        // viewport and select source. A later edit must use that newer selection.
        let second = changed + "// Later change\n"
        let reading = expectation(description: "external read pending")
        let deliver = DispatchSemaphore(value: 0)
        ResponseProtocol.responsesByPath["/api/file"] = try JSONEncoder().encode([
            "path": file.path, "content": second, "hash": "second", "save_guard": "second"
        ])
        ResponseProtocol.beforeResponse = { request in
            guard request.url?.path == "/api/file" else { return }
            reading.fulfill()
            _ = deliver.wait(timeout: .now() + 5)
        }
        let pending = Task { await workspace.refreshCurrentSource() }
        await fulfillment(of: [reading], timeout: 3)
        let latest = NSRange(location: 30, length: 5)
        editor.setSelectedRange(latest)
        scroll.contentView.scroll(to: NSPoint(x: 0, y: 170))
        scroll.reflectScrolledClipView(scroll.contentView)
        let latestScroll = scroll.contentView.bounds.origin
        deliver.signal()
        await pending.value
        try await Task.sleep(for: .milliseconds(150))
        XCTAssertEqual(editor.string, second)
        XCTAssertEqual(editor.selectedRange(), latest)
        XCTAssertEqual(buffer.selection, latest)
        XCTAssertEqual(scroll.contentView.bounds.origin, latestScroll)
        XCTAssertFalse(buffer.dirty)
        XCTAssertTrue(buffer.snippets.notice.isEmpty)
    }
    #endif

    func testMapKeepsExactNamesAndPendingReadIdentities(){
        XCTAssertEqual(first,second);XCTAssertNotEqual(NativeSourceKey(first),NativeSourceKey(second))
        var sources:NativeSourceMap<Int>=[:];sources[first]=1;sources[second]=2
        XCTAssertEqual(sources.count,2);XCTAssertEqual(sources[first],1);XCTAssertEqual(sources[second],2)
        let captured=sources.mapValues{String($0)};XCTAssertEqual(captured[first],"1");XCTAssertEqual(captured[second],"2")
        sources.removeValue(forKey:first);XCTAssertNil(sources[first]);XCTAssertEqual(sources[second],2)
    }
    func testTypedSourceAndUtilityAddressesRemainSeparateThroughAtomicRouting()throws{
        var layout=PaneLayout();layout.activateSource("files",in:"document-group");layout.activateUtility(.agent,in:"document-group");layout.activateSource("agent",in:"document-group")
        XCTAssertNotEqual(NativeTabIdentity.source(NativeSourceKey("files")),.utility(.files));XCTAssertEqual(layout.nodes["document-group"]?.tabs.count,3);XCTAssertTrue(layout.nodes["files-group"]!.tabs.containsUtility(.files))
        let moved=try layout.remappingTabs([Data(nativeSourceTab("files").utf8):nativeSourceTab("terminal"),Data(nativeSourceTab("agent").utf8):nativeSourceTab("files")])
        XCTAssertTrue(moved.nodes["files-group"]!.tabs.containsUtility(.files));XCTAssertTrue(moved.nodes["document-group"]!.tabs.containsUtility(.agent));XCTAssertTrue(moved.nodes["document-group"]!.tabs.containsTab(nativeSourceTab("files")));XCTAssertTrue(moved.nodes["document-group"]!.tabs.containsTab(nativeSourceTab("terminal")));XCTAssertEqual(moved.nodes["document-group"]?.tabs.count,3)
        XCTAssertThrowsError(try layout.remappingTabs([Data(nativeSourceTab("files").utf8):nativeSourceTab("agent")]))
    }
    @MainActor func testUnnamedStateBelongsToBufferRatherThanFilenamePrefix()throws{
        let named=DocumentBuffer(file:FileNode(name:"Untitled-real.txt",path:"Untitled-real.txt",dir:false),text:FileText(path:"Untitled-real.txt",content:"saved named source",hash:"loaded",save_guard:"loaded-guard")),unnamed=DocumentBuffer(file:FileNode(name:"Untitled",path:"arbitrary-draft-address",dir:false),text:FileText(path:"arbitrary-draft-address",content:"unnamed source",hash:""),isUntitledDraft:true)
        let space=Workspace();space.buffers[named.file.path]=named;space.buffers[unnamed.file.path]=unnamed;let capture=try resourceOwnershipSnapshot(space)
        XCTAssertFalse(named.isUntitledDraft);XCTAssertTrue(unnamed.isUntitledDraft);XCTAssertEqual(capture.named[named.file.path],named.text);XCTAssertEqual(capture.guards[named.file.path],"loaded-guard");XCTAssertNil(capture.named[unnamed.file.path]);XCTAssertEqual(capture.unaddressed[unnamed.id.uuidString],unnamed.text)
    }
    func testLanguageFramesRefuseMergedContributorsAndBooleanRequestIdentity()throws{
        let aliases=Data(#"{"jsonrpc":"2.0","id":1,"result":{"changes":{"hick:///é.py":[],"hick:///e\u0301.py":[]}}}"#.utf8)
        XCTAssertThrowsError(try nativeLanguageJSONObject(aliases));XCTAssertThrowsError(try nativeLanguageJSONObject(Data(#"{"id":true,"result":[]}"#.utf8)))
        XCTAssertNoThrow(try nativeLanguageJSONObject(Data(#"{"jsonrpc":"2.0","id":1,"result":{"version":1.5}}"#.utf8)))
    }
    func testWorkspacePlansKeepCanonicalDistinctTargets()throws {
        let sources:NativeSourceMap<SourceSnapshot>=[first:SourceSnapshot(text:"one",writable:true),second:SourceSnapshot(text:"two",writable:true)]
        let edit:[String:Any]=["changes":[languageURI(first):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"first"]],languageURI(second):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"second"]]]]
        let plan=try planWorkspaceEdit("Two exact sources",raw:edit,sources:sources);XCTAssertEqual(plan.patches.count,2);XCTAssertEqual(plan.patches.first{exactUTF8($0.path,first)}?.before,"one");XCTAssertEqual(plan.patches.first{exactUTF8($0.path,second)}?.before,"two");XCTAssertNoThrow(try validateReferencedTargets(plan,referencePaths:[first,second]))
        var partial=plan;partial.patches.removeAll{exactUTF8($0.path,first)};XCTAssertThrowsError(try validateReferencedTargets(partial,referencePaths:[first,second]))
    }
    @MainActor func testObservedDebuggerSourceAddressPreservesRawSpelling()throws{let debug=NativeDebugger(engine:try Engine(address:"http://identity.invalid"));defer{debug.close()};debug.owner=first;debug.owner=second;XCTAssertTrue(exactUTF8(debug.owner,second));debug.launchSource="é";debug.launchSource="e\u{301}";XCTAssertTrue(exactUTF8(debug.launchSource,"e\u{301}"))}
    @MainActor func testReferenceAcquisitionDoesNotDeduplicateCanonicalNames()async throws {
        var round=0,opened:[NativeSourceKey]=[]
        let value=try await acquireReferenceSources([:],query:{round+=1;return [self.first,self.second,self.first]},acquire:{path in opened.append(NativeSourceKey(path));return SourceSnapshot(text:exactUTF8(path,self.first) ? "one":"two",writable:true)},current:{true})
        XCTAssertEqual(round,2);XCTAssertEqual(Set(opened).count,2);XCTAssertEqual(value.sources.count,2);XCTAssertEqual(value.references.count,2)
    }
    #if os(macOS)
    @MainActor func testMountedRequestedTextPrecedesTreeAuthority() async throws {
        let environment = ProcessInfo.processInfo.environment
        guard let origin = environment["HICKORY_APPKIT_STARTUP_AUTHORITY_ORIGIN"],
              let path = environment["HICKORY_OPEN_FILE"],
              let release = environment["HICKORY_STARTUP_AUTHORITY_GATE"].flatMap(URL.init(string:)),
              let mode = environment["HICKORY_STARTUP_AUTHORITY_MODE"] else {
            throw XCTSkip("Use the owned AppKit startup authority fixture")
        }
        _ = NSApplication.shared
        NSApp.setActivationPolicy(.regular)
        let workspace = Workspace()
        workspace.address = origin
        let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 840, height: 560), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = host
        window.makeKeyAndOrderFront(nil)
        defer {
            workspace.debugger?.close()
            workspace.editorRecovery.close()
            workspace.sourceTestIndex?.stop()
            for buffer in workspace.buffers.values { buffer.language?.close() }
            window.orderOut(nil)
        }
        func editor(in view: NSView) -> NSTextView? {
            if let text = view as? NSTextView, text.string.hasPrefix("# owned startup capability fixture") { return text }
            return view.subviews.lazy.compactMap { editor(in: $0) }.first
        }
        func wait(_ message: String, ready: () -> Bool) async throws {
            let clock = ContinuousClock(), deadline = clock.now + .seconds(25)
            while clock.now < deadline {
                host.layoutSubtreeIfNeeded()
                if ready() { return }
                try await Task.sleep(for: .milliseconds(20))
            }
            throw EngineError(message: message + " · " + workspace.status)
        }
        func releaseGate(_ url: URL) async throws {
            var request = URLRequest(url: url)
            request.httpMethod = "POST"
            let (_, response) = try await URLSession.shared.data(for: request)
            XCTAssertEqual((response as? HTTPURLResponse)?.statusCode, 204)
        }
        let treeFirst = mode == "tree-first-retry"
        if treeFirst {
            try await wait("Files did not arrive ahead of the held source reads") {
                workspace.engine != nil && Set(workspace.files.map(\.path)) == [path, "nested/sibling.txt"]
            }
            XCTAssertNil(workspace.currentBuffer)
            try await releaseGate(release)
        }
        try await wait("Requested source did not mount while authority was held") {
            exactUTF8(workspace.currentBuffer?.file.path, path) && editor(in: host) != nil
        }
        let buffer = try XCTUnwrap(workspace.currentBuffer), view = try XCTUnwrap(editor(in: host))
        let source = buffer.text, identity = try XCTUnwrap(buffer.sourceEditorState?()?.identity)
        XCTAssertTrue(workspace.connecting)
        XCTAssertEqual(workspace.engine != nil, treeFirst)
        XCTAssertTrue(buffer.readOnly)
        XCTAssertFalse(view.isEditable)
        XCTAssertNil(buffer.savedGuard)
        let selection = NSRange(location: 42, length: 5)
        view.setSelectedRange(selection)
        let scroll = try XCTUnwrap(view.enclosingScrollView)
        view.layoutSubtreeIfNeeded()
        scroll.contentView.scroll(to: NSPoint(x: 0, y: 260))
        scroll.reflectScrolledClipView(scroll.contentView)
        let manager = try XCTUnwrap(view.layoutManager), container = try XCTUnwrap(view.textContainer)
        func visibleSourceStart() -> Int {
            let offset = scroll.contentView.bounds.origin, origin = view.textContainerOrigin
            let glyph = manager.glyphIndex(for: NSPoint(x: offset.x, y: offset.y - origin.y), in: container)
            return manager.characterIndexForGlyph(at: min(glyph, manager.numberOfGlyphs - 1))
        }
        let top = visibleSourceStart()
        try await wait("Local markup did not attach independently of tree authority") { exactUTF8(buffer.inlineMarkup.source, source) }
        XCTAssertEqual(view.selectedRange(), selection)
        XCTAssertEqual(visibleSourceStart(), top, "The same source text stays at the top when markup changes heading height")
        let viewport = scroll.contentView.bounds.origin
        let bounds = scroll.convert(scroll.bounds, to: host)
        XCTAssertGreaterThan(viewport.y, 0)
        try await releaseGate(treeFirst ? release.appendingPathComponent("guard") : release)
        try await wait("Workspace authority reply did not settle") {
            !workspace.connecting && (mode != "confirmed" || view.isEditable)
        }
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertTrue(editor(in: host) === view)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertTrue(exactUTF8(view.string, source))
        XCTAssertEqual(view.selectedRange(), selection)
        XCTAssertEqual(scroll.contentView.bounds.origin, viewport)
        XCTAssertEqual(scroll.convert(scroll.bounds, to: host), bounds)
        if mode != "confirmed" {
            XCTAssertTrue(buffer.readOnly)
            XCTAssertFalse(view.isEditable)
            XCTAssertNil(buffer.savedGuard)
            XCTAssertTrue(workspace.status.contains(treeFirst ? "Source permission is unconfirmed" : "remains readable"))
        }
        if mode == "retry" || treeFirst {
            if treeFirst { await workspace.refresh() }
            else {
                let connected = await workspace.connect()
                XCTAssertTrue(connected, workspace.status)
            }
            try await wait("Retry did not confirm the reading already on screen") { view.isEditable }
            XCTAssertTrue(workspace.currentBuffer === buffer)
            XCTAssertTrue(editor(in: host) === view)
            XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
            XCTAssertEqual(view.selectedRange(), selection)
            XCTAssertEqual(scroll.contentView.bounds.origin, viewport)
        }
        if mode != "unavailable" {
            XCTAssertFalse(buffer.readOnly)
            XCTAssertNil(buffer.startupObservation)
            XCTAssertEqual(buffer.savedGuard, "owned-startup-guard")
            XCTAssertEqual(Set(workspace.files.map(\.path)), [path, "nested/sibling.txt"])
            let sourceIndex = try XCTUnwrap(buffer.testIndex)
            try await wait("The mounted editor never attempted source-test discovery after confirmation") {
                exactUTF8(sourceIndex.records[path]?.source, source)
            }
            XCTAssertTrue(editor(in: host) === view)
            XCTAssertEqual(view.selectedRange(), selection)
            XCTAssertEqual(scroll.contentView.bounds.origin, viewport)
            XCTAssertTrue(window.makeFirstResponder(view))
            view.insertText("draft", replacementRange: view.selectedRange())
            XCTAssertTrue(buffer.dirty)
            let history = try XCTUnwrap(view.undoManager)
            history.undo()
            XCTAssertTrue(exactUTF8(view.string, source))
            XCTAssertFalse(buffer.dirty)
            if treeFirst {
                history.redo()
                XCTAssertTrue(buffer.dirty)
                XCTAssertTrue(exactUTF8(view.string, (source as NSString).replacingCharacters(in: selection, with: "draft")))
                history.undo()
            }
        }
    }

    @MainActor func testMountedRequestedFileRetainsEditingThroughDelayedDiagnostics() async throws {
        let environment = ProcessInfo.processInfo.environment
        guard let origin = environment["HICKORY_APPKIT_STARTUP_ORIGIN"],
              let path = environment["HICKORY_OPEN_FILE"],
              let release = environment["HICKORY_STARTUP_DIAGNOSTIC_GATE"].flatMap(URL.init(string:)) else {
            throw XCTSkip("Use the owned AppKit startup fixture")
        }
        _ = NSApplication.shared
        NSApp.setActivationPolicy(.regular)
        let workspace = Workspace()
        workspace.address = origin
        let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 840, height: 560), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = host
        window.makeKeyAndOrderFront(nil)
        defer {
            workspace.debugger?.close()
            workspace.editorRecovery.close()
            workspace.sourceTestIndex?.stop()
            for buffer in workspace.buffers.values { buffer.language?.close() }
            window.orderOut(nil)
        }
        func nativeEditor(in view: NSView) -> NSTextView? {
            if let text = view as? NSTextView, text.string.hasPrefix("# owned startup capability fixture") { return text }
            return view.subviews.lazy.compactMap { nativeEditor(in: $0) }.first
        }
        func wait(_ message: String, ready: () -> Bool) async throws {
            let clock = ContinuousClock(), deadline = clock.now + .seconds(30)
            while clock.now < deadline {
                host.layoutSubtreeIfNeeded()
                if ready() { return }
                try await Task.sleep(for: .milliseconds(20))
            }
            XCTFail(message + " · " + workspace.status)
            throw EngineError(message: message)
        }
        try await wait("AppKit did not mount the requested source") {
            exactUTF8(workspace.currentBuffer?.file.path, path) && nativeEditor(in: host) != nil
        }
        let buffer = try XCTUnwrap(workspace.currentBuffer)
        let editor = try XCTUnwrap(nativeEditor(in: host))
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let source = buffer.text
        XCTAssertEqual(editor.string, source)
        XCTAssertTrue(editor.isEditable)
        XCTAssertEqual(Set(workspace.files.map(\.path)), Set([path, "nested/sibling.txt"]))
        XCTAssertTrue(workspace.rootFiles.contains { $0.path == "outside.txt" })
        XCTAssertEqual(workspace.layout.groups().map(\.id), ["files-group", "document-group"])
        XCTAssertTrue(workspace.layout.nodes["files-group"]?.tabs.containsUtility(.files) == true)
        XCTAssertFalse(workspace.problems.contains { $0.message.contains("startup diagnostic") })
        XCTAssertTrue(window.makeFirstResponder(editor))
        let insertion = "# sampled-draft\n"
        editor.setSelectedRange(NSRange(location: 0, length: 0))
        editor.insertText(insertion, replacementRange: editor.selectedRange())
        let draft = insertion + source
        XCTAssertTrue(exactUTF8(buffer.text, draft))
        let selected = NSRange(location: draft.utf16.count / 3, length: 5)
        editor.setSelectedRange(selected)
        let scroll = try XCTUnwrap(editor.enclosingScrollView)
        editor.layoutSubtreeIfNeeded()
        scroll.contentView.scroll(to: NSPoint(x: 0, y: 320))
        scroll.reflectScrolledClipView(scroll.contentView)
        let viewport = scroll.contentView.bounds.origin
        XCTAssertGreaterThan(viewport.y, 0)
        let history = try XCTUnwrap(editor.undoManager)
        XCTAssertTrue(history.canUndo)
        XCTAssertFalse(workspace.problems.contains { $0.message.contains("startup diagnostic") })
        var request = URLRequest(url: release)
        request.httpMethod = "POST"
        let (_, released) = try await URLSession.shared.data(for: request)
        XCTAssertEqual((released as? HTTPURLResponse)?.statusCode, 204)
        try await wait("Current diagnostic did not reach the AppKit editor") {
            workspace.problems.contains { exactUTF8($0.path, path) && $0.current && $0.message == "fresh startup diagnostic" }
                && exactUTF8(buffer.inlineMarkup.source, draft)
        }
        XCTAssertFalse(workspace.problems.contains { $0.message == "stale startup diagnostic" })
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(editor.selectedRange(), selected)
        XCTAssertEqual(scroll.contentView.bounds.origin.x, viewport.x, accuracy: 1)
        XCTAssertEqual(scroll.contentView.bounds.origin.y, viewport.y, accuracy: 1)
        XCTAssertTrue(exactUTF8(editor.string, draft))
        XCTAssertTrue(buffer.dirty)
        history.undo()
        XCTAssertTrue(exactUTF8(editor.string, source))
        XCTAssertFalse(buffer.dirty)
        history.redo()
        XCTAssertTrue(exactUTF8(editor.string, draft))
        XCTAssertTrue(buffer.dirty)
        await workspace.refresh()
        XCTAssertEqual(Set(workspace.files.map(\.path)), Set([path, "nested/sibling.txt"]))
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        buffer.openDiagramFile?("../outside.txt")
        try await wait("Relative diagram link could not leave the displayed folder") {
            exactUTF8(workspace.currentBuffer?.file.path, "outside.txt")
        }
        XCTAssertEqual(Set(workspace.files.map(\.path)), Set([path, "nested/sibling.txt"]))
        await workspace.open(buffer.file)
        XCTAssertTrue(workspace.currentBuffer === buffer)
        XCTAssertTrue(exactUTF8(buffer.text, draft))
    }

    @MainActor func testRefusedDiagramLinkKeepsTheDraftAndSelection() async throws {
        let workspace = Workspace()
        let engine = try Engine(address: "http://127.0.0.1:1")
        let originFile = FileNode(name: "audit.md", path: "nested/audit.md", dir: false)
        let targetFile = FileNode(name: "with spaces.md", path: "with spaces.md", dir: false)
        workspace.rootFiles = [originFile, targetFile]
        let origin = workspace.retainBuffer(originFile, text: FileText(path: originFile.path, content: "# Audit", hash: "saved"), engine: engine)
        let target = workspace.retainBuffer(targetFile, text: FileText(path: targetFile.path, content: "# Target", hash: "saved"), engine: engine)
        defer {
            origin.language?.close()
            target.language?.close()
            workspace.sourceTestIndex?.stop()
        }
        workspace.showSource(originFile.path)
        origin.text = "# Audit draft"
        let selection = NSRange(location: 2, length: 4)
        origin.selection = selection
        let follow = try XCTUnwrap(origin.openDiagramFile)

        follow("../../outside.md")
        XCTAssertEqual(workspace.status, "This link must point to a file in the workspace.")
        XCTAssertTrue(workspace.currentBuffer === origin)
        XCTAssertEqual(origin.selection, selection)
        XCTAssertEqual(origin.text, "# Audit draft")

        follow("../with%20spaces.md")
        let clock = ContinuousClock(), deadline = clock.now + .seconds(2)
        while workspace.currentBuffer !== target && clock.now < deadline {
            try await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertTrue(workspace.currentBuffer === target)
        XCTAssertEqual(origin.selection, selection)
        XCTAssertEqual(origin.text, "# Audit draft")
    }

    @MainActor func testMountedDiagramLinkRejectsUnsafeTargetAndFollowsWorkspaceFile() async throws {
        guard ProcessInfo.processInfo.environment["HICKORY_NATIVE_WEBVIEW_TEST"] == "1" else {
            throw XCTSkip("Enable the focused native WebKit witness explicitly")
        }
        _ = NSApplication.shared
        let workspace = Workspace()
        let engine = try Engine(address: "http://127.0.0.1:1")
        let entry = FileNode(name: "audit.md", path: "docs/audit.md", dir: false)
        let chapter = FileNode(name: "export boundary.md", path: "export boundary.md", dir: false)
        let saved = """
        # Audit workspace

        <h:diagram renderer="graph">
        {"nodes":[{"id":"refused","label":"Outside","href":"../../outside.md"},{"id":"allowed","label":"Export","href":"../export%20boundary.md"},{"id":"ref","label":"Reference"}],"edges":[]}
        </h:diagram>
        """
        let origin = workspace.retainBuffer(entry, text: FileText(path: entry.path, content: saved, hash: "saved"), engine: engine)
        let target = workspace.retainBuffer(chapter, text: FileText(path: chapter.path, content: "# Export boundary", hash: "target"), engine: engine)
        workspace.rootFiles = [entry, chapter]
        workspace.showSource(entry.path)
        origin.text = saved + "\nUnsaved observations."
        origin.inlineMarkup = computeInlineMarkup(origin.text, path: entry.path)
        let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 840, height: 560),
                              styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = host
        window.orderFront(nil)
        defer {
            origin.language?.close()
            target.language?.close()
            workspace.sourceTestIndex?.stop()
            window.orderOut(nil)
        }
        func find<T: NSView>(_ type: T.Type, in view: NSView) -> T? {
            (view as? T) ?? view.subviews.lazy.compactMap { find(type, in: $0) }.first
        }
        var linkedPreview: WKWebView?
        for _ in 0..<160 {
            host.layoutSubtreeIfNeeded()
            if let preview = find(WKWebView.self, in: host),
               let count = try? await preview.evaluateJavaScript("document.querySelectorAll('#formula svg g[role=link]').length"),
               count as? Int == 2 {
                linkedPreview = preview
                break
            }
            try await Task.sleep(for: .milliseconds(25))
        }
        let preview = try XCTUnwrap(linkedPreview)
        let editor = try XCTUnwrap(find(SimpleMacTextView.self, in: host))
        let selection = NSRange(location: 2, length: 4)
        editor.setSelectedRange(selection)
        let draft = editor.string
        let reference = try await preview.evaluateJavaScript("document.querySelector('#formula svg g title').textContent") as? String
        XCTAssertEqual(reference, "No workspace destination")

        _ = try await preview.evaluateJavaScript("""
            document.querySelector('[aria-label="Open Outside"]')
                .dispatchEvent(new MouseEvent('click', {bubbles:true}));
            """)
        for _ in 0..<80 {
            if workspace.status == "This link must point to a file in the workspace." { break }
            try await Task.sleep(for: .milliseconds(25))
        }
        XCTAssertEqual(workspace.status, "This link must point to a file in the workspace.")
        XCTAssertTrue(workspace.currentBuffer === origin)
        XCTAssertEqual(editor.selectedRange(), selection)
        XCTAssertEqual(draft, saved + "\nUnsaved observations.")
        XCTAssertEqual(editor.string, draft)

        _ = try await preview.evaluateJavaScript("""
            document.querySelector('[aria-label="Open Export"]')
                .dispatchEvent(new KeyboardEvent('keydown', {key:'Enter', bubbles:true}));
            """)
        for _ in 0..<80 {
            host.layoutSubtreeIfNeeded()
            if workspace.currentBuffer === target,
               find(SimpleMacTextView.self, in: host)?.string == "# Export boundary" { break }
            try await Task.sleep(for: .milliseconds(25))
        }
        XCTAssertTrue(workspace.currentBuffer === target)
        XCTAssertEqual(find(SimpleMacTextView.self, in: host)?.string, "# Export boundary")
        XCTAssertEqual(origin.text, draft)
        XCTAssertEqual(origin.selection, selection)
    }

    @MainActor func testReservedPhysicalNamesMountCodeAndKeepUtilityTabsReachable()async throws {
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try Engine(address:"http://identity.invalid",session:URLSession(configuration:configuration));space.engine=engine
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests();ResponseProtocol.beforeResponse={request in let path=URLComponents(url:request.url!,resolvingAgainstBaseURL:false)?.queryItems?.first{$0.name=="path"}?.value ?? "";let text="readable code for "+path+"\n";ResponseProtocol.response=(try? JSONSerialization.data(withJSONObject:["path":path,"content":text,"hash":nativeSourceHash(text),"save_guard":"guard-"+path])) ?? Data()}
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
        defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];space.sourceTestIndex?.stop();for b in space.buffers.values{b.language?.close()};window.orderOut(nil)}
        func find(_ view:NSView,_ source:String)->NSTextView?{if let text=view as? NSTextView,exactUTF8(text.string,source){return text};return view.subviews.lazy.compactMap{find($0,source)}.first}
        for path in ["files","agent","terminal","Untitled-real.txt"]{
            await space.open(FileNode(name:path,path:path,dir:false));let buffer=try XCTUnwrap(space.buffers[path]);XCTAssertTrue(space.currentBuffer === buffer);XCTAssertFalse(buffer.isUntitledDraft)
            for _ in 0..<200{host.layoutSubtreeIfNeeded();if find(host,buffer.text) != nil{break};try await Task.sleep(for:.milliseconds(10))};let editor=try XCTUnwrap(find(host,buffer.text)),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity;editor.setSelectedRange(NSRange(location:1,length:0));editor.insertText("!",replacementRange:editor.selectedRange());let draft=buffer.text,selection=editor.selectedRange()
            space.showUtility(.files);XCTAssertNil(space.currentBuffer);XCTAssertTrue(space.layout.nodes["files-group"]!.tabs.containsUtility(.files));space.showSource(path)
            for _ in 0..<200{host.layoutSubtreeIfNeeded();if find(host,draft) != nil{break};try await Task.sleep(for:.milliseconds(10))};XCTAssertTrue(find(host,draft) === editor);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(editor.selectedRange(),selection);editor.undoManager?.undo();XCTAssertEqual(buffer.text,"readable code for "+path+"\n");editor.undoManager?.redo();XCTAssertTrue(exactUTF8(buffer.text,draft))
        }
        let capture=try resourceOwnershipSnapshot(space);XCTAssertEqual(capture.named["Untitled-real.txt"],space.buffers["Untitled-real.txt"]?.text);XCTAssertNotNil(capture.guards["Untitled-real.txt"])
        for utility in [NativeUtilityTab.files,.agent,.terminal]{space.showUtility(utility);XCTAssertNil(space.currentBuffer);XCTAssertTrue(space.layout.groups().contains{$0.tabs.containsUtility(utility)})};XCTAssertEqual(space.buffers.count,4)
    }
    @MainActor func testExactFileWireOpeningKeepsBothNativeEditorsAndUndo()async throws {
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular)
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://identity.invalid",session:URLSession(configuration:configuration)),space=Workspace();space.engine=engine
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests()
        ResponseProtocol.beforeResponse={request in
            let path=URLComponents(url:request.url!,resolvingAgainstBaseURL:false)?.queryItems?.first{$0.name=="path"}?.value ?? ""
            let text=exactUTF8(path,self.first) ? "first source\n":"second source\n"
            ResponseProtocol.response=(try? JSONSerialization.data(withJSONObject:["path":path,"content":text,"hash":nativeSourceHash(text),"save_guard":"loaded-"+Data(path.utf8).base64EncodedString()])) ?? Data()
        }
        let firstFile=FileNode(name:first,path:first,dir:false),secondFile=FileNode(name:second,path:second,dir:false);space.rootFiles=[firstFile,secondFile]
        await space.open(firstFile);let firstBuffer=try XCTUnwrap(space.buffers[first]);XCTAssertTrue(space.currentBuffer === firstBuffer)
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
        defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];space.sourceTestIndex?.stop();for buffer in space.buffers.values{buffer.language?.close()};window.orderOut(nil)}
        func find(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{find($0)}.first}
        func mounted(_ buffer:DocumentBuffer)async throws->NSTextView{for _ in 0..<200{host.layoutSubtreeIfNeeded();if let text=find(host),exactUTF8(text.string,buffer.text){return text};try await Task.sleep(for:.milliseconds(10))};throw EngineError(message:"Exact source editor did not mount")}
        let firstEditor=try await mounted(firstBuffer),firstIdentity=try XCTUnwrap(firstBuffer.sourceEditorState?()).identity,geometry=try XCTUnwrap(firstEditor.enclosingScrollView).convert(firstEditor.enclosingScrollView!.bounds,to:host)
        firstEditor.setSelectedRange(NSRange(location:1,length:0));firstEditor.insertText("!",replacementRange:firstEditor.selectedRange());let firstDraft=firstBuffer.text,selection=firstEditor.selectedRange()
        await space.open(secondFile);let secondBuffer=try XCTUnwrap(space.buffers[second]);XCTAssertFalse(firstBuffer === secondBuffer);XCTAssertTrue(space.currentBuffer === secondBuffer);XCTAssertEqual(space.buffers.count,2)
        let secondEditor=try await mounted(secondBuffer);XCTAssertFalse(firstEditor === secondEditor);secondEditor.setSelectedRange(NSRange(location:2,length:0));secondEditor.insertText("?",replacementRange:secondEditor.selectedRange());let secondDraft=secondBuffer.text
        space.show(first);let returned=try await mounted(firstBuffer);XCTAssertTrue(returned === firstEditor);XCTAssertEqual(firstBuffer.sourceEditorState?()?.identity,firstIdentity);XCTAssertEqual(firstEditor.selectedRange(),selection);XCTAssertTrue(exactUTF8(firstBuffer.text,firstDraft));XCTAssertEqual(firstEditor.enclosingScrollView!.convert(firstEditor.enclosingScrollView!.bounds,to:host),geometry)
        firstEditor.undoManager?.undo();XCTAssertEqual(firstBuffer.text,"first source\n");XCTAssertTrue(exactUTF8(secondBuffer.text,secondDraft));firstEditor.undoManager?.redo();XCTAssertTrue(exactUTF8(firstBuffer.text,firstDraft))
        XCTAssertEqual(space.layout.nodes["document-group"]?.tabs.count,2);XCTAssertNotEqual(firstFile.id,secondFile.id)
        XCTAssertNotEqual(languageURI(first),languageURI(second));XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),2)
    }
    #endif
    func testTabsSelectMoveRemoveAndSwapExactNames()throws{
        var layout=PaneLayout();layout.activate(first,in:"document-group");layout.activate(second,in:"document-group")
        XCTAssertEqual(layout.nodes["document-group"]?.tabs.count,2);XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.active,second))
        layout.activate(first,in:"document-group");XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.active,first))
        let swapped=try layout.remappingTabs([Data(first.utf8):second,Data(second.utf8):first]);XCTAssertTrue(exactUTF8(swapped.nodes["document-group"]?.tabs.first,second));XCTAssertTrue(exactUTF8(swapped.nodes["document-group"]?.active,second))
        layout.move(second,to:"document-group",before:first);XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.tabs.first,second))
        layout.remove(first);XCTAssertEqual(layout.nodes["document-group"]?.tabs.count,1);XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.tabs.first,second))
    }
}
