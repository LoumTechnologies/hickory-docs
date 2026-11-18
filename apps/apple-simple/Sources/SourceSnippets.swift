import Foundation
import Observation
private func computedSnippetInput(_ session:SnippetSession,_ from:Int,_ to:Int)->Bool {session.fields.contains{field in field.transform != nil && field.from<field.to && field.from<=from && to<=field.to && (from<to || from>field.from && from<field.to)}}
struct SourceSnippetHistoryContext {var fields:[SnippetField];var active:Int;var current:Int;var scopes:[SnippetScope]=[];var parallel:ParallelSnippet?=nil;var ranges:[NSRange]=[]}
func snippetHistoryCost(_ value:SourceSnippetHistoryContext?)->Int {
    guard let value else{return 0}
    let limit=32*1024*1024
    let peers=value.parallel?.peers ?? []
    var bytes=value.ranges.count*32 + (value.parallel?.source.utf8.count ?? 0)
    for peer in peers {bytes+=128+peer.text.utf8.count}
    for field in value.fields+peers.flatMap({$0.session?.fields ?? []}) {
        guard bytes<=limit-64 else{return limit+1};bytes+=64
        if let transform=field.transform {guard transform.bytes<=limit,bytes<=limit-transform.bytes else{return limit+1};bytes+=transform.bytes}
        for choice in field.choices {
            let count=choice.utf8.count
            guard count<=limit-32,bytes<=limit-32-count else{return limit+1};bytes+=32+count
        }
    }
    return bytes
}
@MainActor @Observable final class SourceSnippetController {
    private weak var buffer:DocumentBuffer?
    private(set) var intent=0
    private(set) var session:SnippetSession?
    private var group:ParallelSnippet?
    private(set) var notice=""
    @ObservationIgnored var commit:((SnippetInput,SnippetSession)->Bool)?
    @ObservationIgnored var commitParallel:((ParallelSnippetChange,String)->Bool)?
    init(buffer:DocumentBuffer){self.buffer=buffer}
    var parallel: ParallelSnippet? {
        guard let buffer, let group, exactUTF8(buffer.text, group.source) else { return nil }
        return group
    }

    var isActive: Bool { active != nil || parallel != nil }

    private func orderedRanges(_ session: ParallelSnippet) -> [NSRange] {
        let ranges = session.selections
        let primary = session.primaryIndex
        return [ranges[primary]] + ranges.enumerated().compactMap { $0.offset == primary ? nil : $0.element }
    }

    private func reflectSelections(_ ranges: [NSRange], source: String) -> Bool {
        guard let buffer, let first = ranges.first, let select = buffer.selectSourceRange else { return false }
        let previous = buffer.applyingMulti
        buffer.applyingMulti = true
        defer { buffer.applyingMulti = previous }
        guard select(first, source) else { return false }
        if ranges.count > 1 {
            buffer.multiSource = source
            buffer.multiRanges = ranges
        } else {
            buffer.clearMulti()
        }
        return true
    }

    // A provider preview captures a document intent before requesting any
    // plans. Only the original selections and source may enter source history.
    func applyParallel(
        source: String, expansions: [ParallelSnippetExpansion], primary: Int,
        expectedIntent: Int, admitted: () -> Bool
    ) -> Bool {
        guard let buffer, expectedIntent == intent, admitted(), !buffer.readOnly, !buffer.saving,
              exactUTF8(buffer.text, source), exactUTF8(buffer.multiSource, source),
              buffer.multiRanges.count == expansions.count,
              expansions.indices.contains(primary),
              expansions[primary].edit.range == buffer.selection,
              Set(expansions.map { $0.edit.range }) == Set(buffer.multiRanges),
              let prepared = ParallelSnippet.begin(source, expansions: expansions, primary: primary),
              expectedIntent == intent, admitted() else { return false }
        return commitParallel?(prepared, source) == true
    }

