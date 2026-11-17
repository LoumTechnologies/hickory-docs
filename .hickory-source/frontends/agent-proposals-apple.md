<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Admit one complete sealed agent intention

For maintainers reviewing agent source changes on macOS and iOS. Verify the exact UTF-8 material digest before parsing or displaying its explanation, patches or evidence. The immutable reading retains source edits, generated consequences and connected before/after readings together. Claims and requested checks are declarations, never execution receipts.

Bound both the complete JSON structure and displayed pages. Native JSON admission refuses ambiguous contributor keys rather than silently merging byte-distinct physical names. Readable source editors remain independent of this optional capability.

<ui:copy id="agent-proposals-apple-seal">
import Foundation
import Observation
import SwiftUI

struct AgentProposalEntry:Equatable,Identifiable,Decodable {let token:String;let fingerprint:String;var id:String{token}}
struct AgentProposalSection {let title:String;let text:String;let pages:[NSRange]
    init(_ title:String,_ text:String)throws{guard text.utf8.count<=64*1024*1024 else{throw EngineError(message:"Proposal display section exceeds64MiB.")};self.title=title;self.text=text;var ranges:[NSRange]=[];var from=0,offset=0,count=0;for scalar in text.unicodeScalars{if count>0&&count%4000==0{ranges.append(NSRange(location:from,length:offset-from));from=offset};offset+=scalar.value>65535 ? 2:1;count+=1};ranges.append(NSRange(location:from,length:offset-from));pages=ranges}
    func page(_ index:Int)->String{(text as NSString).substring(with:pages[min(max(index,0),pages.count-1)])}
}
private func agentHex(_ text:String,_ count:Int)->Bool{text.utf8.count==count&&text.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
func agentProposalData<T:Encodable>(_ value:T)throws->Data{let encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];return try encoder.encode(value)}
func agentProposalBody<T:Encodable>(_ value:T)throws->[String:Any]{guard let body=try JSONSerialization.jsonObject(with:agentProposalData(value)) as? [String:Any] else{throw EngineError(message:"Proposal request must be an object.")};return body}
func validateNativeAgentProposalResponse(_ data:Data)throws {
 guard data.count<=64*1024*1024 else{throw EngineError(message:"Proposal response exceeds64MiB; retained sources were not changed.")}
 var quoted=false,escaped=false,depth=0,containers=0
 for byte in data {if quoted{if escaped{escaped=false}else if byte==92{escaped=true}else if byte==34{quoted=false}}else if byte==34{quoted=true}else if byte==123||byte==91{depth+=1;containers+=1;guard depth<=64,containers<=65536 else{throw EngineError(message:"Proposal response structure exceeds the native read budget.")}}else if byte==125||byte==93{depth-=1;guard depth>=0 else{throw EngineError(message:"Malformed proposal response.")}}}
 guard !quoted,depth==0 else{throw EngineError(message:"Malformed proposal response.")};try validateNativeJSON(data,canonicalVersionKeys:["version","proposal_protocol"])
 struct Header:Decodable {let version:Int?;let proposal_protocol:Int?};_ = try JSONDecoder().decode(Header.self,from:data)
}
private func agentMaterialBudget(_ text:String)throws{guard text.utf8.count<=32*1024*1024 else{throw EngineError(message:"Complete proposal exceeds32MiB.")};var depth=0,containers=0,quoted=false,escaped=false;for byte in text.utf8{if quoted{if escaped{escaped=false}else if byte==92{escaped=true}else if byte==34{quoted=false}}else if byte==34{quoted=true}else if byte==123||byte==91{depth+=1;containers+=1;guard depth<=64,containers<=65536 else{throw EngineError(message:"Proposal structure exceeds the bounded native reading.")}}else if byte==125||byte==93{depth-=1;guard depth>=0 else{throw EngineError(message:"Malformed proposal structure.")}}};guard !quoted,depth==0 else{throw EngineError(message:"Malformed complete proposal material.")}}
struct AgentProposalEnvelope:Decodable {let version:Int;let proposal_protocol:Int;let token:String;let fingerprint:String;let workspace:String;let namespace:String;let material:String;let source_effects:Bool;let checks_executed:Bool;let host_application:ResourceJSON?}
struct AgentProposalIntention:Decodable {let version:Int;let request_id:String;let title:String;let explanation:String;let claims:[String];let check_ids:[String];let capture:ResourceWireCapture;let editor_snapshot:ResourceJSON?}
struct AgentProposalMaterial:Decodable {let version:Int;let workspace:String;let namespace:String;let origin_authority:String;let intention:AgentProposalIntention;let inputs:ResourceWireInputs;let plan:ResourceWirePlan;let consequences:ResourceJSON;let before:ResourceJSON;let after:ResourceJSON;let saved_guards:[String:String]}
// All retained properties are immutable value snapshots. Parse and page construction
// run off the UI actor; no editor, transport or mutable Foundation container crosses.
final class SealedAgentProposal:@unchecked Sendable {
    let envelope:AgentProposalEnvelope;let material:AgentProposalMaterial;let patches:[ResourceEditorPatch];let sections:[AgentProposalSection];let proposal:ResourceWireProposal;let facts:ResourceWireFacts;let saved:[String:ResourceSaved]
    var token:String{envelope.token};var fingerprint:String{envelope.fingerprint};var reported:Bool{if case .null?=envelope.host_application{return false};return envelope.host_application != nil}
    init(_ envelope:AgentProposalEnvelope,expected:AgentProposalEntry,workspace:String,namespace:String)throws{
        guard envelope.version==1,envelope.proposal_protocol==1,exactUTF8(envelope.token,expected.token),exactUTF8(envelope.fingerprint,expected.fingerprint),agentHex(envelope.token,32),agentHex(envelope.fingerprint,64),exactUTF8(envelope.workspace,workspace),exactUTF8(envelope.namespace,namespace),!envelope.source_effects,!envelope.checks_executed else{throw EngineError(message:"Foreign or unsupported complete proposal. Nothing was applied.")}
        guard envelope.material.utf8.count<=32*1024*1024 else{throw EngineError(message:"Complete proposal exceeds32MiB.")};guard nativeSourceHash(envelope.material)==envelope.fingerprint else{throw EngineError(message:"Complete proposal SHA256 seal failed; no explanation, patch or evidence was admitted.")};try agentMaterialBudget(envelope.material)
        let data=Data(envelope.material.utf8);try validateNativeJSON(data,canonicalVersionKeys:["version","proposal_protocol","behavior_protocol"]);let value=try JSONDecoder().decode(AgentProposalMaterial.self,from:data);let facts=try JSONDecoder().decode(ResourceWireFacts.self,from:agentProposalData(value.consequences))
        guard ["caller-provided intention; not an agent attestation","engine-managed agent tool; live editor capture at preparation; claims remain unverified","engine-managed agent tool; non-live submitted editor snapshot; claims remain unverified"].contains(value.origin_authority),value.version==1,value.intention.version==1,agentHex(value.intention.request_id,32),exactUTF8(value.workspace,workspace),exactUTF8(value.namespace,namespace),!value.intention.title.isEmpty,value.intention.title.utf8.count<=512,!value.intention.explanation.isEmpty,value.intention.explanation.utf8.count<=65536,value.intention.claims.count<=128,value.intention.claims.allSatisfy({$0.utf8.count<=8192}),value.intention.check_ids.count<=256,value.plan.version==1,exactUTF8(value.plan.context,value.inputs.context),exactUTF8(value.plan.context,value.intention.capture.context),exactUTF8(facts.context,value.plan.context),exactUTF8(value.inputs.workspace,workspace),exactUTF8(facts.workspace,workspace),facts.protected_targets.isEmpty else{throw EngineError(message:"Complete proposal material is malformed or targets generated output.")}
        let patches=try value.plan.editors();guard !patches.isEmpty,patches.count<=1000,Set(patches.map{NativeSourceKey($0.path)}).count==patches.count,let saved=facts.saved_sources,Set(saved.keys.map(NativeSourceKey.init))==Set(patches.map{NativeSourceKey($0.path)}) else{throw EngineError(message:"Complete saved consequences do not cover the bounded source namespace.")}
        for reading in [value.before,value.after]{guard case .object(let object)=reading,case .integer(1)?=object["version"],case .integer(1)?=object["behavior_protocol"],case .string(let address)?=object["namespace"],case .string(let runtime)?=object["engine_workspace_id"],exactUTF8(address,namespace),exactUTF8(runtime,workspace) else{throw EngineError(message:"Connected readings belong to another proposal namespace.")}}
        self.envelope=envelope;material=value;self.facts=facts;self.patches=patches;self.saved=saved;proposal=ResourceWireProposal(inputs:value.inputs,capture:value.intention.capture,plan:value.plan)
        func printed<T:Encodable>(_ item:T)throws->String{String(decoding:try agentProposalData(item),as:UTF8.self)}
        sections=try [AgentProposalSection("Explanation · caller/agent text, not evidence",value.intention.explanation),AgentProposalSection("Claims · unverified statements",printed(value.intention.claims)),AgentProposalSection("Complete ordered source/resource patches",printed(value.plan)),AgentProposalSection("Saved and draft generated consequences",printed(value.consequences)),AgentProposalSection("Before contract/explanation/implementation/tests/evidence",printed(value.before)),AgentProposalSection("After connected readings · no checks executed",printed(value.after)),AgentProposalSection("Requested checks · declarations only",printed(value.intention.check_ids))]
    }
    var textOnly:Bool{patches.allSatisfy{$0.before != nil&&$0.after != nil&&$0.before?.identity==$0.after?.identity}&&patches.contains{!exactUTF8($0.before?.text,$0.after?.text)}}
}

