import SwiftUI
import Observation
struct ReviewFile:Decodable,Identifiable {var path:String;var from:String?;var index:String;var tree:String;var from_outside_workspace:Bool?;var id:String{Data(path.utf8).base64EncodedString()}}
struct ReviewChanges:Decodable {var repository:Bool;var branch:String?;var files:[ReviewFile]}
struct ReviewDiff:Decodable {var path:String;var diff:String;var binary:Bool}
struct ReviewIndexResult:Decodable {var ok:Bool}
func reviewIndexPaths(_ file:ReviewFile,unstage:Bool)throws->[String] {
 let rename=(unstage ? file.index:file.tree)=="R"
 if rename && (file.from==nil || file.from_outside_workspace==true){throw EngineError(message:"Open the repository root to change this complete rename. No index request was sent.")}
 var result:[String]=[];for path in rename ? [file.from!,file.path]:[file.path]{if !result.contains(where:{exactUTF8($0,path)}){result.append(path)}};return result
}
@MainActor @Observable final class GitReviewModel {
 var indexPending=false;var needsRefresh=false;var files:[ReviewFile]=[];var branch="";var selected:String?;var staged=false;var detail="";var notice="";var loading=false;var page=0
 private var ticket=UUID()
 func close(){ticket=UUID();loading=false}
 func refresh(request:UUID=UUID(),load:()async throws->ReviewChanges)async {
  let id=request;ticket=id;loading=true;files=[];branch="";selected=nil;detail="";notice="Reading saved Git state…";defer{if ticket==id{loading=false}}
  do{let value=try await load();guard ticket==id else{return};files=value.files;if !indexPending{needsRefresh=false};branch=value.branch ?? "";page=0;notice=value.repository ? "Saved Git state · unsaved editor drafts are not included.":"This workspace is not a Git repository."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 func select(_ path:String,staged:Bool,load:(String,Bool)async throws->ReviewDiff)async {
  guard !indexPending, files.contains(where:{exactUTF8($0.path,path)}) else{return};let id=UUID();ticket=id;selected=path;self.staged=staged;detail="";loading=true;notice="Reading saved diff…";defer{if ticket==id{loading=false}}
  do{let value=try await load(path,staged);guard ticket==id else{return};guard exactUTF8(value.path,path) else{throw EngineError(message:"Diff identity changed. Refresh repository changes.")};detail=String(value.diff.prefix(200000));notice=value.binary ? "Binary change · text comparison unavailable.":value.diff.count>200000 ? "First 200,000 characters shown. Use Terminal for the complete diff.":value.diff.isEmpty ? "No changes on this side.":"Captured Git diff · refresh after external changes."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 func changeIndex(_ path:String,unstage:Bool,write:([String],Bool)async throws->Bool,load:()async throws->ReviewChanges)async {
  guard !indexPending else{return};guard !needsRefresh else{notice="Refresh Git state before another index action.";return}
  guard let file=files.first(where:{exactUTF8($0.path,path)}) else{return}
  let paths:[String];do{paths=try reviewIndexPaths(file,unstage:unstage)}catch{notice=error.localizedDescription;return}
  let id=UUID();ticket=id;indexPending=true;needsRefresh=true;loading=false;notice=unstage ? "Unstaging saved paths…":"Staging saved paths…";defer{indexPending=false}
  do{guard try await write(paths,unstage) else{throw EngineError(message:"Index operation was not confirmed.")};guard ticket==id else{return};indexPending=false;let reading=UUID();await refresh(request:reading,load:load);if ticket==reading && !needsRefresh{notice=unstage ? "Saved paths unstaged. Editor drafts retained.":"Saved paths staged. Review the staged side before committing."}}
  catch{if ticket==id{notice="Index result not confirmed: \(error.localizedDescription) Refresh Git state before retrying."}}
 }

}
struct GitReviewView:View {
 var workspace:Workspace
 private var model:GitReviewModel{workspace.gitReview}
 private func refresh()async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};await model.refresh{try await engine.request("GET","/api/git/changes")}}
 private func select(_ path:String,_ staged:Bool)async {guard let engine=workspace.engine else{return};await model.select(path,staged:staged){path,staged in try await engine.request("GET","/api/git/diff",query:[URLQueryItem(name:"path",value:path),URLQueryItem(name:"staged",value:String(staged))])}}
 private func changeIndex(_ path:String,_ unstage:Bool)async {guard let engine=workspace.engine else{return};await model.changeIndex(path,unstage:unstage,write:{paths,unstage in let result:ReviewIndexResult=try await engine.request("POST",unstage ? "/api/git/unstage":"/api/git/stage",body:["paths":paths]);return result.ok},load:{try await engine.request("GET","/api/git/changes")})}
 private var reviewRowSpacing: CGFloat? {
    #if os(iOS)
    2
    #else
    nil
    #endif
 }

 @ViewBuilder
 private func sideControl(for file: ReviewFile, staged: Bool) -> some View {
    let side = staged ? "Staged" : "Working tree"
    let title = "\(side) \(staged ? file.index : file.tree)"
    let name = "\(side) \(file.path)"
    let action: () -> Void = { guard !model.indexPending else { return }; Task { await select(file.path, staged) } }
    #if os(iOS)
    GitReviewSideButton(title: title, name: name, action: action)
    #else
    Button(title, action: action).accessibilityLabel(name)
    #endif
 }
 var body:some View {
  VStack(alignment:.leading,spacing:8){
   HStack{Text("Repository changes \(model.branch)").font(.headline);Spacer();Button("Refresh"){Task{await refresh()}}.disabled(reason: model.indexPending ? "A Git index update is in progress." : nil);Button("Close"){workspace.showReview=false;workspace.currentBuffer?.focusSource?()}}
   Text(model.notice).font(.caption).textSelection(.enabled)
   Text("Saved working tree and index. Unsaved editor drafts are not included.").font(.caption)
   ScrollView{VStack(alignment:.leading){ForEach(Array(model.files.dropFirst(model.page*20).prefix(20))){file in
    VStack(alignment:.leading, spacing: reviewRowSpacing) {
     Text(file.from.map{"\($0) → \(file.path)"} ?? file.path).textSelection(.enabled)
     HStack {
      sideControl(for: file, staged: false)
      sideControl(for: file, staged: true)
     }.disabled(reason: model.indexPending ? "A Git index update is in progress." : nil)
    }
   }}}
   HStack{Button("Previous"){model.page-=1}.disabled(reason: model.page==0 ? "You’re on the first page." : nil);Text("\(model.files.count) changed files · page \(model.page+1)");Button("Next"){model.page+=1}.disabled(reason: (model.page+1)*20>=model.files.count ? "You’re on the last page." : nil)}.font(.caption)
   if model.needsRefresh && !model.indexPending{Text("Refresh Git state before another index action.").font(.caption)}
   if let selected=model.selected{Text("\(selected) · \(model.staged ? "Staged":"Working tree")").font(.headline);GitConflictView(workspace:workspace,path:selected).id(selected);HStack{Button("Stage saved file"){Task{await changeIndex(selected,false)}}.disabled(reason: model.indexPending ? "A Git index update is in progress." : model.needsRefresh ? "Refresh repository changes after the previous Git operation." : model.files.first(where:{exactUTF8($0.path,selected)})?.tree==" " ? "The selected file has no saved changes to stage." : nil);Button("Unstage file"){Task{await changeIndex(selected,true)}}.disabled(reason: model.indexPending ? "A Git index update is in progress." : model.needsRefresh ? "Refresh repository changes after the previous Git operation." : [" ","?","!"].contains(model.files.first(where:{exactUTF8($0.path,selected)})?.index ?? " ") ? "The selected file has no staged changes to unstage." : nil)};ScrollView([.horizontal,.vertical]){Text(model.detail).font(.system(.caption,design:.monospaced)).textSelection(.enabled).frame(maxWidth:.infinity,alignment:.leading)}}
  }.padding().frame(idealWidth:640,idealHeight:520).task{await refresh()}.onDisappear{model.close()}
 }
}

#if os(iOS)
import UIKit

private struct GitReviewSideButton: UIViewRepresentable {
    var title: String
    var name: String
    var action: () -> Void

    func makeCoordinator() -> Coordinator { Coordinator(action) }

    func makeUIView(context: Context) -> UIButton {
        let button = UIButton(type: .system)
        button.titleLabel?.adjustsFontForContentSizeCategory = true
        button.titleLabel?.lineBreakMode = .byTruncatingTail
        button.addTarget(context.coordinator, action: #selector(Coordinator.activate), for: .primaryActionTriggered)
        return button
    }

    func updateUIView(_ button: UIButton, context: Context) {
        context.coordinator.action = action
        button.setTitle(title, for: .normal)
        button.titleLabel?.font = .preferredFont(forTextStyle: .body, compatibleWith: button.traitCollection)
        button.accessibilityLabel = name
    }

    func sizeThatFits(_ proposal: ProposedViewSize, uiView: UIButton, context: Context) -> CGSize? {
        guard let natural = uiView.titleLabel?.intrinsicContentSize else { return nil }
        return CGSize(width: min(proposal.width ?? natural.width, natural.width), height: natural.height)
    }

    final class Coordinator: NSObject {
        var action: () -> Void
        init(_ action: @escaping () -> Void) { self.action = action }
        @objc func activate() { action() }
    }
}
#endif
