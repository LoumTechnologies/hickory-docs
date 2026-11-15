import SwiftUI
import Observation
struct ConstructedSnapshot:Decodable {var hash:String;var kind:String;var executable:Bool;var size:Int;var preview:String?;var preview_truncated:Bool}
struct ConstructionFragment:Decodable {var name:String;var kind:String;var body:[Int]}
struct ConstructionOrigin:Decodable {var phase:String;var path:String;var source_bytes:[Int];var output_bytes:[Int];var exact:Bool;var source_hash:String?;var source_point:TestSourcePoint?;var fragment:ConstructionFragment?}
struct ConstructionOutput:Decodable {var path:String;var change:String;var before:ConstructedSnapshot?;var after:ConstructedSnapshot?;var contributors:[String];var origins:[ConstructionOrigin];var origins_truncated:Bool}
struct ConstructionReading:Decodable {var version:Int;var entry:String;var focus:String?;var materialized:Bool;var outputs:[ConstructionOutput];var output_count:Int;var outputs_truncated:Bool;var tests:[SourceTestCandidate];var test_count:Int;var tests_truncated:Bool;var test_relationship:String;var evidence_status:String;var notices:[String];var unused_sources:[String]}
@MainActor @Observable final class ConstructionReadingModel {
 var entry="project.md";var focus="";var selected="";var after=true;var value:ConstructionReading?;var notice="";var loading=false;private var ticket=UUID();private var linkTicket=UUID()
 func sourceLink()->()->Bool {let reading=ticket,link=UUID();linkTicket=link;return {[weak self] in self?.ticket==reading&&self?.linkTicket==link&&self?.value != nil}}
 func close(){ticket=UUID();loading=false;value=nil}
 func read(load:(String,String?)async throws->ConstructionReading)async {
  let entry=entry.trimmingCharacters(in:.whitespacesAndNewlines),focus=focus.trimmingCharacters(in:.whitespacesAndNewlines);let expectedFocus=focus.isEmpty ? nil:focus
  let id=UUID();ticket=id;value=nil;loading=true;notice="Reading construction…";defer{if ticket==id{loading=false}}
  do{let reply=try await load(entry,expectedFocus);guard ticket==id else{return};guard reply.version==1,exactUTF8(reply.entry,entry),reply.focus.map({Array($0.utf8)})==expectedFocus.map({Array($0.utf8)}),!reply.materialized else{throw EngineError(message:"Construction identity changed or unsupported response. Refresh the reading.")};value=reply;selected="";notice="Captured construction and draft consequences. Editor changes after capture are not included."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
struct ConstructionReadingView:View {
 var workspace:Workspace;@State private var model=ConstructionReadingModel()
 private func read()async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};let sources=Dictionary(uniqueKeysWithValues:workspace.buffers.values.filter{!$0.readOnly}.map{($0.file.path,$0.text)})
  await model.read{entry,focus in try await engine.request("POST","/api/construction/reading",body:["entry":entry,"sources":sources,"focus":focus as Any? ?? NSNull()])}
 }
 private func openPoint(_ point:TestSourcePoint)async {
  let current=model.sourceLink(),epoch=workspace.sourceNavigationEpoch
  let valid={current()&&workspace.showConstruction&&workspace.sourceNavigationEpoch==epoch}
  func find(_ files:[FileNode])->FileNode?{for file in files{if exactUTF8(file.path,point.path){return file};if let child=find(file.children ?? []){return child}};return nil}
  do{
   guard let file=workspace.buffers.values.first(where:{exactUTF8($0.file.path,point.path)})?.file ?? find(workspace.rootFiles),!file.dir else{throw EngineError(message:"Source no longer listed. Refresh the construction reading.")}
   let buffer=try await workspace.loadBuffer(file),source=buffer.text
   let hash=await Task.detached(priority:.userInitiated){nativeSourceHash(source)}.value
   guard valid() else{return}
   guard exactUTF8(buffer.file.path,point.path),exactUTF8(buffer.text,source),constructionPointOffset(source,point,actualHash:hash) != nil else{throw EngineError(message:"Source changed or position is unavailable. Refresh the construction reading.")}
   await workspace.openSemanticLocation(SemanticLocation(uri:languageURI(point.path),line:point.line-1,character:point.character),allowDirty:true,expectedSource:source,isCurrent:valid,onRevealed:{if valid(){workspace.showConstruction=false}})
  }catch{if valid(){model.notice=error.localizedDescription}}
 }
 var body:some View {VStack(alignment:.leading,spacing:8){
  HStack{Text("Construction reading").font(.headline);Spacer();Button("Close"){workspace.showConstruction=false;workspace.currentBuffer?.focusSource?()}}
  TextField("Construction entry",text:$model.entry);TextField("Contributor or output (optional)",text:$model.focus);Button("Read captured drafts"){Task{await read()}}.disabled(model.loading||model.entry.isEmpty)
  Text(model.notice).font(.caption).textSelection(.enabled)
  ScrollView{VStack(alignment:.leading,spacing:8){if let value=model.value{
   Text("Captured entry: \(value.entry) · focus: \(value.focus ?? "all")").font(.caption)
   Text("\(value.output_count) related outputs\(value.outputs_truncated ? " · first 300 shown":""). \(value.test_count) detected tests\(value.tests_truncated ? " · test list truncated":""). Evidence: \(value.evidence_status). Outputs have not been written.").font(.caption)
   Text(value.test_relationship).font(.caption);ForEach(Array(value.notices.enumerated()),id:\.offset){_,notice in Text(notice).font(.caption)}
   if !value.unused_sources.isEmpty{Text("\(value.unused_sources.count) supplied drafts did not contribute to this entry.").font(.caption)}
   ForEach(value.outputs,id:\.path){output in Button("\(output.change) · \(output.path)"){model.selected=output.path}.buttonStyle(.plain)}
   if let output=value.outputs.first(where:{$0.path==model.selected}) ?? value.outputs.first{
    Text("\(output.change) · \(output.path)").font(.headline);Text("Contributors: \(output.contributors.joined(separator:", "))").font(.caption)
    HStack{Button("Saved construction"){model.after=false};Button("Draft construction"){model.after=true}}
    if let snapshot=model.after ? output.after:output.before{Text("\(snapshot.kind) · \(snapshot.size) bytes · executable \(String(snapshot.executable))").font(.caption);Text(snapshot.preview ?? "Binary output; text preview unavailable.").font(.system(.caption,design:.monospaced)).textSelection(.enabled);if snapshot.preview_truncated{Text("Preview abbreviated.").font(.caption)}}else{Text("Output absent from this construction.")}
    DisclosureGroup("Compiler contribution ranges"){ForEach(Array(output.origins.enumerated()),id:\.offset){_,origin in Text("\(origin.phase) · \(origin.path) bytes \(origin.source_bytes.map(String.init).joined(separator:"–")) → output bytes \(origin.output_bytes.map(String.init).joined(separator:"–")) · \(origin.exact ? "equal byte slices":"reported origin; exact bytes unconfirmed")").font(.caption);if let fragment=origin.fragment{Text("Declared \(fragment.kind) #\(fragment.name)").font(.caption)};if let point=origin.source_point{Button("Open contributor"){Task{await openPoint(point)}}.accessibilityLabel("Open \(origin.phase) contributor \(origin.path)")}};if output.origins_truncated{Text("First 100 origins shown.").font(.caption)}}
   }
   DisclosureGroup("Detected test declarations"){ForEach(value.tests,id:\.id){test in Text("\(test.name) · \(test.source?.path ?? test.target.path):\(test.source?.line ?? test.target.line)").font(.caption)}}
  }}}
 }.padding().frame(idealWidth:680,idealHeight:600).onDisappear{model.close()}}
}
func constructionPointOffset(_ source:String,_ point:TestSourcePoint,actualHash:String)->Int? {
 guard point.hash.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,point.hash==actualHash,point.line>0,point.character>=0,let offset=languageOffset(source,["line":point.line-1,"character":point.character]) else{return nil}
 let units=Array(source.utf16)
 if offset>0,offset<units.count,(0xd800...0xdbff).contains(units[offset-1]),(0xdc00...0xdfff).contains(units[offset]){return nil}
 return offset
}
