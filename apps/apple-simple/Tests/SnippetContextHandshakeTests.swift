import XCTest
@testable import HickorySimple

@MainActor final class SnippetContextHandshakeTests:XCTestCase {
 private let source="λ😀\r\nword"
 private var proof:[String:Any]{["source_hash":"sha256:"+nativeSourceHash(source),"source_revision":String(repeating:"a",count:64)]}
 private var original:[String:Any]{["label":"field","data":["hickory_completion":"original","hickory_context":proof,"opaque":["provider":["untouched"]]]]}
 private func requested(_ sourceProof:[String:Any]?=nil)->[String:Any]{["label":"field","data":["hickory_completion":"resolved","hickory_context":sourceProof ?? proof,"hickory_context_request":["version":1,"variables":["CLIPBOARD"]]]]}
 func testDemandedClipboardUsesOneCapturedSnapshotWithoutMutatingOriginal()async throws {
  let item=original,originalBytes=try JSONSerialization.data(withJSONObject:item,options:.sortedKeys);var calls=[[String:Any]](),reads=0
  let result=try await resolveSnippetCompletion(item,source:source,from:5,to:9,current:{true},request:{params in calls.append(params);return calls.count==1 ? self.requested():["label":"done","data":["hickory_context":self.proof]]},clipboard:{reads+=1;return "clipboard"})
  XCTAssertEqual(result["label"] as? String,"done");XCTAssertEqual(reads,1);XCTAssertEqual(calls.count,2);XCTAssertEqual(try JSONSerialization.data(withJSONObject:item,options:.sortedKeys),originalBytes)
  let first=(calls[0]["data"] as! [String:Any])["hickory_context"] as! [String:Any],second=(calls[1]["data"] as! [String:Any])["hickory_context"] as! [String:Any]
  XCTAssertEqual(first["random_hex"] as? String,second["random_hex"] as? String)
  XCTAssertEqual(try JSONSerialization.data(withJSONObject:first["clock"]!,options:.sortedKeys),try JSONSerialization.data(withJSONObject:second["clock"]!,options:.sortedKeys))
  XCTAssertTrue(first["clipboard"] is NSNull);XCTAssertEqual((second["clipboard"] as? [String:Any])?["text"] as? String,"clipboard")
 }
 func testForeignProofAndRetiredClipboardCannotCauseSecondProviderRequest()async {
  for foreign in [true,false]{var current=true,calls=0,reads=0
   do{_ = try await resolveSnippetCompletion(original,source:source,from:5,to:9,current:{current},request:{_ in calls+=1;var proof=self.proof;if foreign{proof["source_revision"]=String(repeating:"b",count:64)};return self.requested(proof)},clipboard:{reads+=1;current=false;return "secret"});XCTFail("Stale context was accepted") }catch{}
   XCTAssertEqual(calls,1);XCTAssertEqual(reads,foreign ? 0:1)
  }
 }
}
