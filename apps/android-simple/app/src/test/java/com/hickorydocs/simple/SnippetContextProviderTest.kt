package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
import androidx.compose.ui.text.TextRange
class SnippetContextProviderTest {
 @Test(timeout=120000) fun liveProviderContextCompletionAndHistory()=kotlinx.coroutines.runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
  val root=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use maintained native template fixture --provider-context")).canonicalFile
  check(root.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n")
  val events=root.parentFile.resolve("provider-events.jsonl")
  fun resolves(path:String)=if(!events.exists())0 else events.readLines().count{runCatching{JSONObject(it).optString("resolve").endsWith("/$path")}.getOrDefault(false)}
  val engine=Engine(origin!!);val (_,tree)=engine.files()
  fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  for(path in listOf("variables.py","denied.py","unused.py")){
   val file=find(path) ?: error("Provider fixture input missing: $path");val saved=engine.file(file);assertEquals("old\r\nsecond\n",saved.content)
   val buffer=DocumentBuffer(file,saved);val state=buffer.state;val scroll=buffer.sourceScroll.value;buffer.selectSourceRange(0,3,buffer.text)
   val language=NativeLanguage(engine,buffer,this);buffer.language=language;language.start();var reads=0;engine.snippetClipboardReader={reads++;if(path=="denied.py")null else "copied"}
   try{
    repeat(500){if(!language.connection.active||language.status!="LSP connected")kotlinx.coroutines.delay(10)};check(language.connection.active&&language.status=="LSP connected"){language.status}
    val before=resolves(path);language.complete();val offered=language.suggestions.singleOrNull{it.label=="context completion"} ?: error(language.status)
    assertTrue(language.completionNotice,language.accept(offered));assertEquals(before+1,resolves(path))
    if(path=="variables.py"){
     val line=buffer.text.substringBefore("\r\n");val parts=line.split('|');assertEquals(7,parts.size);assertEquals(path,parts[0]);assertEquals(java.time.ZonedDateTime.now().year.toString(),parts[1]);assertTrue(parts[2].matches(Regex("[0-9a-f]{6}")));assertEquals(listOf("old","old","OLD","copied"),parts.drop(3));assertEquals(1,reads)
     assertTrue(buffer.snippets.applyInput(buffer.text,state.selection.min,state.selection.max,"new"));val after=parts.take(3).joinToString("|")+"|new|new|NEW|copied\r\nsecond\n";assertEquals(after,buffer.text)
     assertTrue(buffer.snippets.undoSource());assertEquals(line+"\r\nsecond\n",buffer.text);assertTrue(buffer.snippets.undoSource());assertEquals(saved.content,buffer.text);repeat(2){assertTrue(buffer.snippets.redoSource())};assertEquals(after,buffer.text)
    }else if(path=="denied.py"){assertEquals(1,reads);assertEquals("fallback\r\nsecond\n",buffer.text);assertTrue(language.completionNotice.contains("Clipboard unavailable"));assertTrue(buffer.snippets.undoSource());assertEquals(saved.content,buffer.text);assertTrue(buffer.snippets.redoSource());assertEquals("fallback\r\nsecond\n",buffer.text)}
    else{assertEquals(0,reads);assertEquals(saved.content,buffer.text)}
    assertSame(state,buffer.state);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved.content,buffer.base);assertEquals(saved.content,engine.file(file).content)
   }finally{language.close()}
  }
 }
}
