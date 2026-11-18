package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class SnippetExpansionTest {
 @Test fun sharedWireAndImportPositions() {
  val cases=JSONObject(javaClass.classLoader!!.getResource("snippet-expansion-cases.json")!!.readText()).getJSONArray("cases")
  for(index in 0 until cases.length()) {
   val value=cases.getJSONObject(index);val name=value.getString("name");val item=value.getJSONObject("item");val result=runCatching{completionExpansion(item)}
   if(value.optBoolean("error")){assertTrue(name,result.isFailure);continue}
   if(value.optBoolean("plain")){assertNull(name,result.getOrThrow());continue}
   val expansion=result.getOrThrow()!!;val expected=value.getJSONObject("expected");val selection=expected.getJSONObject("selection");assertEquals(name,expected.getString("text"),expansion.text);assertEquals(name,selection.getInt("from"),expansion.selection!!.from);assertEquals(name,selection.getInt("to"),expansion.selection.to)
   val draft=planCompletion("hick:///a.py",item,SourceSnapshot("va",true),0,2,expansion);assertEquals(name,expected.getString("after"),draft.plan.patches[0].after)
   val fields=expected.getJSONArray("fields");assertEquals(name,fields.length(),draft.fields.size)
   draft.fields.forEachIndexed {i,field->val expectedField=fields.getJSONObject(i);assertEquals(name,expectedField.getInt("index"),field.index);assertEquals(name,expectedField.getInt("sourceFrom"),field.from);assertEquals(name,expectedField.getInt("sourceTo"),field.to);if(expectedField.has("id")){assertEquals(name,expectedField.getInt("id"),field.id);assertEquals(name,if(expectedField.isNull("parent"))null else expectedField.getInt("parent"),field.parent)}}
  }
 }
 @Test fun choiceMetadataAmplificationRefusesBeforePreparingSource(){
  val alternative="x".repeat(1024);val syntax="$"+"{1|,"+alternative+"|}"+"$1".repeat(1029)+"$0";val fields=org.json.JSONArray()
  for(id in 0 until 1030)fields.put(JSONObject().put("id",id).put("parent",JSONObject.NULL).put("index",1).put("from",0).put("to",0).put("choices",org.json.JSONArray().put("").put(alternative)))
  fields.put(JSONObject().put("id",1030).put("parent",JSONObject.NULL).put("index",0).put("from",0).put("to",0).put("choices",org.json.JSONArray()))
  val item=JSONObject().put("label","large choices").put("insertTextFormat",2).put("insertText",syntax).put("data",JSONObject().put("hickory_snippet",JSONObject().put("version",1).put("syntax",syntax).put("text","").put("fields",fields)))
  assertTrue(runCatching{completionExpansion(item)}.isFailure)
 }

 @Test fun emptyAlternativesHaveCountAndSlotBudgetsBeforeCopy(){
  fun item(count:Int,mirrors:Int):JSONObject {
   val alternatives=org.json.JSONArray();repeat(count){alternatives.put("")};val fields=org.json.JSONArray()
   repeat(mirrors){fields.put(JSONObject().put("id",it).put("parent",JSONObject.NULL).put("index",1).put("from",0).put("to",0).put("choices",alternatives))}
   fields.put(JSONObject().put("id",mirrors).put("parent",JSONObject.NULL).put("index",0).put("from",0).put("to",0).put("choices",org.json.JSONArray()))
   return JSONObject().put("label","empty choices").put("insertTextFormat",2).put("insertText","syntax").put("data",JSONObject().put("hickory_snippet",JSONObject().put("version",1).put("syntax","syntax").put("text","").put("fields",fields)))
  }
  assertEquals(4096,completionExpansion(item(4096,1))!!.fields.first().choices.size)
  assertTrue(runCatching{completionExpansion(item(4097,1))}.isFailure)
  assertTrue(runCatching{completionExpansion(item(4096,8))}.isFailure)
 }

 @Test fun decodedRowsShareTheirCalculatedMirrorBudget(){
  val syntax="""${'$'}{1:a}|${'$'}{1/(.*)/${'$'}{1:/upcase}/}${'$'}0"""
  val raw="""{"version":2,"case_map":"hickory-ucd-16.0.0-v1","regex":["capture",1,["repeat",["dot"],0,null,false]],"format":[["capture",1,"upcase"]],"flags":"","groups":1}"""
  val fields=org.json.JSONArray().put(JSONObject().put("id",0).put("parent",JSONObject.NULL).put("index",1).put("from",0).put("to",1).put("choices",org.json.JSONArray())).put(JSONObject().put("id",1).put("parent",JSONObject.NULL).put("index",1).put("from",2).put("to",3).put("choices",org.json.JSONArray()).put("transform",JSONObject(raw))).put(JSONObject().put("id",2).put("parent",JSONObject.NULL).put("index",0).put("from",3).put("to",3).put("choices",org.json.JSONArray()))
  val item=JSONObject().put("label","shared mirror").put("insertTextFormat",2).put("insertText",syntax).put("data",JSONObject().put("hickory_snippet",JSONObject().put("version",2).put("syntax",syntax).put("text","a|A").put("fields",fields)))
  assertEquals("a|A",completionExpansion(item)!!.text);val budget=SnippetTransformBudget();completionExpansion(item,budget);val first=budget.work;assertTrue(first>0);completionExpansion(item,budget);assertTrue(budget.work>first);assertTrue(runCatching{completionExpansion(item,SnippetTransformBudget(100000))}.isFailure)
 }
}
