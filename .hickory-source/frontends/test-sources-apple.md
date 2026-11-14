<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain source test observations on apple

For maintainers replacing the test UI without replacing source and execution admission.

Keep source snapshots and provider observations as distinct values. Unknown source mappings stay absent; target runtime locations remain available without guessing an authored owner.

<ui:copy id="test-sources-apple-values">
import Foundation
struct TestSourceRequest: Equatable {
    static func ==(lhs:Self,rhs:Self)->Bool {exactUTF8(lhs.path,rhs.path)&&exactUTF8(lhs.source,rhs.source)&&exactUTF8(lhs.projectEntry,rhs.projectEntry)&&exactSourceValues(lhs.sources)==exactSourceValues(rhs.sources)}
    var path:String;var source:String;var projectEntry:String?;var sources:[String:String]=[:]
    var body:[String:Any] {var value:[String:Any]=["path":path,"source":source,"sources":sources];if let projectEntry {value["project_entry"]=projectEntry};return value}
}
struct TestSourceSnapshot:Equatable {var workspace:String;var request:TestSourceRequest
    static func ==(lhs:Self,rhs:Self)->Bool {exactUTF8(lhs.workspace,rhs.workspace)&&lhs.request==rhs.request}
}
struct TestSourcePoint:Decodable {var path:String;var line:Int;var character:Int;var hash:String}
struct SourceTestCandidate:Decodable,Identifiable {
    var id:String;var language:String;var run_scope:String;var name:String;var name_filter:String?;var confidence:String;var source:TestSourcePoint?;var target:TestSourcePoint
    var point:TestSourcePoint {source ?? target}
}
struct SourceTestScan:Decodable {
    var version:Int;var input_path:String;var fingerprint:String;var source_hashes:[String:String];var candidates:[SourceTestCandidate]
    var scope:String;var syntax_complete:Bool;var notices:[String];var unsupported_languages:[String];var unclassified_outputs:[String];var unused_sources:[String]
}
</ui:copy>

A view becomes stale when its primary draft, selected construction entry, workspace identity or any contributing supplied draft changes. An unrelated unused draft must not retire it. Newly retained contributor text requires a fresh scan. Saved files are checked independently by the engine at execution admission.

<ui:copy id="test-sources-apple-current">
func testSourceCurrent(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ now:TestSourceSnapshot)->Bool {
    let a=before.request,b=now.request
    guard facts.version==1,exactUTF8(facts.input_path,a.path),exactUTF8(before.workspace,now.workspace),exactUTF8(a.path,b.path),exactUTF8(a.source,b.source),exactUTF8(a.projectEntry,b.projectEntry) else {return false}
    let previous=exactSourceValues(a.sources),current=exactSourceValues(b.sources)
    return facts.source_hashes.keys.allSatisfy {exactUTF8($0,a.path) || previous[Data($0.utf8)]==current[Data($0.utf8)]}
}
</ui:copy>

Build a versioned saved-run request only for a candidate in the current observation. Dynamic declarations cannot become name filters; a user must explicitly select the containing runner scope or discover concrete cases.

<ui:copy id="test-sources-apple-selection">
enum TestSourceSelection:String {case nameFilter="name_filter",containingScope="containing_scope"}
func sourceTestRun(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ now:TestSourceSnapshot,id:String,selection:TestSourceSelection)throws->[String:Any] {
    guard testSourceCurrent(before,facts,now) else {throw EngineError(message:"Test source changed. Refresh candidates before running.")}
    guard let candidate=facts.candidates.first(where:{$0.id==id}) else {throw EngineError(message:"Candidate unavailable. Refresh test candidates.")}
    guard selection != .nameFilter || candidate.name_filter != nil else {throw EngineError(message:"This declaration needs runner discovery or an explicit containing-scope run.")}
    return ["version":1,"input":["kind":"saved"],"analysis":before.request.body,"fingerprint":facts.fingerprint,"candidate_id":id,"selection":["kind":selection.rawValue]]
}
</ui:copy>

The focused witnesses change primary/contributor/workspace observations, preserve an unrelated draft, reject a future protocol version, and distinguish dynamic declaration filters from explicit containing-scope execution.

<ui:copy id="test-sources-apple-witness">
import XCTest
@testable import HickorySimple
final class TestSourcesTests:XCTestCase {
    func fixture()->(TestSourceSnapshot,SourceTestScan) {
        let before=TestSourceSnapshot(workspace:"w1",request:TestSourceRequest(path:"cases.md",source:"draft",projectEntry:"project.md",sources:["helper.md":"helper","unused.md":"old"]))
        let point=TestSourcePoint(path:"test.py",line:1,character:4,hash:"target")
        let candidate=SourceTestCandidate(id:"case",language:"python",run_scope:"file",name:"test_case",name_filter:"test_case",confidence:"syntax",source:nil,target:point)
        return (before,SourceTestScan(version:1,input_path:"cases.md",fingerprint:"scan",source_hashes:["cases.md":"draft-hash","helper.md":"helper-hash"],candidates:[candidate],scope:"project",syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],unused_sources:["unused.md"]))
    }
    func testContributorAndWorkspaceIdentity() {
        let (before,facts)=fixture();XCTAssertTrue(testSourceCurrent(before,facts,before));var now=before
        now.workspace="w2";XCTAssertFalse(testSourceCurrent(before,facts,now));now=before
        now.request.source="changed";XCTAssertFalse(testSourceCurrent(before,facts,now));now=before
        now.request.sources["helper.md"]="changed";XCTAssertFalse(testSourceCurrent(before,facts,now));now=before
        now.request.sources["unused.md"]="new";XCTAssertTrue(testSourceCurrent(before,facts,now))
        var future=facts;future.version=2;XCTAssertFalse(testSourceCurrent(before,future,before))
    }
    func testExplicitSelectionAndDynamicRefusal()throws {
        let (before,facts)=fixture();let value=try sourceTestRun(before,facts,before,id:"case",selection:.nameFilter)
        XCTAssertEqual(value["fingerprint"] as? String,"scan");XCTAssertEqual((value["input"] as? [String:String])?["kind"],"saved")
        var dynamic=facts;dynamic.candidates[0].name_filter=nil
        XCTAssertThrowsError(try sourceTestRun(before,dynamic,before,id:"case",selection:.nameFilter))
        XCTAssertNoThrow(try sourceTestRun(before,dynamic,before,id:"case",selection:.containingScope))
        XCTAssertThrowsError(try sourceTestRun(before,facts,before,id:"missing",selection:.nameFilter))
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/TestSources.swift"><ui:paste select="#test-sources-apple-values"/><ui:paste select="#test-sources-apple-current"/><ui:paste select="#test-sources-apple-selection"/></ui:file>
<ui:file path="apps/apple-simple/Tests/TestSourcesTests.swift"><ui:paste select="#test-sources-apple-witness"/></ui:file>
</ui:doc>