    // Native typing produces one edit for the primary selection. Linked
    // replacements use each peer's local field, including distinct grapheme
    // widths when deleting at an empty caret.
    func inputParallel(source: String, primaryRange: NSRange, text: String) -> Bool {
        guard let buffer, let parallel, !buffer.readOnly, !buffer.saving,
              exactUTF8(source, parallel.source), buffer.multiRanges == orderedRanges(parallel),
              let primary = buffer.multiRanges.first, buffer.selection == primary else { return false }
        let original = source as NSString
        let ranges = parallel.selections
        let edited: [NSRange]
        if primaryRange == primary {
            edited = ranges
        } else if text.isEmpty, primary.length == 0, NSMaxRange(primaryRange) == primary.location {
            edited = ranges.map { $0.length == 0 && $0.location > 0 ?
                original.rangeOfComposedCharacterSequence(at: $0.location - 1) : $0 }
        } else if text.isEmpty, primary.length == 0, primaryRange.location == primary.location {
            edited = ranges.map { $0.length == 0 && $0.location < original.length ?
                original.rangeOfComposedCharacterSequence(at: $0.location) : $0 }
        } else {
            return false
        }
        guard edited[parallel.primaryIndex] == primaryRange,
              let result = parallel.editing(source, input: edited.map { NativeSourceEdit(range: $0, text: text) }) else { return false }
        return commitParallel?(result, source) == true
    }
    var active:SnippetSession? {guard let buffer,let session,exactUTF8(buffer.text,session.source) else{return nil};return session}
    var choices:[String] {
        if let parallel {
            let fields=parallel.peers.compactMap{peer in peer.session.flatMap{live in live.fields.first{$0.id==live.current&&$0.index>0}}}
            guard fields.count==parallel.peers.count,let first=fields.first else{return []}
            return first.choices.filter{value in fields.allSatisfy{field in field.choices.contains{exactUTF8($0,value)}}}
        }
        return active.flatMap{live in live.fields.first{$0.id==live.current}}?.choices ?? []
    }
    var historyContext:SourceSnippetHistoryContext? {
        let ranges=buffer.flatMap{exactUTF8($0.text,$0.multiSource)&&$0.multiRanges.count>1 ? $0.multiRanges:nil} ?? []
        if let parallel{return SourceSnippetHistoryContext(fields:[],active:0,current:0,parallel:parallel,ranges:ranges)}
        return active.map{SourceSnippetHistoryContext(fields:$0.fields,active:$0.active,current:$0.current,scopes:$0.scopes)}
            ?? (ranges.isEmpty ? nil:SourceSnippetHistoryContext(fields:[],active:0,current:0,ranges:ranges))
    }
    func restoreHistory(_ context:SourceSnippetHistoryContext?){
        guard let context,let buffer else{restore(nil);return}
        if let parallel=context.parallel,exactUTF8(parallel.source,buffer.text),
           reflectSelections(orderedRanges(parallel),source:buffer.text) {
            session=nil;group=parallel;intent+=1;report();return
        }
        if !context.ranges.isEmpty {_ = reflectSelections(context.ranges,source:buffer.text)}
        guard beginSnippet(buffer.text,fields:context.fields) != nil else{restore(nil);return}
        restore(SnippetSession(source:buffer.text,fields:context.fields,active:context.active,current:context.current,scopes:context.scopes))
    }
    func restore(_ value:SnippetSession?) {session=value;group=nil;intent+=1;report()}
    func sourceChanged(_ source:String) {
        if let expected=group?.source ?? session?.source,!exactUTF8(source,expected){finish()}
    }
    func selectionChanged(_ selected:NSRange) {
        if let buffer,!buffer.applyingMulti,let parallel,orderedRanges(parallel).first != selected {_ = finish()}
    }
    func reportFailure(_ message:String){notice=message;buffer?.language?.completionNotice=message}
    private func report(){
        notice=group.map{"\($0.peers.count) template carets · Tab next · Shift-Tab previous · Esc finish"}
            ?? session.map{"Template field \($0.active) · Tab next · Shift-Tab previous · Esc finish"}
            ?? "Template editing finished · draft"
        buffer?.language?.completionNotice=notice
    }
    @discardableResult func finish()->Bool {guard session != nil || group != nil else{return false};restore(nil);return true}
    @discardableResult func move(_ direction:Int)->Bool {
        if let parallel,let buffer {
            guard !buffer.readOnly,!buffer.saving,buffer.multiRanges==orderedRanges(parallel),
                  let next=parallel.moving(buffer.text,direction:direction) else{_ = finish();return false}
            if direction==1 && parallel.peers.allSatisfy({($0.session?.active ?? 0)==0}){_ = finish();return false}
            guard reflectSelections(orderedRanges(next),source:next.source) else{return false}
            group=next;intent+=1;buffer.focusSource?();report();return true
        }
        guard let buffer,let current=active else{_ = finish();return false}
        guard let next=advanceSnippet(current,source:buffer.text,from:buffer.selection.location,to:NSMaxRange(buffer.selection),direction:direction) else{if direction == -1,current.active==(current.scopes.last?.order.first ?? current.fields.filter({$0.transform==nil}).map({$0.index}).filter({$0>0}).min()){return true};_ = finish();return false}
        let old=session;session=next.session
        guard buffer.selectSourceRange?(NSRange(location:next.selection.from,length:next.selection.to-next.selection.from),current.source)==true else{session=old;return false}
        intent+=1;buffer.focusSource?();report();return true
    }
    // nil leaves ordinary native input in charge; true means one linked batch committed.
    func input(source:String,range:NSRange,text:String,caret:Int?=nil)->Bool? {
        guard let buffer,let current=active,!buffer.readOnly,!buffer.saving,exactUTF8(source,buffer.text),buffer.multiRanges.count<2,!computedSnippetInput(current,range.location,NSMaxRange(range)),var input=editSnippet(current,source:source,from:range.location,to:NSMaxRange(range),text:text),let commit else{_ = finish();return nil}
        if let caret {
            guard caret>=range.location,caret<=range.location+text.utf16.count else{_ = finish();return nil}
            let selected=input.selection.from+caret-(range.location+text.utf16.count);input.selection=CompletionSelection(from:selected,to:selected)
        }
        guard commit(input,current) else{_ = finish();return nil};return true
    }
    func prepareCompletion(_ draft:CompletionDraft)throws->CompletionDraft {
        if parallel != nil {throw EngineError(message:"Finish editing the multiple template fields before accepting a completion.")}
        guard let current=active else{return draft}
        var result=draft
        guard let edit=draft.primary else{throw EngineError(message:"Completion primary edit is unavailable.")}
        let inner=draft.fields.map{field in var value=field;value.from-=draft.insertionFrom;value.to-=draft.insertionFrom;return value}
        guard let input=inner.isEmpty ? completeSnippet(current,source:draft.before,primary:edit,additional:draft.additional):nestSnippet(current,source:draft.before,primary:edit,additional:draft.additional,inner:inner) else{throw EngineError(message:"Completion overlaps template fields or is unavailable. Finish the template and retry.")}
        result.plan=ChangePlan(title:draft.plan.title,patches:exactUTF8(input.session.source,draft.before) ? []:[SourcePatch(path:draft.path,before:draft.before,after:input.session.source,edits:input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)})]);result.selection=input.selection;result.insertionFrom=input.selection.from-edit.text.utf16.count;result.continuing=input.session;return result
    }
    @discardableResult func choose(_ value:String)->Bool {
        if let parallel,let buffer {
            guard !buffer.readOnly,!buffer.saving,buffer.multiRanges==orderedRanges(parallel),
                  choices.contains(where:{exactUTF8($0,value)}) else{return false}
            let edits=parallel.peers.compactMap {peer -> NativeSourceEdit? in
                guard let live=peer.session,let field=live.fields.first(where:{$0.id==live.current}) else{return nil}
                return NativeSourceEdit(range:NSRange(location:peer.range.location+field.from,length:field.to-field.from),text:value)
            }
            guard let next=parallel.editing(buffer.text,input:edits) else{return false}
            return commitParallel?(next,buffer.text)==true
        }
        guard let buffer,let current=active,!buffer.readOnly,!buffer.saving,let field=current.fields.first(where:{$0.id==current.current}),field.choices.contains(where:{exactUTF8($0,value)}) else{return false}
        return input(source:buffer.text,range:NSRange(location:field.from,length:field.to-field.from),text:value)==true
    }
}
@MainActor func snippetChangePlan(path:String,before:String,input:SnippetInput)->ChangePlan? {
    guard validSourceEdits(input.edits,length:before.utf16.count) else{return nil}
    let value=NSMutableString(string:before)
    for edit in input.edits.sorted(by:{$0.range.location>$1.range.location}){value.replaceCharacters(in:edit.range,with:edit.text)}
    guard exactUTF8(value as String,input.session.source) else{return nil}
    return ChangePlan(title:"Edit template field",patches:[SourcePatch(path:path,before:before,after:input.session.source,edits:input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)})])
}

