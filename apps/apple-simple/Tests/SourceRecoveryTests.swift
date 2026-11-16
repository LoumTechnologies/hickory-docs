import XCTest
@testable import HickorySimple
@MainActor final class SourceRecoveryTests:XCTestCase {
 let record=String(repeating:"a",count:32),workspace=String(repeating:"b",count:32)
 func page()->RecoveryPage{RecoveryPage(version:1,workspace:workspace,ids:[record],ignored:0)}
 func reading(_ hash:String=String(repeating:"c",count:64))->RecoveryReading{RecoveryReading(version:1,workspace:workspace,id:record,fingerprint:hash,title:"Move",title_truncated:false,paths:[RecoveryPath(path:"a #😀.md",saved_before:RecoverySummary(kind:"file",bytes:0),saved_after:RecoverySummary(kind:"missing"),draft_before:RecoverySummary(kind:"file",bytes:4),draft_after:RecoverySummary(kind:"missing"))],all_paths_recorded_applied:false,unrecognized_files:[],draft_text:"😀",saved_text:"")}
 func testExactSnapshotRequestsAndChangedEvidence()async{
  let model=RecoveryReader();await model.list{_ in self.page()};var route="";await model.read(record,path:"a #😀.md",snapshot:"saved_before"){route=$0;return self.reading()};XCTAssertTrue(route.contains("path=a%20%23%F0%9F%98%80.md&saved=before"));XCTAssertEqual(model.value?.saved_text,"");XCTAssertEqual(model.value?.draft_text,"😀")
  await model.read(record,path:"a #😀.md"){_ in self.reading(String(repeating:"d",count:64))};XCTAssertNil(model.value);XCTAssertTrue(model.notice.contains("evidence changed"))
  await model.list{_ in self.page()};await model.list(after:record){_ in var page=self.page();page.workspace="other";page.ids=[];return page};XCTAssertNil(model.page)
 }
 func testClosedAndSupersededReadsCannotReplaceSelection()async{
  let model=RecoveryReader();await model.list{_ in self.page()};var pending:CheckedContinuation<RecoveryReading,Never>?
  let old=Task{await model.read(record){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};await model.list{_ in self.page()};pending?.resume(returning:reading());await old.value;XCTAssertNil(model.value)
  pending=nil;let closed=Task{await model.read(record){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:reading());await closed.value;XCTAssertNil(model.value);XCTAssertNil(model.page)
 }
 func testRecoveryIdentityUsesExactManifestAddress()throws {
  XCTAssertEqual(try resourceRecoveryIdentity(".hick-cache/resource-changes/"+record+"/manifest.json"),record)
  for value in [record,"../.hick-cache/resource-changes/"+record+"/manifest.json",".hick-cache/resource-changes/"+record+"/../manifest.json",".hick-cache/resource-changes/"+record.uppercased()+"/manifest.json"] {XCTAssertThrowsError(try resourceRecoveryIdentity(value))}
 }
 private func json(_ value:Any)throws->ResourceJSON {try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:value))}
 private func publicationFixture(failWrite:Bool=false)throws->(Workspace,ResourceRestoration,RecoveryReading,()->Int) {
  let space=Workspace();space.engine=try Engine(address:"http://127.0.0.1:1")
  let buffer=DocumentBuffer(file:FileNode(name:"b.py",path:"b.py",dir:false),text:FileText(path:"b.py",content:"dirty",hash:"old"));buffer.selection=NSRange(location:2,length:0);space.buffers["b.py"]=buffer;space.layout.activate("b.py",in:"document-group")
  let model=space.resourceRestoration,missing:[String:Any]=["kind":"missing"],source:[String:Any]=["kind":"file","identity":"source","text":"dirty","writable":true,"removable":true,"version":NSNull()],after:[String:Any]=["kind":"file","identity":"source","text":"prior dirty","writable":true,"removable":true,"version":NSNull()],absent:[String:Any]=["kind":"missing","creatable":true]
  func saved(_ text:String)->[String:Any] {["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":NSNull()]}
  let summary=RecoverySummary(kind:"file",bytes:5),recordValue=RecoveryReading(version:1,workspace:workspace,id:record,fingerprint:String(repeating:"c",count:64),title:"Move",title_truncated:false,paths:[RecoveryPath(path:"a.py",saved_before:summary,saved_after:RecoverySummary(kind:"missing"),draft_before:summary,draft_after:RecoverySummary(kind:"missing")),RecoveryPath(path:"b.py",saved_before:RecoverySummary(kind:"missing"),saved_after:summary,draft_before:RecoverySummary(kind:"missing"),draft_after:summary)],all_paths_recorded_applied:true,unrecognized_files:[])
  var context="",writes=0,request=""
  model.transport={method,route,body in
   if route=="/api/changes/resource-inputs" {
    context=body!["context"] as! String
    let drafts=body!["drafts"] as! [String:[String:Any]];XCTAssertEqual(drafts["b.py"]?["base_sha256"] as? String,nativeSourceHash("dirty"))
    return try self.json(["version":1,"context":context,"workspace":self.workspace,"ownership":["mode":"ordinary","entry":NSNull(),"protected_entries":[],"saved_sources":[:],"draft_sources":[:]],"sources":["a.py":absent,"b.py":source],"saved_sources":["a.py":missing,"b.py":saved("dirty")],"refusals":[:],"source_files_modified":false])
   }
   if route=="/api/changes/prepare-restoration" {
    let plan:[String:Any]=["version":1,"context":context,"title":"Restore Move","patches":[["path":"a.py","before":absent,"after":after],["path":"b.py","before":source,"after":absent]],"annotations":[]]
    let layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]]
    return try self.json(["version":1,"restoration":["version":1,"recovery_id":self.record,"recovery_fingerprint":String(repeating:"c",count:64),"workspace":self.workspace,"context":context,"plan":plan,"saved_before":["a.py":missing,"b.py":saved("dirty")],"saved_after":["a.py":saved("saved"),"b.py":missing]],"consequences":["version":1,"workspace":self.workspace,"context":context,"fingerprint":String(repeating:"d",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false]])
   }
   if method=="POST",route=="/api/changes/restore-resources" {writes+=1;request=body!["request_id"] as! String;if failWrite{throw EngineError(message:"Lost response")}}
   guard route=="/api/changes/restore-resources" || route=="/api/changes/resource-save?request_id="+request else{throw EngineError(message:"Unexpected resource request")}
   return try self.json(["version":2,"adoption_current":true,"request_id":request,"workspace":self.workspace,"context":context,"outcome":["complete":true,"recovery":".hick-cache/resource-changes/"+String(repeating:"e",count:32)+"/manifest.json","applied":["a.py","b.py"]],"saved_sources":["a.py":saved("saved"),"b.py":missing],"source_files":[["file":["name":"a.py","path":"a.py","dir":false],"saved":saved("saved"),"hash":"new"]]])
  }
  return (space,model,recordValue,{writes})
 }
 func testRestorationPublishesSeparateSavedAndDirtyDraftThroughLiveNamespace()async throws {
  let (space,model,value,writes)=try publicationFixture();let buffer=space.currentBuffer!
  await model.prepare(space,record:value);XCTAssertNotNil(model.review,model.notice);XCTAssertEqual(writes(),0)
  await model.publish(space);XCTAssertEqual(writes(),1);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(space.active,"a.py")
  XCTAssertEqual(buffer.text,"prior dirty");XCTAssertEqual(buffer.base,"saved");XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertEqual(model.lastRecovery,String(repeating:"e",count:32));buffer.language?.close()
 }
 func testNewOwnershipDraftRetiresReviewAndLostPublicationCannotReplay()async throws {
  let (space,model,value,writes)=try publicationFixture();await model.prepare(space,record:value);XCTAssertNotNil(model.review,model.notice)
  space.buffers["new.md"]=DocumentBuffer(file:FileNode(name:"new.md",path:"new.md",dir:false),text:FileText(path:"new.md",content:"",hash:""));space.buffers["new.md"]!.text="new ownership"
  await model.publish(space);XCTAssertEqual(writes(),0);XCTAssertNil(model.review);XCTAssertTrue(model.notice.contains("Source changed"))
  let (other,pending,reading,attempts)=try publicationFixture(failWrite:true);await pending.prepare(other,record:reading);await pending.publish(other);XCTAssertTrue(pending.hasPending);XCTAssertEqual(attempts(),1)
  await pending.publish(other);await pending.prepare(other,record:reading);XCTAssertEqual(attempts(),1);XCTAssertNil(pending.review)
  await pending.status(other);XCTAssertFalse(pending.hasPending,pending.notice);XCTAssertEqual(attempts(),1);XCTAssertEqual(other.active,"a.py");other.currentBuffer?.language?.close()
 }

 private func resourceSaveFixture()throws->(Workspace,ResourceRestoration,()->Int) {
  let (space,model,_,_)=try publicationFixture();let initial=model.transport!;var writes=0
  func saved(_ text:String)->[String:Any] {["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":NSNull()]}
  model.transport={method,route,body in
   if route=="/api/changes/resource-targets" {return try self.json(["version":1,"paths":["a.py","b.py"],"acquired":false,"saved":false])}
   if route=="/api/changes/plan-resources" {
    let sources=body!["sources"] as! [String:Any]
    return try self.json(["plan":["version":1,"context":body!["context"]!,"title":body!["title"]!,"patches":[["path":"a.py","before":sources["a.py"]!,"after":sources["b.py"]!],["path":"b.py","before":sources["b.py"]!,"after":sources["a.py"]!]],"annotations":[]]])
   }
   if route=="/api/changes/resource-consequences" {
    let capture=body!["capture"] as! [String:Any],layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]]
    return try self.json(["version":1,"workspace":self.workspace,"context":capture["context"]!,"fingerprint":String(repeating:"d",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false,"saved_sources":["a.py":saved("dirty"),"b.py":["kind":"missing"]]])
   }
   if route=="/api/changes/resource-save",method=="POST" {
    writes+=1;let proposal=body!["proposal"] as! [String:Any],capture=proposal["capture"] as! [String:Any]
    XCTAssertNil(body!["reviewed_recovery_fingerprint"])
    return try self.json(["version":2,"adoption_current":true,"request_id":body!["request_id"]!,"workspace":self.workspace,"context":capture["context"]!,"outcome":["complete":true,"recovery":".hick-cache/resource-changes/"+String(repeating:"f",count:32)+"/manifest.json","applied":["a.py","b.py"]],"saved_sources":["a.py":saved("dirty"),"b.py":["kind":"missing"]],"source_files":[["file":["name":"a.py","path":"a.py","dir":false],"saved":saved("dirty"),"hash":"new"]]])
   }
   return try await initial(method,route,body)
  }
  return (space,model,{writes})
 }
 func testProviderResourceActionUsesWholeReviewAndConfirmedSave()async throws {
  let (space,model,writes)=try resourceSaveFixture();let buffer=space.currentBuffer!,engine=space.engine!;buffer.language=NativeLanguage(engine:engine,buffer:buffer)
  let edit:[String:Any]=["documentChanges":[["kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///a.py"]]]
  space.changes.open("actions",workspace:space)
  await space.changes.prepare(space,proposal:{_,_ in [["title":"Move source","kind":"refactor.move"]]},resolution:{_ in ["title":"Move source","kind":"refactor.move","edit":edit]})
  await space.changes.choose(0,workspace:space)
  XCTAssertNotNil(model.review,model.notice);XCTAssertNotNil(model.review?.save);XCTAssertNil(space.changes.plan);XCTAssertEqual(writes(),0);XCTAssertEqual(space.active,"b.py")
  await model.publish(space);XCTAssertEqual(writes(),1);XCTAssertFalse(model.hasPending,model.notice);XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(space.active,"a.py");XCTAssertEqual(buffer.text,"dirty");XCTAssertEqual(buffer.base,"dirty");XCTAssertFalse(buffer.dirty)
  XCTAssertEqual(space.changes.resourceHistoryRecord("undo"),String(repeating:"f",count:32));buffer.language?.close()
 }
 func testMixedDraftAndResourceHistoryKeepsOrderingAndRetiresStaleIntent()async throws {
  let (space,model,_,_)=try publicationFixture();let buffer=space.currentBuffer!
  let edit:[String:Any]=["changes":["hick:///b.py":[["range":["start":["line":0,"character":0],"end":["line":0,"character":0]],"newText":"X"]]]]
  try space.changes.offer("Prefix",edit:edit,expected:["b.py":"dirty"],workspace:space);space.changes.apply(space);XCTAssertEqual(buffer.text,"Xdirty")
  space.changes.recordResourcePublication(record,intent:nil)
  let intent=ResourceHistoryIntent(direction:"undo",record:record,token:space.changes.historySnapshot)
  space.changes.recordResourcePublication(String(repeating:"e",count:32),intent:intent)
  XCTAssertNil(space.changes.resourceHistoryRecord("undo"));XCTAssertEqual(space.changes.resourceHistoryRecord("redo"),String(repeating:"e",count:32))
  space.changes.open("undo",workspace:space);XCTAssertEqual(buffer.text,"dirty");XCTAssertNil(space.changes.resourceHistoryRecord("redo"))
  space.changes.open("redo",workspace:space);XCTAssertEqual(buffer.text,"Xdirty");XCTAssertEqual(space.changes.resourceHistoryRecord("redo"),String(repeating:"e",count:32))
  await model.prepareHistory(space,intent:intent);XCTAssertNil(model.review);XCTAssertTrue(model.notice.contains("history changed"))
 }

 func testActualEngineResourceSaveRestorationAndRedoPreserveDirtyBaseline()async throws {
  #if os(macOS)
  let repository=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
  let executable=ProcessInfo.processInfo.environment["HICKORY_SIMPLE_TEST_ENGINE"] ?? repository.appendingPathComponent("target/source-edit-focus/debug/examples/engine_client").path
  guard FileManager.default.isExecutableFile(atPath:executable) else{throw XCTSkip("Build the focused engine_client fixture before native resource protocol acceptance.")}
  let temporary=FileManager.default.temporaryDirectory.appendingPathComponent("apple-resource-"+UUID().uuidString),root=temporary.appendingPathComponent("workspace")
  try FileManager.default.createDirectory(at:root,withIntermediateDirectories:true);defer{try? FileManager.default.removeItem(at:temporary)}
  let saved="saved baseline\n",draft=saved+"# retained draft\n";try Data(saved.utf8).write(to:root.appendingPathComponent("a.py"))
  let process=Process(),output=Pipe();process.executableURL=URL(fileURLWithPath:executable);process.arguments=[root.path,"window","http://127.0.0.1:1"];process.standardOutput=output;process.standardError=FileHandle.nullDevice
  var environment=ProcessInfo.processInfo.environment;environment["HICKORY_STATE_DIR"]=temporary.appendingPathComponent("state").path;process.environment=environment
  try process.run();defer{if process.isRunning{process.terminate()};process.waitUntilExit()}
  let greeting=try XCTUnwrap(String(data:output.fileHandleForReading.availableData,encoding:.utf8)),range=try XCTUnwrap(greeting.range(of:"http://127\\.0\\.0\\.1:[0-9]+",options:.regularExpression))
  let engine=try Engine(address:String(greeting[range])),space=Workspace();space.engine=engine;space.rootFiles=try await engine.files().tree
  let buffer=try await space.loadBuffer(FileNode(name:"a.py",path:"a.py",dir:false));buffer.text=draft;buffer.selection=NSRange(location:2,length:0);space.layout.activate("a.py",in:"document-group")
  defer{buffer.language?.close()}
  let model=space.resourceRestoration,geometry=space.layout.nodes["root"]
  await model.prepareEdit(space,title:"Move retained source",edit:["documentChanges":[["kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"]]])
  XCTAssertNotNil(model.review,model.notice);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("a.py"),encoding:.utf8),saved);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("b.py").path))
  await model.publish(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(space.active,"b.py");XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,draft)
  XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("b.py"),encoding:.utf8),draft);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("a.py").path))
  await model.prepareInverse(space);XCTAssertNotNil(model.review,model.notice);await model.publish(space)
  XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(space.active,"a.py");XCTAssertTrue(space.currentBuffer === buffer);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(buffer.base,saved);XCTAssertTrue(buffer.dirty)
  XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("a.py"),encoding:.utf8),saved);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("b.py").path))
  await model.prepareInverse(space);XCTAssertNotNil(model.review,model.notice);await model.publish(space)
  XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(space.active,"b.py");XCTAssertEqual(buffer.base,draft);XCTAssertFalse(buffer.dirty)
  XCTAssertEqual(space.layout.nodes["root"],geometry);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0))
  await model.prepareInverse(space);XCTAssertNotNil(model.review,model.notice);buffer.text += "# newer local draft\n";await model.publish(space)
  XCTAssertFalse(model.hasPending);XCTAssertNil(model.review);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent("b.py"),encoding:.utf8),draft);XCTAssertTrue(buffer.text.hasSuffix("# newer local draft\n"))
  #endif
 }

 func testMalformedRecoveryAddressRetainsConfirmedReceiptWithoutAdoptionOrRetry()async throws {
  let (space,model,reading,writes)=try publicationFixture(),original=model.transport!
  model.transport={method,route,body in
   let result=try await original(method,route,body)
   if method=="POST",route=="/api/changes/restore-resources",case .object(var value)=result,case .object(var outcome)?=value["outcome"] {outcome["recovery"] = .string("unexpected-record-address");value["outcome"] = .object(outcome);return .object(value)}
   return result
  }
  let buffer=space.currentBuffer!
  await model.prepare(space,record:reading);await model.publish(space)
  XCTAssertTrue(model.hasPending);XCTAssertNotNil(model.confirmedReceipt);XCTAssertEqual(writes(),1);XCTAssertEqual(space.active,"b.py");XCTAssertTrue(space.currentBuffer === buffer);XCTAssertTrue(model.notice.contains("Saved publication is confirmed"))
  await model.publish(space);XCTAssertEqual(writes(),1)
  await model.status(space);XCTAssertFalse(model.hasPending,model.notice);XCTAssertEqual(writes(),1);XCTAssertEqual(space.active,"a.py");buffer.language?.close()
 }
 func testResourceActionWithProviderCommandRefusesWholeAction()async throws {
  let (space,model,writes)=try resourceSaveFixture();let buffer=space.currentBuffer!;buffer.language=NativeLanguage(engine:space.engine!,buffer:buffer)
  let edit:[String:Any]=["documentChanges":[["kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///a.py"]]]
  space.changes.open("actions",workspace:space)
  await space.changes.prepare(space,proposal:{_,_ in [["title":"Move and command","kind":"refactor.move"]]},resolution:{_ in ["title":"Move and command","edit":edit,"command":["command":"unsupported"]]})
  await space.changes.choose(0,workspace:space)
  XCTAssertNil(model.review);XCTAssertNil(space.changes.plan);XCTAssertEqual(writes(),0);XCTAssertEqual(space.active,"b.py");XCTAssertEqual(buffer.text,"dirty");XCTAssertTrue(space.changes.notice.contains("no edit portion"));buffer.language?.close()
 }

 func testDetachedSaveAsRetainsFailureAndRetiresOnlyCopiedDirtyHandle()async throws {
  let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
  let space=Workspace();space.engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config))
  let source=DocumentBuffer(file:FileNode(name:"reused.py",path:"reused.py",dir:false),text:FileText(path:"reused.py",content:"saved",hash:"old"));source.text="dirty retained text";source.resourceDeleted=true
  let occupant=DocumentBuffer(file:source.file,text:FileText(path:"reused.py",content:"independent source",hash:"independent"))
  space.buffers["reused.py"]=occupant;space.retainedResourceEditors["deleted"]=source
  let retainedTab=space.retainedResourceTab(source);space.layout.activate("reused.py",in:"document-group");space.layout.activate(retainedTab,in:"document-group")
  let submitted=source.text
  space.sourceRootAddress="/fixture";space.sourceWorkspaceID="runtime-workspace"
  var refuse=true
  ResponseProtocol.beforeResponse={request in
   ResponseProtocol.status=refuse ? 409:200
   let value:[String:Any]
   if request.url?.path=="/api/files" {ResponseProtocol.status=200;value=["root":"fixture","root_path":"/fixture","workspace_id":"runtime-workspace","tree":[]]}
   else if request.url?.path != "/api/source-history/create" {ResponseProtocol.status=200;value=[:]}
   else if refuse{value=["error":"Destination unavailable"]}
   else{value=["version":1,"history_protocol":1,"workspace":"runtime-workspace","namespace":"/fixture","path":"copy.py","source":submitted,"source_hash":nativeSourceHash(submitted),"hash":"saved","request_id":source.creationAttempt?.id ?? "missing-attempt","save_guard":"confirmed-copy","history":["available":true,"outcome":"published"]]}
   ResponseProtocol.response=try! JSONSerialization.data(withJSONObject:value)
  }
  defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];space.currentBuffer?.language?.close()}
  space.fileCommand="Save As";space.filePath="copy.py";await space.performFileCommand()
  XCTAssertTrue(space.retainedResourceEditors["deleted"] === source);XCTAssertTrue(source.dirty);XCTAssertEqual(space.active,retainedTab);XCTAssertTrue(space.buffers["reused.py"] === occupant);XCTAssertNil(space.buffers["copy.py"])
  refuse=false;space.fileCommand="Save As";space.filePath="copy.py";await space.performFileCommand()
  XCTAssertTrue(space.retainedResourceEditors.isEmpty,space.status);XCTAssertFalse(space.layout.groups().contains{$0.tabs.contains(retainedTab)});XCTAssertTrue(space.buffers["reused.py"] === occupant)
  XCTAssertEqual(space.currentBuffer?.file.path,"copy.py");XCTAssertEqual(space.currentBuffer?.text,"dirty retained text");XCTAssertFalse(space.currentBuffer?.dirty ?? true);XCTAssertFalse(space.hasUnsavedChanges)
 }

}
