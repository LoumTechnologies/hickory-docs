import Foundation
struct WorkspaceEntryStamp: Decodable, Equatable {
 let version: UInt32
 let identity: String
 let generation: UInt32
 let path: String?
 enum CodingKeys:String,CodingKey {case version,identity,generation,path}
 init(from decoder:Decoder)throws {
  let values=try decoder.container(keyedBy:CodingKeys.self)
  version=try values.decode(UInt32.self,forKey:.version)
  identity=try values.decode(String.self,forKey:.identity)
  generation=try values.decode(UInt32.self,forKey:.generation)
  path=try values.decode(String?.self,forKey:.path)
  guard version==1,workspaceEntryHex(identity,32) else{throw EntrySelectionFailure("Workspace entry stamp is malformed.")}
 }
 static func ==(left:Self,right:Self)->Bool {
  left.version==right.version&&left.identity==right.identity&&left.generation==right.generation&&exactUTF8(left.path,right.path)
 }
}
private func workspaceEntryHex(_ text:String,_ count:Int)->Bool {text.utf8.count==count&&text.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
struct WorkspaceEntrySelectionProof {
 private struct Wire: Decodable {
  let version: UInt32
  let entry_protocol: UInt32
  let workspace: String
  let namespace: String
  let entry: WorkspaceEntryStamp
  let guardValue: String
  let source_guard: String?
  let source_hash: String?
  let movable: Bool
  let source_effects: Bool
  enum CodingKeys:String,CodingKey {case version,entry_protocol,workspace,namespace,entry,source_guard,source_hash,movable,source_effects;case guardValue="guard"}
 }
 private let material: Data
 let workspace: String
 let namespace: String
 let entry: WorkspaceEntryStamp
 let movable: Bool
 init(_ value:Any)throws {
  guard let object=value as? [String:Any],JSONSerialization.isValidJSONObject(object) else {throw EntrySelectionFailure("Workspace entry response is malformed.")}
  guard let entryBody=object["entry"] as? [String:Any],entryBody.keys.contains("path"),object.keys.allSatisfy({["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].contains($0)}),entryBody.keys.allSatisfy({["version","identity","generation","path"].contains($0)}) else {throw EntrySelectionFailure("Workspace entry path field is missing.")}
  let data=try JSONSerialization.data(withJSONObject:object,options:[.sortedKeys]),wire=try JSONDecoder().decode(Wire.self,from:data)
  let hex=workspaceEntryHex
  guard wire.version==1,wire.entry_protocol==1,hex(wire.workspace,32),!wire.namespace.isEmpty,hex(wire.guardValue,64),!wire.source_effects,wire.entry.version==1,hex(wire.entry.identity,32) else {throw EntrySelectionFailure("Workspace entry response has no supported exact authority.")}
  if let path=wire.entry.path {
   guard !path.isEmpty,!path.contains(":"),!path.contains("\\"),!path.unicodeScalars.contains(where:{$0.value<32||$0.value==127}),!path.components(separatedBy:"/").contains(where:{$0.isEmpty||$0=="."||$0==".."}),let physical=wire.source_guard,!physical.isEmpty,physical.utf8.count<=16384,let hash=wire.source_hash,hash.hasPrefix("sha256:"),hex(String(hash.dropFirst(7)),64) else {throw EntrySelectionFailure("Workspace entry lacks a confined path and original physical source proof.")}
  } else if object.keys.contains("source_guard") || object.keys.contains("source_hash") || wire.movable {throw EntrySelectionFailure("An absent entry cannot carry source or move authority.")}
  material=data;workspace=wire.workspace;namespace=wire.namespace;entry=wire.entry;movable=wire.movable
 }
 func body()throws->Any {try JSONSerialization.jsonObject(with:material)}
}
struct EntrySelectionFailure: LocalizedError {let message:String;init(_ message:String){self.message=message};var errorDescription:String?{message}}
@MainActor struct FrozenWorkspaceEntrySelection {
 let entryPath: String?
 let proof: WorkspaceEntrySelectionProof
 let current: ()->Bool
}
@MainActor enum WorkspaceEntrySelections {
 static func read(load:@MainActor () async throws->Any,workspace:String,namespace:String,current:@escaping @MainActor ()->Bool) async throws->FrozenWorkspaceEntrySelection {
  guard current() else {throw EntrySelectionFailure("Original source context changed before entry acquisition.")}
  let proof=try WorkspaceEntrySelectionProof(await load())
  guard current(),proof.workspace==workspace,proof.namespace==namespace else {throw EntrySelectionFailure("Workspace entry response belongs to an obsolete source context.")}
  return FrozenWorkspaceEntrySelection(entryPath:proof.entry.path,proof:proof,current:current)
 }
}

struct WorkspaceEntryIntent:Decodable {
 let version:Int
 let workspace:String
 let namespace:String
 let token:String
 let context:String
 let kind:String
 let title:String
 let before_entry:WorkspaceEntryStamp
 let after_entry:WorkspaceEntryStamp
 let edit:ResourceJSON
 let source_files_modified:Bool

 func admit(_ proof:WorkspaceEntrySelectionProof,kind expected:String,path:String?)throws {
  guard version==1,workspace==proof.workspace,exactUTF8(namespace,proof.namespace),kind==expected,!source_files_modified,
   workspaceEntryHex(token,64),context==token,before_entry==proof.entry,after_entry.identity==before_entry.identity,
   UInt64(after_entry.generation)==UInt64(before_entry.generation)+1,exactUTF8(after_entry.path,path),!title.isEmpty else {
   throw EntrySelectionFailure("The entry review did not retain the requested construction and address.")
  }
 }
}

struct WorkspaceEntryRetainedTransition:Decodable {
 let version:Int
 let state:String
 let request_id:String?
 let manifest_sha256:String?
 let kind:String?
 let before_entry:WorkspaceEntryStamp?
 let after_entry:WorkspaceEntryStamp?
 let notice:String?

 static func read(_ value:ResourceJSON?,record:String)throws->(before:WorkspaceEntryStamp,after:WorkspaceEntryStamp)? {
  guard let value else{return nil}
  if case .null=value{return nil}
  let transition=try JSONDecoder().decode(Self.self,from:JSONEncoder().encode(value))
  if transition.version==1,transition.state=="unavailable",let notice=transition.notice,!notice.isEmpty{throw EntrySelectionFailure(notice)}
  guard transition.version==1,transition.state=="retained",transition.request_id==record,
   let manifest=transition.manifest_sha256,workspaceEntryHex(manifest,64),let kind=transition.kind,["move","restoration","create","repair"].contains(kind),
   let before=transition.before_entry,let after=transition.after_entry,before.identity==after.identity,
   UInt64(after.generation)==UInt64(before.generation)+1 else {throw EntrySelectionFailure("This recovery record lacks a supported entry transition.")}
  return (before,after)
 }
}

struct WorkspaceEntryPublication:Decodable {
 let version:Int
 let state:String
 let request_id:String
 let before_entry:WorkspaceEntryStamp
 let after_entry:WorkspaceEntryStamp
 let current_entry:WorkspaceEntryStamp

 static func confirm(_ outcome:Self?,request:String,intent:WorkspaceEntryIntent?)throws {
  guard let intent else{
   if outcome != nil{throw EntrySelectionFailure("Unexpected main construction publication.")}
   return
  }
  guard let outcome,outcome.version==1,outcome.state=="confirmed",outcome.request_id==request,
   outcome.before_entry==intent.before_entry,outcome.after_entry==intent.after_entry,outcome.current_entry==intent.after_entry else {
   throw EntrySelectionFailure("The original main construction publication is unconfirmed.")
  }
 }
}
