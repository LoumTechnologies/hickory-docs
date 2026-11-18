package com.hickorydocs.simple

import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

class SnippetContextHandshakeTest {
 private val source="λ😀\r\nword"
 private fun proof()=JSONObject().put("source_hash","sha256:"+debugHash(source)).put("source_revision","a".repeat(64))
 private fun original()=JSONObject().put("label","field").put("data",JSONObject().put("hickory_completion","original").put("hickory_context",proof()).put("opaque",JSONObject().put("provider","untouched")))
 private fun requested(p:JSONObject=proof())=JSONObject().put("label","field").put("data",JSONObject().put("hickory_completion","resolved").put("hickory_context",p).put("hickory_context_request",JSONObject().put("version",1).put("variables",org.json.JSONArray().put("CLIPBOARD"))))
 @Test fun demandedClipboardKeepsSnapshotAndOriginalOpaqueData()=runBlocking {
  val original=original();val before=original.toString();val calls=mutableListOf<JSONObject>();var reads=0
  val result=resolveSnippetCompletion(original,source,5,9,{true},{params->calls+=params;if(calls.size==1)requested() else JSONObject().put("label","done").put("data",JSONObject().put("hickory_context",proof()))},{reads++;"clipboard"})
  assertEquals("done",result.getString("label"));assertEquals(1,reads);assertEquals(2,calls.size);assertEquals(before,original.toString())
  val first=calls[0].getJSONObject("data").getJSONObject("hickory_context");val second=calls[1].getJSONObject("data").getJSONObject("hickory_context")
  assertEquals(first.opt("random_hex"),second.opt("random_hex"));assertEquals(first.getJSONObject("clock").toString(),second.getJSONObject("clock").toString());assertTrue(first.isNull("clipboard"));assertEquals("clipboard",second.getJSONObject("clipboard").getString("text"))
 }
 @Test fun foreignProofAndRetiredClipboardCannotCauseSecondProviderRequest()=runBlocking {
  for(foreign in listOf(true,false)){var current=true;var calls=0;var reads=0
   val result=runCatching{resolveSnippetCompletion(original(),source,5,9,{current},{calls++;requested(proof().also{if(foreign)it.put("source_revision","b".repeat(64))})},{reads++;current=false;"secret"})}
   assertTrue(result.isFailure);assertEquals(1,calls);assertEquals(if(foreign)0 else 1,reads)
  }
 }
}
