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
 func testRenameIndexActionsDoNotReplayAfterUnconfirmedResults()async throws {
  let model=GitReviewModel(),file=ReviewFile(path:"new.py",from:"old.py",index:"R",tree:"M");var calls:[[String]]=[];var fail=false
  func load()async throws->ReviewChanges{ReviewChanges(repository:true,files:[file])}
  func write(_ paths:[String],_ unstage:Bool)async throws->Bool{calls.append(paths);if fail{throw EngineError(message:"connection ended")};return true}
  await model.refresh(load:load);await model.changeIndex("new.py",unstage:true,write:write,load:load);XCTAssertEqual(calls,[["old.py","new.py"]]);XCTAssertFalse(model.needsRefresh)
  fail=true;await model.changeIndex("new.py",unstage:false,write:write,load:load);XCTAssertEqual(calls.count,2);XCTAssertTrue(model.needsRefresh);await model.changeIndex("new.py",unstage:false,write:write,load:load);XCTAssertEqual(calls.count,2)
  await model.refresh(load:load);fail=false;await model.changeIndex("new.py",unstage:false,write:write,load:load);XCTAssertEqual(calls.count,3);XCTAssertFalse(model.needsRefresh)
  var outside=file;outside.from_outside_workspace=true;XCTAssertThrowsError(try reviewIndexPaths(outside,unstage:true))
  let renamed=ReviewFile(path:"cafe\u{301}.py",from:"caf\u{e9}.py",index:"R",tree:" ");XCTAssertEqual(try reviewIndexPaths(renamed,unstage:true).count,2)
 }

 func testPendingIndexWriteSurvivesCloseAndRequiresAReadAfterSettlement()async {
  let model=GitReviewModel();var pending:CheckedContinuation<Bool,Never>?;var calls=0
  func load()async throws->ReviewChanges{ReviewChanges(repository:true,files:[ReviewFile(path:"a.py",index:" ",tree:"M")])}
  await model.refresh(load:load)
  let write=Task{await model.changeIndex("a.py",unstage:false,write:{_,_ in calls+=1;return await withCheckedContinuation{pending=$0}},load:load)}
  while pending==nil{await Task.yield()};model.close();await model.refresh(load:load);XCTAssertTrue(model.indexPending);XCTAssertTrue(model.needsRefresh)
  await model.changeIndex("a.py",unstage:false,write:{_,_ in calls+=1;return true},load:load);XCTAssertEqual(calls,1)
  pending?.resume(returning:true);await write.value;XCTAssertFalse(model.indexPending);XCTAssertTrue(model.needsRefresh);await model.refresh(load:load);XCTAssertFalse(model.needsRefresh)
 }

}
