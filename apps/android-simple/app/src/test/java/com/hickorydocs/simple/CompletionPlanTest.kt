package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class CompletionPlanTest {
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
}
