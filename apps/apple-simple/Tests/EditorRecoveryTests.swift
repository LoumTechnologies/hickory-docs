import XCTest
import SwiftUI
@testable import HickorySimple
#if os(macOS)
import AppKit
#endif
@MainActor final class EditorRecoveryTests:XCTestCase {
 func buffer(_ path:String,_ text:String="draft 😀",unnamed:Bool=false,owner:String?=nil)->DocumentBuffer{DocumentBuffer(file:FileNode(name:path,path:path,dir:false,generated_by:owner),text:FileText(path:path,content:text,hash:"old",save_guard:"original"),isUntitledDraft:unnamed)}
 func wire(_ raw:[String:Any])throws->ResourceJSON{try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:raw))}
 func testRawReviewRefusesDuplicateAliasedVersionsAndExcessiveNesting()throws {
  for raw in [#"{"version":1,"version":1}"#,#"{"version":1,"recovery_protocol":1.0}"#,String(repeating:"[",count:65)+"0"+String(repeating:"]",count:65)]{XCTAssertThrowsError(try validateNativeReviewJSON(Data(raw.utf8),maxBytes:1024,versionKeys:["version","recovery_protocol"],label:"Recovery"))}
 }
 func testCaptureKeepsLiteralReservedAndUntitledNamedSourcesDistinct()throws {
  let w=Workspace();for path in ["files","agent","terminal","Untitled-real.txt","é.txt","e\u{301}.txt"]{let b=buffer(path);w.buffers[path]=b;w.layout.activateSource(path,in:"document-group")};let note=buffer("anonymous","note",unnamed:true);w.buffers[note.file.path]=note;let generated=buffer("out.py","output",owner:"project.md");w.buffers[generated.file.path]=generated
  let snapshot=try captureEditorRecovery(w);XCTAssertEqual(snapshot.buffers.count,8);XCTAssertEqual(snapshot.buffers.filter{$0.address.kind=="source"}.count,6);XCTAssertEqual(snapshot.buffers.filter{$0.address.kind=="unnamed"}.count,1);XCTAssertEqual(snapshot.buffers.filter{$0.address.kind=="generated"}.count,1);XCTAssertTrue(snapshot.buffers.allSatisfy{$0.baseline?.save_guard=="original"});try validateEditorRecoverySnapshot(snapshot)
 }
 func testColdRestoreAddsCopiesWithoutGuardsAndLeavesExistingOwnerIntact()throws {
  let old=Workspace(),source=buffer("files"),output=buffer("out.py","generated",owner:"project.md");old.buffers[source.file.path]=source;old.buffers[output.file.path]=output;old.layout.activateSource("files",in:"document-group");let snapshot=try captureEditorRecovery(old)
  let fresh=Workspace(),independent=buffer("files","independent draft");fresh.buffers["files"]=independent;fresh.layout.activateSource("files",in:"document-group");let identity=independent.sourceEditIdentity;try restoreEditorRecovery(snapshot,into:fresh)
  XCTAssertTrue(fresh.buffers["files"] === independent);XCTAssertEqual(independent.text,"independent draft");XCTAssertEqual(independent.sourceEditIdentity,identity);let recovered=fresh.buffers.values.filter{$0 !== independent};XCTAssertEqual(recovered.count,2);XCTAssertTrue(recovered.allSatisfy{$0.savedGuard==nil});XCTAssertEqual(recovered.filter{$0.readOnly}.count,1);XCTAssertEqual(recovered.filter{$0.isUntitledDraft}.count,1);XCTAssertTrue(recovered.allSatisfy{$0.recoveryBaseline?.save_guard=="original"})
 }
 func testMalformedLayoutCannotImportAnyPartialBuffer()throws {
  let old=Workspace(),b=buffer("files");old.buffers[b.file.path]=b;var snapshot=try captureEditorRecovery(old);snapshot.layout = .object(["tree":.object([:])]);let target=Workspace();XCTAssertThrowsError(try restoreEditorRecovery(snapshot,into:target));XCTAssertTrue(target.buffers.isEmpty)
 }
 func testSelectionAndFullRetainedLeaseRefuseInvalidRecovery()throws {
  let w=Workspace(),b=buffer("files");w.buffers[b.file.path]=b;var snapshot=try captureEditorRecovery(w);snapshot.buffers[0].view.selection.head=7;XCTAssertThrowsError(try validateEditorRecoverySnapshot(snapshot));let lease=try editorRecoveryLease(w);XCTAssertTrue(lease());w.retainedResourceEditors["new"]=buffer("later","note",unnamed:true);XCTAssertFalse(lease())
 }
 func testExactUncertainRequestPrecedesANewerCheckpoint()async throws {
  let w=Workspace();w.engine=try Engine(address:"http://fixture.invalid");w.sourceWorkspaceID=String(repeating:"e",count:32);w.sourceRootAddress="/owned";let b=buffer("files");w.buffers[b.file.path]=b;let model=w.editorRecovery;var requests:[[String:Any]]=[],lose=true
  model.transport={method,_,body in
   if method=="GET"{return try self.wire(["version":1,"recovery_protocol":1,"engine_workspace_id":String(repeating:"e",count:32),"workspace_id":String(repeating:"a",count:64),"namespace":"/owned","window_slot":"0","current_window_slot":"0","window_slots":["0"],"sessions":[],"notices":[],"next":NSNull(),"source_effects":false])}
   let body=body!;requests.append(body);if lose{lose=false;throw EngineError(message:"Lost response")};return try self.wire(["version":1,"recovery_protocol":1,"client_id":body["client_id"]!,"window_slot":"0","workspace_id":String(repeating:"a",count:64),"namespace":"/owned","request_id":body["request_id"]!,"revision":(body["base_revision"] as! Int)+1,"snapshot_hash":String(repeating:"f",count:64),"source_effects":false,"source_saved":false])
  }
  await model.checkpoint(w);b.text="newer draft";await model.checkpoint(w);XCTAssertEqual(requests.count,2);XCTAssertEqual(requests[0]["request_id"] as? String,requests[1]["request_id"] as? String);XCTAssertEqual(try JSONSerialization.data(withJSONObject:requests[0]["snapshot"]!,options:[.sortedKeys]),try JSONSerialization.data(withJSONObject:requests[1]["snapshot"]!,options:[.sortedKeys]));XCTAssertEqual(model.revision,1);await model.checkpoint(w);XCTAssertEqual(model.revision,2);XCTAssertNotEqual(requests[1]["request_id"] as? String,requests[2]["request_id"] as? String)
 }

 func managementHost()throws->(Workspace,DocumentBuffer){let w=Workspace();w.engine=try Engine(address:"http://fixture.invalid");w.sourceWorkspaceID=String(repeating:"e",count:32);w.sourceRootAddress="/owned";let b=buffer("files");w.buffers[b.file.path]=b;return(w,b)}
 func managementRow(_ client:String=String(repeating:"c",count:32))->[String:Any]{let versions=(1...3).map{revision in ["revision":revision,"request_id":String(format:"%032x",revision),"snapshot_hash":String(repeating:String(revision),count:64),"host":"macos","buffers":1] as [String:Any]};return ["client_id":client,"window_slot":"0","workspace_id":String(repeating:"a",count:64),"namespace":"/owned","current_workspace":true,"revision":3,"request_id":String(format:"%032x",3),"snapshot_hash":String(repeating:"3",count:64),"host":"macos","buffers":1,"source_effects":false,"revisions":versions,"record_guard":"original private physical guard"]}
 func managementMeta(_ rows:[[String:Any]],next:String?=nil,namespace:String="/owned")->[String:Any]{["version":1,"recovery_protocol":1,"engine_workspace_id":String(repeating:"e",count:32),"workspace_id":String(repeating:"a",count:64),"namespace":namespace,"window_slot":"0","current_window_slot":"0","window_slots":["0"],"sessions":rows,"unreadable_sessions":[],"notices":[],"next":next as Any? ?? NSNull(),"source_effects":false]}
 func testManagementReachesAll32SessionsAndPreservesCatalogueOnBadPage()async throws {
  let(w,_)=try managementHost(),model=w.editorRecovery;let rows=(1...32).map{managementRow(String(format:"%032x",$0))};var bad=false
  model.transport={_,path,_ in let cursor=URLComponents(string:path)?.queryItems?.first{$0.name=="after"}?.value;let offset=cursor.flatMap{Int($0,radix:16)} ?? 0;return try self.wire(self.managementMeta(Array(rows[offset..<min(offset+8,32)]),next:offset+8<32 ? String(format:"%032x",offset+8):nil,namespace:bad ? "/wrong":"/owned"))}
  await model.open(w);XCTAssertEqual(model.catalogue?.sessions.count,8);bad=true;await model.more(w);XCTAssertEqual(model.catalogue?.sessions.count,8);bad=false;for _ in 0..<3{await model.more(w)};XCTAssertEqual(model.catalogue?.sessions.count,32);XCTAssertNil(model.catalogue?.next)
 }
 func testEveryRetainedRevisionReadsItsOwnCompleteMaterial()async throws {
  let(w,b)=try managementHost(),model=w.editorRecovery;var versions:[[String:Any]]=[],envelopes:[UInt32:[String:Any]]=[:]
  for revision in 1...3{var snapshot=try captureEditorRecovery(w);snapshot.buffers[0].text="retained \(revision) 😀";let material=String(data:try editorRecoveryData(snapshot),encoding:.utf8)!,hash=nativeSourceHash(material),request=String(format:"%032x",revision);versions.append(["revision":revision,"request_id":request,"snapshot_hash":hash,"host":snapshot.host,"buffers":1]);envelopes[UInt32(revision)]=["version":1,"recovery_protocol":1,"engine_workspace_id":w.sourceWorkspaceID!,"client_id":String(repeating:"c",count:32),"window_slot":"0","workspace_id":String(repeating:"a",count:64),"namespace":"/owned","revision":revision,"request_id":request,"snapshot_hash":hash,"host":snapshot.host,"buffers":1,"material":material,"source_effects":false,"source_saved":false,"current_workspace":true,"authority":[["buffer_id":snapshot.buffers[0].id,"baseline_current":false,"write_permission":false]]]}
  var raw=managementRow();raw["revisions"]=versions;for key in ["revision","request_id","snapshot_hash","host","buffers"]{raw[key]=versions.last![key]};let row=try JSONDecoder().decode(EditorRecoverySession.self,from:JSONSerialization.data(withJSONObject:raw));var requested:[UInt32]=[]
  model.transport={_,path,_ in let revision=UInt32(URLComponents(string:path)!.queryItems!.first{$0.name=="revision"}!.value!)!;requested.append(revision);return try self.wire(envelopes[revision]!)}
  for revision in [UInt32(1),2,3]{await model.read(row,w,revision:revision);XCTAssertEqual(model.snapshot?.buffers[0].text,"retained \(revision) 😀",model.notice);XCTAssertEqual(model.readingRevision,revision)};await model.read(row,w,revision:9);XCTAssertEqual(requested,[1,2,3]);XCTAssertNil(model.snapshot);XCTAssertEqual(b.text,"draft 😀")
 }
 func testLostRemovalKeepsExactRequestPausesCaptureAndPreservesReplacementOnReplay()async throws {
  let(w,b)=try managementHost(),model=w.editorRecovery;var row=managementRow(),requests:[[String:Any]]=[],lose=true
  model.transport={method,path,body in if method=="GET"{return try self.wire(self.managementMeta([row]))};XCTAssertFalse(path.hasSuffix("checkpoint"));let body=body!;requests.append(body);if lose{lose=false;throw EngineError(message:"Lost private removal acknowledgment")};return try self.wire(["version":1,"recovery_protocol":1,"engine_workspace_id":w.sourceWorkspaceID!,"workspace_id":String(repeating:"a",count:64),"namespace":"/owned","window_slot":"0","client_id":body["client_id"]!,"request_id":body["request_id"]!,"guard_hash":nativeSourceHash(body["record_guard"] as! String),"forgotten":true,"replayed":true,"source_effects":false,"source_saved":false])}
  await model.open(w);let original=try XCTUnwrap(model.catalogue?.sessions.first);await model.forget(original.removal,w);XCTAssertTrue(model.hasPendingForget);await model.checkpoint(w);XCTAssertEqual(requests.count,1);row["record_guard"]="new private physical guard";await model.open(w);await model.retryForget(w);XCTAssertEqual(requests[0]["request_id"] as? String,requests[1]["request_id"] as? String);XCTAssertEqual(requests[0]["record_guard"] as? String,requests[1]["record_guard"] as? String);XCTAssertFalse(model.hasPendingForget);XCTAssertEqual(model.catalogue?.sessions.first?.record_guard,"new private physical guard");XCTAssertEqual(b.text,"draft 😀")
 }
 func testForgedPrivateRemovalReceiptRetainsRequestAndStopRetryDoesNotDispatchAgain()async throws {
  let(w,b)=try managementHost(),model=w.editorRecovery;var posts=0
  model.transport={method,_,body in if method=="GET"{return try self.wire(self.managementMeta([self.managementRow()]))};posts+=1;let body=body!;return try self.wire(["version":1,"recovery_protocol":1,"engine_workspace_id":w.sourceWorkspaceID!,"workspace_id":String(repeating:"a",count:64),"namespace":"/owned","window_slot":"0","client_id":body["client_id"]!,"request_id":body["request_id"]!,"guard_hash":nativeSourceHash(body["record_guard"] as! String),"forgotten":true,"replayed":false,"source_effects":false,"source_saved":true])}
  await model.open(w);await model.forget(try XCTUnwrap(model.catalogue?.sessions.first).removal,w);XCTAssertTrue(model.hasPendingForget);XCTAssertEqual(model.catalogue?.sessions.count,1);model.stopRetryingForget();XCTAssertFalse(model.hasPendingForget);XCTAssertEqual(posts,1);XCTAssertEqual(b.text,"draft 😀")
 }
 func testMalformedRetainedRevisionOrderRefusesEntireCatalogue()async throws {
  let(w,_)=try managementHost(),model=w.editorRecovery;var row=managementRow();row["revisions"]=[(row["revisions"] as! [[String:Any]])[2],(row["revisions"] as! [[String:Any]])[0]];model.transport={_,_,_ in try self.wire(self.managementMeta([row]))};await model.open(w);XCTAssertNil(model.catalogue)
 }
 #if os(macOS)
 func testLiveOwnedProcessRestartRecovery()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_SESSION_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_NATIVE_SESSION_WORKSPACE"],let phase=ProcessInfo.processInfo.environment["HICKORY_NATIVE_SESSION_PHASE"] else{throw XCTSkip("Use the maintained owned two-process editor-session fixture.")}
  let directory=URL(fileURLWithPath:root);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(".hickory-native-session-fixture"),encoding:.utf8),"owned-native-session-v1\n")
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let listing=try await engine.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
  func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if exactUTF8(node.path,path){return node};if let child=find(path,node.children ?? []){return child}};return nil}
  let file=try XCTUnwrap(find(phase=="capture" ? "files":"note.py",w.rootFiles)),b=try await w.loadBuffer(file);w.layout.activateSource(file.path,in:"document-group");let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:900,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
  defer{w.editorRecovery.close();w.debugger?.close();w.sourceTestIndex?.stop();for item in w.buffers.values{item.language?.close()};window.orderOut(nil);window.contentView=nil}
  func editors(_ view:NSView)->[NSTextView]{(view as? NSTextView).map{[$0]} ?? view.subviews.flatMap{editors($0)}}
  func settle(_ message:String,_ predicate:()->Bool)async throws{for _ in 0..<300{if predicate(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:message)}
  try await settle("Native source did not attach"){!editors(host).isEmpty};let editor=try XCTUnwrap(editors(host).first),scroll=try XCTUnwrap(editor.enclosingScrollView),identity=try XCTUnwrap(b.sourceEditorState?()).identity,manager=try XCTUnwrap(editor.undoManager)
  if phase=="capture" {
   editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("\nunsaved λ😀",replacementRange:editor.selectedRange());editor.setSelectedRange(NSRange(location:9,length:3));b.selection=editor.selectedRange();scroll.contentView.scroll(to:CGPoint(x:0,y:180));scroll.reflectScrolledClipView(scroll.contentView)
   let note=buffer("anonymous","anonymous retained note",unnamed:true);w.buffers[note.file.path]=note;let source=b.text,selection=b.selection,viewport=scroll.contentView.bounds.origin
   var observed=false
   for _ in 0..<100 {
    let catalogue:EditorRecoveryCatalogue=try await engine.request("GET","/api/workspace/recovery")
    for row in catalogue.sessions where row.host=="macos"&&row.buffers==2 {
     let envelope:EditorRecoveryEnvelope=try await engine.request("GET","/api/workspace/recovery/"+row.client_id,query:[URLQueryItem(name:"revision",value:String(row.revision)),URLQueryItem(name:"window_slot",value:row.window_slot)])
     let snapshot=try JSONDecoder().decode(EditorRecoverySnapshot.self,from:Data(envelope.material.utf8));if snapshot.buffers.contains(where:{exactUTF8($0.address.path,"files")&&exactUTF8($0.text,source)&&$0.view.selection.anchor==selection.location&&$0.view.selection.head==NSMaxRange(selection)&&abs($0.view.scroll_y-viewport.y)<1})&&snapshot.buffers.contains(where:{$0.address.kind=="unnamed"&&$0.text=="anonymous retained note"}){observed=true;break}
    };if observed{break};try await Task.sleep(for:.milliseconds(100))
   }
   XCTAssertTrue(observed,"Automatic native checkpoint did not retain the exact source, unnamed note and view.");XCTAssertFalse(try String(contentsOf:directory.appendingPathComponent("files"),encoding:.utf8).contains("unsaved"));return
  }
  XCTAssertEqual(phase,"restore");editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("# independent new draft",replacementRange:editor.selectedRange());editor.setSelectedRange(NSRange(location:2,length:2));b.selection=editor.selectedRange();let before=b.text,selection=b.selection
  try XCTUnwrap(w.commands().first{$0.id=="workspace.recovery"}).run();XCTAssertTrue(w.showRecovery);w.showRecovery=false;await w.editorRecovery.open(w);try await settle("Session catalogue did not settle"){!w.editorRecovery.busy&&w.editorRecovery.catalogue != nil}
  let row=try XCTUnwrap(w.editorRecovery.catalogue?.sessions.first{$0.host=="macos"&&$0.buffers==2});await w.editorRecovery.read(row,w);try await settle("Session reading did not settle"){!w.editorRecovery.busy};let snapshot=try XCTUnwrap(w.editorRecovery.snapshot,w.editorRecovery.notice),saved=try XCTUnwrap(snapshot.buffers.first{$0.address.path=="files"});XCTAssertTrue(w.editorRecovery.canRestore,w.editorRecovery.notice)
  editor.setSelectedRange(NSRange(location:3,length:0));b.selection=editor.selectedRange();w.editorRecovery.restore(w);XCTAssertNil(w.buffers.values.first{$0.isUntitledDraft&&$0.text==saved.text});XCTAssertTrue(w.editorRecovery.notice.contains("lease changed"));editor.setSelectedRange(selection);b.selection=selection;await w.editorRecovery.read(row,w);w.editorRecovery.restore(w)
  let restored=try XCTUnwrap(w.buffers.values.first{$0.isUntitledDraft&&$0.text==saved.text},w.editorRecovery.notice);XCTAssertNil(restored.savedGuard);XCTAssertTrue(w.currentBuffer === restored);w.editorRecovery.close();host.layoutSubtreeIfNeeded();try await settle("Recovered native view did not restore its viewport"){restored.recoveryViewport==nil&&restored.sourceEditorState?()?.attached==true};XCTAssertEqual(restored.selection.location,saved.view.selection.anchor);XCTAssertEqual(NSMaxRange(restored.selection),saved.view.selection.head);XCTAssertEqual(restored.captureRecoveryViewport?().y ?? -1,saved.view.scroll_y,accuracy:1)
  XCTAssertTrue(w.buffers["note.py"] === b);XCTAssertEqual(b.text,before);XCTAssertEqual(b.selection,selection);XCTAssertEqual(b.sourceEditorState?()?.identity,identity);manager.undo();XCTAssertEqual(b.text,"value=1\n");XCTAssertFalse(try String(contentsOf:directory.appendingPathComponent("files"),encoding:.utf8).contains("unsaved"));XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("note.py"),encoding:.utf8),"value=1\n")
 }
 #endif

}