private enum MarkedFieldGroup {
    case single(SnippetSession)
    case parallel(ParallelSnippet)
}

struct MarkedSnippetResult {
    let source: String
    let edits: [NativeSourceEdit]
    let selection: NSRange
    let context: SourceSnippetHistoryContext?
}

@MainActor struct CapturedMarkedSnippet {
    let path: String
    let source: String
    let selection: NSRange
    let history: SourceSnippetHistoryContext?
    private let originalRange: NSRange
    private let group: MarkedFieldGroup
    private let intent: Int

    init?(buffer: DocumentBuffer, source: String, selection: NSRange, range: NSRange) {
        guard !buffer.readOnly, !buffer.saving, exactUTF8(buffer.text, source),
              range.location >= 0, range.length >= 0, range.location <= source.utf16.count,
              range.length <= source.utf16.count - range.location,
              source.utf8.count <= 8 * 1024 * 1024 else { return nil }
        if let parallel = buffer.snippets.parallel {
            guard parallel.selections[parallel.primaryIndex] == range else { return nil }
            group = .parallel(parallel)
        } else if let active = buffer.snippets.active {
            guard active.fields.contains(where: {
                $0.id == active.current && $0.transform == nil &&
                $0.from <= range.location && NSMaxRange(range) <= $0.to
            }), !computedSnippetInput(active, range.location, NSMaxRange(range)) else { return nil }
            group = .single(active)
        } else { return nil }

        self.path = buffer.file.path
        self.source = source
        self.selection = selection
        self.originalRange = range
        self.history = buffer.snippets.historyContext
        // Earlier completion and choice offers cannot survive marked input.
        _ = buffer.snippets.finish()
        buffer.clearMulti()
        self.intent = buffer.snippets.intent
    }

