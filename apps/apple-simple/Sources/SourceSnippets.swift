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
