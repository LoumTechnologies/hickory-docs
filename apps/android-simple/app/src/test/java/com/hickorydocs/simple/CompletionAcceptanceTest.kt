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
 private fun resolveGroupOffer(offer:JSONObject,importText:String="# import\n"):JSONObject =
  offer.put("textEdit",edit(if(offer.getJSONObject("data").getString("hickory_completion")=="primary")0 else 1,0,2,"value"))
   .put("additionalTextEdits",JSONArray().put(edit(0,0,0,importText)))
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

}
