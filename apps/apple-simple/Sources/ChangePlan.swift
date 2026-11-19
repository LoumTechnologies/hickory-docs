import Foundation
import CoreFoundation
struct SourceSnapshot { var text: String; var writable: Bool }
struct SourceReplacement: Equatable { var from: Int; var to: Int; var text: String
    static func ==(lhs:Self,rhs:Self)->Bool {lhs.from==rhs.from&&lhs.to==rhs.to&&exactUTF8(lhs.text,rhs.text)}
}
func reviewedSourceReplacement(_ before: String, _ after: String) -> SourceReplacement? {
    let old = Array(before.utf16), new = Array(after.utf16)
    let sharedLength = min(old.count, new.count)
    var from = 0
    while from < sharedLength && old[from] == new[from] { from += 1 }
    if from == old.count && from == new.count { return nil }
    func split(_ units: [UInt16], _ at: Int) -> Bool {
        guard at > 0 && at < units.count else { return false }
        let left = units[at - 1], right = units[at]
        return (0xD800...0xDBFF).contains(left) && (0xDC00...0xDFFF).contains(right)
            || (left == 13 && right == 10)
    }
    if split(old, from) || split(new, from) { from -= 1 }
    var suffix = 0
    while suffix < sharedLength - from && old[old.count - suffix - 1] == new[new.count - suffix - 1] { suffix += 1 }
    if split(old, old.count - suffix) || split(new, new.count - suffix) { suffix -= 1 }
    return SourceReplacement(from: from, to: old.count - suffix,
        text: String(decoding: new[from..<(new.count - suffix)], as: UTF16.self))
}
extension SourceReplacement {
    func reviewedSelection(_ previous: NSRange) -> NSRange {
        let end = from + text.utf16.count
        func map(_ offset: Int, lower: Bool) -> Int {
            if offset == from && from == to {
                return lower || previous.length == 0 ? end : from
            }
            if offset <= from { return offset }
            if offset >= to { return offset + end - to }
            return lower && previous.length > 0 ? from : end
        }
        let start = map(previous.location, lower: true)
        return NSRange(location: start, length: max(0, map(NSMaxRange(previous), lower: false) - start))
    }
}
struct SourcePatch { var path: String; var before: String; var after: String; var edits: [SourceReplacement] }
struct ChangePlan { var title: String; var patches: [SourcePatch] }
func sourceChangeInteger(_ raw:Any?)->Int? {
    guard let number=raw as? NSNumber,CFGetTypeID(number) != CFBooleanGetTypeID(),
          let value=Int(exactly:number.doubleValue),number.compare(NSNumber(value:value)) == .orderedSame else{return nil}
    return value
}
func sourceChangeOffset(_ source: String, _ raw: Any?) throws -> Int {
    guard let point = raw as? [String: Any], let line = sourceChangeInteger(point["line"]), let character = sourceChangeInteger(point["character"]),
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
func planWorkspaceEdit(_ title: String, raw: Any, sources: NativeSourceMap<SourceSnapshot>,allowNoop:Bool=false) throws -> ChangePlan {
    var groups: NativeSourceMap<[SourceReplacement]> = [:]
    for target in try workspaceEditTargets(raw) {
        let path=target.path,items=target.items
        guard let source=sources[path] else{throw EngineError(message:"Acquire the affected source before preparing this proposal: \(path)")}
        guard source.writable else { throw EngineError(message: "Source is generated, read-only or saving: \(path)") }
        var replacements = groups[path] ?? []
        for item in items {
            guard let range = item["range"] as? [String: Any], let text = item["newText"] as? String else { throw EngineError(message: "Malformed source replacement") }
            let replacement = try SourceReplacement(from: sourceChangeOffset(source.text, range["start"]), to: sourceChangeOffset(source.text, range["end"]), text: text)
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
    guard !patches.isEmpty || allowNoop else { throw EngineError(message: "The proposal contains no source changes") }
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
func validateChangePlan(_ plan: ChangePlan, sources: NativeSourceMap<SourceSnapshot>) -> String? {
    for patch in plan.patches { guard let source = sources[patch.path], source.writable, exactUTF8(source.text,patch.before) else { return "Source changed, is unavailable, read-only or saving: \(patch.path). Prepare a fresh proposal." } }
    return nil
}
func validateReferencedTargets(_ plan: ChangePlan, referencePaths: [String]) throws {
    let targets = Set(plan.patches.map{NativeSourceKey($0.path)}), missing = Set(referencePaths.map(NativeSourceKey.init)).subtracting(targets).map(\.string).sorted()
    guard missing.isEmpty else { throw EngineError(message:"Rename omitted reported reference source targets: \(missing.joined(separator:", ")). Refresh project analysis and prepare again. No edits were applied.") }
}
