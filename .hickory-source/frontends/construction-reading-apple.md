<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read captured compiler constructions

For maintainers changing workspace services independently of the editor renderer.

The reading owns a replaceable read adapter and retires pending replies on close or a newer request. Check the captured entry and focus identity before accepting a response. The compiler owns contribution ranges and output changes; the frontend does not infer test execution from declarations or authorize source edits from these facts.

<ui:copy id="construction-reading-apple-model">
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
 func read(entryOverride:String?=nil,load:(String,String?)async throws->ConstructionReading)async {
  let entry=entryOverride ?? entry.trimmingCharacters(in:.whitespacesAndNewlines),focus=focus.trimmingCharacters(in:.whitespacesAndNewlines);let expectedFocus=focus.isEmpty ? nil:focus
  let id=UUID();ticket=id;value=nil;loading=true;notice="Reading construction…";defer{if ticket==id{loading=false}}
  do{let reply=try await load(entry,expectedFocus);guard ticket==id else{return};guard reply.version==1,exactUTF8(reply.entry,entry),reply.focus.map({Array($0.utf8)})==expectedFocus.map({Array($0.utf8)}),!reply.materialized else{throw EngineError(message:"Construction identity changed or unsupported response. Refresh the reading.")};value=reply;selected="";notice="Captured construction and draft consequences. Editor changes after capture are not included."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
</ui:copy>

## Compare captured outputs without resizing the editor

Use the existing floating surface and menus. A blank field reads the bound
workspace entry. A typed address retains its literal bytes. Capture the draft
roster and original intent before requesting the workspace entry. Changing
the address, contributor, or captured source retires the pending reading.

Label the entry, focus, preview side, binary output and every truncation.
Only service-supplied source points with matching acquired source hashes and
valid scalar positions enable navigation. Closing restores source focus and
retires the request.

<ui:copy id="construction-reading-apple-view">
@MainActor private final class ConstructionEntryIntent {var id=UUID()}
struct ConstructionReadingView:View {
 var workspace:Workspace;@State private var model=ConstructionReadingModel();@State private var connected=ConnectedBehaviorModel()
 @State private var address=""
 @State private var readingIntent=ConstructionEntryIntent()
 @State private var acquiring=false

 @MainActor private func read(connectedOnly:Bool=false)async {
  guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return}
  let id=UUID(),epoch=workspace.sourceNavigationEpoch
  let namespace=workspace.sourceRootAddress,runtime=workspace.sourceWorkspaceID
  let sourceCurrent=captureRetainedSourceNamespace(workspace)
  let explicit=address.trimmingCharacters(in:.whitespacesAndNewlines).isEmpty ? "":address
  let focus=model.focus,reading=readingIntent
  reading.id=id
  defer{if reading.id==id{acquiring=false}}
  let current={ [weak workspace=workspace,weak reading,weak model=model] in
   guard let workspace,reading?.id==id,model?.focus==focus else{return false}
   return workspace.showConstruction && workspace.sourceNavigationEpoch == epoch
    && workspace.engine?.session === engine.session
    && workspace.sourceRootAddress.map{Data($0.utf8)} == namespace.map{Data($0.utf8)}
    && workspace.sourceWorkspaceID == runtime
    && sourceCurrent()
  }
  if connectedOnly{connected.close()}else{model.close()}
  do {
   var entry=explicit
   var proof:WorkspaceEntrySelectionProof?
   if explicit.isEmpty {
    guard let namespace,let runtime else{throw EngineError(message:"Refresh Files to bind the current workspace entry.")}
    acquiring=true;model.notice="Reading the current workspace entry…"
    let selected=try await WorkspaceEntrySelections.read(load:{
     let reply:ResourceJSON=try await engine.request("GET","/api/workspace/entry")
     return try JSONSerialization.jsonObject(with:JSONEncoder().encode(reply))
    },workspace:runtime,namespace:namespace,current:current)
    guard let path=selected.entryPath else{throw EngineError(message:"This workspace has no default construction entry. Enter an authored path.")}
    entry=path;proof=selected.proof
   }
   guard current() else{return}
   acquiring=false
   if connectedOnly {
    await connected.read(workspace,entry:entry,focus:focus.isEmpty ? nil:focus,entrySelection:proof,original:current)
   }else{
    model.entry=entry
    let sources=Dictionary(uniqueKeysWithValues:workspace.buffers.values.filter{!$0.readOnly}.map{($0.file.path,$0.text)})
    await model.read(entryOverride:entry){entry,focus in
     var body:[String:Any]=["entry":entry,"sources":sources,"focus":focus as Any? ?? NSNull()]
     if let proof{body["entry_selection"]=try proof.body()}
     let result:ConstructionReading=try await engine.request("POST","/api/construction/reading",body:body)
     guard current() else{throw EngineError(message:"Original construction selection changed. Refresh the reading.")}
     return result
    }
   }
  }catch{if readingIntent.id==id{model.notice=error.localizedDescription}}
 }

 private func retireReading() {
  readingIntent.id=UUID();acquiring=false;model.close();connected.close();model.notice=""
 }
 private func openPoint(_ point:TestSourcePoint)async {
  let current=model.sourceLink(),epoch=workspace.sourceNavigationEpoch
  let valid={current()&&workspace.showConstruction&&workspace.sourceNavigationEpoch==epoch}
  guard valid() else { return }
  let prepared = workspace.beginSourceNavigationPreparation()
  func find(_ files:[FileNode])->FileNode?{for file in files{if exactUTF8(file.path,point.path){return file};if let child=find(file.children ?? []){return child}};return nil}
  do{
   guard let file=workspace.buffers.values.first(where:{exactUTF8($0.file.path,point.path)})?.file ?? find(workspace.rootFiles),!file.dir else{throw EngineError(message:"Source no longer listed. Refresh the construction reading.")}
   let buffer=try await workspace.loadBuffer(file),source=buffer.text,revision=buffer.sourceEditIdentity
   let hash=await Task.detached(priority:.userInitiated){nativeSourceHash(source)}.value
   guard valid(), prepared() else{return}
   let sourceCurrent = { valid() && workspace.buffers[point.path] === buffer && buffer.sourceEditIdentity == revision }
   guard sourceCurrent(),exactUTF8(buffer.file.path,point.path),constructionPointOffset(source,point,actualHash:hash) != nil else{throw EngineError(message:"Source changed or position is unavailable. Refresh the construction reading.")}
   await workspace.openSemanticLocation(SemanticLocation(uri:languageURI(point.path),line:point.line-1,character:point.character),allowDirty:true,expectedSource:source,isCurrent:sourceCurrent,onRevealed:{if valid(){workspace.showConstruction=false}})
  }catch{if valid(), prepared(){model.notice=error.localizedDescription}}
 }
 var body:some View {VStack(alignment:.leading,spacing:8){
  HStack{Text("Construction reading").font(.headline);Spacer();Button("Close"){workspace.showConstruction=false;workspace.currentBuffer?.focusSource?()}}
  TextField("Construction entry",text:Binding(get:{address},set:{retireReading();address=$0}),prompt:Text("Current workspace entry"))
  TextField("Contributor or output (optional)",text:Binding(get:{model.focus},set:{retireReading();model.focus=$0}))
  Button("Read captured drafts"){Task{await read()}}.disabled(reason: model.loading ? "Wait for the current reading to load." : acquiring ? "A source capture is in progress." : nil)
  Button("Read connected behavior and relevant verification"){Task{await read(connectedOnly:true)}}.disabled(reason: connected.busy ? "An operation is already in progress." : acquiring ? "A source capture is in progress." : nil)
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
  }
   ConnectedBehaviorView(workspace:workspace,model:connected)
  }}
 }.padding().frame(idealWidth:680,idealHeight:600).onDisappear{readingIntent.id=UUID();model.close();connected.close()}}
}
</ui:copy>

