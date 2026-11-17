package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
data class ProviderSourceProof(val uri:String,val path:String,val sourceHash:String)
fun providerSourceProofs(action:Any?):List<ProviderSourceProof> {
 val data=(action as? JSONObject)?.optJSONObject("data") ?: error("Missing or malformed engine source proof. Prepare the provider action again.")
 val token=data.opt("hickory_action") as? String
 val rows=data.opt("hickory_sources") as? JSONArray
 require(token!=null&&token.matches(Regex("[\\x21-\\x7e]{1,1024}"))&&rows!=null&&rows.length() in 1..10000){"Missing or malformed engine source proof. Prepare the provider action again."}
 val seen=mutableSetOf<String>();var previous="";val result=mutableListOf<ProviderSourceProof>()
 for(index in 0 until rows.length()){
  val row=rows.opt(index) as? JSONObject ?: error("Malformed engine source proof row.")
  val uri=row.opt("uri") as? String;val hash=row.opt("source_hash") as? String
  require(row.keys().asSequence().toSet()==setOf("uri","source_hash")&&uri!=null&&uri.matches(Regex("[\\x21-\\x7e]+"))&&hash!=null&&hash.matches(Regex("sha256:[a-f0-9]{64}"))){"Malformed engine source proof row."}
  val path=SemanticLocation(uri,0,0).path ?: error("Unsupported engine source proof identity.")
  require(uri>previous&&seen.add(path)){"Engine source proof identities are duplicated or unsorted."};previous=uri;result.add(ProviderSourceProof(uri,path,hash))
 }
 return result
}
fun providerSourceProofPaths(action:Any?):List<String> = providerSourceProofs(action).map{it.path}
fun validateProviderSourceProofs(action:Any?,sources:Map<String,SourceSnapshot>,requiredExisting:Set<String>) {
 val rows=providerSourceProofs(action);val covered=rows.map{it.path}.toSet()
 for(path in requiredExisting)require(path in covered){"Edited source lacks engine coordinate proof: $path. Prepare again."}
 for(row in rows){val source=sources[row.path] ?: error("Acquire and synchronize provider proof source before querying again: ${row.path}");require("sha256:"+debugHash(source.text)==row.sourceHash){"Provider source proof changed: ${row.path}. Prepare again."}}
}
