import Foundation
struct ProviderSourceProof {var uri:String;var path:String;var sourceHash:String}
func providerSourceProofs(_ action:Any)throws->[ProviderSourceProof] {
 guard let action=action as? [String:Any],let data=action["data"] as? [String:Any],let token=data["hickory_action"] as? String,!token.isEmpty,token.utf8.count<=1024,token.utf8.allSatisfy({(33...126).contains($0)}),let rows=data["hickory_sources"] as? [[String:Any]],!rows.isEmpty,rows.count<=10000 else{throw EngineError(message:"Missing or malformed engine source proof. Prepare the provider action again.")}
 var seen=Set<NativeSourceKey>();var previous:[UInt8]=[];var result:[ProviderSourceProof]=[]
 for row in rows {
  guard Set(row.keys)==Set(["uri","source_hash"]),let uri=row["uri"] as? String,!uri.isEmpty,uri.utf8.allSatisfy({(33...126).contains($0)}),let hash=row["source_hash"] as? String,hash.utf8.count==71,hash.range(of:"^sha256:[a-f0-9]{64}$",options:.regularExpression) != nil else{throw EngineError(message:"Malformed engine source proof row.")}
  let bytes=Array(uri.utf8)
  guard let path=SemanticLocation(uri:uri,line:0,character:0).path,previous.lexicographicallyPrecedes(bytes),seen.insert(NativeSourceKey(path)).inserted else{throw EngineError(message:"Engine source proof identities are unsupported, duplicated or unsorted.")}
  previous=bytes;result.append(ProviderSourceProof(uri:uri,path:path,sourceHash:hash))
 }
 return result
}
func providerSourceProofPaths(_ action:Any)throws->[String] {try providerSourceProofs(action).map(\.path)}
func validateProviderSourceProofs(_ action:Any,sources:NativeSourceMap<SourceSnapshot>,requiredExisting:[String])throws {
 let rows=try providerSourceProofs(action),covered=Set(rows.map{NativeSourceKey($0.path)})
 for path in requiredExisting where !covered.contains(NativeSourceKey(path)){throw EngineError(message:"Edited source lacks engine coordinate proof: \(path). Prepare again.")}
 for row in rows {
  guard let source=sources[row.path] else{throw EngineError(message:"Acquire and synchronize provider proof source before querying again: \(row.path)")}
  guard exactUTF8("sha256:"+nativeSourceHash(source.text),row.sourceHash) else{throw EngineError(message:"Provider source proof changed: \(row.path). Prepare again.")}
 }
}
