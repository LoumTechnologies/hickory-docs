package com.hickorydocs.simple
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.runBlocking
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class CompletionIntentTest {
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun offeredCompletionCannotReviveAfterSourceOrSelectionRoundTrip()=runBlocking {
  for(source in listOf(true,false)){val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("va\n","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)};try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(0,2,"value"),buffer.text,buffer.state.selection,JSONObject().put("label","value"),buffer.sourceRevision);roundTrip(buffer,source);var calls=0;val accepted=service.accept(item){calls++;resolved()};assertFalse(accepted);assertEquals(0,calls);assertEquals("va\n",buffer.text);assertEquals(TextRange(2),buffer.state.selection);assertTrue(service.completionNotice.contains("out of date"))}finally{service.close()}}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun pendingCompletionCannotReviveAfterSourceOrSelectionRoundTrip()=runBlocking {
  for(source in listOf(true,false)){val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("va\n","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)};try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(0,2,"value"),buffer.text,buffer.state.selection,JSONObject().put("label","value"),buffer.sourceRevision);val accepted=service.accept(item){roundTrip(buffer,source);resolved()};assertFalse(accepted);assertEquals("va\n",buffer.text);assertEquals(TextRange(2),buffer.state.selection);assertTrue(service.completionNotice.contains("changed"))}finally{service.close()}}
 }
 private fun resolved()=JSONObject("""{"label":"value","textEdit":{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":2}},"newText":"value"}}""")
 private fun roundTrip(buffer:DocumentBuffer,source:Boolean){if(source){assertTrue(buffer.applySourceEdits(listOf(NativeSourceEdit(0,2,"xy")),buffer.text));assertTrue(buffer.snippets.undoSource())}else{assertTrue(buffer.selectSourceRange(0,0,buffer.text));assertTrue(buffer.selectSourceRange(2,2,buffer.text))}}
}
