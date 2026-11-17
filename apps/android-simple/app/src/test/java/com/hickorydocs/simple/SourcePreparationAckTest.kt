package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class SourcePreparationAckTest {
 @Test fun plainAndLiterateSourcePreparationProofs(){
  val source="value = 'λ'\n";val good=JSONObject().put("processed",true).put("current",true).put("parsed",false).put("open_files",1).put("source_hash","sha256:"+debugHash(source)).put("source_revision","a".repeat(64)).put("source_files_modified",false)
  validateNativeSourcePreparation(good,source);assertTrue(runCatching{validateNativeSourcePreparation(good,source,true)}.isFailure);validateNativeSourcePreparation(JSONObject(good.toString()).put("parsed",true),source,true)
  val bad=listOf("processed" to false,"current" to false,"processed" to 1,"current" to "true","parsed" to "false","open_files" to -1,"source_hash" to "sha256:"+"0".repeat(64),"source_revision" to "legacy-counter","source_revision" to true,"source_files_modified" to true,"source_files_modified" to 0)
  for((key,value)in bad)assertTrue(key,runCatching{validateNativeSourcePreparation(JSONObject(good.toString()).put(key,value),source)}.isFailure)
  val missing=JSONObject(good.toString());missing.remove("source_revision");assertTrue(runCatching{validateNativeSourcePreparation(missing,source)}.isFailure);assertTrue(runCatching{validateNativeSourcePreparation(good,source+" ")}.isFailure)
 }
}
