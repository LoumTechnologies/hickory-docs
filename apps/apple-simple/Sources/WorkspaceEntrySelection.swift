import Foundation
struct WorkspaceEntryStamp: Decodable, Equatable {
 let version: UInt32
 let identity: String
 let generation: UInt32
 let path: String?
}
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
 init(_ value:Any)throws {
  guard let object=value as? [String:Any],JSONSerialization.isValidJSONObject(object) else {throw EntrySelectionFailure("Workspace entry response is malformed.")}
  guard let entryBody=object["entry"] as? [String:Any],entryBody.keys.contains("path"),object.keys.allSatisfy({["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].contains($0)}),entryBody.keys.allSatisfy({["version","identity","generation","path"].contains($0)}) else {throw EntrySelectionFailure("Workspace entry path field is missing.")}
  let data=try JSONSerialization.data(withJSONObject:object,options:[.sortedKeys]),wire=try JSONDecoder().decode(Wire.self,from:data)
  func hex(_ text:String,_ count:Int)->Bool {text.utf8.count==count && text.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
  guard wire.version==1,wire.entry_protocol==1,hex(wire.workspace,32),!wire.namespace.isEmpty,hex(wire.guardValue,64),!wire.source_effects,wire.entry.version==1,hex(wire.entry.identity,32) else {throw EntrySelectionFailure("Workspace entry response has no supported exact authority.")}
  if let path=wire.entry.path {
   guard !path.isEmpty,!path.contains(":"),!path.contains("\\"),!path.unicodeScalars.contains(where:{$0.value<32||$0.value==127}),!path.components(separatedBy:"/").contains(where:{$0.isEmpty||$0=="."||$0==".."}),let physical=wire.source_guard,!physical.isEmpty,physical.utf8.count<=16384,let hash=wire.source_hash,hash.hasPrefix("sha256:"),hex(String(hash.dropFirst(7)),64) else {throw EntrySelectionFailure("Workspace entry lacks a confined path and original physical source proof.")}
  } else if object.keys.contains("source_guard") || object.keys.contains("source_hash") || wire.movable {throw EntrySelectionFailure("An absent entry cannot carry source or move authority.")}
  material=data;workspace=wire.workspace;namespace=wire.namespace;entry=wire.entry
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
