import XCTest
@testable import HickorySimple
@MainActor final class SourceRecoveryTests:XCTestCase {
 let record=String(repeating:"a",count:32),workspace=String(repeating:"b",count:32)
 func page()->RecoveryPage{RecoveryPage(version:1,workspace:workspace,ids:[record],ignored:0)}
 func reading(_ hash:String=String(repeating:"c",count:64))->RecoveryReading{RecoveryReading(version:1,workspace:workspace,id:record,fingerprint:hash,title:"Move",title_truncated:false,paths:[RecoveryPath(path:"a #😀.md",saved_before:RecoverySummary(kind:"file",bytes:0),saved_after:RecoverySummary(kind:"missing"),draft_before:RecoverySummary(kind:"file",bytes:4),draft_after:RecoverySummary(kind:"missing"))],all_paths_recorded_applied:false,unrecognized_files:[],draft_text:"😀",saved_text:"")}
 func testExactSnapshotRequestsAndChangedEvidence()async{
  let model=RecoveryReader();await model.list{_ in self.page()};var route="";await model.read(record,path:"a #😀.md",snapshot:"saved_before"){route=$0;return self.reading()};XCTAssertTrue(route.contains("path=a%20%23%F0%9F%98%80.md&saved=before"));XCTAssertEqual(model.value?.saved_text,"");XCTAssertEqual(model.value?.draft_text,"😀")
  await model.read(record,path:"a #😀.md"){_ in self.reading(String(repeating:"d",count:64))};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("evidence changed"))
  await model.list{_ in self.page()};await model.list(after:record){_ in var page=self.page();page.workspace="other";page.ids=[];return page};XCTAssertNil(model.page)
 }
 func testClosedAndSupersededReadsCannotReplaceSelection()async{
  let model=RecoveryReader();await model.list{_ in self.page()};var pending:CheckedContinuation<RecoveryReading,Never>?
  let old=Task{await model.read(record){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.list{_ in self.page()};pending?.resume(returning:reading());await old.value;XCTAssertNil(model.value)
  pending=nil;let closed=Task{await model.read(record){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:reading());await closed.value;XCTAssertNil(model.value);XCTAssertNil(model.page)
 }
}
