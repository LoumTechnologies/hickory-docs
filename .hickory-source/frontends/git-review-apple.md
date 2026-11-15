<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read saved repository changes through existing menus

For maintainers changing workspace services independently of the editor renderer.

Review saved Git state independently of editor drafts. A small model owns list/diff requests and a monotonic intent: refreshing, changing sides or closing retires older replies. The UI only presents that state in the existing floating surface; no source, save, staging or history command runs during inspection. Keep staged and working-tree columns distinct and bound displayed diff text without presenting a truncated view as complete.

<ui:copy id="simple-git-review-apple">
import SwiftUI
import Observation
struct ReviewFile:Decodable,Identifiable {var path:String;var from:String?;var index:String;var tree:String;var id:String{Data(path.utf8).base64EncodedString()}}
struct ReviewChanges:Decodable {var repository:Bool;var branch:String?;var files:[ReviewFile]}
struct ReviewDiff:Decodable {var path:String;var diff:String;var binary:Bool}
@MainActor @Observable final class GitReviewModel {
 var files:[ReviewFile]=[];var branch="";var selected:String?;var staged=false;var detail="";var notice="";var loading=false;var page=0
 private var ticket=UUID()
 func close(){ticket=UUID();loading=false}
 func refresh(load:()async throws->ReviewChanges)async {
  let id=UUID();ticket=id;loading=true;files=[];branch="";selected=nil;detail="";notice="Reading saved Git state…";defer{if ticket==id{loading=false}}
  do{let value=try await load();guard ticket==id else{return};files=value.files;branch=value.branch ?? "";page=0;notice=value.repository ? "Saved Git state · unsaved editor drafts are not included.":"This workspace is not a Git repository."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 func select(_ path:String,staged:Bool,load:(String,Bool)async throws->ReviewDiff)async {
  guard files.contains(where:{exactUTF8($0.path,path)}) else{return};let id=UUID();ticket=id;selected=path;self.staged=staged;detail="";loading=true;notice="Reading saved diff…";defer{if ticket==id{loading=false}}
  do{let value=try await load(path,staged);guard ticket==id else{return};guard exactUTF8(value.path,path) else{throw EngineError(message:"Diff identity changed. Refresh repository changes.")};detail=String(value.diff.prefix(200000));notice=value.binary ? "Binary change · text comparison unavailable.":value.diff.count>200000 ? "First 200,000 characters shown. Use Terminal for the complete diff.":value.diff.isEmpty ? "No changes on this side.":"Captured Git diff · refresh after external changes."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
</ui:copy>

## Present the captured reading without moving code

Use the existing floating surface and menu vocabulary. Paging limits the visible file list; staged and working-tree buttons select an explicit side. Keep saved Git state visibly separate from unsaved editor drafts, preserve code focus on close and retire pending requests when the surface disappears. Diff text stays selectable and never becomes an editor mutation.

<ui:copy id="simple-git-review-apple-view">
struct GitReviewView:View {
 var workspace:Workspace
 @State private var model=GitReviewModel()
 private func refresh()async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};await model.refresh{try await engine.request("GET","/api/git/changes")}}
 private func select(_ path:String,_ staged:Bool)async {guard let engine=workspace.engine else{return};await model.select(path,staged:staged){path,staged in try await engine.request("GET","/api/git/diff",query:[URLQueryItem(name:"path",value:path),URLQueryItem(name:"staged",value:String(staged))])}}
 var body:some View {
  VStack(alignment:.leading,spacing:8){
   HStack{Text("Repository changes \(model.branch)").font(.headline);Spacer();Button("Refresh"){Task{await refresh()}};Button("Close"){workspace.showReview=false;workspace.currentBuffer?.focusSource?()}}
   Text(model.notice).font(.caption).textSelection(.enabled)
   Text("Saved working tree and index. Unsaved editor drafts are not included.").font(.caption)
   ScrollView{VStack(alignment:.leading){ForEach(Array(model.files.dropFirst(model.page*20).prefix(20))){file in
    VStack(alignment:.leading){Text(file.from.map{"\($0) → \(file.path)"} ?? file.path).textSelection(.enabled);HStack{Button("Working tree \(file.tree)"){Task{await select(file.path,false)}}.accessibilityLabel("Working tree \(file.path)");Button("Staged \(file.index)"){Task{await select(file.path,true)}}.accessibilityLabel("Staged \(file.path)")}}
   }}}
   HStack{Button("Previous"){model.page-=1}.disabled(model.page==0);Text("\(model.files.count) changed files · page \(model.page+1)");Button("Next"){model.page+=1}.disabled((model.page+1)*20>=model.files.count)}.font(.caption)
   if let selected=model.selected{Text("\(selected) · \(model.staged ? "Staged":"Working tree")").font(.headline);ScrollView([.horizontal,.vertical]){Text(model.detail).font(.system(.caption,design:.monospaced)).textSelection(.enabled).frame(maxWidth:.infinity,alignment:.leading)}}
  }.padding().frame(idealWidth:640,idealHeight:520).task{await refresh()}.onDisappear{model.close()}
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/GitReview.swift"><ui:paste select="#simple-git-review-apple"/><ui:paste select="#simple-git-review-apple-view"/></ui:file>
## Reject obsolete and foreign diff replies

Hold two diff replies and release them in reverse order. The newer path and side must remain selected; closing retires a held reply. A mismatched response identity refuses before displaying its text. These are service safety witnesses, independent of popover layout and native physical input.

<ui:copy id="simple-git-review-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class GitReviewTests:XCTestCase {
 func testByteDistinctUnicodePathsKeepSeparateRowsAndDiffIdentities()async {
  let first="caf\u{e9}.py",second="cafe\u{301}.py",a=ReviewFile(path:first,index:"M",tree:"M"),b=ReviewFile(path:second,index:"M",tree:"M")
  XCTAssertNotEqual(a.id,b.id);let model=GitReviewModel();await model.refresh{ReviewChanges(repository:true,branch:"main",files:[a,b])}
  await model.select(second,staged:false){_,_ in ReviewDiff(path:first,diff:"wrong Unicode source",binary:false)}
  XCTAssertEqual(model.detail,"");XCTAssertTrue(model.notice.contains("identity changed"))
 }
 func testLatestSideAndClosedRepliesRetainTheirOwnIdentity()async {
  let model=GitReviewModel();await model.refresh{ReviewChanges(repository:true,branch:"main",files:["a.py","b.py"].map{ReviewFile(path:$0,index:"M",tree:"M")})}
  var pending:CheckedContinuation<ReviewDiff,Never>?
  let first=Task{await model.select("a.py",staged:false){_,_ in await withCheckedContinuation{pending=$0}}}
  while pending==nil{await Task.yield()}
  await model.select("b.py",staged:true){path,_ in ReviewDiff(path:path,diff:"new staged",binary:false)}
  pending?.resume(returning:ReviewDiff(path:"a.py",diff:"obsolete",binary:false));await first.value
  XCTAssertEqual(model.selected,"b.py");XCTAssertTrue(model.staged);XCTAssertEqual(model.detail,"new staged")
  pending=nil;let closed=Task{await model.select("a.py",staged:false){_,_ in await withCheckedContinuation{pending=$0}}}
  while pending==nil{await Task.yield()};model.close();pending?.resume(returning:ReviewDiff(path:"a.py",diff:"late after close",binary:false));await closed.value;XCTAssertEqual(model.detail,"")
  await model.select("a.py",staged:false){_,_ in ReviewDiff(path:"b.py",diff:"wrong source",binary:false)};XCTAssertEqual(model.detail,"");XCTAssertTrue(model.notice.contains("identity changed"))
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/GitReviewTests.swift"><ui:paste select="#simple-git-review-apple-tests"/></ui:file>
</ui:doc>
