<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preview native changes without owning the editor

For maintainers changing workspace services independently of the editor renderer.

The existing Edit menu requests a rename, formatting or provider action. Capture all retained source buffers and the originating selection before asking; an obsolete provider answer or edited target cannot become a preview. The preview pages every before/after line without reserving editor space. Application preflights every target, including retained editors and IME state, before synchronously committing the prepared replacements. It changes drafts only. The bounded workspace history orders draft inverse plans and resource recovery identities together. A resource undo/redo prepares a restoration review; only confirmed publication transfers its inverse record to the other stack. A history revision binds that review so a later workspace change cannot borrow an older stack position. The exact inverse model drives draft undo/redo; later local edits refuse coordinated undo rather than being overwritten. Provider commands still refuse as complete actions. Provider resource operations use the engine’s complete namespace acquisition, construction analysis and saved-source publication review. The resource path never falls back to applying only text edits. Newly acquired provider sources synchronize and the full proposal is queried again before review.

Rename discovery acquires previously unopened reference targets through the existing workspace buffer loader. Their language sessions synchronize before a fresh reference query; the discovering answer cannot authorize edits against later-loaded text. Acquisition never activates a tab. Captured source snapshots remain immutable inputs to the final provider proposal, and complete draft-plan validation still precedes application. Native preparation retains its own fixed request-validity closure so an older reply cannot borrow a newer request's authority.

Provider choices resolve through their captured provider before entering the same complete change-plan preview. A latest-choice ticket and the fixed preparation snapshot reject late answers or a replaced choice list. Disabled actions and complete edit-plus-command results refuse explicitly; resolving never applies an edit portion or saves source. The native view passes the rendered preparation identity so an old row index cannot select a newer list item.

Provider edit discovery uses the same bounded acquisition boundary. Newly loaded targets synchronize before the host re-queries actions at the captured selection and resolves a uniquely matching title and kind. Changed or ambiguous choices refuse instead of selecting an arbitrary replacement. Tokens from the discovering response never authorize the final plan. Original draft snapshots remain fixed; only missing sources are added, and acquisition does not activate tabs.

Explicit cleanup enters before language-provider acquisition. Its fixed source, selection, buffer identity and request ticket authorize only that preparation. Returning ordinary source replacements lets the existing preview, writability checks, coordinated apply and inverse history handle it; no second editor or save path is introduced. A no-op reports the observation without creating an undo entry. The cleanup profile is literal and explicit: significant trailing spaces can change, so review the before/after text.

Reviewed capabilities can offer a complete source proposal without a language provider. Every target must have the exact supplied reviewed text, and the ordinary planner and writability checks must accept the whole proposal before it becomes a preview. Capture target buffer identities, text and selections for the approval boundary. Source or selection changes refuse; successful approval uses the same coordinated inverse history as provider changes. The proposal mode has no provider-preparation button and never saves implicitly.

File-wide fixes request the standard `source.fixAll` family for the whole captured source. Retain only explicitly classified complete actions; unrelated quick fixes are never merged or applied speculatively. The existing complete action preview, provider requery, target acquisition and coordinated draft history own application. An empty result is unavailable, not a successful fix.

