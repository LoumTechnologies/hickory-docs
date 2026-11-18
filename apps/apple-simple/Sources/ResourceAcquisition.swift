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
struct ResourceRetainedCapture:Encodable {
 let drafts:[String:ResourceWireDraft]
 let ownership_sources:[String:String]
 let unaddressed_sources:[String:String]
 let saved_guards:[String:String]
}

@MainActor func resourceRetainedCapture(_ workspace:Workspace,paths:[String],interrupted:InterruptedWorkspaceEntry?=nil)throws->ResourceRetainedCapture {
 var drafts:[String:ResourceWireDraft]=[:]
 for path in paths {
  guard let buffer=workspace.buffers[path] else{continue}
  drafts[path]=ResourceWireDraft(text:buffer.text,base_sha256:nativeSourceHash(buffer.base),version:nil,
   writable:!buffer.readOnly && !buffer.saving,save_guard:buffer.savedGuard)
 }
 let ownership=try resourceOwnershipSnapshot(workspace)
 var guards=ownership.guards
 if let interrupted,interrupted.missingOriginal {guards.removeValue(forKey:interrupted.source)}
 return ResourceRetainedCapture(drafts:drafts,ownership_sources:ownership.named,
  unaddressed_sources:ownership.unaddressed,saved_guards:guards)
}

@MainActor func acquireResourceInputs(_ w:Workspace,paths:[String],edit:ResourceJSON,context suppliedContext:String?=nil,interrupted:InterruptedWorkspaceEntry?=nil,current:@escaping()->Bool,load:((FileNode)async throws->DocumentBuffer)?=nil,readInputs:(ResourceWireCapture)async throws->ResourceWireInputs)async throws->ResourceAcquisition {
 guard !paths.isEmpty,paths.count<=1000,Set(paths).count==paths.count else{throw EngineError(message:"Choose bounded resource paths distinguishable by the native wire format.")}
 let lease=ResourceAcquisitionLease(w,current:current),context=suppliedContext ?? UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased()
 func capture()throws->ResourceWireCapture {
  guard lease.current() else{throw EngineError(message:"Original source owners changed. Prepare the complete intention again.")}
  let retained=try resourceRetainedCapture(w,paths:paths,interrupted:interrupted)
  return ResourceWireCapture(context:context,edit:edit,drafts:retained.drafts,project_entry:nil,
   ownership_sources:retained.ownership_sources,unaddressed_sources:retained.unaddressed_sources,saved_guards:retained.saved_guards)
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
