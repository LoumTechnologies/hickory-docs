import XCTest
@testable import HickorySimple
final class WorkspaceEntrySelectionTests:XCTestCase {
 private func original()->[String:Any]{["version":1,"entry_protocol":1,"workspace":String(repeating:"a",count:32),"namespace":"/workspace","entry":["version":1,"identity":String(repeating:"b",count:32),"generation":7,"path":"project.md"],"guard":String(repeating:"c",count:64),"source_guard":"opaque physical observation","source_hash":"sha256:"+String(repeating:"d",count:64),"movable":true,"source_effects":false]}
 func testOriginalProofDoesNotFollowMutableResponseOrReturnedBody()throws {var response=original();let proof=try WorkspaceEntrySelectionProof(response);response["namespace"]="/other";var body=try XCTUnwrap(proof.body() as? [String:Any]);body["namespace"]="/mutated";XCTAssertEqual(proof.namespace,"/workspace");XCTAssertEqual(proof.entry.path,"project.md");XCTAssertEqual((try proof.body() as? [String:Any])?["namespace"] as? String,"/workspace")}
 func testStrictTypesAndGenerationsRefuseIncompleteAuthority(){for (key,value) in [("unexpected",1 as Any),("source_effects","false" as Any),("source_effects",0 as Any),("movable",1 as Any),("source_guard",NSNull()),("source_hash","wrong" as Any),("guard",String(repeating:"C",count:64) as Any)]{var response=original();response[key]=value;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response))};for generation in [-1,4_294_967_296]{var response=original(),entry=response["entry"] as! [String:Any];entry["generation"]=generation;response["entry"]=entry;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response))}}
 func testPositiveAbsenceCannotBorrowSourceOrMoveAuthority()throws {var response=original(),entry=response["entry"] as! [String:Any];entry["path"]=NSNull();response["entry"]=entry;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response));response.removeValue(forKey:"source_guard");response.removeValue(forKey:"source_hash");response["movable"]=false;XCTAssertNil(try WorkspaceEntrySelectionProof(response).entry.path)}
 func testNoEffectRequiresTheExactRequestAndOriginalAddress() throws {
  let response=original(),before=try XCTUnwrap(response["entry"] as? [String:Any]),request=String(repeating:"a",count:32)
  let after=before.merging(["generation":8,"path":"moved.spec"]){_,new in new}
  let token=String(repeating:"e",count:64)
  let intentBody:[String:Any]=["version":1,"workspace":response["workspace"]!,"namespace":"/workspace","context":token,
    "token":token,"kind":"move","title":"Move","edit":[:],"before_entry":before,"after_entry":after,"source_files_modified":false]
  let intent=try JSONDecoder().decode(WorkspaceEntryIntent.self,from:JSONSerialization.data(withJSONObject:intentBody))
  func outcome(consumed:Any,current:[String:Any])throws->WorkspaceEntryPublication {
   let body:[String:Any]=["version":1,"state":"no_effect","request_id":request,"before_entry":before,"after_entry":after,
      "current_entry":current,"reservation_consumed":consumed]
   return try JSONDecoder().decode(WorkspaceEntryPublication.self,from:JSONSerialization.data(withJSONObject:body))
  }
  XCTAssertTrue(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:false,current:before),request:request,intent:intent))
  let retained=before.merging(["generation":8]){_,new in new}
  XCTAssertTrue(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:true,current:retained),request:request,intent:intent))
  XCTAssertThrowsError(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:true,current:after),request:request,intent:intent))
  XCTAssertThrowsError(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:1,current:retained),request:request,intent:intent))
 }
 @MainActor func testAsynchronousAcquisitionNeverRebindsOriginalLease()async throws {let response=original();let workspace=response["workspace"] as! String;var live=false,reads=0;do{_ = try await WorkspaceEntrySelections.read(load:{reads+=1;return response},workspace:workspace,namespace:"/workspace",current:{live});XCTFail("stale source must refuse before request")}catch{};XCTAssertEqual(reads,0);live=true;do{_ = try await WorkspaceEntrySelections.read(load:{live=false;return response},workspace:workspace,namespace:"/workspace",current:{live});XCTFail("late reply must refuse")}catch{};live=true;let frozen=try await WorkspaceEntrySelections.read(load:{response},workspace:workspace,namespace:"/workspace",current:{live});XCTAssertEqual(frozen.entryPath,"project.md");live=false;XCTAssertFalse(frozen.current())}
}
