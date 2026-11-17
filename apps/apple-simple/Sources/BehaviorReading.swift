import SwiftUI
import Observation

// Typed envelopes retain capability and provenance words instead of turning declarations into verdicts.
struct ConnectedBehaviorCheck:Decodable,Identifiable {var id:String;var label:String;var path:String;var available:Bool;var reason:String}
struct ConnectedBehaviorNode:Decodable,Identifiable {var id:String;var kind:String;var path:String?;var source_path:String?;var fragment_id:String?;var label:String;var hash:String?;var preview:String?;var status:String;var basis:String?;var authority:String?}
struct ConnectedBehaviorGraph:Decodable {var nodes:[String:ConnectedBehaviorNode];var edges:[ResourceJSON];var checks:[ConnectedBehaviorCheck];var notices:[String]?;var truncated:Bool?}
struct ConnectedBehaviorReading:Decodable {var version:Int;var behavior_protocol:Int;var workspace_id:String;var namespace:String;var engine_workspace_id:String;var reading_fingerprint:String;var context:ResourceJSON;var verification_available:Bool?;var graph:ConnectedBehaviorGraph;var evidence:ResourceJSON?;var evidence_limit:String?;var receipts:ConnectedBehaviorReceipts?}
struct ConnectedBehaviorReceipts:Decodable {var records:[ConnectedBehaviorReceipt]}
struct ConnectedBehaviorProfile:Decodable,Identifiable,Equatable {var id:String;var name:String;var fingerprint:String="";enum CodingKeys:String,CodingKey{case id,name};init(id:String,name:String,fingerprint:String){self.id=id;self.name=name;self.fingerprint=fingerprint}}
struct ConnectedBehaviorProfileCatalog:Decodable {var fingerprint:String;var profiles:[ConnectedBehaviorProfile];var notices:[String]?}
struct ConnectedBehaviorReceipt:Decodable,Identifiable {var version:Int;var request_id:String;var workspace_id:String;var engine_workspace_id:String?;var origin:String?;var reading_fingerprint:String;var check_id:String;var label:String?;var status:String;var outcome:String;var notice:String?;var error:String?;var output:String?;var exit_code:Int?;var profile:ConnectedBehaviorProfileIdentity?;var terminal:ConnectedBehaviorTerminal?;var id:String{request_id}}
struct ConnectedBehaviorProfileIdentity:Decodable {var id:String;var fingerprint:String}
struct ConnectedBehaviorTerminal:Decodable {var id:String}
struct ConnectedBehaviorPage:Decodable {var version:Int;var behavior_protocol:Int;var workspace_id:String;var reading_fingerprint:String;var node_id:String;var source_hash:String;var page_hash:String;var source_path:String?;var fragment_id:String?;var origin:String;var state:String;var target_status:String;var units:String;var offset:Int;var total:Int;var next:Int?;var text:String;var notice:String}
func connectedBehaviorHex(_ text:String,_ count:Int=64)->Bool{text.utf8.count==count&&text.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
func connectedBehaviorData<T:Encodable>(_ value:T)throws->Data{let encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];return try encoder.encode(value)}
func connectedBehaviorBody(_ value:ResourceJSON)throws->[String:Any]{guard let object=try JSONSerialization.jsonObject(with:connectedBehaviorData(value)) as? [String:Any] else{throw EngineError(message:"A captured behavior request must be an object.")};return object}
func connectedBehaviorJSON(_ body:[String:Any])throws->ResourceJSON{try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:body,options:[.sortedKeys]))}
func connectedBehaviorDecode<T:Decodable>(_ raw:ResourceJSON)throws->T{try JSONDecoder().decode(T.self,from:connectedBehaviorData(raw))}
func connectedBehaviorText(_ value:ResourceJSON)->String{(try? String(data:connectedBehaviorData(value),encoding:.utf8)) ?? "Unavailable reading"}
func validateConnectedBehavior(_ value:ConnectedBehaviorReading,context:ResourceJSON,namespace:String,runtime:String)throws {
 guard value.version==1,value.behavior_protocol==1,exactUTF8(value.namespace,namespace),exactUTF8(value.engine_workspace_id,runtime),connectedBehaviorHex(runtime,32),connectedBehaviorHex(value.workspace_id)||(value.workspace_id.isEmpty&&value.verification_available==false),connectedBehaviorHex(value.reading_fingerprint),try connectedBehaviorData(value.context)==connectedBehaviorData(context),value.graph.nodes.count<=10000,value.graph.checks.count<=10000,Set(value.graph.checks.map{Data($0.id.utf8)}).count==value.graph.checks.count else{throw EngineError(message:"Foreign, malformed or unsupported connected behavior reading. No check was started.")}
 for (id,node) in value.graph.nodes {guard exactUTF8(id,node.id) else{throw EngineError(message:"Graph node identity is ambiguous.")}}
}
func validateConnectedBehaviorPage(_ page:ConnectedBehaviorPage,reading:ConnectedBehaviorReading,node:ConnectedBehaviorNode,offset:Int?,prior:ConnectedBehaviorPage?)throws {
 let count=page.text.unicodeScalars.count,sum=page.offset.addingReportingOverflow(page.text.unicodeScalars.count),end=sum.partialValue
 guard !sum.overflow,page.version==1,page.behavior_protocol==1,exactUTF8(page.workspace_id,reading.workspace_id),exactUTF8(page.reading_fingerprint,reading.reading_fingerprint),exactUTF8(page.node_id,node.id),page.source_hash==node.hash,page.source_path.map{Data($0.utf8)}==node.source_path.map{Data($0.utf8)},page.fragment_id.map{Data($0.utf8)}==node.fragment_id.map{Data($0.utf8)},page.units=="unicode-scalars",page.offset>=0,page.total>=0,page.offset<=page.total,count<=4000,count<=page.total-page.offset,page.next==(end<page.total ? end:nil),offset==nil||offset==page.offset,connectedBehaviorHex(page.page_hash),nativeSourceHash(page.text)==page.page_hash,["constructed","captured-draft","saved-source","saved-baseline","retained-summary"].contains(page.origin),["captured","historical-deleted-baseline"].contains(page.state),["whole-source","observed-fragment-context","missing-declared-fragment","ambiguous-declared-fragment"].contains(page.target_status) else{throw EngineError(message:"Foreign or malformed captured Unicode source page. Prior readable text retained.")}
 if page.state=="historical-deleted-baseline" {guard page.origin=="saved-baseline",node.status=="saved-baseline-absent-from-draft" else{throw EngineError(message:"Historical source origin does not match this node.")}}
 if node.fragment_id==nil {guard page.target_status=="whole-source" else{throw EngineError(message:"Fragment status does not match the captured source." )}}
 if let prior,exactUTF8(prior.node_id,node.id),prior.source_hash==page.source_hash {guard prior.total==page.total,prior.origin==page.origin,prior.state==page.state,prior.target_status==page.target_status else{throw EngineError(message:"Captured source changed between pages.")}}
}

