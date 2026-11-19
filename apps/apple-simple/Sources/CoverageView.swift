import SwiftUI
import WebKit
import Observation
// Several engine values can share URLSession.shared. The original API origin
// remains part of coverage reply and reader ownership as well as the session.
func coverageEngineMatches(_ current:Engine?,_ original:Engine?)->Bool {
    guard let current,let original else{return false}
    return current.session === original.session && exactUTF8(current.origin.absoluteString,original.origin.absoluteString)
}
@MainActor @Observable final class SimpleCoverageState {
    let identity=UUID().uuidString
    var selected:String?;var notice=""
    let digests=SourceDigests()
    private var record:CoverageProjectionRecord?
    private var projected:[String:[CoverageProjectedLine]]=[:]
    private var ticket=UUID()
    func stop(){ticket=UUID();digests.stop();record=nil;projected=[:]}
    func select(_ id:String?,workspace:Workspace){
        ticket=UUID();let read=ticket;selected=id;record=nil;projected=[:];notice=""
        guard let id,let engine=workspace.engine else{return}
        Task {do {let value:CoverageProjectionRecord=try await engine.request("POST","/api/tests/coverage",body:["action":"get","id":id])
            let rows=await Task.detached {coverageProjection(value)}.value
            guard workspace.coverage === self,coverageEngineMatches(workspace.engine,engine),ticket==read,value.id==id else{return};record=value;projected=rows
            for buffer in workspace.buffers.values {observe(buffer)}
        }catch{if workspace.coverage === self,coverageEngineMatches(workspace.engine,engine),ticket==read {notice=error.localizedDescription}}}
    }
    func observe(_ buffer:DocumentBuffer){guard projected[buffer.file.path] != nil else{return};digests.enqueue(buffer.file.path,source:buffer.text,current:{[weak buffer] in buffer?.text})}
    func rows(_ buffer:DocumentBuffer)->[CoverageProjectedLine]{
        guard let expected=record?.identity.source_hashes[buffer.file.path],digests.current(buffer.file.path,source:buffer.text)==expected.lowercased() else{return []}
        return projected[buffer.file.path] ?? []
    }
}
extension DocumentBuffer {
    func coverageRows()->[CoverageProjectedLine]{coverageSource?.rows(self) ?? []}
    func readCoverageSource(){coverageSource?.observe(self)}
}
@MainActor extension Workspace {
    func navigateCoverage(_ value:[String:Any],state:SimpleCoverageState,isCurrent:@escaping ()->Bool={true}) async throws {
        let server=engine,epoch=sourceNavigationEpoch
        let alive={isCurrent() && self.coverage === state && coverageEngineMatches(self.engine,server) && self.sourceNavigationEpoch==epoch}
        guard alive(),let path=value["path"] as? String else {throw EngineError(message:"Workspace or coverage reader changed. Reopen coverage.")}
        let prepared = beginSourceNavigationPreparation()
        func find(_ files:[FileNode])->FileNode? {for file in files {if exactUTF8(file.path,path) {return file};if let child=find(file.children ?? []) {return child}};return nil}
        guard let file=buffers[path]?.file ?? find(rootFiles),!file.dir else {throw EngineError(message:"Source no longer listed. Refresh the workspace.")}
        let buffer: DocumentBuffer
        do { buffer = try await loadBuffer(file) }
        catch { if prepared() { throw error }; return }
        guard prepared() else { return }
        let source=buffer.text,identity=buffer.sourceEditIdentity,selection=buffer.selection
        guard alive(),buffers[path] === buffer else {throw EngineError(message:"Workspace or source editor changed. Reopen coverage.")}
        let sourceCurrent={alive() && self.buffers[path] === buffer && buffer.sourceEditIdentity==identity && exactUTF8(buffer.text,source) && buffer.selection==selection}
        if value["line"]==nil || value["line"] is NSNull || value["sourceHash"]==nil || value["sourceHash"] is NSNull {
            guard sourceCurrent() else{throw EngineError(message:"Source or coverage reader changed. Reopen coverage.")}
            let departed=currentSemanticLocation();showSource(path);if let arrived=currentSemanticLocation(){navigationTrail.record(from:departed,to:arrived)};showCoverage=false;return
        }
        guard let line=sourceChangeInteger(value["line"]),let hash=value["sourceHash"] as? String else{throw EngineError(message:"Malformed coverage source position. Reopen the report.")}
        let actual=await Task.detached {nativeSourceHash(source)}.value
        guard prepared() else { return }
        guard sourceCurrent(),actual==hash.lowercased(),line>0,line<=source.components(separatedBy:"\n").count else {throw EngineError(message:"Source changed or report position is unavailable. Refresh coverage before navigating.")}
        var revealed=false
        await openSemanticLocation(SemanticLocation(uri:languageURI(path),line:line-1,character:0),allowDirty:true,expectedSource:source,isCurrent:sourceCurrent,onRevealed:{revealed=true})
        guard alive(),buffers[path] === buffer,revealed else{throw EngineError(message:"Coverage source navigation did not complete. The report remains readable.")};showCoverage=false
    }
}
@MainActor final class CoverageWebCoordinator:NSObject,WKScriptMessageHandler,WKNavigationDelegate {
    weak var view:WKWebView?;let workspace:Workspace;let state:SimpleCoverageState;let engine:Engine?
    var allowedURL:URL?;var mounted=true;var ready=false;var active:Set<Int>=[]
    init(_ workspace:Workspace){self.workspace=workspace;state=workspace.coverage;engine=workspace.engine}
    var current:Bool {mounted && workspace.coverage === state && coverageEngineMatches(workspace.engine,engine)}
    func display(){guard ready,current else{return};let buffer=workspace.currentBuffer
        send("update",["workspace":state.identity,"path":buffer?.file.path as Any? ?? NSNull(),"language":buffer?.file.language ?? "unsupported","selectedId":state.selected as Any? ?? NSNull()])
    }
    func send(_ method:String,_ value:Any){guard current,let data=try? JSONSerialization.data(withJSONObject:value,options:[.fragmentsAllowed]),let json=String(data:data,encoding:.utf8) else{return};view?.evaluateJavaScript("globalThis.hickoryCoverage?."+method+"("+json+")",completionHandler:nil)}
    func reply(_ id:Int,_ result:Any,_ error:String?){guard current,let data=try? JSONSerialization.data(withJSONObject:[id,result,error as Any? ?? NSNull()]),let json=String(data:data,encoding:.utf8) else{return};view?.evaluateJavaScript("globalThis.hickoryCoverage?.reply(..."+json+")",completionHandler:nil)}
    func userContentController(_ controller:WKUserContentController,didReceive message:WKScriptMessage){
        guard current,message.frameInfo.isMainFrame,message.frameInfo.request.url==allowedURL,let event=message.body as? [String:Any],let kind=event["kind"] as? String else{return}
        if kind=="ready" {ready=true;display();return}
        guard event["workspace"] as? String==state.identity else{return}
        if kind=="close" {mounted=false;workspace.showCoverage=false;workspace.currentBuffer?.focusSource?();return}
        if kind=="select" {state.select(event["id"] as? String,workspace:workspace);return}
        guard let id=sourceChangeInteger(event["id"]),id>0,!active.contains(id),active.count<64 else{return};active.insert(id)
        Task {[weak self] in guard let self else{return};defer{active.remove(id)}
            do {guard current else{return}
                if kind=="api",let body=event["body"] as? [String:Any],let action=body["action"] as? String,["list","get","locations","inspect","discover","run","rerun-failed","manual-start","browser-checkpoint","checkpoint","stop","import","compare","annotate"].contains(action),let engine {
                    let result:NativeTableJSON=try await engine.request("POST","/api/tests/coverage",body:body);reply(id,result.object,nil)
                }else if kind=="navigate",let location=event["location"] as? [String:Any] {try await workspace.navigateCoverage(location,state:state,isCurrent:{[weak self] in self?.current==true});reply(id,NSNull(),nil)}
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
