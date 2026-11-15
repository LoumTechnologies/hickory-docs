import XCTest
@testable import HickorySimple
@MainActor final class ConstructionReadingTests:XCTestCase {
 func value(_ entry:String="project.md",materialized:Bool=false)->ConstructionReading{ConstructionReading(version:1,entry:entry,materialized:materialized,outputs:[],output_count:0,outputs_truncated:false,tests:[],test_count:0,tests_truncated:false,test_relationship:"declaration",evidence_status:"not_queried",notices:[],unused_sources:[])}
 func testClosedAndSupersededRepliesCannotReplaceTheCapturedReading()async{
  let model=ConstructionReadingModel();var pending:CheckedContinuation<ConstructionReading,Never>?
  let old=Task{await model.read{_,_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.read{_,_ in var reply=self.value();reply.notices=["new"];return reply};pending?.resume(returning:value());await old.value;XCTAssertEqual(model.value?.notices,["new"])
  pending=nil;let closed=Task{await model.read{_,_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:value());await closed.value;XCTAssertNil(model.value)
  await model.read{_,_ in self.value("other.md")};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("identity changed"))
  await model.read{_,_ in self.value(materialized:true)};XCTAssertNil(model.value)
 }
}
