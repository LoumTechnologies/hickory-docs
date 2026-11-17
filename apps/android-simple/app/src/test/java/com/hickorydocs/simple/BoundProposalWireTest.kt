package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import kotlinx.coroutines.*
class BoundProposalWireTest {
 @Test(timeout=20000) fun preparedQueriesRetainEnvelopeAndRejectEditUndoAndClose()=runBlocking {
  val root=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("scripts/simple-language-wire-fixture.py").isFile};val process=ProcessBuilder("python3",root.resolve("scripts/simple-language-wire-fixture.py").path).redirectErrorStream(true).start()
  try{val greeting=JSONObject(process.inputStream.bufferedReader().readLine());val engine=Engine(greeting.getString("origin"));val source="value = 'λ'\n";val buffer=DocumentBuffer(FileNode("bound.py","bound.py",false,emptyList(),null,null),FileText(content=source,hash="saved"));val language=NativeLanguage(engine,buffer,this);buffer.language=language;language.start()
   try{
    suspend fun wait(count:Int){withTimeout(5000){while(engine.request("GET","/state").getJSONArray("proposals").length()<count)delay(10)}}
    for(kind in listOf("rename","format")){val result=language.propose(kind,"renamed") as JSONObject;val data=result.getJSONObject("data");val proof=data.getJSONObject("fixture_expected");assertEquals(false,result.get("source_files_modified"));assertFalse(result.has("disabled"));assertEquals(if(kind=="rename")"hick/rename" else "hick/formatting",data.getString("fixture_method"));assertEquals("fixture-only",data.getString("hickory_action"));assertEquals("sha256:"+debugHash(source),proof.getString("source_hash"));assertEquals(64,proof.getString("source_revision").length);assertNotNull(result.getJSONObject("edit"));assertEquals(source,buffer.text)}
    val aba=async{runCatching{language.propose("rename","later")}};wait(3);assertTrue(buffer.applySourceEdits(listOf(NativeSourceEdit(0,0,"# changed\n")),source));assertTrue(buffer.snippets.undoSource());assertTrue("Equal-byte edit/Undo renewed a query",aba.await().isFailure);assertEquals(source,buffer.text)
    val closed=async{runCatching{language.propose("format","")}};wait(4);language.close();assertTrue("Detached provider retained proposal authority",closed.await().isFailure);assertEquals(source,buffer.text)
   }finally{language.close()}
  }finally{process.destroy();if(!process.waitFor(2,java.util.concurrent.TimeUnit.SECONDS)){process.destroyForcibly();process.waitFor()}}
 }
}
