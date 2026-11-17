import Foundation
import Observation
import SwiftUI
func editorRecoveryToken(_ id:UUID=UUID())->String{id.uuidString.replacingOccurrences(of:"-",with:"").lowercased()}
struct EditorRecoveryAddress:Codable {var kind:String;var path:String?;var name:String?;var owner:String?;var original_path:String?
 var label:String{path ?? name ?? "Recovered source"}
}
struct EditorRecoveryBaseline:Codable {var text:String;var save_guard:String?}
struct EditorRecoveryPosition:Codable {var anchor:Int;var head:Int}
struct EditorRecoveryViewport:Codable {var selection:EditorRecoveryPosition;var scroll_x:Double;var scroll_y:Double}
struct EditorRecoveryBuffer:Codable {var id:String;var address:EditorRecoveryAddress;var text:String;var baseline:EditorRecoveryBaseline?;var view:EditorRecoveryViewport}
struct EditorRecoverySnapshot:Codable {var host:String;var buffers:[EditorRecoveryBuffer];var layout:ResourceJSON}
struct EditorRecoveryPlacement:Codable {var address:String;var id:String}
struct EditorRecoveryLayout:Codable {var tree:PaneLayout;var focused:String;var tabs:[EditorRecoveryPlacement]}
struct EditorRecoverySession:Decodable,Identifiable {var id:String{client_id};var client_id:String;var window_slot:String;var workspace_id:String;var namespace:String;var current_workspace:Bool;var revision:UInt32;var request_id:String;var snapshot_hash:String;var host:String;var buffers:Int;var source_effects:Bool}
struct EditorRecoveryCatalogue:Decodable {var version:Int;var recovery_protocol:Int;var engine_workspace_id:String;var workspace_id:String;var namespace:String;var window_slot:String;var current_window_slot:String;var window_slots:[String];var sessions:[EditorRecoverySession];var notices:[String];var next:String?;var source_effects:Bool}
struct EditorRecoveryEnvelope:Decodable {
 struct Authority:Decodable {var buffer_id:String;var baseline_current:Bool;var write_permission:Bool;var reason:String?}
 var version:Int;var recovery_protocol:Int;var engine_workspace_id:String;var client_id:String;var window_slot:String;var workspace_id:String;var namespace:String;var revision:UInt32;var request_id:String;var snapshot_hash:String;var material:String;var source_effects:Bool;var source_saved:Bool;var current_workspace:Bool;var authority:[Authority]
}
struct EditorRecoveryReceipt:Decodable {var version:Int;var recovery_protocol:Int;var client_id:String;var window_slot:String;var workspace_id:String;var namespace:String;var request_id:String;var revision:UInt32;var snapshot_hash:String;var source_effects:Bool;var source_saved:Bool}
struct EditorRecoveryCheckpoint:Encodable {var version=1;var workspace:String;var window_slot:String;var client_id:String;var request_id:String;var base_revision:UInt32;var snapshot:EditorRecoverySnapshot}
func editorRecoverySlot(_ value:String)->Bool{UInt64(value).map{String($0)==value}==true}
func validateEditorRecoverySnapshot(_ value:EditorRecoverySnapshot)throws {
 guard ["browser","macos","ios","android"].contains(value.host),value.buffers.count<=256,Set(value.buffers.map{$0.id}).count==value.buffers.count else{throw EngineError(message:"Unsupported or ambiguous private editor snapshot.")}
 var bytes=0
 for b in value.buffers {
  guard sourceHistoryHex(b.id,32),["source","unnamed","generated","detached"].contains(b.address.kind),!b.address.label.isEmpty,b.address.label.utf8.count<=16384,!b.address.label.contains("\0"),b.address.kind != "generated" || b.address.owner != nil else{throw EngineError(message:"Malformed private source identity.")}
  for position in [b.view.selection.anchor,b.view.selection.head] {guard position>=0,position<=b.text.utf16.count,String.Index(utf16Offset:position,in:b.text).samePosition(in:b.text.unicodeScalars) != nil else{throw EngineError(message:"Recovery selection splits a Unicode scalar or exceeds its text.")}}
  guard [b.view.scroll_x,b.view.scroll_y].allSatisfy({$0.isFinite&&$0>=0&&$0<=1e9}) else{throw EngineError(message:"Invalid private recovery viewport.")}
  bytes+=b.text.utf8.count+(b.baseline?.text.utf8.count ?? 0);guard bytes<=32*1024*1024,(b.baseline?.save_guard?.utf8.count ?? 0)<=16384 else{throw EngineError(message:"Private editor snapshot exceeds its bounded text budget.")}
 }
 guard try JSONEncoder().encode(value.layout).count<=512*1024 else{throw EngineError(message:"Private pane layout exceeds its metadata budget.")}
}
func editorRecoveryData<T:Encodable>(_ value:T)throws->Data{let encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];return try encoder.encode(value)}
func editorRecoveryBody<T:Encodable>(_ value:T)throws->[String:Any]{guard let result=try JSONSerialization.jsonObject(with:JSONEncoder().encode(value)) as? [String:Any] else{throw EngineError(message:"Private checkpoint must be an object.")};return result}
@MainActor func captureEditorRecovery(_ w:Workspace)throws->EditorRecoverySnapshot {
 let rows=Array(Dictionary(grouping:Array(w.buffers.values)+Array(w.retainedResourceEditors.values),by:{ObjectIdentifier($0)}).values.compactMap{$0.first}).sorted{$0.id.uuidString<$1.id.uuidString}
 let buffers=try rows.map{b->EditorRecoveryBuffer in
  let editor=b.sourceEditorState?();guard editor?.composing != true,!b.saving,editor.map({exactUTF8($0.source,b.text)}) ?? true else{throw EngineError(message:"Finish native input or saving before the next private checkpoint.")}
  let named=w.buffers[b.file.path] === b && !b.resourceDeleted && !b.isUntitledDraft
  let address=b.file.generated_by.map{EditorRecoveryAddress(kind:"generated",path:b.file.path,owner:$0)} ?? (named ? EditorRecoveryAddress(kind:"source",path:b.file.path):b.isUntitledDraft ? EditorRecoveryAddress(kind:"unnamed",name:b.file.name):EditorRecoveryAddress(kind:"detached",name:b.file.name,original_path:b.file.path))
  let selection=editor?.selection ?? b.selection,point=b.captureRecoveryViewport?() ?? CGPoint(x:b.recoveryViewport?.scroll_x ?? 0,y:b.recoveryViewport?.scroll_y ?? 0)
  let baseline=(b.isUntitledDraft || b.resourceDeleted || b.readOnly) ? b.recoveryBaseline ?? EditorRecoveryBaseline(text:b.base,save_guard:b.savedGuard):EditorRecoveryBaseline(text:b.base,save_guard:b.savedGuard)
  return EditorRecoveryBuffer(id:editorRecoveryToken(b.id),address:address,text:b.text,baseline:baseline,view:EditorRecoveryViewport(selection:EditorRecoveryPosition(anchor:selection.location,head:NSMaxRange(selection)),scroll_x:max(0,point.x),scroll_y:max(0,point.y)))
 }
 let tabs=w.layout.groups().flatMap{group in group.tabs.compactMap{address in w.bufferForTab(address).map{EditorRecoveryPlacement(address:address,id:editorRecoveryToken($0.id))}}}
 let layout=try JSONDecoder().decode(ResourceJSON.self,from:JSONEncoder().encode(EditorRecoveryLayout(tree:w.layout,focused:w.focused,tabs:tabs)))
 #if os(macOS)
 let host="macos"
 #else
 let host="ios"
 #endif
 let result=EditorRecoverySnapshot(host:host,buffers:buffers,layout:layout);try validateEditorRecoverySnapshot(result);return result
}
@MainActor func editorRecoveryLease(_ w:Workspace)throws->()->Bool {
 let namespace=captureRetainedSourceNamespace(w),engine=w.engine,runtime=w.sourceWorkspaceID,root=w.sourceRootAddress,layout=try editorRecoveryData(EditorRecoveryLayout(tree:w.layout,focused:w.focused,tabs:[]))
 let rows=(Array(w.buffers.values)+Array(w.retainedResourceEditors.values)).map{b in (b,b.captureRecoveryViewport?(),b.sourceEditorState?()?.identity,b.selection)}
 return { [weak w] in guard let w,w.engine?.session === engine?.session,exactUTF8(w.sourceWorkspaceID,runtime),exactUTF8(w.sourceRootAddress,root),namespace(),(try? editorRecoveryData(EditorRecoveryLayout(tree:w.layout,focused:w.focused,tabs:[])))==layout else{return false};return rows.allSatisfy{b,point,id,selection in b.selection==selection&&b.captureRecoveryViewport?()==point&&b.sourceEditorState?()?.identity==id&&b.sourceEditorState?()?.composing != true}}
}
@MainActor func restoreEditorRecovery(_ snapshot:EditorRecoverySnapshot,into w:Workspace)throws {
 try validateEditorRecoverySnapshot(snapshot);let prefix="recovered-"+editorRecoveryToken();var added:[String:DocumentBuffer]=[:],paths:[String:String]=[:]
 for row in snapshot.buffers {
  let path=prefix+"/"+row.id;guard w.buffers[path]==nil else{throw EngineError(message:"Recovered draft address was reused. Preview again.")}
  let file=FileNode(name:"Recovered "+row.address.label,path:path,dir:false,generated_by:row.address.kind=="generated" ? row.address.owner:nil)
  let b=DocumentBuffer(file:file,text:FileText(path:path,content:row.text,hash:""),isUntitledDraft:row.address.kind != "generated");b.base="";b.selection=NSRange(location:min(row.view.selection.anchor,row.view.selection.head),length:abs(row.view.selection.head-row.view.selection.anchor));b.recoveryViewport=row.view;b.recoveryBaseline=row.baseline;b.saveGuardReason="Private session recovery grants no save authority. Use guarded Save As to publish this draft.";added[path]=b;paths[row.id]=path
 }
 var recovered=PaneLayout(rootID:prefix+"-sources",nodes:[:]);recovered.nodes[recovered.rootID]=PaneNode(id:recovered.rootID,tabs:added.keys.sorted().map(nativeSourceTab),active:added.keys.sorted().first.map(nativeSourceTab));var focus=recovered.rootID
 if ["macos","ios"].contains(snapshot.host) {
  let bytes=try JSONEncoder().encode(snapshot.layout);try validateNativeJSON(bytes);let saved=try JSONDecoder().decode(EditorRecoveryLayout.self,from:bytes)
  var placements=NativeSourceMap<String>();for item in saved.tabs{guard placements[item.address]==nil,paths[item.id] != nil else{throw EngineError(message:"Ambiguous private source placement.")};placements[item.address]=item.id}
  var nodes:[String:PaneNode]=[:],seen:Set<String>=[],placed:Set<String>=[]
  func project(_ id:String,_ depth:Int=0)throws->String {
   guard depth<=32,seen.count<512,seen.insert(id).inserted,let node=saved.tree.nodes[id],exactUTF8(node.id,id) else{throw EngineError(message:"Malformed private pane tree. Open drafts were preserved.")};let next=prefix+"-"+id
   if let first=node.first,let second=node.second {guard ["horizontal","vertical"].contains(node.axis ?? ""),node.ratio.isFinite,(0.1...0.9).contains(node.ratio),node.tabs.isEmpty else{throw EngineError(message:"Malformed private split proportions.")};nodes[next]=PaneNode(id:next,axis:node.axis,first:try project(first,depth+1),second:try project(second,depth+1),ratio:node.ratio)}
   else {guard node.first==nil,node.second==nil,node.axis==nil,node.tabs.count<=512 else{throw EngineError(message:"Malformed private pane group.")};let tabs=try node.tabs.compactMap{address->String? in guard let id=placements[address],let path=paths[id] else{return nil};guard placed.insert(id).inserted else{throw EngineError(message:"One recovered editor occupies multiple pane addresses.")};return nativeSourceTab(path)};let active=node.active.flatMap{placements[$0]}.flatMap{paths[$0]}.map(nativeSourceTab);nodes[next]=PaneNode(id:next,tabs:tabs,active:active ?? tabs.first)};return next
  }
  let root=try project(saved.tree.rootID);recovered=PaneLayout(rootID:root,nodes:nodes);let missing=paths.filter{!placed.contains($0.key)}.values.sorted().map(nativeSourceTab)
  if !missing.isEmpty{let group=prefix+"-unplaced",root=prefix+"-unplaced-split";recovered.nodes[group]=PaneNode(id:group,tabs:missing,active:missing.first);recovered.nodes[root]=PaneNode(id:root,axis:"horizontal",first:recovered.rootID,second:group,ratio:0.75);recovered.rootID=root}
  focus=recovered.groups().first(where:{$0.id==prefix+"-"+saved.focused && !$0.tabs.isEmpty})?.id ?? recovered.groups().first(where:{!$0.tabs.isEmpty})?.id ?? recovered.rootID
 }
 var next=w.layout;guard recovered.nodes.keys.allSatisfy({next.nodes[$0]==nil}) else{throw EngineError(message:"Recovered layout address collision.")};for(id,node)in recovered.nodes{next.nodes[id]=node};let root=prefix+"-session";next.nodes[root]=PaneNode(id:root,axis:"horizontal",first:next.rootID,second:recovered.rootID,ratio:0.5);next.rootID=root
 for(path,b)in added{w.buffers[path]=b};w.layout=next;w.focused=focus
}
@MainActor @Observable final class EditorRecoveryModel {
 var showing=false;private(set) var busy=false;private(set) var notice="";private(set) var catalogue:EditorRecoveryCatalogue?;private(set) var snapshot:EditorRecoverySnapshot?;private(set) var revision:UInt32=0
 var transport:((String,String,[String:Any]?)async throws->ResourceJSON)?
 private var client=editorRecoveryToken();private var scope="";private var last:Data?;private var writing=false;private var ticket=UUID();private var current:()->Bool={false}
 private struct Pending {var checkpoint:EditorRecoveryCheckpoint;var physical:String;var namespace:String;var bytes:Data}
 private var pending:Pending?
 private func call<T:Decodable>(_ e:Engine,_ method:String,_ path:String,_ body:[String:Any]?=nil)async throws->T{if let transport{return try JSONDecoder().decode(T.self,from:JSONEncoder().encode(try await transport(method,path,body)))};let route=URLComponents(string:path)!;return try await e.request(method,route.path,query:route.queryItems ?? [],body:body)}
 private func list(_ w:Workspace,slot:String?=nil)async throws->EditorRecoveryCatalogue {
  guard let e=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress else{throw EngineError(message:"Refresh Files before private session recovery.")};let value:EditorRecoveryCatalogue=try await call(e,"GET","/api/workspace/recovery"+(slot.map{"?window_slot="+$0} ?? ""))
  guard w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace),value.version==1,value.recovery_protocol==1,exactUTF8(value.engine_workspace_id,workspace),exactUTF8(value.namespace,namespace),sourceHistoryHex(value.workspace_id,64),editorRecoverySlot(value.window_slot),editorRecoverySlot(value.current_window_slot),value.window_slots.allSatisfy(editorRecoverySlot),!value.source_effects,value.sessions.count<=32,value.sessions.allSatisfy({sourceHistoryHex($0.client_id,32)&&sourceHistoryHex($0.snapshot_hash,64)&&sourceHistoryHex($0.request_id,32)&&sourceHistoryHex($0.workspace_id,64)&&$0.window_slot==value.window_slot&&$0.revision>0 && !$0.source_effects && (0...256).contains($0.buffers)&&["browser","macos","ios","android"].contains($0.host)}) else{throw EngineError(message:"Foreign or unsupported private session catalogue.")};return value
 }
 func captureLoop(_ w:Workspace)async {while !Task.isCancelled {await checkpoint(w);do{try await Task.sleep(for:.milliseconds(1500))}catch{return}}}
 func checkpoint(_ w:Workspace)async {
  guard !writing,let e=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress else{return};writing=true;defer{writing=false}
  do{let key=String(describing:ObjectIdentifier(e.session))+workspace+namespace;if scope != key{scope=key;client=editorRecoveryToken();revision=0;last=nil;pending=nil}
   let value=try captureEditorRecovery(w),encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];let bytes=try encoder.encode(value)
   if pending==nil,bytes==last{return}
   if pending==nil{let meta=try await list(w);guard w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace) else{return};pending=Pending(checkpoint:EditorRecoveryCheckpoint(workspace:workspace,window_slot:meta.current_window_slot,client_id:client,request_id:editorRecoveryToken(),base_revision:revision,snapshot:value),physical:meta.workspace_id,namespace:namespace,bytes:bytes)}
   guard let held=pending,w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace) else{return}
   let result:EditorRecoveryReceipt=try await call(e,"POST","/api/workspace/recovery/checkpoint",editorRecoveryBody(held.checkpoint))
   guard result.version==1,result.recovery_protocol==1,result.client_id==held.checkpoint.client_id,result.window_slot==held.checkpoint.window_slot,result.workspace_id==held.physical,exactUTF8(result.namespace,held.namespace),result.request_id==held.checkpoint.request_id,held.checkpoint.base_revision<UInt32.max,result.revision==held.checkpoint.base_revision+1,sourceHistoryHex(result.snapshot_hash,64),!result.source_effects,!result.source_saved else{throw EngineError(message:"Private checkpoint acknowledgment changed. The exact request remains retained.")}
   revision=result.revision;last=held.bytes;pending=nil;notice="Private editor checkpoint retained · source files unchanged."
  }catch{notice=error.localizedDescription+" Readable drafts remain available; no source save was claimed."}
 }
 func close(){ticket=UUID();showing=false;busy=false;snapshot=nil;current={false}}
 func open(_ w:Workspace,slot:String?=nil)async {showing=true;let id=UUID();ticket=id;busy=true;snapshot=nil;current={false};defer{if ticket==id{busy=false}};do{let value=try await list(w,slot:slot);guard ticket==id else{return};catalogue=value;notice="Preview a private session. Recovery adds draft copies without replacing open editors or saving source."}catch{if ticket==id{notice=error.localizedDescription}}}
 func read(_ row:EditorRecoverySession,_ w:Workspace)async {guard !busy,let e=w.engine,let workspace=w.sourceWorkspaceID else{return};let id=UUID();ticket=id;busy=true;snapshot=nil;current={false};defer{if ticket==id{busy=false}}
  do{let lease=try editorRecoveryLease(w),value:EditorRecoveryEnvelope=try await call(e,"GET","/api/workspace/recovery/"+row.client_id+"?revision="+String(row.revision)+"&window_slot="+row.window_slot)
   guard value.version==1,value.recovery_protocol==1,exactUTF8(value.engine_workspace_id,workspace),value.client_id==row.client_id,value.window_slot==row.window_slot,value.workspace_id==row.workspace_id,exactUTF8(value.namespace,row.namespace),value.revision==row.revision,value.request_id==row.request_id,value.snapshot_hash==row.snapshot_hash,!value.source_effects,!value.source_saved,value.material.utf8.count<=64*1024*1024,nativeSourceHash(value.material)==value.snapshot_hash else{throw EngineError(message:"Complete private snapshot receipt changed.")}
   let bytes=Data(value.material.utf8);let parsed=try await Task.detached(priority:.userInitiated){try validateNativeReviewJSON(bytes,maxBytes:64*1024*1024,versionKeys:[],label:"Private snapshot");let parsed=try JSONDecoder().decode(EditorRecoverySnapshot.self,from:bytes);try validateEditorRecoverySnapshot(parsed);return parsed}.value
   guard ticket==id,w.engine?.session === e.session,lease(),parsed.host==row.host,parsed.buffers.count==row.buffers,value.authority.count==parsed.buffers.count,parsed.buffers.allSatisfy({b in value.authority.filter{$0.buffer_id==b.id && !$0.write_permission}.count==1}) else{throw EngineError(message:"Private reading or host editor lease changed. Preview again.")};snapshot=parsed;current=lease;notice="Complete checkpoint fingerprint verified. Recovery grants no source write permission."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }
 var canRestore:Bool{!busy&&snapshot != nil&&current()}
 func restore(_ w:Workspace){guard canRestore,let value=snapshot else{notice="Host editor lease changed. Preview again; open drafts were preserved.";return};do{try restoreEditorRecovery(value,into:w);snapshot=nil;current={false};notice="Recovered draft copies and layout. Existing editors and saved source were retained."}catch{notice=error.localizedDescription}}
}
struct EditorRecoveryView:View {
 var workspace:Workspace;@State private var index=0;@State private var page=0
 var body:some View {let model=workspace.editorRecovery
  ScrollView{VStack(alignment:.leading,spacing:8){HStack{Text("Editor session recovery").font(.headline);Spacer();Button("Close"){model.close()}};Text(model.notice).font(.caption);Text("Private checkpoints are draft recovery, not source saves. Recovery adds copies beside existing editors; utility panes stay in place.").font(.caption)
   Button("Refresh sessions"){Task{await model.open(workspace,slot:model.catalogue?.window_slot)}}.disabled(model.busy)
   if let catalogue=model.catalogue{Picker("Retained window",selection:Binding(get:{catalogue.window_slot},set:{slot in Task{await model.open(workspace,slot:slot)}})){ForEach(catalogue.window_slots,id:\.self){Text($0).tag($0)}};ForEach(Array(catalogue.notices.enumerated()),id:\.offset){Text($0.element).font(.caption)};ForEach(catalogue.sessions){row in Button("Read \(row.host) · \(row.buffers) buffers · \(row.client_id) · revision \(row.revision)"){index=0;page=0;Task{await model.read(row,workspace)}}.disabled(model.busy)}}
   if let snapshot=model.snapshot,!snapshot.buffers.isEmpty{let selected=min(index,snapshot.buffers.count-1),row=snapshot.buffers[selected],scalars=Array(row.text.unicodeScalars),count=max(1,(scalars.count+15999)/16000),shown=min(page,count-1)
    Picker("Captured buffer",selection:$index){ForEach(Array(snapshot.buffers.enumerated()),id:\.element.id){i,b in Text(b.address.kind+" · "+b.address.label).tag(i)}}.onChange(of:index){_,_ in page=0}
    Text(String(String.UnicodeScalarView(scalars.dropFirst(shown*16000).prefix(16000)))).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    Text("Text page \(shown+1)/\(count) · \(row.address.kind=="generated" ? "read-only generated text":"unnamed draft; guarded Save As required")").font(.caption)
    HStack{Button("Previous text"){page=shown-1}.disabled(shown==0);Button("Next text"){page=shown+1}.disabled(shown+1>=count)}
    Button("Recover complete session as draft copies"){model.restore(workspace)}.disabled(!model.canRestore)
   }
  }.frame(maxWidth:.infinity,alignment:.leading).padding()}.frame(idealWidth:680,idealHeight:600).task{if !workspace.editorRecovery.busy{await workspace.editorRecovery.open(workspace)}}.onDisappear{workspace.editorRecovery.close()}
 }
}
