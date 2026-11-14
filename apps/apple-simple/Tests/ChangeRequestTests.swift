import XCTest
@testable import HickorySimple
@MainActor final class ChangeRequestTests:XCTestCase {
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
