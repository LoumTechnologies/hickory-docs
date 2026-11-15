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
