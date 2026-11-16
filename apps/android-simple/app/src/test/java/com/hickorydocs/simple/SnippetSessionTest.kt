package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
class SnippetSessionTest {
 private fun fields(values:JSONArray)=(0 until values.length()).map{index->val value=values.getJSONObject(index);val choices=value.getJSONArray("choices");SnippetField(value.getInt("index"),value.getInt("from"),value.getInt("to"),(0 until choices.length()).map{choices.getString(it)},if(value.isNull("id"))null else value.getInt("id"),if(value.isNull("parent"))null else value.getInt("parent"))}
 @Test fun sharedLinkedEditsScopeAndTraversal() {
  val cases=JSONObject(javaClass.classLoader!!.getResource("snippet-session-cases.json")!!.readText()).getJSONArray("cases")
  for(index in 0 until cases.length()){
   val value=cases.getJSONObject(index);val name=value.getString("name");var source=value.getString("source");var session=beginSnippet(source,fields(value.getJSONArray("fields")))
   if(value.optBoolean("invalidBegin")){assertNull(name,session);continue};assertNotNull(name,session)
   val steps=value.getJSONArray("steps");for(offset in 0 until steps.length()){
    val step=steps.getJSONObject(offset);val current=session!!;val input=step.optString("currentSource",source);val from=step.getInt("from");val to=step.getInt("to");val action=step.getString("action")
    val selected:CompletionSelection;val next:SnippetSession?
    if(action=="edit"||action=="complete"||action=="nest"){
     val additions=step.optJSONArray("additional") ?: JSONArray();val additional=(0 until additions.length()).map{val edit=additions.getJSONObject(it);SourceReplacement(edit.getInt("from"),edit.getInt("to"),edit.getString("text"))}
     val result=if(action=="nest")nestSnippet(current,input,SourceReplacement(from,to,step.optString("text","")),additional,fields(step.optJSONArray("inner") ?: JSONArray()))else if(action=="complete")completeSnippet(current,input,SourceReplacement(from,to,step.optString("text","")),additional)else editSnippet(current,input,from,to,step.optString("text","").repeat(step.optInt("repeat",1)))
     if(step.optBoolean("refuse")){assertNull(name,result);continue};assertNotNull(name,result);assertEquals(name,step.getString("after"),result!!.session.source);source=result.session.source;selected=result.selection;next=result.session
    }else{
     val result=advanceSnippet(current,input,from,to,if(action=="next")1 else -1)
     if(step.optBoolean("refuse")){assertNull(name,result);continue};assertNotNull(name,result);selected=result!!.selection;next=result.session
    }
    session=next;val expected=step.getJSONObject("selection");assertEquals(name,expected.getInt("from"),selected.from);assertEquals(name,expected.getInt("to"),selected.to)
    if(step.has("fields"))assertEquals(name,fields(step.getJSONArray("fields")),next!!.fields)
    if(step.has("active"))assertEquals(name,if(step.isNull("active"))null else step.getInt("active"),next?.active)
   }
  }
 }
 @Test fun nestedEmptyChoiceAmplificationIsRefusedBeforeInnerClone(){
  val source="aaaaaaaa";val outer=(0 until 8).map{SnippetField(1,it,it+1,emptyList(),it,null)}+SnippetField(0,8,8,emptyList(),8,null)
  val session=beginSnippet(source,outer)!!;var reads=0
  val emptyChoices=object:AbstractList<String>(){override val size=4096;override fun get(index:Int):String{reads++;return ""}}
  val inner=listOf(SnippetField(1,0,0,emptyChoices,0,null),SnippetField(0,0,0,emptyList(),1,null))
  assertNull(nestSnippet(session,source,SourceReplacement(0,1,""),emptyList(),inner));assertEquals(4096,reads)
  assertNull(beginSnippet("",listOf(SnippetField(1,0,0,List(4097){""},0,null),SnippetField(0,0,0,emptyList(),1,null))))
  assertNotNull(beginSnippet("",listOf(SnippetField(1,0,0,List(4096){""},0,null),SnippetField(0,0,0,emptyList(),1,null))))
 }

}