<ui:copy id="simple-apple-workspace-changes">
import SwiftUI
func fileWideFixActions(_ actions:[[String:Any]])->[[String:Any]] {actions.filter{guard let kind=$0["kind"] as? String else{return false};return kind=="source.fixAll" || kind.hasPrefix("source.fixAll.")}}
import Observation
struct ResourceHistoryIntent {var direction:String;var record:String;var token:UUID}
private enum WorkspaceHistoryStep {case drafts(ChangePlan);case resource(String)}
@MainActor @Observable final class WorkspaceChanges {
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
    private var captured: (sources:NativeSourceMap<SourceSnapshot>, current:()->Bool)?
    private var undo: [WorkspaceHistoryStep] = []
    private var redo: [WorkspaceHistoryStep] = []
    private(set) var historySnapshot=UUID()
    func resourceHistoryRecord(_ direction:String)->String? {
        if case .resource(let record)? = (direction=="undo" ? undo.last:redo.last) {return record};return nil
    }
    func recordResourcePublication(_ record:String,intent:ResourceHistoryIntent?) {
        if let intent,intent.token==historySnapshot,resourceHistoryRecord(intent.direction)==intent.record {
            if intent.direction=="undo" {undo.removeLast();redo=Array(redo.suffix(31))+[.resource(record)]}
            else {redo.removeLast();undo=Array(undo.suffix(31))+[.resource(record)]}
        }else {undo=Array(undo.suffix(31))+[.resource(record)];redo=[]}
        historySnapshot=UUID()
    }
    func reset() { revision = UUID(); mode = nil; captured = nil; plan = nil; choices = []; undo = []; redo = []; historySnapshot=UUID(); notice = "" }
    func close(_ workspace: Workspace) { revision = UUID(); mode = nil; workspace.resourceRestoration.cancel();workspace.currentBuffer?.focusSource?() }
    func open(_ kind: String, workspace: Workspace) {
        revision = UUID(); mode = kind; name = ""; plan = nil; choices = []; notice = ""; actionPage = 0
        guard kind == "undo" || kind == "redo" else { return }
        let target = kind == "undo" ? undo.last : redo.last
        guard let target else { notice = "No workspace change to \(kind)."; return }
        switch target {
        case .resource(let record):
            let intent=ResourceHistoryIntent(direction:kind,record:record,token:historySnapshot)
            Task {await workspace.resourceRestoration.prepareHistory(workspace,intent:intent)}
            notice="Prepare and review saved-source restoration before publishing this workspace \(kind)."
        case .drafts(let target):
            do {
                let refresh=try workspace.applyWorkspacePlan(target)
                if kind=="undo" {undo.removeLast();redo.append(.drafts(invertChangePlan(target)))}else{redo.removeLast();undo.append(.drafts(invertChangePlan(target)))}
                historySnapshot=UUID()
                notice="Workspace \(kind) applied to \(target.patches.count) drafts. Not saved or verified."+refresh
            }catch{notice=error.localizedDescription}
        }
    }
    func prepare(_ workspace: Workspace,proposal:((String,String) async throws->Any)?=nil,resolution:(([String:Any]) async throws->[String:Any])?=nil,acquisition:((String) async throws->SourceSnapshot)?=nil) async {
        if mode == "cleanup" { await prepareCleanup(workspace,proposal:proposal); return }
        guard let origin = workspace.currentBuffer, let language = origin.language, !origin.readOnly else { notice = "Choose writable source with a connected language provider."; return }
        revision = UUID(); let request = revision, source = origin.text, selection = origin.selection, identity = language.proposalIdentity
        let sourceIdentity=origin.sourceEditIdentity
        let sources = workspace.sourceSnapshots()
        captured = (sources,{ [weak self, weak origin, weak language,weak workspace] in
            guard let self, let origin, let language,let workspace else { return false }
            return self.revision == request && workspace.buffers[origin.file.path] === origin && origin.sourceEditIdentity==sourceIdentity && exactUTF8(origin.text,source) && origin.selection == selection && origin.language === language && language.proposalIdentity == identity
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
                    guard !target.readOnly,!target.saving,let service=target.language else{throw EngineError(message:"Referenced source cannot synchronize: \(path). Retry language services.")}
                    _ = try await service.readOutline(source:text)
                    guard valid(),exactUTF8(target.text,text) else{throw EngineError(message:"Source changed while reading \(path). Prepare again.")}
                    return SourceSnapshot(text:text,writable:!target.readOnly && !target.saving)
                }
        let load:(String) async throws->SourceSnapshot={path in
            guard valid() else{throw EngineError(message:"Original source owners changed before acquisition. Prepare again.")}
            let wasRetained=workspace.buffers[path] != nil,source=try await acquireTarget(path)
            guard lease.current(allowing:path),let target=workspace.buffers[path],wasRetained || target.agentSourceRevision==0 && exactUTF8(target.text,target.base),exactUTF8(target.text,source.text),!target.readOnly,!target.saving else{throw EngineError(message:"Original source owners or the acquired target changed. Prepare again.")}
            lease.retain(path,target);return source
        }
        let actionKind=mode=="fixAll" ? "fixAll":"actions"
        self.acquisition=load;actionQuery={let actions=try await propose(actionKind,"") as? [[String:Any]] ?? [];return actionKind=="fixAll" ? fileWideFixActions(actions):actions}
        plan = nil; choices = []; referencePaths = []; notice = "Preparing complete source changes…"
        let kind = mode ?? "actions", nextName = name
        do {
            if kind == "rename" {
                let acquired=try await acquireReferenceSources(sources,query:{
                    let locations=semanticLocations(try await propose("references",""))
                    return try locations.map {location in guard let path=location.path else{throw EngineError(message:"Reference target has no writable workspace source identity. No edits were applied.")};return path}
                },acquire:load,current:valid)
                guard valid() else{return};captured?.sources=acquired.sources;referencePaths=acquired.references
            }
            let result = try await propose(kind,nextName)
            guard valid() else { if revision == request { notice = "Source, selection or provider changed. Prepare again." }; return }
            if kind == "actions" || kind == "fixAll" { let actions=result as? [[String:Any]] ?? [];choices=kind=="fixAll" ? fileWideFixActions(actions):actions;notice=choices.isEmpty ? (kind=="fixAll" ? "The current provider offers no file-wide diagnostic fixes. No edits were applied.":"No provider actions here. Support depends on the installed language server."):"Choose one complete provider action to preview. Nothing is saved or verified." }
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
            if mode=="fixAll",let first,fileWideFixActions([first]).isEmpty{throw EngineError(message:"The provider changed the complete file-wide action kind. Prepare again.")}
            if let resolved=first,let edit=resolved["edit"],isResourceWorkspaceEdit(edit) {
                guard self.revision==request,self.choiceRevision==choice,captured.current() else{throw EngineError(message:"Source, selection or provider changed. Prepare actions again.")}
                if self.mode=="fixAll",fileWideFixActions([resolved]).isEmpty{throw EngineError(message:"The provider changed the complete file-wide action kind. Prepare again.")}
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
            let acquired=try await acquireSourceProposal(captured.sources,query:{
                let resolved:[String:Any]
                if let initial=first{resolved=initial;first=nil}
                else{
                    let candidates=try await actionQuery().filter {exactUTF8($0["title"] as? String ?? "",action["title"] as? String ?? "") && exactUTF8($0["kind"] as? String ?? "",action["kind"] as? String ?? "")}
                    guard candidates.count==1 else{throw EngineError(message:"The selected provider action changed or is ambiguous. Prepare actions again.")}
                    resolved=try await resolver(candidates[0])
                }
                guard self.revision==request,self.choiceRevision==choice,captured.current() else{throw EngineError(message:"Source, selection or provider changed. Prepare actions again.")}
                if self.mode=="fixAll",fileWideFixActions([resolved]).isEmpty{throw EngineError(message:"The provider changed the complete file-wide action kind. Prepare again.")}
                if let disabled=resolved["disabled"] as? [String:Any]{throw EngineError(message:disabled["reason"] as? String ?? "Provider action disabled")}
                if resolved["command"] != nil{throw EngineError(message:"This action requires a provider command; its edit has not been applied.")}
                return (paths:try providerActionPaths(resolved),value:resolved)
            },acquire:acquisition,current:{self.revision==request && self.choiceRevision==choice && captured.current()})
            guard revision==request,choiceRevision==choice else{return}
            self.captured?.sources=acquired.sources;let resolved=acquired.value
            guard let edit=resolved["edit"] else{throw EngineError(message:"Provider source acquisition did not produce an edit. Prepare again.")}
            try install(resolved["title"] as? String ?? "Provider action",edit:edit,workspace:workspace)
        }catch{if revision==request,choiceRevision==choice{notice=error.localizedDescription}}
    }
    func apply(_ workspace: Workspace) {
        guard let plan else { return }
        do {
            guard captured?.current() == true else { throw EngineError(message:"Source, selection or provider changed. Prepare again.") }
            let refresh = try workspace.applyWorkspacePlan(plan)
            undo = Array(undo.suffix(31)) + [.drafts(invertChangePlan(plan))]; redo = []; historySnapshot=UUID(); self.plan = nil; captured = nil
            notice = "Applied \(plan.patches.count) source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch." + refresh
        } catch { notice = error.localizedDescription }
    }
}
</ui:copy>

## Commit source authority before reflecting native views

All buffers and retained editors are preflighted before any model changes. Commit the authorized draft texts synchronously on the main actor, then reflect the same replacements through native undo-aware editing. A late presentation failure cannot leave only some source models changed: keep all draft changes, report the views needing refresh, and retain the batch inverse. Missing editor views borrow the committed text when first attached.

<ui:copy id="simple-apple-workspace-changes-application">
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
</ui:copy>

## Page the complete preview in the native presentation

The existing Edit menu presents this bounded floating sheet. Every affected source and every before/after line remains reachable through file selection and line pages. The view reads the controller and requests operations; it does not open sockets, fabricate provider edits or save source.

<ui:copy id="simple-apple-workspace-changes-presentation">
struct WorkspaceChangesView: View {
    @Bindable var workspace: Workspace
    var body: some View {
        @Bindable var changes = workspace.changes
        let actionSnapshot=changes.actionSnapshot
        VStack(alignment:.leading,spacing:8) {
            HStack { Text("Workspace change preview").font(.headline); Spacer(); Button("Close") { changes.close(workspace) } }
            Text(workspace.resourceRestoration.review==nil ? "Source drafts · not saved or verified":"Saved baselines and separate dirty drafts · review before publication").font(.caption)
            if changes.mode == "rename" { TextField("New symbol name",text:$changes.name).textFieldStyle(.roundedBorder) }
            if changes.mode != "undo", changes.mode != "redo", changes.mode != "proposal" { Button(changes.mode == "fixAll" ? "Prepare file-wide diagnostic fixes" : changes.mode == "actions" ? "Prepare provider actions" : changes.mode == "format" ? "Prepare formatting" : changes.mode == "cleanup" ? "Prepare cleanup" : "Prepare rename") { Task { await changes.prepare(workspace) } }.disabled(changes.mode == "rename" && changes.name.trimmingCharacters(in:.whitespacesAndNewlines).isEmpty) }
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
</ui:copy>



## Decode cleanup into the existing workspace edit model

Keep the network response typed, then pass its ordinary replacements through the same source-range validation and preview planner as provider actions. Decoding does not authorize edits or bypass the source capture.

<ui:copy id="cleanup-apple-wire">
private struct CleanupResponse:Decodable {
    struct Point:Decodable {let line:Int;let character:Int;var wire:[String:Int]{["line":line,"character":character]}}
    struct Span:Decodable {let start:Point;let end:Point}
    struct Replacement:Decodable {let range:Span;let newText:String}
    let changes:[String:[Replacement]]
    var workspaceEdit:[String:Any]{["changes":changes.mapValues{edits in edits.map{["range":["start":$0.range.start.wire,"end":$0.range.end.wire],"newText":$0.newText] as [String:Any]}}]}
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/WorkspaceChanges.swift"><ui:paste select="#simple-apple-workspace-changes"/><ui:paste select="#simple-apple-workspace-changes-application"/><ui:paste select="#simple-apple-workspace-changes-presentation"/><ui:paste select="#cleanup-apple-wire"/></ui:file>
## Retire older proposal requests

Hold two provider replies against the same unchanged source and selection. Completing the newer rename first must leave its preview intact when the older result arrives. A byte-distinct Unicode name change also retires the previous preview. Each preparation therefore retains its own validity closure; consulting a mutable current-request property would let old work borrow new authority. The injected provider boundary tests asynchronous intent safety without claiming an actual language-server run; source stays unchanged until explicit application.

<ui:copy id="native-change-request-apple-witness">
import XCTest
@testable import HickorySimple
@MainActor final class ChangeRequestTests:XCTestCase {
    func testCleanupAndReviewedDraftRefuseEqualByteRevisionRenewal()async throws {
        let w=Workspace(),f=FileNode(name:"a.py",path:"a.py",dir:false),b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"old\n",hash:"base"));w.buffers[f.path]=b;w.show(f.path)
        let edit:[String:Any]=["changes":[languageURI(f.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]]]]
        try w.changes.offer("Reviewed",edit:edit,expected:[f.path:b.text],workspace:w);b.text="later";b.text="old\n";w.changes.apply(w);XCTAssertEqual(b.text,"old\n");XCTAssertTrue(w.changes.notice.contains("Source, selection or provider changed"))
        w.changes.open("cleanup",workspace:w);await w.changes.prepare(w,proposal:{_,_ in b.text="later";b.text="old\n";return edit});XCTAssertNil(w.changes.plan);XCTAssertEqual(b.text,"old\n");XCTAssertTrue(w.changes.notice.contains("Source or selection changed"))
    }

    func testOlderRenameReplyCannotBorrowNewerRequestAuthority()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"value\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        XCTAssertTrue(workspace.currentBuffer === buffer)
        let changes=workspace.changes;changes.open("rename",workspace:workspace)
        var held:[String:CheckedContinuation<Any,Error>]=[:]
        let proposal:(String,String) async throws->Any={kind,name in if kind=="references"{return [] as [[String:Any]]};return try await withCheckedThrowingContinuation {held[name]=$0}}
        func settle(_ condition:@escaping ()->Bool)async{let deadline=Date().addingTimeInterval(2);while !condition()&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
        func edit(_ name:String)->Any{["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":5]],"newText":name]]]]}
        changes.name="first";let first=Task {await changes.prepare(workspace,proposal:proposal)};await settle{held["first"] != nil}
        changes.name="second";let second=Task {await changes.prepare(workspace,proposal:proposal)};await settle{held["second"] != nil}
        try XCTUnwrap(held["second"]).resume(returning:edit("second"));await second.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n")
        try XCTUnwrap(held["first"]).resume(returning:edit("first"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n");XCTAssertEqual(buffer.text,"value\n")
        changes.name="é";let unicode=Task {await changes.prepare(workspace,proposal:proposal)};await settle{held["é"] != nil}
        try XCTUnwrap(held["é"]).resume(returning:edit("é"));await unicode.value;XCTAssertNotNil(changes.plan)
        changes.name="e\u{301}";XCTAssertNil(changes.plan,"A byte-distinct rename intent must retire its previous preview")
    }
}
</ui:copy>

