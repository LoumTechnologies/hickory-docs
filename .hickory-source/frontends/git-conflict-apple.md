<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect immutable Git conflict stages

For maintainers changing workspace services independently of the editor renderer.

Conflict inspection is an optional capability inside the existing repository review. It validates literal source identity, protocol version, immutable object names, stage uniqueness and explicit text availability. A fixed read ticket retires held/closed/foreign replies. Stage labels preserve Git’s meaning during rebase. Source text and index remain separate from this observation; this view does not publish a resolution. Page the retained stage text without reserving code space, and disclose a cap for a very long page.

<ui:copy id="git-conflict-apple">
import SwiftUI
import Observation
struct ConflictStage:Decodable {var stage:Int;var mode:String;var oid:String;var text:String?;var unavailable:String?}
struct ConflictReading:Decodable {var version:Int;var path:String;var fingerprint:String;var stages:[ConflictStage];var index_modified:Bool;var source_files_modified:Bool}
func validateConflict(_ value:ConflictReading,path:String)throws {
 func hex(_ value:String,_ lengths:[Int])->Bool{lengths.contains(value.utf8.count)&&value.utf8.allSatisfy{(48...57).contains($0)||(65...70).contains($0)||(97...102).contains($0)}}
 guard value.version==1,exactUTF8(value.path,path),!value.index_modified,!value.source_files_modified,hex(value.fingerprint,[64]),(1...3).contains(value.stages.count) else{throw EngineError(message:"Conflict identity changed or unsupported response. Refresh the conflict.")}
 var seen=Set<Int>()
 for stage in value.stages{guard (1...3).contains(stage.stage),seen.insert(stage.stage).inserted,hex(stage.oid,[40,64]),stage.mode.utf8.count==6,stage.mode.utf8.allSatisfy({(48...55).contains($0)}),(stage.text != nil && stage.unavailable==nil && ["100644","100755"].contains(stage.mode))||(stage.text==nil && stage.unavailable != nil) else{throw EngineError(message:"Malformed conflict stage. Refresh repository changes.")}}
}
@MainActor @Observable final class GitConflictModel {
 var value:ConflictReading?;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();loading=false;value=nil}
 func read(_ path:String,load:(String)async throws->ConflictReading)async{let id=UUID();ticket=id;value=nil;loading=true;notice="Reading unmerged stages…";defer{if ticket==id{loading=false}}
 do{let result=try await load(path);guard ticket==id else{return};try validateConflict(result,path:path);value=result;notice="Captured immutable Git stages. Working text and unsaved drafts are separate."}catch{if ticket==id{notice=error.localizedDescription}}}
}
struct GitConflictView:View {
 var workspace:Workspace;var path:String;@State private var model=GitConflictModel();@State private var selected=2;@State private var page=0
 var body:some View {
  VStack(alignment:.leading,spacing:6){
   Button("Inspect conflict stages"){Task{guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};page=0;await model.read(path){path in try await engine.request("GET","/api/git/conflict",query:[URLQueryItem(name:"path",value:path)])}}}.disabled(model.loading)
   if !model.notice.isEmpty{Text(model.notice).font(.caption)}
   if let value=model.value{
    Text("Stage 1: base · Stage 2 and Stage 3 retain Git’s meanings, including during rebase.").font(.caption)
    HStack{ForEach(value.stages,id:\.stage){stage in Button("Stage \(stage.stage)\(stage.stage==1 ? " · Base":"")"){selected=stage.stage;page=0}}}
    if let stage=value.stages.first(where:{$0.stage==selected}) ?? value.stages.first{
     Text("Stage \(stage.stage) · mode \(stage.mode)").font(.caption);Text(stage.oid).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
     if let text=stage.text{let rows=text.components(separatedBy:"\n"),last=max(0,(rows.count+99)/100-1),index=min(page,last),preview=rows.dropFirst(index*100).prefix(100).joined(separator:"\n")
      Text("Lines \(index*100+1)–\(min(rows.count,(index+1)*100))\(preview.count>200000 ? " · page preview capped at 200,000 characters":"")").font(.caption)
      ScrollView([.horizontal,.vertical]){Text(String(preview.prefix(200000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}.frame(maxHeight:220)
      HStack{Button("Previous stage lines"){page=index-1}.disabled(index==0);Button("Next stage lines"){page=index+1}.disabled(index==last)}
     }else{Text(stage.unavailable ?? "Text unavailable").font(.caption)}
    }
   }
  }.onDisappear{model.close()}
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/GitConflict.swift"><ui:paste select="#git-conflict-apple"/></ui:file>
## Retire stale conflict observations

Release held conflict reads out of order, close during a pending read, and supply a byte-distinct foreign path. Only the latest exact identity may display. Validate immutable object names, unique stages and explicit binary/text availability. These are service-model witnesses, not native physical input acceptance.

<ui:copy id="git-conflict-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class GitConflictTests:XCTestCase {
 func reading(_ path:String,_ text:String="captured")->ConflictReading{ConflictReading(version:1,path:path,fingerprint:String(repeating:"a",count:64),stages:[ConflictStage(stage:2,mode:"100644",oid:String(repeating:"b",count:40),text:text,unavailable:nil)],index_modified:false,source_files_modified:false)}
 func testOlderClosedAndForeignRepliesAreRetired()async throws{
  let model=GitConflictModel();var held:[CheckedContinuation<ConflictReading,Error>]=[]
  let load:(String)async throws->ConflictReading={_ in try await withCheckedThrowingContinuation{held.append($0)}}
  func settle(_ count:Int)async{let deadline=Date().addingTimeInterval(2);while held.count<count&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
  let first=Task{await model.read("a.txt",load:load)};await settle(1);let second=Task{await model.read("b.txt",load:load)};await settle(2);XCTAssertEqual(held.count,2);guard held.count==2 else{return}
  held[1].resume(returning:reading("b.txt","new"));await second.value;held[0].resume(returning:reading("a.txt","old"));await first.value;XCTAssertEqual(model.value?.path,"b.txt")
  let closed=Task{await model.read("a.txt",load:load)};await settle(3);model.close();held[2].resume(returning:reading("a.txt"));await closed.value;XCTAssertNil(model.value)
  await model.read("é.txt"){_ in self.reading("e\u{301}.txt")};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("identity changed"))
 }
 func testImmutableStageAndAvailabilityValidation()throws{
  let value=reading("a.txt");XCTAssertNoThrow(try validateConflict(value,path:"a.txt"))
  var duplicate=value;duplicate.stages.append(duplicate.stages[0]);XCTAssertThrowsError(try validateConflict(duplicate,path:"a.txt"))
  var bad=value;bad.stages[0].oid="main";XCTAssertThrowsError(try validateConflict(bad,path:"a.txt"))
  bad=value;bad.stages[0].mode="120000";XCTAssertThrowsError(try validateConflict(bad,path:"a.txt"))
  bad=value;bad.stages[0].text=nil;XCTAssertThrowsError(try validateConflict(bad,path:"a.txt"));bad.stages[0].unavailable="Binary";XCTAssertNoThrow(try validateConflict(bad,path:"a.txt"))
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/GitConflictTests.swift"><ui:paste select="#git-conflict-apple-tests"/></ui:file>
</ui:doc>
