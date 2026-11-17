import XCTest
@testable import HickorySimple
@MainActor final class ResourceAcquisitionTests:XCTestCase {
 func buffer(_ path:String,_ text:String,_ guardToken:String)->DocumentBuffer{DocumentBuffer(file:FileNode(name:path,path:path,dir:false),text:FileText(path:path,content:text,hash:"old",save_guard:guardToken))}
 func inputs(_ capture:ResourceWireCapture)->ResourceWireInputs {
  ResourceWireInputs(version:1,context:capture.context,workspace:"owned-workspace",ownership:ResourceWireOwnership(mode:"captured",entry:nil,protected_entries:[],saved_sources:[:],draft_sources:[:]),sources:["target.py":ResourceWireSource(kind:"file",identity:"target-identity",text:"saved target",writable:true,removable:true)],saved_sources:["target.py":ResourceSaved(kind:"file",text:"saved target",sha256:nativeSourceHash("saved target"),readonly:false,executable:false,mode:420)],refusals:[:],source_files_modified:false,saved_guards:["target.py":"original-target"])
 }
 func testClosedAcquisitionPreservesOriginalSelectionAndUsesOneGuardedRecapture()async throws {
  let w=Workspace(),owner=buffer("owner.md","construction","original-owner");w.buffers[owner.file.path]=owner;w.layout.activateSource(owner.file.path,in:"document-group");owner.selection=NSRange(location:2,length:1);let revision=owner.sourceEditIdentity,selection=owner.selection,layout=try editorRecoveryData(w.layout);var captures:[ResourceWireCapture]=[]
  let result=try await acquireResourceInputs(w,paths:["target.py"],edit:.object(["changes":.object([:])]),current:{true},load:{file in let b=self.buffer(file.path,"saved target","original-target");w.buffers[file.path]=b;return b},readInputs:{capture in captures.append(capture);return self.inputs(capture)})
  XCTAssertTrue(result.acquired);XCTAssertEqual(captures.count,2);XCTAssertTrue(result.current());XCTAssertEqual(owner.sourceEditIdentity,revision);XCTAssertEqual(owner.selection,selection);XCTAssertEqual(try editorRecoveryData(w.layout),layout);XCTAssertTrue(w.currentBuffer === owner);XCTAssertEqual(result.capture.drafts["target.py"]?.save_guard,"original-target")
 }
 func testOriginalEditUndoReplacementGuardOrNewOwnerCannotBeBlessedByALoad()async throws {
  for mode in ["undo","replacement","guard","unknown"] {
   let w=Workspace(),owner=buffer("owner.md","construction","original-owner");w.buffers[owner.file.path]=owner
   do{_ = try await acquireResourceInputs(w,paths:["target.py"],edit:.object([:]),current:{true},load:{file in
    switch mode{case "undo":owner.text="changed";owner.text="construction";case "replacement":w.buffers[owner.file.path]=self.buffer(owner.file.path,owner.text,"original-owner");case "guard":owner.savedGuard="foreign";default:w.buffers["other.md"]=self.buffer("other.md","unknown construction","other")}
    let b=self.buffer(file.path,"saved target","original-target");w.buffers[file.path]=b;return b
   },readInputs:{self.inputs($0)});XCTFail("Renewed stale original owners: "+mode)}catch{XCTAssertTrue(error.localizedDescription.contains("changed"),mode+": "+error.localizedDescription)}
  }
 }
 func testClosedTargetNeedsItsOriginalGuardBytesAndPristineIntent()async throws {
  for mode in ["guard","dirty","undo","address"] {
   let w=Workspace()
   do{_ = try await acquireResourceInputs(w,paths:["target.py"],edit:.object([:]),current:{true},load:{file in
    let b=self.buffer(mode=="address" ? "foreign.py":file.path,"saved target",mode=="guard" ? "new-physical-object":"original-target")
    if mode=="dirty"{b.text="new draft"};if mode=="undo"{b.text="new draft";b.text="saved target"};w.buffers[file.path]=b;return b
   },readInputs:{self.inputs($0)});XCTFail("Admitted changed closed target: "+mode)}catch{XCTAssertTrue(error.localizedDescription.contains("changed"),mode+": "+error.localizedDescription)}
  }
 }
 func testLateReplyCannotRenewTheOriginalOwnerAndLaterEditsRetireTheResult()async throws {
  let w=Workspace(),owner=buffer("owner.md","construction","original-owner");w.buffers[owner.file.path]=owner;var first=true
  do{_ = try await acquireResourceInputs(w,paths:["target.py"],edit:.object([:]),current:{true},readInputs:{capture in if first{first=false;owner.text="edited during inputs"};return self.inputs(capture)});XCTFail("Admitted an obsolete first reply")}catch{}
  owner.text="construction"
  let result=try await acquireResourceInputs(w,paths:["target.py"],edit:.object([:]),current:{true},load:{file in let b=self.buffer(file.path,"saved target","original-target");w.buffers[file.path]=b;return b},readInputs:{self.inputs($0)})
  XCTAssertTrue(result.current());owner.text="later";owner.text="construction";XCTAssertFalse(result.current())
 }
}