## Exercise provider-independent cleanup and its authority

Use the real workspace change model with a source that has no language service. Preview leaves it unchanged; apply and coordinated undo/redo retain the saved baseline. Held replies must use their own capture, and closing retires pending preparation. These model witnesses do not claim physical input acceptance.

<ui:copy id="cleanup-apple-witness">
extension ChangeRequestTests {
    func testCleanupUsesCapturedSourceWithoutAProviderAndRetainsWorkspaceUndo() async throws {
        let workspace=Workspace(),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let before="😀 value \t\r\n",after="😀 value\r\n"
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:before,hash:"base"))
        workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes
        let edit:[String:Any]=["changes":[languageURI(file.path):[["range":["start":["line":0,"character":8],"end":["line":0,"character":10]],"newText":""]]]]
        changes.open("cleanup",workspace:workspace)
        await changes.prepare(workspace,proposal:{kind,_ in XCTAssertEqual(kind,"cleanup");return edit})
        XCTAssertEqual(buffer.text,before);XCTAssertEqual(changes.plan?.patches.first?.after,after)
        changes.apply(workspace);XCTAssertEqual(buffer.text,after)
        changes.open("undo",workspace:workspace);XCTAssertEqual(buffer.text,before)
        changes.open("redo",workspace:workspace);XCTAssertEqual(buffer.text,after)
        XCTAssertEqual(buffer.base,before)
        changes.open("cleanup",workspace:workspace)
        await changes.prepare(workspace,proposal:{_,_ in ["changes":[languageURI(file.path):[]]]})
        XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("No trailing"))
    }
    func testCleanupCannotBorrowNewerAuthorityOrApplyAfterClose() async throws {
        let workspace=Workspace(),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"x ",hash:"base"))
        workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes;changes.open("cleanup",workspace:workspace)
        var held:[CheckedContinuation<Any,Error>]=[]
        let proposal:(String,String) async throws->Any={_,_ in try await withCheckedThrowingContinuation{held.append($0)}}
        func settle(_ count:Int) async {let deadline=Date().addingTimeInterval(2);while held.count<count && Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
        func edit(_ text:String)->Any{["changes":[languageURI(file.path):[["range":["start":["line":0,"character":1],"end":["line":0,"character":2]],"newText":text]]]]}
        let first=Task{await changes.prepare(workspace,proposal:proposal)};await settle(1)
        let second=Task{await changes.prepare(workspace,proposal:proposal)};await settle(2)
        XCTAssertEqual(held.count,2);guard held.count==2 else{return}
        held[1].resume(returning:edit(""));await second.value;XCTAssertEqual(changes.plan?.patches.first?.after,"x")
        held[0].resume(returning:edit("old"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"x")
        let closed=Task{await changes.prepare(workspace,proposal:proposal)};await settle(3)
        changes.close(workspace);held[2].resume(returning:edit(""));await closed.value
        XCTAssertNil(changes.plan);XCTAssertEqual(buffer.text,"x ")
    }
}
</ui:copy>

