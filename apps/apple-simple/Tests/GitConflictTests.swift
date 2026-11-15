import XCTest
@testable import HickorySimple
@MainActor final class GitConflictTests:XCTestCase {
 func reading(_ path:String,_ text:String="captured")->ConflictReading{ConflictReading(version:1,path:path,fingerprint:String(repeating:"a",count:64),stages:[ConflictStage(stage:2,mode:"100644",oid:String(repeating:"b",count:40),text:text,unavailable:nil)],index_modified:false,source_files_modified:false)}
 func testOlderClosedAndForeignRepliesAreRetired()async throws{
  let model=GitConflictModel();var held:[CheckedContinuation<ConflictReading,Error>]=[]
  let load:(String)async throws->ConflictReading={_ in try await withCheckedThrowingContinuation{held.append($0)}}
  func settle(_ count:Int)async{let deadline=Date().addingTimeInterval(2);while held.count<count&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
  let first=Task{await model.read("a.txt",load:load)};await settle(1);let second=Task{await model.read("b.txt",load:load)};await settle(2);XCTAssertEqual(held.count,2);guard held.count==2 else{return}
  held[1].resume(returning:reading("b.txt","new"));await second.value;held[0].resume(returning:reading("a.txt","old"));await first.value;XCTAssertEqual(model.value?.path,"b.txt")
  let closed=Task{await model.read("a.txt",load:load)};await settle(3);model.close();held[2].resume(returning:reading("a.txt"));await closed.value;XCTAssertNil(model.value)
  await model.read("é.txt"){_ in self.reading("e\u{301}.txt")};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("identity changed"))
 }
 func testImmutableStageAndAvailabilityValidation()throws{
  let value=reading("a.txt");XCTAssertNoThrow(try validateConflict(value,path:"a.txt"))
  var duplicate=value;duplicate.stages.append(duplicate.stages[0]);XCTAssertThrowsError(try validateConflict(duplicate,path:"a.txt"))
  var bad=value;bad.stages[0].oid="main";XCTAssertThrowsError(try validateConflict(bad,path:"a.txt"))
  bad=value;bad.stages[0].mode="120000";XCTAssertThrowsError(try validateConflict(bad,path:"a.txt"))
  bad=value;bad.stages[0].text=nil;XCTAssertThrowsError(try validateConflict(bad,path:"a.txt"));bad.stages[0].unavailable="Binary";XCTAssertNoThrow(try validateConflict(bad,path:"a.txt"))
 }
}
extension GitConflictTests {
 func testChangedBinaryAndClosedChoicesCannotOfferSource()async throws{
  let model=GitConflictModel();var offered=0;let load:(String)async throws->ConflictReading={_ in self.reading("a.txt")};let offer:(ConflictStage,ConflictReading,@escaping ()->Bool)async throws->Void={_,_,_ in offered+=1}
  await model.read("a.txt",load:load);await model.choose(2,load:load,offer:offer);XCTAssertEqual(offered,1)
  await model.choose(2,load:{_ in var value=self.reading("a.txt");value.fingerprint=String(repeating:"c",count:64);return value},offer:offer);XCTAssertEqual(offered,1);XCTAssertTrue(model.notice.contains("stages changed"))
  await model.choose(2,load:{_ in self.reading("a.txt","different bytes")},offer:offer);XCTAssertEqual(offered,1)
  var held:CheckedContinuation<ConflictReading,Error>?
  let pending=Task{await model.choose(2,load:{_ in try await withCheckedThrowingContinuation{held=$0}},offer:offer)}
  let deadline=Date().addingTimeInterval(2);while held==nil&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))};model.close();try XCTUnwrap(held).resume(returning:reading("a.txt"));await pending.value;XCTAssertEqual(offered,1)
  var binary=reading("a.txt");binary.stages[0].text=nil;binary.stages[0].unavailable="Binary";await model.read("a.txt"){_ in binary};await model.choose(2,load:{_ in binary},offer:offer);XCTAssertEqual(offered,1)
 }
}
