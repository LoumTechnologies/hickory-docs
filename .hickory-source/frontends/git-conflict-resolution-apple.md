<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose complete Apple conflict groups with separate source and index authority

For maintainers following a conflict group from immutable Git stages through a source preview, guarded Save and explicit Stage.

An absent conflict member omitted from the outputs has no source patch: the server deliberately excludes Missing→Missing while retaining that member in the index resolution. Compare the complete patch set against this derived source namespace, rather than requiring a fictitious file edit for an absent member.

## Keep the complete reading and every destination explicit

Group membership, retained identities, destination names, text or stage OIDs and executable modes belong to one workspace-held model. Choosing a stage never guesses a rename. Omitted members are explicit deletions. The original reading and complete generated consequences remain inspectable; source drafts, saved files and the Git index are separate states. Decode and prepare complete review pages off the main actor, then attach them only while the original source lease is still current.

<ui:copy id="git-resolution-apple-types">
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
</ui:copy>

## Retain original source owners and make each effect one-shot

Preparation uses the shared acquisition lease. Before a draft or saved source action, the service revalidates the complete reviewed plan and saved metadata. Save and Stage receive different request IDs. Once POST may have started, closing or cancellation keeps the original request inspectable and never resends it. Status uses the original engine; a completed refusal releases its pending slot only after positive proof of the complete original source/index preimage.

<ui:copy id="git-resolution-apple-controller">
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
</ui:copy>

## Use the existing floating Git review

Controls stay inside the existing review popover. The model owns proposal and request state across view closure. Every destination names whether it retains a member identity or creates a new one; stage OIDs and mode choices remain visible. Page the complete immutable receipt rather than shortening generated consequences.

<ui:copy id="git-resolution-apple-view">
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
   HStack{Button("Compose conflict group…"){Task{await model.read(workspace,anchor:path)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil);Button("Discard composition"){model.reset()}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)}
   Text(model.notice).font(.caption).textSelection(.enabled)
   if let receipt=model.confirmedReceipt{Text("Retained saved-source receipt "+receipt.request_id+" · complete "+String(receipt.outcome.complete)+". Local editor adoption is checked separately.").font(.caption).textSelection(.enabled)}
   if model.hasPending{Button("Inspect original resolution request"){Task{await model.status(workspace)}}.disabled(reason: model.busy ? "An operation is already in progress." : nil)}
   if let reading=model.reading {
    Text("Select every conflict member. Retained identities and rename relationships are explicit choices. Stage 1 is base; stages 2 and 3 retain Git’s meanings, including during rebase.").font(.caption)
    ForEach(reading.paths){row in Toggle(row.path,isOn:Binding(get:{model.members.containsSource(row.path)},set:{_ in model.choose(row.path)})).disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil);if let unavailable=row.unavailable{Text(unavailable).font(.caption)}}
    Button("Read selected group stages"){Task{await model.read(workspace,anchor:path,selected:true)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : model.members.isEmpty ? "Select paths to include in the resolution." : nil)
    ForEach(model.outputs){output in
     TextField("Conflict destination",text:Binding(get:{model.outputs.first{$0.id==output.id}?.path ?? ""},set:{value in model.update(output.id){$0.path=value}})).disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)
     Text("Retained identity: "+(output.identityFrom ?? "new source")).font(.caption)
     HStack{Button("New identity"){model.update(output.id){$0.identityFrom=nil}};ForEach(model.members,id:\.nativeSourceKey){member in Button("Retain "+member){model.update(output.id){$0.identityFrom=member}}}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)
     TextEditor(text:text(output)).font(.system(.caption,design:.monospaced)).frame(minHeight:90,maxHeight:180).disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)
     HStack{Button("Mode "+output.mode){model.update(output.id){$0.mode=$0.mode=="100644" ? "100755":"100644"}};Button("Remove destination (delete if not retained elsewhere)"){model.remove(output.id)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)
     GitResolutionStageChoices(model:model,reading:reading,output:output).disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)
    }
    Button("Add explicit destination"){model.add()}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : nil)
    Text("Members without a retained destination will be deleted: "+model.members.filter{member in !model.outputs.contains{exactUTF8($0.identityFrom,member)}}.joined(separator:", ")).font(.caption)
    Button("Preview complete resolution consequences"){page=0;Task{await model.prepare(workspace)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : model.members.isEmpty ? "Select paths to include in the resolution." : nil)
   }
   if let review=model.preview{Text("Complete immutable source/resource proposal, saved metadata and both generated layers").font(.headline);let shown=min(page,review.pages.pages.count-1)
    Text(review.pages.page(shown)).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    HStack{Button("Previous consequences"){page=shown-1}.disabled(reason: shown==0 ? "You’re on the first page." : nil);Text("\(shown+1)/\(review.pages.pages.count)");Button("Next consequences"){page=shown+1}.disabled(reason: shown+1>=review.pages.pages.count ? "You’re on the last page." : nil)}
    if !review.value.stage_available{Text(review.value.stage_unavailable ?? "Staging unavailable").font(.caption)}
    Button("Apply text drafts only"){Task{await model.draft(workspace)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : !review.current() ? "Refresh the resolution preview against current source." : !review.draftable ? "This resolution changes resources or metadata. Use the reviewed Save action." : nil)
    Button("Save complete resolution group"){Task{await model.act(workspace,stage:false)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : !review.current() ? "Refresh the resolution preview against current source." : !review.value.needs_save ? "This resolution has no pending changes to save." : model.saved ? "The reviewed resolution has already been saved." : nil)
    Button("Stage complete saved group"){Task{await model.act(workspace,stage:true)}}.disabled(reason: blocked ? (model.hasPending ? "Inspect the pending resolution request first." : "A conflict resolution operation is in progress.") : !review.current() ? "Refresh the resolution preview against current source." : !review.value.stage_available ? "This resolution cannot be staged in the current Git context." : review.value.needs_save && !model.saved ? "Save the reviewed resolution before staging." : nil)
   }
  }.onDisappear{model.close()}
 }
}
</ui:copy>

