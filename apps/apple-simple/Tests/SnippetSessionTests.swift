import XCTest
@testable import HickorySimple
final class SnippetSessionTests:XCTestCase {
    func fields(_ values:[[String:Any]])->[SnippetField]{values.map{SnippetField(index:$0["index"] as! Int,from:$0["from"] as! Int,to:$0["to"] as! Int,choices:$0["choices"] as! [String],id:$0["id"] as? Int,parent:$0["parent"] as? Int)}}
    func testSharedLinkedEditsScopeAndTraversal()throws {
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-session-cases.json"))) as! [String:Any]
        for value in vectors["cases"] as! [[String:Any]] {
            let name=value["name"] as! String;var source=value["source"] as! String,session=beginSnippet(value["source"] as! String,fields:fields(value["fields"] as! [[String:Any]]))
            if value["invalidBegin"] as? Bool==true{XCTAssertNil(session,name);continue};XCTAssertNotNil(session,name)
            for step in value["steps"] as! [[String:Any]] {
                let current=try XCTUnwrap(session),input=step["currentSource"] as? String ?? source,from=step["from"] as! Int,to=step["to"] as! Int,action=step["action"] as! String
                let selected:CompletionSelection?,next:SnippetSession?
                if action=="edit" || action=="complete" || action=="nest" {
                    let additional=(step["additional"] as? [[String:Any]] ?? []).map{SourceReplacement(from:$0["from"] as! Int,to:$0["to"] as! Int,text:$0["text"] as! String)}
                    let result=action=="nest" ? nestSnippet(current,source:input,primary:SourceReplacement(from:from,to:to,text:step["text"] as? String ?? ""),additional:additional,inner:fields(step["inner"] as? [[String:Any]] ?? [])):action=="complete" ? completeSnippet(current,source:input,primary:SourceReplacement(from:from,to:to,text:step["text"] as? String ?? ""),additional:additional):editSnippet(current,source:input,from:from,to:to,text:String(repeating:step["text"] as? String ?? "",count:step["repeat"] as? Int ?? 1))
                    if step["refuse"] as? Bool==true{XCTAssertNil(result,name);continue}
                    let applied=try XCTUnwrap(result);XCTAssertTrue(exactUTF8(applied.session.source,step["after"] as! String),name);source=applied.session.source;selected=applied.selection;next=applied.session
                }else{
                    let result=advanceSnippet(current,source:input,from:from,to:to,direction:action=="next" ? 1:-1)
                    if step["refuse"] as? Bool==true{XCTAssertNil(result,name);continue};let applied=try XCTUnwrap(result);selected=applied.selection;next=applied.session
                }
                session=next;let expected=step["selection"] as! [String:Int];XCTAssertEqual(selected?.from,expected["from"],name);XCTAssertEqual(selected?.to,expected["to"],name)
                if let expected=step["fields"] as? [[String:Any]]{let actual=try XCTUnwrap(next).fields;XCTAssertEqual(actual.count,expected.count,name);for(field,value)in zip(actual,expected){XCTAssertEqual(field.id,value["id"] as? Int,name);XCTAssertEqual(field.parent,value["parent"] as? Int,name);XCTAssertEqual(field.index,value["index"] as? Int,name);XCTAssertEqual(field.from,value["from"] as? Int,name);XCTAssertEqual(field.to,value["to"] as? Int,name)}}
                if step["active"] != nil{XCTAssertEqual(next?.active,step["active"] as? Int,name)}
            }
        }
    }
}
