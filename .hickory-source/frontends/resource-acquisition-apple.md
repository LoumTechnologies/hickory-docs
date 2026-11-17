<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Acquire closed Apple resource targets without renewing stale source authority

For maintainers preparing provider resource changes or a complete Git conflict group.

## Retain the original owners across every acquisition await

Loading a closed target must not turn a later editor state into the original intention. Capture all existing buffer bindings, retained owners, source revisions and physical baseline guards before asking the engine for inputs. Permit only the one target currently being acquired to appear. A new unrelated owner, an independent handle, edit/Undo or changed guard retires preparation without moving or replacing any editor.

Provider draft preparation reuses this lease instead of recapturing the namespace after loading. Seal a pristine new target before awaiting language synchronization, so edits and equal-byte Undo during that await invalidate the original review. Retaining one explicit target extends the lease; it never replaces the original owner observations.

<ui:copy id="resource-acquisition-apple-lease">
import Foundation
struct ResourceAcquisition {var capture:ResourceWireCapture;var inputs:ResourceWireInputs;var acquired:Bool;var current:()->Bool}
@MainActor private struct ResourceAcquisitionOwner {
 let buffer:DocumentBuffer;let identity:UUID;let source:ExactSourceText;let base:ExactSourceText;let guardToken:Data?;let path:Data;let doc:Data?;let generated:Data?;let readOnly:Bool;let deleted:Bool;let unnamed:Bool
 init(_ b:DocumentBuffer){buffer=b;identity=b.sourceEditIdentity;source=b.sourceRevision;base=ExactSourceText(b.base);guardToken=b.savedGuard.map{Data($0.utf8)};path=Data(b.file.path.utf8);doc=b.file.doc_id.map{Data($0.utf8)};generated=b.file.generated_by.map{Data($0.utf8)};readOnly=b.readOnly;deleted=b.resourceDeleted;unnamed=b.isUntitledDraft}
 func current()->Bool{let b=buffer;return b.sourceEditIdentity==identity&&b.sourceRevision==source&&ExactSourceText(b.base)==base&&b.savedGuard.map{Data($0.utf8)}==guardToken&&Data(b.file.path.utf8)==path&&b.file.doc_id.map{Data($0.utf8)}==doc&&b.file.generated_by.map{Data($0.utf8)}==generated&&b.readOnly==readOnly&&b.resourceDeleted==deleted&&b.isUntitledDraft==unnamed && !b.saving}
}
@MainActor final class ResourceAcquisitionLease {
 private weak var workspace:Workspace?;private let engine:Engine?;private let epoch:UUID;private let runtime:String?;private let root:String?;private var addresses:NativeSourceMap<DocumentBuffer>;private let retained:[String:DocumentBuffer];private var owners:[ResourceAcquisitionOwner];private let external:()->Bool
 init(_ w:Workspace,current:@escaping()->Bool){workspace=w;engine=w.engine;epoch=w.sourceNavigationEpoch;runtime=w.sourceWorkspaceID;root=w.sourceRootAddress;addresses=w.buffers;retained=w.retainedResourceEditors;owners=(Array(w.buffers.values)+Array(w.retainedResourceEditors.values)).map(ResourceAcquisitionOwner.init);external=current}
 func current(allowing path:String?=nil)->Bool {
  guard let w=workspace,external(),w.engine?.session === engine?.session,w.sourceNavigationEpoch==epoch,exactUTF8(w.sourceWorkspaceID,runtime),exactUTF8(w.sourceRootAddress,root),addresses.allSatisfy({key,b in w.buffers[key] === b}),w.buffers.allSatisfy({key,b in addresses[key] === b || path.map{exactUTF8(key,$0)}==true}),w.retainedResourceEditors.count==retained.count,retained.allSatisfy({key,b in w.retainedResourceEditors[key] === b}),owners.allSatisfy({$0.current()}) else{return false};return true
 }
 func retain(_ path:String,_ b:DocumentBuffer){addresses[path]=b;owners.append(ResourceAcquisitionOwner(b))}
}
</ui:copy>

## Compare newly loaded targets with the first physical observation

