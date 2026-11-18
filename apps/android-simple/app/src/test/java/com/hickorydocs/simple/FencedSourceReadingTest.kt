package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
class FencedSourceReadingTest {
 @Test(timeout=120000) fun liveFencedSourceLanguageReadings()=kotlinx.coroutines.runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
  val root=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use test-fenced-source-readings.py --host android")).canonicalFile
  check(root.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n")
  val rows=JSONObject(root.parentFile.resolve("fixture.json").readText()).getJSONArray("rows");val engine=Engine(origin!!)
  val (_,tree)=engine.files()
  fun find(nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path=="notes.md")return node;find(node.children)?.let{return it}};return null}
  val file=find(tree) ?: error("Missing authored notes.md");val saved=engine.file(file);val buffer=DocumentBuffer(file,saved);val state=buffer.state;val scroll=buffer.sourceScroll.value
  val language=NativeLanguage(engine,buffer,this);buffer.language=language;language.start()
  try{
   repeat(500){if(!language.connection.active||language.status!="LSP connected")kotlinx.coroutines.delay(10)};check(language.connection.active&&language.status=="LSP connected"){language.status}
   for(index in 0 until rows.length()){
    val row=rows.getJSONObject(index);val query=row.getJSONObject("query");val definition=row.getJSONObject("definition");val offset=languageOffset(buffer.text,query) ?: error("Invalid authored query")
    assertTrue(buffer.selectSourceRange(offset,offset,buffer.text));val expectedHover=if(definition.getInt("line")==4)"def greet" else "function greet";for(attempt in 0 until 200){language.inspect();if(language.hover.contains(expectedHover))break;kotlinx.coroutines.delay(50)};assertTrue(language.hover,language.hover.contains(expectedHover));language.navigate(SemanticQuery.Definition)
    val target=language.locations.firstOrNull{it.path=="notes.md"} ?: error(language.locationStatus)
    assertEquals(definition.getInt("line"),target.line);assertEquals(definition.getInt("character"),target.character);assertTrue(language.open(target))
    val destination=languageOffset(buffer.text,definition) ?: error("Invalid authored definition");assertEquals(destination,state.selection.min);assertEquals(destination,state.selection.max)
   }
   assertEquals(saved.content,buffer.text);assertEquals(saved.content,buffer.base);assertSame(state,buffer.state);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved.content,engine.file(file).content)
  }finally{language.close()}
 }
}
