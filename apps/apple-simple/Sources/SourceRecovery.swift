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
// Only annotations and provider edits are structurally open JSON. State,
// baselines and publication admission use the typed values below.
indirect enum ResourceJSON:Codable {
 case object([String:ResourceJSON]),array([ResourceJSON]),string(String),integer(Int64),unsigned(UInt64),number(Double),bool(Bool),null
 init(from decoder:Decoder)throws {let value=try decoder.singleValueContainer();if value.decodeNil(){self = .null}else if let item=try? value.decode(Bool.self){self = .bool(item)}else if let item=try? value.decode(String.self){self = .string(item)}else if let item=try? value.decode(Int64.self){self = .integer(item)}else if let item=try? value.decode(UInt64.self){self = .unsigned(item)}else if let item=try? value.decode(Double.self){self = .number(item)}else if let item=try? value.decode([String:ResourceJSON].self){self = .object(item)}else{self = .array(try value.decode([ResourceJSON].self))}}
 func encode(to encoder:Encoder)throws {var value=encoder.singleValueContainer();switch self {case .object(let item):try value.encode(item);case .array(let item):try value.encode(item);case .string(let item):try value.encode(item);case .integer(let item):try value.encode(item);case .unsigned(let item):try value.encode(item);case .number(let item):try value.encode(item);case .bool(let item):try value.encode(item);case .null:try value.encodeNil()}}
}
private func resourceBody<T:Encodable>(_ value:T)throws->[String:Any] {guard let object=try JSONSerialization.jsonObject(with:JSONEncoder().encode(value)) as? [String:Any] else{throw EngineError(message:"Resource request must be a typed object.")};return object}
func resourceRecoveryIdentity(_ path:String)throws->String {
 let parts=path.split(separator:"/",omittingEmptySubsequences:false)
 guard parts.count==4,parts[0]==".hick-cache",parts[1]=="resource-changes",parts[3]=="manifest.json",parts[2].range(of:"^[0-9a-f]{32}$",options:.regularExpression) != nil else{throw EngineError(message:"Saved publication returned an invalid recovery record address. Retain its request identity and inspect Source recovery before adopting editors.")}
 return String(parts[2])
}
struct ResourceWireSource:Codable {
 var kind:String;var identity:String?;var text:String?;var writable:Bool?;var removable:Bool?;var version:Int?;var creatable:Bool?
 func source()throws->ResourceSource? {
  if kind=="missing",creatable != nil{return nil}
  guard kind=="file",let identity,!identity.isEmpty,let text,writable != nil,removable != nil else{throw EngineError(message:"Resource source state is incomplete. Prepare the complete proposal again.")}
  return ResourceSource(identity:identity,text:text)
 }
}
struct ResourceWirePatch:Codable {var path:String;var before:ResourceWireSource;var after:ResourceWireSource}
struct ResourceWirePlan:Codable {var version:Int;var context:String;var title:String;var patches:[ResourceWirePatch];var annotations:[ResourceJSON]
 func editors()throws->[ResourceEditorPatch] {
  guard version==1,!context.isEmpty,!patches.isEmpty,patches.count<=1000 else{throw EngineError(message:"Resource plan identity or size is invalid.")}
  guard Set(patches.map{$0.path}).count==patches.count else{throw EngineError(message:"The native wire maps cannot distinguish duplicate or canonically equivalent path spellings. No resource change was published; use a source scope with distinct names.")}
  return try patches.map{ResourceEditorPatch(path:$0.path,before:try $0.before.source(),after:try $0.after.source())}
 }
}
struct ResourceWireOwnership:Codable {var mode:String;var entry:String?;var protected_entries:[String];var saved_sources:[String:String];var draft_sources:[String:String]}
struct ResourceWireInputs:Codable {var version:Int;var context:String;var workspace:String;var ownership:ResourceWireOwnership;var sources:[String:ResourceWireSource];var saved_sources:[String:ResourceSaved];var refusals:[String:String];var source_files_modified:Bool;var saved_guards:[String:String]?=nil}
struct ResourceWireDraft:Codable {var text:String;var base_sha256:String;var version:Int?;var writable:Bool;var save_guard:String?=nil}
struct ResourceWireCapture:Codable {var context:String;var edit:ResourceJSON;var drafts:[String:ResourceWireDraft];var project_entry:String?;var ownership_sources:[String:String];var unaddressed_sources:[String:String]=[:];var saved_guards:[String:String]=[:]}
struct ResourceWireLayer:Decodable {var entries:[String];var removed_entries:[String];var source_hashes:[String:String];var output_paths:[String]}
struct ResourceWireFacts:Decodable {var version:Int;var workspace:String;var context:String;var fingerprint:String;var saved:ResourceWireLayer;var draft:ResourceWireLayer;var protected_targets:[String];var source_files_modified:Bool;var saved_sources:[String:ResourceSaved]?}
struct ResourceWireTargets:Decodable {var version:Int;var paths:[String]}
struct ResourceWirePlanned:Decodable {var plan:ResourceWirePlan}
struct ResourceWireProposal:Codable {var inputs:ResourceWireInputs;var capture:ResourceWireCapture;var plan:ResourceWirePlan}
@MainActor func resourceOwnershipSnapshot(_ workspace:Workspace)throws->(named:[String:String],unaddressed:[String:String],guards:[String:String]) {
 let all=Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values),unique=Dictionary(grouping:all,by:{ObjectIdentifier($0)}).values.compactMap{$0.first};var named:[String:String]=[:],unaddressed:[String:String]=[:],guards:[String:String]=[:]
 guard unique.count<=4096 else{throw EngineError(message:"Retained source roster exceeds the bounded resource review. Preserve sources and choose a smaller workspace scope.")};var cost=0
 for buffer in unique where buffer.file.generated_by==nil {
  let bytes=buffer.text.utf8.count;guard bytes<=8*1024*1024,bytes<=16*1024*1024-cost else{throw EngineError(message:"Retained sources exceed the bounded resource review text capacity. Preserve them and choose a smaller scope.")};cost+=bytes
  if buffer.resourceDeleted||buffer.isUntitledDraft||buffer.file.path.hasPrefix("/"){unaddressed[buffer.id.uuidString]=buffer.text;continue}
  guard named[buffer.file.path]==nil else{throw EngineError(message:"Retained source names have ambiguous Unicode spellings; no source ownership was merged.")};named[buffer.file.path]=buffer.text;if let guardToken=buffer.savedGuard{guards[buffer.file.path]=guardToken}
 }
 return(named,unaddressed,guards)
}
@MainActor func captureRetainedSourceNamespace(_ workspace:Workspace)->()->Bool {
 let epoch=workspace.sourceNavigationEpoch,engine=workspace.engine,all=Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values),unique=Dictionary(grouping:all,by:{ObjectIdentifier($0)}).values.compactMap{$0.first}
 let snapshots=unique.map{($0,ObjectIdentifier($0),$0.sourceEditIdentity,ExactSourceText($0.file.path),$0.sourceRevision,ExactSourceText($0.base),$0.savedGuard.map{ExactSourceText($0)},$0.readOnly,$0.resourceDeleted,$0.file.generated_by.map{ExactSourceText($0)})}
 return{[weak workspace] in guard let workspace,workspace.sourceNavigationEpoch==epoch,workspace.engine?.origin==engine?.origin,workspace.engine?.session === engine?.session else{return false};let now=Set((Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values)).map{ObjectIdentifier($0)});guard now==Set(snapshots.map{$0.1}) else{return false};return snapshots.allSatisfy{buffer,_,revision,path,text,base,guardToken,readonly,deleted,generated in buffer.sourceEditIdentity==revision&&ExactSourceText(buffer.file.path)==path&&buffer.sourceRevision==text&&ExactSourceText(buffer.base)==base&&buffer.savedGuard.map{ExactSourceText($0)}==guardToken&&buffer.readOnly==readonly&&buffer.resourceDeleted==deleted&&buffer.file.generated_by.map{ExactSourceText($0)}==generated && !buffer.saving}}
}
func isResourceWorkspaceEdit(_ raw:Any)->Bool {guard let value=raw as? [String:Any],let operations=value["documentChanges"] as? [[String:Any]] else{return false};return operations.contains{$0["kind"] != nil}}
struct ResourceWireRestoreRequest:Codable {var recovery_id:String;var inputs:ResourceWireInputs;var capture:ResourceWireCapture}
struct ResourceWireRestoration:Decodable {var version:Int;var recovery_id:String;var recovery_fingerprint:String;var workspace:String;var context:String;var plan:ResourceWirePlan;var saved_before:[String:ResourceSaved];var saved_after:[String:ResourceSaved]}
struct ResourceWirePrepared:Decodable {var version:Int;var restoration:ResourceWireRestoration;var consequences:ResourceWireFacts}
struct ResourceRestorationReview {
 let identity=UUID();var preflight:(()async throws->Void)?=nil;var published:((String)->Void)?=nil
 var request:ResourceWireRestoreRequest;var result:ResourceWirePrepared;var patches:[ResourceEditorPatch];var retained:[String:DocumentBuffer];var current:()->Bool;var save:ResourceWireProposal?=nil;var history:ResourceHistoryIntent?=nil
}