## Verify request identity and refusal before using a real repository

The controlled capability below supplies typed complete readings and publication receipts. These tests check client refusal and one-shot behavior; only the owned Git witness establishes real source/index effects. No mock response is counted as physical publication evidence.

<ui:copy id="git-resolution-apple-tests">
import XCTest
import SwiftUI
@testable import HickorySimple
#if os(macOS)
import AppKit
#endif
@MainActor final class GitConflictResolutionTests:XCTestCase {
 @MainActor final class Host {
  let w=Workspace();let b:DocumentBuffer;var calls:[String]=[];var mode="normal";var context="";var prepared:[String:Any]=[:];var lastRequest="";var session:URLSession
  init()throws{session=URLSession(configuration:.ephemeral);w.engine=try Engine(address:"http://127.0.0.1:1",session:session);w.sourceRootAddress="/fixture";w.sourceWorkspaceID=String(repeating:"e",count:32);b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"before",hash:"old",save_guard:"original"));w.buffers["a.py"]=b;w.layout.activateSource("a.py",in:"document-group");w.rootFiles=[b.file];w.gitResolution.transport={ [unowned self] method,path,body in try await self.call(method,path,body)}}
  func close(){w.gitResolution.transport=nil;w.buffers.values.forEach{$0.language?.close()};session.invalidateAndCancel()}
  func json(_ raw:Any)throws->ResourceJSON{try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:raw))}
  func saved(_ text:String)->[String:Any]{["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":420]}
  func source(_ text:String)->[String:Any]{["kind":"file","identity":"source-a","text":text,"writable":true,"removable":true,"version":NSNull()]}
  func call(_ method:String,_ path:String,_ body:[String:Any]?)async throws->ResourceJSON {
   calls.append(method+" "+path)
   if path.hasSuffix("/read"){
    let entry:[String:Any]=["path":"a.py","mode":"100644","oid":String(repeating:"a",count:40),"stage":2]
    let stage:[String:Any]=["stage":2,"mode":"100644","oid":String(repeating:"a",count:40),"text":"ours","unavailable":NSNull()]
    return try json(["version":1,"resolution_protocol":1,"observation":["version":1,"repository":"repo","fingerprint":String(repeating:"b",count:64),"index_hash":String(repeating:"c",count:64),"heads":["HEAD":String(repeating:"a",count:40)],"entries":[entry],"outside_conflicts":0],"members":body!["members"]!,"paths":[["path":"a.py","stages":[stage],"working":saved("before"),"unavailable":NSNull()]],"source_files_modified":false,"index_modified":false])
   }
   if path=="/api/changes/resource-inputs"{context=body!["context"] as! String;return try json(["version":1,"context":context,"workspace":String(repeating:"f",count:64),"ownership":["mode":"ordinary","entry":NSNull(),"protected_entries":[],"saved_sources":[:],"draft_sources":[:]],"sources":["a.py":source(b.text)],"saved_sources":["a.py":saved(b.base)],"refusals":[:],"source_files_modified":false,"saved_guards":["a.py":b.savedGuard!]])}
   if path.hasSuffix("/prepare"){
    let input=body!["inputs"] as! [String:Any],capture=body!["capture"] as! [String:Any],after=mode=="noop" ? "before":"resolved",plan:[String:Any]=["version":1,"context":context,"title":"Compose","patches":[["path":"a.py","before":(input["sources"] as! [String:Any])["a.py"]!,"after":source(after)]],"annotations":[]],layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]]
    var value:[String:Any]=["version":1,"resolution_protocol":1,"token":String(repeating:"1",count:32),"fingerprint":String(repeating:"2",count:64),"members":["a.py"],"remaining_conflicts":[],"proposal":["inputs":input,"capture":capture,"plan":plan],"plan":plan,"consequences":["version":1,"workspace":String(repeating:"f",count:64),"context":context,"fingerprint":String(repeating:"3",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false,"saved_sources":["a.py":saved(after)]],"saved_sources":["a.py":saved(after)],"needs_save":mode != "noop","stage_available":true,"stage_unavailable":NSNull(),"source_files_modified":false,"index_modified":false]
    if mode=="bad-token"{value["token"]="foreign"};if mode=="partial"{value["saved_sources"]=[:]} ;if mode=="bad-remaining"{value["remaining_conflicts"]=["foreign.py"]};prepared=value;return try json(value)
   }
   if path.hasSuffix("/validate"){return try json(["version":1,"token":String(repeating:"1",count:32),"fingerprint":String(repeating:"2",count:64),"stage":body?["stage"] ?? false,"current":true,"plan":prepared["plan"]!,"saved_sources":prepared["saved_sources"]!,"source_files_modified":false,"index_modified":false])}
   if path.hasSuffix("/save"){lastRequest=body!["request_id"] as! String;if mode=="lost"{throw EngineError(message:"Lost response")};return try json(["version":2,"request_id":lastRequest,"workspace":String(repeating:"f",count:64),"context":context,"adoption_current":true,"outcome":["complete":true,"recovery":".hick-cache/resource-changes/"+String(repeating:"4",count:32)+"/manifest.json","applied":["a.py"]],"saved_sources":["a.py":saved("resolved")],"source_files":[["file":["name":"a.py","path":"a.py","dir":false],"saved":saved("resolved"),"hash":"new","save_guard":"published"]]])}
   if path.hasSuffix("/stage"){return try json(["version":1,"operation":"git_conflict_stage","request_id":body!["request_id"]!,"token":String(repeating:"1",count:32),"review_fingerprint":String(repeating:"2",count:64),"complete":mode != "bad-stage","index_modified":true,"index_uncertain":mode=="bad-stage","source_files_modified":false,"fingerprint":String(repeating:"5",count:64),"resolved_members":["a.py"],"remaining_conflicts":[]])}
   if path.contains("/status?"){return try json(["version":1,"request_id":lastRequest,"state":mode=="completed" ? "completed":"unknown","outcome":NSNull(),"error":"refused"])}
   throw EngineError(message:"Unexpected fixture route "+path)
  }
  func preview()async{await w.gitResolution.read(w,anchor:"a.py");await w.gitResolution.prepare(w)}
 }
 func testCompleteReviewDraftUndoAndSeparateSavedStage()async throws {
  let h=try Host();defer{h.close()};let m=h.w.gitResolution;await h.preview();XCTAssertNotNil(m.preview,m.notice);XCTAssertEqual(h.b.text,"before");await m.draft(h.w);XCTAssertEqual(h.b.text,"resolved",m.notice);h.w.changes.open("undo",workspace:h.w);XCTAssertEqual(h.b.text,"before");h.w.changes.open("redo",workspace:h.w);XCTAssertEqual(h.b.text,"resolved");h.w.changes.open("undo",workspace:h.w)
  await m.read(h.w,anchor:"a.py",selected:true);await m.prepare(h.w);await m.act(h.w,stage:false);XCTAssertTrue(m.saved,m.notice);XCTAssertFalse(m.hasPending);XCTAssertEqual(h.b.text,"resolved");XCTAssertEqual(h.b.base,"resolved");XCTAssertEqual(h.b.savedGuard,"published");XCTAssertEqual(h.calls.filter{$0.hasSuffix("/save")}.count,1);XCTAssertFalse(h.calls.contains{$0.hasSuffix("/stage")});await m.act(h.w,stage:true);XCTAssertTrue(m.notice.contains("staged"),m.notice);XCTAssertFalse(m.hasPending);XCTAssertEqual(h.calls.filter{$0.hasSuffix("/stage")}.count,1)
 }
 func testNoopSaveIsUnavailableButCompleteSavedGroupCanStage()async throws {
  let h=try Host();defer{h.close()};h.mode="noop";await h.preview();let m=h.w.gitResolution;XCTAssertNotNil(m.preview,m.notice);await m.act(h.w,stage:false);XCTAssertFalse(h.calls.contains{$0.hasSuffix("/save")});await m.act(h.w,stage:true);XCTAssertTrue(m.notice.contains("staged"),m.notice);XCTAssertEqual(h.b.text,"before")
 }
 func testMalformedTokenPartialMetadataAndForeignRemainingGroupRefuse()async throws {for mode in ["bad-token","partial","bad-remaining"]{let h=try Host();defer{h.close()};h.mode=mode;await h.preview();XCTAssertNil(h.w.gitResolution.preview,mode);await h.w.gitResolution.act(h.w,stage:false);XCTAssertFalse(h.calls.contains{$0.hasSuffix("/save")});XCTAssertEqual(h.b.text,"before")}}
 func testOriginalOwnerUndoGuardBaselineAndNewConstructorRetireBeforeSave()async throws {for mode in ["undo","guard","baseline","owner"]{let h=try Host();defer{h.close()};await h.preview();XCTAssertNotNil(h.w.gitResolution.preview,h.w.gitResolution.notice);switch mode{case "undo":h.b.text="changed";h.b.text="before";case "guard":h.b.savedGuard="foreign";case "baseline":h.b.base="other";default:h.w.buffers["new.md"]=DocumentBuffer(file:FileNode(name:"new.md",path:"new.md",dir:false),text:FileText(path:"new.md",content:"new constructor",hash:"new",save_guard:"other"))};await h.w.gitResolution.act(h.w,stage:false);XCTAssertFalse(h.calls.contains{$0.hasSuffix("/save")},mode);XCTAssertFalse(h.w.gitResolution.hasPending)}}
 func testLostWriteNeverRepeatsAndOnlyPositiveOriginalPreimageRetiresIt()async throws {
  let h=try Host();defer{h.close()};await h.preview();h.mode="lost";let m=h.w.gitResolution;await m.act(h.w,stage:false);XCTAssertTrue(m.hasPending);m.close();await m.act(h.w,stage:false);XCTAssertEqual(h.calls.filter{$0.hasSuffix("/save")}.count,1);await m.status(h.w);XCTAssertTrue(m.hasPending);h.mode="completed";await m.status(h.w);XCTAssertFalse(m.hasPending,m.notice);XCTAssertNil(m.preview);XCTAssertEqual(h.b.text,"before")
 }
 func testIncompleteStageKeepsOriginalRequestAndCannotRepeat()async throws {
  let h=try Host();defer{h.close()};h.mode="noop";await h.preview();h.mode="bad-stage";let m=h.w.gitResolution;await m.act(h.w,stage:true);XCTAssertTrue(m.hasPending);await m.act(h.w,stage:true);XCTAssertEqual(h.calls.filter{$0.hasSuffix("/stage")}.count,1);XCTAssertEqual(h.b.text,"before")
 }
 #if os(macOS)
 func testLiveNativeCompositionSaveStageAndEditorHistory()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_GIT_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_NATIVE_GIT_WORKSPACE"],let input=ProcessInfo.processInfo.environment["HICKORY_NATIVE_GIT_INPUT"],let kind=ProcessInfo.processInfo.environment["HICKORY_NATIVE_GIT_CASE"] else{throw XCTSkip("Use the maintained owned native Git resolution fixture.")}
  let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-native-git-fixture"),encoding:.utf8)=="owned-native-git-v1\n" else{throw EngineError(message:"An isolated owned Git fixture is required.")}
  func git(_ args:[String])throws->Data{let process=Process(),pipe=Pipe();process.executableURL=URL(fileURLWithPath:"/usr/bin/git");process.arguments=["-C",root,"--literal-pathspecs"]+args;process.standardOutput=pipe;process.standardError=FileHandle.nullDevice;try process.run();let bytes=pipe.fileHandleForReading.readDataToEndOfFile();process.waitUntilExit();guard process.terminationStatus==0 else{throw EngineError(message:"Owned fixture Git command failed.")};return bytes}
  let indexURL=directory.appendingPathComponent(".git/index"),index=try Data(contentsOf:indexURL),head=try git(["rev-parse","HEAD"])
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:origin),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if exactUTF8(node.path,path){return node};if let child=find(path,node.children ?? []){return child}};return nil}
  let file=try XCTUnwrap(find(input,w.rootFiles)),b=try await w.loadBuffer(file),disk=try Data(contentsOf:directory.appendingPathComponent(input));w.layout.activateSource(input,in:"document-group")
  let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
  defer{w.gitResolution.close();w.debugger?.close();w.sourceTestIndex?.stop();for item in Array(w.buffers.values)+Array(w.retainedResourceEditors.values){item.language?.close()};window.orderOut(nil);window.contentView=nil}
  func editor(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
  func settle(_ message:String,_ predicate:()->Bool)async throws{for _ in 0..<300{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:message)}
  try await settle("Native source editor did not attach"){editor(host) != nil};let native=try XCTUnwrap(editor(host)),scroll=try XCTUnwrap(native.enclosingScrollView),identity=try XCTUnwrap(b.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host),manager=try XCTUnwrap(native.undoManager)
  native.setSelectedRange(NSRange(location:native.string.utf16.count,length:0));native.insertText("\n# retained unsaved draft",replacementRange:native.selectedRange());native.setSelectedRange(NSRange(location:2,length:2));b.selection=native.selectedRange();let draft=b.text,selection=b.selection
  try XCTUnwrap(w.commands().first{$0.id=="workspace.review"}).run();try await settle("Repository review did not load"){!w.gitReview.loading&&w.gitReview.files.contains{exactUTF8($0.path,input)}}
  await w.gitReview.select(input,staged:false){path,staged in try await e.request("GET","/api/git/diff",query:[URLQueryItem(name:"path",value:path),URLQueryItem(name:"staged",value:String(staged))])}
  let m=w.gitResolution;await m.read(w,anchor:input);XCTAssertNotNil(m.reading,m.notice)
  if kind=="rename"{for member in try XCTUnwrap(m.reading).paths.map(\.path) where !m.members.containsSource(member){m.choose(member)};await m.read(w,anchor:input,selected:true)}
  let destination=kind=="rename" ? "resolved λ%2F?#.py":input,resolved="composed = 41\n"
  if kind=="delete"{for output in m.outputs{m.remove(output.id)}}else{let output=try XCTUnwrap(m.outputs.first);m.update(output.id){$0.path=destination;$0.identityFrom=input;$0.text=resolved;$0.stage=nil;$0.stagePath=nil;$0.oid=nil}}
  await m.prepare(w);XCTAssertNotNil(m.preview,m.notice);XCTAssertEqual(b.text,draft);XCTAssertEqual(b.selection,selection);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(try Data(contentsOf:directory.appendingPathComponent(input)),disk);XCTAssertEqual(try Data(contentsOf:indexURL),index)
  if kind=="stale"{native.insertText("later",replacementRange:native.selectedRange());let changed=b.text;await m.act(w,stage:false);XCTAssertFalse(m.hasPending);XCTAssertEqual(b.text,changed);XCTAssertEqual(try Data(contentsOf:directory.appendingPathComponent(input)),disk);XCTAssertEqual(try Data(contentsOf:indexURL),index);return}
  if kind=="index"{try Data("foreign index change\n".utf8).write(to:directory.appendingPathComponent("clean.py"));_ = try git(["add","--","clean.py"]);let changed=try Data(contentsOf:indexURL);await m.act(w,stage:false);XCTAssertFalse(m.hasPending,m.notice);XCTAssertEqual(b.text,draft);XCTAssertEqual(try Data(contentsOf:directory.appendingPathComponent(input)),disk);XCTAssertEqual(try Data(contentsOf:indexURL),changed);return}
  if kind=="native"{await m.draft(w);XCTAssertEqual(b.text,resolved,m.notice);w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,draft);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,resolved);w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,draft);try await Task.sleep(for:.milliseconds(150));try XCTUnwrap(w.commands().first{$0.id=="workspace.review"}).run();try await settle("Repository review did not reopen"){!w.gitReview.loading};await w.gitReview.select(input,staged:false){path,staged in try await e.request("GET","/api/git/diff",query:[URLQueryItem(name:"path",value:path),URLQueryItem(name:"staged",value:String(staged))])};try await Task.sleep(for:.milliseconds(150));await m.read(w,anchor:input,selected:true);await m.prepare(w);XCTAssertNotNil(m.preview,m.notice)}
  let saveSelection=b.selection,members=try XCTUnwrap(m.preview).value.members;await m.act(w,stage:false);XCTAssertFalse(m.hasPending,m.notice);XCTAssertTrue(m.saved,m.notice);XCTAssertEqual(try Data(contentsOf:indexURL),index);XCTAssertTrue(w.currentBuffer === b);XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertTrue(editor(host) === native);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
  if kind=="delete"{XCTAssertTrue(b.resourceDeleted);XCTAssertEqual(b.text,draft);XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent(input).path))}else{XCTAssertEqual(b.file.path,destination);XCTAssertEqual(b.text,resolved);XCTAssertEqual(b.base,resolved);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(destination),encoding:.utf8),resolved)}
  await m.act(w,stage:true);XCTAssertFalse(m.hasPending,m.notice);XCTAssertTrue(m.notice.contains("staged"),m.notice);XCTAssertEqual(try git(["rev-parse","HEAD"]),head);XCTAssertEqual(try git(["ls-files","--unmerged","--"]+members),Data());XCTAssertEqual(try git(["show",":clean.py"]),Data("clean\n".utf8));XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertTrue(editor(host) === native)
  if kind != "delete"{XCTAssertEqual(try git(["show",":0:"+destination]),Data(resolved.utf8));manager.undo();XCTAssertEqual(b.text,draft);XCTAssertEqual(b.selection,saveSelection);manager.redo();XCTAssertEqual(b.text,resolved);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(destination),encoding:.utf8),resolved);XCTAssertEqual(try git(["show",":0:"+destination]),Data(resolved.utf8))}
 }
 #endif

}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/GitConflictResolution.swift"><ui:paste select="#git-resolution-apple-types"/><ui:paste select="#git-resolution-apple-controller"/><ui:paste select="#git-resolution-apple-view"/></ui:file>
<ui:file path="apps/apple-simple/Tests/GitConflictResolutionTests.swift"><ui:paste select="#git-resolution-apple-tests"/></ui:file>

