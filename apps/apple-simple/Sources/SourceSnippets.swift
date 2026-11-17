import Foundation
import Observation
private func computedSnippetInput(_ session:SnippetSession,_ from:Int,_ to:Int)->Bool {session.fields.contains{field in field.transform != nil && field.from<field.to && field.from<=from && to<=field.to && (from<to || from>field.from && from<field.to)}}
struct SourceSnippetHistoryContext {var fields:[SnippetField];var active:Int;var current:Int;var scopes:[SnippetScope]=[]}
func snippetHistoryCost(_ value:SourceSnippetHistoryContext?)->Int {
    guard let value else{return 0};let limit=32*1024*1024;var bytes=0
    for field in value.fields {
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
    private(set) var notice=""
    @ObservationIgnored var commit:((SnippetInput,SnippetSession)->Bool)?
    init(buffer:DocumentBuffer){self.buffer=buffer}
    var active:SnippetSession? {guard let buffer,let session,exactUTF8(buffer.text,session.source) else{return nil};return session}
    var choices:[String] {guard let active else{return []};return active.fields.first(where:{$0.id==active.current})?.choices ?? []}
    var historyContext:SourceSnippetHistoryContext? {active.map{SourceSnippetHistoryContext(fields:$0.fields,active:$0.active,current:$0.current,scopes:$0.scopes)}}
    func restoreHistory(_ context:SourceSnippetHistoryContext?){
        guard let context,let buffer,beginSnippet(buffer.text,fields:context.fields) != nil else{restore(nil);return}
        restore(SnippetSession(source:buffer.text,fields:context.fields,active:context.active,current:context.current,scopes:context.scopes))
    }
    func restore(_ value:SnippetSession?) {session=value;intent+=1;report()}
    func sourceChanged(_ source:String) {if let session,!exactUTF8(source,session.source){finish()}}
    func reportFailure(_ message:String){notice=message;buffer?.language?.completionNotice=message}
    private func report(){notice=session.map{"Template field \($0.active) · Tab next · Shift-Tab previous · Esc finish"} ?? "Template editing finished · draft";buffer?.language?.completionNotice=notice}
    @discardableResult func finish()->Bool {guard session != nil else{return false};session=nil;intent+=1;report();return true}
    @discardableResult func move(_ direction:Int)->Bool {
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
        guard let current=active else{return draft}
        var result=draft
        guard let edit=draft.primary else{throw EngineError(message:"Completion primary edit is unavailable.")}
        let inner=draft.fields.map{field in var value=field;value.from-=draft.insertionFrom;value.to-=draft.insertionFrom;return value}
        guard let input=inner.isEmpty ? completeSnippet(current,source:draft.before,primary:edit,additional:draft.additional):nestSnippet(current,source:draft.before,primary:edit,additional:draft.additional,inner:inner) else{throw EngineError(message:"Completion overlaps template fields or is unavailable. Finish the template and retry.")}
        result.plan=ChangePlan(title:draft.plan.title,patches:exactUTF8(input.session.source,draft.before) ? []:[SourcePatch(path:draft.path,before:draft.before,after:input.session.source,edits:input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)})]);result.selection=input.selection;result.insertionFrom=input.selection.from-edit.text.utf16.count;result.continuing=input.session;return result
    }
    @discardableResult func choose(_ value:String)->Bool {
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
