package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
class CoverageBindingTest {
 @Test fun mixedOwnerMarksRetireImmediatelyAfterExactByteEdit()=runBlocking {
  val source="// é\nvalue\n";val path="owner.kt";val hash=java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}
  val record=JSONObject().put("id","measured").put("source_changed_during_run",false).put("identity",JSONObject().put("source_hashes",JSONObject().put(path,hash).put("generated.kt","a".repeat(64))))
   .put("evidence",JSONObject().put("report",JSONObject().put("files",JSONObject().put("generated.kt",JSONObject().put("line_counts",false).put("lines",JSONObject().put("1",1).put("2",0))))))
   .put("mappings",JSONObject().put("generated.kt",JSONObject().put("1",JSONObject().put("path",path).put("line",2)).put("2",JSONObject().put("path",path).put("line",2))))
  val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
  listener.createContext("/api/tests/coverage"){exchange->val bytes=record.toString().toByteArray();exchange.requestBody.close();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};listener.start()
  val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:${listener.address.port}");val state=workspace.coverage;state.digests=SourceDigests(this)
  val buffer=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText(source,hash));buffer.coverageSource=state;workspace.buffers[path]=buffer
  suspend fun settle(condition:()->Boolean){withTimeout(3000){while(!condition()){delay(2)}}}
  try{state.select("measured",workspace);settle{buffer.coverageRows().isNotEmpty()};val row=buffer.coverageRows().first();assertEquals(1,row.line);assertEquals("mixed",row.state);assertNull(row.count);assertEquals(2,row.origins.size)
   buffer.state.edit{replace(0,length,"// e\u0301\nvalue\n")};assertTrue(buffer.coverageRows().isEmpty());buffer.readCoverageSource();settle{state.digests!!.current(path,buffer.text)!=null};assertTrue(buffer.coverageRows().isEmpty())
   buffer.state.edit{replace(0,length,source)};buffer.readCoverageSource();settle{buffer.coverageRows().isNotEmpty()};assertEquals("mixed",buffer.coverageRows().first().state);state.select(null,workspace);assertTrue(buffer.coverageRows().isEmpty())
  }finally{state.stop();listener.stop(0)}
 }
}