    func finish(buffer: DocumentBuffer, observed: String, selected: NSRange) -> MarkedSnippetResult? {
        guard exactUTF8(path, buffer.file.path), exactUTF8(observed, buffer.text),
              !buffer.readOnly, !buffer.saving, buffer.snippets.intent == intent else { return nil }
        let raw = source as NSString
        let prefix = raw.substring(to: originalRange.location)
        let suffix = raw.substring(from: NSMaxRange(originalRange))
        guard let inserted = markedSourceReplacement(observed, prefix: prefix, suffix: suffix),
              inserted.utf8.count <= 1024 * 1024, selected.location >= originalRange.location,
              selected.length >= 0, NSMaxRange(selected) <= originalRange.location + inserted.utf16.count else { return nil }
        let offset = selected.location - originalRange.location - inserted.utf16.count
        switch group {
        case .single(let current):
            guard let plan = editSnippet(current, source: source, from: originalRange.location,
                                         to: NSMaxRange(originalRange), text: inserted) else { return nil }
            let target = NSRange(location: plan.selection.from + offset, length: selected.length)
            let session = plan.session
            return MarkedSnippetResult(source: session.source, edits: plan.edits, selection: target,
                context: SourceSnippetHistoryContext(fields: session.fields, active: session.active,
                                                     current: session.current, scopes: session.scopes))
        case .parallel(let current):
            let edits = current.selections.map { NativeSourceEdit(range: $0, text: inserted) }
            guard let plan = current.editing(source, input: edits) else { return nil }
            let primary = plan.state.primaryIndex
            var ranges = plan.state.selections
            ranges[primary].location += offset
            ranges[primary].length = selected.length
            guard let next = plan.state.selecting(plan.state.source, ranges: ranges, primary: primary) else { return nil }
            return MarkedSnippetResult(source: next.source, edits: plan.edits, selection: ranges[primary],
                context: SourceSnippetHistoryContext(fields: [], active: 0, current: 0, parallel: next))
        }
    }
}

