import SwiftUI
import Observation
@MainActor @Observable final class TestSourcesModel {
    let identity=UUID().uuidString
    var entry="";var query="";var page=0;var reading=false;var running=false;var notice=""
    var observation:(TestSourceSnapshot,SourceTestScan)?
    struct Run:Identifiable {var id:String;var name:String;var scope:String;var state:String;var exit:Int?;var output="";var sourceHash:String;var fingerprint:String;var contributors:[String:String]}
    var runs:[Run]=[];var selected:String?;private var ticket=UUID();private var outputTickets:[String:UUID]=[:]
    func snapshot(_ workspace:Workspace)->TestSourceSnapshot? {
        guard let buffer=workspace.currentBuffer,let engine=workspace.engine else {return nil}
        let entry=entry.trimmingCharacters(in:.whitespacesAndNewlines)
        let sources=Dictionary(uniqueKeysWithValues:workspace.buffers.values.filter {!$0.readOnly}.map {($0.file.path,$0.text)})
        return TestSourceSnapshot(workspace:engine.origin.absoluteString+identity,request:TestSourceRequest(path:buffer.file.path,source:buffer.text,projectEntry:entry.isEmpty ? nil:entry,sources:sources))
    }
    func current(_ workspace:Workspace)->Bool {guard let (before,facts)=observation,let now=snapshot(workspace) else {return false};return testSourceCurrent(before,facts,now)}
    func refresh(_ workspace:Workspace) async {
        guard let before=snapshot(workspace),let engine=workspace.engine else {notice="Open a source file first.";return}
        let id=UUID();ticket=id;reading=true;observation=nil;notice="";defer {if ticket==id {reading=false}}
        do {let facts:SourceTestScan=try await engine.request("POST","/api/tests/source-candidates",body:before.request.body)
            guard ticket==id,workspace.testSources === self else {return}
            guard let now=snapshot(workspace),testSourceCurrent(before,facts,now) else {notice="Source changed during inspection. Refresh candidates.";return}
            observation=(before,facts);page=0
        }catch {if ticket==id,workspace.testSources === self {notice=error.localizedDescription}}
    }
    func run(_ candidate:SourceTestCandidate,_ selection:TestSourceSelection,workspace:Workspace) async {
        guard !running,let (before,facts)=observation,let now=snapshot(workspace),let engine=workspace.engine else {return}
        running=true;notice="";defer {running=false}
        struct Response:Decodable {var id:String;var state:String;var exit_code:Int?;var source_hash:String;var analysis_fingerprint:String;var checked_sources:[String:String]}
        do {let body=try sourceTestRun(before,facts,now,id:candidate.id,selection:selection)
            let reply:Response=try await engine.request("POST","/api/tests/run-source",body:body)
            runs.insert(Run(id:reply.id,name:candidate.name,scope:selection == .nameFilter ? "name filter":candidate.run_scope,state:reply.state,exit:reply.exit_code,sourceHash:reply.source_hash,fingerprint:reply.analysis_fingerprint,contributors:reply.checked_sources),at:0);selected=reply.id
            notice="Run admitted against saved bytes. Files remain mutable while it executes."
        }catch {notice=error.localizedDescription}
    }
    func refreshRun(_ id:String,workspace:Workspace) async {
        let read=UUID();outputTickets[id]=read
        guard let engine=workspace.engine else {return}
        struct Summary:Decodable {var id:String;var state:String;var exit_code:Int?}
        struct Sessions:Decodable {var sessions:[Summary]}
        do {async let sessions:Sessions=engine.request("GET","/api/terminals")
            async let screen:TerminalPage=engine.request("GET","/api/terminals/"+id+"/output",query:[URLQueryItem(name:"view",value:"screen"),URLQueryItem(name:"offset",value:"0"),URLQueryItem(name:"limit",value:"16384")])
            let (list,output)=try await (sessions,screen)
            guard outputTickets[id]==read,workspace.testSources === self,let index=runs.firstIndex(where:{$0.id==id}) else {return}
            let summary=list.sessions.first(where:{$0.id==id});runs[index].state=summary?.state ?? "session unavailable";runs[index].exit=summary?.exit_code
            runs[index].output=String(decoding:output.bytes,as:UTF8.self)+(output.next == nil ? "":"\n[Screen page truncated. Open terminal for retained output.]")
        }catch {if outputTickets[id]==read,workspace.testSources === self {notice=error.localizedDescription}}
    }
}
extension Workspace {
    func navigateTestPoint(_ point:TestSourcePoint) async {
        func find(_ files:[FileNode])->FileNode? {for file in files {if file.path==point.path {return file};if let child=find(file.children ?? []) {return child}};return nil}
        guard let file=buffers[point.path]?.file ?? find(rootFiles),!file.dir else {testSources.notice="Source no longer listed. Refresh files and candidates.";return}
        let model=testSources
        do {let buffer=try await loadBuffer(file);let source=buffer.text
            guard testSources === model,nativeSourceHash(source)==point.hash.lowercased() else {model.notice="Source changed. Refresh candidates before navigating.";return}
            await openSemanticLocation(SemanticLocation(uri:languageURI(point.path),line:point.line-1,character:point.character),allowDirty:true,expectedSource:source)
            if testSources === model {showTests=false}
        }catch {model.notice=error.localizedDescription}
    }
}
struct TestSourcesView:View {
    var workspace:Workspace
    private var model:TestSourcesModel {workspace.testSources}
    var body:some View {
        @Bindable var model=model
        let valid=model.current(workspace)
        let candidates=(model.observation?.1.candidates ?? []).filter {model.query.isEmpty || ($0.name+" "+$0.target.path).localizedCaseInsensitiveContains(model.query)}
        let last=max(0,(candidates.count-1)/100),page=min(model.page,last)
        ScrollView {VStack(alignment:.leading,spacing:8) {
            HStack {Text("Source tests").font(.headline);Spacer();Button("Close"){workspace.showTests=false;workspace.currentBuffer?.focusSource?()}}
            Text("Source declarations are candidates. Name filters may match several cases; parameterized cases need runner discovery. Terminal runs are separate from per-case verification.").font(.caption)
            Text(workspace.currentBuffer?.file.path ?? "No source selected").font(.caption)
            TextField("Construction entry (optional)",text:$model.entry).textFieldStyle(.roundedBorder)
            Button(model.reading ? "Reading source…":"Refresh candidates"){Task {await model.refresh(workspace)}}.disabled(model.reading || workspace.currentBuffer==nil)
            if let (_,facts)=model.observation {Text("\(valid ? "Source observation":"Stale observation — refresh before running") · \(facts.scope) · \(facts.syntax_complete ? "parsed":"partial syntax")").font(.caption)
                ForEach(facts.notices+facts.unsupported_languages.map {"No source detector: "+$0}+facts.unclassified_outputs.map {"Unclassified output: "+$0},id:\.self){Text($0).font(.caption)}
            }
            if !model.notice.isEmpty {Text(model.notice).font(.caption).textSelection(.enabled).accessibilityLabel("Test notice")}
            TextField("Filter source tests",text:$model.query).textFieldStyle(.roundedBorder).onChange(of:model.query){_,_ in model.page=0}
            ForEach(Array(candidates.dropFirst(page*100).prefix(100))) {item in VStack(alignment:.leading) {
                Text(item.name).fontWeight(.semibold);Text("\(item.confidence) · \(item.target.path):\(item.target.line)").font(.caption)
                Button("\(item.source == nil ? "Open runtime source":"Open authored source") \(item.point.line)"){Task {await workspace.navigateTestPoint(item.point)}}.disabled(!valid)
                HStack {Button("Run name filter"){Task {await model.run(item,.nameFilter,workspace:workspace)}}.disabled(!valid || model.running || item.name_filter==nil)
                    Button("Run \(item.run_scope.replacingOccurrences(of:"_",with:" ")) scope"){Task {await model.run(item,.containingScope,workspace:workspace)}}.disabled(!valid || model.running)}
            }}
            if model.observation != nil,candidates.isEmpty {Text("No source candidates returned. This does not prove the project has no tests.").font(.caption)}
            if last>0 {HStack {Button("Previous"){model.page=page-1}.disabled(page==0);Text("\(page+1)/\(last+1)");Button("Next"){model.page=page+1}.disabled(page==last)}}
            Divider();Text("Retained test runs (\(model.runs.count))").font(.headline)
            ForEach(model.runs) {run in Button("\(run.name) · \(run.scope) · \(run.state)\(run.exit.map {" · exit \($0)"} ?? "")"){model.selected=run.id;Task {await model.refreshRun(run.id,workspace:workspace)}}}
            if let run=model.runs.first(where:{$0.id==model.selected}) {
                HStack {Button("Refresh this run"){Task {await model.refreshRun(run.id,workspace:workspace)}};Button("Open terminal output"){workspace.terminal.id=run.id;workspace.terminal.error="";workspace.show("terminal");workspace.showTests=false}}
                DisclosureGroup("Saved revisions checked at admission") {Text("Files were not frozen during execution. These revisions identify this run, not current verification.").font(.caption);Text(run.fingerprint+"\nTarget: "+run.sourceHash+"\n"+run.contributors.keys.sorted().map {"\($0): \(run.contributors[$0]!)"}.joined(separator:"\n")).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}
                Text(run.output.isEmpty ? "Select or refresh this run to read its output.":run.output).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
            }
        }.padding()}.frame(minWidth:320,idealWidth:600,maxWidth:760,minHeight:240,idealHeight:600,maxHeight:760)
    }
}
