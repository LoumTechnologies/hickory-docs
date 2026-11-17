import Foundation
import SwiftUI
import Observation
private func gitResolutionPaths(_ paths:[String])->Set<Data>{Set(paths.map{Data($0.utf8)})}
private func gitResolutionSorted(_ paths:[String])->[String]{paths.sorted{Data($0.utf8).lexicographicallyPrecedes(Data($1.utf8))}}
private func gitResolutionData<T:Encodable>(_ value:T)throws->Data{let encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];return try encoder.encode(value)}
private func gitResolutionSaved(_ a:[String:ResourceSaved],_ b:[String:ResourceSaved])->Bool{gitResolutionPaths(Array(a.keys))==gitResolutionPaths(Array(b.keys)) && a.allSatisfy{path,state in b.first{exactUTF8($0.key,path)}.map{state.matches($0.value)}==true}}
struct GitResolutionPath:Decodable,Identifiable {var id:NativeSourceKey{NativeSourceKey(path)};var path:String;var stages:[ConflictStage];var working:ResourceSaved?;var unavailable:String?}
struct GitResolutionIndexEntry:Decodable {var path:String;var mode:String;var oid:String;var stage:Int}
struct GitResolutionObservation:Decodable {var version:Int;var repository:String;var fingerprint:String;var index_hash:String;var heads:[String:String?];var entries:[GitResolutionIndexEntry];var outside_conflicts:UInt}
struct GitResolutionReading:Decodable {var observation:GitResolutionObservation;var version:Int;var resolution_protocol:Int;var members:[String];var paths:[GitResolutionPath];var index_modified:Bool;var source_files_modified:Bool}
struct GitResolutionOutput:Identifiable {
 let id=UUID();var path:String;var identityFrom:String?;var text:String;var mode="100644";var stagePath:String?;var stage:Int?;var oid:String?
 var body:[String:Any]{["path":path,"identity_from":identityFrom as Any? ?? NSNull(),"mode":mode,"content":stage.map{["kind":"stage","path":stagePath as Any? ?? NSNull(),"stage":$0,"oid":oid as Any? ?? NSNull()] as [String:Any]} ?? ["kind":"text","text":text]]}
}
struct GitResolutionPrepared:Decodable {
 var version:Int;var resolution_protocol:Int;var token:String;var fingerprint:String;var members:[String];var remaining_conflicts:[String];var proposal:ResourceWireProposal;var plan:ResourceWirePlan;var consequences:ResourceWireFacts;var saved_sources:[String:ResourceSaved];var needs_save:Bool;var stage_available:Bool;var stage_unavailable:String?;var source_files_modified:Bool;var index_modified:Bool
}
struct GitResolutionMaterial:@unchecked Sendable {
 let value:GitResolutionPrepared;let raw:ResourceJSON;let patches:[ResourceEditorPatch];let pages:AgentProposalSection
 init(_ raw:ResourceJSON)throws {
  let bytes=try gitResolutionData(raw),value=try JSONDecoder().decode(GitResolutionPrepared.self,from:bytes);let patches=try value.plan.editors()
  guard value.version==1,value.resolution_protocol==1,sourceHistoryHex(value.token,32),sourceHistoryHex(value.fingerprint,64),!value.source_files_modified,!value.index_modified,value.proposal.inputs.version==1,value.proposal.inputs.saved_guards != nil,value.proposal.inputs.refusals.isEmpty,!value.proposal.inputs.source_files_modified,value.plan.context==value.proposal.capture.context,value.plan.context==value.proposal.inputs.context,value.consequences.version==1,value.consequences.context==value.plan.context,value.consequences.workspace==value.proposal.inputs.workspace,!value.consequences.source_files_modified,value.consequences.protected_targets.isEmpty,sourceHistoryHex(value.consequences.fingerprint,64),try gitResolutionData(value.plan)==gitResolutionData(value.proposal.plan),gitResolutionPaths(Array(value.saved_sources.keys))==gitResolutionPaths(patches.map{$0.path}),value.members.count==gitResolutionPaths(value.members).count,value.remaining_conflicts.count==gitResolutionPaths(value.remaining_conflicts).count else{throw EngineError(message:"Complete conflict proposal, source metadata or generated consequences changed.")}
  for patch in patches{guard let saved=value.saved_sources.first(where:{exactUTF8($0.key,patch.path)})?.value,(saved.kind=="file")==((patch.after) != nil),saved.kind=="missing" || saved.kind=="file" && saved.text.map{nativeSourceHash($0)==saved.sha256}==true else{throw EngineError(message:"Conflict proposal lacks a complete saved source namespace.")}}
  self.value=value;self.raw=raw;self.patches=patches;pages=try AgentProposalSection("Complete conflict proposal",String(decoding:bytes,as:UTF8.self))
 }
 }
