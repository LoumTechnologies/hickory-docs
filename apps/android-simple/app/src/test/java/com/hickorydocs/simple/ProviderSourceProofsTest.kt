package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
import org.junit.Test
import org.junit.Assert.*
class ProviderSourceProofsTest {
 fun row(uri:String="hick:///a",hash:String="sha256:"+"0".repeat(64))=JSONObject().put("uri",uri).put("source_hash",hash)
 fun action(rows:Any)=JSONObject().put("data",JSONObject().put("hickory_action","opaque:1").put("hickory_sources",rows))
 fun rejects(block:()->Unit){try{block();fail("malformed or unproved action accepted")}catch(expected:IllegalArgumentException){}catch(expected:IllegalStateException){}}
 @Test fun discoversContextAndRejectsMalformedEvidence(){
  assertEquals(listOf("a","b"),providerSourceProofPaths(action(JSONArray().put(row()).put(row("hick:///b")))))
  val bad=listOf(JSONObject(),action(JSONArray()),action(JSONArray().put(row("file:///a"))),action(JSONArray().put(row("hick:///a?x"))),action(JSONArray().put(row("hick:///../a"))),action(JSONArray().put(row("hick:///a","sha256:"+"A".repeat(64)))),action(JSONArray().put(row().put("extra",true))),action(JSONArray().put(row()).put(row())),action(JSONArray().put(row("hick:///b")).put(row())),action(JSONArray().put(row("hick:///%61")).put(row())))
  for(value in bad)rejects{providerSourceProofs(value)}
 }
 @Test fun frozenUTF8ContextHashAndExistingEditCoverage(){
  val original=SourceSnapshot("é😀",false);val proof=action(JSONArray().put(row("hick:///a","sha256:"+debugHash(original.text))))
  validateProviderSourceProofs(proof,mapOf("a" to original),setOf("a"))
  rejects{validateProviderSourceProofs(proof,mapOf("a" to SourceSnapshot("e\u0301😀",false)),emptySet())}
  rejects{validateProviderSourceProofs(proof,emptyMap(),emptySet())}
  rejects{validateProviderSourceProofs(proof,mapOf("a" to original),setOf("unproved-existing"))}
  validateProviderSourceProofs(proof,mapOf("a" to original),emptySet()) // Caller excludes created destinations.
 }
}
