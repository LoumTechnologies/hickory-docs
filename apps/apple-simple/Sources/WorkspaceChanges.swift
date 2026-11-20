import SwiftUI
import CoreFoundation
func scopedProviderActions(_ actions:[[String:Any]],_ scope:String?)->[[String:Any]] {let prefix:String?=scope=="fixAll" ? "source.fixAll":scope=="sourceActions" ? "source":nil;guard let prefix else{return actions};return actions.filter{guard let kind=$0["kind"] as? String else{return false};return kind==prefix || kind.hasPrefix(prefix+".")}}
func fileWideFixActions(_ actions:[[String:Any]])->[[String:Any]] {scopedProviderActions(actions,"fixAll")}
import Observation
struct ResourceHistoryIntent {var direction:String;var record:String;var token:UUID}
private struct WorkspaceDraftHistory {let plan:ChangePlan;let read:()throws->ChangePlan}
private enum WorkspaceHistoryStep {case drafts(WorkspaceDraftHistory);case resource(String)}
@MainActor private func captureDraftHistory(_ plan:ChangePlan,workspace:Workspace)->WorkspaceDraftHistory {
 let server=workspace.engine,root=workspace.sourceRootAddress,runtime=workspace.sourceWorkspaceID,targets=plan.patches.map{($0,workspace.buffers[$0.path])}
 return WorkspaceDraftHistory(plan:plan,read:{[weak workspace] in
  func refuse()->EngineError{EngineError(message:"Workspace history no longer owns the original source editors. Drafts were retained.")}
  guard let workspace,workspace.engine?.session === server?.session,exactUTF8(workspace.sourceRootAddress,root),exactUTF8(workspace.sourceWorkspaceID,runtime),!targets.isEmpty else{throw refuse()}
  let patches=try targets.map{patch,buffer->SourcePatch in
   guard let buffer,!buffer.readOnly,!buffer.saving else{throw refuse()};let addresses=workspace.buffers.filter{$0.value === buffer}
   guard addresses.count==1,let address=addresses.first,exactUTF8(address.key,buffer.file.path) else{throw refuse()};var current=patch;current.path=address.key;return current
  }
  guard Set(patches.map{NativeSourceKey($0.path)}).count==patches.count else{throw refuse()};let current=ChangePlan(title:plan.title,patches:patches)
  if let failure=validateChangePlan(current,sources:workspace.sourceSnapshots()){throw EngineError(message:failure)};return current
 })
}
@MainActor @Observable final class WorkspaceChanges {
    private(set) var entrySelection:FrozenWorkspaceEntrySelection?
    private var entryScope=UUID()
    var mode: String?
    var name = "" { didSet { if !exactUTF8(name,oldValue) { revision = UUID(); plan = nil; choices = []; notice = "" } } }
    var notice = ""
    var plan: ChangePlan?
    var choices: [[String:Any]] = []
    var file = 0; var page = 0; var actionPage = 0
    var referencePaths: [String] = []
    private var revision = UUID()
    var actionSnapshot:UUID {revision}
    private var choiceRevision=UUID()
    private var resolver:(([String:Any]) async throws->[String:Any])?
    private var acquisition:((String) async throws->SourceSnapshot)?
    private var actionQuery:(() async throws->[[String:Any]])?
    private var requiresProviderProofs=true
    private var captured: (sources:NativeSourceMap<SourceSnapshot>, current:()->Bool)?
    private var undo: [WorkspaceHistoryStep] = []
    private var redo: [WorkspaceHistoryStep] = []
    private(set) var historySnapshot=UUID()
    func resourceHistoryRecord(_ direction:String)->String? {
        if case .resource(let record)? = (direction=="undo" ? undo.last:redo.last) {return record};return nil
    }
    func recordDraftApplication(_ plan:ChangePlan,workspace:Workspace) {
        guard plan.patches.contains(where:{!exactUTF8($0.before,$0.after)}) else{return}
        undo=Array(undo.suffix(31))+[.drafts(captureDraftHistory(invertChangePlan(plan),workspace:workspace))];redo=[];historySnapshot=UUID()
    }
    func recordResourcePublication(_ record:String,intent:ResourceHistoryIntent?) {
        if let intent,intent.token==historySnapshot,resourceHistoryRecord(intent.direction)==intent.record {
            if intent.direction=="undo" {undo.removeLast();redo=Array(redo.suffix(31))+[.resource(record)]}
            else {redo.removeLast();undo=Array(undo.suffix(31))+[.resource(record)]}
        }else {undo=Array(undo.suffix(31))+[.resource(record)];redo=[]}
        historySnapshot=UUID()
    }
    func reset() { revision = UUID(); retireEntryMove(); mode = nil; captured = nil; plan = nil; choices = []; undo = []; redo = []; historySnapshot=UUID(); notice = "" }
    func close(_ workspace: Workspace) { revision = UUID();retireEntryMove();mode = nil;workspace.resourceRestoration.cancel();workspace.currentBuffer?.focusSource?() }
    func retireEntryMove() {entryScope=UUID();entrySelection=nil}
    func open(_ kind: String, workspace: Workspace) {
        revision = UUID();retireEntryMove();mode = kind; name = ""; plan = nil; choices = []; notice = ""; actionPage = 0
        if kind == "entryMove" {
            workspace.resourceRestoration.cancel()
            let scope=entryScope
            notice="Reading the current main construction…"
            Task { await openEntryMove(workspace,scope:scope) }
            return
        }
        guard kind == "undo" || kind == "redo" else { return }
        let target = kind == "undo" ? undo.last : redo.last
        guard let target else { notice = "No workspace change to \(kind)."; return }
        switch target {
        case .resource(let record):
            let intent=ResourceHistoryIntent(direction:kind,record:record,token:historySnapshot)
            Task {await workspace.resourceRestoration.prepareHistory(workspace,intent:intent)}
            notice="Prepare and review saved-source restoration before publishing this workspace \(kind)."
        case .drafts(let step):
            do {
                let target=try step.read()
                let refresh=try workspace.applyWorkspacePlan(target)
                if kind=="undo" {undo.removeLast();redo.append(.drafts(captureDraftHistory(invertChangePlan(target),workspace:workspace)))}else{redo.removeLast();undo.append(.drafts(captureDraftHistory(invertChangePlan(target),workspace:workspace)))}
                historySnapshot=UUID()
                notice="Workspace \(kind) applied to \(target.patches.count) drafts. Not saved or verified."+refresh
            }catch{notice=error.localizedDescription}
        }
    }
    private func openEntryMove(_ workspace:Workspace,scope:UUID)async {
        let current={ [weak self] in self?.entryScope==scope }
        do {
            let selection=try await workspace.resourceRestoration.readEntry(workspace,current:current)
            guard selection.entryPath != nil else{throw EntrySelectionFailure("This workspace has no configured main construction.")}
            guard selection.proof.movable else{throw EntrySelectionFailure("The current main construction cannot be moved. Check its saved source.")}
            entrySelection=selection
            notice=""
        }catch{if current(){notice=error.localizedDescription}}
    }
    func prepare(_ workspace: Workspace,proposal:((String,String) async throws->Any)?=nil,resolution:(([String:Any]) async throws->[String:Any])?=nil,acquisition:((String) async throws->SourceSnapshot)?=nil,enforceProviderProofs:Bool=false) async {
        if mode == "entryMove" {
            guard let selection=entrySelection,!name.isEmpty,!exactUTF8(name,selection.entryPath) else{return}
            revision=UUID();let id=revision
            await workspace.resourceRestoration.prepareEntryMove(workspace,selection:selection,destination:name,current:{ [weak self] in self?.revision==id })
            return
        }
        if mode == "cleanup" { await prepareCleanup(workspace,proposal:proposal); return }
        guard let origin = workspace.currentBuffer, let language = origin.language, !origin.readOnly else { notice = "Choose writable source with a connected language provider."; return }
        requiresProviderProofs=proposal==nil || enforceProviderProofs
        let proofRequired=requiresProviderProofs,server=workspace.engine
        revision = UUID(); let request = revision, source = origin.text, selection = origin.selection, identity = language.proposalIdentity
        let sourceIdentity=origin.sourceEditIdentity
        let sources = workspace.sourceSnapshots()
        captured = (sources,{ [weak self, weak origin, weak language,weak workspace] in
            guard let self, let origin, let language,let workspace else { return false }
            return self.revision == request && workspace.engine?.session === server?.session && exactUTF8(workspace.engine?.origin.absoluteString,server?.origin.absoluteString) && workspace.buffers[origin.file.path] === origin && origin.sourceEditIdentity==sourceIdentity && exactUTF8(origin.text,source) && origin.selection == selection && origin.language === language && language.proposalIdentity == identity
        })
        resolver=resolution ?? {[weak language] action in guard let language else{throw EngineError(message:"Provider changed. Prepare actions again.")};return try await language.resolveProposal(action)}
        let lease=ResourceAcquisitionLease(workspace,current:captured!.current)
        captured?.current={lease.current()}
        let valid=captured!.current
        let propose: (String,String) async throws->Any = proposal ?? {kind,name in try await language.propose(kind,name:name)}
                var listed:NativeSourceMap<FileNode>=[:]
                func collect(_ nodes:[FileNode]){for file in nodes{if !file.dir{listed[file.path]=file};collect(file.children ?? [])}};collect(workspace.rootFiles)
        let acquireTarget:(String) async throws->SourceSnapshot=acquisition ?? {[weak workspace] path in
                    guard let workspace else{throw EngineError(message:"Workspace closed. Prepare again.")}
                    guard valid() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}
                    guard let file=workspace.buffers[path]?.file ?? listed[path],!file.dir else{throw EngineError(message:"Referenced source is no longer listed: \(path). Refresh files and prepare again.")}
                    let wasRetained=workspace.buffers[path] != nil
                    let target=try await workspace.loadBuffer(file),text=target.text
                    guard wasRetained || target.agentSourceRevision==0 && exactUTF8(target.text,target.base) else{throw EngineError(message:"The newly acquired source was edited before original ownership could be sealed.")}
                    guard lease.current(allowing:path) else{throw EngineError(message:"Original source owners changed during acquisition. Prepare again.")};lease.retain(path,target)
                    guard !target.saving,let service=target.language else{throw EngineError(message:"Referenced source cannot synchronize: \(path). Retry language services.")}
                    _ = try await service.readOutline(source:text)
                    guard valid(),exactUTF8(target.text,text) else{throw EngineError(message:"Source changed while reading \(path). Prepare again.")}
                    return SourceSnapshot(text:text,writable:!target.readOnly && !target.saving)
                }
        let load:(String) async throws->SourceSnapshot={path in
            guard valid() else{throw EngineError(message:"Original source owners changed before acquisition. Prepare again.")}
            let wasRetained=workspace.buffers[path] != nil,source=try await acquireTarget(path)
            guard lease.current(allowing:path),let target=workspace.buffers[path],wasRetained || target.agentSourceRevision==0 && exactUTF8(target.text,target.base),exactUTF8(target.text,source.text),!target.saving else{throw EngineError(message:"Original source owners or the acquired target changed. Prepare again.")}
            lease.retain(path,target);return source
        }
        let actionKind=mode=="fixAll" ? "fixAll":mode=="sourceActions" ? "sourceActions":"actions"
        self.acquisition=load;actionQuery={let actions=try await propose(actionKind,"") as? [[String:Any]] ?? [];return scopedProviderActions(actions,actionKind)}
        plan = nil; choices = []; referencePaths = []; notice = "Preparing complete source changes…"
        let kind = mode ?? "actions", nextName = name
        do {
            if proposal==nil {_ = try await language.readOutline(source:source);guard valid() else{throw EngineError(message:"Original source preparation changed. Prepare again.")}}
            if kind == "rename" {
                let acquired=try await acquireReferenceSources(sources,query:{
                    let locations=semanticLocations(try await propose("references",""))
                    return try locations.map {location in guard let path=location.path else{throw EngineError(message:"Reference target has no writable workspace source identity. No edits were applied.")};return path}
                },acquire:load,current:valid)
                guard valid() else{return};captured?.sources=acquired.sources;referencePaths=acquired.references
            }
            let result = try await propose(kind,nextName)
            guard valid() else { if revision == request { notice = "Source, selection or provider changed. Prepare again." }; return }
            if kind == "actions" || kind == "fixAll" || kind == "sourceActions" { let actions=result as? [[String:Any]] ?? [];choices=scopedProviderActions(actions,kind);notice=choices.isEmpty ? (kind=="fixAll" ? "The current provider offers no file-wide diagnostic fixes. No edits were applied.":kind=="sourceActions" ? "No source cleanup or generation actions are available here. No edits were applied.":"No provider actions here. Support depends on the installed language server."):"Choose one complete provider action to preview. Nothing is saved or verified." }
            else if proofRequired {
                var first:Any?=result
                let acquired=try await acquireProviderAction(workspace,sources:captured?.sources ?? sources,query:{
                    let value:Any;if let initial=first{value=initial;first=nil}else{value=try await propose(kind,nextName)}
                    guard let action=value as? [String:Any],let modified=action["source_files_modified"] as? NSNumber,CFGetTypeID(modified)==CFBooleanGetTypeID(),!modified.boolValue else{throw EngineError(message:"Provider did not confirm one complete no-effect source proposal.")};return action
                },load:load,current:valid,proofs:true)
                captured?.sources=acquired.sources;let action=acquired.value;let edit=action["edit"]!
                if isResourceWorkspaceEdit(edit){await workspace.resourceRestoration.prepareEdit(workspace,title:action["title"] as? String ?? "Provider resource change",edit:edit,current:valid)}
                else{try install(action["title"] as? String ?? "Provider source change",edit:edit,workspace:workspace)}
            }
            else if isResourceWorkspaceEdit(result) {
                await workspace.resourceRestoration.prepareEdit(workspace,title:kind == "rename" ? "Rename to \(nextName)":"Provider resource change",edit:result,current:valid,requery:{try await propose(kind,nextName)})
                if valid(){plan=nil;choices=[];notice=workspace.resourceRestoration.notice}
            }
            else { try install(kind == "rename" ? "Rename to \(nextName)" : "Format source", edit:result, workspace:workspace) }
        } catch { if revision == request { notice = error.localizedDescription } }
    }
    private func prepareCleanup(_ workspace:Workspace,proposal:((String,String) async throws->Any)?) async {
        guard let origin=workspace.currentBuffer,!origin.readOnly,!origin.saving else {notice="Choose writable source before preparing cleanup.";return}
        revision=UUID();let request=revision,source=origin.text,selection=origin.selection,path=origin.file.path
        captured=(workspace.sourceSnapshots(),{[weak self,weak workspace,weak origin] in
            guard let self,let workspace,let origin else{return false}
            return self.revision==request && workspace.buffers[path] === origin && exactUTF8(origin.text,source) && origin.selection==selection
        })
        let lease=ResourceAcquisitionLease(workspace,current:captured!.current);captured?.current={lease.current()}
        let valid=captured!.current
        plan=nil;choices=[];referencePaths=[];notice="Preparing trailing-whitespace cleanup…"
        do {
            let edit:Any
            if let proposal {edit=try await proposal("cleanup","")}
            else {
                guard let engine=workspace.engine else{throw EngineError(message:"Connect to a workspace server first.")}
                let address=languageURI(path)
                let response:CleanupResponse=try await engine.request("POST","/api/changes/cleanup",body:["uri":address,"source":source,"profile":"trim_trailing_horizontal"])
                edit=response.workspaceEdit
            }
            guard valid() else{if revision==request{notice="Source or selection changed. Prepare cleanup again."};return}
            try install("Trim trailing whitespace",edit:edit,workspace:workspace)
        }catch{if revision==request{notice=error.localizedDescription}}
    }
    func offer(_ title:String,edit:Any,expected:[String:String],workspace:Workspace)throws {
        let sources=workspace.sourceSnapshots(),next=try planWorkspaceEdit(title,raw:edit,sources:sources)
        if let refusal=validateChangePlan(next,sources:sources){throw EngineError(message:refusal)}
        let targets=try next.patches.map{patch -> (DocumentBuffer,String,NSRange) in
            guard let before=expected.first(where:{exactUTF8($0.key,patch.path)})?.value,exactUTF8(before,patch.before),let buffer=workspace.buffers.values.first(where:{exactUTF8($0.file.path,patch.path)}),exactUTF8(buffer.text,before) else{throw EngineError(message:"Reviewed source changed or a target was not captured. Prepare the whole proposal again.")}
            return (buffer,before,buffer.selection)
        }
        revision=UUID();let request=revision,lease=ResourceAcquisitionLease(workspace,current:{true})
        captured=(sources,{[weak self,weak workspace] in
            guard let self,let workspace,self.revision==request,lease.current() else{return false}
            return targets.allSatisfy{buffer,text,selection in workspace.buffers.values.contains(where:{$0 === buffer})&&exactUTF8(buffer.text,text)&&buffer.selection==selection}
        })
        mode="proposal";plan=next;choices=[];referencePaths=[];notice="";file=0;page=0;actionPage=0
    }
    private func providerResourcePreimages(_ edit:Any,workspace:Workspace,current:@escaping()->Bool)async throws->[String] {
        guard current(),let engine=workspace.engine else{throw EngineError(message:"Original source workspace changed. Prepare again.")}
        let targets:ResourceWireTargets=try await engine.request("POST","/api/changes/resource-targets",body:["edit":edit])
        guard current(),targets.version==1,!targets.paths.isEmpty,targets.paths.count<=1000,Set(targets.paths).count==targets.paths.count else{throw EngineError(message:"Complete resource source identities changed. Prepare again.")}
        let raw=try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:edit))
        let observation=try await acquireResourceInputs(workspace,paths:targets.paths,edit:raw,current:current,readInputs:{capture in try await engine.request("POST","/api/changes/resource-inputs",body:resourceBody(capture))})
        guard current(),observation.current() else{throw EngineError(message:"Original resource preimages changed. Prepare again.")}
        return observation.inputs.sources.compactMap{path,source in source.kind=="file" ? path:nil}
    }
    private func acquireProviderAction(_ workspace:Workspace,sources:NativeSourceMap<SourceSnapshot>,query:@escaping()async throws->[String:Any],load:@escaping(String)async throws->SourceSnapshot,current:@escaping()->Bool,proofs:Bool)async throws->SourceProposalAcquisition<[String:Any]> {
        var writable=Set<NativeSourceKey>()
        let acquired=try await acquireSourceProposal(sources,query:{
            let action=try await query()
            guard current() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}
            if let disabled=action["disabled"] as? [String:Any]{throw EngineError(message:disabled["reason"] as? String ?? "Provider action disabled")}
            guard action["command"]==nil else{throw EngineError(message:"This action requires a provider command; no edit portion was applied.")}
            let proofPaths=proofs ? try providerSourceProofPaths(action):[]
            let pending=(action["data"] as? [String:Any])?["hickory_acquire"] != nil ? try providerActionPaths(action):[]
            let editPaths: [String]
            if let edit=action["edit"],!isResourceWorkspaceEdit(edit){editPaths=try workspaceEditTargets(edit).map(\.path)}else{editPaths=[]}
            writable=Set(editPaths.map(NativeSourceKey.init))
            let paths=proofPaths+pending+editPaths
            return (paths:proofs ? paths:paths.filter{sources[$0]==nil},value:action)
        },acquire:load,current:current,requireWritable:{writable.contains(NativeSourceKey($0))})
        guard current(),let edit=acquired.value["edit"] else{throw EngineError(message:"Provider source acquisition did not produce a complete edit. Prepare again.")}
        if proofs {
            let required = isResourceWorkspaceEdit(edit) ? try await providerResourcePreimages(edit,workspace:workspace,current:current):try workspaceEditTargets(edit).map(\.path)
            try validateProviderSourceProofs(acquired.value,sources:acquired.sources,requiredExisting:required)
            for path in required {guard acquired.sources[path]?.writable==true else{throw EngineError(message:"Source is generated, read-only or saving: \(path)")}}
        }
        return acquired
    }
    private func install(_ title: String, edit: Any, workspace: Workspace) throws {
        guard let captured, captured.current() else { throw EngineError(message:"Source, selection or provider changed. Prepare again.") }
        let next = try planWorkspaceEdit(title,raw:edit,sources:captured.sources,allowNoop:mode == "cleanup")
        if let refusal = validateChangePlan(next,sources:workspace.sourceSnapshots()) { throw EngineError(message:refusal) }
        try validateReferencedTargets(next,referencePaths:referencePaths)
        if next.patches.isEmpty {plan=nil;notice="No trailing ASCII spaces or tabs to remove.";return}
        plan = next; choices = []; file = 0; page = 0; notice = ""
    }
    func choose(_ index:Int,workspace:Workspace,expectedSnapshot:UUID?=nil) async {
        guard expectedSnapshot==nil || expectedSnapshot==revision else{notice="Code action choices changed. Prepare actions again.";return}
        guard choices.indices.contains(index),let captured,let resolver,let acquisition,let actionQuery else{return};let action=choices[index],request=revision,choice=UUID();choiceRevision=choice
        do {
            guard captured.current() else{throw EngineError(message:"Source, selection or provider changed. Prepare actions again.")}
            if let disabled=action["disabled"] as? [String:Any]{throw EngineError(message:disabled["reason"] as? String ?? "Provider action disabled")}
            plan=nil;notice="Resolving complete provider action…"
            var first:[String:Any]?=try await resolver(action)
            if ["fixAll","sourceActions"].contains(mode ?? ""),let first,scopedProviderActions([first],mode).isEmpty{throw EngineError(message:"The provider changed the complete file-wide action kind. Prepare again.")}
            if !requiresProviderProofs,let resolved=first,let edit=resolved["edit"],isResourceWorkspaceEdit(edit) {
                guard self.revision==request,self.choiceRevision==choice,captured.current() else{throw EngineError(message:"Source, selection or provider changed. Prepare actions again.")}
                if ["fixAll","sourceActions"].contains(self.mode ?? ""),scopedProviderActions([resolved],self.mode).isEmpty{throw EngineError(message:"The provider changed the complete file-wide action kind. Prepare again.")}
                if let disabled=resolved["disabled"] as? [String:Any]{throw EngineError(message:disabled["reason"] as? String ?? "Provider action disabled")}
                guard resolved["command"]==nil else{throw EngineError(message:"This action requires a provider command; no edit portion was applied.")}
                await workspace.resourceRestoration.prepareEdit(workspace,title:resolved["title"] as? String ?? "Provider resource action",edit:edit,current:{self.revision==request && self.choiceRevision==choice && captured.current()},requery:{
                    let candidates=try await actionQuery().filter{exactUTF8($0["title"] as? String ?? "",action["title"] as? String ?? "") && exactUTF8($0["kind"] as? String ?? "",action["kind"] as? String ?? "")}
                    guard candidates.count==1 else{throw EngineError(message:"The selected resource action changed or is ambiguous. Prepare again.")}
                    let next=try await resolver(candidates[0])
                    guard next["command"]==nil,next["disabled"]==nil,let edit=next["edit"],isResourceWorkspaceEdit(edit) else{throw EngineError(message:"The complete provider resource action changed. No changes were applied.")}
                    return edit
                })
                if revision==request,choiceRevision==choice {plan=nil;choices=[];notice=workspace.resourceRestoration.notice}
                return
            }
            let acquired=try await acquireProviderAction(workspace,sources:captured.sources,query:{
                let resolved:[String:Any]
                if let initial=first{resolved=initial;first=nil}
                else{
                    let candidates=try await actionQuery().filter {exactUTF8($0["title"] as? String ?? "",action["title"] as? String ?? "") && exactUTF8($0["kind"] as? String ?? "",action["kind"] as? String ?? "")}
                    guard candidates.count==1 else{throw EngineError(message:"The selected provider action changed or is ambiguous. Prepare actions again.")}
                    resolved=try await resolver(candidates[0])
                }
                guard self.revision==request,self.choiceRevision==choice,captured.current() else{throw EngineError(message:"Source, selection or provider changed. Prepare actions again.")}
                if ["fixAll","sourceActions"].contains(self.mode ?? ""),scopedProviderActions([resolved],self.mode).isEmpty{throw EngineError(message:"The provider changed the complete file-wide action kind. Prepare again.")}
                if let disabled=resolved["disabled"] as? [String:Any]{throw EngineError(message:disabled["reason"] as? String ?? "Provider action disabled")}
                if resolved["command"] != nil{throw EngineError(message:"This action requires a provider command; its edit has not been applied.")}
                return resolved
            },load:acquisition,current:{self.revision==request && self.choiceRevision==choice && captured.current()},proofs:requiresProviderProofs)
            guard revision==request,choiceRevision==choice else{return}
            self.captured?.sources=acquired.sources;let resolved=acquired.value
            guard let edit=resolved["edit"] else{throw EngineError(message:"Provider source acquisition did not produce an edit. Prepare again.")}
            if isResourceWorkspaceEdit(edit){await workspace.resourceRestoration.prepareEdit(workspace,title:resolved["title"] as? String ?? "Provider resource action",edit:edit,current:{self.revision==request && self.choiceRevision==choice && captured.current()})}
            else{try install(resolved["title"] as? String ?? "Provider action",edit:edit,workspace:workspace)}
        }catch{if revision==request,choiceRevision==choice{notice=error.localizedDescription}}
    }
    func apply(_ workspace: Workspace) {
        guard let plan else { return }
        do {
            guard captured?.current() == true else { throw EngineError(message:"Source, selection or provider changed. Prepare again.") }
            let refresh = try workspace.applyWorkspacePlan(plan)
            undo = Array(undo.suffix(31)) + [.drafts(captureDraftHistory(invertChangePlan(plan),workspace:workspace))]; redo = []; historySnapshot=UUID(); self.plan = nil; captured = nil
            notice = "Applied \(plan.patches.count) source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch." + refresh
        } catch { notice = error.localizedDescription }
    }
}
@MainActor extension Workspace {
    func sourceSnapshots() -> NativeSourceMap<SourceSnapshot> { buffers.mapValues { SourceSnapshot(text:$0.text,writable:!$0.readOnly && !$0.saving) } }
    func applyWorkspacePlan(_ plan: ChangePlan) throws -> String {
        if let refusal = validateChangePlan(plan,sources:sourceSnapshots()) { throw EngineError(message:refusal) }
        let targets = plan.patches.map { patch in (buffer:buffers[patch.path]!,patch:patch,edits:patch.edits.map { NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text) }) }
        for target in targets {
            if target.buffer.applySourceEdits != nil && target.buffer.canReflectWorkspaceEdits?(target.edits,target.patch.before) != true { throw EngineError(message:"Finish composition or restore current source before applying: \(target.patch.path)") }
        }
        // Source authority commits synchronously before presentation catches up.
        // No file writes or awaits occur inside this boundary.
        for target in targets { target.buffer.text = target.patch.after }
        var delayed: [String] = []
        for target in targets where target.buffer.reflectWorkspaceEdits != nil {
            if target.buffer.reflectWorkspaceEdits?(target.edits,target.patch.before,target.patch.after) != true { delayed.append(target.patch.path) }
        }
        return delayed.isEmpty ? "" : " Drafts were applied; reopen these editor views to refresh presentation: " + delayed.joined(separator:", ")
    }
}
struct WorkspaceChangesView: View {
    @Bindable var workspace: Workspace
    var body: some View {
        @Bindable var changes = workspace.changes
        let actionSnapshot=changes.actionSnapshot
        VStack(alignment:.leading,spacing:8) {
            HStack { Text("Workspace change preview").font(.headline); Spacer(); Button("Close") { changes.close(workspace) } }
            Text(workspace.resourceRestoration.review==nil ? "Source drafts · not saved or verified":"Saved baselines and separate dirty drafts · review before publication").font(.caption)
            if changes.mode == "rename" {
                TextField("New symbol name", text: $changes.name)
                    .textFieldStyle(.roundedBorder)
#if os(iOS)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
#endif
            }
            if changes.mode == "entryMove",let path=changes.entrySelection?.entryPath {
                LabeledContent("Current main construction",value:path)
                TextField("New workspace-relative path",text:$changes.name).textFieldStyle(.roundedBorder)
                    .onChange(of:changes.name){_,_ in workspace.resourceRestoration.cancel()}
                    .disabled(workspace.resourceRestoration.hasPending)
                Button("Prepare move"){Task{await changes.prepare(workspace)}}
                    .disabled(changes.name.isEmpty||exactUTF8(changes.name,path)||workspace.resourceRestoration.busy||workspace.resourceRestoration.hasPending)
            }
            if changes.mode != "undo", changes.mode != "redo", changes.mode != "proposal", changes.mode != "entryMove" { Button(changes.mode == "fixAll" ? "Prepare file-wide diagnostic fixes" : changes.mode == "sourceActions" ? "Prepare source actions" : changes.mode == "actions" ? "Prepare provider actions" : changes.mode == "format" ? "Prepare formatting" : changes.mode == "cleanup" ? "Prepare cleanup" : "Prepare rename") { Task { await changes.prepare(workspace) } }.disabled(changes.mode == "rename" && changes.name.trimmingCharacters(in:.whitespacesAndNewlines).isEmpty) }
            if !changes.notice.isEmpty { Text(changes.notice).font(.caption) }
            ResourceRestorationPreview(workspace:workspace,model:workspace.resourceRestoration)
            ScrollView { VStack(alignment:.leading,spacing:6) {
                ForEach(Array(changes.choices.indices.dropFirst(changes.actionPage*100).prefix(100)),id:\.self) { index in
                    Button(changes.choices[index]["title"] as? String ?? "Provider action") { Task {await changes.choose(index,workspace:workspace,expectedSnapshot:actionSnapshot)} }
                    if let disabled = changes.choices[index]["disabled"] as? [String:Any], let reason = disabled["reason"] as? String { Text(reason).font(.caption) }
                }
                if let plan = changes.plan, let patch = plan.patches.dropFirst(min(changes.file,plan.patches.count-1)).first {
                    Text("\(plan.title) · \(plan.patches.count) source files")
                    if changes.mode == "rename" { Text("Reference targets: \(Set(changes.referencePaths).count) files · project analysis scope not established").font(.caption) }
                    Picker("Preview source file",selection:$changes.file) { ForEach(plan.patches.indices,id:\.self) { index in Text(plan.patches[index].path).tag(index) } }.onChange(of:changes.file) { _,_ in changes.page = 0 }
                    let before = patch.before.components(separatedBy:"\n"), after = patch.after.components(separatedBy:"\n")
                    let last = max(0,(max(before.count,after.count)+99)/100-1), selected = min(changes.page,last)
                    Text("Before · lines \(selected*100+1)–\(min(before.count,(selected+1)*100))").font(.caption)
                    Text(before.dropFirst(selected*100).prefix(100).joined(separator:"\n")).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
                    Text("After").font(.caption); Text(after.dropFirst(selected*100).prefix(100).joined(separator:"\n")).font(.system(.caption,design:.monospaced)).textSelection(.enabled)
                    if last > 0 { HStack { Button("Previous lines") { changes.page = selected-1 }.disabled(selected == 0); Text("\(selected+1)/\(last+1)"); Button("Next lines") { changes.page = selected+1 }.disabled(selected == last) } }
                    Button("Apply to source drafts") { changes.apply(workspace) }
                }
            } }
            if changes.choices.count > 100 { HStack { Button("Previous actions") { changes.actionPage -= 1 }.disabled(changes.actionPage == 0); Button("Next actions") { changes.actionPage += 1 }.disabled((changes.actionPage+1)*100 >= changes.choices.count) } }
        }.padding(8).frame(minWidth:260,idealWidth:380,maxWidth:480,minHeight:320,idealHeight:480,maxHeight:600)
    }
}
private struct CleanupResponse:Decodable {
    struct Point:Decodable {let line:Int;let character:Int;var wire:[String:Int]{["line":line,"character":character]}}
    struct Span:Decodable {let start:Point;let end:Point}
    struct Replacement:Decodable {let range:Span;let newText:String}
    let changes:[String:[Replacement]]
    var workspaceEdit:[String:Any]{["changes":changes.mapValues{edits in edits.map{["range":["start":$0.range.start.wire,"end":$0.range.end.wire],"newText":$0.newText] as [String:Any]}}]}
}
