<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect saved attribution without borrowing draft positions

For maintainers changing workspace services independently of the editor renderer.

Saved-file attribution is a captured reading, not a claim that line positions match an unsaved editor draft. Keep its requested path, reported line text, author, provider and immutable commit reference together. Reject foreign paths, repeated line identities and obsolete replies before presentation. Uncommitted lines and older providers without a full reference remain readable; only a valid reported full identity can open an immutable commit reading. This capability performs no source or index mutation.

<ui:copy id="simple-git-attribution-apple">
import SwiftUI
import Observation
struct AttributionLine:Decodable,Identifiable {
 var line:Int;var commit:String;var full_commit:String?;var author:String;var summary:String;var uncommitted:Bool;var provider:String?;var text:String?;var id:Int{line}
 var immutableCommit:String?{guard !uncommitted,let full_commit,full_commit.range(of:"^([0-9a-f]{40}|[0-9a-f]{64})$",options:.regularExpression) != nil else{return nil};return full_commit}
}
struct AttributionReading:Decodable {var path:String;var lines:[AttributionLine]}
@MainActor @Observable final class GitAttributionModel {
 let path:String;var lines:[AttributionLine]=[];var notice="";var page=0;var loading=false;private var ticket=UUID()
 init(path:String){self.path=path}
 func close(){ticket=UUID();loading=false;lines=[];notice=""}
 func refresh(load:(String)async throws->AttributionReading)async {
  let id=UUID();ticket=id;lines=[];page=0;loading=true;notice="Reading saved-file attribution…";defer{if ticket==id{loading=false}}
  do{let value=try await load(path);guard ticket==id else{return};guard exactUTF8(value.path,path) else{throw EngineError(message:"Attribution identity changed. Refresh the saved file.")};guard value.lines.allSatisfy({$0.line>0}),Set(value.lines.map{$0.line}).count==value.lines.count else{throw EngineError(message:"Ambiguous attribution lines. Refresh the saved file.")};lines=value.lines.sorted{$0.line<$1.line};notice=lines.isEmpty ? "No saved Git or recorded attribution is available.":"Captured saved attribution · draft line positions may differ."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
</ui:copy>

## Present saved facts and reuse immutable commit reading

Use the existing floating menu surface, retaining the captured path in its heading. Report saved line numbers and provider labels without anchoring them onto dirty draft positions. Only a current row with a full committed identity can construct a typed commit request for the existing historical reader. Paging and explicit line-text limits keep inspection compact; closing restores code focus and retires the request.

<ui:copy id="simple-git-attribution-apple-view">
struct GitAttributionView:View {
 var workspace:Workspace;var model:GitAttributionModel
 private func refresh()async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};await model.refresh{try await engine.request("GET","/api/blame",query:[URLQueryItem(name:"path",value:$0)])}}
 var body:some View {
  VStack(alignment:.leading,spacing:8){
   HStack{Text("Saved attribution · \(model.path)").font(.headline);Spacer();Button("Refresh"){Task{await refresh()}};Button("Close"){workspace.showAttribution=false;workspace.currentBuffer?.focusSource?()}}
   Text(model.notice).font(.caption).textSelection(.enabled);Text("Saved-file lines only. Unsaved editor changes are not included.").font(.caption)
   ScrollView{VStack(alignment:.leading,spacing:12){ForEach(Array(model.lines.dropFirst(model.page*50).prefix(50))){line in
    VStack(alignment:.leading){Text("Saved line \(line.line) · \(line.author) · \(line.commit.isEmpty ? "No committed revision":line.commit)").font(.headline);Text("\(line.summary) · \(line.provider ?? (line.uncommitted ? "Uncommitted saved line":"Git blame"))").font(.caption).textSelection(.enabled);Text(line.text.map{String($0.prefix(2000))} ?? "Line text unavailable.").font(.system(.caption,design:.monospaced)).textSelection(.enabled)
     if (line.text?.count ?? 0)>2000{Text("Line text abbreviated.").font(.caption)}
     if let sha=line.immutableCommit{Button("Read immutable commit"){guard model.lines.contains(where:{$0.line==line.line&&$0.immutableCommit==sha}) else{return};workspace.gitHistory.close();workspace.gitHistory=GitHistoryModel(request:.commit(sha,"Commit from saved attribution · \(model.path):\(line.line)"));workspace.showAttribution=false;workspace.showReview=false;workspace.showHistory=true}.accessibilityLabel("Read commit for saved line \(line.line)")}else{Text("Full immutable commit reference unavailable.").font(.caption)}
    }
   }}}
   HStack{Button("Previous"){model.page-=1}.disabled(model.page==0);Text("\(model.lines.count) reported lines · page \(model.page+1)");Button("Next"){model.page+=1}.disabled((model.page+1)*50>=model.lines.count)}.font(.caption)
  }.padding().frame(idealWidth:680,idealHeight:600).task(id:ObjectIdentifier(model)){await refresh()}.onDisappear{model.close()}
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/GitAttribution.swift"><ui:paste select="#simple-git-attribution-apple"/><ui:paste select="#simple-git-attribution-apple-view"/></ui:file>
## Reject stale attribution and ambiguous references

Hold attribution replies, replace their request, and close the reading before releasing old work. Foreign paths and repeated line identities must refuse without presenting a mixed snapshot. Only a valid full reference on a committed line admits immutable reading; abbreviated, missing and uncommitted references remain display facts.

<ui:copy id="simple-git-attribution-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class GitAttributionTests:XCTestCase {
 func row(_ line:Int,_ full:String?=nil)->AttributionLine{AttributionLine(line:line,commit:"12345678",full_commit:full,author:"Fixture",summary:"base",uncommitted:false,text:"saved")}
 func testCapturedPathsAndLineIdentitiesRetireObsoleteReplies()async {
  let model=GitAttributionModel(path:"a.py");var pending:CheckedContinuation<AttributionReading,Never>?
  let old=Task{await model.refresh{_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.refresh{AttributionReading(path:$0,lines:[self.row(2,String(repeating:"a",count:40))])};pending?.resume(returning:AttributionReading(path:"a.py",lines:[row(1)]));await old.value;XCTAssertEqual(model.lines.map{$0.line},[2])
  pending=nil;let closed=Task{await model.refresh{_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:AttributionReading(path:"a.py",lines:[row(1)]));await closed.value;XCTAssertTrue(model.lines.isEmpty)
  let foreign=GitAttributionModel(path:"cafe\u{301}.py");await foreign.refresh{_ in AttributionReading(path:"caf\u{e9}.py",lines:[self.row(1)])};XCTAssertTrue(foreign.lines.isEmpty);XCTAssertTrue(foreign.notice.contains("identity changed"))
  await model.refresh{AttributionReading(path:$0,lines:[self.row(1),self.row(1)])};XCTAssertTrue(model.lines.isEmpty);XCTAssertTrue(model.notice.contains("Ambiguous"))
 }
 func testCompactLabelsAndDirtyLinesAreNotImmutableReferences(){
  XCTAssertNil(row(1).immutableCommit);XCTAssertNil(row(1,"12345678").immutableCommit);var dirty=row(1,String(repeating:"a",count:40));dirty.uncommitted=true;XCTAssertNil(dirty.immutableCommit);XCTAssertEqual(row(1,String(repeating:"a",count:40)).immutableCommit,String(repeating:"a",count:40));XCTAssertEqual(row(1,String(repeating:"b",count:64)).immutableCommit,String(repeating:"b",count:64))
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/GitAttributionTests.swift"><ui:paste select="#simple-git-attribution-apple-tests"/></ui:file>
</ui:doc>
