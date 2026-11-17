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
struct EditorRecoveryRevision:Decodable,Identifiable {var id:UInt32{revision};var revision:UInt32;var request_id:String;var snapshot_hash:String;var host:String;var buffers:Int}
struct EditorRecoveryRemoval {var client_id:String;var window_slot:String;var record_guard:String?}
struct EditorRecoveryUnreadable:Decodable,Identifiable {var id:String{client_id};var client_id:String;var window_slot:String;var record_guard:String;var bytes:Int;var reason:String;var removal:EditorRecoveryRemoval{EditorRecoveryRemoval(client_id:client_id,window_slot:window_slot,record_guard:record_guard)}}
struct EditorRecoverySession:Decodable,Identifiable {var id:String{client_id};var client_id:String;var window_slot:String;var workspace_id:String;var namespace:String;var current_workspace:Bool;var revision:UInt32;var request_id:String;var snapshot_hash:String;var host:String;var buffers:Int;var source_effects:Bool;var revisions:[EditorRecoveryRevision]?=nil;var record_guard:String?=nil
 var versions:[EditorRecoveryRevision]{revisions ?? [EditorRecoveryRevision(revision:revision,request_id:request_id,snapshot_hash:snapshot_hash,host:host,buffers:buffers)]}
 var removal:EditorRecoveryRemoval{EditorRecoveryRemoval(client_id:client_id,window_slot:window_slot,record_guard:record_guard)}
}
struct EditorRecoveryCatalogue:Decodable {var version:Int;var recovery_protocol:Int;var engine_workspace_id:String;var workspace_id:String;var namespace:String;var window_slot:String;var current_window_slot:String;var window_slots:[String];var sessions:[EditorRecoverySession];var notices:[String];var next:String?;var source_effects:Bool;var unreadable_sessions:[EditorRecoveryUnreadable]?=nil}
struct EditorRecoveryEnvelope:Decodable {
 struct Authority:Decodable {var buffer_id:String;var baseline_current:Bool;var write_permission:Bool;var reason:String?}
 var version:Int;var recovery_protocol:Int;var engine_workspace_id:String;var client_id:String;var window_slot:String;var workspace_id:String;var namespace:String;var revision:UInt32;var request_id:String;var snapshot_hash:String;var material:String;var source_effects:Bool;var source_saved:Bool;var current_workspace:Bool;var authority:[Authority]
}
struct EditorRecoveryReceipt:Decodable {var version:Int;var recovery_protocol:Int;var client_id:String;var window_slot:String;var workspace_id:String;var namespace:String;var request_id:String;var revision:UInt32;var snapshot_hash:String;var source_effects:Bool;var source_saved:Bool}
struct EditorRecoveryCheckpoint:Encodable {var version=1;var workspace:String;var window_slot:String;var client_id:String;var request_id:String;var base_revision:UInt32;var snapshot:EditorRecoverySnapshot}

