import XCTest
@testable import HickorySimple
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
final class ResourceEditorsTests:XCTestCase {
 final class Editor {var selection=2;var scroll=108;var undo=["old"]}
 func testMoveDeleteRestoreAndCreateKeepOpaqueEditorState() throws {
  let editor=Editor(),a=ResourceSource(identity:"a",text:"dirty"),capture=ResourceEditorCapture(editor:editor,source:"dirty")
  let move=try routeResourceEditors([ResourceEditorPatch(path:"a.py",before:a,after:nil),ResourceEditorPatch(path:"b.py",before:nil,after:a)],live:["a":capture]);XCTAssertTrue(move[0].editor===editor);XCTAssertEqual(move[0].to,"b.py")
  let other=Editor(),b=ResourceSource(identity:"b",text:"B"),otherCapture=ResourceEditorCapture(editor:other,source:"B")
  let swapped=try routeResourceEditors([ResourceEditorPatch(path:"a.py",before:a,after:b),ResourceEditorPatch(path:"b.py",before:b,after:a)],live:["a":capture,"b":otherCapture]);XCTAssertTrue(swapped.first(where:{$0.to=="b.py"})?.editor===editor);XCTAssertTrue(swapped.first(where:{$0.to=="a.py"})?.editor===other)
  let deleted=try routeResourceEditors([ResourceEditorPatch(path:"b.py",before:a,after:nil)],live:["a":capture]);XCTAssertNil(deleted[0].to);XCTAssertTrue(deleted[0].editor===editor)
  let restored=try routeResourceEditors([ResourceEditorPatch(path:"b.py",before:nil,after:ResourceSource(identity:"restored",text:"dirty"))],live:[:],retained:["restored":capture]);XCTAssertTrue(restored[0].editor===editor)
  let created:[ResourceEditorRoute<Editor>]=try routeResourceEditors([ResourceEditorPatch(path:"new.py",before:nil,after:ResourceSource(identity:"new",text:""))],live:[:]);XCTAssertNil(created[0].editor);XCTAssertEqual(created[0].after,"")
  XCTAssertEqual(editor.selection,2);XCTAssertEqual(editor.scroll,108);XCTAssertEqual(editor.undo,["old"])
 }
 func testConfirmedReceiptRequiresWholeNamespaceAndSeparateSavedBaseline() throws {
  let saved=ResourceSaved(kind:"file",text:"saved",sha256:String(repeating:"a",count:64),readonly:false,executable:false,mode:nil)
  let absent=ResourceSaved(kind:"missing"),patches=[ResourceEditorPatch(path:"old.py",before:ResourceSource(identity:"a",text:"dirty"),after:nil),ResourceEditorPatch(path:"new.py",before:nil,after:ResourceSource(identity:"a",text:"dirty"))]
  var receipt=ResourcePublication<String>(version:1,request_id:"request",workspace:"workspace",context:"context",outcome:.init(complete:true,recovery:"record",applied:["old.py","new.py"]),saved_sources:["old.py":absent,"new.py":saved],source_files:[.init(file:"new.py",saved:saved,hash:"fresh")])
  func confirm() throws->[Data:ResourcePublication<String>.Entry] {try confirmedResourceFiles(patches,receipt:receipt,request:"request",workspace:"workspace",context:"context",expected:["old.py":absent,"new.py":saved]){($0,true)}}
  XCTAssertEqual(try confirm()[Data("new.py".utf8)]?.saved.text,"saved")
  receipt.outcome.complete=false;XCTAssertThrowsError(try confirm());receipt.outcome.complete=true
  receipt.request_id="other";XCTAssertThrowsError(try confirm());receipt.request_id="request"
  receipt.source_files.append(receipt.source_files[0]);XCTAssertThrowsError(try confirm())
 }
 func testCompletePreflightAndByteExactNamespaces() throws {
  let editor=Editor(),capture=ResourceEditorCapture(editor:editor,source:"a"),a=ResourceSource(identity:"a",text:"a"),patch=ResourceEditorPatch(path:"a.py",before:a,after:nil)
  XCTAssertThrowsError(try routeResourceEditors([patch,patch],live:["a":capture]));XCTAssertThrowsError(try routeResourceEditors([patch],live:[String:ResourceEditorCapture<Editor>]()))
  XCTAssertThrowsError(try routeResourceEditors([patch],live:["a":ResourceEditorCapture(editor:editor,source:"later")]))
  XCTAssertThrowsError(try routeResourceEditors([patch],live:["a":capture],retained:["a":capture]))
  let routes:[ResourceEditorRoute<Editor>]=try routeResourceEditors([ResourceEditorPatch(path:"é.py",before:nil,after:ResourceSource(identity:"first",text:"")),ResourceEditorPatch(path:"e\u{301}.py",before:nil,after:ResourceSource(identity:"second",text:""))],live:[:]);XCTAssertEqual(routes.count,2)
  XCTAssertEqual(editor.undo,["old"])
 }
}