## Validate a captured point before moving the editor

A source link needs a matching SHA-256 value computed over the acquired text. Require a positive line, an in-range UTF-16 column and a scalar boundary; never clamp a stale point into the document. File identity, workspace lifetime and captured-reading lifetime are checked separately by the navigation adapter.

<ui:copy id="construction-reading-apple-source-point">
func constructionPointOffset(_ source:String,_ point:TestSourcePoint,actualHash:String)->Int? {
 guard point.hash.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,point.hash==actualHash,point.line>0,point.character>=0,let offset=languageOffset(source,["line":point.line-1,"character":point.character]) else{return nil}
 let units=Array(source.utf16)
 if offset>0,offset<units.count,(0xd800...0xdbff).contains(units[offset-1]),(0xdc00...0xdfff).contains(units[offset]){return nil}
 return offset
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ConstructionReading.swift"><ui:paste select="#construction-reading-apple-model"/><ui:paste select="#construction-reading-apple-view"/><ui:paste select="#construction-reading-apple-source-point"/></ui:file>
## Retire replaced and closed readings

Release held compiler replies after a newer reading or after closing. Neither may replace the retained result. A different entry refuses instead of appearing under the selected construction. Source-point cases also require exact hashes and Unicode scalar boundaries, and a newer link retires an older link. These service witnesses do not establish native physical input or assistive technology acceptance.

<ui:copy id="construction-reading-apple-lifetime-witness">
import XCTest
@testable import HickorySimple
@MainActor final class ConstructionReadingTests:XCTestCase {
 func testBoundEntryOverridePreservesLiteralAddress()async {
  let model=ConstructionReadingModel()
  let address=" folder/entry.md "
  await model.read(entryOverride:address){entry,focus in
   XCTAssertEqual(Array(entry.utf8),Array(address.utf8))
   XCTAssertNil(focus)
   return self.value(entry)
  }
  XCTAssertEqual(model.value?.entry,address)
 }
 func value(_ entry:String="project.md",materialized:Bool=false)->ConstructionReading{ConstructionReading(version:1,entry:entry,materialized:materialized,outputs:[],output_count:0,outputs_truncated:false,tests:[],test_count:0,tests_truncated:false,test_relationship:"declaration",evidence_status:"not_queried",notices:[],unused_sources:[])}
 func testClosedAndSupersededRepliesCannotReplaceTheCapturedReading()async{
  let model=ConstructionReadingModel();var pending:CheckedContinuation<ConstructionReading,Never>?
  let old=Task{await model.read{_,_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.read{_,_ in var reply=self.value();reply.notices=["new"];return reply};pending?.resume(returning:value());await old.value;XCTAssertEqual(model.value?.notices,["new"])
  pending=nil;let closed=Task{await model.read{_,_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:value());await closed.value;XCTAssertNil(model.value)
  await model.read{_,_ in self.value("other.md")};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("identity changed"))
  await model.read{_,_ in self.value(materialized:true)};XCTAssertNil(model.value)
 }
 func testSourcePointsRequireExactHashesAndScalarBoundariesAndRetireOlderLinks()async{
  let source="😀e\u{301}\né😀code",hash=nativeSourceHash(source)
  let point=TestSourcePoint(path:"part.md",line:2,character:3,hash:hash)
  XCTAssertEqual(constructionPointOffset(source,point,actualHash:hash),(source as NSString).range(of:"code").location)
  var split=point;split.character=2;XCTAssertNil(constructionPointOffset(source,split,actualHash:hash))
  split=point;split.line=0;XCTAssertNil(constructionPointOffset(source,split,actualHash:hash))
  split=point;split.character=99;XCTAssertNil(constructionPointOffset(source,split,actualHash:hash))
  XCTAssertNil(constructionPointOffset(source,point,actualHash:String(repeating:"b",count:64)))
  let model=ConstructionReadingModel();await model.read{_,_ in self.value()};let old=model.sourceLink(),latest=model.sourceLink();XCTAssertFalse(old());XCTAssertTrue(latest());model.close();XCTAssertFalse(latest())
 }

}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ConstructionReadingTests.swift"><ui:paste select="#construction-reading-apple-lifetime-witness"/></ui:file>
</ui:doc>
