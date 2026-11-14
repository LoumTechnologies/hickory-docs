<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preview native changes without owning the editor

For maintainers changing workspace services independently of the editor renderer.

The existing Edit menu requests a rename, formatting or provider action. Capture all retained source buffers and the originating selection before asking; an obsolete provider answer or edited target cannot become a preview. The preview pages every before/after line without reserving editor space. Application preflights every target, including retained editors and IME state, before synchronously committing the prepared replacements. It changes drafts only. The same exact inverse model drives bounded workspace undo/redo; later local edits refuse coordinated undo rather than being overwritten. Provider commands and resource operations remain explicit unsupported results, not partially applied edits.

<ui:copy id="simple-apple-workspace-changes">
import SwiftUI
import Observation
@MainActor @Observable final class WorkspaceChanges {
    var mode: String?
    var name = "" { didSet { if name != oldValue { revision = UUID(); plan = nil; choices = []; notice = "" } } }
    var notice = ""
    var plan: ChangePlan?
    var choices: [[String:Any]] = []
    var file = 0; var page = 0; var actionPage = 0
    var referencePaths: [String] = []
    private var revision = UUID()
    private var captured: (sources:[String:SourceSnapshot], current:()->Bool)?
    private var undo: [ChangePlan] = []
    private var redo: [ChangePlan] = []
    func reset() { revision = UUID(); mode = nil; captured = nil; plan = nil; choices = []; undo = []; redo = []; notice = "" }
    func close(_ workspace: Workspace) { revision = UUID(); mode = nil; workspace.currentBuffer?.focusSource?() }
    func open(_ kind: String, workspace: Workspace) {
        revision = UUID(); mode = kind; name = ""; plan = nil; choices = []; notice = ""; actionPage = 0
        guard kind == "undo" || kind == "redo" else { return }
        let target = kind == "undo" ? undo.last : redo.last
        guard let target else { notice = "No workspace change to \(kind)."; return }
        do {
            let refresh = try workspace.applyWorkspacePlan(target)
            if kind == "undo" { undo.removeLast(); redo.append(invertChangePlan(target)) } else { redo.removeLast(); undo.append(invertChangePlan(target)) }
            notice = "Workspace \(kind) applied to \(target.patches.count) drafts. Not saved or verified." + refresh
        } catch { notice = error.localizedDescription }
    }
    func prepare(_ workspace: Workspace) async {
        guard let origin = workspace.currentBuffer, let language = origin.language, !origin.readOnly else { notice = "Choose writable source with a connected language provider."; return }
        revision = UUID(); let request = revision, source = origin.text, selection = origin.selection, identity = language.proposalIdentity
        let sources = workspace.sourceSnapshots()
        captured = (sources,{ [weak self, weak origin, weak language] in
            guard let self, let origin, let language else { return false }
            return self.revision == request && origin.text == source && origin.selection == selection && origin.language === language && language.proposalIdentity == identity
        })
        plan = nil; choices = []; referencePaths = []; notice = "Preparing complete source changes…"
        let kind = mode ?? "actions", nextName = name
        do {
            if kind == "rename" {
                let references = try await language.propose("references",name:"")
                guard captured?.current() == true else { if revision == request { notice = "Source, selection or provider changed. Prepare again." }; return }
                referencePaths = semanticLocations(references).compactMap(\.path)
            }
            let result = try await language.propose(kind,name:nextName)
            guard captured?.current() == true else { if revision == request { notice = "Source, selection or provider changed. Prepare again." }; return }
            if kind == "actions" { choices = result as? [[String:Any]] ?? []; notice = choices.isEmpty ? "No provider actions here. Support depends on the installed language server." : "Choose a provider action to preview." }
            else { try install(kind == "rename" ? "Rename to \(nextName)" : "Format source", edit:result, workspace:workspace) }
        } catch { if revision == request { notice = error.localizedDescription } }
    }
    private func install(_ title: String, edit: Any, workspace: Workspace) throws {
        guard let captured, captured.current() else { throw EngineError(message:"Source, selection or provider changed. Prepare again.") }
        let next = try planWorkspaceEdit(title,raw:edit,sources:captured.sources)
        if let refusal = validateChangePlan(next,sources:workspace.sourceSnapshots()) { throw EngineError(message:refusal) }
        try validateReferencedTargets(next,referencePaths:referencePaths)
        plan = next; choices = []; file = 0; page = 0; notice = ""
    }
    func choose(_ index: Int, workspace: Workspace) {
        guard choices.indices.contains(index) else { return }; let action = choices[index]
        do {
            if let disabled = action["disabled"] as? [String:Any] { throw EngineError(message:disabled["reason"] as? String ?? "Provider action disabled") }
            if action["command"] != nil { throw EngineError(message:"This action requires a provider command; its edit has not been applied.") }
            try install(action["title"] as? String ?? "Provider action",edit:action["edit"] ?? NSNull(),workspace:workspace)
        } catch { notice = error.localizedDescription }
    }
    func apply(_ workspace: Workspace) {
        guard let plan else { return }
        do {
            guard captured?.current() == true else { throw EngineError(message:"Source, selection or provider changed. Prepare again.") }
            let refresh = try workspace.applyWorkspacePlan(plan)
            undo = Array(undo.suffix(31)) + [invertChangePlan(plan)]; redo = []; self.plan = nil; captured = nil
            notice = "Applied \(plan.patches.count) source drafts. Not saved or verified. Use Save All after reviewing; Undo workspace change restores this batch." + refresh
        } catch { notice = error.localizedDescription }
    }
}
</ui:copy>

## Commit source authority before reflecting native views

All buffers and retained editors are preflighted before any model changes. Commit the authorized draft texts synchronously on the main actor, then reflect the same replacements through native undo-aware editing. A late presentation failure cannot leave only some source models changed: keep all draft changes, report the views needing refresh, and retain the batch inverse. Missing editor views borrow the committed text when first attached.

<ui:copy id="simple-apple-workspace-changes-application">
@MainActor extension Workspace {
    func sourceSnapshots() -> [String:SourceSnapshot] { buffers.mapValues { SourceSnapshot(text:$0.text,writable:!$0.readOnly && !$0.saving) } }
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
        VStack(alignment:.leading,spacing:8) {
            HStack { Text("Workspace change preview").font(.headline); Spacer(); Button("Close") { changes.close(workspace) } }
            Text("Source drafts · not saved or verified").font(.caption)
            if changes.mode == "rename" { TextField("New symbol name",text:$changes.name).textFieldStyle(.roundedBorder) }
            if changes.mode != "undo", changes.mode != "redo" { Button(changes.mode == "actions" ? "Prepare provider actions" : changes.mode == "format" ? "Prepare formatting" : "Prepare rename") { Task { await changes.prepare(workspace) } }.disabled(changes.mode == "rename" && changes.name.trimmingCharacters(in:.whitespacesAndNewlines).isEmpty) }
            if !changes.notice.isEmpty { Text(changes.notice).font(.caption) }
            ScrollView { VStack(alignment:.leading,spacing:6) {
                ForEach(Array(changes.choices.indices.dropFirst(changes.actionPage*100).prefix(100)),id:\.self) { index in
                    Button(changes.choices[index]["title"] as? String ?? "Provider action") { changes.choose(index,workspace:workspace) }
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



<ui:file path="apps/apple-simple/Sources/WorkspaceChanges.swift"><ui:paste select="#simple-apple-workspace-changes"/><ui:paste select="#simple-apple-workspace-changes-application"/><ui:paste select="#simple-apple-workspace-changes-presentation"/></ui:file>
</ui:doc>
