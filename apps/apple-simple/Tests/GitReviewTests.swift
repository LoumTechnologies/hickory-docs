import XCTest
@testable import HickorySimple
@MainActor final class GitReviewTests:XCTestCase {
 func testByteDistinctUnicodePathsKeepSeparateRowsAndDiffIdentities()async {
  let first="caf\u{e9}.py",second="cafe\u{301}.py",a=ReviewFile(path:first,index:"M",tree:"M"),b=ReviewFile(path:second,index:"M",tree:"M")
  XCTAssertNotEqual(a.id,b.id);let model=GitReviewModel();await model.refresh{ReviewChanges(repository:true,branch:"main",files:[a,b])}
  await model.select(second,staged:false){_,_ in ReviewDiff(path:first,diff:"wrong Unicode source",binary:false)}
  XCTAssertEqual(model.detail,"");XCTAssertTrue(model.notice.contains("identity changed"))
 }
 func testLatestSideAndClosedRepliesRetainTheirOwnIdentity()async {
  let model=GitReviewModel();await model.refresh{ReviewChanges(repository:true,branch:"main",files:["a.py","b.py"].map{ReviewFile(path:$0,index:"M",tree:"M")})}
  var pending:CheckedContinuation<ReviewDiff,Never>?
  let first=Task{await model.select("a.py",staged:false){_,_ in await withCheckedContinuation{pending=$0}}}
  while pending==nil{await Task.yield()}
  await model.select("b.py",staged:true){path,_ in ReviewDiff(path:path,diff:"new staged",binary:false)}
  pending?.resume(returning:ReviewDiff(path:"a.py",diff:"obsolete",binary:false));await first.value
  XCTAssertEqual(model.selected,"b.py");XCTAssertTrue(model.staged);XCTAssertEqual(model.detail,"new staged")
  pending=nil;let closed=Task{await model.select("a.py",staged:false){_,_ in await withCheckedContinuation{pending=$0}}}
  while pending==nil{await Task.yield()};model.close();pending?.resume(returning:ReviewDiff(path:"a.py",diff:"late after close",binary:false));await closed.value;XCTAssertEqual(model.detail,"")
  await model.select("a.py",staged:false){_,_ in ReviewDiff(path:"b.py",diff:"wrong source",binary:false)};XCTAssertEqual(model.detail,"");XCTAssertTrue(model.notice.contains("identity changed"))
 }
}
