import XCTest
@testable import HickorySimple
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
final class ResourceEditorsTests:XCTestCase {
 final class Editor {var selection=2;var scroll=108;var undo=["old"]}
 func testTabAddressesMoveSimultaneouslyWithoutChangingPaneGeometry() throws {
  var layout=PaneLayout();layout.activate("a.py",in:"document-group");layout.activate("b.py",in:"document-group")
  let before=layout
  let swapped=try layout.remappingTabs([Data("a.py".utf8):"b.py",Data("b.py".utf8):"a.py"])
  XCTAssertEqual(swapped.nodes["document-group"]?.tabs,["b.py","a.py"]);XCTAssertEqual(swapped.nodes["document-group"]?.active,"a.py")
  XCTAssertEqual(swapped.nodes["root"],before.nodes["root"]);XCTAssertEqual(swapped.rootID,before.rootID)
  XCTAssertThrowsError(try layout.remappingTabs([Data("a.py".utf8):"b.py"]))
  XCTAssertEqual(layout.nodes,before.nodes)
 }
 @MainActor func testLiveReceiptAdoptsMoveDeletionAndExplicitRetainedRestoration() throws {
  let workspace=Workspace();workspace.engine=try Engine(address:"http://127.0.0.1:1")
  let source=ResourceSource(identity:"source",text:"dirty"),missing=ResourceSaved(kind:"missing")
  let saved=ResourceSaved(kind:"file",text:"saved",sha256:String(repeating:"a",count:64),readonly:false,executable:false,mode:nil)
  let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"saved",hash:"old"));buffer.text="dirty";buffer.selection=NSRange(location:2,length:1)
  workspace.buffers["a.py"]=buffer;workspace.layout.activate("a.py",in:"document-group")
  let geometry=workspace.layout.nodes["root"]
  func receipt(_ paths:[String],_ values:[String:ResourceSaved],_ path:String?)->ResourcePublication<FileNode> {
   ResourcePublication(version:2,request_id:"request",workspace:"workspace",context:"context",outcome:.init(complete:true,recovery:"record",applied:paths),saved_sources:values,source_files:path.map{[.init(file:FileNode(name:$0,path:$0,dir:false),saved:saved,hash:"fresh")]} ?? [],adoption_current:true)
  }
  let move=[ResourceEditorPatch(path:"a.py",before:source,after:nil),ResourceEditorPatch(path:"b.py",before:nil,after:source)]
  let moving=workspace.captureResourceEditors(move)
  _ = try workspace.adoptResourceReceipt(move,receipt:receipt(["a.py","b.py"],["a.py":missing,"b.py":saved],"b.py"),request:"request",workspace:"workspace",context:"context",expected:["a.py":missing,"b.py":saved],current:moving)
  XCTAssertTrue(workspace.buffers["b.py"] === buffer);XCTAssertNil(workspace.buffers["a.py"]);XCTAssertEqual(workspace.active,"b.py");XCTAssertTrue(workspace.currentBuffer === buffer)
  XCTAssertEqual(buffer.selection,NSRange(location:2,length:1));XCTAssertEqual(buffer.text,"dirty");XCTAssertEqual(buffer.base,"saved");XCTAssertTrue(buffer.dirty)
  let deletion=[ResourceEditorPatch(path:"b.py",before:source,after:nil)]
  _ = try workspace.adoptResourceReceipt(deletion,receipt:receipt(["b.py"],["b.py":missing],nil),request:"request",workspace:"workspace",context:"context",expected:["b.py":missing],current:workspace.captureResourceEditors(deletion))
  let retained=workspace.active!
  XCTAssertTrue(buffer.readOnly);XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertNil(workspace.buffers["b.py"]);XCTAssertTrue(workspace.hasUnsavedChanges)
  workspace.close(retained);XCTAssertEqual(workspace.active,retained)
  let restore=[ResourceEditorPatch(path:"b.py",before:nil,after:source)]
  let restoring=["source":buffer]
  _ = try workspace.adoptResourceReceipt(restore,receipt:receipt(["b.py"],["b.py":saved],"b.py"),request:"request",workspace:"workspace",context:"context",expected:["b.py":saved],current:workspace.captureResourceEditors(restore,restoring:restoring),restoring:restoring)
  XCTAssertTrue(workspace.currentBuffer === buffer);XCTAssertEqual(workspace.active,"b.py");XCTAssertFalse(buffer.readOnly);XCTAssertTrue(workspace.retainedResourceEditors.isEmpty)
  XCTAssertEqual(workspace.layout.nodes["root"],geometry);XCTAssertEqual(buffer.selection,NSRange(location:2,length:1))
  buffer.language?.close()
 }

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
  var receipt=ResourcePublication<String>(version:2,request_id:"request",workspace:"workspace",context:"context",outcome:.init(complete:true,recovery:"record",applied:["old.py","new.py"]),saved_sources:["old.py":absent,"new.py":saved],source_files:[.init(file:"new.py",saved:saved,hash:"fresh")],adoption_current:true)
  func confirm() throws->[Data:ResourcePublication<String>.Entry] {try confirmedResourceFiles(patches,receipt:receipt,request:"request",workspace:"workspace",context:"context",expected:["old.py":absent,"new.py":saved]){($0,true)}}
  XCTAssertEqual(try confirm()[Data("new.py".utf8)]?.saved.text,"saved")
  receipt.version=1;XCTAssertThrowsError(try confirm());receipt.version=2
  receipt.adoption_current=false;receipt.adoption_reason="Historical publication; source was replaced.";XCTAssertThrowsError(try confirm());receipt.adoption_current=true
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
