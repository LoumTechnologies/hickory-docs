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
