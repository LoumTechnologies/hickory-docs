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
}
