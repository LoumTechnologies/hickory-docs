import XCTest
import SwiftUI
import WebKit
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CoverageViewTests:XCTestCase {
    func testLateReportSourceReadKeepsNewerEditorAndCanBeRetried() async throws {
        for kind in ["coverage", "unbound", "test", "report-choice"] {
            let arrived = expectation(description: "\(kind) destination read")
            let gate = DispatchSemaphore(value: 0)
            ResponseProtocol.status = 200
            ResponseProtocol.responsesByPath = [:]
            ResponseProtocol.response = Data(#"{"path":"target.txt","content":"first\nsecond\n","hash":"target"}"#.utf8)
            ResponseProtocol.beforeResponse = { request in
                guard request.url?.path == "/api/file" else { return }
                arrived.fulfill()
                _ = gate.wait(timeout: .now() + 5)
            }
            let config = URLSessionConfiguration.ephemeral
            config.protocolClasses = [ResponseProtocol.self]
            let session = URLSession(configuration: config)
            let workspace = Workspace()
            workspace.engine = try Engine(address: "http://fixture.invalid", session: session)
            defer {
                gate.signal()
                ResponseProtocol.beforeResponse = nil
                workspace.buffers.values.forEach { $0.language?.close() }
                session.invalidateAndCancel()
            }

            let draft = FileNode(name: "draft.txt", path: "draft.txt", dir: false)
            let target = FileNode(name: "target.txt", path: "target.txt", dir: false)
            let retained = DocumentBuffer(file: draft, text: FileText(path: draft.path, content: "saved", hash: "saved"))
            retained.text = "unsaved draft"
            retained.selection = NSRange(location: 2, length: 3)
            workspace.buffers[draft.path] = retained
            workspace.rootFiles = [draft, target]
            if kind == "report-choice" { workspace.showSource(draft.path) } else { workspace.showUtility(.agent) }
            workspace.showTests = kind == "test"
            workspace.showCoverage = kind != "test"

            let hash = nativeSourceHash("first\nsecond\n")
            func navigate() async throws -> Bool? {
                if kind == "test" {
                    await workspace.navigateTestPoint(TestSourcePoint(path: target.path, line: 2, character: 0, hash: hash))
                    return nil
                } else {
                    let point: [String: Any] = kind == "unbound" ? ["path": target.path] : ["path": target.path, "line": 2, "sourceHash": hash]
                    return try await workspace.navigateCoverage(point, state: workspace.coverage)
                }
            }
            let pending = Task { try await navigate() }
            await fulfillment(of: [arrived], timeout: 3)
            if kind == "report-choice" { workspace.coverage.retireNavigation() } else { workspace.showSource(draft.path) }
            if kind == "test" {
                workspace.showTests = false
                workspace.showTests = true
            }
            gate.signal()
            let retired = try await pending.value
            if kind != "test" { XCTAssertEqual(retired, false, kind) }

            XCTAssertTrue(workspace.currentBuffer === retained, kind)
            XCTAssertEqual(retained.text, "unsaved draft", kind)
            XCTAssertEqual(retained.selection, NSRange(location: 2, length: 3), kind)
            XCTAssertFalse(workspace.layout.groups().contains { $0.tabs.containsTab(nativeSourceTab(target.path)) }, kind)
            XCTAssertEqual(workspace.showTests, kind == "test", kind)
            XCTAssertEqual(workspace.showCoverage, kind != "test", kind)
            let cached = try XCTUnwrap(workspace.buffers[target.path])
            var selected: NSRange?
            cached.selectSourceRange = { range, _ in selected = range; return true }
            let fresh = try await navigate()
            if kind != "test" { XCTAssertEqual(fresh, true, kind) }
            XCTAssertTrue(workspace.currentBuffer === cached, kind)
            XCTAssertEqual(selected, kind == "unbound" ? nil : NSRange(location: 6, length: 0), kind)
            XCTAssertFalse(workspace.showTests, kind)
            XCTAssertFalse(workspace.showCoverage, kind)
        }
    }

    func testCoverageNavigationRetiresClosedForeignAndEqualByteReplies()async throws {
        for mode in ["closed","engine","owner","undo","selection","fraction","boolean","success","unbound"] {
            let w=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"literal %?#.py",path:"literal %?#.py",dir:false),source="first\nsecond\n",b=DocumentBuffer(file:FileNode(name:"literal %?#.py",path:"literal %?#.py",dir:false),text:FileText(path:"literal %?#.py",content:"first\nsecond\n",hash:"base"))
            w.engine=engine;w.buffers[f.path]=b;w.showSource(f.path);w.showCoverage=true;b.selection=NSRange(location:2,length:0);b.selectSourceRange={range,text in guard exactUTF8(text,b.text) else{return false};b.selection=range;return true}
            let state=w.coverage;var calls=0;var location:[String:Any]=["path":f.path,"line":2,"sourceHash":nativeSourceHash(source)]
            if mode=="fraction"{location["line"]=1.5};if mode=="boolean"{location["line"]=true};if mode=="unbound"{location["line"]=NSNull();location["sourceHash"]=NSNull()}
            let current:()->Bool={calls+=1;if mode=="closed"&&calls>=2{return false};if mode=="engine"&&calls==2{w.engine=try! Engine(address:"http://127.0.0.1:2")};if calls==3{if mode=="undo"{b.text="later";b.text=source};if mode=="selection"{b.selection=NSRange(location:1,length:0)};if mode=="owner"{w.buffers[f.path]=DocumentBuffer(file:f,text:FileText(path:f.path,content:source,hash:"base"))}};return true}
            do{try await w.navigateCoverage(location,state:state,isCurrent:current);XCTAssertTrue(mode=="success"||mode=="unbound",mode)}catch{XCTAssertFalse(mode=="success"||mode=="unbound","\(mode): \(error)")}
            if mode=="success"{XCTAssertEqual(b.selection,NSRange(location:6,length:0));XCTAssertFalse(w.showCoverage)}else if mode=="unbound"{XCTAssertEqual(b.selection,NSRange(location:2,length:0));XCTAssertFalse(w.showCoverage)}else{XCTAssertTrue(w.showCoverage,mode)}
            XCTAssertEqual(b.text,source);XCTAssertEqual(b.base,source)
        }
    }

    #if os(macOS)
    func testLiveOwnedCollectorSurfaceResultsRerunAndSourceGuards()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_COLLECTOR_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_COLLECTOR_WORKSPACE"] else{throw XCTSkip("Use the maintained owned collector-host fixture.")}
        let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-collector-fixture"),encoding:.utf8)=="owned-collector-v1\n" else{throw EngineError(message:"An owned disposable collector workspace is required.")}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let listing=try await engine.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
        let file=try XCTUnwrap(w.rootFiles.first{exactUTF8($0.path,"app.py")}),buffer=try await w.loadBuffer(file),source=buffer.text;w.showSource(file.path)
        let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:1000,height:740),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{w.showCoverage=false;for item in w.buffers.values{item.language?.close()};w.coverage.stop();w.sourceTestIndex?.stop();window.orderOut(nil);window.contentView=nil}
        func settle(_ message:String,_ condition:()->Bool)async throws{for _ in 0..<1200{if condition(){return};try await Task.sleep(for:.milliseconds(50))};throw EngineError(message:message)}
        func native(_ view:NSView)->NSTextView?{if let value=view as? NSTextView{return value};return view.subviews.lazy.compactMap{native($0)}.first}
        try await settle("Native source did not mount"){buffer.sourceEditorState?()?.attached==true};let editor=try XCTUnwrap(native(host)),manager=try XCTUnwrap(editor.undoManager),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,scroll=try XCTUnwrap(editor.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host)
        editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("# retained redo\n",replacementRange:editor.selectedRange());manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo)
        func findWeb(_ view:NSView)->WKWebView?{if let web=view as? WKWebView,web.url?.lastPathComponent=="coverage.html"{return web};return view.subviews.lazy.compactMap{findWeb($0)}.first}
        func web()async throws->WKWebView{var found:WKWebView?;try await settle("Local coverage renderer did not mount"){found=NSApp.windows.filter{$0.isVisible}.compactMap{$0.contentView}.lazy.compactMap{findWeb($0)}.first;return found != nil};return try XCTUnwrap(found)}
        func text(_ web:WKWebView)async throws->String{(try await web.evaluateJavaScript("document.body.innerText")) as? String ?? ""}
        func awaitText(_ web:WKWebView,_ expected:String)async throws{for _ in 0..<1200{if (try? await text(web))?.contains(expected)==true{return};try await Task.sleep(for:.milliseconds(50))};throw EngineError(message:"Coverage surface did not show \(expected)")}
        func js(_ value:String)throws->String{String(data:try JSONSerialization.data(withJSONObject:value,options:.fragmentsAllowed),encoding:.utf8)!}
        func click(_ web:WKWebView,_ label:String)async throws{let quoted=try js(label);_ = try await web.evaluateJavaScript("(()=>{const button=[...document.querySelectorAll('button')].find(node=>node.textContent.trim()==="+quoted+");if(!button||button.disabled)throw Error('Control unavailable: '+"+quoted+");button.click();return true})()")}
        func runID(_ web:WKWebView)async throws->String{let value=try await web.evaluateJavaScript("document.querySelector('.coverage-columns main>p code')?.textContent");return try XCTUnwrap(value as? String)}
        try XCTUnwrap(w.commands().first{$0.id=="workspace.coverage"}).run();let firstWeb=try await web();try await awaitText(firstWeb,"Discover runner cases");try await click(firstWeb,"Discover runner cases");try await awaitText(firstWeb,"discovered · test_app.py::test_positive");try await awaitText(firstWeb,"discovered · test_app.py::test_negative");XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent(".hick-cache/executions").path))
        try await click(firstWeb,"Run saved project with coverage");try await awaitText(firstWeb,"automated · failed · current source");try await awaitText(firstWeb,"executable lines covered");let first=try await runID(firstWeb);XCTAssertEqual(first.count,32);try await awaitText(firstWeb,"passed · test_app.py::test_positive");try await awaitText(firstWeb,"failed · test_app.py::test_negative")
        XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);try await settle("Real coverage marks did not bind"){!buffer.coverageRows().isEmpty}
        try await click(firstWeb,"Rerun failed tests from selected run");for _ in 0..<1200{if let id=try? await runID(firstWeb),id != first{break};try await Task.sleep(for:.milliseconds(50))};try await awaitText(firstWeb,"automated · failed · current source");let second=try await runID(firstWeb);XCTAssertNotEqual(first,second);try await awaitText(firstWeb,"failed · test_app.py::test_negative");let rerunText=try await text(firstWeb);XCTAssertFalse(rerunText.contains("passed · test_app.py::test_positive"))
        let firstJSON=try js(first);_ = try await firstWeb.evaluateJavaScript("(()=>{const button=[...document.querySelectorAll('.coverage-run')].find(node=>node.textContent.includes("+firstJSON+"));if(!button)throw Error('Original run missing');button.click();return true})()");try await awaitText(firstWeb,"passed · test_app.py::test_positive");let historyText=try await text(firstWeb);XCTAssertTrue(historyText.contains(second))
        try await click(firstWeb,"Close");try await settle("Coverage did not close"){!w.showCoverage};XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);manager.redo();XCTAssertTrue(buffer.text.contains("retained redo"));manager.undo();XCTAssertEqual(buffer.text,source)
        editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("# unsaved after measurement\n",replacementRange:editor.selectedRange());let draft=buffer.text,selection=buffer.selection;XCTAssertTrue(buffer.coverageRows().isEmpty)
        try XCTUnwrap(w.commands().first{$0.id=="workspace.coverage"}).run();let staleWeb=try await web();try await awaitText(staleWeb,"executable lines covered");try await click(staleWeb,"app.py");try await awaitText(staleWeb,"Source changed");XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.selection,selection);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("app.py"),encoding:.utf8),source)
    }
    func testBundledCoverageSurfaceUsesWorkspaceBoundNativeMessages()throws {
        let workspace=Workspace(),state=workspace.coverage
        workspace.engine = try Engine(address: "http://127.0.0.1:1")
        let source="let value = 1\n",file=FileNode(name:"draft.swift",path:"draft.swift",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:nativeSourceHash(source)))
        workspace.buffers[file.path]=buffer;buffer.text+="// unsaved\n";buffer.selection=NSRange(location:4,length:2)
        let coordinator=CoverageWebCoordinator(workspace),view=coverageWebView(coordinator)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:740,height:640),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=view
        defer{coordinator.destroy();window.orderOut(nil)}
        func pump(_ condition:()->Bool){let deadline=Date().addingTimeInterval(15);while !condition()&&Date()<deadline{RunLoop.current.run(until:Date().addingTimeInterval(0.02))}}
        func evaluate(_ script:String)throws->Any?{var result:Result<Any?,Error>?;view.evaluateJavaScript(script){value,error in result=error.map(Result.failure) ?? .success(value)};pump{result != nil};return try XCTUnwrap(result).get()}
        pump{coordinator.ready}
        if !coordinator.ready {
            let state = try? evaluate("JSON.stringify({href:location.href,ready:document.readyState,bridge:typeof globalThis.hickoryCoverage})")
            XCTFail("Bundled coverage renderer did not become ready in WebKit: \(String(describing: state))")
        }
        guard coordinator.ready else{return}
        var rendered=false;pump{rendered=(try? evaluate("!!document.querySelector('[aria-label=\"Tests and code coverage\"]')")) as? Bool ?? false;return rendered};XCTAssertTrue(rendered)
        _ = try evaluate("webkit.messageHandlers.coverage.postMessage({kind:'select',workspace:'obsolete',id:'old'});null")
        RunLoop.current.run(until:Date().addingTimeInterval(0.05));XCTAssertNil(state.selected)
        let identity=String(data:try JSONSerialization.data(withJSONObject:state.identity,options:[.fragmentsAllowed]),encoding:.utf8)!
        _ = try evaluate("webkit.messageHandlers.coverage.postMessage({kind:'select',workspace:\(identity),id:'chosen'});null");pump{state.selected=="chosen"};XCTAssertEqual(state.selected,"chosen")
        workspace.coverage=SimpleCoverageState()
        _ = try evaluate("webkit.messageHandlers.coverage.postMessage({kind:'select',workspace:\(identity),id:'late'});null")
        RunLoop.current.run(until:Date().addingTimeInterval(0.05));XCTAssertEqual(state.selected,"chosen");XCTAssertNil(workspace.coverage.selected)
        XCTAssertTrue(exactUTF8(buffer.text,source+"// unsaved\n"));XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.selection,NSRange(location:4,length:2))
    }
    #endif
}
