<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep search useful while source knowledge changes

For maintainers changing workspace services independently of the editor renderer.

Exercise a mixed workspace rather than the ranking implementation. Duplicate file listings collapse; a generated file stays discoverable and read-only; a foreign-source symbol is rejected. Only the current source contributes type/member/general-symbol targets. Disabled actions remain visible. After a draft changes, file and action search survives while every old symbol disappears. A capped view retains the full match count, and fuzzy matching handles an emoji scalar without splitting its surrogate pair.

<ui:copy id="simple-search-everywhere-apple-tests">
import XCTest
@testable import HickorySimple
final class SearchEverywhereTests:XCTestCase {
    func testSourceKnowledgeAndFallbacks() {
        let file=FileNode(name:"Widget.py",path:"src/Widget.py",dir:false),generated=FileNode(name:"out.py",path:"out.py",dir:false,generated_by:"owner")
        var source="source"
        let values:[[String:Any]]=[5,6,99].enumerated().map {i,kind in ["name":["Widget","render","unknown"][i],"kind":kind,"range":["start":["line":0,"character":i],"end":["line":0,"character":i]]]}
        var locations=semanticLocations(values,defaultURI:"hick:///src/Widget.py")
        locations.append(SemanticLocation(uri:"hick:///foreign.py",line:0,character:0,name:"foreign",kind:5))
        let knowledge=[AnalysisKnowledge(path:file.path,source:"source",locations:locations,state:"ready",notice:"")],commands=[WorkspaceCommand(id:"symbol.rename",label:"Rename symbol",group:"Edit",reason:"Open source first.",run:{})]
        func targets()->[SearchTarget] {everywhereTargets([file,file,generated],knowledge:knowledge,current:{_ in source},commands:commands)}
        XCTAssertEqual(targets().count,6);XCTAssertEqual(searchEverywhere(targets(),query:"wdgt",kind:"type").items.first?.label,"Widget")
        XCTAssertEqual(searchEverywhere(targets(),query:"render",kind:"member").total,1);XCTAssertEqual(searchEverywhere(targets(),query:"unknown",kind:"symbol").total,1)
        XCTAssertEqual(searchEverywhere(targets(),query:"rename",kind:"action").items.first?.reason,"Open source first.");XCTAssertTrue(searchEverywhere(targets(),query:"out.py").items.first?.detail.contains("read-only")==true)
        XCTAssertEqual(searchEverywhere(targets(),query:"",limit:1).items.count,1);XCTAssertEqual(searchEverywhere(targets(),query:"",limit:1).total,6)
        source="draft";XCTAssertEqual(targets().count,3);XCTAssertEqual(searchEverywhere(targets(),query:"",kind:"type").total,0)
        XCTAssertEqual(searchEverywhere([SearchTarget(id:"emoji",kind:"file",label:"😀x😀",detail:"")],query:"😀😀").total,1)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SearchEverywhereTests.swift"><ui:paste select="#simple-search-everywhere-apple-tests"/></ui:file>
</ui:doc>
