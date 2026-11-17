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
        let valid=captured!.current
        let propose: (String,String) async throws->Any = proposal ?? {kind,name in try await language.propose(kind,name:name)}
                var listed:NativeSourceMap<FileNode>=[:]
                func collect(_ nodes:[FileNode]){for file in nodes{if !file.dir{listed[file.path]=file};collect(file.children ?? [])}};collect(workspace.rootFiles)
        let load:(String) async throws->SourceSnapshot=acquisition ?? {[weak workspace] path in
                    guard let workspace else{throw EngineError(message:"Workspace closed. Prepare again.")}
                    guard valid() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}
                    guard let file=workspace.buffers[path]?.file ?? listed[path],!file.dir else{throw EngineError(message:"Referenced source is no longer listed: \(path). Refresh files and prepare again.")}
                    let target=try await workspace.loadBuffer(file),text=target.text
                    guard !target.readOnly,!target.saving,let service=target.language else{throw EngineError(message:"Referenced source cannot synchronize: \(path). Retry language services.")}
                    _ = try await service.readOutline(source:text)
                    guard valid(),exactUTF8(target.text,text) else{throw EngineError(message:"Source changed while reading \(path). Prepare again.")}
                    return SourceSnapshot(text:text,writable:!target.readOnly && !target.saving)
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
        revision=UUID();let request=revision
        captured=(sources,{[weak self,weak workspace] in
            guard let self,let workspace,self.revision==request else{return false}
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
private struct CleanupResponse:Decodable {
    struct Point:Decodable {let line:Int;let character:Int;var wire:[String:Int]{["line":line,"character":character]}}
    struct Span:Decodable {let start:Point;let end:Point}
    struct Replacement:Decodable {let range:Span;let newText:String}
    let changes:[String:[Replacement]]
    var workspaceEdit:[String:Any]{["changes":changes.mapValues{edits in edits.map{["range":["start":$0.range.start.wire,"end":$0.range.end.wire],"newText":$0.newText] as [String:Any]}}]}
}
