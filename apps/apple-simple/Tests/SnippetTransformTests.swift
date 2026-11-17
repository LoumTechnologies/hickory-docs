import XCTest
@testable import HickorySimple
final class SnippetTransformTests:XCTestCase {
 func testSharedTransformVectors()throws {
  let root=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent(),cases=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:root.appendingPathComponent("apps/simple-contract/snippet-transform-cases.json"))) as? [[String:Any]])
  for row in cases {let program=try readSnippetTransform(row["program"]!),input=try XCTUnwrap(row["input"] as? String);var budget=SnippetTransformBudget();if row["reject"] as? Bool==true {XCTAssertThrowsError(try applySnippetTransform(program,input,budget:&budget),row["name"] as? String ?? "case")}else{XCTAssertEqual(Array(try applySnippetTransform(program,input,budget:&budget).utf8),Array((row["expected"] as! String).utf8),row["name"] as? String ?? "case")}}
 }
 func testMalformedDescriptorsAndSharedActionWorkLimit()throws {
  let raw:[String:Any]=["version":2,"case_map":snippetCaseVersion,"regex":["dot"],"format":[["capture",0,""]],"flags":"","groups":0]
  for(key,value)in [("version",1),("case_map","other"),("groups",true),("flags","gg"),("regex",["class",[[5,1]],false]),("regex",["capture",true,["dot"]])] as [(String,Any)] {var changed=raw;changed[key]=value;XCTAssertThrowsError(try readSnippetTransform(changed))}
  let program=try readSnippetTransform(raw);var budget=SnippetTransformBudget(work:99999);XCTAssertThrowsError(try applySnippetTransform(program,"a",budget:&budget))
 }
}
