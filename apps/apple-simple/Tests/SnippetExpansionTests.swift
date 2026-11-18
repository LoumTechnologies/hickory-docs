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
    func testEmptyChoicesReserveOverheadBeforeCopyingMetadata()throws {
        func item(_ count:Int,_ mirrors:Int=1)->[String:Any] {
            let choices=Array(repeating:"",count:count),syntax="${1|"+String(repeating:",",count:count-1)+"|}"+String(repeating:"$1",count:mirrors-1)+"$0"
            var fields:[[String:Any]]=(0..<mirrors).map{["id":$0,"parent":NSNull(),"index":1,"from":0,"to":0,"choices":choices]}
            fields.append(["id":mirrors,"parent":NSNull(),"index":0,"from":0,"to":0,"choices":[]])
            return ["label":"empty alternatives","insertTextFormat":2,"insertText":syntax,"data":["hickory_snippet":["version":1,"syntax":syntax,"text":"","fields":fields]]]
        }
        XCTAssertEqual(try completionExpansion(item(4096))?.fields.first?.choices.count,4096)
        XCTAssertThrowsError(try completionExpansion(item(4097)))
        XCTAssertThrowsError(try completionExpansion(item(4096,8)))
    }

 func testDecodedRowsShareTheirCalculatedMirrorBudget()throws {
  let syntax="${1:a}|${1/(.*)/${1:/upcase}/}$0",program:[String:Any]=["version":2,"case_map":"hickory-ucd-16.0.0-v1","regex":["capture",1,["repeat",["dot"],0,NSNull(),false]],"format":[["capture",1,"upcase"]],"flags":"","groups":1]
  let fields:[[String:Any]]=[["id":0,"parent":NSNull(),"index":1,"from":0,"to":1,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":2,"to":3,"choices":[],"transform":program],["id":2,"parent":NSNull(),"index":0,"from":3,"to":3,"choices":[]]]
  let item:[String:Any]=["label":"shared mirror","insertTextFormat":2,"insertText":syntax,"data":["hickory_snippet":["version":2,"syntax":syntax,"text":"a|A","fields":fields]]]
  XCTAssertEqual(try completionExpansion(item)?.text,"a|A");var budget=SnippetTransformBudget();_ = try completionExpansion(item,budget:&budget);let first=budget.work;XCTAssertGreaterThan(first,0);_ = try completionExpansion(item,budget:&budget);XCTAssertGreaterThan(budget.work,first);var limited=SnippetTransformBudget(work:100000);XCTAssertThrowsError(try completionExpansion(item,budget:&limited))
 }
}