## Exercise source and index authority in real native editors

The owned runner creates five disposable Git repositories, starts its own engine and invokes exactly one native acceptance per case. Text composition checks coordinated draft Undo/Redo and later native receipt Undo/Redo; deletion retains the existing editor; rename resolves the entire three-path group. Source edits and external index mutations must refuse Save before publication. Each test uses an actual AppKit editor and asserts identity and geometry; it does not establish physical pointer, keyboard or accessibility acceptance.

<ui:copy id="git-resolution-apple-runner">
#!/usr/bin/env python3
"""Owned real Git/native editor conflict composition acceptance; use the Xcode toolchain."""
import importlib.util,os,tempfile,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
def mod(name,path):
 s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
f=mod('fixture','scripts/test-simple-browser.py');g=mod('gitfixture','scripts/test-git-conflict-resolutions.py')
for kind in ['native','delete','rename','stale','index']:
 with tempfile.TemporaryDirectory(prefix='hickory-apple-git-') as t:
  root=Path(t)/'workspace';path=g.make(root,kind);path='left.py' if kind=='rename' else path
  (root/'.hickory-native-git-fixture').write_text('owned-native-git-v1\n')
  engine=f.start([str(R/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(Path(t)/'state')})
  try:
   env={**os.environ,'HICKORY_NATIVE_GIT_ORIGIN':f.origin(engine),'HICKORY_NATIVE_GIT_WORKSPACE':str(root),'HICKORY_NATIVE_GIT_INPUT':path,'HICKORY_NATIVE_GIT_CASE':kind}
   log=Path('/tmp/apple-git-'+kind+'.log')
   with log.open('w') as out:p=subprocess.run(['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--skip-build','--filter','GitConflictResolutionTests.testLiveNativeCompositionSaveStageAndEditorHistory$'],env=env,stdout=out,stderr=subprocess.STDOUT)
   print(kind,p.returncode,flush=True)
   if p.returncode: raise RuntimeError(str(log)+'\n'+log.read_text()[-7000:])
   assert 'Executed 1 test, with 0 failures' in log.read_text() and 'Test skipped' not in log.read_text()
  finally:f.stop(engine)
</ui:copy>
<ui:file path="scripts/test-git-resolution-apple.py"><ui:paste select="#git-resolution-apple-runner"/></ui:file>
</ui:doc>
