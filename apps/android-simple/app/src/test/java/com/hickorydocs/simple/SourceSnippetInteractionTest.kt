package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.async
class SourceSnippetInteractionTest {
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun computedInputAndNativeHistoryAreOneAction(){
  val root=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("apps/simple-contract/snippet-derived-cases.json").isFile};val transform=readSnippetTransform(org.json.JSONObject(root.resolve("apps/simple-contract/snippet-derived-cases.json").readText()).getJSONObject("programs").getJSONObject("upper"));val source="foo FOO";val values=listOf(SnippetField(1,0,3,emptyList(),0),SnippetField(1,4,7,emptyList(),1,transform=transform),SnippetField(0,7,7,emptyList(),2));val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));buffer.state.edit{selection=TextRange(0,3)};buffer.snippets.restore(beginSnippet(source,values));assertEquals(3*64L+transform.bytes,snippetHistoryCost(buffer.snippets.active));assertTrue(buffer.snippets.applyInput(source,0,3,"λ😀"));assertEquals("λ😀 Λ😀",buffer.text);assertEquals(TextRange(3),buffer.state.selection);assertNotNull(buffer.snippets.active);assertTrue(buffer.snippets.undoSource());assertEquals(source,buffer.text);assertNotNull(buffer.snippets.active);assertTrue(buffer.snippets.redoSource());assertEquals("λ😀 Λ😀",buffer.text);assertEquals(TextRange(3),buffer.state.selection)
  var staged:SnippetSession?=buffer.snippets.active;buffer.state.edit{val before=buffer.snippets.captureBefore(originalText,originalSelection);replace(4,7,"bar");selection=TextRange(7);assertFalse(applySourceSnippetInput(buffer,publish={staged=it}));buffer.snippets.finishTextChange(before,asCharSequence(),selection)};buffer.snippets.publish(staged);assertEquals("λ😀 bar",buffer.text);assertNull(buffer.snippets.active)
 }

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
 @Test fun providerCompletionCombinesImportsAndRefusesInteriorEdits()=kotlinx.coroutines.runBlocking {
  val buffer=buffer();val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val item=NativeLanguageCompletion("long","",NativeSourceEdit(0,1,"long"),"a + a",buffer.state.selection,org.json.JSONObject().put("label","long"))
  val result=org.json.JSONObject("{\"label\":\"long\",\"textEdit\":{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":1}},\"newText\":\"long\"}}")
  try{assertTrue(service.accept(item){result});assertEquals("long + long",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
   buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source)
   val additions=org.json.JSONArray().put(org.json.JSONObject().put("range",org.json.JSONObject().put("start",org.json.JSONObject().put("line",0).put("character",0)).put("end",org.json.JSONObject().put("line",0).put("character",0))).put("newText","# import\n")).put(org.json.JSONObject().put("range",org.json.JSONObject().put("start",org.json.JSONObject().put("line",0).put("character",5)).put("end",org.json.JSONObject().put("line",0).put("character",5))).put("newText","\n# end"))
   assertTrue(service.accept(item){org.json.JSONObject(result.toString()).put("additionalTextEdits",additions)});assertEquals("# import\nlong + long\n# end",buffer.text);assertEquals(TextRange(13),buffer.state.selection);assertEquals(listOf(9,16,20),buffer.snippets.active?.fields?.map{it.from})
   buffer.snippets.undoSource();assertEquals("a + a",buffer.text);assertEquals(buffer.text,buffer.snippets.active?.source);buffer.snippets.redoSource();assertEquals(TextRange(13),buffer.state.selection);buffer.snippets.undoSource()
   val refusal=org.json.JSONObject(result.toString()).put("additionalTextEdits",org.json.JSONArray("[{\"range\":{\"start\":{\"line\":0,\"character\":2},\"end\":{\"line\":0,\"character\":2}},\"newText\":\"\\nimport other\"}]"))
   assertFalse(service.accept(item){refusal});assertEquals("a + a",buffer.text);assertTrue(service.completionNotice.contains("overlaps template"))
  }finally{service.close()}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun nestedCompletionRestoresScopeThroughNativeHistory()=kotlinx.coroutines.runBlocking {
  val buffer=buffer();val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service
  val syntax="\${1:x}:\$1\$0";val raw=org.json.JSONObject("""{"label":"nested","insertTextFormat":2,"textEdit":{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"placeholder"},"data":{"hickory_snippet":{"version":1,"syntax":"placeholder","text":"x:x","fields":[{"id":0,"parent":null,"index":1,"from":0,"to":1,"choices":[]},{"id":1,"parent":null,"index":1,"from":2,"to":3,"choices":[]},{"id":2,"parent":null,"index":0,"from":3,"to":3,"choices":[]}]}}}""")
  raw.getJSONObject("textEdit").put("newText",syntax);raw.getJSONObject("data").getJSONObject("hickory_snippet").put("syntax",syntax)
  val item=NativeLanguageCompletion("nested","",NativeSourceEdit(0,1,syntax),"a + a",buffer.state.selection,raw)
  try{assertTrue(service.accept(item){raw});assertEquals("x:x + x:x",buffer.text);assertEquals(1,buffer.snippets.active?.scopes?.size)
   assertTrue(buffer.snippets.applyInput(buffer.text,0,1,"long"));assertEquals("long:long + long:long",buffer.text)
   buffer.snippets.undoSource();assertEquals(1,buffer.snippets.active?.scopes?.size);buffer.snippets.redoSource();assertEquals(1,buffer.snippets.active?.scopes?.size);assertEquals(TextRange(4),buffer.state.selection)
   assertTrue(buffer.snippets.move(1));assertEquals(TextRange(9),buffer.state.selection);assertEquals(1,buffer.snippets.active?.active);assertTrue(buffer.snippets.move(1));assertEquals(TextRange(21),buffer.state.selection);assertNull(buffer.snippets.active)
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
