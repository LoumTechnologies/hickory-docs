package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONArray
import org.json.JSONObject
class SnippetTransformTest {
 @Test fun sharedTransformVectors(){val root=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("apps/simple-contract/snippet-transform-cases.json").isFile};val rows=JSONArray(root.resolve("apps/simple-contract/snippet-transform-cases.json").readText());for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val program=readSnippetTransform(row.getJSONObject("program"));if(row.optBoolean("reject"))assertTrue(row.getString("name"),runCatching{applySnippetTransform(program,row.getString("input"))}.isFailure)else assertEquals(row.getString("name"),row.getString("expected"),applySnippetTransform(program,row.getString("input")))}}
 @Test fun malformedDescriptorsAndSharedActionWorkLimit(){val raw=JSONObject("""{"version":2,"case_map":"hickory-ucd-16.0.0-v1","regex":["dot"],"format":[["capture",0,""]],"flags":"","groups":0}""");for((key,value)in listOf("version" to 1,"case_map" to "other","groups" to true,"flags" to "gg","regex" to JSONArray("""["class",[[5,1]],false]"""),"regex" to JSONArray("""["capture",true,["dot"]]""")))assertTrue(key,runCatching{readSnippetTransform(JSONObject(raw.toString()).put(key,value))}.isFailure);assertTrue(runCatching{applySnippetTransform(readSnippetTransform(raw),"a",SnippetTransformBudget(99999))}.isFailure);for(input in listOf("\uD800","\uDFFF"))assertTrue(runCatching{applySnippetTransform(readSnippetTransform(raw),input)}.isFailure);assertTrue(runCatching{readSnippetTransform(JSONObject(raw.toString()).put("format",JSONArray().put("\uD800")))}.isFailure)}
}