The first resource-input reading establishes the closed source's saved bytes and guard. A loaded editor must still have that exact baseline, no source intent since construction and no dirty text. Reacquiring inputs uses the original owner lease and newly admitted targets; it never silently recaptures changed unrelated drafts. The caller may requery a provider under this same lease before using its final edit.

<ui:copy id="resource-acquisition-apple-inputs">
@MainActor func acquireResourceInputs(_ w:Workspace,paths:[String],edit:ResourceJSON,current:@escaping()->Bool,load:((FileNode)async throws->DocumentBuffer)?=nil,readInputs:(ResourceWireCapture)async throws->ResourceWireInputs)async throws->ResourceAcquisition {
 guard !paths.isEmpty,paths.count<=1000,Set(paths).count==paths.count else{throw EngineError(message:"Choose bounded resource paths distinguishable by the native wire format.")}
 let lease=ResourceAcquisitionLease(w,current:current),context=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased()
 func capture()throws->ResourceWireCapture {
  guard lease.current() else{throw EngineError(message:"Original source owners changed. Prepare the complete intention again.")};var drafts:[String:ResourceWireDraft]=[:]
  for path in paths {if let b=w.buffers[path]{drafts[path]=ResourceWireDraft(text:b.text,base_sha256:nativeSourceHash(b.base),version:nil,writable:!b.readOnly && !b.saving,save_guard:b.savedGuard)}}
  let ownership=try resourceOwnershipSnapshot(w);return ResourceWireCapture(context:context,edit:edit,drafts:drafts,project_entry:nil,ownership_sources:ownership.named,unaddressed_sources:ownership.unaddressed,saved_guards:ownership.guards)
 }
 func admit(_ inputs:ResourceWireInputs)throws {guard lease.current(),inputs.version==1,inputs.context==context,!inputs.workspace.isEmpty,inputs.saved_guards != nil,inputs.refusals.isEmpty,!inputs.source_files_modified,Set(inputs.sources.keys)==Set(paths) else{throw EngineError(message:"Complete resource inputs changed or refused a source. Preserve drafts and prepare again.")}}
 var captured=try capture(),inputs=try await readInputs(captured);try admit(inputs);let first=inputs;var acquired=false
 func node(_ path:String,_ files:[FileNode])->FileNode?{for item in files{if exactUTF8(item.path,path){return item};if let child=node(path,item.children ?? []){return child}};return nil}
 for path in paths where first.sources[path]?.kind=="file" && w.buffers[path]==nil {
  guard lease.current(),let expected=first.sources[path]?.text,let saved=first.saved_guards?[path],!saved.isEmpty else{throw EngineError(message:"Closed resource target has no original physical observation.")}
  let file=node(path,w.rootFiles) ?? FileNode(name:(path as NSString).lastPathComponent,path:path,dir:false)
  let b:DocumentBuffer;if let load{b=try await load(file)}else{b=try await w.loadBuffer(file)}
  guard lease.current(allowing:path),w.buffers[path] === b,exactUTF8(b.file.path,path),exactUTF8(b.text,expected),exactUTF8(b.base,expected),exactUTF8(b.savedGuard,saved),b.agentSourceRevision==0,!b.saving else{throw EngineError(message:"Source ownership or the originally observed closed target changed during acquisition. No preview was renewed.")}
  lease.retain(path,b);acquired=true
 }
 if acquired {captured=try capture();inputs=try await readInputs(captured);try admit(inputs);guard first.workspace==inputs.workspace else{throw EngineError(message:"Resource workspace changed after acquisition.")}}
 return ResourceAcquisition(capture:captured,inputs:inputs,acquired:acquired,current:{lease.current()})
}
</ui:copy>

## Exercise edits, independent handles and physical-observation changes

The fixture substitutes only the ordinary input/read capabilities. It asserts refusal before a preview can acquire renewed authority, including text restored by Undo and equal bytes under a different physical guard. The real resource-publication witness remains a separate integration check.

<ui:copy id="resource-acquisition-apple-tests">
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ResourceAcquisition.swift"><ui:paste select="#resource-acquisition-apple-lease"/><ui:paste select="#resource-acquisition-apple-inputs"/></ui:file>
<ui:file path="apps/apple-simple/Tests/ResourceAcquisitionTests.swift"><ui:paste select="#resource-acquisition-apple-tests"/></ui:file>
</ui:doc>
