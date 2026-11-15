import SwiftUI
import Observation
private func recoveryQuery(_ text:String)->String {text.addingPercentEncoding(withAllowedCharacters:CharacterSet(charactersIn:"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"))!}
struct RecoveryPage:Decodable {var version:Int;var workspace:String;var ids:[String];var next:String?;var ignored:Int}
struct RecoverySummary:Decodable {var kind:String;var bytes:Int?;var sha256:String?}
struct RecoveryPath:Decodable {var path:String;var saved_before:RecoverySummary;var saved_after:RecoverySummary;var draft_before:RecoverySummary;var draft_after:RecoverySummary
 func summary(_ snapshot:String)->RecoverySummary {switch snapshot {case "saved_before":return saved_before;case "saved_after":return saved_after;case "draft_after":return draft_after;default:return draft_before}}
}
struct RecoveryReading:Decodable {var version:Int;var workspace:String;var id:String;var fingerprint:String;var title:String;var title_truncated:Bool;var paths:[RecoveryPath];var all_paths_recorded_applied:Bool;var unrecognized_files:[String];var draft_text:String?;var saved_text:String?}
@MainActor @Observable final class RecoveryReader {
 var page:RecoveryPage?;var value:RecoveryReading?;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();page=nil;value=nil;loading=false}
 func list(after:String?=nil,load:(String)async throws->RecoveryPage)async {
  let id=UUID();ticket=id;loading=true;value=nil;notice="Reading retained recovery identities…";defer{if ticket==id{loading=false}}
  do{let next=try await load("/api/changes/resource-recoveries"+(after.map{"?after="+recoveryQuery($0)} ?? ""));guard ticket==id else{return}
   guard next.version==1,!next.workspace.isEmpty,next.ids.count<=100,next.ids.allSatisfy({$0.range(of:"^[0-9a-f]{32}$",options:.regularExpression) != nil}),zip(next.ids,next.ids.dropFirst()).allSatisfy({$0.0<$0.1}),after.map({cursor in next.ids.allSatisfy{$0>cursor}&&next.workspace==page?.workspace}) ?? true,next.next==nil||next.next==next.ids.last else{throw EngineError(message:"Recovery page identity changed. Refresh from the beginning.")}
   page=next;notice=next.ids.isEmpty ? "No retained resource recovery records.":"Choose a retained record. Identities are sorted, not chronological."
  }catch{if ticket==id{page=nil;notice=error.localizedDescription}}
 }
 func read(_ record:String,path:String?=nil,snapshot:String="draft_before",load:(String)async throws->RecoveryReading)async {
  let workspace=page?.workspace,previous=value?.id==record ? value:nil,id=UUID();ticket=id;loading=true;value=nil;notice="Reading retained source evidence…";defer{if ticket==id{loading=false}}
  do{guard let workspace,page?.ids.contains(record)==true else{throw EngineError(message:"Refresh recovery identities before selecting a record.")}
   let parts=snapshot.split(separator:"_");guard parts.count==2,["saved","draft"].contains(String(parts[0])),["before","after"].contains(String(parts[1])) else{throw EngineError(message:"Choose a supported recovery snapshot.")}
   let route="/api/changes/resource-recovery?id="+recoveryQuery(record)+(path.map{"&path="+recoveryQuery($0)+"&"+parts[0]+"="+parts[1]} ?? "")
   let reply=try await load(route);guard ticket==id else{return}
   guard reply.version==1,reply.workspace==workspace,reply.id==record,reply.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,previous.map({$0.fingerprint==reply.fingerprint}) ?? true,path.map({path in reply.paths.contains{exactUTF8($0.path,path)}}) ?? true else{throw EngineError(message:"Recovery evidence changed. Refresh identities and read again.")}
   value=reply;notice="Retained evidence only. Current source and editor drafts are unchanged."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
}
struct SourceRecoveryView:View {
 var workspace:Workspace;@State private var model=RecoveryReader();@State private var path="";@State private var snapshot="draft_before"
 private func list(_ after:String?=nil)async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};path="";await model.list(after:after){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 private func read(_ id:String,_ path:String?=nil,_ snapshot:String="draft_before")async {guard let engine=workspace.engine else{return};await model.read(id,path:path,snapshot:snapshot){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 var body:some View {VStack(alignment:.leading,spacing:8){
  HStack{Text("Source recovery").font(.headline);Spacer();Button("Close"){workspace.showRecovery=false;workspace.currentBuffer?.focusSource?()}}
  Button("Refresh records"){Task{await list()}}.disabled(model.loading);Text(model.notice).font(.caption).textSelection(.enabled)
  ScrollView{VStack(alignment:.leading,spacing:8){
   if let page=model.page {Picker("Retained record",selection:Binding(get:{model.value?.id ?? ""},set:{id in if !id.isEmpty{path="";Task{await read(id)}}})){Text("Choose record").tag("");ForEach(page.ids,id:\.self){Text($0).tag($0)}}.disabled(model.loading)
    if page.ignored>0 {Text("\(page.ignored) unrecognized entries omitted.").font(.caption)}
    if let next=page.next {Button("Next 100 identities"){Task{await list(next)}}.disabled(model.loading)}
   }
   if let value=model.value {
    Text(value.title+(value.title_truncated ? "…":"")).font(.headline)
    Text((value.all_paths_recorded_applied ? "Every path has an applied receipt.":"The record does not confirm every path was applied.")+" These receipts describe the recorded operation, not current disk state.").font(.caption)
    if !value.unrecognized_files.isEmpty {Text("\(value.unrecognized_files.count) unrecognized files in this record.").font(.caption)}
    Picker("Recorded source",selection:Binding(get:{path},set:{next in path=next;Task{await read(value.id,next,snapshot)}})){Text("Choose source").tag("");ForEach(Array(value.paths.enumerated()),id:\.offset){_,item in Text(item.path).tag(item.path)}}.disabled(model.loading)
    Picker("Snapshot",selection:Binding(get:{snapshot},set:{next in snapshot=next;if !path.isEmpty{Task{await read(value.id,path,next)}}})){ForEach(["saved_before","draft_before","saved_after","draft_after"],id:\.self){Text($0.replacingOccurrences(of:"_",with:" ")).tag($0)}}.disabled(model.loading)
    if let selected=value.paths.first(where:{exactUTF8($0.path,path)}) {let summary=selected.summary(snapshot)
     Text(summary.kind=="missing" ? "Source absent in this snapshot.":"\(summary.bytes ?? 0) UTF-8 bytes · \(summary.sha256 ?? "")").font(.caption).textSelection(.enabled)
     if let text=snapshot.hasPrefix("saved") ? value.saved_text:value.draft_text {Text(String(text.prefix(32000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled);if text.count>32000 {Text("First 32,000 characters shown.").font(.caption)}}
    }
   }
  }.frame(maxWidth:.infinity,alignment:.leading)}
 }.padding().frame(idealWidth:680,idealHeight:600).task{await list()}.onDisappear{model.close()}}
}
