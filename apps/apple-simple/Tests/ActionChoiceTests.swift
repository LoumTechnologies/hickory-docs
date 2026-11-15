import XCTest
@testable import HickorySimple
@MainActor final class ActionChoiceTests:XCTestCase {
    func testLatestResolvedChoiceWinsAndRenderedSnapshotsCannotBorrowNewLists()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"old\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes;changes.open("actions",workspace:workspace)
        let actions:[[String:Any]]=["first","second"].map {["title":$0,"data":["hickory_action":$0]]}
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={action in try await withCheckedThrowingContinuation {held[action["title"] as! String]=$0}}
        func prepare()async{await changes.prepare(workspace,proposal:{_,_ in actions},resolution:resolve)}
        func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
        func resolved(_ name:String)->[String:Any]{["title":name,"edit":["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":name]]]]]}
        await prepare();let snapshot=changes.actionSnapshot
        let first=Task{await changes.choose(0,workspace:workspace,expectedSnapshot:snapshot)};await settle{held["first"] != nil}
        let second=Task{await changes.choose(1,workspace:workspace,expectedSnapshot:snapshot)};await settle{held["second"] != nil}
        try XCTUnwrap(held["second"]).resume(returning:resolved("second"));await second.value
        try XCTUnwrap(held["first"]).resume(returning:resolved("first"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n")
        held=[:];await prepare();await changes.choose(0,workspace:workspace,expectedSnapshot:snapshot);XCTAssertTrue(held.isEmpty);XCTAssertNil(changes.plan)
        let current=changes.actionSnapshot;let closing=Task{await changes.choose(0,workspace:workspace,expectedSnapshot:current)};await settle{held["first"] != nil};changes.close(workspace)
        try XCTUnwrap(held["first"]).resume(returning:resolved("first"));await closing.value;XCTAssertNil(changes.plan);XCTAssertEqual(buffer.text,"old\n")
        changes.open("actions",workspace:workspace)
        await changes.prepare(workspace,proposal:{_,_ in actions},resolution:{_ in var value=resolved("partial");value["command"]=["title":"Effect","command":"provider.effect"];return value})
        await changes.choose(0,workspace:workspace,expectedSnapshot:changes.actionSnapshot);XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("provider command"));XCTAssertEqual(buffer.text,"old\n")
    }
}
