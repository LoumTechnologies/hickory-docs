package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class CompletionPlanTest {
 @Test fun rebasedComputedProgramsRemainUsableAfterImportEdits(){
  val programs=JSONObject(javaClass.classLoader!!.getResource("snippet-derived-cases.json")!!.readText()).getJSONObject("programs");val transform=readSnippetTransform(programs.get("upper"));val fields=listOf(SnippetField(1,0,3,emptyList(),0),SnippetField(1,4,7,emptyList(),1,transform=transform),SnippetField(0,7,7,emptyList(),2))
  val item=JSONObject().put("label","derived").put("insertTextFormat",2).put("insertText","derived").put("additionalTextEdits",org.json.JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",0))).put("newText","import\n")))
  val draft=planCompletion("hick:///a.py",item,SourceSnapshot("😀 old\n",true),3,6,CompletionExpansion("foo FOO",CompletionSelection(0,3),fields));val after=draft.plan.patches.first().after;assertEquals("import\n😀 foo FOO\n",after);assertEquals(14,draft.fields[1].from);assertSame(transform,draft.fields[1].transform);val session=beginSnippet(after,draft.fields);assertNotNull(session);assertEquals("import\n😀 ßa SSA\n",editSnippet(session!!,after,draft.selection.from,draft.selection.to,"ßa")!!.session.source)
 }
 @Test fun sharedSourceImportSelectionAndRefusalVectors() {
  val cases=JSONObject(javaClass.classLoader!!.getResource("completion-plan-cases.json")!!.readText()).getJSONArray("cases")
  for(index in 0 until cases.length()) {
   val item=cases.getJSONObject(index);val name=item.getString("name");val source=item.getString("source")
   val expanded=item.optJSONObject("expansion");val selected=expanded?.optJSONObject("selection");val expansion=expanded?.let{CompletionExpansion(it.getString("text"),selected?.let{CompletionSelection(it.getInt("from"),it.getInt("to"))})}
   val value=runCatching{planCompletion(item.optString("uri","hick:///a.py"),item.getJSONObject("item"),SourceSnapshot(source,item.optBoolean("writable",true)),item.getInt("from"),item.getInt("to"),expansion)}
   if(item.optBoolean("error")){assertTrue(name,value.isFailure);continue}
   val draft=value.getOrThrow();val selection=item.getJSONObject("selection");assertEquals(name,source,draft.before);assertEquals(name,item.getString("after"),draft.plan.patches.firstOrNull()?.after ?: source);assertEquals(name,selection.getInt("from"),draft.selection.from);assertEquals(name,selection.getInt("to"),draft.selection.to);assertEquals(name,item.getInt("insertionFrom"),draft.insertionFrom)
   if(item.has("patchCount"))assertEquals(name,item.getInt("patchCount"),draft.plan.patches.size)
   draft.plan.patches.firstOrNull()?.let{patch->val inverse=invertChangePlan(draft.plan);assertEquals(name,source,inverse.patches.first().after);assertNull(name,validateChangePlan(inverse,mapOf("a.py" to SourceSnapshot(patch.after,true))))}
  }
 }
 @Test fun ordinaryGroupChecksExactImplicitPrefixesAndRetainsEveryMappedCaret() {
  val uri="hick:///a.py"
  fun item()=JSONObject().put("label","value").put("insertText","value")
  fun group(source:String,secondFrom:Int,secondTo:Int,secondPrefix:Int?)=planCompletionGroup(uri,SourceSnapshot(source,true),listOf(
   CapturedAndroidCompletion(item(),2,2,0),CapturedAndroidCompletion(item(),secondFrom,secondTo,secondPrefix)))
  val accepted=group("va\nva\n",5,5,3)
  assertEquals("value\nvalue\n",accepted.plan.patches.single().after)
  assertEquals(listOf(5,11),accepted.carets)
  assertTrue(runCatching{group("va\nvo\n",5,5,3)}.isFailure)
  assertTrue(runCatching{group("va\nva\n",3,5,null)}.isFailure)
  assertTrue(runCatching{group("va\nva\n",2,2,0)}.isFailure)
 }

}
