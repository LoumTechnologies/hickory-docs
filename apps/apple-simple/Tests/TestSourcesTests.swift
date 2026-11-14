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