</ui:copy>


## Preserve original sender authority through every asynchronous boundary

Apply the original sealed edit ranges only after reconstructing every sealed before/after patch. Replacing a whole document would unnecessarily move the caret and lose the original edit mapping. Preparation failures leave the proposal available for review.

Reading a proposal never grants effects. Acquire only its exact missing target editors, keep every original retained owner unchanged, then capture a new complete host lease. Managed intentions also require their original editor IDs and monotonic sender revisions; an edit followed by Undo cannot manufacture the original observation. Closing a source retires that sender context without discarding its editor or native history.

<ui:copy id="agent-proposals-apple-controller">
@MainActor func nativeAgentContext(_ w:Workspace)throws->[String:Any] {
 let ownership=try resourceOwnershipSnapshot(w);var seen:Set<UUID>=[];var rows:[[String:Any]]=[]
 for b in Array(w.buffers.values)+Array(w.retainedResourceEditors.values) where b.file.generated_by==nil && seen.insert(b.id).inserted {
  let named=ownership.named.first{exactUTF8($0.key,b.file.path)} != nil && w.buffers[b.file.path] === b
  var row:[String:Any]=["id":b.id.uuidString,"name":b.file.name,"path":named ? b.file.path as Any:NSNull(),"content":b.text,"focused":w.currentBuffer === b,"kind":b.file.doc_id==nil ? "source":"construction-source","document":b.file.doc_id as Any? ?? NSNull(),"observations":b.agentSourceRevision.map{[["kind":"editor_revision","version":1,"source_revision":String($0)]]} ?? []]
  if named,let guardToken=b.savedGuard{row["save_guard"]=guardToken};rows.append(row)
 };return ["live":false,"buffers":rows]
}
private struct AgentProposalParseInput:@unchecked Sendable {let envelope:AgentProposalEnvelope;let entry:AgentProposalEntry;let workspace:String;let namespace:String
 func parse()throws->SealedAgentProposal{try SealedAgentProposal(envelope,expected:entry,workspace:workspace,namespace:namespace)}
}
private struct AgentProposalReport:Decodable {let version:Int;let proposal_protocol:Int;let token:String;let fingerprint:String;let workspace:String;let namespace:String;let request_id:String;let host:String;let after_hashes:[String:String];let report_fingerprint:String;let saved:Bool;let checks_executed:Bool}
private struct AgentProposalCatalogue:Decodable {let version:Int;let proposal_protocol:Int;let workspace:String;let namespace:String;let reviews:[AgentProposalEntry]}
private struct AgentProposalValidation:Decodable {let version:Int;let proposal_protocol:Int;let admitted:Bool;let source_effects:Bool;let checks_executed:Bool;let token:String;let fingerprint:String;let workspace:String;let namespace:String}
@MainActor @Observable final class AgentProposalModel {
 var showing=false;private(set) var busy=false;private(set) var notice="";private(set) var entries:[AgentProposalEntry]=[];private(set) var value:SealedAgentProposal?;private(set) var phase="Not applied"
 var transport:((String,String,[String:Any]?)async throws->ResourceJSON)?
 private var ticket=UUID();private var current:()->Bool={false};private var consumed:Set<String>=[];private weak var owner:Workspace?;private var server:Engine?;private var staged:UUID?;private var reports:[String:[String:Any]]=[:]
 var canApply:Bool{reports.count<32&&consumed.count<4096 && !busy&&current()&&value.map{!$0.reported && !consumed.contains($0.token)}==true}
 var authorityCurrent:Bool{current()}
 var hasReport:Bool{value.map{reports[$0.token] != nil}==true}
 func close(){showing=false;if owner?.resourceRestoration.hasPending != true{if owner?.resourceRestoration.review?.identity==staged{owner?.resourceRestoration.cancel()};ticket=UUID();current={false};busy=false;staged=nil}}
 private func call<T:Decodable>(_ engine:Engine,_ method:String,_ path:String,_ body:[String:Any]?=nil)async throws->T {if let transport{return try JSONDecoder().decode(T.self,from:agentProposalData(try await transport(method,path,body)))};return try await engine.request(method,path,body:body)}
 func open(_ w:Workspace)async {showing=true;guard !busy,w.resourceRestoration.hasPending==false,let engine=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress else{notice="Connect and refresh Files before reviewing retained intentions.";return};let id=UUID();ticket=id;busy=true;value=nil;current={false};owner=w;server=engine;defer{if ticket==id{busy=false}}
  do{let rows:AgentProposalCatalogue=try await call(engine,"GET","/api/agent-proposals");guard ticket==id,w.engine?.session === engine.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace),rows.version==1,rows.proposal_protocol==1,exactUTF8(rows.workspace,workspace),exactUTF8(rows.namespace,namespace),rows.reviews.count<=32,rows.reviews.allSatisfy({agentHex($0.token,32)&&agentHex($0.fingerprint,64)}),Set(rows.reviews.map{$0.token}).count==rows.reviews.count else{throw EngineError(message:"Unsupported, obsolete or foreign proposal catalogue.")};entries=rows.reviews;notice="Read a complete sealed intention. Catalogue labels are not admitted review material."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 private func matches(_ w:Workspace,_ p:SealedAgentProposal)throws {
  let roster=try resourceOwnershipSnapshot(w),capture=p.material.intention.capture
  func equalMap(_ a:[String:String],_ b:[String:String])->Bool{a.count==b.count&&a.allSatisfy{key,value in b.first{exactUTF8($0.key,key)}.map{exactUTF8($0.value,value)}==true}}
  var allowed=capture.ownership_sources;for patch in p.patches where allowed[patch.path]==nil {if let before=patch.before{allowed[patch.path]=before.text}}
  guard roster.named.allSatisfy({key,text in allowed.first{exactUTF8($0.key,key)}.map{exactUTF8($0.value,text)}==true}),capture.ownership_sources.allSatisfy({key,text in roster.named.first{exactUTF8($0.key,key)}.map{exactUTF8($0.value,text)}==true}),equalMap(roster.unaddressed,capture.unaddressed_sources) else{throw EngineError(message:"Complete retained ownership roster changed. Request a new intention.")}
  let buffers=Array(w.buffers.values)+Array(w.retainedResourceEditors.values)
  if let original=p.material.intention.editor_snapshot,case .object(let object)=original,case .array(let rows)?=object["buffers"] {
   guard case .bool?=object["live"],rows.count<=4096 else{throw EngineError(message:"Malformed bounded original editor context.")}
   var ids:Set<String>=[]
   for row in rows{guard case .object(let fields)=row,case .string(let id)?=fields["id"],ids.insert(id).inserted,let b=buffers.first(where:{$0.id.uuidString==id}),case .string(let revision)?=fields["source_revision"],b.agentSourceRevision.map({String($0)==revision})==true,case .string(let hash)?=fields["source_hash"],nativeSourceHash(b.text)==hash else{throw EngineError(message:"Original sender editor/revision is unavailable or changed. A fresh host lease cannot upgrade it.")};if case .string(let path)?=fields["path"]{guard exactUTF8(path,b.file.path),w.buffers[path] === b,case .string(let guardToken)?=fields["save_guard"],exactUTF8(guardToken,b.savedGuard) else{throw EngineError(message:"Original physical source identity changed.")}}else{guard roster.unaddressed[id] != nil else{throw EngineError(message:"Original anonymous editor changed identity.")}}}
  }else if p.material.origin_authority.hasPrefix("engine-managed"){throw EngineError(message:"Managed intention has no original sender editor binding.")}
  for patch in p.patches {if let before=patch.before{guard let b=w.buffers[patch.path],!b.readOnly,!b.saving,!b.resourceDeleted,exactUTF8(b.text,before.text),let guardToken=capture.drafts[patch.path]?.save_guard ?? p.material.saved_guards[patch.path],exactUTF8(b.savedGuard,guardToken) else{throw EngineError(message:"Reviewed source or original loaded guard changed: "+patch.path)}}}
  for (path,guardToken) in capture.saved_guards{if let b=w.buffers[path]{guard exactUTF8(b.savedGuard,guardToken) else{throw EngineError(message:"Captured ownership baseline changed: "+path)}}}
 }
 func read(_ w:Workspace,_ entry:AgentProposalEntry)async {guard !busy,entries.contains(entry),let engine=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress,!w.resourceRestoration.hasPending else{return};let id=UUID(),epoch=w.sourceNavigationEpoch;ticket=id;busy=true;current={false};value=nil;staged=nil;defer{if ticket==id{busy=false}}
  do{let envelope:AgentProposalEnvelope=try await call(engine,"GET","/api/agent-proposals/"+entry.token);let input=AgentProposalParseInput(envelope:envelope,entry:entry,workspace:workspace,namespace:namespace);let p=try await Task.detached(priority:.userInitiated){try input.parse()}.value;guard ticket==id,w.sourceNavigationEpoch==epoch,w.engine?.session === engine.session else{throw EngineError(message:"Obsolete proposal reading.")};value=p;owner=w;server=engine
   let original=Array(w.buffers.values)+Array(w.retainedResourceEditors.values);let revisions=original.map{($0,$0.sourceEditIdentity)};let expected=Set(p.patches.filter{$0.before != nil}.map{NativeSourceKey($0.path)})
   for patch in p.patches where patch.before != nil && w.buffers[patch.path]==nil {_ = try await w.loadBuffer(FileNode(name:(patch.path as NSString).lastPathComponent,path:patch.path,dir:false));guard ticket==id,w.sourceNavigationEpoch==epoch,w.engine?.session === engine.session,revisions.allSatisfy({$0.0.sourceEditIdentity==$0.1}),w.buffers.values.allSatisfy({b in original.contains(where:{$0 === b})||expected.contains(NativeSourceKey(b.file.path))}) else{throw EngineError(message:"Host namespace changed while acquiring exact reviewed targets.")}}
   try matches(w,p);let held=captureRetainedSourceNamespace(w),senderRevisions=(Array(w.buffers.values)+Array(w.retainedResourceEditors.values)).map{($0,$0.agentSourceRevision)};current={ [weak self,weak w] in guard let self,let w else{return false};return self.ticket==id&&self.value === p&&held()&&senderRevisions.allSatisfy{$0.0.agentSourceRevision==$0.1}};phase="Reviewed only";notice="Complete SHA256 seal verified. Claims remain unverified; no source effects or checks occurred."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 private func validate(_ w:Workspace,_ p:SealedAgentProposal)async throws {guard value === p,current(),let engine=server,w.engine?.session === engine.session else{throw EngineError(message:"Complete intention or editor lease changed.")};try matches(w,p);let result:AgentProposalValidation=try await call(engine,"POST","/api/agent-proposals/validate",["version":1,"token":p.token,"fingerprint":p.fingerprint]);guard result.version==1,result.proposal_protocol==1,result.admitted,!result.source_effects,!result.checks_executed,exactUTF8(result.token,p.token),exactUTF8(result.fingerprint,p.fingerprint),exactUTF8(result.workspace,p.envelope.workspace),exactUTF8(result.namespace,p.envelope.namespace),value === p,current() else{throw EngineError(message:"Complete seal validation or current host authority changed.")};try matches(w,p)}
 func applyDraft(_ w:Workspace)async {guard canApply,let p=value,p.textOnly,staged==nil else{return};busy=true;defer{busy=false};do{try await validate(w,p);guard w.changes.mode==nil else{throw EngineError(message:"Finish the existing workspace preview first.")};let edit=try agentProposalBody(p.material.intention.capture.edit)
 let planned=try planWorkspaceEdit(p.material.intention.title,raw:edit,sources:w.sourceSnapshots())
 guard planned.patches.count==p.patches.count,planned.patches.allSatisfy({patch in p.patches.contains{exactUTF8($0.path,patch.path)&&exactUTF8($0.before?.text,patch.before)&&exactUTF8($0.after?.text,patch.after)}}) else{throw EngineError(message:"Original edit ranges do not reconstruct the complete sealed proposal.")}
 let before=Dictionary(uniqueKeysWithValues:planned.patches.map{($0.path,$0.before)})
 try w.changes.offer(p.material.intention.title,edit:edit,expected:before,workspace:w);consumed.insert(p.token);w.changes.apply(w);guard w.changes.plan==nil else{throw EngineError(message:w.changes.notice)};phase="Applied as one unsaved workspace batch";current={false};retainReport(p);await sendReport()}catch{notice=error.localizedDescription}}
 func prepareSave(_ w:Workspace)async {guard canApply,let p=value,staged==nil else{return};busy=true;defer{busy=false};do{try await validate(w,p);staged=try w.resourceRestoration.acceptSealed(w,proposal:p.proposal,facts:p.facts,current:current,preflight:{[weak self,weak w] in guard let self,let w else{throw EngineError(message:"Original proposal host closed.")};try await self.validate(w,p);self.consumed.insert(p.token)},published:{[weak self] recovery in self?.phase="Saved confirmed namespace · recovery "+recovery;self?.current={false};self?.retainReport(p)});notice="Review every saved and draft consequence above, then explicitly Save the entire namespace."}catch{notice=error.localizedDescription}}
 var canSave:Bool{!busy&&current()&&staged != nil&&owner?.resourceRestoration.review?.identity==staged}
 func save(_ w:Workspace)async {guard canSave else{return};busy=true;await w.resourceRestoration.publish(w);notice=w.resourceRestoration.notice;busy=false;if hasReport{await sendReport()}}
 func publicationStatus(_ w:Workspace)async {guard w === owner,w.resourceRestoration.hasPending else{return};await w.resourceRestoration.status(w);notice=w.resourceRestoration.notice;if hasReport{await sendReport()}}
 private func retainReport(_ p:SealedAgentProposal){reports[p.token]=["version":1,"token":p.token,"fingerprint":p.fingerprint,"workspace":p.envelope.workspace,"namespace":p.envelope.namespace,"request_id":UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased(),"host":"apple-simple","after_hashes":Dictionary(uniqueKeysWithValues:p.patches.map{($0.path,$0.after.map{nativeSourceHash($0.text)} ?? "missing")})]}
 func sendReport()async {guard let p=value,let body=reports[p.token],let engine=server else{return};do{let result:AgentProposalReport=try await call(engine,"POST","/api/agent-proposals/report",body)
 let expected=body["after_hashes"] as? [String:String] ?? [:]
 guard result.version==1,result.proposal_protocol==1,!result.saved,!result.checks_executed,exactUTF8(result.token,p.token),exactUTF8(result.fingerprint,p.fingerprint),exactUTF8(result.workspace,p.envelope.workspace),exactUTF8(result.namespace,p.envelope.namespace),exactUTF8(result.request_id,body["request_id"] as? String),exactUTF8(result.host,"apple-simple"),agentHex(result.report_fingerprint,64),result.after_hashes.count==expected.count,result.after_hashes.allSatisfy({key,text in expected.first{exactUTF8($0.key,key)}.map{exactUTF8($0.value,text)}==true}) else{throw EngineError(message:"Complete host report acknowledgment did not match the exact retained transition.")};reports[p.token]=nil;notice="Host draft transition acknowledged; this is not independent save or test evidence."}catch{notice=error.localizedDescription+" Drafts remain applied. Retry only this exact report, never Apply."}}
}
</ui:copy>


## Keep review beside code without permanent chrome

The main menu and action search open a workspace-owned floating reading. Closing it retires a still-unapplied lease. A consumed publication request and its recovery remain inspectable through the shared resource publisher; closing never turns an unknown result into permission to send again.

<ui:copy id="agent-proposals-apple-view">
struct AgentProposalView:View {
 @Bindable var workspace:Workspace
 @State private var section=0;@State private var page=0
 var body:some View {let model=workspace.agentProposals
  ScrollView {VStack(alignment:.leading,spacing:8){HStack{Text("Agent proposal review").bold();Spacer();Button("Close"){model.close();workspace.currentBuffer?.focusSource?()}}
   Text("Agent statements and requested checks are not observed verdicts.").font(.caption)
   Text(model.notice).font(.caption).textSelection(.enabled);Text(model.phase).font(.caption)
   Button("Refresh retained intentions"){Task{await model.open(workspace)}}.disabled(model.busy||workspace.resourceRestoration.hasPending)
   ForEach(model.entries){entry in Button("Read sealed proposal "+String(entry.token.prefix(12))){section=0;page=0;Task{await model.read(workspace,entry)}}.disabled(model.busy||workspace.resourceRestoration.hasPending)}
   if let value=model.value {Text(value.material.intention.title).bold();Text("Origin: "+value.material.origin_authority).font(.caption);Text("Complete SHA256 "+value.fingerprint).font(.caption2).textSelection(.enabled)
    Text(model.authorityCurrent ? "Current retained editor lease":"Historical editor/source lease · request a new intention before another application").font(.caption)
    ForEach(Array(value.sections.enumerated()),id:\.offset){index,item in Button(item.title){section=index;page=0}}
    let item=value.sections[min(section,value.sections.count-1)];Text(item.title).bold();Text(item.page(page)).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    HStack{Button("Previous material page"){page=max(0,page-1)}.disabled(page==0);Text("\(page+1)/\(item.pages.count)");Button("Next material page"){page=min(item.pages.count-1,page+1)}.disabled(page+1>=item.pages.count)}
    if value.textOnly {Button("Apply complete draft batch"){Task{await model.applyDraft(workspace)}}.disabled(!model.canApply)}
    Button("Prepare guarded namespace Save"){Task{await model.prepareSave(workspace)}}.disabled(!model.canApply)
    Button("Save reviewed namespace"){Task{await model.save(workspace)}}.disabled(!model.canSave)
    if workspace.resourceRestoration.hasPending{Button("Inspect original publication status"){Task{await model.publicationStatus(workspace)}}.disabled(model.busy)}
    if model.hasReport{Button("Retry exact host report · no Apply"){Task{await model.sendReport()}}.disabled(model.busy)}
   }
  }.padding()}.frame(idealWidth:700,idealHeight:620)
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/AgentProposals.swift"><ui:paste select="#agent-proposals-apple-seal"/><ui:paste select="#agent-proposals-apple-controller"/><ui:paste select="#agent-proposals-apple-view"/></ui:file>

## Refuse unauthenticated material before any readable claim is admitted

These focused cases target the admission boundary and sender lifecycle; they do not establish native renderer or real-engine application acceptance. The controlled fixture remains a separate executed-flow requirement.

<ui:copy id="agent-proposals-apple-tests">
import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
final class AgentProposalTests:XCTestCase {
 #if os(macOS)
 @MainActor func testLiveOwnedManagedProposalRetainsEditorDraftUndoAndNamespaceRecovery()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_PROPOSAL_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_NATIVE_PROPOSAL_WORKSPACE"] else{throw XCTSkip("Opt in with owned scripts/test-agent-proposals-browser.py --serve-only fixture.")}
  let directory=URL(fileURLWithPath:root),marker=try String(contentsOf:directory.appendingPathComponent(".hickory-native-agent-proposal-fixture"),encoding:.utf8);guard marker=="owned-native-agent-proposal-v1\n" else{throw EngineError(message:"An isolated owned proposal fixture is required.")}
  let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let tree=try await engine.files();guard let resolved=root.withCString({realpath($0,nil)}) else{throw EngineError(message:"Owned proposal fixture root is unavailable.")};defer{free(resolved)};guard exactUTF8(tree.root_path,String(cString:resolved)) else{throw EngineError(message:"Server does not own this isolated proposal fixture namespace.")};w.sourceRootAddress=tree.root_path;w.sourceWorkspaceID=tree.workspace_id;w.files=tree.tree;w.rootFiles=tree.tree
  let b=try await w.loadBuffer(FileNode(name:"note.py",path:"note.py",dir:false));w.layout.activateSource("note.py",in:"document-group");let source=b.text,generated=try Data(contentsOf:directory.appendingPathComponent("generated.py"));XCTAssertEqual(source,"value=1\n")
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
  defer{for buffer in w.buffers.values{buffer.language?.close()};window.orderOut(nil)}
  func settle(_ message:String,_ ready:()->Bool)async throws {for _ in 0..<400{if ready(){return};try await Task.sleep(for:.milliseconds(10))};throw EngineError(message:message)}
  func find(_ view:NSView)->NSTextView?{if let text=view as? NSTextView,exactUTF8(text.string,b.text){return text};return view.subviews.lazy.compactMap{find($0)}.first}
  try await settle("Native source editor did not mount"){host.layoutSubtreeIfNeeded();return find(host) != nil};let editor=try XCTUnwrap(find(host));editor.setSelectedRange(NSRange(location:2,length:0));b.selection=NSRange(location:2,length:0);let geometry=editor.frame,selection=editor.selectedRange()
  func prepare(_ edit:[String:Any])async throws->AgentProposalEntry {
   let intention:[String:Any]=["version":1,"request_id":UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased(),"origin":"owned controlled ACP fixture","title":"Explain and change implementation together","explanation":"The complete implementation and connected consequences are reviewed together.","claims":["Agent claim only; not an executed verdict"],"check_ids":[],"capture":["context":UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased(),"project_entry":"project.md","drafts":[:],"ownership_sources":[:],"unaddressed_sources":[:],"saved_guards":[:],"edit":edit],"reading":["version":1,"entry":"project.md","sources":[:],"missing_sources":[],"focus":b.file.path,"unaddressed_sources":[:],"saved_guards":[:]]]
   let parent=directory.deletingLastPathComponent(),calls=parent.appendingPathComponent("calls.json"),result=parent.appendingPathComponent("tool-result.json");try JSONSerialization.data(withJSONObject:["intention":intention]).write(to:calls,options:.atomic);if FileManager.default.fileExists(atPath:result.path){try FileManager.default.removeItem(at:result)}
   let _:ResourceJSON=try await engine.request("POST","/api/docs/workspace/agent/acp",body:["backend":"owned-proposal"])
   let _:ResourceJSON=try await engine.request("POST","/api/docs/workspace/agent",body:["backend":"owned-proposal","prompt":"Prepare the owned intention only; do not apply, save or run it.","context":try nativeAgentContext(w)])
   try await settle("Controlled ACP tool did not retain a complete proposal"){FileManager.default.fileExists(atPath:result.path)}
   let reply=try JSONSerialization.jsonObject(with:Data(contentsOf:result)) as? [String:Any],answer=reply?["result"] as? [String:Any],content=answer?["content"] as? [[String:Any]],text=try XCTUnwrap(content?.first?["text"] as? String),summary=try JSONSerialization.jsonObject(with:Data(text.utf8)) as? [String:Any];XCTAssertEqual(answer?["isError"] as? Bool,false);return AgentProposalEntry(token:try XCTUnwrap(summary?["token"] as? String),fingerprint:try XCTUnwrap(summary?["fingerprint"] as? String))
  }
  let edit:[String:Any]=["changes":[languageURI("note.py"):[["range":["start":["line":0,"character":6],"end":["line":0,"character":7]],"newText":"41"]]]]
  let entry=try await prepare(edit),m=w.agentProposals;try XCTUnwrap(w.commands().first{$0.id=="agent.proposals"}).run();try await settle("Proposal menu did not open"){!m.busy&&m.entries.contains(entry)};await m.read(w,entry);XCTAssertTrue(m.canApply,m.notice);XCTAssertTrue(m.value!.material.origin_authority.hasPrefix("engine-managed"));XCTAssertEqual(b.text,source);await m.applyDraft(w);XCTAssertEqual(b.text,"value=41\n",m.notice);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("note.py"),encoding:.utf8),source);XCTAssertTrue(find(host) === editor);XCTAssertEqual(editor.selectedRange(),selection);XCTAssertEqual(editor.frame,geometry);XCTAssertFalse(m.canApply);XCTAssertFalse(m.hasReport,m.notice)
  w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"value=41\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source)
  m.close();let moved="moved λ%2F?#.py",move:[String:Any]=["documentChanges":[["kind":"rename","oldUri":languageURI("note.py"),"newUri":languageURI(moved)]]],resource=try await prepare(move);await m.open(w);await m.read(w,resource);XCTAssertTrue(m.canApply,m.notice);await m.prepareSave(w);XCTAssertTrue(m.canSave,m.notice);await m.save(w);XCTAssertEqual(w.resourceRestoration.confirmedReceipt?.adoption_current,true,w.resourceRestoration.notice);XCTAssertTrue(w.buffers[moved] === b);XCTAssertTrue(find(host) === editor);XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent("note.py").path));XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(moved),encoding:.utf8),source);XCTAssertEqual(try Data(contentsOf:directory.appendingPathComponent("generated.py")),generated)
  await w.resourceRestoration.prepareInverse(w);XCTAssertNotNil(w.resourceRestoration.review,w.resourceRestoration.notice);await w.resourceRestoration.publish(w);XCTAssertTrue(w.buffers["note.py"] === b);XCTAssertTrue(find(host) === editor);XCTAssertEqual(b.text,source)
 }
 #endif

 @MainActor private func fixture(_ b:DocumentBuffer?=nil,managed:Bool=false)throws->(AgentProposalEntry,ResourceJSON) {
  let workspace=String(repeating:"b",count:32),before="value=1\n",after="value=41\n",guardToken="original-loaded",context="captured"
  func source(_ text:String)->[String:Any]{["kind":"file","identity":"source-note","text":text,"writable":true,"removable":true,"version":NSNull()]}
  func saved(_ text:String)->[String:Any]{["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":420]}
  let layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]],guards=["note.py":guardToken]
  let edit:[String:Any]=["changes":[languageURI("note.py"):[["range":["start":["line":0,"character":0],"end":["line":1,"character":0]],"newText":after]]]]
  let capture:[String:Any]=["context":context,"edit":edit,"project_entry":NSNull(),"drafts":["note.py":["text":before,"base_sha256":nativeSourceHash(before),"save_guard":guardToken,"writable":true]],"ownership_sources":["note.py":before],"unaddressed_sources":[:],"saved_guards":guards]
  let plan:[String:Any]=["version":1,"context":context,"title":"Complete change","annotations":[],"patches":[["path":"note.py","before":source(before),"after":source(after)]]]
  let inputs:[String:Any]=["version":1,"context":context,"workspace":workspace,"sources":["note.py":source(before)],"saved_sources":["note.py":saved(before)],"saved_guards":guards,"source_files_modified":false,"refusals":[:],"ownership":["mode":"none","saved_sources":[:],"draft_sources":[:],"protected_entries":[]]]
  let facts:[String:Any]=["version":1,"context":context,"workspace":workspace,"fingerprint":String(repeating:"f",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false,"saved_sources":["note.py":saved(after)]]
  let reading:[String:Any]=["version":1,"behavior_protocol":1,"namespace":"/fixture","engine_workspace_id":workspace]
  var intention:[String:Any]=["version":1,"request_id":String(repeating:"c",count:32),"title":"Complete change","explanation":"Implementation and explanation remain one intention.","claims":["Unverified agent claim"],"check_ids":[],"capture":capture]
  if let b{intention["editor_snapshot"]=["live":false,"buffers":[["id":b.id.uuidString,"path":"note.py","source_hash":nativeSourceHash(b.text),"save_guard":guardToken,"source_revision":b.agentSourceRevision.map(String.init) as Any? ?? NSNull()]]]}
  let material:[String:Any]=["version":1,"workspace":workspace,"namespace":"/fixture","origin_authority":managed ? "engine-managed agent tool; non-live submitted editor snapshot; claims remain unverified":"caller-provided intention; not an agent attestation","intention":intention,"inputs":inputs,"plan":plan,"consequences":facts,"before":reading,"after":reading,"saved_guards":guards]
  let text=String(decoding:try JSONSerialization.data(withJSONObject:material,options:[.sortedKeys]),as:UTF8.self),entry=AgentProposalEntry(token:String(repeating:"a",count:32),fingerprint:nativeSourceHash(text))
  let raw:[String:Any]=["version":1,"proposal_protocol":1,"token":entry.token,"fingerprint":entry.fingerprint,"workspace":workspace,"namespace":"/fixture","material":text,"source_effects":false,"checks_executed":false,"host_application":NSNull()]
  return(entry,try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:raw)))
 }
 @MainActor private func host()throws->(Workspace,DocumentBuffer){let w=Workspace();w.engine=try Engine(address:"http://fixture.invalid");w.sourceWorkspaceID=String(repeating:"b",count:32);w.sourceRootAddress="/fixture";let b=DocumentBuffer(file:FileNode(name:"note.py",path:"note.py",dir:false),text:FileText(path:"note.py",content:"value=1\n",hash:"baseline",save_guard:"original-loaded"));w.buffers["note.py"]=b;return(w,b)}
 @MainActor func testCompleteManagedReadRetiresAfterEqualTextUndoAndExtraEditor()async throws {
  let(w,b)=try host(),(entry,envelope)=try fixture(b,managed:true),m=w.agentProposals
  m.transport={method,path,body in if path=="/api/agent-proposals" {return .object(["version":.integer(1),"proposal_protocol":.integer(1),"workspace":.string(String(repeating:"b",count:32)),"namespace":.string("/fixture"),"reviews":.array([.object(["token":.string(entry.token),"fingerprint":.string(entry.fingerprint)])])])};return envelope}
  await m.open(w);await m.read(w,entry);XCTAssertTrue(m.canApply,m.notice);b.text="other";b.text="value=1\n";XCTAssertFalse(m.canApply);await m.read(w,entry);XCTAssertFalse(m.canApply);XCTAssertTrue(m.notice.contains("revision"),m.notice)
 }
 @MainActor func testSealedLeaseRetiresNewUnaddressedConstructionWithoutChangingCode()async throws {
  let(w,b)=try host(),(entry,envelope)=try fixture(),m=w.agentProposals
  m.transport={_,path,_ in path=="/api/agent-proposals" ? .object(["version":.integer(1),"proposal_protocol":.integer(1),"workspace":.string(String(repeating:"b",count:32)),"namespace":.string("/fixture"),"reviews":.array([.object(["token":.string(entry.token),"fingerprint":.string(entry.fingerprint)])])]):envelope}
  await m.open(w);await m.read(w,entry);XCTAssertTrue(m.canApply,m.notice)
  let added=DocumentBuffer(file:FileNode(name:"Untitled",path:"draft",dir:false),text:FileText(path:"draft",content:"<h:include path='other.md'/>",hash:""),isUntitledDraft:true);w.buffers["draft"]=added;XCTAssertFalse(m.canApply);await m.applyDraft(w);XCTAssertEqual(b.text,"value=1\n");XCTAssertEqual(added.text,"<h:include path='other.md'/>")
 }

 func testDigestFailurePrecedesInnerJSONAndExplanationAdmission()throws {
  let material="{not admitted explanation";let row=AgentProposalEntry(token:String(repeating:"a",count:32),fingerprint:String(repeating:"b",count:64))
  let envelope=AgentProposalEnvelope(version:1,proposal_protocol:1,token:row.token,fingerprint:row.fingerprint,workspace:"workspace",namespace:"namespace",material:material,source_effects:false,checks_executed:false,host_application:nil)
  XCTAssertThrowsError(try SealedAgentProposal(envelope,expected:row,workspace:"workspace",namespace:"namespace")){XCTAssertTrue($0.localizedDescription.contains("SHA256"))}
 }
 func testForeignNamespaceCannotAdmitEvenCorrectDigest()throws {
  let material="{}",row=AgentProposalEntry(token:String(repeating:"a",count:32),fingerprint:nativeSourceHash(material))
  let envelope=AgentProposalEnvelope(version:1,proposal_protocol:1,token:row.token,fingerprint:row.fingerprint,workspace:"workspace",namespace:"é",material:material,source_effects:false,checks_executed:false,host_application:nil)
  XCTAssertThrowsError(try SealedAgentProposal(envelope,expected:row,workspace:"workspace",namespace:"e\u{301}"))
 }
 func testOuterWireRefusesDeepUnknownStructureAndProtocolAliases()throws {let nested=String(repeating:"[",count:65)+"0"+String(repeating:"]",count:65);XCTAssertThrowsError(try validateNativeAgentProposalResponse(Data(nested.utf8)));for token in ["1.0","true","1e0"]{XCTAssertThrowsError(try validateNativeAgentProposalResponse(Data(("{\"version\":"+token+"}").utf8)))};XCTAssertNoThrow(try validateNativeAgentProposalResponse(Data("{\"version\":1,\"proposal_protocol\":1,\"material\":\"literal { [\"}".utf8)))}
 func testPagesNeverSplitUnicodeScalar()throws {let text=String(repeating:"🙂",count:4001),section=try AgentProposalSection("Literal",text);XCTAssertEqual(section.pages.count,2);XCTAssertEqual(section.page(0).unicodeScalars.count,4000);XCTAssertEqual(section.page(1),"🙂")}
 @MainActor func testSenderRevisionRetiresEqualTextUndoAndCloseContext()throws {
  let b=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value=1\n",hash:""));let original=b.agentSourceRevision;b.text="value=2\n";b.text="value=1\n";XCTAssertNotEqual(b.agentSourceRevision,original);let equal=b.agentSourceRevision;b.retireAgentContext();XCTAssertNotEqual(equal,b.agentSourceRevision)
 }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/AgentProposalTests.swift"><ui:paste select="#agent-proposals-apple-tests"/></ui:file>

</ui:doc>