@MainActor func connectedBehaviorRetainedContext(_ workspace:Workspace,entry:String,focus:String?)throws->ResourceJSON {
 var sources:[String:String]=[:],guards:[String:String]=[:],unaddressed:[String:String]=[:],missing:[String]=[];var seen=Set<String>(),bytes=0
 let roster=(workspace.buffers.values+Array(workspace.retainedResourceEditors.values)).reduce(into:[UUID:DocumentBuffer]()){$0[$1.id]=$1}.values
 guard roster.count<=4096 else{throw EngineError(message:"Retained behavior context exceeds its editor budget.")}
 for b in roster where b.file.generated_by==nil {let path=b.file.path;guard b.text.utf8.count<=8*1024*1024 else{throw EngineError(message:"Retained behavior source exceeds its capture budget.")};bytes+=b.text.utf8.count+path.utf8.count+(b.savedGuard?.utf8.count ?? 0);guard bytes<=32*1024*1024 else{throw EngineError(message:"Retained behavior context exceeds32MiB.")};let addressable = !b.isUntitledDraft && !path.hasPrefix("/") && !path.split(separator:"/").contains(where:{$0==".."||$0=="."}),named=addressable && !b.resourceDeleted && workspace.buffers[path] === b
  if named {guard seen.insert(path).inserted else{throw EngineError(message:"The graph JSON adapter cannot represent canonically equivalent source keys. All editors remain readable; use a distinct source scope.")};sources[path]=b.text;if let guardValue=b.savedGuard,!guardValue.isEmpty{guards[path]=guardValue}}else{unaddressed[b.id.uuidString.lowercased()]=b.text;if addressable,b.resourceDeleted,workspace.buffers[path]==nil{missing.append(path)}}
 }
 return try connectedBehaviorJSON(["version":1,"entry":entry,"focus":focus as Any? ?? NSNull(),"sources":sources,"missing_sources":missing,"unaddressed_sources":unaddressed,"saved_guards":guards])
}

