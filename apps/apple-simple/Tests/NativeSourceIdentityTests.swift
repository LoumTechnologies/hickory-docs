import XCTest
#if os(macOS)
import AppKit
import SwiftUI
#endif
@testable import HickorySimple
final class NativeSourceIdentityTests:XCTestCase {
    let first="caf\u{e9}.py",second="cafe\u{301}.py"
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
