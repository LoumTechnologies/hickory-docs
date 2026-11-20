package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
import androidx.compose.ui.text.TextRange
class CompletionAcceptanceTest {
 private fun edit(line:Int,from:Int,to:Int,text:String)=JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",line).put("character",from)).put("end",JSONObject().put("line",line).put("character",to))).put("newText",text)
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun completeImportAndCaretAreOneUndo()=runBlocking {
  val source="header\nva\n";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(9)}
  try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(7,9,"value"),source,buffer.state.selection,JSONObject().put("label","value").put("data",JSONObject().put("hickory_completion","held")))
   assertTrue(service.accept(item){JSONObject().put("label","value").put("textEdit",edit(1,0,2,"value")).put("additionalTextEdits",JSONArray().put(edit(0,0,6,"longheader")))})
   assertEquals("longheader\nvalue\n",buffer.text);assertEquals(16,buffer.state.selection.start);assertEquals(source,buffer.base);assertEquals("base",buffer.hash)
   buffer.state.undoState.undo();assertEquals(source,buffer.text);buffer.state.undoState.redo();assertEquals("longheader\nvalue\n",buffer.text);assertEquals(16,buffer.state.selection.start)
  }finally{service.close()}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun snippetFieldsAndImportsAreOneUndo()=runBlocking {
  val source="va";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val resolved=JSONObject(javaClass.classLoader!!.getResource("snippet-expansion-cases.json")!!.readText()).getJSONArray("cases").getJSONObject(0).getJSONObject("item")
  try{val item=NativeLanguageCompletion("snippet","",NativeSourceEdit(0,2,""),source,buffer.state.selection,resolved)
   assertTrue(service.accept(item){resolved});assertEquals("#\n😀 + 😀",buffer.text);assertEquals(TextRange(2,4),buffer.state.selection);assertEquals(source,buffer.base)
   buffer.state.undoState.undo();assertEquals(source,buffer.text);buffer.state.undoState.redo();assertEquals(TextRange(2,4),buffer.state.selection)
  }finally{service.close()}
 }
 @Test fun newerChoiceAndDismissalRetireHeldResolution()=runBlocking {
  val source="va\n";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val held=mutableMapOf<String,CompletableDeferred<JSONObject>>()
  val resolve:suspend (JSONObject)->JSONObject={item->val promise=CompletableDeferred<JSONObject>();held[item.getString("label")]=promise;promise.await()}
  fun item(label:String)=NativeLanguageCompletion(label,"",NativeSourceEdit(0,2,label),source,TextRange(2),JSONObject().put("label",label).put("data",JSONObject().put("hickory_completion",label)))
  fun result(label:String)=JSONObject().put("label",label).put("textEdit",edit(0,0,2,label)).put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# import\n")))
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  try{val first=async{service.accept(item("old"),resolve)};settle{held["old"]!=null};val second=async{service.accept(item("new"),resolve)};settle{held["new"]!=null}
   held.getValue("new").complete(result("new"));assertTrue(second.await());val notice=service.completionNotice;held.getValue("old").complete(result("old"));assertFalse(first.await());assertEquals("# import\nnew\n",buffer.text);assertEquals(notice,service.completionNotice)
   buffer.state.edit{replace(0,length,source);selection=TextRange(2)};val pending=async{service.accept(item("dismissed"),resolve)};settle{held["dismissed"]!=null};service.cancelCompletion();held.getValue("dismissed").complete(result("dismissed"));assertFalse(pending.await());assertEquals(source,buffer.text)
  }finally{service.close()}
 }
 private fun groupOffer(source:String,token:String)=JSONObject().put("label","value").put("kind",6).put("detail","fixture value")
  .put("data",JSONObject().put("hickory_completion",token).put("hickory_context",JSONObject()
   .put("source_hash","sha256:"+debugHash(source)).put("source_revision","a".repeat(64))))
 private fun groupItem(buffer:DocumentBuffer)=NativeLanguageCompletion("value","fixture value",NativeSourceEdit(0,2,"value"),
  buffer.text,buffer.state.selection,groupOffer(buffer.text,"primary"),buffer.sourceRevision,
  NativeCompletionGroupCapture(buffer.multiRanges,buffer.multiSelectionRevision,buffer.file,0,buffer.snippets.intent))
 private fun resolveGroupOffer(offer:JSONObject,importText:String="# import\n"):JSONObject {
  val data=offer.getJSONObject("data")
  val expected=data.getJSONObject("hickory_context").optJSONObject("expected")
  if(expected!=null)data.put("hickory_context",expected)
  return offer.put("textEdit",edit(if(offer.getJSONObject("data").getString("hickory_completion")=="primary")0 else 1,0,2,"value"))
   .put("additionalTextEdits",JSONArray().put(edit(0,0,0,importText)))
 }
 private fun roster(vararg pairs:Pair<Int,Int>)=pairs.map { (from,to)->NativeSourceEdit(from,to,"") }

 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun groupAdoptsTwoDistinctProofsAndRestoresBothCaretsWithOneHistoryStep()=runBlocking {
  val source="va\nva\n";val before=roster(0 to 2,3 to 5)
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
  val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val originalEditor=buffer.state;val tokens=mutableListOf<String>()
  try {
   assertTrue(buffer.selectMultiple(before,source))
   assertTrue(service.accept(groupItem(buffer),resolution={offer->
    tokens+=offer.getJSONObject("data").getString("hickory_completion")
    assertEquals(source,buffer.text)
    resolveGroupOffer(offer)
   },query={position->
    assertEquals(5,position)
    listOf(groupOffer(source,"unrelated").put("detail","another match"),groupOffer(source,"peer"))
   }))
   assertEquals(listOf("primary","peer"),tokens)
   assertEquals("# import\nvalue\nvalue\n",buffer.text)
   val after=roster(14 to 14,20 to 20)
   assertEquals(after,buffer.multiRanges);assertEquals(TextRange(14),buffer.state.selection)
   assertTrue(buffer.snippets.undoSource())
   assertEquals(source,buffer.text);assertEquals(before,buffer.multiRanges);assertEquals(TextRange(0,2),buffer.state.selection)
   assertFalse(buffer.snippets.canUndo)
   assertTrue(buffer.snippets.redoSource())
   assertEquals("# import\nvalue\nvalue\n",buffer.text);assertEquals(after,buffer.multiRanges)
   assertEquals(TextRange(14),buffer.state.selection);assertSame(originalEditor,buffer.state)
   assertEquals(source,buffer.base);assertEquals("base",buffer.hash)
  } finally { service.close() }
 }

 @Test fun groupRefusesAmbiguousReusedAndRetiredProofsAndConflictingEdits()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  for (failure in listOf("ambiguous","reused","roster round trip","imports","wrong range","snippet","composition","revision")) {
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
   try {
    assertTrue(buffer.selectMultiple(initial,source))
    val accepted=service.accept(groupItem(buffer),resolution={offer->
     val token=offer.getJSONObject("data").getString("hickory_completion")
     val output=resolveGroupOffer(offer,if(token=="peer"&&failure=="imports")"# different\n" else "# import\n")
     if(token=="peer"&&failure=="wrong range")output.put("textEdit",edit(1,1,2,"value"))
     if(token=="peer"&&failure=="snippet")output.put("insertTextFormat",2)
     if(token=="peer"&&failure=="composition")buffer.snippets.enteringMarkedEvent()
     output
    },query={
     val peer=groupOffer(source,if(failure=="reused")"primary" else "peer")
     if(failure=="roster round trip"){
      assertTrue(buffer.selectMultiple(roster(0 to 2,4 to 5),source))
      assertTrue(buffer.selectMultiple(initial,source))
     }
     if(failure=="revision")peer.getJSONObject("data").getJSONObject("hickory_context").put("source_revision","b".repeat(64))
     if(failure=="ambiguous")listOf(peer,groupOffer(source,"ambiguous")) else listOf(peer)
    })
    assertFalse(failure,accepted);assertEquals(failure,source,buffer.text)
    assertEquals(failure,initial,buffer.multiRanges);assertEquals(failure,TextRange(0,2),buffer.state.selection)
    assertFalse(failure,buffer.snippets.canUndo)
   } finally {
    if(buffer.snippets.inMarkedInput)buffer.snippets.leavingMarkedEvent()
    service.close()
   }
  }
 }

 private val sharedSnippetSyntax="${'$'}{1:x} + ${'$'}1 / ${'$'}CURSOR_NUMBER / ${'$'}CLIPBOARD${'$'}0"

 private fun groupSnippetReply(source:String,token:String,index:Int,clip:String?):JSONObject {
  val indent=" ".repeat(index)
  val syntax=indent+sharedSnippetSyntax
  val item=groupOffer(source,token).put("insertTextFormat",2)
   .put("textEdit",edit(index,0,2,syntax))
   .put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# import\n")))
  val data=item.getJSONObject("data")
  if(clip==null){
   data.put("hickory_context_request",JSONObject().put("version",1).put("variables",JSONArray().put("CLIPBOARD")))
   return item
  }
  val expanded=indent+"x + x / ${index+1} / $clip"
  fun field(index:Int,from:Int,to:Int,id:Int)=JSONObject().put("index",index).put("from",from)
   .put("to",to).put("id",id).put("parent",JSONObject.NULL).put("choices",JSONArray())
  data.put("hickory_snippet",JSONObject().put("version",1).put("syntax",syntax)
   .put("text",expanded).put("fields",JSONArray().put(field(1,index,index+1,0)).put(field(1,index+4,index+5,1))
    .put(field(0,expanded.length,expanded.length,2))))
  return item
 }

 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun groupedLazySnippetsShareOneContextAndClipboardWithExactNativeHistory()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
  val engine=Engine("http://127.0.0.1:1")
  var clipboardReads=0
  engine.snippetClipboardReader={clipboardReads++;"clipboard"}
  val service=NativeLanguage(engine,buffer,this);buffer.language=service
  val captured=mutableListOf<JSONObject>()
  try{
   assertTrue(buffer.selectMultiple(initial,source))
   assertTrue(service.accept(groupItem(buffer),query={ offset ->
    assertEquals(5,offset)
    assertEquals("Resolve the current offer before querying the next caret",1,captured.size)
    listOf(groupOffer(source,"peer"))
   },resolution={item->
    assertEquals(source,buffer.text)
    val data=item.getJSONObject("data");val ctx=data.getJSONObject("hickory_context")
    captured.add(ctx)
    val ordinal=ctx.getJSONObject("carets")
    val index=ordinal.getInt("index")
    val selection=ctx.getJSONObject("selection")
    assertEquals(2,ordinal.getInt("count"))
    assertEquals(initial[index].from,selection.getInt("from"));assertEquals(initial[index].to,selection.getInt("to"))
    val token=data.getString("hickory_completion")
    val expected=if(index==0)"primary" else "peer"
    if(ctx.isNull("clipboard")){
     assertEquals(0,clipboardReads);assertEquals(expected,token)
     groupSnippetReply(source,"$expected-negotiation",index,null)
    }else{
     assertEquals(1,clipboardReads);assertEquals("$expected-negotiation",token)
     assertTrue(ctx.getJSONObject("clipboard").getBoolean("available"))
     groupSnippetReply(source,token,index,ctx.getJSONObject("clipboard").getString("text"))
    }
   }))
   assertEquals(1,clipboardReads)
   assertEquals(listOf(0,1,0,1),captured.map {it.getJSONObject("carets").getInt("index")})
   assertEquals(1,captured.map {it.getJSONObject("clock").toString()}.distinct().size)
   assertEquals(1,captured.map {it.getString("random_hex")}.distinct().size)
   val draft="# import\nx + x / 1 / clipboard\n x + x / 2 / clipboard\n"
   assertEquals(draft,buffer.text)
   val next=draft.indexOf('\n',9)+1
   val active=roster(9 to 10,next+1 to next+2)
   assertEquals(active,buffer.multiRanges);assertNotNull(buffer.snippets.parallel)
   assertTrue(buffer.snippets.inputParallel("done"))
   assertEquals("# import\ndone + done / 1 / clipboard\n done + done / 2 / clipboard\n",buffer.text)
   assertTrue(buffer.snippets.undoSource());assertEquals(draft,buffer.text);assertEquals(active,buffer.multiRanges)
   assertTrue(buffer.snippets.undoSource());assertEquals(source,buffer.text);assertEquals(initial,buffer.multiRanges)
   assertNull(buffer.snippets.parallel);assertFalse(buffer.snippets.canUndo)
   assertTrue(buffer.snippets.redoSource());assertEquals(draft,buffer.text);assertEquals(active,buffer.multiRanges)
   assertNotNull(buffer.snippets.parallel);assertEquals(source,buffer.base)
  }finally{service.close()}
 }

 @Test fun groupedClipboardStaysUnreadIfAnyFirstReplyCannotEnterTheGroup()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  for(failure in listOf("mixed","reused","malformed","conflicting imports","overlap")){
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val engine=Engine("http://127.0.0.1:1")
   var clipboardReads=0
   engine.snippetClipboardReader={clipboardReads++;"private"}
   val service=NativeLanguage(engine,buffer,this);buffer.language=service
   try{
    assertTrue(buffer.selectMultiple(initial,source))
    assertFalse(service.accept(groupItem(buffer),query={ listOf(groupOffer(source,"peer")) },resolution={item->
     val data=item.getJSONObject("data")
     val token=data.getString("hickory_completion")
     if(token=="primary")groupSnippetReply(source,"rotated",0,null).apply {
      if(failure=="overlap")put("additionalTextEdits",JSONArray().put(edit(1,0,2,"replacement")))
     }
     else when(failure){
      "mixed" -> resolveGroupOffer(groupOffer(source,token))
      "reused" -> groupSnippetReply(source,"rotated",1,null)
      "conflicting imports" -> groupSnippetReply(source,"different",1,null).put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# other\n")))
      "overlap" -> groupSnippetReply(source,"different",1,null).put("additionalTextEdits",JSONArray().put(edit(1,0,2,"replacement")))
      else -> groupSnippetReply(source,"different",1,null).apply {
       getJSONObject("data").getJSONObject("hickory_context_request").put("variables",JSONArray().put("wrong"))
      }
     }
    }))
    assertEquals(failure,0,clipboardReads);assertEquals(failure,source,buffer.text)
    assertEquals(failure,initial,buffer.multiRanges);assertNull(buffer.snippets.parallel)
    assertFalse(failure,buffer.snippets.canUndo)
   } finally {service.close()}
  }
 }

 @Test fun groupedClipboardRetryMustPreserveEachProjectedEditRole()=runBlocking {
  val source="va!\nva!\n";val initial=roster(0 to 2,4 to 6)
  for(failure in listOf("syntax","range","imports")){
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val engine=Engine("http://127.0.0.1:1")
   var clipboardReads=0
   engine.snippetClipboardReader={clipboardReads++;"clipboard"}
   val service=NativeLanguage(engine,buffer,this);buffer.language=service
   try{
    assertTrue(buffer.selectMultiple(initial,source))
    assertFalse(service.accept(groupItem(buffer),query={listOf(groupOffer(source,"peer"))},resolution={item->
     val data=item.getJSONObject("data");val ctx=data.getJSONObject("hickory_context")
     val index=ctx.getJSONObject("carets").getInt("index")
     val token=data.getString("hickory_completion")
     if(ctx.isNull("clipboard"))groupSnippetReply(source,"$token-rotation",index,null)
     else groupSnippetReply(source,token,index,"clipboard").apply {
      val textEdit=getJSONObject("textEdit")
      when(failure){
       "syntax" -> if(index==0){
        val syntax=textEdit.getString("newText")+"!"
        textEdit.put("newText",syntax)
        getJSONObject("data").getJSONObject("hickory_snippet").put("syntax",syntax)
       }
       "range" -> if(index==0)textEdit.getJSONObject("range").getJSONObject("end").put("character",3)
       "imports" -> put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# replaced\n")))
      }
     }
    }))
    assertTrue(service.completionNotice,service.completionNotice.contains("edits changed"))
    assertEquals(failure,1,clipboardReads);assertEquals(failure,source,buffer.text)
    assertEquals(failure,initial,buffer.multiRanges);assertNull(buffer.snippets.parallel)
    assertFalse(failure,buffer.snippets.canUndo)
   }finally{service.close()}
  }
 }

 @Test fun sourceEqualGroupSelectsCaretsOrFieldsWithoutTextHistory()=runBlocking {
  val source="va\nva\n";val initial=roster(0 to 2,3 to 5)
  for(snippet in listOf(false,true)){
   val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"))
   val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
   try{
    assertTrue(buffer.selectMultiple(initial,source))
    assertTrue(service.accept(groupItem(buffer),query={listOf(groupOffer(source,"peer"))},resolution={item->
     val token=item.getJSONObject("data").getString("hickory_completion")
     val row=if(token=="primary")0 else 1
     val syntax=if(snippet)"${'$'}{1:va}${'$'}0" else "va"
     val result=groupOffer(source,token).put("textEdit",edit(row,0,2,syntax))
     if(snippet){
      fun field(index:Int,from:Int,to:Int,id:Int)=JSONObject().put("index",index).put("from",from)
       .put("to",to).put("id",id).put("parent",JSONObject.NULL).put("choices",JSONArray())
      result.put("insertTextFormat",2).getJSONObject("data").put("hickory_snippet",
       JSONObject().put("version",1).put("syntax",syntax).put("text","va")
        .put("fields",JSONArray().put(field(1,0,2,0)).put(field(0,2,2,1))))
     }
     result
    }))
    assertEquals(source,buffer.text);assertEquals("Completion selected",service.completionNotice)
    assertEquals(if(snippet)initial else roster(2 to 2,5 to 5),buffer.multiRanges)
    assertEquals(snippet,buffer.snippets.parallel!=null);assertFalse(buffer.snippets.canUndo)
    assertFalse(buffer.snippets.canRedo)
   }finally{service.close()}
  }
 }

}
