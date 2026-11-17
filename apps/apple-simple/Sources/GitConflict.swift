import SwiftUI
import Observation
struct ConflictStage:Decodable {var stage:Int;var mode:String;var oid:String;var text:String?;var unavailable:String?}
struct ConflictReading:Decodable {var version:Int;var path:String;var fingerprint:String;var stages:[ConflictStage];var index_modified:Bool;var source_files_modified:Bool}
private func conflictStageHex(_ value:String,_ lengths:[Int])->Bool{lengths.contains(value.utf8.count)&&value.utf8.allSatisfy{(48...57).contains($0)||(65...70).contains($0)||(97...102).contains($0)}}
func validateConflictStages(_ stages:[ConflictStage])throws {
 guard (1...3).contains(stages.count) else{throw EngineError(message:"Malformed conflict stage count.")};var seen=Set<Int>()
 for stage in stages{guard (1...3).contains(stage.stage),seen.insert(stage.stage).inserted,conflictStageHex(stage.oid,[40,64]),stage.mode.utf8.count==6,stage.mode.utf8.allSatisfy({(48...55).contains($0)}),(stage.text != nil && stage.unavailable==nil && ["100644","100755"].contains(stage.mode))||(stage.text==nil && stage.unavailable != nil) else{throw EngineError(message:"Malformed conflict stage. Refresh repository changes.")}}
}
func validateConflict(_ value:ConflictReading,path:String)throws {
 guard value.version==1,exactUTF8(value.path,path),!value.index_modified,!value.source_files_modified,conflictStageHex(value.fingerprint,[64]) else{throw EngineError(message:"Conflict identity changed or unsupported response. Refresh the conflict.")};try validateConflictStages(value.stages)
}
@MainActor @Observable final class GitConflictModel {
 var value:ConflictReading?;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();loading=false;value=nil}
 func read(_ path:String,load:(String)async throws->ConflictReading)async{let id=UUID();ticket=id;value=nil;loading=true;notice="Reading unmerged stages…";defer{if ticket==id{loading=false}}
 do{let result=try await load(path);guard ticket==id else{return};try validateConflict(result,path:path);value=result;notice="Captured immutable Git stages. Working text and unsaved drafts are separate."}catch{if ticket==id{notice=error.localizedDescription}}}
 func choose(_ stage:Int,load:(String)async throws->ConflictReading,offer:(ConflictStage,ConflictReading,@escaping ()->Bool)async throws->Void)async{
  guard let value,let item=value.stages.first(where:{$0.stage==stage}),item.text != nil else{notice="Choose an available regular text stage first.";return}
  let id=UUID();ticket=id;let current={[weak self] in self?.ticket==id && self?.value != nil};loading=true;notice="Checking captured conflict stages…";defer{if ticket==id{loading=false}}
  do{let fresh=try await load(value.path);guard current() else{return};try validateConflict(fresh,path:value.path);guard exactUTF8(fresh.fingerprint,value.fingerprint),fresh.stages.contains(where:{$0.stage==item.stage && exactUTF8($0.mode,item.mode) && exactUTF8($0.oid,item.oid) && $0.text.map{Array($0.utf8)}==item.text.map{Array($0.utf8)}}) else{throw EngineError(message:"Conflict stages changed. Inspect them again before choosing a resolution.")};try await offer(item,value,current);if current(){notice="Prepared a source change preview. Nothing saved or staged."}}catch{if current(){notice=error.localizedDescription}}
 }

}
struct GitConflictView:View {
 var workspace:Workspace;var path:String;@State private var model=GitConflictModel();@State private var selected=2;@State private var page=0
 private func choose(_ stage:Int)async{
  guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};let epoch=workspace.sourceNavigationEpoch
  await model.choose(stage,load:{path in try await engine.request("GET","/api/git/conflict",query:[URLQueryItem(name:"path",value:path)])},offer:{item,value,current in
   let valid={current()&&workspace.sourceNavigationEpoch==epoch&&workspace.engine?.origin==engine.origin}
   guard valid() else{throw EngineError(message:"Workspace changed. Prepare the conflict again.")}
   func find(_ files:[FileNode])->FileNode?{for file in files{if exactUTF8(file.path,value.path){return file};if let child=find(file.children ?? []){return child}};return nil}
   guard let file=workspace.buffers.values.first(where:{exactUTF8($0.file.path,value.path)})?.file ?? find(workspace.rootFiles),!file.dir else{throw EngineError(message:"Conflict source is no longer listed. Refresh files and try again.")}
   let buffer=try await workspace.loadBuffer(file);guard valid() else{throw EngineError(message:"Workspace changed. Prepare the conflict again.")}
   guard !buffer.readOnly,!buffer.saving,let replacement=item.text else{throw EngineError(message:"Choose an authored, editable source that is not saving.")}
   let source=buffer.text,rows=source.components(separatedBy:"\n")
   let edit:[String:Any]=["changes":[languageURI(value.path):[["range":["start":["line":0,"character":0],"end":["line":rows.count-1,"character":rows.last!.utf16.count]],"newText":replacement]]]]
   try workspace.changes.offer("Use Git stage \(item.stage) · \(item.oid)",edit:edit,expected:[value.path:source],workspace:workspace);workspace.showReview=false
  })
 }
 var body:some View {
  VStack(alignment:.leading,spacing:6){
   GitResolutionView(workspace:workspace,path:path)
   Button("Inspect conflict stages"){Task{guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};page=0;await model.read(path){path in try await engine.request("GET","/api/git/conflict",query:[URLQueryItem(name:"path",value:path)])}}}.disabled(model.loading)
   if !model.notice.isEmpty{Text(model.notice).font(.caption)}
   if let value=model.value{
    Text("Stage 1: base · Stage 2 and Stage 3 retain Git’s meanings, including during rebase.").font(.caption)
    HStack{ForEach(value.stages,id:\.stage){stage in Button("Stage \(stage.stage)\(stage.stage==1 ? " · Base":"")"){selected=stage.stage;page=0}}}
    if let stage=value.stages.first(where:{$0.stage==selected}) ?? value.stages.first{
     Text("Stage \(stage.stage) · mode \(stage.mode)").font(.caption);Text(stage.oid).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
     if let text=stage.text{Button("Preview this stage as source"){Task{await choose(stage.stage)}}.disabled(model.loading);let rows=text.components(separatedBy:"\n"),last=max(0,(rows.count+99)/100-1),index=min(page,last),preview=rows.dropFirst(index*100).prefix(100).joined(separator:"\n")
      Text("Lines \(index*100+1)–\(min(rows.count,(index+1)*100))\(preview.count>200000 ? " · page preview capped at 200,000 characters":"")").font(.caption)
      ScrollView([.horizontal,.vertical]){Text(String(preview.prefix(200000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}.frame(maxHeight:220)
      HStack{Button("Previous stage lines"){page=index-1}.disabled(index==0);Button("Next stage lines"){page=index+1}.disabled(index==last)}
     }else{Text(stage.unavailable ?? "Text unavailable").font(.caption)}
    }
   }
  }.onDisappear{model.close()}
 }
}
