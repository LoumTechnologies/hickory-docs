import XCTest
@testable import HickorySimple
@MainActor final class ChangeRequestTests:XCTestCase {
    func testCleanupAndReviewedDraftRefuseEqualByteRevisionRenewal()async throws {
        let w=Workspace(),f=FileNode(name:"a.py",path:"a.py",dir:false),b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"old\n",hash:"base"));w.buffers[f.path]=b;w.show(f.path)
        let edit:[String:Any]=["changes":[languageURI(f.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]]]]
        try w.changes.offer("Reviewed",edit:edit,expected:[f.path:b.text],workspace:w);b.text="later";b.text="old\n";w.changes.apply(w);XCTAssertEqual(b.text,"old\n");XCTAssertTrue(w.changes.notice.contains("Source, selection or provider changed"))
        w.changes.open("cleanup",workspace:w);await w.changes.prepare(w,proposal:{_,_ in b.text="later";b.text="old\n";return edit});XCTAssertNil(w.changes.plan);XCTAssertEqual(b.text,"old\n");XCTAssertTrue(w.changes.notice.contains("Source or selection changed"))
    }

    func testOlderRenameReplyCannotBorrowNewerRequestAuthority()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"value\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        XCTAssertTrue(workspace.currentBuffer === buffer)
        let changes=workspace.changes;changes.open("rename",workspace:workspace)
        var held:[String:CheckedContinuation<Any,Error>]=[:]
        let proposal:(String,String) async throws->Any={kind,name in if kind=="references"{return [] as [[String:Any]]};return try await withCheckedThrowingContinuation {held[name]=$0}}
        func settle(_ condition:@escaping ()->Bool)async{let deadline=Date().addingTimeInterval(2);while !condition()&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
        func edit(_ name:String)->Any{["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":5]],"newText":name]]]]}
        changes.name="first";let first=Task {await changes.prepare(workspace,proposal:proposal)};await settle{held["first"] != nil}
        changes.name="second";let second=Task {await changes.prepare(workspace,proposal:proposal)};await settle{held["second"] != nil}
        try XCTUnwrap(held["second"]).resume(returning:edit("second"));await second.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n")
        try XCTUnwrap(held["first"]).resume(returning:edit("first"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n");XCTAssertEqual(buffer.text,"value\n")
        changes.name="é";let unicode=Task {await changes.prepare(workspace,proposal:proposal)};await settle{held["é"] != nil}
        try XCTUnwrap(held["é"]).resume(returning:edit("é"));await unicode.value;XCTAssertNotNil(changes.plan)
        changes.name="e\u{301}";XCTAssertNil(changes.plan,"A byte-distinct rename intent must retire its previous preview")
    }
}
extension ChangeRequestTests {
    func testCleanupUsesCapturedSourceWithoutAProviderAndRetainsWorkspaceUndo() async throws {
        let workspace=Workspace(),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let before="😀 value \t\r\n",after="😀 value\r\n"
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:before,hash:"base"))
        workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes
        let edit:[String:Any]=["changes":[languageURI(file.path):[["range":["start":["line":0,"character":8],"end":["line":0,"character":10]],"newText":""]]]]
        changes.open("cleanup",workspace:workspace)
        await changes.prepare(workspace,proposal:{kind,_ in XCTAssertEqual(kind,"cleanup");return edit})
        XCTAssertEqual(buffer.text,before);XCTAssertEqual(changes.plan?.patches.first?.after,after)
        changes.apply(workspace);XCTAssertEqual(buffer.text,after)
        changes.open("undo",workspace:workspace);XCTAssertEqual(buffer.text,before)
        changes.open("redo",workspace:workspace);XCTAssertEqual(buffer.text,after)
        XCTAssertEqual(buffer.base,before)
        changes.open("cleanup",workspace:workspace)
        await changes.prepare(workspace,proposal:{_,_ in ["changes":[languageURI(file.path):[]]]})
        XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("No trailing"))
    }
    func testCleanupCannotBorrowNewerAuthorityOrApplyAfterClose() async throws {
        let workspace=Workspace(),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"x ",hash:"base"))
        workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes;changes.open("cleanup",workspace:workspace)
        var held:[CheckedContinuation<Any,Error>]=[]
        let proposal:(String,String) async throws->Any={_,_ in try await withCheckedThrowingContinuation{held.append($0)}}
        func settle(_ count:Int) async {let deadline=Date().addingTimeInterval(2);while held.count<count && Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
        func edit(_ text:String)->Any{["changes":[languageURI(file.path):[["range":["start":["line":0,"character":1],"end":["line":0,"character":2]],"newText":text]]]]}
        let first=Task{await changes.prepare(workspace,proposal:proposal)};await settle(1)
        let second=Task{await changes.prepare(workspace,proposal:proposal)};await settle(2)
        XCTAssertEqual(held.count,2);guard held.count==2 else{return}
        held[1].resume(returning:edit(""));await second.value;XCTAssertEqual(changes.plan?.patches.first?.after,"x")
        held[0].resume(returning:edit("old"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"x")
        let closed=Task{await changes.prepare(workspace,proposal:proposal)};await settle(3)
        changes.close(workspace);held[2].resume(returning:edit(""));await closed.value
        XCTAssertNil(changes.plan);XCTAssertEqual(buffer.text,"x ")
    }
}
extension ChangeRequestTests {
 func testReviewedSourceProposalUsesSharedPreviewApplyAndUndo()throws{
  let workspace=Workspace(),file=FileNode(name:"a.txt",path:"a.txt",dir:false),before="draft\n",after="incoming\n"
  let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:before,hash:"base"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  let edit:[String:Any]=["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":1,"character":0]],"newText":after]]]]
  XCTAssertThrowsError(try workspace.changes.offer("Reviewed source",edit:edit,expected:[:],workspace:workspace));XCTAssertNil(workspace.changes.mode);XCTAssertEqual(buffer.text,before)
  try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:before],workspace:workspace);XCTAssertEqual(workspace.changes.mode,"proposal");XCTAssertEqual(buffer.text,before)
  workspace.changes.apply(workspace);XCTAssertEqual(buffer.text,after);XCTAssertEqual(buffer.base,before)
  workspace.changes.open("undo",workspace:workspace);XCTAssertEqual(buffer.text,before);workspace.changes.open("redo",workspace:workspace);XCTAssertEqual(buffer.text,after)
 }
 func testReviewedSourceProposalRefusesEditsOrSelectionChangesBeforeApproval()throws{
  let workspace=Workspace(),file=FileNode(name:"a.txt",path:"a.txt",dir:false)
  let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"x",hash:"base"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  let edit:[String:Any]=["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":"y"]]]]
  try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:"x"],workspace:workspace);buffer.text="typed";workspace.changes.apply(workspace);XCTAssertEqual(buffer.text,"typed")
  XCTAssertThrowsError(try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:"x"],workspace:workspace))
  buffer.text="x";try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:"x"],workspace:workspace);buffer.selection=NSRange(location:1,length:0);workspace.changes.apply(workspace);XCTAssertEqual(buffer.text,"x")
 }
}