@MainActor final class GitResolutionPreview {
 let material:GitResolutionMaterial;var current:()->Bool
 var value:GitResolutionPrepared{material.value};var raw:ResourceJSON{material.raw};var patches:[ResourceEditorPatch]{material.patches};var pages:AgentProposalSection{material.pages}
 init(_ material:GitResolutionMaterial,current:@escaping()->Bool){self.material=material;self.current=current}
 var draftable:Bool{patches.allSatisfy{$0.before != nil && $0.after != nil && exactUTF8($0.before?.identity,$0.after?.identity)} && patches.contains{!exactUTF8($0.before?.text,$0.after?.text)}}
}
struct GitResolutionValidation:Decodable {var version:Int;var token:String;var fingerprint:String;var stage:Bool;var current:Bool;var plan:ResourceWirePlan;var saved_sources:[String:ResourceSaved];var source_files_modified:Bool;var index_modified:Bool}
struct GitResolutionStageReceipt:Decodable {var version:Int;var operation:String;var request_id:String;var token:String;var review_fingerprint:String;var complete:Bool;var index_modified:Bool;var index_uncertain:Bool;var source_files_modified:Bool;var fingerprint:String;var resolved_members:[String];var remaining_conflicts:[String]}
func validateGitResolutionStage(_ raw:GitResolutionStageReceipt,request:String,review:GitResolutionPrepared)throws {
 guard raw.version==1,raw.operation=="git_conflict_stage",raw.request_id==request,raw.token==review.token,raw.review_fingerprint==review.fingerprint,raw.complete,raw.index_modified,!raw.index_uncertain,!raw.source_files_modified,sourceHistoryHex(raw.fingerprint,64),raw.resolved_members.count==gitResolutionPaths(raw.resolved_members).count,raw.remaining_conflicts.count==gitResolutionPaths(raw.remaining_conflicts).count,gitResolutionPaths(raw.resolved_members)==gitResolutionPaths(review.members),gitResolutionPaths(raw.remaining_conflicts)==gitResolutionPaths(review.remaining_conflicts) else{throw EngineError(message:"Complete index publication is unconfirmed. Retain and inspect its original request.")}
}
@MainActor @Observable final class GitResolutionModel {
 private(set) var reading:GitResolutionReading?;private(set) var members:[String]=[];private(set) var outputs:[GitResolutionOutput]=[];private(set) var preview:GitResolutionPreview?;private(set) var notice="";private(set) var busy=false;private(set) var saved=false;private(set) var modified=false
 var transport:((String,String,[String:Any]?)async throws->ResourceJSON)?
 private var rawReading:ResourceJSON?;private var ticket=UUID();private var writeGuard:UUID?
 private struct Pending {let id:String;let preview:GitResolutionPreview;let stage:Bool;let engine:Engine;weak var workspace:Workspace?;let runtime:String?;let root:String?}
 private var pending:Pending?;private(set) var confirmedReceipt:ResourcePublication<FileNode>?
 var hasPending:Bool{pending != nil}
 var hasDraft:Bool{modified}
 func reset(){guard !hasPending else{return};ticket=UUID();writeGuard=nil;reading=nil;rawReading=nil;members=[];outputs=[];preview=nil;notice="";busy=false;saved=false;modified=false;confirmedReceipt=nil}
 func close(){guard !hasPending else{return};ticket=UUID();preview=nil;busy=false}
 private func invalidate(){ticket=UUID();preview=nil;saved=false;modified=true}
 func choose(_ path:String){guard !busy,!hasPending,reading?.paths.contains(where:{exactUTF8($0.path,path)})==true else{return};invalidate();if let at=members.firstSourceIndex(path){members.remove(at:at)}else{members.append(path)}}
 func add(){guard !busy,!hasPending else{return};invalidate();outputs.append(GitResolutionOutput(path:"",identityFrom:nil,text:""))}
 func update(_ id:UUID,_ change:(inout GitResolutionOutput)->Void){guard !busy,!hasPending,let index=outputs.firstIndex(where:{$0.id==id}) else{return};let old=outputs[index];var next=old;change(&next);guard !exactUTF8(old.path,next.path) || !exactUTF8(old.identityFrom,next.identityFrom) || !exactUTF8(old.text,next.text) || old.mode != next.mode || !exactUTF8(old.stagePath,next.stagePath) || old.stage != next.stage || !exactUTF8(old.oid,next.oid) else{return};invalidate();outputs[index]=next}
 func remove(_ id:UUID){guard !busy,!hasPending else{return};invalidate();outputs.removeAll{$0.id==id}}
 private func call<T:Decodable>(_ engine:Engine,_ method:String,_ route:String,_ body:[String:Any]?=nil)async throws->T{if let transport{return try JSONDecoder().decode(T.self,from:gitResolutionData(try await transport(method,route,body)))};let url=URLComponents(string:route)!;return try await engine.request(method,url.path,query:url.queryItems ?? [],body:body)}
 func read(_ w:Workspace,anchor:String,selected:Bool=false)async {
  guard !busy,!hasPending,let e=w.engine else{return};let id=UUID(),epoch=w.sourceNavigationEpoch;ticket=id;busy=true;preview=nil;defer{if ticket==id{busy=false}}
  do{let paths=selected ? gitResolutionSorted(members):[anchor];let raw:ResourceJSON=try await call(e,"POST","/api/git/conflict-resolution/read",["version":1,"members":paths]);let value=try JSONDecoder().decode(GitResolutionReading.self,from:gitResolutionData(raw))
   guard ticket==id,w.engine?.session === e.session,w.sourceNavigationEpoch==epoch else{return};guard value.version==1,value.resolution_protocol==1,!value.index_modified,!value.source_files_modified,value.observation.version==1,!value.observation.repository.isEmpty,sourceHistoryHex(value.observation.fingerprint,64),sourceHistoryHex(value.observation.index_hash,64),value.observation.entries.count<=3000,value.paths.count<=1000,gitResolutionPaths(value.paths.map{$0.path}).count==value.paths.count,gitResolutionPaths(value.members)==gitResolutionPaths(paths) else{throw EngineError(message:"Complete conflict reading changed or is unsupported.")}
   for row in value.paths {try validateConflictStages(row.stages);if let source=row.working,source.kind=="file"{guard let text=source.text,nativeSourceHash(text)==source.sha256 else{throw EngineError(message:"Conflict working-source bytes changed.")}}}
   guard gitResolutionPaths(value.observation.entries.map{$0.path})==gitResolutionPaths(value.paths.map{$0.path}) else{throw EngineError(message:"Conflict observation does not enumerate its complete path scope.")}
   reading=value;rawReading=raw;members=paths
   if !selected,outputs.isEmpty,let row=value.paths.first(where:{exactUTF8($0.path,anchor)}),let working=row.working,working.kind=="file",let text=working.text{outputs=[GitResolutionOutput(path:anchor,identityFrom:anchor,text:w.buffers[anchor]?.text ?? text,mode:working.executable==true ? "100755":"100644")]}
   notice="Select the complete group, read selected stages, and choose every retained destination. Omitted members will be deleted."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 func prepare(_ w:Workspace)async {
  guard !busy,!hasPending,let reading=rawReading,let e=w.engine else{return};let id=UUID(),epoch=w.sourceNavigationEpoch;ticket=id;busy=true;preview=nil;saved=false;defer{if ticket==id{busy=false}}
  do{guard !w.resourceRestoration.busy,!w.resourceRestoration.hasPending,!members.isEmpty,members.count<=1000,outputs.count<=1000 else{throw EngineError(message:"Choose a complete bounded group and finish other pending source publications.")}
   let paths=gitResolutionSorted(Array(Set((members+outputs.map{$0.path}).map{NativeSourceKey($0)})).map{$0.string});guard paths.allSatisfy({!$0.isEmpty}),Set(paths).count==paths.count else{throw EngineError(message:"Choose every destination explicitly with names distinguishable by this native wire format.")}
   var edits:[String:ResourceJSON]=[:];for path in paths{edits[languageURI(path)] = .array([])};let edit=ResourceJSON.object(["changes":.object(edits)])
   let valid={w.engine?.session === e.session && w.sourceNavigationEpoch==epoch && (self.ticket==id || self.writeGuard==id)}
   let acquisition=try await acquireResourceInputs(w,paths:paths,edit:edit,current:valid,readInputs:{capture in try await self.call(e,"POST","/api/changes/resource-inputs",resourceBody(capture))})
   let body:[String:Any]=["version":1,"reading":try resourceBody(reading),"members":gitResolutionSorted(members),"outputs":outputs.map{$0.body},"inputs":try resourceBody(acquisition.inputs),"capture":try resourceBody(acquisition.capture)]
   let raw:ResourceJSON=try await call(e,"POST","/api/git/conflict-resolution/prepare",body);guard acquisition.current() else{throw EngineError(message:"Original source owners changed while preparing the conflict group.")};let material=try await Task.detached(priority:.userInitiated){try GitResolutionMaterial(raw)}.value;guard acquisition.current() else{throw EngineError(message:"Original sources changed while decoding the complete conflict review.")};let p=GitResolutionPreview(material,current:acquisition.current)
   let patchPaths=paths.filter{path in acquisition.inputs.sources[path]?.kind != "missing" || outputs.contains{exactUTF8($0.path,path)}}
   guard gitResolutionPaths(p.value.members)==gitResolutionPaths(members),gitResolutionPaths(p.patches.map{$0.path})==gitResolutionPaths(patchPaths),p.value.plan.context==acquisition.capture.context,p.value.proposal.inputs.workspace==acquisition.inputs.workspace,try gitResolutionData(p.value.proposal.inputs)==gitResolutionData(acquisition.inputs),try gitResolutionData(p.value.proposal.capture)==gitResolutionData(acquisition.capture),gitResolutionPaths(p.value.remaining_conflicts)==gitResolutionPaths(self.reading?.paths.filter{!members.containsSource($0.path)}.map{$0.path} ?? []),p.patches.allSatisfy({patch in patch.before.map{exactUTF8(w.buffers[patch.path]?.text,$0.text)} ?? (w.buffers[patch.path]==nil)}) else{throw EngineError(message:"Conflict review does not cover exactly the selected source namespace.")}
   preview=p;notice="Review the complete source/resource proposal, saved metadata and both generated layers. Draft Apply, Save and Stage are separate."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 private func body(_ p:GitResolutionPreview,stage:Bool=false)->[String:Any]{["version":1,"token":p.value.token,"fingerprint":p.value.fingerprint,"stage":stage]}
 private func validate(_ p:GitResolutionPreview,_ e:Engine,stage:Bool=false)async throws {
  guard p.current() else{throw EngineError(message:"Original source owners changed. Prepare again.")};let value:GitResolutionValidation=try await call(e,"POST","/api/git/conflict-resolution/validate",body(p,stage:stage))
  guard value.version==1,value.token==p.value.token,value.fingerprint==p.value.fingerprint,value.stage==stage,value.current,!value.source_files_modified,!value.index_modified else{throw EngineError(message:"Complete source, saved metadata or index authority changed. Prepare again.")}
  let expected=p.value;let same=try await Task.detached(priority:.userInitiated){try gitResolutionData(value.plan)==gitResolutionData(expected.plan) && gitResolutionSaved(value.saved_sources,expected.saved_sources)}.value
  guard p.current(),same else{throw EngineError(message:"Original source lease or complete reviewed metadata changed during validation.")}
 }
 func draft(_ w:Workspace)async {
  guard !busy,!hasPending,let p=preview,let e=w.engine else{return};let operation=ticket;busy=true;defer{if ticket==operation{busy=false}}
  do{guard p.draftable,w.changes.mode==nil else{throw EngineError(message:"Namespace changes require guarded Save. Finish the other source preview first.")};try await validate(p,e);var changes:[String:Any]=[:],before:[String:String]=[:]
   for patch in p.patches{let source=patch.before!.text;before[patch.path]=source;changes[languageURI(patch.path)]=[["range":["start":["line":0,"character":0],"end":languagePosition(source,source.utf16.count)],"newText":patch.after!.text]]}
   try w.changes.offer("Compose Git conflict group",edit:["changes":changes],expected:before,workspace:w);w.changes.apply(w);guard w.changes.plan==nil else{throw EngineError(message:w.changes.notice)};preview=nil;notice="Applied coordinated source drafts. Nothing saved or staged. Prepare again before saving."
  }catch{if ticket==operation{notice=error.localizedDescription}}
 }
 func act(_ w:Workspace,stage:Bool)async {
  guard !busy,!hasPending,let p=preview,let e=w.engine else{return};let operation=ticket
  defer{if ticket==operation || pending?.preview === p{busy=false}}
  do{guard p.current(),!w.resourceRestoration.busy,!w.resourceRestoration.hasPending,stage ? p.value.stage_available && (!p.value.needs_save || saved):p.value.needs_save && !saved else{throw EngineError(message:"Source changed or this publication is unavailable. Save the complete reviewed group before separate Stage.")};busy=true
   if !stage{try await validate(p,e)};guard p.current() else{throw EngineError(message:"Original source owners changed before publication.")};let id=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased();pending=Pending(id:id,preview:p,stage:stage,engine:e,workspace:w,runtime:w.sourceWorkspaceID,root:w.sourceRootAddress);writeGuard=ticket;var request=body(p);request["request_id"]=id;request["stage"]=nil
   let raw:ResourceJSON=try await call(e,"POST","/api/git/conflict-resolution/"+(stage ? "stage":"save"),request);try accept(w,raw)
  }catch{if ticket==operation || pending?.preview === p{notice=error.localizedDescription+(hasPending ? " Outcome unconfirmed; inspect the original request. Never repeat Save or Stage.":"")}}
 }
 private func owned(_ p:Pending,_ w:Workspace)->Bool{p.workspace === w && w.engine?.session === p.engine.session && exactUTF8(w.sourceWorkspaceID,p.runtime) && exactUTF8(w.sourceRootAddress,p.root)}
 private func accept(_ w:Workspace,_ raw:ResourceJSON)throws {
  guard let pending,owned(pending,w) else{throw EngineError(message:"Publication belongs to its original workspace and connection.")};let p=pending.preview
  if pending.stage{let receipt=try JSONDecoder().decode(GitResolutionStageReceipt.self,from:gitResolutionData(raw));try validateGitResolutionStage(receipt,request:pending.id,review:p.value);self.pending=nil;writeGuard=nil;preview=nil;modified=false;reading=nil;rawReading=nil;members=[];outputs=[];notice="Complete saved conflict group staged. Unsaved drafts remain separate; no tests were run."}
  else{let receipt=try JSONDecoder().decode(ResourcePublication<FileNode>.self,from:gitResolutionData(raw));_ = try confirmedResourceFiles(p.patches,receipt:receipt,request:pending.id,workspace:p.value.consequences.workspace,context:p.value.consequences.context,expected:p.value.saved_sources,address:{($0.path,!$0.dir && $0.generated_by==nil)});confirmedReceipt=receipt
   do{let recovery=try resourceRecoveryIdentity(receipt.outcome.recovery);_ = try w.adoptResourceReceipt(p.patches,receipt:receipt,request:pending.id,workspace:p.value.consequences.workspace,context:p.value.consequences.context,expected:p.value.saved_sources,current:p.current);w.changes.recordResourcePublication(recovery,intent:nil);self.pending=nil;writeGuard=nil;saved=true;modified=false;p.current=captureRetainedSourceNamespace(w);preview=p;notice="Saved conflict group confirmed; native editors and source recovery retained. Choose Stage complete group separately."}
   catch{throw EngineError(message:"Saved publication receipt is retained, but adoption is unconfirmed: "+error.localizedDescription)}
  }
 }
 func status(_ w:Workspace)async {
  guard !busy,let pending else{return};busy=true;defer{busy=false}
  do{guard owned(pending,w) else{throw EngineError(message:"Inspect this request on its original workspace connection.")};struct Status:Decodable{var version:Int;var request_id:String;var state:String;var outcome:ResourceJSON?;var error:String?}
   let value:Status=try await call(pending.engine,"GET","/api/git/conflict-resolution/status?request_id="+pending.id);guard value.version==1,value.request_id==pending.id else{throw EngineError(message:"Foreign publication status refused.")}
   switch value.state {case "pending","unknown":notice="Original request "+pending.id+" is "+value.state+". No retry, adoption or staging is authorized."
    case "completed":if let outcome=value.outcome, (try? accept(w,outcome)) != nil{return};guard pending.preview.current() else{throw EngineError(message:"Publication completed without confirmable adoption. Inspect retained recovery.")};try await validate(pending.preview,pending.engine,stage:pending.stage);self.pending=nil;writeGuard=nil;preview=nil;saved=false;notice="Completed refused publication; the complete original source/index preimage is confirmed. Read and prepare a fresh review."
    default:throw EngineError(message:"Unsupported publication status. Retain the original request.")}
  }catch{notice=error.localizedDescription+" Retain this request and inspect recovery; never retry an uncertain write."}
 }
}
private struct GitResolutionStageChoices:View {
 var model:GitResolutionModel;var reading:GitResolutionReading;var output:GitResolutionOutput
 private var selected:[GitResolutionPath]{reading.paths.filter{model.members.containsSource($0.path)}}
 var body:some View {VStack(alignment:.leading){ForEach(selected){row in ForEach(row.stages,id:\.stage){stage in
  if let source=stage.text {let label="Use "+row.path+" stage "+String(stage.stage)+" · "+stage.oid
   Button(label){model.update(output.id){value in value.text=source;value.stagePath=row.path;value.stage=stage.stage;value.oid=stage.oid;value.mode=stage.mode}}
  }
 }}}}
}
struct GitResolutionView:View {
 var workspace:Workspace;var path:String;@State private var page=0
 private var model:GitResolutionModel{workspace.gitResolution}
 private func text(_ output:GitResolutionOutput)->Binding<String>{Binding(get:{model.outputs.first{$0.id==output.id}?.text ?? ""},set:{value in model.update(output.id){$0.text=value;$0.stage=nil;$0.stagePath=nil;$0.oid=nil}})}
 var body:some View {let blocked=model.busy || model.hasPending
  VStack(alignment:.leading,spacing:6){
   HStack{Button("Compose conflict group…"){Task{await model.read(workspace,anchor:path)}}.disabled(blocked);Button("Discard composition"){model.reset()}.disabled(blocked)}
   Text(model.notice).font(.caption).textSelection(.enabled)
   if let receipt=model.confirmedReceipt{Text("Retained saved-source receipt "+receipt.request_id+" · complete "+String(receipt.outcome.complete)+". Local editor adoption is checked separately.").font(.caption).textSelection(.enabled)}
   if model.hasPending{Button("Inspect original resolution request"){Task{await model.status(workspace)}}.disabled(model.busy)}
   if let reading=model.reading {
    Text("Select every conflict member. Retained identities and rename relationships are explicit choices. Stage 1 is base; stages 2 and 3 retain Git’s meanings, including during rebase.").font(.caption)
    ForEach(reading.paths){row in Toggle(row.path,isOn:Binding(get:{model.members.containsSource(row.path)},set:{_ in model.choose(row.path)})).disabled(blocked);if let unavailable=row.unavailable{Text(unavailable).font(.caption)}}
    Button("Read selected group stages"){Task{await model.read(workspace,anchor:path,selected:true)}}.disabled(blocked || model.members.isEmpty)
    ForEach(model.outputs){output in
     TextField("Conflict destination",text:Binding(get:{model.outputs.first{$0.id==output.id}?.path ?? ""},set:{value in model.update(output.id){$0.path=value}})).disabled(blocked)
     Text("Retained identity: "+(output.identityFrom ?? "new source")).font(.caption)
     HStack{Button("New identity"){model.update(output.id){$0.identityFrom=nil}};ForEach(model.members,id:\.nativeSourceKey){member in Button("Retain "+member){model.update(output.id){$0.identityFrom=member}}}}.disabled(blocked)
     TextEditor(text:text(output)).font(.system(.caption,design:.monospaced)).frame(minHeight:90,maxHeight:180).disabled(blocked)
     HStack{Button("Mode "+output.mode){model.update(output.id){$0.mode=$0.mode=="100644" ? "100755":"100644"}};Button("Remove destination (delete if not retained elsewhere)"){model.remove(output.id)}}.disabled(blocked)
     GitResolutionStageChoices(model:model,reading:reading,output:output).disabled(blocked)
    }
    Button("Add explicit destination"){model.add()}.disabled(blocked)
    Text("Members without a retained destination will be deleted: "+model.members.filter{member in !model.outputs.contains{exactUTF8($0.identityFrom,member)}}.joined(separator:", ")).font(.caption)
    Button("Preview complete resolution consequences"){page=0;Task{await model.prepare(workspace)}}.disabled(blocked || model.members.isEmpty)
   }
   if let review=model.preview{Text("Complete immutable source/resource proposal, saved metadata and both generated layers").font(.headline);let shown=min(page,review.pages.pages.count-1)
    Text(review.pages.page(shown)).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    HStack{Button("Previous consequences"){page=shown-1}.disabled(shown==0);Text("\(shown+1)/\(review.pages.pages.count)");Button("Next consequences"){page=shown+1}.disabled(shown+1>=review.pages.pages.count)}
    if !review.value.stage_available{Text(review.value.stage_unavailable ?? "Staging unavailable").font(.caption)}
    Button("Apply text drafts only"){Task{await model.draft(workspace)}}.disabled(blocked || !review.current() || !review.draftable)
    Button("Save complete resolution group"){Task{await model.act(workspace,stage:false)}}.disabled(blocked || !review.current() || !review.value.needs_save || model.saved)
    Button("Stage complete saved group"){Task{await model.act(workspace,stage:true)}}.disabled(blocked || !review.current() || !review.value.stage_available || review.value.needs_save && !model.saved)
   }
  }.onDisappear{model.close()}
 }
}