@MainActor @Observable final class ConnectedBehaviorModel {
 typealias Transport=(String,[String:Any]?)async throws->ResourceJSON
 private var transport:Transport?;private var engine:Engine?;private var ticket=UUID();private var pageTicket=UUID();private var pageReading:UUID?;private var valid:()->Bool={false};private var consumed:[String:(String,UUID)]=[:]
 private(set) var value:ConnectedBehaviorReading?;private(set) var page:ConnectedBehaviorPage?;private(set) var receipts:[ConnectedBehaviorReceipt]=[];private(set) var profiles:[ConnectedBehaviorProfile]=[];private(set) var busy=false;private(set) var pageBusy=false;private(set) var notice=""
 init(transport:Transport?=nil){self.transport=transport}
 private func call(_ engine:Engine,_ path:String,_ body:[String:Any]?=nil)async throws->ResourceJSON {
  if let transport{return try await transport(path,body)}
  guard let endpoint=URLComponents(string:path),endpoint.scheme==nil,endpoint.host==nil,endpoint.path.hasPrefix("/") else{throw EngineError(message:"Invalid internal behavior endpoint.")}
  return try await engine.request(body==nil ? "GET":"POST",endpoint.path,query:endpoint.queryItems ?? [],body:body)
 }
 var current:Bool{value != nil&&valid()}
 var currentPage:Bool{current&&page != nil&&pageReading==ticket}
 func close(){ticket=UUID();pageTicket=UUID();valid={false};busy=false;pageBusy=false;profiles=[]}
 func read(_ workspace:Workspace,entry:String,focus:String?)async {
  guard let engine=workspace.engine,let namespace=workspace.sourceRootAddress,let runtime=workspace.sourceWorkspaceID else{notice="Refresh Files to bind the physical workspace and engine lifetime.";return}
  let epoch=workspace.sourceNavigationEpoch,rows=workspace.buffers,retained=workspace.retainedResourceEditors
  let unique=(Array(rows.values)+Array(retained.values)).reduce(into:[UUID:DocumentBuffer]()){$0[$1.id]=$1}.values
  let snapshots=unique.map{($0,$0.sourceEditIdentity,$0.text,$0.base,$0.hash,$0.savedGuard,$0.file.path,$0.file.generated_by,$0.resourceDeleted,$0.selection,$0.sourceEditorState?()?.identity,$0.isUntitledDraft)}
  let valid={ [weak workspace] in guard let workspace,!workspace.connecting,workspace.showConstruction,workspace.engine?.session===engine.session,workspace.sourceNavigationEpoch==epoch,workspace.sourceRootAddress.map({exactUTF8($0,namespace)})==true,workspace.sourceWorkspaceID.map({exactUTF8($0,runtime)})==true,workspace.buffers.count==rows.count,workspace.retainedResourceEditors.count==retained.count,rows.allSatisfy({workspace.buffers[$0.key] === $0.value}),retained.allSatisfy({workspace.retainedResourceEditors[$0.key] === $0.value}) else{return false};return snapshots.allSatisfy{s in let b=s.0;return b.sourceEditIdentity==s.1&&exactUTF8(b.text,s.2)&&exactUTF8(b.base,s.3)&&exactUTF8(b.hash,s.4)&&b.savedGuard==s.5&&exactUTF8(b.file.path,s.6)&&b.file.generated_by==s.7&&b.resourceDeleted==s.8&&b.selection==s.9 && !b.saving&&b.sourceEditorState?()?.composing != true&&b.sourceEditorState?()?.identity==s.10&&b.isUntitledDraft==s.11}}
  do{let context=try connectedBehaviorRetainedContext(workspace,entry:entry,focus:focus);await read(engine:engine,namespace:namespace,runtime:runtime,context:context,current:valid)}catch{notice=error.localizedDescription}
 }
 func read(engine:Engine,namespace:String,runtime:String,context:ResourceJSON,current:@escaping()->Bool)async {
  let id=UUID();ticket=id;pageTicket=UUID();pageBusy=false;valid={false};busy=true;profiles=[];notice="Reading connected behavior…";defer{if ticket==id{busy=false}}
  do{let reply:ConnectedBehaviorReading=try connectedBehaviorDecode(try await call(engine,"/api/behavior/reading",connectedBehaviorBody(context)));guard ticket==id else{return};try validateConnectedBehavior(reply,context:context,namespace:namespace,runtime:runtime);value=reply;self.engine=engine;valid=current
   for receipt in (reply.receipts?.records ?? []).reversed(){if (try? validateReceipt(receipt,pending:nil,workspace:receipt.workspace_id)) != nil{retain(receipt);if receipt.workspace_id==reply.workspace_id,receipt.engine_workspace_id==runtime,receipt.origin != "unknown",["admitting","process-running","request-pending"].contains(receipt.status),consumed.count<256{consumed[key(reply.reading_fingerprint,receipt.check_id,receipt.profile?.id)]=(receipt.id,id)}}}
   notice=current() ? "Declared relationships, observed construction and qualified evidence. A process exit is not a test verdict.":"Historical captured reading: source/editor ownership changed."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 private func admission(_ value:ConnectedBehaviorReading)throws->[String:Any]{guard current,self.value?.reading_fingerprint==value.reading_fingerprint,value.verification_available==true else{throw EngineError(message:"Refresh current construction authority before verification. Readable captured evidence is retained.")};return ["workspace_id":value.workspace_id,"engine_workspace_id":value.engine_workspace_id,"reading_fingerprint":value.reading_fingerprint,"context":try connectedBehaviorBody(value.context)]}
 private func key(_ fingerprint:String,_ check:String,_ profile:String?)->String{fingerprint+"/"+Data(check.utf8).base64EncodedString()+"/"+Data((profile ?? "").utf8).base64EncodedString()}
 func readSource(_ nodeID:String,offset:Int?=nil)async {
  guard let value,let engine,let node=value.graph.nodes[nodeID],let hash=node.hash,connectedBehaviorHex(hash) else{notice="Full captured source is unavailable for this node.";return};let scope=ticket,id=UUID();pageTicket=id;pageBusy=true;defer{if pageTicket==id{pageBusy=false}}
  do{guard current else{throw EngineError(message:"Refresh the captured source context.")};var body:[String:Any]=["version":1,"workspace_id":value.workspace_id,"reading_fingerprint":value.reading_fingerprint,"node_id":node.id,"source_hash":hash,"context":try connectedBehaviorBody(value.context)];if let offset{guard offset>=0 else{throw EngineError(message:"Choose a nonnegative scalar offset.")};body["offset"]=offset};let reply:ConnectedBehaviorPage=try connectedBehaviorDecode(try await call(engine,"/api/behavior/source",body));guard ticket==scope,pageTicket==id,current else{return};try validateConnectedBehaviorPage(reply,reading:value,node:node,offset:offset,prior:pageReading==ticket ? page:nil);page=reply;pageReading=ticket;notice=reply.notice
  }catch{if ticket==scope,pageTicket==id{notice=error.localizedDescription}}
 }
 func continueSource(_ captured:ConnectedBehaviorPage,offset:Int)async {guard currentPage,page?.node_id==captured.node_id,page?.offset==captured.offset,page?.source_hash==captured.source_hash else{notice="This page belongs to a retired reading; prior text remains readable.";return};await readSource(captured.node_id,offset:offset)}
 func chooseProfiles()async {guard let value,let engine else{return};let scope=ticket;do{let reply:ConnectedBehaviorProfileCatalog=try connectedBehaviorDecode(try await call(engine,"/api/behavior/profiles",admission(value)));guard ticket==scope else{return};_ = try admission(value);guard connectedBehaviorHex(reply.fingerprint),Set(reply.profiles.map{Data($0.id.utf8)}).count==reply.profiles.count else{throw EngineError(message:"Profile inventory is malformed.")};profiles=reply.profiles.map{ConnectedBehaviorProfile(id:$0.id,name:$0.name,fingerprint:reply.fingerprint)};notice=(["Choose an existing profile explicitly. Its relationship to this check is user-declared."]+(reply.notices ?? [])).joined(separator:"\n")}catch{notice=error.localizedDescription}}
 private func validateReceipt(_ value:ConnectedBehaviorReceipt,pending:ConnectedBehaviorReceipt?,workspace:String)throws {
  guard value.version==1,connectedBehaviorHex(value.id,32),exactUTF8(value.workspace_id,workspace),connectedBehaviorHex(value.reading_fingerprint),!value.check_id.isEmpty,value.engine_workspace_id.map({connectedBehaviorHex($0,32)}) ?? true else{throw EngineError(message:"Foreign verification receipt; keep the original request.")}
  if let pending{guard value.id==pending.id,value.engine_workspace_id==pending.engine_workspace_id,value.workspace_id==pending.workspace_id,value.reading_fingerprint==pending.reading_fingerprint,value.check_id==pending.check_id,value.profile?.id==pending.profile?.id,value.profile?.fingerprint==pending.profile?.fingerprint else{throw EngineError(message:"Verification reply belongs to another source, runtime or selected check.")}}
 }
 private func retain(_ receipt:ConnectedBehaviorReceipt){receipts=([receipt]+receipts.filter{$0.id != receipt.id}).prefix(50).map{$0}}
 func run(_ checkID:String,profile:ConnectedBehaviorProfile?=nil)async {
  guard let value,let engine else{return};var request:String?
  do{var body=try admission(value);guard let check=value.graph.checks.first(where:{$0.id==checkID}),check.available||profile != nil else{throw EngineError(message:"This declared check has no supported provider. Choose an existing profile explicitly if appropriate.")};if let profile{guard profiles.contains(profile) else{throw EngineError(message:"Choose the current profile inventory again.")}}
   let key=key(value.reading_fingerprint,checkID,profile?.id);if let prior=consumed[key]{guard prior.1 != ticket,receipts.contains(where:{$0.id==prior.0&&["process-finished","admission-refused","request-refused"].contains($0.status)}) else{throw EngineError(message:"Original verification request is consumed. Inspect its retained outcome; uncertain replies never authorize replacement.")}};guard consumed.count<256||consumed[key] != nil else{throw EngineError(message:"Retained verification capacity reached.")}
   let id=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased();request=id;consumed[key]=(id,ticket);body["version"]=1;body["check_id"]=checkID;body["request_id"]=id;body["profile"]=profile.map{["id":$0.id,"fingerprint":$0.fingerprint]} as Any? ?? NSNull()
   let pending=ConnectedBehaviorReceipt(version:1,request_id:id,workspace_id:value.workspace_id,engine_workspace_id:value.engine_workspace_id,reading_fingerprint:value.reading_fingerprint,check_id:checkID,label:check.label,status:"request-pending",outcome:"unknown",notice:"Uncertain response never authorizes retry",profile:profile.map{ConnectedBehaviorProfileIdentity(id:$0.id,fingerprint:$0.fingerprint)});retain(pending)
   let reply:ConnectedBehaviorReceipt=try connectedBehaviorDecode(try await call(engine,"/api/behavior/verify",body));try validateReceipt(reply,pending:pending,workspace:value.workspace_id);retain(reply);notice="Retained process outcome. Process exit is not a collected case verdict."
  }catch{notice=error.localizedDescription+(request==nil ? "":" Inspect the original request; no replacement is started automatically.")}
 }
 func refresh(_ receipt:ConnectedBehaviorReceipt)async {guard let engine else{return};do{let reply:ConnectedBehaviorReceipt=try connectedBehaviorDecode(try await call(engine,"/api/behavior/verification?id="+receipt.id));try validateReceipt(reply,pending:receipt,workspace:receipt.workspace_id);retain(reply);notice="Retained process observation refreshed without restarting."}catch{notice=error.localizedDescription}}
 func openProcess(_ workspace:Workspace,receipt:ConnectedBehaviorReceipt)async {guard let engine,workspace.engine?.session===engine.session,receipt.engine_workspace_id==workspace.sourceWorkspaceID,receipt.origin=="current-workspace" else{notice="Historical or unknown process origin has no live terminal authority.";return}
  do{let fresh:ConnectedBehaviorReceipt=try connectedBehaviorDecode(try await call(engine,"/api/behavior/verification?id="+receipt.id));try validateReceipt(fresh,pending:receipt,workspace:receipt.workspace_id);retain(fresh);guard workspace.engine?.session===engine.session,fresh.origin=="current-workspace",fresh.engine_workspace_id==workspace.sourceWorkspaceID,let terminal=fresh.terminal?.id,!terminal.isEmpty else{throw EngineError(message:"Retained process is unavailable; no replacement was adopted.")};workspace.terminal.id=terminal;workspace.showUtility(.terminal)}catch{notice=error.localizedDescription}
 }

 func openCurrentSource(_ workspace:Workspace,path:String)async {guard let value,let engine,workspace.engine?.session===engine.session,workspace.sourceRootAddress.map({exactUTF8($0,value.namespace)})==true,workspace.sourceWorkspaceID==value.engine_workspace_id else{notice="Reading belongs to the original workspace lifetime.";return};let scope=ticket
  func find(_ rows:[FileNode])->FileNode?{for file in rows{if exactUTF8(file.path,path){return file};if let found=find(file.children ?? []){return found}};return nil}
  guard let file=workspace.buffers.values.first(where:{exactUTF8($0.file.path,path)})?.file ?? find(workspace.rootFiles),!file.dir else{notice="Current file is unavailable. Captured full text remains readable; no path fallback was attempted.";return};do{_ = try await workspace.loadBuffer(file);guard ticket==scope,workspace.engine?.session===engine.session,workspace.sourceRootAddress.map({exactUTF8($0,value.namespace)})==true,workspace.sourceWorkspaceID==value.engine_workspace_id else{return};workspace.showSource(file.path);notice="Opened current readable source without inferring a captured position."}catch{notice=error.localizedDescription}}
}
struct ConnectedBehaviorView:View {
 var workspace:Workspace;var model:ConnectedBehaviorModel;@State private var selectedNode="";@State private var selectedCheck=""
 var body:some View {VStack(alignment:.leading,spacing:7){
  Text("Connected behavior and relevant verification").font(.headline);Text(model.notice).font(.caption).textSelection(.enabled)
  if let value=model.value {
   Text("Captured physical workspace \(value.workspace_id) · engine lifetime \(value.engine_workspace_id)").font(.caption).textSelection(.enabled)
   Text(model.current ? "Current captured drafts":"Historical reading · refresh before execution").font(.caption)
   Text(value.evidence_limit ?? "Declarations and process observations are not collected test verdicts.").font(.caption)
   if value.verification_available != true {Text("Verification unavailable or unknown; this reading remains readable.").font(.caption)}
   ForEach(value.graph.notices ?? [],id:\.self){Text($0).font(.caption)}
   if value.graph.truncated==true {Text("Graph bounded/truncated; absent relationships are not disproved.").font(.caption)}
   DisclosureGroup("Connected contract, explanation, implementation, output, test and evidence nodes"){
    ForEach(value.graph.nodes.values.sorted{$0.id<$1.id}){node in Button("\(node.kind) · \(node.label) · \(node.status)"){selectedNode=node.id}.buttonStyle(.plain)}
   }
   if let node=value.graph.nodes[selectedNode] {
    Text("\(node.kind) · \(node.label) · \(node.status)").font(.headline);Text("Basis: \(node.basis ?? "unknown") · authority: \(node.authority ?? "unknown")").font(.caption)
    Text(node.preview ?? "No abbreviated text supplied.").font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    Button("Read full captured text"){Task{await model.readSource(node.id)}}.disabled(!model.current||model.pageBusy||node.hash==nil)
    if let path=node.source_path {Button("Open current readable source separately"){Task{await model.openCurrentSource(workspace,path:path)}}}
   }
   if let page=model.page {
    Text("\(model.currentPage ? "Captured":"Historical") source · \(page.origin) · \(page.state) · \(page.target_status) · Unicode scalars \(page.offset)…\(page.offset+page.text.unicodeScalars.count) / \(page.total)").font(.caption)
    Text(page.text).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    HStack{Button("Source beginning"){Task{await model.continueSource(page,offset:0)}}.disabled(!model.currentPage||model.pageBusy);if let next=page.next{Button("Next captured source page"){Task{await model.continueSource(page,offset:next)}}.disabled(!model.currentPage||model.pageBusy)}}
   }
   DisclosureGroup("Declared and observed relationship details"){ForEach(Array(value.graph.edges.enumerated()),id:\.offset){_,edge in Text(connectedBehaviorText(edge)).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}}
   if let evidence=value.evidence {DisclosureGroup("Qualified evidence and change impact"){Text(connectedBehaviorText(evidence)).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}}
   ForEach(value.graph.checks){check in Button("\(check.label) · \(check.available ? "provider available":"unavailable")"){selectedCheck=check.id}.buttonStyle(.plain)}
   if let check=value.graph.checks.first(where:{$0.id==selectedCheck}) {
    Text(check.reason).font(.caption)
    Button("Run selected relevant source test"){Task{await model.run(check.id)}}.disabled(!model.current||value.verification_available != true || !check.available)
    Button("Choose existing profile explicitly"){Task{await model.chooseProfiles()}}.disabled(!model.current||value.verification_available != true)
    ForEach(model.profiles){profile in Button("Run user-declared profile · \(profile.name)"){Task{await model.run(check.id,profile:profile)}}.disabled(!model.current)}
   }
  }
  ForEach(model.receipts){receipt in DisclosureGroup("\(receipt.label ?? receipt.check_id) · \(receipt.status) · \(receipt.outcome)"){
   Text("Request \(receipt.id) · origin \(receipt.origin ?? "unknown") · exit \(receipt.exit_code.map(String.init) ?? "unknown"). Process outcome only; no inferred case verdict.").font(.caption).textSelection(.enabled)
   Text(receipt.notice ?? receipt.error ?? "Retained process observation").font(.caption);if let output=receipt.output{Text(output).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}
   Button("Refresh original request"){Task{await model.refresh(receipt)}};Button("Open retained process if still current"){Task{await model.openProcess(workspace,receipt:receipt)}}.disabled(receipt.origin != "current-workspace")
  }}
 }}
}