## Keep reviewed-source proposals inside coordinated draft history

Offer exact reviewed text without a language provider. Missing or stale captures refuse before becoming previews. Approval applies through the same workspace history, while source or selection changes after preparation refuse. The saved baseline remains untouched until a separate save.

<ui:copy id="reviewed-source-apple-tests">
extension ChangeRequestTests {
 func testReviewedSourceProposalUsesSharedPreviewApplyAndUndo()throws{
  let workspace=Workspace(),file=FileNode(name:"a.txt",path:"a.txt",dir:false),before="draft\n",after="incoming\n"
  let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:before,hash:"base"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  let edit:[String:Any]=["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":1,"character":0]],"newText":after]]]]
  XCTAssertThrowsError(try workspace.changes.offer("Reviewed source",edit:edit,expected:[:],workspace:workspace));XCTAssertNil(workspace.changes.mode);XCTAssertEqual(buffer.text,before)
  try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:before],workspace:workspace);XCTAssertEqual(workspace.changes.mode,"proposal");XCTAssertEqual(buffer.text,before)
  workspace.changes.apply(workspace);XCTAssertEqual(buffer.text,after);XCTAssertEqual(buffer.base,before)
  workspace.changes.open("undo",workspace:workspace);XCTAssertEqual(buffer.text,before);workspace.changes.open("redo",workspace:workspace);XCTAssertEqual(buffer.text,after)
 }
 func testReviewedSourceProposalRefusesEditsOrSelectionChangesBeforeApproval()throws{
  let workspace=Workspace(),file=FileNode(name:"a.txt",path:"a.txt",dir:false)
  let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"x",hash:"base"));workspace.buffers[file.path]=buffer;workspace.show(file.path)
  let edit:[String:Any]=["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":"y"]]]]
  try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:"x"],workspace:workspace);buffer.text="typed";workspace.changes.apply(workspace);XCTAssertEqual(buffer.text,"typed")
  XCTAssertThrowsError(try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:"x"],workspace:workspace))
  buffer.text="x";try workspace.changes.offer("Reviewed source",edit:edit,expected:[file.path:"x"],workspace:workspace);buffer.selection=NSRange(location:1,length:0);workspace.changes.apply(workspace);XCTAssertEqual(buffer.text,"x")
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ChangeRequestTests.swift"><ui:paste select="#native-change-request-apple-witness"/><ui:paste select="#cleanup-apple-witness"/><ui:paste select="#reviewed-source-apple-tests"/></ui:file>
## Keep resolved choices with their preparation

