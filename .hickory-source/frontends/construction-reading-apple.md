<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read captured compiler constructions

For maintainers changing workspace services independently of the editor renderer.

The reading owns a replaceable read adapter and retires pending replies on close or a newer request. Check the captured entry and focus identity before accepting a response. The compiler owns contribution ranges and output changes; the frontend does not infer test execution from declarations or authorize source edits from these facts.

<ui:copy id="construction-reading-apple-model">
import SwiftUI
import Observation
struct ConstructedSnapshot:Decodable {var hash:String;var kind:String;var executable:Bool;var size:Int;var preview:String?;var preview_truncated:Bool}
struct ConstructionOrigin:Decodable {var phase:String;var path:String;var source_bytes:[Int];var output_bytes:[Int];var exact:Bool;var source_hash:String?}
struct ConstructionOutput:Decodable {var path:String;var change:String;var before:ConstructedSnapshot?;var after:ConstructedSnapshot?;var contributors:[String];var origins:[ConstructionOrigin];var origins_truncated:Bool}
struct ConstructionReading:Decodable {var version:Int;var entry:String;var focus:String?;var materialized:Bool;var outputs:[ConstructionOutput];var output_count:Int;var outputs_truncated:Bool;var tests:[SourceTestCandidate];var test_count:Int;var tests_truncated:Bool;var test_relationship:String;var evidence_status:String;var notices:[String];var unused_sources:[String]}
@MainActor @Observable final class ConstructionReadingModel {
 var entry="project.md";var focus="";var selected="";var after=true;var value:ConstructionReading?;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();loading=false;value=nil}
 func read(load:(String,String?)async throws->ConstructionReading)async {
  let entry=entry.trimmingCharacters(in:.whitespacesAndNewlines),focus=focus.trimmingCharacters(in:.whitespacesAndNewlines);let expectedFocus=focus.isEmpty ? nil:focus
  let id=UUID();ticket=id;value=nil;loading=true;notice="Reading construction…";defer{if ticket==id{loading=false}}
  do{let reply=try await load(entry,expectedFocus);guard ticket==id else{return};guard reply.version==1,exactUTF8(reply.entry,entry),reply.focus.map({Array($0.utf8)})==expectedFocus.map({Array($0.utf8)}),!reply.materialized else{throw EngineError(message:"Construction identity changed or unsupported response. Refresh the reading.")};value=reply;selected="";notice="Captured construction and draft consequences. Editor changes after capture are not included."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
</ui:copy>

## Compare captured outputs without resizing the editor

Use the existing floating surface and menus. Capture retained authored drafts only when the reader requests analysis. Label the entry, focus, preview side, binary output and every truncation. Contributor byte ranges are readable observations; this surface does not turn them into editor positions. Closing restores source focus and retires the request.

<ui:copy id="construction-reading-apple-view">
struct ConstructionReadingView:View {
 var workspace:Workspace;@State private var model=ConstructionReadingModel()
 private func read()async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};let sources=Dictionary(uniqueKeysWithValues:workspace.buffers.values.filter{!$0.readOnly}.map{($0.file.path,$0.text)})
  await model.read{entry,focus in try await engine.request("POST","/api/construction/reading",body:["entry":entry,"sources":sources,"focus":focus as Any? ?? NSNull()])}
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
    DisclosureGroup("Compiler contribution ranges"){ForEach(Array(output.origins.enumerated()),id:\.offset){_,origin in Text("\(origin.phase) · \(origin.path) bytes \(origin.source_bytes.map(String.init).joined(separator:"–")) → output bytes \(origin.output_bytes.map(String.init).joined(separator:"–")) · \(origin.exact ? "equal byte slices":"reported origin; exact bytes unconfirmed")").font(.caption)};if output.origins_truncated{Text("First 100 origins shown.").font(.caption)}}
   }
   DisclosureGroup("Detected test declarations"){ForEach(value.tests,id:\.id){test in Text("\(test.name) · \(test.source?.path ?? test.target.path):\(test.source?.line ?? test.target.line)").font(.caption)}}
  }}}
 }.padding().frame(idealWidth:680,idealHeight:600).onDisappear{model.close()}}
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ConstructionReading.swift"><ui:paste select="#construction-reading-apple-model"/><ui:paste select="#construction-reading-apple-view"/></ui:file>
## Retire replaced and closed readings

Release held compiler replies after a newer reading or after closing. Neither may replace the retained result. A different entry refuses instead of appearing under the selected construction. These service witnesses do not establish native physical input or assistive technology acceptance.

<ui:copy id="construction-reading-apple-lifetime-witness">
import XCTest
@testable import HickorySimple
@MainActor final class ConstructionReadingTests:XCTestCase {
 func value(_ entry:String="project.md",materialized:Bool=false)->ConstructionReading{ConstructionReading(version:1,entry:entry,materialized:materialized,outputs:[],output_count:0,outputs_truncated:false,tests:[],test_count:0,tests_truncated:false,test_relationship:"declaration",evidence_status:"not_queried",notices:[],unused_sources:[])}
 func testClosedAndSupersededRepliesCannotReplaceTheCapturedReading()async{
  let model=ConstructionReadingModel();var pending:CheckedContinuation<ConstructionReading,Never>?
  let old=Task{await model.read{_,_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.read{_,_ in var reply=self.value();reply.notices=["new"];return reply};pending?.resume(returning:value());await old.value;XCTAssertEqual(model.value?.notices,["new"])
  pending=nil;let closed=Task{await model.read{_,_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:value());await closed.value;XCTAssertNil(model.value)
  await model.read{_,_ in self.value("other.md")};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("identity changed"))
  await model.read{_,_ in self.value(materialized:true)};XCTAssertNil(model.value)
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ConstructionReadingTests.swift"><ui:paste select="#construction-reading-apple-lifetime-witness"/></ui:file>
</ui:doc>
