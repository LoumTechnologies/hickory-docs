import XCTest
@testable import HickorySimple
final class SnippetCaseTests:XCTestCase {
 func testFrozenUnicodeCases()throws{let url=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent("simple-contract/snippet-case-cases.json");let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:url)) as! [String:Any];XCTAssertEqual(snippetCaseVersion,vectors["case_map"] as? String);for c in vectors["cases"] as! [[String:String]]{XCTAssertEqual(Array(snippetCase(c["value"]!,c["mode"]!).utf8),Array(c["expected"]!.utf8),c["value"]!)};for c in vectors["words"] as! [[String:Any]]{XCTAssertEqual(snippetCaseWords(c["value"] as! String).map{Array($0.utf8)},(c["expected"] as! [String]).map{Array($0.utf8)})}}
}
