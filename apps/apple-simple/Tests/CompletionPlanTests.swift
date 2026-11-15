import XCTest
@testable import HickorySimple
final class CompletionPlanTests:XCTestCase {
    func testSharedSourceImportSelectionAndRefusalVectors()throws {
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let raw=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/completion-plan-cases.json"))) as! [String:Any]
        for item in raw["cases"] as! [[String:Any]] {
            let name=item["name"] as! String,source=item["source"] as! String
            let expanded=item["expansion"] as? [String:Any],selected=expanded?["selection"] as? [String:Int]
            let expansion=expanded.map{CompletionExpansion(text:$0["text"] as! String,selection:selected.map{CompletionSelection(from:$0["from"]!,to:$0["to"]!)})}
            let prepare={try planCompletion(item["uri"] as? String ?? "hick:///a.py",item:item["item"] as! [String:Any],snapshot:SourceSnapshot(text:source,writable:item["writable"] as? Bool ?? true),from:item["from"] as! Int,to:item["to"] as! Int,expansion:expansion)}
            if item["error"] as? Bool==true{XCTAssertThrowsError(try prepare(),name);continue}
            let value=try prepare(),selection=item["selection"] as! [String:Int]
            XCTAssertTrue(exactUTF8(value.before,source),name);XCTAssertTrue(exactUTF8(value.plan.patches.first?.after ?? source,item["after"] as! String),name)
            XCTAssertEqual(value.selection.from,selection["from"],name);XCTAssertEqual(value.selection.to,selection["to"],name);XCTAssertEqual(value.insertionFrom,item["insertionFrom"] as? Int,name)
            if let count=item["patchCount"] as? Int{XCTAssertEqual(value.plan.patches.count,count,name)}
            if let patch=value.plan.patches.first{let inverse=invertChangePlan(value.plan);XCTAssertTrue(exactUTF8(inverse.patches[0].after,source),name);XCTAssertNil(validateChangePlan(inverse,sources:["a.py":SourceSnapshot(text:patch.after,writable:true)]),name)}
        }
    }
}