struct EditorRecoveryForgetRequest:Encodable {var version=1;var workspace:String;var window_slot:String;var client_id:String;var request_id:String;var record_guard:String}
struct EditorRecoveryForgetReceipt:Decodable {var version:Int;var recovery_protocol:Int;var engine_workspace_id:String;var workspace_id:String;var namespace:String;var window_slot:String;var client_id:String;var request_id:String;var guard_hash:String;var forgotten:Bool;var replayed:Bool;var source_effects:Bool;var source_saved:Bool}
func editorRecoverySelectRevision(_ row:EditorRecoverySession,_ revision:UInt32?)throws->EditorRecoverySession {
 let wanted=revision ?? row.revision;guard let metadata=row.versions.first(where:{$0.revision==wanted}) else{throw EngineError(message:"Selected recovery revision is unavailable. Refresh sessions.")}
 var result=row;result.revision=metadata.revision;result.request_id=metadata.request_id;result.snapshot_hash=metadata.snapshot_hash;result.host=metadata.host;result.buffers=metadata.buffers;return result
}
func validateEditorRecoveryCatalogue(_ value:EditorRecoveryCatalogue,after:String?=nil)throws {
 func valid(_ row:EditorRecoveryRevision)->Bool{row.revision>0&&sourceHistoryHex(row.request_id,32)&&sourceHistoryHex(row.snapshot_hash,64)&&["browser","macos","ios","android"].contains(row.host)&&row.buffers>=0&&row.buffers<=256}
 var seen=Set<String>();for row in value.sessions {
  guard seen.insert(row.client_id).inserted,after.map({row.client_id>$0}) ?? true,row.record_guard.map({!$0.isEmpty&&$0.utf8.count<=16384}) ?? true else{throw EngineError(message:"Private session identities or record guards are malformed.")}
  if let versions=row.revisions{guard !versions.isEmpty,versions.count<=3 else{throw EngineError(message:"Retained revision list is malformed.")};var previous:UInt32=0;for version in versions{guard valid(version),version.revision>previous else{throw EngineError(message:"Retained revision ordering changed.")};previous=version.revision};let last=versions.last!;guard last.revision==row.revision,last.request_id==row.request_id,last.snapshot_hash==row.snapshot_hash,last.host==row.host,last.buffers==row.buffers else{throw EngineError(message:"Latest private revision metadata changed.")}}
 }
 guard (value.unreadable_sessions?.count ?? 0)+value.sessions.count<=32 else{throw EngineError(message:"Private catalogue exceeds32sessions.")}
 for row in value.unreadable_sessions ?? []{guard sourceHistoryHex(row.client_id,32),seen.insert(row.client_id).inserted,row.window_slot==value.window_slot,!row.record_guard.isEmpty,row.record_guard.utf8.count<=16384,row.bytes>=0,row.bytes<=64*1024*1024,after.map({row.client_id>$0}) ?? true else{throw EngineError(message:"Unreadable private record metadata changed.")}}
 if let next=value.next{guard sourceHistoryHex(next,32),after.map({next>$0}) ?? true else{throw EngineError(message:"Recovery page cursor did not advance.")}}
}
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
 private struct PendingForget {var body:EditorRecoveryForgetRequest;var physical:String;var namespace:String}
 private var pendingForget:PendingForget?
 private(set) var readingRevision:UInt32?
 private func call<T:Decodable>(_ e:Engine,_ method:String,_ path:String,_ body:[String:Any]?=nil)async throws->T{if let transport{return try JSONDecoder().decode(T.self,from:JSONEncoder().encode(try await transport(method,path,body)))};let route=URLComponents(string:path)!;return try await e.request(method,route.path,query:route.queryItems ?? [],body:body)}
 private func list(_ w:Workspace,slot:String?=nil,after:String?=nil)async throws->EditorRecoveryCatalogue {
  guard let e=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress else{throw EngineError(message:"Refresh Files before private session recovery.")};let value:EditorRecoveryCatalogue=try await call(e,"GET","/api/workspace/recovery"+([slot.map{"window_slot="+$0},after.map{"after="+$0}].compactMap{$0}.isEmpty ? "":"?"+[slot.map{"window_slot="+$0},after.map{"after="+$0}].compactMap{$0}.joined(separator:"&")))
  guard w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace),value.version==1,value.recovery_protocol==1,exactUTF8(value.engine_workspace_id,workspace),exactUTF8(value.namespace,namespace),sourceHistoryHex(value.workspace_id,64),editorRecoverySlot(value.window_slot),editorRecoverySlot(value.current_window_slot),value.window_slots.allSatisfy(editorRecoverySlot),!value.source_effects,value.sessions.count<=32,value.sessions.allSatisfy({sourceHistoryHex($0.client_id,32)&&sourceHistoryHex($0.snapshot_hash,64)&&sourceHistoryHex($0.request_id,32)&&sourceHistoryHex($0.workspace_id,64)&&$0.window_slot==value.window_slot&&$0.revision>0 && !$0.source_effects && (0...256).contains($0.buffers)&&["browser","macos","ios","android"].contains($0.host)}) else{throw EngineError(message:"Foreign or unsupported private session catalogue.")} ;try validateEditorRecoveryCatalogue(value,after:after);return value
 }
 func captureLoop(_ w:Workspace)async {while !Task.isCancelled {await checkpoint(w);do{try await Task.sleep(for:.milliseconds(1500))}catch{return}}}
 func checkpoint(_ w:Workspace)async {
  guard !writing,!busy,let e=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress else{return};writing=true;defer{writing=false}
  do{let key=String(describing:ObjectIdentifier(e.session))+"\0"+workspace+"\0"+namespace;if !exactUTF8(scope,key){scope=key;client=editorRecoveryToken();revision=0;last=nil;pending=nil;if pendingForget.map({$0.body.workspace != workspace || !exactUTF8($0.namespace,namespace)})==true{pendingForget=nil}}
   if pendingForget != nil{return}
   let value=try captureEditorRecovery(w),encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];let bytes=try encoder.encode(value)
   if pending==nil,bytes==last{return}
   if pending==nil{let meta=try await list(w);guard w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace) else{return};pending=Pending(checkpoint:EditorRecoveryCheckpoint(workspace:workspace,window_slot:meta.current_window_slot,client_id:client,request_id:editorRecoveryToken(),base_revision:revision,snapshot:value),physical:meta.workspace_id,namespace:namespace,bytes:bytes)}
   guard let held=pending,w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace) else{return}
   let result:EditorRecoveryReceipt=try await call(e,"POST","/api/workspace/recovery/checkpoint",editorRecoveryBody(held.checkpoint))
   guard result.version==1,result.recovery_protocol==1,result.client_id==held.checkpoint.client_id,result.window_slot==held.checkpoint.window_slot,result.workspace_id==held.physical,exactUTF8(result.namespace,held.namespace),result.request_id==held.checkpoint.request_id,held.checkpoint.base_revision<UInt32.max,result.revision==held.checkpoint.base_revision+1,sourceHistoryHex(result.snapshot_hash,64),!result.source_effects,!result.source_saved else{throw EngineError(message:"Private checkpoint acknowledgment changed. The exact request remains retained.")}
   revision=result.revision;last=held.bytes;pending=nil;notice="Private editor checkpoint retained · source files unchanged."
  }catch{notice=error.localizedDescription+" Readable drafts remain available; no source save was claimed."}
 }
 func close(){ticket=UUID();showing=false;busy=false;snapshot=nil;readingRevision=nil;current={false}}
 func open(_ w:Workspace,slot:String?=nil)async {showing=true;let id=UUID();ticket=id;busy=true;snapshot=nil;readingRevision=nil;current={false};defer{if ticket==id{busy=false}};do{let value=try await list(w,slot:slot);guard ticket==id else{return};catalogue=value;notice="Preview a private session. Recovery adds draft copies without replacing open editors or saving source."}catch{if ticket==id{notice=error.localizedDescription}}}
 func read(_ original:EditorRecoverySession,_ w:Workspace,revision:UInt32?=nil)async {guard !busy,let e=w.engine,let workspace=w.sourceWorkspaceID else{return};let id=UUID();ticket=id;busy=true;snapshot=nil;readingRevision=nil;current={false};defer{if ticket==id{busy=false}}
  do{let row=try editorRecoverySelectRevision(original,revision);let lease=try editorRecoveryLease(w),value:EditorRecoveryEnvelope=try await call(e,"GET","/api/workspace/recovery/"+row.client_id+"?revision="+String(row.revision)+"&window_slot="+row.window_slot)
   guard value.version==1,value.recovery_protocol==1,exactUTF8(value.engine_workspace_id,workspace),value.client_id==row.client_id,value.window_slot==row.window_slot,value.workspace_id==row.workspace_id,exactUTF8(value.namespace,row.namespace),value.revision==row.revision,value.request_id==row.request_id,value.snapshot_hash==row.snapshot_hash,!value.source_effects,!value.source_saved,value.material.utf8.count<=64*1024*1024,nativeSourceHash(value.material)==value.snapshot_hash else{throw EngineError(message:"Complete private snapshot receipt changed.")}
   let bytes=Data(value.material.utf8);let parsed=try await Task.detached(priority:.userInitiated){try validateNativeReviewJSON(bytes,maxBytes:64*1024*1024,versionKeys:[],label:"Private snapshot");let parsed=try JSONDecoder().decode(EditorRecoverySnapshot.self,from:bytes);try validateEditorRecoverySnapshot(parsed);return parsed}.value
   guard ticket==id,w.engine?.session === e.session,lease(),parsed.host==row.host,parsed.buffers.count==row.buffers,value.authority.count==parsed.buffers.count,parsed.buffers.allSatisfy({b in value.authority.filter{$0.buffer_id==b.id && !$0.write_permission}.count==1}) else{throw EngineError(message:"Private reading or host editor lease changed. Preview again.")};snapshot=parsed;readingRevision=value.revision;current=lease;notice="Complete checkpoint fingerprint verified. Recovery grants no source write permission."
  }catch{if ticket==id{notice=error.localizedDescription}}
 }

 func clearReading(){ticket=UUID();busy=false;snapshot=nil;readingRevision=nil;current={false}}
 func more(_ w:Workspace)async {guard !busy,let original=catalogue,let cursor=original.next else{return};let id=UUID();ticket=id;busy=true;snapshot=nil;readingRevision=nil;current={false};defer{if ticket==id{busy=false}}
  do{var page=try await list(w,slot:original.window_slot,after:cursor);guard ticket==id else{return};let old=original.sessions.map(\.client_id)+(original.unreadable_sessions ?? []).map(\.client_id),added=page.sessions.map(\.client_id)+(page.unreadable_sessions ?? []).map(\.client_id);guard page.workspace_id==original.workspace_id,page.current_window_slot==original.current_window_slot,page.window_slot==original.window_slot,old.count+added.count<=32,Set(old).isDisjoint(with:added) else{throw EngineError(message:"Private catalogue changed or exceeds32sessions. Refresh first.")};page.sessions=original.sessions+page.sessions;page.unreadable_sessions=(original.unreadable_sessions ?? [])+(page.unreadable_sessions ?? []);page.notices=original.notices+page.notices;catalogue=page;notice="Loaded more private sessions. Current editors and source remain unchanged."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 var hasPendingForget:Bool{pendingForget != nil}
 func stopRetryingForget(){guard !busy,let held=pendingForget else{return};pendingForget=nil;if held.body.client_id==client{client=editorRecoveryToken();revision=0;pending=nil;last=nil};notice="Stopped retrying the original removal; its outcome may be unknown. Refresh sessions. Current editors and source are unchanged."}
 func retryForget(_ w:Workspace)async {guard let held=pendingForget else{return};await forget(EditorRecoveryRemoval(client_id:held.body.client_id,window_slot:held.body.window_slot,record_guard:held.body.record_guard),w)}
 func forget(_ target:EditorRecoveryRemoval,_ w:Workspace)async {guard !busy,!writing,let e=w.engine,let workspace=w.sourceWorkspaceID,let namespace=w.sourceRootAddress else{return};let id=UUID();ticket=id;busy=true;snapshot=nil;readingRevision=nil;current={false};defer{if ticket==id{busy=false}}
  do{guard let meta=catalogue,meta.engine_workspace_id==workspace,exactUTF8(meta.namespace,namespace),let guardText=target.record_guard else{throw EngineError(message:"Refresh the private session before forgetting it.")}
   if pendingForget==nil{let choices=meta.sessions.map(\.removal)+(meta.unreadable_sessions ?? []).map(\.removal);guard choices.contains(where:{$0.client_id==target.client_id&&$0.window_slot==target.window_slot&&exactUTF8($0.record_guard,guardText)}) else{throw EngineError(message:"Selected private record changed.")};pendingForget=PendingForget(body:EditorRecoveryForgetRequest(workspace:workspace,window_slot:target.window_slot,client_id:target.client_id,request_id:editorRecoveryToken(),record_guard:guardText),physical:meta.workspace_id,namespace:namespace)}
   guard let held=pendingForget,held.body.workspace==workspace,exactUTF8(held.namespace,namespace),held.body.client_id==target.client_id,held.body.window_slot==target.window_slot,exactUTF8(held.body.record_guard,guardText) else{throw EngineError(message:"Resolve the original uncertain removal before selecting another session.")}
   let result:EditorRecoveryForgetReceipt=try await call(e,"POST","/api/workspace/recovery/forget",editorRecoveryBody(held.body))
   guard result.version==1,result.recovery_protocol==1,result.engine_workspace_id==workspace,result.workspace_id==held.physical,exactUTF8(result.namespace,namespace),result.window_slot==held.body.window_slot,result.client_id==held.body.client_id,result.request_id==held.body.request_id,result.guard_hash==nativeSourceHash(held.body.record_guard),result.forgotten,!result.source_effects,!result.source_saved,w.engine?.session === e.session,exactUTF8(w.sourceWorkspaceID,workspace),exactUTF8(w.sourceRootAddress,namespace),pendingForget?.body.request_id==held.body.request_id else{throw EngineError(message:"Private removal acknowledgment or connection changed. Exact request retained.")}
   pendingForget=nil;if held.body.client_id==client{client=editorRecoveryToken();revision=0;pending=nil;last=nil}
   if var catalogue=catalogue{catalogue.sessions.removeAll{$0.client_id==held.body.client_id&&$0.window_slot==held.body.window_slot&&exactUTF8($0.record_guard,held.body.record_guard)};catalogue.unreadable_sessions=catalogue.unreadable_sessions?.filter{!($0.client_id==held.body.client_id&&$0.window_slot==held.body.window_slot&&exactUTF8($0.record_guard,held.body.record_guard))};self.catalogue=catalogue}
   notice="Original private session forgotten. Current editors and source were retained; replay never removes a replacement."
  }catch{if ticket==id{notice=error.localizedDescription+" Retry the exact original removal or stop retrying and refresh."}}
 }
 var canRestore:Bool{!busy&&snapshot != nil&&current()}
 func restore(_ w:Workspace){guard canRestore,let value=snapshot else{notice="Host editor lease changed. Preview again; open drafts were preserved.";return};do{try restoreEditorRecovery(value,into:w);snapshot=nil;current={false};notice="Recovered draft copies and layout. Existing editors and saved source were retained."}catch{notice=error.localizedDescription}}
}
struct EditorRecoveryView:View {
 var workspace:Workspace;@State private var index=0;@State private var page=0;@State private var selected:[String:UInt32]=[:];@State private var armed:String?
 private func chosen(_ row:EditorRecoverySession)->UInt32{let value=selected[row.client_id] ?? row.revision;return row.versions.contains(where:{$0.revision==value}) ? value:row.revision}
 private func removal(_ row:EditorRecoveryRemoval,count:Int?)->some View {let model=workspace.editorRecovery;let confirming=row.record_guard != nil&&exactUTF8(armed,row.record_guard)
  return Button(confirming ? (count.map{"Confirm forget entire private session (\($0) retained revisions)"} ?? "Confirm forget unreadable private record"):"Forget private session…"){guard let guardText=row.record_guard else{return};if !confirming{armed=guardText}else{armed=nil;Task{await model.forget(row,workspace)}}}.disabled(model.busy||row.record_guard==nil).accessibilityLabel((confirming ? "Confirm forget entire private session ":"Forget private session ")+row.client_id)
 }
 private func session(_ row:EditorRecoverySession)->some View {let model=workspace.editorRecovery
  return VStack(alignment:.leading){Picker("Retained revision "+row.client_id,selection:Binding(get:{chosen(row)},set:{value in selected[row.client_id]=value;model.clearReading();page=0;index=0;armed=nil})){ForEach(row.versions){version in Text("Revision \(version.revision) · \(version.buffers) buffers").tag(version.revision)}}.disabled(model.busy)
   Button("Read \(row.host) · \(row.client_id) · revision \(chosen(row))"){index=0;page=0;Task{await model.read(row,workspace,revision:chosen(row))}}.disabled(model.busy);removal(row.removal,count:row.revisions?.count)
  }
 }
 var body:some View {let model=workspace.editorRecovery
  ScrollView{VStack(alignment:.leading,spacing:8){HStack{Text("Editor session recovery").font(.headline);Spacer();Button("Close"){model.close()}};Text(model.notice).font(.caption);Text("Private checkpoints are draft recovery, not source saves. Recovery adds copies beside current editors. Forget removes the entire selected private session, including every retained revision; source and current editors remain unchanged.").font(.caption)
   Button("Refresh sessions"){armed=nil;Task{await model.open(workspace,slot:model.catalogue?.window_slot)}}.disabled(model.busy)
   if model.hasPendingForget{Button("Retry original private removal"){Task{await model.retryForget(workspace)}}.disabled(model.busy);Button("Stop retrying and refresh before another removal"){model.stopRetryingForget()}.disabled(model.busy)}
   if let catalogue=model.catalogue{Picker("Retained window",selection:Binding(get:{catalogue.window_slot},set:{slot in armed=nil;Task{await model.open(workspace,slot:slot)}})){ForEach(catalogue.window_slots,id:\.self){Text($0).tag($0)}}.disabled(model.busy);ForEach(Array(catalogue.notices.enumerated()),id:\.offset){Text($0.element).font(.caption)};ForEach(catalogue.sessions){row in session(row)};ForEach(catalogue.unreadable_sessions ?? []){row in VStack(alignment:.leading){Text("Unreadable private record \(row.client_id): \(row.reason). Contents were not adopted.").font(.caption);removal(row.removal,count:nil)}};if catalogue.next != nil{Button("More private sessions"){Task{await model.more(workspace)}}.disabled(model.busy)}}
   if let snapshot=model.snapshot,!snapshot.buffers.isEmpty{let selected=min(index,snapshot.buffers.count-1),row=snapshot.buffers[selected],scalars=Array(row.text.unicodeScalars),count=max(1,(scalars.count+15999)/16000),shown=min(page,count-1)
    Picker("Captured buffer",selection:$index){ForEach(Array(snapshot.buffers.enumerated()),id:\.element.id){i,b in Text(b.address.kind+" · "+b.address.label).tag(i)}}.onChange(of:index){_,_ in page=0}
    if let revision=model.readingRevision{Text("Reading retained revision \(revision)").font(.caption)}
    Text(String(String.UnicodeScalarView(scalars.dropFirst(shown*16000).prefix(16000)))).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
    Text("Text page \(shown+1)/\(count) · \(row.address.kind=="generated" ? "read-only generated text":"unnamed draft; guarded Save As required")").font(.caption)
    HStack{Button("Previous text"){page=shown-1}.disabled(shown==0);Button("Next text"){page=shown+1}.disabled(shown+1>=count)}
    Button("Recover complete session as draft copies"){model.restore(workspace)}.disabled(!model.canRestore)
   }
  }.frame(maxWidth:.infinity,alignment:.leading).padding()}.frame(idealWidth:680,idealHeight:600).task{if !workspace.editorRecovery.busy{await workspace.editorRecovery.open(workspace)}}.onDisappear{workspace.editorRecovery.close()}
 }
}
