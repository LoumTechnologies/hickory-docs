package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.async
class SourceSnippetInteractionTest {
 private fun fields()=listOf(SnippetField(1,0,1,listOf("a","long"),0),SnippetField(1,4,5,listOf("a","long"),1),SnippetField(0,5,5,emptyList(),2))
 private fun buffer()=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("a + a","base")).also{it.state.edit{selection=TextRange(0,1)};it.snippets.restore(beginSnippet(it.text,fields()))}
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun actualTextStateUsesTheProductionInputTransformation() {
  val buffer=buffer();var staged:SnippetSession?=null
  buffer.state.edit{val before=buffer.snippets.captureBefore(originalText,originalSelection);replace(0,1,"long");selection=TextRange(4);assertTrue(applySourceSnippetInput(buffer,publish={staged=it}));buffer.snippets.finishTextChange(before,asCharSequence(),selection)};buffer.snippets.publish(staged)
  assertEquals("long + long",buffer.text);assertEquals(TextRange(4),buffer.state.selection);assertEquals(buffer.text,buffer.snippets.active?.source);assertEquals("a + a",buffer.base)
  buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source);buffer.snippets.redoSource();assertEquals("long + long",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
  assertTrue(buffer.snippets.move(-1));assertNotNull(buffer.snippets.active);assertTrue(buffer.snippets.move(1));assertEquals(TextRange(11),buffer.state.selection);assertNull(buffer.snippets.active)
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun preparedNativeEnterPreservesLinkedTextAndCaret(){
  val source="call() + call()";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));buffer.state.edit{selection=TextRange(5)}
  buffer.snippets.restore(beginSnippet(source,listOf(SnippetField(1,0,6,emptyList(),0),SnippetField(1,9,15,emptyList(),1),SnippetField(0,15,15,emptyList(),2))));var staged:SnippetSession?=null
  buffer.state.edit{val before=buffer.snippets.captureBefore(originalText,originalSelection);replace(5,5,"\n");selection=TextRange(6);assertTrue(applySourceSnippetInput(buffer,prepare={_,_,_,_->SnippetPrimaryEdit(NativeSourceEdit(5,5,"\n  \n"),8)},publish={staged=it}));buffer.snippets.finishTextChange(before,asCharSequence(),selection)};buffer.snippets.publish(staged)
  assertEquals("call(\n  \n) + call(\n  \n)",buffer.text);assertEquals(TextRange(8),buffer.state.selection);buffer.snippets.undoSource();assertEquals(source,buffer.text);buffer.snippets.redoSource();assertEquals(TextRange(8),buffer.state.selection)
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun providerCompletionPreservesMirrorsAndRefusesImportSideEffects()=kotlinx.coroutines.runBlocking {
  val buffer=buffer();val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val item=NativeLanguageCompletion("long","",NativeSourceEdit(0,1,"long"),"a + a",buffer.state.selection,org.json.JSONObject().put("label","long"))
  val result=org.json.JSONObject("{\"label\":\"long\",\"textEdit\":{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":1}},\"newText\":\"long\"}}")
  try{assertTrue(service.accept(item){result});assertEquals("long + long",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
   buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
   val refusal=org.json.JSONObject(result.toString()).put("additionalTextEdits",org.json.JSONArray("[{\"range\":{\"start\":{\"line\":0,\"character\":5},\"end\":{\"line\":0,\"character\":5}},\"newText\":\"\\nimport other\"}]"))
   assertFalse(service.accept(item){refusal});assertEquals("a + a",buffer.text);assertTrue(service.completionNotice.contains("Finish the current template"))
  }finally{service.close()}
 }
 @Test fun equalPositionNavigationRetiresHeldCompletion()=kotlinx.coroutines.runBlocking {
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("a","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(0,1)}
  buffer.snippets.restore(beginSnippet("a",listOf(SnippetField(2,0,1,emptyList(),1,0),SnippetField(1,0,1,emptyList(),0),SnippetField(0,1,1,emptyList(),2))))
  val entered=kotlinx.coroutines.CompletableDeferred<Unit>();val held=kotlinx.coroutines.CompletableDeferred<org.json.JSONObject>();val selected=buffer.state.selection
  val work=kotlinx.coroutines.CoroutineScope(coroutineContext).async{service.accept(NativeLanguageCompletion("long","",NativeSourceEdit(0,1,"long"),"a",selected,org.json.JSONObject().put("label","long"))){entered.complete(Unit);held.await()}}
  try{entered.await();assertTrue(buffer.snippets.move(1));assertEquals(selected,buffer.state.selection);held.complete(org.json.JSONObject().put("label","long").put("insertText","long"));assertFalse(work.await());assertEquals("a",buffer.text)}finally{service.close()}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun sourceLocalChoiceIsOneDraftUndo() {
  val buffer=buffer();assertNull(buffer.language);assertTrue(buffer.snippets.choose("long"));assertEquals("long + long",buffer.text);assertEquals(TextRange(4),buffer.state.selection)
  buffer.snippets.undoSource();assertEquals("a + a",buffer.text);buffer.snippets.redoSource();assertEquals("long + long",buffer.text)
 }
 @Test fun acceptsSourceEqualCompletion()=kotlinx.coroutines.runBlocking {
  val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("va","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val original=org.json.JSONObject().put("label","va");val result=org.json.JSONObject("{\"label\":\"va\",\"textEdit\":{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":2}},\"newText\":\"va\"}}")
  try{assertTrue(service.accept(NativeLanguageCompletion("va","",NativeSourceEdit(0,2,"va"),"va",buffer.state.selection,original)){result});assertEquals("va",buffer.text);assertEquals("va",buffer.base)}finally{service.close()}
 }
 @Test fun unavailableMirroringRetainsOrdinaryInput() {
  for(hidden in listOf(false,true)){val buffer=buffer();buffer.saving=!hidden;var staged:SnippetSession?=null
   buffer.state.edit{replace(0,1,"x");selection=TextRange(1);assertFalse(applySourceSnippetInput(buffer,canReflect={!hidden},publish={staged=it}))};buffer.snippets.publish(staged)
   assertEquals("x + a",buffer.text);assertNull(buffer.snippets.active)
  }
 }
}
