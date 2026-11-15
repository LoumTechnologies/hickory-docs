import XCTest
@testable import HickorySimple
final class SnippetExpansionTests:XCTestCase {
    func testSharedWireAndImportPositions()throws {
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-expansion-cases.json"))) as! [String:Any]
        for value in vectors["cases"] as! [[String:Any]] {
            let name=value["name"] as! String,item=value["item"] as! [String:Any]
            if value["error"] as? Bool==true{XCTAssertThrowsError(try completionExpansion(item),name);continue}
            if value["plain"] as? Bool==true{XCTAssertNil(try completionExpansion(item),name);continue}
            let expansion=try XCTUnwrap(try completionExpansion(item)),expected=value["expected"] as! [String:Any],selection=expected["selection"] as! [String:Int]
            XCTAssertTrue(exactUTF8(expansion.text,expected["text"] as! String),name);XCTAssertEqual(expansion.selection?.from,selection["from"],name);XCTAssertEqual(expansion.selection?.to,selection["to"],name)
            let draft=try planCompletion("hick:///a.py",item:item,snapshot:SourceSnapshot(text:"va",writable:true),from:0,to:2,expansion:expansion)
            XCTAssertTrue(exactUTF8(draft.plan.patches[0].after,expected["after"] as! String),name)
            let fields=expected["fields"] as! [[String:Any]];XCTAssertEqual(draft.fields.count,fields.count,name)
            for (actual,field) in zip(draft.fields,fields){XCTAssertEqual(actual.index,field["index"] as? Int,name);XCTAssertEqual(actual.from,field["sourceFrom"] as? Int,name);XCTAssertEqual(actual.to,field["sourceTo"] as? Int,name);if field["id"] != nil{XCTAssertEqual(actual.id,field["id"] as? Int,name);XCTAssertEqual(actual.parent,field["parent"] as? Int,name)}}
        }
    }
    func testChoiceMetadataAmplificationRefusesBeforePreparingSource()throws {
        let alternative=String(repeating:"x",count:1024),syntax="${1|,"+String(repeating:"x",count:1024)+"|}"+String(repeating:"$1",count:1029)+"$0"
        var fields:[[String:Any]]=(0..<1030).map{["id":$0,"parent":NSNull(),"index":1,"from":0,"to":0,"choices":["",alternative]]}
        fields.append(["id":1030,"parent":NSNull(),"index":0,"from":0,"to":0,"choices":[]])
        XCTAssertThrowsError(try completionExpansion(["label":"large choices","insertTextFormat":2,"insertText":syntax,"data":["hickory_snippet":["version":1,"syntax":syntax,"text":"","fields":fields]]]))
    }

}
