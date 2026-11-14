import XCTest
@testable import HickorySimple
final class CoverageProjectionTests:XCTestCase {
    struct Fixture:Decodable {var version:Int;var cases:[Case]}
    struct Case:Decodable {var name:String;var record:CoverageProjectionRecord;var expected:[String:[CoverageProjectedLine]]}
    func testBundledCoverageOwnershipAndUnknowns()throws {
        let url=try XCTUnwrap(simpleResource("coverage-projections",withExtension:"json"),"Rebuild the shared capability resources.")
        let fixture=try JSONDecoder().decode(Fixture.self,from:Data(contentsOf:url));XCTAssertEqual(fixture.version,1);XCTAssertEqual(fixture.cases.count,5)
        for test in fixture.cases {XCTAssertEqual(coverageProjection(test.record),test.expected,test.name)}
    }
}
