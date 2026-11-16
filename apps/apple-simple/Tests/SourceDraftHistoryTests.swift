import XCTest
@testable import HickorySimple
@MainActor final class SourceDraftHistoryTests:XCTestCase {
    func testOversizedContextCannotEvictEarlierUndoState() {
        let history=SourceDraftHistory(),manager=UndoManager();manager.groupsByEvent=false
        var source="a",restored=""
        func commit(_ before:String,_ after:String,_ first:String,_ last:String)->Bool {
            let plan=ChangePlan(title:"Replace source",patches:[SourcePatch(path:"a.py",before:before,after:after,edits:[SourceReplacement(from:0,to:before.utf16.count,text:after)])])
            return history.commit(plan,path:"a.py",before:before,selectionBefore:NSRange(location:0,length:0),selectionAfter:NSRange(location:1,length:0),contextBefore:first,contextAfter:last,manager:manager,
                read:{source},apply:{edits,before,_ in let text=NSMutableString(string:before);for edit in edits.sorted(by:{$0.range.location>$1.range.location}){text.replaceCharacters(in:edit.range,with:edit.text)};source=text as String;return true},notice:{_ in XCTFail("Source undo must remain available")},restoreContext:{restored=$0},contextCost:{$0=="oversized" ? 32*1024*1024+1:64},discardContext:{restored="discarded"})
        }
        XCTAssertTrue(commit("a","b","old-a","old-b"));XCTAssertTrue(commit("b","c","oversized","oversized"))
        manager.undo();XCTAssertEqual(source,"b");XCTAssertEqual(restored,"discarded")
        manager.undo();XCTAssertEqual(source,"a");XCTAssertEqual(restored,"old-a");XCTAssertTrue(manager.canRedo)
        manager.redo();XCTAssertEqual(source,"b");XCTAssertEqual(restored,"old-b")
        manager.redo();XCTAssertEqual(source,"c");XCTAssertEqual(restored,"discarded")
    }
}
