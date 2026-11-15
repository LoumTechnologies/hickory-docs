import Foundation
struct SourceSnapshot { var text: String; var writable: Bool }
struct SourceReplacement: Equatable { var from: Int; var to: Int; var text: String
    static func ==(lhs:Self,rhs:Self)->Bool {lhs.from==rhs.from&&lhs.to==rhs.to&&exactUTF8(lhs.text,rhs.text)}
}
struct SourcePatch { var path: String; var before: String; var after: String; var edits: [SourceReplacement] }
struct ChangePlan { var title: String; var patches: [SourcePatch] }
private func changeOffset(_ source: String, _ raw: Any?) throws -> Int {
    guard let point = raw as? [String: Any], let line = point["line"] as? Int, let character = point["character"] as? Int,
          line >= 0, character >= 0, let value = languageOffset(source, ["line":line,"character":character]),
          String.Index(utf16Offset: value, in: source).samePosition(in: source.unicodeScalars) != nil else { throw EngineError(message: "Invalid or non-scalar UTF-16 source position") }
    return value
}
struct WorkspaceEditTarget {var path:String;var items:[[String:Any]]}
func workspaceEditTargets(_ raw:Any)throws->[WorkspaceEditTarget] {
    guard let edit=raw as? [String:Any] else{throw EngineError(message:"No workspace edit to preview")}
    var targets:[WorkspaceEditTarget]=[]
    func add(_ rawURI:Any?,_ rawEdits:Any?,_ version:Any?=nil)throws {
        guard let uri=rawURI as? String,let items=rawEdits as? [[String:Any]] else{throw EngineError(message:"Malformed workspace edit target")}
        guard version==nil || version is NSNull else{throw EngineError(message:"Authored source version could not be established. Refresh the proposal.")}
        guard let path=SemanticLocation(uri:uri,line:0,character:0).path else{throw EngineError(message:"Unsupported source identity: \(uri)")}
        targets.append(WorkspaceEditTarget(path:path,items:items))
    }
    if let changes = edit["changes"] {
        guard let changes = changes as? [String: Any] else { throw EngineError(message: "Malformed workspace changes") }
        for (uri, edits) in changes { try add(uri, edits) }
    }
    if let changes = edit["documentChanges"] {
        guard targets.isEmpty, let changes = changes as? [[String: Any]] else { throw EngineError(message: "Ambiguous workspace change representation") }
        for change in changes {
            guard change["kind"] == nil, let document = change["textDocument"] as? [String: Any] else { throw EngineError(message: "Resource create, move and delete require a filesystem proposal") }
            try add(document["uri"], change["edits"], document["version"])
        }
    }
    return targets
}
func providerActionPaths(_ action:[String:Any])throws->[String] {
    guard let raw=(action["data"] as? [String:Any])?["hickory_acquire"] else{guard let edit=action["edit"] else{throw EngineError(message:"No source edit to preview")};return try workspaceEditTargets(edit).map(\.path)}
    guard let items=raw as? [[String:Any]],!items.isEmpty,items.count<=10000 else{throw EngineError(message:"Malformed provider source acquisition")}
    return try items.map {item in guard let uri=item["uri"] as? String,let path=SemanticLocation(uri:uri,line:0,character:0).path else{throw EngineError(message:"Unsupported provider acquisition source")};return path}
}
func planWorkspaceEdit(_ title: String, raw: Any, sources: [String: SourceSnapshot]) throws -> ChangePlan {
    var groups: [String: [SourceReplacement]] = [:]
    for target in try workspaceEditTargets(raw) {
        let path=target.path,items=target.items
        guard let source=sources[path] else{throw EngineError(message:"Acquire the affected source before preparing this proposal: \(path)")}
        guard source.writable else { throw EngineError(message: "Source is generated, read-only or saving: \(path)") }
        var replacements = groups[path] ?? []
        for item in items {
            guard let range = item["range"] as? [String: Any], let text = item["newText"] as? String else { throw EngineError(message: "Malformed source replacement") }
            let replacement = try SourceReplacement(from: changeOffset(source.text, range["start"]), to: changeOffset(source.text, range["end"]), text: text)
            guard replacement.from <= replacement.to else { throw EngineError(message: "Source replacement runs backwards") }
            if !replacements.contains(replacement) { replacements.append(replacement) }
            guard replacements.count <= 10000 else { throw EngineError(message: "Proposal exceeds 10,000 replacements in one source") }
        }
        groups[path] = replacements
    }
    var patches: [SourcePatch] = []
    for path in groups.keys.sorted() {
        let edits = groups[path]!.sorted { ($0.from,$0.to) < ($1.from,$1.to) }
        for (left,right) in zip(edits,edits.dropFirst()) where right.from < left.to || right.from == left.from { throw EngineError(message: "Overlapping source edits: \(path)") }
        let before = sources[path]!.text
        let after = NSMutableString(string: before)
        for item in edits.reversed() { after.replaceCharacters(in: NSRange(location:item.from,length:item.to-item.from), with: item.text) }
        if !exactUTF8(before,after as String) { patches.append(SourcePatch(path:path,before:before,after:after as String,edits:edits)) }
    }
    guard !patches.isEmpty else { throw EngineError(message: "The provider proposed no source changes") }
    return ChangePlan(title:title,patches:patches)
}
func invertChangePlan(_ plan: ChangePlan) -> ChangePlan {
    ChangePlan(title:"Undo \(plan.title)",patches:plan.patches.map { patch in
        var delta = 0
        let edits = patch.edits.map { edit -> SourceReplacement in
            let from = edit.from + delta; delta += edit.text.utf16.count - (edit.to-edit.from)
            return SourceReplacement(from:from,to:from+edit.text.utf16.count,text:(patch.before as NSString).substring(with:NSRange(location:edit.from,length:edit.to-edit.from)))
        }
        return SourcePatch(path:patch.path,before:patch.after,after:patch.before,edits:edits)
    })
}
func validateChangePlan(_ plan: ChangePlan, sources: [String: SourceSnapshot]) -> String? {
    for patch in plan.patches { guard let source = sources[patch.path], source.writable, exactUTF8(source.text,patch.before) else { return "Source changed, is unavailable, read-only or saving: \(patch.path). Prepare a fresh proposal." } }
    return nil
}
func validateReferencedTargets(_ plan: ChangePlan, referencePaths: [String]) throws {
    let targets = Set(plan.patches.map(\.path)), missing = Set(referencePaths).subtracting(targets).sorted()
    guard missing.isEmpty else { throw EngineError(message:"Rename omitted reported reference source targets: \(missing.joined(separator:", ")). Refresh project analysis and prepare again. No edits were applied.") }
}
