<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep native rename and formatting tied to prepared source

These focused witnesses use the owned channel peer and the real native WebSocket transport. The peer grants no edit authority: it returns an inert action envelope and echoes the exact preparation stamps that it observed. Tests prove that rename and formatting use the custom guarded methods, retain the full envelope, and refuse equal-byte edit/Undo or detached-provider replies. Complete workspace proof admission and actual provider semantics are exercised separately by the workspace review witnesses.

<ui:copy id="native-bound-proposal-wire-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class BoundProposalWireTests:XCTestCase {
 func testPreparedQueriesRetainEnvelopeAndRejectEditUndoAndClose()async throws {
  #if os(macOS)
  let root=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent(),process=Process(),pipe=Pipe();process.executableURL=URL(fileURLWithPath:"/usr/bin/env");process.arguments=["python3",root.appendingPathComponent("scripts/simple-language-wire-fixture.py").path];process.standardOutput=pipe;process.standardError=pipe;try process.run();defer{if process.isRunning{process.terminate()};process.waitUntilExit()}
  let greeting=try XCTUnwrap(JSONSerialization.jsonObject(with:pipe.fileHandleForReading.availableData) as? [String:Any]),engine=try Engine(address:try XCTUnwrap(greeting["origin"] as? String)),source="value = 'λ'\n",buffer=DocumentBuffer(file:FileNode(name:"bound.py",path:"bound.py",dir:false),text:FileText(path:"bound.py",content:source,hash:"saved")),language=NativeLanguage(engine:engine,buffer:buffer);buffer.language=language;language.start();defer{language.close()}
  struct Call:Decodable{let method:String};struct State:Decodable{let proposals:[Call]}
  func wait(_ count:Int)async throws{for _ in 0..<500{let state:State=try await engine.request("GET","/state");if state.proposals.count>=count{return};try await Task.sleep(for:.milliseconds(10))};throw EngineError(message:"Bound proposal request did not reach the owned peer.")}
  for kind in ["rename","format"]{let raw=try await language.propose(kind,name:"renamed");let result=try XCTUnwrap(raw as? [String:Any]),data=try XCTUnwrap(result["data"] as? [String:Any]),proof=try XCTUnwrap(data["fixture_expected"] as? [String:Any]);XCTAssertEqual(result["source_files_modified"] as? Bool,false);XCTAssertNil(result["disabled"]);XCTAssertEqual(data["fixture_method"] as? String,kind=="rename" ? "hick/rename":"hick/formatting");XCTAssertEqual(data["hickory_action"] as? String,"fixture-only");XCTAssertEqual(proof["source_hash"] as? String,"sha256:"+nativeSourceHash(source));XCTAssertEqual((proof["source_revision"] as? String)?.count,64);XCTAssertNotNil(result["edit"]);XCTAssertEqual(buffer.text,source)}
  let aba=Task{try await language.propose("rename",name:"later")};try await wait(3);buffer.text="# changed\n"+source;buffer.text=source;do{_=try await aba.value;XCTFail("Equal-byte edit/Undo renewed a prepared query")}catch{};XCTAssertEqual(buffer.text,source)
  let closed=Task{try await language.propose("format",name:"")};try await wait(4);language.close();do{_=try await closed.value;XCTFail("Detached provider retained proposal authority")}catch{};XCTAssertEqual(buffer.text,source)
  #else
  throw XCTSkip("Owned process fixture runs on macOS; shared iOS source is compiled separately.")
  #endif
 }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/BoundProposalWireTests.swift"><ui:paste select="#native-bound-proposal-wire-apple-tests"/></ui:file>

<ui:copy id="native-bound-proposal-wire-android-tests">
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
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/BoundProposalWireTest.kt"><ui:paste select="#native-bound-proposal-wire-android-tests"/></ui:file>
</ui:doc>
