<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read runner results and coverage on apple

For maintainers changing workspace services independently of the editor renderer.

The existing menu opens a floating, replaceable tool surface. The editor remains mounted and retains its own selection, scroll and undo state. A native local-only web renderer delegates only fixed coverage operations to the connected engine; workspace replacement retires its replies. Source navigation opens unbound files without positional claims, and accepts a reported line only after matching its captured hash against available text. Selecting a report retains its record identity; source-line mark binding is a separate capability.

<ui:copy id="simple-coverage-apple">
import SwiftUI
import WebKit
import Observation
@MainActor @Observable final class SimpleCoverageState {
    let identity=UUID().uuidString
    var selected:String?;var notice=""
}
@MainActor extension Workspace {
    func navigateCoverage(_ value:[String:Any],state:SimpleCoverageState) async throws {
        guard coverage === state,let path=value["path"] as? String else {throw EngineError(message:"Workspace changed. Reopen coverage.")}
        func find(_ files:[FileNode])->FileNode? {for file in files {if file.path==path {return file};if let child=find(file.children ?? []) {return child}};return nil}
        guard let file=buffers[path]?.file ?? find(rootFiles),!file.dir else {throw EngineError(message:"Source no longer listed. Refresh the workspace.")}
        let buffer=try await loadBuffer(file),source=buffer.text
        guard coverage === state else {throw EngineError(message:"Workspace changed. Reopen coverage.")}
        guard let line=value["line"] as? Int,let hash=value["sourceHash"] as? String else {await open(file);showCoverage=false;return}
        let actual=await Task.detached {nativeSourceHash(source)}.value
        guard coverage === state,exactUTF8(source,buffer.text),actual==hash.lowercased(),line>0,line<=source.components(separatedBy:"\n").count else {throw EngineError(message:"Source changed or report position is unavailable. Refresh coverage before navigating.")}
        await openSemanticLocation(SemanticLocation(uri:languageURI(path),line:line-1,character:0),allowDirty:true,expectedSource:source)
        if coverage === state {showCoverage=false}
    }
}
@MainActor final class CoverageWebCoordinator:NSObject,WKScriptMessageHandler,WKNavigationDelegate {
    weak var view:WKWebView?;let workspace:Workspace;let state:SimpleCoverageState;let engine:Engine?
    var allowedURL:URL?;var mounted=true;var ready=false;var active:Set<Int>=[]
    init(_ workspace:Workspace){self.workspace=workspace;state=workspace.coverage;engine=workspace.engine}
    var current:Bool {mounted && workspace.coverage === state}
    func display(){guard ready,current else{return};let buffer=workspace.currentBuffer
        send("update",["workspace":state.identity,"path":buffer?.file.path as Any? ?? NSNull(),"language":buffer?.file.language ?? "unsupported","selectedId":state.selected as Any? ?? NSNull()])
    }
    func send(_ method:String,_ value:Any){guard current,let data=try? JSONSerialization.data(withJSONObject:value,options:[.fragmentsAllowed]),let json=String(data:data,encoding:.utf8) else{return};view?.evaluateJavaScript("globalThis.hickoryCoverage?."+method+"("+json+")",completionHandler:nil)}
    func reply(_ id:Int,_ result:Any,_ error:String?){guard current,let data=try? JSONSerialization.data(withJSONObject:[id,result,error as Any? ?? NSNull()]),let json=String(data:data,encoding:.utf8) else{return};view?.evaluateJavaScript("globalThis.hickoryCoverage?.reply(..."+json+")",completionHandler:nil)}
    func userContentController(_ controller:WKUserContentController,didReceive message:WKScriptMessage){
        guard current,message.frameInfo.isMainFrame,message.frameInfo.request.url==allowedURL,let event=message.body as? [String:Any],let kind=event["kind"] as? String else{return}
        if kind=="ready" {ready=true;display();return}
        guard event["workspace"] as? String==state.identity else{return}
        if kind=="close" {workspace.showCoverage=false;workspace.currentBuffer?.focusSource?();return}
        if kind=="select" {state.selected=event["id"] as? String;return}
        guard let id=event["id"] as? Int,id>0,!active.contains(id),active.count<64 else{return};active.insert(id)
        Task {[weak self] in guard let self else{return};defer{active.remove(id)}
            do {guard current else{return}
                if kind=="api",let body=event["body"] as? [String:Any],let action=body["action"] as? String,["list","get","locations","inspect","discover","run","rerun-failed","manual-start","browser-checkpoint","checkpoint","stop","import","compare","annotate"].contains(action),let engine {
                    let result:NativeTableJSON=try await engine.request("POST","/api/tests/coverage",body:body);reply(id,result.object,nil)
                }else if kind=="navigate",let location=event["location"] as? [String:Any] {try await workspace.navigateCoverage(location,state:state);reply(id,NSNull(),nil)}
                else {throw EngineError(message:"Unsupported coverage request. Reopen the tool.")}
            }catch {reply(id,NSNull(),error.localizedDescription)}
        }
    }
    func webView(_ webView:WKWebView,decidePolicyFor navigationAction:WKNavigationAction,decisionHandler:@escaping(WKNavigationActionPolicy)->Void){decisionHandler(navigationAction.targetFrame?.isMainFrame != false && navigationAction.request.url==allowedURL ? .allow:.cancel)}
    func webViewWebContentProcessDidTerminate(_ webView:WKWebView){ready=false;state.notice="Coverage renderer stopped. Close and reopen it; source remains available."}
    func destroy(){mounted=false;view?.configuration.userContentController.removeScriptMessageHandler(forName:"coverage");view?.stopLoading();view?.navigationDelegate=nil;view=nil}
}
@MainActor func coverageWebView(_ coordinator:CoverageWebCoordinator)->WKWebView {
    let configuration=WKWebViewConfiguration();configuration.websiteDataStore = .nonPersistent();configuration.userContentController.add(coordinator,name:"coverage")
    let view=WKWebView(frame:.zero,configuration:configuration);coordinator.view=view;view.navigationDelegate=coordinator
    if let file=simpleResource("coverage",withExtension:"html"){coordinator.allowedURL=file;view.loadFileURL(file,allowingReadAccessTo:file.deletingLastPathComponent())}
    else{coordinator.state.notice="Coverage assets unavailable. Rebuild the app's local resources."};return view
}
#if os(macOS)
private struct CoverageWeb: NSViewRepresentable {
    var workspace:Workspace
    func makeCoordinator()->CoverageWebCoordinator {CoverageWebCoordinator(workspace)}
    func makeNSView(context:Context)->WKWebView {coverageWebView(context.coordinator)}
    func updateNSView(_ view:WKWebView,context:Context){}
    static func dismantleNSView(_ view:WKWebView,coordinator:CoverageWebCoordinator){coordinator.destroy()}
}
#else
private struct CoverageWeb:UIViewRepresentable {
    var workspace:Workspace
    func makeCoordinator()->CoverageWebCoordinator {CoverageWebCoordinator(workspace)}
    func makeUIView(context:Context)->WKWebView {coverageWebView(context.coordinator)}
    func updateUIView(_ view:WKWebView,context:Context){}
    static func dismantleUIView(_ view:WKWebView,coordinator:CoverageWebCoordinator){coordinator.destroy()}
}
#endif
struct SimpleCoverageView:View {
    var workspace:Workspace
    var body:some View {VStack {if !workspace.coverage.notice.isEmpty {Text(workspace.coverage.notice).font(.caption)};CoverageWeb(workspace:workspace)}.frame(minWidth:320,idealWidth:740,maxWidth:900,minHeight:240,idealHeight:640,maxHeight:800)}
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/CoverageView.swift"><ui:paste select="#simple-coverage-apple"/></ui:file>
## Load the real bundled renderer

This focused WebKit witness loads the packaged shared renderer, waits for its actual dialog, rejects messages for another workspace and retires a replaced host. Its source buffer retains unsaved bytes and selection. This is transport/rendering evidence, not native pointer or IME acceptance or a collector execution.

<ui:copy id="simple-coverage-webkit-witness">
import XCTest
import WebKit
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CoverageViewTests:XCTestCase {
    #if os(macOS)
    func testBundledCoverageSurfaceUsesWorkspaceBoundNativeMessages()throws {
        let workspace=Workspace(),state=workspace.coverage
        let source="let value = 1\n",file=FileNode(name:"draft.swift",path:"draft.swift",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:nativeSourceHash(source)))
        workspace.buffers[file.path]=buffer;buffer.text+="// unsaved\n";buffer.selection=NSRange(location:4,length:2)
        let coordinator=CoverageWebCoordinator(workspace),view=coverageWebView(coordinator)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:740,height:640),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=view
        defer{coordinator.destroy();window.orderOut(nil)}
        func pump(_ condition:()->Bool){let deadline=Date().addingTimeInterval(15);while !condition()&&Date()<deadline{RunLoop.current.run(until:Date().addingTimeInterval(0.02))}}
        func evaluate(_ script:String)throws->Any?{var result:Result<Any?,Error>?;view.evaluateJavaScript(script){value,error in result=error.map(Result.failure) ?? .success(value)};pump{result != nil};return try XCTUnwrap(result).get()}
        pump{coordinator.ready};XCTAssertTrue(coordinator.ready,"Bundled coverage renderer must load in actual WebKit")
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
</ui:copy>

<ui:file path="apps/apple-simple/Tests/CoverageViewTests.swift"><ui:paste select="#simple-coverage-webkit-witness"/></ui:file>
</ui:doc>
