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
