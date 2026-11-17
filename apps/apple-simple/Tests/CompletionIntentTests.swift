import XCTest
@testable import HickorySimple
@MainActor final class CompletionIntentTests:XCTestCase {
 private func fixture()throws->(DocumentBuffer,NativeLanguage){let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va\n",hash:"base"));let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service;buffer.selection=NSRange(location:2,length:0);return(buffer,service)}
 private func offer(_ buffer:DocumentBuffer)->NativeCompletion{NativeCompletion(id:1,label:"value",detail:"",range:NSRange(location:0,length:2),insert:"value",source:buffer.text,selection:buffer.selection,original:["label":"value"],sourceIdentity:buffer.sourceEditIdentity,selectionIdentity:buffer.selectionIdentity)}
 private func resolved()->[String:Any]{["label":"value","textEdit":["range":["start":["line":0,"character":0],"end":["line":0,"character":2]],"newText":"value"]]}
 private func roundTrip(_ buffer:DocumentBuffer,source:Bool){if source{let original=buffer.text;buffer.text="xy\n";buffer.text=original}else{let original=buffer.selection;buffer.selection=NSRange(location:0,length:0);buffer.selection=original}}
 func testOfferedCompletionCannotReviveAfterSourceOrSelectionRoundTrip()async throws {
  for source in [true,false]{let(buffer,service)=try fixture();defer{service.close()};let item=offer(buffer);roundTrip(buffer,source:source);var calls=0,commits=0;buffer.applyCompletionDraft={_,_ in commits+=1;return true};let accepted=await service.accept(item,resolution:{_ in calls+=1;return self.resolved()});XCTAssertFalse(accepted);XCTAssertEqual(calls,0);XCTAssertEqual(commits,0);XCTAssertEqual(buffer.text,"va\n");XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertTrue(service.completionNotice.contains("out of date"))}
 }
 func testPendingCompletionCannotReviveAfterSourceOrSelectionRoundTrip()async throws {
  for source in [true,false]{let(buffer,service)=try fixture();defer{service.close()};let item=offer(buffer);var commits=0;buffer.applyCompletionDraft={_,_ in commits+=1;return true};let accepted=await service.accept(item,resolution:{_ in self.roundTrip(buffer,source:source);return self.resolved()});XCTAssertFalse(accepted);XCTAssertEqual(commits,0);XCTAssertEqual(buffer.text,"va\n");XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertTrue(service.completionNotice.contains("changed"))}
 }
}
