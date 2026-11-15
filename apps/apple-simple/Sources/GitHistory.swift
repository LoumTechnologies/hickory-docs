import SwiftUI
import Observation
enum GitHistoryRequest {
 case log(String?)
 case commit(String,String)
 var title:String{switch self{case .log(let path):return path.map{"Saved history · \($0)"} ?? "Repository history";case .commit(_,let title):return title}}
 var commitID:String?{if case .commit(let sha,_)=self{return sha};return nil}
 var path:String?{if case .log(let path)=self{return path};return nil}
 var isLog:Bool{if case .log=self{return true};return false}
}
struct HistoryCommit:Decodable,Identifiable {var sha:String;var short:String;var author:String;var subject:String;var id:String{sha}}
struct HistoryLog:Decodable {var repository:Bool;var commits:[HistoryCommit]}
struct HistoryFile:Decodable {var path:String;var from:String;var status:String;var before:String;var after:String;var binary:Bool}
struct HistoryReading:Decodable {var sha:String;var message:String;var files:[HistoryFile]}
@MainActor @Observable final class GitHistoryModel {
 let request:GitHistoryRequest
 init(request:GitHistoryRequest = .log(nil)){self.request=request}
 var commits:[HistoryCommit]=[];var reading:HistoryReading?;var file=0;var before=false;var page=0;var limit=120;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();loading=false;reading=nil;notice=""}
 func refresh(limit:Int=120,load:(Int,String?)async throws->HistoryLog)async {
  guard request.isLog else{notice="Choose a log request before reading commit choices.";return};let path=request.path
  let id=UUID();ticket=id;self.limit=max(1,min(1000,limit));commits=[];reading=nil;page=0;loading=true;notice="Reading saved history…";defer{if ticket==id{loading=false}}
  do{let value=try await load(self.limit,path);guard ticket==id else{return};guard value.commits.allSatisfy({$0.sha.range(of:"^([0-9a-f]{40}|[0-9a-f]{64})$",options:.regularExpression) != nil}) else{throw EngineError(message:"Malformed commit identity. Refresh repository history.")};commits=value.commits;notice=value.repository ? "\(commits.count) reported commits · saved history only.":"This workspace is not a Git repository."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 func select(_ sha:String,load:(String)async throws->HistoryReading)async {
  guard commits.contains(where:{$0.sha==sha}) else{return};await readCommit(sha,load:load)
 }
 func readCommit(_ sha:String,load:(String)async throws->HistoryReading)async {
  guard sha.range(of:"^([0-9a-f]{40}|[0-9a-f]{64})$",options:.regularExpression) != nil,(request.commitID==nil || request.commitID==sha) else{ticket=UUID();reading=nil;loading=false;notice="Commit reference is missing or changed. Refresh its source and choose the commit again.";return};let id=UUID();ticket=id;reading=nil;loading=true;notice="Reading immutable commit…";defer{if ticket==id{loading=false}}
  do{let value=try await load(sha);guard ticket==id else{return};guard value.sha==sha else{throw EngineError(message:"Commit identity changed. Refresh repository history.")};reading=value;file=0;before=false;notice="Immutable commit reading · current editor draft is retained."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
struct GitHistoryView:View {
 var workspace:Workspace
 var model:GitHistoryModel
 private func refresh(_ count:Int=120)async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};if case .commit(let sha,_)=model.request{await model.readCommit(sha){try await engine.request("GET","/api/git/reading",query:[URLQueryItem(name:"sha",value:$0)])};return};await model.refresh(limit:count){count,path in var query=[URLQueryItem(name:"limit",value:String(count))];if let path{query.append(URLQueryItem(name:"path",value:path))};return try await engine.request("GET","/api/git/log",query:query)}}
 private func select(_ sha:String)async {guard let engine=workspace.engine else{return};await model.select(sha){try await engine.request("GET","/api/git/reading",query:[URLQueryItem(name:"sha",value:$0)])}}
 var body:some View {
  VStack(alignment:.leading,spacing:8){
   HStack{Text(model.request.title).font(.headline);Spacer();Button("Refresh"){Task{await refresh(model.limit)}};Button("Close"){workspace.showHistory=false;workspace.currentBuffer?.focusSource?()}}
   Text(model.notice).font(.caption).textSelection(.enabled)
   if model.request.isLog{
   ScrollView{VStack(alignment:.leading){ForEach(Array(model.commits.dropFirst(model.page*20).prefix(20))){commit in Button("\(commit.short) · \(commit.subject) · \(commit.author)"){Task{await select(commit.sha)}}.accessibilityLabel("Read commit \(commit.sha)")}}}
   HStack{Button("Previous"){model.page-=1}.disabled(model.page==0);Text("Page \(model.page+1)");Button("Next"){model.page+=1}.disabled((model.page+1)*20>=model.commits.count);Button("Load more commits"){Task{await refresh(min(1000,model.limit+120))}}.disabled(model.limit>=1000 || model.commits.count<model.limit)}.font(.caption)
   }
   if let reading=model.reading{Text(reading.sha).font(.caption).textSelection(.enabled);Text(String(reading.message.prefix(20000))).font(.caption).textSelection(.enabled);if reading.message.count>20000{Text("First 20,000 message characters shown.").font(.caption)}
    if reading.files.indices.contains(model.file){Picker("Historical file",selection:Binding(get:{model.file},set:{model.file=$0})){ForEach(reading.files.indices,id:\.self){index in Text("\(reading.files[index].status) · \(reading.files[index].path)").tag(index)}}
     Picker("Historical side",selection:Binding(get:{model.before},set:{model.before=$0})){Text("Before commit").tag(true);Text("After commit").tag(false)}
     let file=reading.files[model.file],content=model.before ? file.before:file.after
     Text(file.binary ? "Binary content · text reading unavailable.":content.count>200000 ? "First 200,000 characters shown.":content.isEmpty ? "No text on this side.":"Immutable file content.").font(.caption)
     ScrollView([.horizontal,.vertical]){Text(file.binary ? "":String(content.prefix(200000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled).frame(maxWidth:.infinity,alignment:.leading)}
    }else{Text("No files under this workspace in this commit.")}
   }
  }.padding().frame(idealWidth:680,idealHeight:600).task(id:ObjectIdentifier(model)){await refresh()}.onDisappear{model.close()}
 }
}
