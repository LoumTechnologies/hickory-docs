import XCTest
@testable import HickorySimple
@MainActor final class GitAttributionTests:XCTestCase {
 func row(_ line:Int,_ full:String?=nil)->AttributionLine{AttributionLine(line:line,commit:"12345678",full_commit:full,author:"Fixture",summary:"base",uncommitted:false,text:"saved")}
 func testCapturedPathsAndLineIdentitiesRetireObsoleteReplies()async {
  let model=GitAttributionModel(path:"a.py");var pending:CheckedContinuation<AttributionReading,Never>?
  let old=Task{await model.refresh{_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.refresh{AttributionReading(path:$0,lines:[self.row(2,String(repeating:"a",count:40))])};pending?.resume(returning:AttributionReading(path:"a.py",lines:[row(1)]));await old.value;XCTAssertEqual(model.lines.map{$0.line},[2])
  pending=nil;let closed=Task{await model.refresh{_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:AttributionReading(path:"a.py",lines:[row(1)]));await closed.value;XCTAssertTrue(model.lines.isEmpty)
  let foreign=GitAttributionModel(path:"cafe\u{301}.py");await foreign.refresh{_ in AttributionReading(path:"caf\u{e9}.py",lines:[self.row(1)])};XCTAssertTrue(foreign.lines.isEmpty);XCTAssertTrue(foreign.notice.contains("identity changed"))
  await model.refresh{AttributionReading(path:$0,lines:[self.row(1),self.row(1)])};XCTAssertTrue(model.lines.isEmpty);XCTAssertTrue(model.notice.contains("Ambiguous"))
 }
 func testCompactLabelsAndDirtyLinesAreNotImmutableReferences(){
  XCTAssertNil(row(1).immutableCommit);XCTAssertNil(row(1,"12345678").immutableCommit);var dirty=row(1,String(repeating:"a",count:40));dirty.uncommitted=true;XCTAssertNil(dirty.immutableCommit);XCTAssertEqual(row(1,String(repeating:"a",count:40)).immutableCommit,String(repeating:"a",count:40));XCTAssertEqual(row(1,String(repeating:"b",count:64)).immutableCommit,String(repeating:"b",count:64))
 }
}