// A pending write outlives the popover. Never replay it automatically: status is
// read-only, and local admission is repeated before adopting a confirmed result.
@MainActor @Observable final class ResourceRestoration {
 private(set) var confirmedReceipt:ResourcePublication<FileNode>?
 var review:ResourceRestorationReview?;var notice="";var busy=false;var lastRecovery:String?;var selectedPath=0
 private var admitting=false
 private var ticket=UUID();private var pending:(id:String,review:ResourceRestorationReview)?;private weak var publishingWorkspace:Workspace?
 var hasPending:Bool {pending != nil}
 var pendingRequest:String? {pending?.id}
 var transport:((String,String,[String:Any]?) async throws->ResourceJSON)?
 private func call<T:Decodable>(_ workspace:Workspace,_ method:String,_ route:String,_ body:[String:Any]?=nil)async throws->T {
  if let transport {return try JSONDecoder().decode(T.self,from:JSONEncoder().encode(try await transport(method,route,body)))}
  guard let engine=workspace.engine else{throw EngineError(message:"Connect to a workspace server first.")}
  let address=URLComponents(string:route)!
  let value:ResourceJSON=try await engine.request(method,address.path,query:address.queryItems ?? [],body:body)
  return try JSONDecoder().decode(T.self,from:JSONEncoder().encode(value))
 }
 func cancel(){guard !hasPending else{return};ticket=UUID();review=nil;busy=false;notice=""}
 private func currentWorkspace(_ workspace:Workspace)->()->Bool {captureRetainedSourceNamespace(workspace)}
 func prepare(_ workspace:Workspace,record:RecoveryReading,history:ResourceHistoryIntent?=nil)async {
  guard !busy,!hasPending else{notice="Inspect the pending publication before preparing another restoration.";return}
  let id=UUID();ticket=id;busy=true;review=nil;notice="Acquiring current sources and construction ownership…";defer{if ticket==id{busy=false}}
  let initial=workspace.sourceNavigationEpoch
  let historyCurrent:()->Bool={history.map{workspace.changes.historySnapshot==$0.token && workspace.changes.resourceHistoryRecord($0.direction)==$0.record} ?? true}
  do {
   guard historyCurrent(),record.paths.count>0,record.paths.count<=1000 else{throw EngineError(message:"Choose a bounded recovery record.")}
   func node(_ path:String,_ nodes:[FileNode])->FileNode? {for item in nodes {if exactUTF8(item.path,path){return item};if let child=node(path,item.children ?? []){return child}};return nil}
   for item in record.paths where item.saved_after.kind=="file" && workspace.buffers[item.path]==nil {
    let file=node(item.path,workspace.rootFiles) ?? FileNode(name:(item.path as NSString).lastPathComponent,path:item.path,dir:false)
    _ = try await workspace.loadBuffer(file)
    guard ticket==id,workspace.sourceNavigationEpoch==initial else{throw EngineError(message:"Workspace changed while acquiring recovery sources.")}
   }
   let captured=currentWorkspace(workspace),context=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased()
   var edits:[String:ResourceJSON]=[:],drafts:[String:ResourceWireDraft]=[:]
   for item in record.paths {
    edits[languageURI(item.path)] = .array([])
    if let buffer=workspace.buffers[item.path] {drafts[item.path]=ResourceWireDraft(text:buffer.text,base_sha256:nativeSourceHash(buffer.base),version:nil,writable:!buffer.readOnly && !buffer.saving,save_guard:buffer.savedGuard)}
   }
   let ownership=try resourceOwnershipSnapshot(workspace)
   let capture=ResourceWireCapture(context:context,edit:.object(["changes":.object(edits)]),drafts:drafts,project_entry:nil,ownership_sources:ownership.named,unaddressed_sources:ownership.unaddressed,saved_guards:ownership.guards)
   let inputs:ResourceWireInputs=try await call(workspace,"POST","/api/changes/resource-inputs",resourceBody(capture))
   guard ticket==id,captured(),inputs.version==1,inputs.saved_guards != nil,inputs.context==context,inputs.workspace==record.workspace,inputs.refusals.isEmpty else{throw EngineError(message:"Current source, ownership or workspace changed. Refresh and prepare restoration again.")}
   let request=ResourceWireRestoreRequest(recovery_id:record.id,inputs:inputs,capture:capture)
   let result:ResourceWirePrepared=try await call(workspace,"POST","/api/changes/prepare-restoration",resourceBody(request))
   guard ticket==id,captured(),historyCurrent(),result.version==1,result.restoration.version==1,result.restoration.recovery_id==record.id,result.restoration.recovery_fingerprint==record.fingerprint,result.restoration.workspace==record.workspace,result.restoration.context==context,result.restoration.plan.context==context,result.consequences.version==1,result.consequences.context==context,result.consequences.workspace==record.workspace,result.consequences.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,result.consequences.protected_targets.isEmpty else{throw EngineError(message:"Recovery evidence, construction consequences or current source changed. Prepare again.")}
   let patches=try result.restoration.plan.editors()
   guard Set(patches.map{$0.path})==Set(record.paths.map{$0.path}),patches.allSatisfy({patch in patch.before.map{exactUTF8(workspace.buffers[patch.path]?.text,$0.text)} ?? (workspace.buffers[patch.path]==nil)}) else{throw EngineError(message:"Every current source must match the complete restoration plan.")}
   var retained:[String:DocumentBuffer]=[:]
   for patch in patches {if let after=patch.after,let buffer=workspace.retainedResourceEditors[after.identity] {retained[after.identity]=buffer}}
   let local=workspace.captureResourceEditors(patches,restoring:retained)
   review=ResourceRestorationReview(request:request,result:result,patches:patches,retained:retained,current:{captured() && local() && historyCurrent()},history:history);selectedPath=0
   notice="Review saved baselines and dirty drafts separately. Publication changes saved source; generated files are only analyzed."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 func prepareEdit(_ workspace:Workspace,title:String,edit:Any,current:@escaping()->Bool={true},requery:(() async throws->Any)?=nil)async {
  guard !busy,!hasPending else{notice="Inspect the pending publication before preparing another resource proposal.";return}
  let id=UUID();ticket=id;busy=true;review=nil;notice="Acquiring the complete resource proposal…";defer{if ticket==id{busy=false}}
  do {
   var selectedEdit=edit
   var raw=try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:edit))
   let discovered:ResourceWireTargets=try await call(workspace,"POST","/api/changes/resource-targets",["edit":edit])
   guard ticket==id,current(),discovered.version==1,!discovered.paths.isEmpty,discovered.paths.count<=1000,Set(discovered.paths).count==discovered.paths.count else{throw EngineError(message:"Resource targets or provider changed. Prepare the complete proposal again.")}
   let acquisition=try await acquireResourceInputs(workspace,paths:discovered.paths,edit:raw,current:{self.ticket==id&&current()},readInputs:{capture in try await self.call(workspace,"POST","/api/changes/resource-inputs",resourceBody(capture))})
   var sourceCapture=acquisition.capture,inputs=acquisition.inputs;let captured=acquisition.current,context=sourceCapture.context
   if acquisition.acquired,let requery {
    for path in discovered.paths {if let buffer=workspace.buffers[path],let service=buffer.language {let source=buffer.text;_ = try await service.readOutline(source:source);guard exactUTF8(buffer.text,source),ticket==id,current(),captured() else{throw EngineError(message:"Original source changed while synchronizing the resource proposal.")}}}
    selectedEdit=try await requery()
    let rediscovered:ResourceWireTargets=try await call(workspace,"POST","/api/changes/resource-targets",["edit":selectedEdit])
    guard rediscovered.version==1,Set(rediscovered.paths.map{Data($0.utf8)})==Set(discovered.paths.map{Data($0.utf8)}),ticket==id,current(),captured() else{throw EngineError(message:"Provider resource targets or original source owners changed after acquisition. Prepare again.")}
    raw=try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:selectedEdit));sourceCapture.edit=raw
    inputs=try await call(workspace,"POST","/api/changes/resource-inputs",resourceBody(sourceCapture))
   }
   guard ticket==id,current(),captured(),inputs.version==1,inputs.saved_guards != nil,inputs.context==context,inputs.refusals.isEmpty,Set(inputs.sources.keys)==Set(discovered.paths) else{throw EngineError(message:"Complete resource sources changed. Prepare again.")}
   let planned:ResourceWirePlanned=try await call(workspace,"POST","/api/changes/plan-resources",["context":context,"title":title,"edit":selectedEdit,"sources":try resourceBody(inputs)["sources"]!])
   let patches=try planned.plan.editors()
   let proposal=ResourceWireProposal(inputs:inputs,capture:sourceCapture,plan:planned.plan)
   let facts:ResourceWireFacts=try await call(workspace,"POST","/api/changes/resource-consequences",resourceBody(proposal))
   guard ticket==id,current(),captured(),planned.plan.context==context,facts.version==1,facts.context==context,facts.workspace==inputs.workspace,facts.protected_targets.isEmpty,facts.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,let saved=facts.saved_sources,Set(saved.keys)==Set(patches.map{$0.path}) else{throw EngineError(message:"Resource consequences changed or complete saved baselines are unavailable. Prepare again.")}
   guard patches.allSatisfy({patch in patch.before.map{exactUTF8(workspace.buffers[patch.path]?.text,$0.text)} ?? (workspace.buffers[patch.path]==nil)}) else{throw EngineError(message:"Every current source must match the complete resource plan.")}
   let local=workspace.captureResourceEditors(patches)
   let restoration=ResourceWireRestoration(version:1,recovery_id:"",recovery_fingerprint:"",workspace:inputs.workspace,context:context,plan:planned.plan,saved_before:inputs.saved_sources,saved_after:saved)
   review=ResourceRestorationReview(request:ResourceWireRestoreRequest(recovery_id:"",inputs:inputs,capture:sourceCapture),result:ResourceWirePrepared(version:1,restoration:restoration,consequences:facts),patches:patches,retained:[:],current:{captured() && local() && current()},save:proposal);selectedPath=0
   notice="Review every text and namespace consequence together. Publish explicitly after reviewing saved source and generated consequences."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 func prepareHistory(_ workspace:Workspace,intent:ResourceHistoryIntent)async {
  guard !busy,!hasPending,workspace.changes.historySnapshot==intent.token,workspace.changes.resourceHistoryRecord(intent.direction)==intent.record else{notice="Workspace history changed. Prepare its current undo/redo entry.";return}
  let id=UUID(),epoch=workspace.sourceNavigationEpoch;ticket=id;busy=true
  do {
   let record:RecoveryReading=try await call(workspace,"GET","/api/changes/resource-recovery?id="+recoveryQuery(intent.record))
   guard ticket==id,workspace.sourceNavigationEpoch==epoch,record.version==1,record.id==intent.record,workspace.changes.historySnapshot==intent.token,workspace.changes.resourceHistoryRecord(intent.direction)==intent.record else{throw EngineError(message:"Workspace history or recovery changed. Prepare again.")}
   busy=false;await prepare(workspace,record:record,history:intent)
  }catch{if ticket==id{busy=false;notice=error.localizedDescription}}
 }
 func prepareInverse(_ workspace:Workspace)async {
  guard let record=lastRecovery,!busy,!hasPending else{return}
  let id=UUID(),epoch=workspace.sourceNavigationEpoch;ticket=id;busy=true
  do {let value:RecoveryReading=try await call(workspace,"GET","/api/changes/resource-recovery?id="+recoveryQuery(record));guard ticket==id,workspace.sourceNavigationEpoch==epoch,value.version==1,value.id==record,value.fingerprint.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil else{throw EngineError(message:"Recovery identity or workspace changed.")};busy=false;await prepare(workspace,record:value)}catch{if ticket==id{busy=false;notice=error.localizedDescription}}
 }
 func acceptSealed(_ workspace:Workspace,proposal:ResourceWireProposal,facts:ResourceWireFacts,current:@escaping()->Bool,preflight:@escaping()async throws->Void,published:@escaping(String)->Void)throws->UUID {
  guard !busy,!hasPending,review==nil,current(),let saved=facts.saved_sources else{throw EngineError(message:"Finish the existing resource review before staging the complete intention.")}
  let patches=try proposal.plan.editors();let local=workspace.captureResourceEditors(patches)
  guard facts.context==proposal.plan.context,facts.workspace==proposal.inputs.workspace,Set(saved.keys.map(NativeSourceKey.init))==Set(patches.map{NativeSourceKey($0.path)}),patches.allSatisfy({p in p.before.map{exactUTF8(workspace.buffers[p.path]?.text,$0.text)} ?? (workspace.buffers[p.path]==nil)}) else{throw EngineError(message:"Complete sealed resource sources are not current.")}
  let restoration=ResourceWireRestoration(version:1,recovery_id:"",recovery_fingerprint:"",workspace:proposal.inputs.workspace,context:proposal.plan.context,plan:proposal.plan,saved_before:proposal.inputs.saved_sources,saved_after:saved)
  var next=ResourceRestorationReview(request:ResourceWireRestoreRequest(recovery_id:"",inputs:proposal.inputs,capture:proposal.capture),result:ResourceWirePrepared(version:1,restoration:restoration,consequences:facts),patches:patches,retained:[:],current:{current()&&local()},save:proposal)
  next.preflight=preflight;next.published=published;review=next;return next.identity
 }
 func publish(_ workspace:Workspace)async {
  guard !busy,!admitting,pending==nil,let value=review else{return}
  guard value.current() else{review=nil;notice="Source changed after review. Prepare restoration again.";return}
  admitting=true;defer{admitting=false}
  do{try await value.preflight?()}catch{notice=error.localizedDescription;return}
  guard review?.identity==value.identity,value.current(),pending==nil,!busy else{notice="Complete review changed during final admission.";return}
  let id=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased();pending=(id,value);confirmedReceipt=nil;publishingWorkspace=workspace;review=nil;busy=true
  defer{busy=false}
  do {
   var body:[String:Any]=["request_id":id,"reviewed_fingerprint":value.result.consequences.fingerprint]
   if let save=value.save {body["proposal"]=try resourceBody(save)}
   else {body["proposal"]=try resourceBody(value.request);body["reviewed_recovery_fingerprint"]=value.result.restoration.recovery_fingerprint}
   let receipt:ResourcePublication<FileNode>=try await call(workspace,"POST",value.save==nil ? "/api/changes/restore-resources":"/api/changes/resource-save",body)
   try adopt(workspace,id:id,value:value,receipt:receipt)
  }catch{notice=error.localizedDescription+(confirmedReceipt?.request_id==id ? " Saved publication is confirmed; editor adoption is incomplete. Preserve local drafts and inspect Source recovery. This action will not be resent.":" Publication is unconfirmed. Inspect status or retained recovery; this action will not be retried.")}
 }
 func status(_ workspace:Workspace)async {
  guard !busy,let (id,value)=pending else{return};busy=true;defer{busy=false}
  do {guard publishingWorkspace === workspace else{throw EngineError(message:"Inspect recovery in the original workspace.")};let receipt:ResourcePublication<FileNode>=try await call(workspace,"GET","/api/changes/resource-save?request_id="+id);try adopt(workspace,id:id,value:value,receipt:receipt)}catch{notice=error.localizedDescription+(confirmedReceipt?.request_id==id ? " Saved publication is confirmed; preserve drafts and inspect recovery because editor adoption is incomplete.":" Preserve drafts and inspect retained recovery before another publication.")}
 }
 private func adopt(_ workspace:Workspace,id:String,value:ResourceRestorationReview,receipt:ResourcePublication<FileNode>)throws {
  guard publishingWorkspace === workspace else{throw EngineError(message:"Workspace changed during publication. Inspect recovery in its original workspace.")}
  _ = try confirmedResourceFiles(value.patches,receipt:receipt,request:id,workspace:value.result.restoration.workspace,context:value.request.capture.context,expected:value.result.restoration.saved_after){($0.path,!$0.dir && $0.generated_by==nil)}
  confirmedReceipt=receipt
  let recovery=try resourceRecoveryIdentity(receipt.outcome.recovery)
  _ = try workspace.adoptResourceReceipt(value.patches,receipt:receipt,request:id,workspace:value.result.restoration.workspace,context:value.request.capture.context,expected:value.result.restoration.saved_after,current:value.current,restoring:value.retained)
  workspace.changes.recordResourcePublication(recovery,intent:value.history)
  pending=nil;lastRecovery=recovery;value.published?(recovery);notice=value.save==nil ? "Restored confirmed saved baselines and retained dirty drafts. Prepare the inverse restoration to redo this operation.":"Saved the reviewed resource namespace. Use Undo workspace change to review its inverse. No tests were executed by this save."
  Task {await workspace.refresh()}
 }
}
struct SourceRecoveryView:View {
 var workspace:Workspace;@State private var model=RecoveryReader();@State private var path="";@State private var snapshot="draft_before"
 private func list(_ after:String?=nil)async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};workspace.resourceRestoration.cancel();path="";await model.list(after:after){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 private func read(_ id:String,_ path:String?=nil,_ snapshot:String="draft_before")async {guard let engine=workspace.engine else{return};workspace.resourceRestoration.cancel();await model.read(id,path:path,snapshot:snapshot){let route=URLComponents(string:$0)!;return try await engine.request("GET",route.path,query:route.queryItems ?? [])}}
 var body:some View {VStack(alignment:.leading,spacing:8){
  HStack{Text("Source recovery").font(.headline);Spacer();Button("Editor sessions…"){workspace.showRecovery=false;workspace.editorRecovery.showing=true}.disabled(workspace.resourceRestoration.busy || workspace.resourceRestoration.hasPending);Button("Close"){workspace.showRecovery=false;workspace.currentBuffer?.focusSource?()}}
  Button("Refresh records"){Task{await list()}}.disabled(model.loading);Text(model.notice).font(.caption).textSelection(.enabled)
  ResourceRestorationPreview(workspace:workspace,model:workspace.resourceRestoration)
  ScrollView{VStack(alignment:.leading,spacing:8){
   if let page=model.page {Picker("Retained record",selection:Binding(get:{model.value?.id ?? ""},set:{id in if !id.isEmpty{path="";Task{await read(id)}}})){Text("Choose record").tag("");ForEach(page.ids,id:\.self){Text($0).tag($0)}}.disabled(model.loading)
    if page.ignored>0 {Text("\(page.ignored) unrecognized entries omitted.").font(.caption)}
    if let next=page.next {Button("Next 100 identities"){Task{await list(next)}}.disabled(model.loading)}
   }
   if let value=model.value {
    Text(value.title+(value.title_truncated ? "…":"")).font(.headline)
    Button("Prepare restoration"){Task{await workspace.resourceRestoration.prepare(workspace,record:value)}}.disabled(model.loading || workspace.resourceRestoration.busy || workspace.resourceRestoration.hasPending)
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
 }.padding().frame(idealWidth:680,idealHeight:600).task{await list()}.onDisappear{model.close();workspace.resourceRestoration.cancel()}}
}

struct ResourceRestorationPreview:View {
 var workspace:Workspace;var model:ResourceRestoration
 @ViewBuilder private func saved(_ name:String,_ value:ResourceSaved?) -> some View {
  Text(name).font(.headline)
  if let value {Text(value.kind=="missing" ? "Source absent.":"Saved file · mode \(value.mode.map(String.init) ?? "provider default") · \(value.readonly==true ? "read-only":"writable")").font(.caption)
   if let text=value.text {Text(String(text.prefix(32000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled);if text.count>32000{Text("First 32,000 characters shown.").font(.caption)}}
  }else{Text("Saved baseline missing from review. Publication will refuse.").font(.caption)}
 }
 var body:some View {VStack(alignment:.leading,spacing:8){
  if !model.notice.isEmpty {Text(model.notice).font(.caption).textSelection(.enabled)}
  if let request=model.pendingRequest {Text("Resource request: "+request).font(.caption).textSelection(.enabled);Text(model.confirmedReceipt==nil ? "Publication result unknown. Preserve local editors until status or recovery confirms it.":"Saved publication confirmed; editor adoption is incomplete.").font(.caption);Button("Inspect publication status"){Task{await model.status(workspace)}}.disabled(model.busy)}
  if model.lastRecovery != nil {Button("Prepare inverse restoration (undo/redo)"){Task{await model.prepareInverse(workspace)}}.disabled(model.busy || model.hasPending)}
  if let review=model.review {
   Text("Review source change: \(review.result.restoration.plan.title)").font(.headline)
   Picker("Restored source",selection:Binding(get:{model.selectedPath},set:{model.selectedPath=$0})){ForEach(Array(review.patches.enumerated()),id:\.offset){index,patch in Text(patch.path).tag(index)}}
   ScrollView([.horizontal,.vertical]){VStack(alignment:.leading,spacing:8){
    if review.patches.indices.contains(model.selectedPath){let patch=review.patches[model.selectedPath]
     saved("Saved before",review.result.restoration.saved_before[patch.path]);saved("Saved after",review.result.restoration.saved_after[patch.path])
     Text("Draft before").font(.headline);Text(patch.before.map{String($0.text.prefix(32000))} ?? "Source absent.").font(.system(.caption,design:.monospaced)).textSelection(.enabled)
     Text("Draft after").font(.headline);Text(patch.after.map{String($0.text.prefix(32000))} ?? "Source absent.").font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    }
    Text("Saved construction outputs: "+review.result.consequences.saved.output_paths.joined(separator:", ")).font(.caption).textSelection(.enabled)
    Text("Draft construction outputs: "+review.result.consequences.draft.output_paths.joined(separator:", ")).font(.caption).textSelection(.enabled)
    Text("Construction consequences are observed relationships; no tests or coverage were executed by this preview.").font(.caption)
   }.frame(maxWidth:.infinity,alignment:.leading)}.frame(maxHeight:240)
   HStack{Button(review.save==nil ? "Publish reviewed restoration":"Save reviewed resource change"){Task{await model.publish(workspace)}}.disabled(model.busy || model.hasPending);Button("Discard preview"){model.cancel()}.disabled(model.busy)}
  }
 }}
}
