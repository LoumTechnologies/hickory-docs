import XCTest
@testable import HickorySimple
@MainActor final class GitHistoryTests:XCTestCase {
 func testHeldAndClosedReadingsRetainTheSelectedCommitIdentity()async {
  let model=GitHistoryModel(),a=String(repeating:"a",count:40),b=String(repeating:"b",count:40)
  await model.refresh(){_,_ in HistoryLog(repository:true,commits:[a,b].map{HistoryCommit(sha:$0,short:String($0.prefix(7)),author:"fixture",subject:"saved commit")})}
  var pending:CheckedContinuation<HistoryReading,Never>?
  let first=Task{await model.select(a){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()}
  await model.select(b){HistoryReading(sha:$0,message:"new",files:[])};pending?.resume(returning:HistoryReading(sha:a,message:"obsolete",files:[]));await first.value;XCTAssertEqual(model.reading?.sha,b);XCTAssertEqual(model.reading?.message,"new")
  pending=nil;let closed=Task{await model.select(a){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:HistoryReading(sha:a,message:"closed",files:[]));await closed.value;XCTAssertNil(model.reading)
  await model.select(a){_ in HistoryReading(sha:b,message:"foreign",files:[])};XCTAssertNil(model.reading);XCTAssertTrue(model.notice.contains("identity changed"))
 }
 func testMalformedLogIdentityNeverBecomesAReadingChoice()async {
  let model=GitHistoryModel();await model.refresh(limit:2000){_,_ in HistoryLog(repository:true,commits:[HistoryCommit(sha:"HEAD",short:"HEAD",author:"fixture",subject:"ambiguous")])};XCTAssertEqual(model.limit,1000);XCTAssertTrue(model.commits.isEmpty);XCTAssertTrue(model.notice.contains("Malformed commit identity"))
 }
 func testObservedFullCommitRequestDoesNotManufactureALogChoice()async {
  let sha=String(repeating:"c",count:40),model=GitHistoryModel(request:.commit(String(repeating:"c",count:40),"Saved attribution"));var calls:[String]=[]
  func load(_ value:String)async throws->HistoryReading{calls.append(value);return HistoryReading(sha:value,message:"observed",files:[])}
  await model.readCommit(sha,load:load);XCTAssertEqual(calls,[sha]);XCTAssertEqual(model.reading?.sha,sha)
  await model.readCommit("12345678",load:load);XCTAssertEqual(calls,[sha]);XCTAssertNil(model.reading);await model.readCommit(String(repeating:"d",count:40),load:load);XCTAssertEqual(calls,[sha]);XCTAssertNil(model.reading)
 }

}