Complete a newer choice before an older resolver reply, then reject a row from a replaced preparation and a reply after closing. These controlled callbacks exercise the actual native change model; source stays unchanged until explicit application. They do not claim native pointer or provider-language acceptance.

<ui:copy id="native-action-choice-apple-witness">
import XCTest
import SwiftUI
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class ActionChoiceTests:XCTestCase {
    func testOriginalTargetOwnersNeverRenewDuringQueriesOrApproval()async throws {
        for change in ["undo","guard","baseline","handle","unknown"] {
            let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false),other=FileNode(name:"other.py",path:"other.py",dir:false)
            let b=DocumentBuffer(file:f,text:FileText(path:f.path,content:"old\n",hash:"base")),target=DocumentBuffer(file:other,text:FileText(path:other.path,content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.buffers[other.path]=target;w.show(f.path)
            let action:[String:Any]=["title":"Fix all","kind":"source.fixAll.provider"]
            w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in
                switch change{case "undo":target.text="later";target.text="old\n";case "guard":target.savedGuard="foreign";case "baseline":target.base="foreign";case "handle":w.buffers[other.path]=DocumentBuffer(file:other,text:FileText(path:other.path,content:target.text,hash:target.hash));default:w.buffers["unknown.md"]=DocumentBuffer(file:FileNode(name:"unknown.md",path:"unknown.md",dir:false),text:FileText(path:"unknown.md",content:"unmodeled construction",hash:"new"))};return [action]
            });XCTAssertTrue(w.changes.choices.isEmpty,change);XCTAssertNil(w.changes.plan,change);XCTAssertEqual(b.text,"old\n")
        }
    }
    func testAcquiredEditUndoCannotBecomeANewProviderBaseline()async throws {
        let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false),b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.show(f.path)
        let action:[String:Any]=["title":"Fix all","kind":"source.fixAll.provider","data":["hickory_acquire":[["uri":languageURI("other.py")]]]]
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in [action]},resolution:{$0},acquisition:{path in let target=DocumentBuffer(file:FileNode(name:path,path:path,dir:false),text:FileText(path:path,content:"old\n",hash:"base"));w.buffers[path]=target;target.text="later";target.text="old\n";return SourceSnapshot(text:target.text,writable:true)});await w.changes.choose(0,workspace:w);XCTAssertNil(w.changes.plan);XCTAssertEqual(b.text,"old\n");XCTAssertTrue(w.changes.notice.contains("acquired target") || w.changes.notice.contains("sealed"),w.changes.notice)
    }

    #if os(macOS)
    func testLiveOwnedBulkFixCommandReviewApplyAndUndo()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_BULK_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_BULK_WORKSPACE"] else{throw XCTSkip("Use the maintained owned bulk-fix fixture.")}
        let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-bulk-fixture"),encoding:.utf8)=="owned-bulk-v1\n" else{throw EngineError(message:"An owned disposable workspace is required.")}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:origin),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
        let f=try XCTUnwrap(w.rootFiles.first{$0.path=="a.py"}),b=try await w.loadBuffer(f),source=b.text;w.layout.activateSource(f.path,in:"document-group")
        let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{for buffer in w.buffers.values{buffer.language?.close()};w.sourceTestIndex?.stop();window.orderOut(nil);window.contentView=nil}
        for _ in 0..<200{if b.sourceEditorState?() != nil{break};try await Task.sleep(for:.milliseconds(20))};let editor=try XCTUnwrap(b.sourceEditorState?());let size=host.bounds.size
        _ = b.selectSourceRange?(NSRange(location:2,length:0),source);let selection=b.selection
        try XCTUnwrap(w.commands().first{$0.id=="symbol.fixAll"}).run();XCTAssertEqual(w.changes.mode,"fixAll")
        for _ in 0..<200{await w.changes.prepare(w);if !w.changes.choices.isEmpty{break};try await Task.sleep(for:.milliseconds(100))}
        XCTAssertEqual(w.changes.choices.count,1,w.changes.notice);await w.changes.choose(0,workspace:w);XCTAssertEqual(w.changes.plan?.patches.count,2,w.changes.notice);XCTAssertEqual(b.text,source);XCTAssertEqual(b.selection,selection);XCTAssertEqual(b.sourceEditorState?()?.identity,editor.identity);XCTAssertEqual(host.bounds.size,size)
        XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("a.py"),encoding:.utf8),source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("other.py"),encoding:.utf8),source)
        w.changes.apply(w);XCTAssertEqual(b.text,"python_fixed = 1\n");XCTAssertEqual(w.buffers["other.py"]?.text,"other_fixed = 1\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source);XCTAssertEqual(w.buffers["other.py"]?.text,source);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"python_fixed = 1\n");XCTAssertEqual(b.sourceEditorState?()?.identity,editor.identity);XCTAssertEqual(host.bounds.size,size)
        XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("a.py"),encoding:.utf8),source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("other.py"),encoding:.utf8),source)
    }
    #endif

    func testCompleteFileFixChoiceFiltersPreviewsAndUsesCoordinatedUndo()async throws {
        let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false)
        let b=DocumentBuffer(file:f,text:FileText(path:f.path,content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.show(f.path)
        let action:[String:Any]=["title":"Fix all","kind":"source.fixAll.provider","edit":["changes":[languageURI(f.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]]]]]
        let choices=[action,["title":"single","kind":"quickfix"],["title":"unclassified"],["title":"prefix","kind":"source.fixAllWrong"]]
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{kind,_ in XCTAssertEqual(kind,"fixAll");return choices})
        XCTAssertEqual(w.changes.choices.count,1);await w.changes.choose(0,workspace:w);XCTAssertEqual(w.changes.plan?.patches.first?.after,"new\n");XCTAssertEqual(b.text,"old\n")
        w.changes.apply(w);XCTAssertEqual(b.text,"new\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,"old\n");w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"new\n")
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in [["title":"ordinary","kind":"quickfix"]]});XCTAssertTrue(w.changes.choices.isEmpty);XCTAssertTrue(w.changes.notice.contains("no file-wide"))
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in b.text="later";b.text="new\n";return [action]});XCTAssertTrue(w.changes.choices.isEmpty);XCTAssertNil(w.changes.plan)
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in [action]},resolution:{_ in var changed=action;changed["kind"]="quickfix";return changed});await w.changes.choose(0,workspace:w);XCTAssertNil(w.changes.plan);XCTAssertEqual(b.text,"new\n")
    }

    func testUnopenedActionTargetsAcquireThenRequeryAndRefuseAmbiguousChoices()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"old\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes
        let action:[String:Any]=["title":"Fix both","kind":"quickfix","data":["hickory_action":"first"]]
        var queries=0,resolutions=0,loads:[String]=[];var ambiguous=false
        let propose:(String,String) async throws->Any={_,_ in queries+=1;return ambiguous && queries>1 ? [action,action]:[action]}
        let resolve:([String:Any]) async throws->[String:Any]={_ in
            resolutions+=1
            if resolutions==1{return ["title":"Fix both","data":["hickory_acquire":[["uri":languageURI("other.py")]]]]}
            let replacement:[String:Any]=["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]
            return ["title":"Fix both","edit":["changes":[languageURI("a.py"):[replacement],languageURI("other.py"):[replacement]]]]
        }
        let load:(String) async throws->SourceSnapshot={path in
            loads.append(path);let target=DocumentBuffer(file:FileNode(name:path,path:path,dir:false),text:FileText(path:path,content:"old\n",hash:"base"));workspace.buffers[path]=target
            return SourceSnapshot(text:target.text,writable:true)
        }
        changes.open("actions",workspace:workspace);await changes.prepare(workspace,proposal:propose,resolution:resolve,acquisition:load);await changes.choose(0,workspace:workspace)
        XCTAssertEqual(queries,2);XCTAssertEqual(resolutions,2);XCTAssertEqual(loads,["other.py"]);XCTAssertEqual(changes.plan?.patches.count,2);XCTAssertEqual(workspace.currentBuffer?.file.path,"a.py");XCTAssertEqual(buffer.text,"old\n")
        workspace.buffers["other.py"]?.text="later\n";changes.apply(workspace);XCTAssertTrue(changes.notice.contains("Source, selection or provider changed"));XCTAssertEqual(buffer.text,"old\n")
        workspace.buffers.removeValue(forKey:"other.py");queries=0;ambiguous=true
        changes.open("actions",workspace:workspace);await changes.prepare(workspace,proposal:propose,resolution:resolve,acquisition:load);await changes.choose(0,workspace:workspace)
        XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("ambiguous"));XCTAssertEqual(buffer.text,"old\n")
    }

    func testLatestResolvedChoiceWinsAndRenderedSnapshotsCannotBorrowNewLists()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"old\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes;changes.open("actions",workspace:workspace)
        let actions:[[String:Any]]=["first","second"].map {["title":$0,"data":["hickory_action":$0]]}
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={action in try await withCheckedThrowingContinuation {held[action["title"] as! String]=$0}}
        func prepare()async{await changes.prepare(workspace,proposal:{_,_ in actions},resolution:resolve)}
        func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
        func resolved(_ name:String)->[String:Any]{["title":name,"edit":["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":name]]]]]}
        await prepare();let snapshot=changes.actionSnapshot
        let first=Task{await changes.choose(0,workspace:workspace,expectedSnapshot:snapshot)};await settle{held["first"] != nil}
        let second=Task{await changes.choose(1,workspace:workspace,expectedSnapshot:snapshot)};await settle{held["second"] != nil}
        try XCTUnwrap(held["second"]).resume(returning:resolved("second"));await second.value
        try XCTUnwrap(held["first"]).resume(returning:resolved("first"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n")
        held=[:];await prepare();await changes.choose(0,workspace:workspace,expectedSnapshot:snapshot);XCTAssertTrue(held.isEmpty);XCTAssertNil(changes.plan)
        let current=changes.actionSnapshot;let closing=Task{await changes.choose(0,workspace:workspace,expectedSnapshot:current)};await settle{held["first"] != nil};changes.close(workspace)
        try XCTUnwrap(held["first"]).resume(returning:resolved("first"));await closing.value;XCTAssertNil(changes.plan);XCTAssertEqual(buffer.text,"old\n")
        changes.open("actions",workspace:workspace)
        await changes.prepare(workspace,proposal:{_,_ in actions},resolution:{_ in var value=resolved("partial");value["command"]=["title":"Effect","command":"provider.effect"];return value})
        await changes.choose(0,workspace:workspace,expectedSnapshot:changes.actionSnapshot);XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("provider command"));XCTAssertEqual(buffer.text,"old\n")
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/ActionChoiceTests.swift"><ui:paste select="#native-action-choice-apple-witness"/></ui:file>
</ui:doc>