private func markedSourceReplacement(_ observed: String, prefix: String, suffix: String) -> String? {
    let text = observed as NSString
    let leading = prefix.utf16.count
    let trailing = suffix.utf16.count
    guard text.length >= leading + trailing,
          exactUTF8(text.substring(to: leading), prefix),
          exactUTF8(text.substring(from: text.length - trailing), suffix) else { return nil }
    return text.substring(with: NSRange(location: leading, length: text.length - leading - trailing))
}

// Both sides are already observed source revisions. Use a single minimal
// exact replacement for native history or for reflecting the mirror delta.
func markedSourceEdit(from before: String, to after: String) -> NativeSourceEdit? {
    if exactUTF8(before, after) { return nil }
    let old = before as NSString
    let new = after as NSString
    let shortest = min(old.length, new.length)
    func boundary(_ value: NSString, at: Int) -> Bool {
        guard at > 0, at < value.length else { return true }
        let left = value.character(at: at - 1)
        let right = value.character(at: at)
        return !(left == 13 && right == 10 ||
                 (0xd800...0xdbff).contains(left) && (0xdc00...0xdfff).contains(right))
    }
    var start = 0
    while start < shortest && old.character(at: start) == new.character(at: start) { start += 1 }
    while !boundary(old, at: start) || !boundary(new, at: start) { start -= 1 }
    var suffix = 0
    while suffix < shortest - start &&
          old.character(at: old.length - suffix - 1) == new.character(at: new.length - suffix - 1) {
        suffix += 1
    }
    while !boundary(old, at: old.length - suffix) || !boundary(new, at: new.length - suffix) { suffix -= 1 }
    return NativeSourceEdit(range: NSRange(location: start, length: old.length - start - suffix),
                            text: new.substring(with: NSRange(location: start, length: new.length - start - suffix)))
}

@MainActor func completeMarkedSnippet(
    _ capture: CapturedMarkedSnippet, buffer: DocumentBuffer, observed: String, selected: NSRange,
    history: SourceDraftHistory, manager: UndoManager?, admitted: Bool,
    apply: @escaping SourceDraftHistory.Apply
) {
    guard exactUTF8(capture.path, buffer.file.path), exactUTF8(observed, buffer.text) else { return }
    var finalSource = observed
    var finalSelection = selected
    var finalEdits: [NativeSourceEdit]?
    var context: SourceSnippetHistoryContext?
    if admitted, let next = capture.finish(buffer: buffer, observed: observed, selected: selected) {
        let delta = markedSourceEdit(from: observed, to: next.source)
        manager?.disableUndoRegistration()
        let reflected = apply(delta.map { [$0] } ?? [], observed, next.selection)
        manager?.enableUndoRegistration()
        if reflected, exactUTF8(buffer.text, next.source) {
            finalSource = next.source
            finalSelection = next.selection
            finalEdits = next.edits
            context = next.context
        }
    }
    guard exactUTF8(buffer.text, finalSource) else { return }
    let edits = finalEdits ?? markedSourceEdit(from: capture.source, to: finalSource).map { [$0] } ?? []
    let patches = exactUTF8(capture.source, finalSource) ? [] : [
        SourcePatch(path: capture.path, before: capture.source, after: finalSource,
                    edits: edits.map { SourceReplacement(from: $0.range.location, to: NSMaxRange($0.range), text: $0.text) })
    ]
    let plan = ChangePlan(title: "Compose template field", patches: patches)
    _ = history.commit(plan, path: capture.path, before: capture.source,
        selectionBefore: capture.selection, selectionAfter: finalSelection,
        contextBefore: capture.history, contextAfter: context, manager: manager,
        read: { [weak buffer] in buffer?.text ?? "" }, apply: apply,
        notice: { [weak buffer] message in buffer?.snippets.reportFailure(message) },
        restoreContext: { [weak buffer] context in buffer?.snippets.restoreHistory(context) },
        contextCost: snippetHistoryCost,
        discardContext: { [weak buffer] in _ = buffer?.snippets.finish() },
        alreadyObserved: true)
}
