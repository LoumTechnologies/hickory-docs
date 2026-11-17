package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.assertEquals
import org.json.JSONObject
import java.io.File
class SnippetCaseTest {
 @Test fun frozenUnicodeCases(){var root=File(System.getProperty("user.dir"));while(!File(root,"apps/simple-contract/snippet-case-cases.json").isFile){root=root.parentFile?:error("Missing shared case vectors")};val vectors=JSONObject(File(root,"apps/simple-contract/snippet-case-cases.json").readText());assertEquals(vectors.getString("case_map"),snippetCaseVersion);val cases=vectors.getJSONArray("cases");for(i in 0 until cases.length()){val c=cases.getJSONObject(i);assertEquals(c.getString("value"),c.getString("expected"),snippetCase(c.getString("value"),c.getString("mode")))};val words=vectors.getJSONArray("words");for(i in 0 until words.length()){val c=words.getJSONObject(i);val expected=c.getJSONArray("expected");assertEquals((0 until expected.length()).map{expected.getString(it)},snippetCaseWords(c.getString("value")))}}
}
