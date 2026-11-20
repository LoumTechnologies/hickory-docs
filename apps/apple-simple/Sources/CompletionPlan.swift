import Foundation
struct CompletionSelection {var from:Int;var to:Int}
struct CompletionExpansion {var text:String;var selection:CompletionSelection?=nil;var fields:[SnippetField]=[]}
struct CompletionCaretBatch {var before:[NSRange];var after:[NSRange];var parallel:ParallelSnippet?=nil}
struct CompletionDraft {var path:String;var before:String;var plan:ChangePlan;var insertionFrom:Int;var selection:CompletionSelection;var fields:[SnippetField]=[];var continuing:SnippetSession?=nil;var primary:SourceReplacement?=nil;var additional:[SourceReplacement]=[];var caretBatch:CompletionCaretBatch?=nil}
func completionInsertion(_ item:[String:Any])throws->String {
    guard let label=item["label"] as? String else{throw EngineError(message:"Malformed completion")}
    if let edit=item["textEdit"] {guard let object=edit as? [String:Any],let text=object["newText"] as? String else{throw EngineError(message:"Missing completion insertion text")};return text}
    if let raw=item["insertText"]{guard let text=raw as? String else{throw EngineError(message:"Missing completion insertion text")};return text}
    return label
}
func planCompletion(_ uri:String,item:[String:Any],snapshot:SourceSnapshot,from initialFrom:Int,to initialTo:Int,expansion:CompletionExpansion?=nil)throws->CompletionDraft {
    let source=snapshot.text
    guard snapshot.writable else{throw EngineError(message:"Completion source is generated, read-only or saving")}
    guard let path=SemanticLocation(uri:uri,line:0,character:0).path else{throw EngineError(message:"Unsupported completion source identity")}
    if let command=item["command"],!(command is NSNull){throw EngineError(message:"This completion requires a provider command; no edits were applied.")}
    let format:Int
    if let raw=item["insertTextFormat"]{guard let value=sourceChangeInteger(raw),(value==1 || value==2) else{throw EngineError(message:"Unsupported completion insertion format")};format=value}else{format=1}
    let rawText=try completionInsertion(item)
    guard format != 2 || expansion != nil else{throw EngineError(message:"Snippet expansion unavailable; no edits were applied.")}
    let inserted=format==2 ? expansion! : CompletionExpansion(text:rawText)
    func validateOffset(_ text:String,_ offset:Int)throws {
        guard offset>=0,offset<=text.utf16.count else{throw EngineError(message:"Invalid completion selection")}
        _ = try sourceChangeOffset(text,languagePosition(text,offset))
    }
    try validateOffset(source,initialFrom);try validateOffset(source,initialTo)
    guard initialFrom<=initialTo else{throw EngineError(message:"Completion fallback runs backwards")}
    var from=initialFrom,to=initialTo
    if let edit=item["textEdit"] as? [String:Any] {
        if edit["range"] != nil && (edit["insert"] != nil || edit["replace"] != nil){throw EngineError(message:"Ambiguous completion edit")}
        guard let range=(edit["range"] ?? edit["replace"]) as? [String:Any] else{throw EngineError(message:"Missing completion range")}
        from=try sourceChangeOffset(source,range["start"]);to=try sourceChangeOffset(source,range["end"])
        guard from<=to else{throw EngineError(message:"Completion range runs backwards")}
    }
    var additions:[SourceReplacement]=[]
    if let raw=item["additionalTextEdits"] {
        guard let items=raw as? [[String:Any]] else{throw EngineError(message:"Malformed additional completion edits")}
        for edit in items {
            guard let range=edit["range"] as? [String:Any],let text=edit["newText"] as? String else{throw EngineError(message:"Malformed additional completion edit")}
            let change=try SourceReplacement(from:sourceChangeOffset(source,range["start"]),to:sourceChangeOffset(source,range["end"]),text:text)
            guard change.from<=change.to else{throw EngineError(message:"Additional completion edit runs backwards")}
            if !additions.contains(change){additions.append(change)}
        }
    }
    let prefix=additions.filter{$0.from==from && $0.to==from}.map(\.text).joined()
    let others=additions.filter{$0.from != from || $0.to != from},primary=SourceReplacement(from:from,to:to,text:prefix+inserted.text)
    let edits=([primary]+others).map{edit in ["range":["start":languagePosition(source,edit.from),"end":languagePosition(source,edit.to)],"newText":edit.text] as [String:Any]}
    let plan=try planWorkspaceEdit("Complete \(item["label"] as? String ?? "")",raw:["changes":[uri:edits]],sources:[path:snapshot],allowNoop:true)
    let delta=others.filter{$0.from<from}.reduce(0){$0+$1.text.utf16.count-($1.to-$1.from)},insertionFrom=from+delta+prefix.utf16.count
    let selection=inserted.selection ?? CompletionSelection(from:inserted.text.utf16.count,to:inserted.text.utf16.count)
    try validateOffset(inserted.text,selection.from);try validateOffset(inserted.text,selection.to)
    guard selection.from<=selection.to else{throw EngineError(message:"Completion selection runs backwards")}
    let fields=try inserted.fields.map{field in
        try validateOffset(inserted.text,field.from);try validateOffset(inserted.text,field.to)
        guard field.from<=field.to else{throw EngineError(message:"Invalid snippet field")}
        return SnippetField(index:field.index,from:insertionFrom+field.from,to:insertionFrom+field.to,choices:field.choices,id:field.id,parent:field.parent,transform:field.transform)
    }
    return CompletionDraft(path:path,before:source,plan:plan,insertionFrom:insertionFrom,selection:CompletionSelection(from:insertionFrom+selection.from,to:insertionFrom+selection.to),fields:fields,primary:SourceReplacement(from:from,to:to,text:inserted.text),additional:additions)
}

func validCompletionCarets(_ carets:[NSRange],primary:NSRange,source:String)->Bool {
    let length=source.utf16.count
    func aligned(_ offset:Int)->Bool {
        String.Index(utf16Offset:offset,in:source).samePosition(in:source.unicodeScalars) != nil
    }
    guard (2...16).contains(carets.count),carets.first==primary,carets.allSatisfy({ range in
        (0...length).contains(range.location) && (0...(length-range.location)).contains(range.length)
            && aligned(range.location) && aligned(NSMaxRange(range))
    }) else {return false}
    let ordered=carets.sorted {$0.location<$1.location}
    return zip(ordered,ordered.dropFirst()).allSatisfy {left,right in
        right.location >= NSMaxRange(left) && right.location != left.location
    }
}

func completionGroupEdits(_ drafts:[CompletionDraft],carets:[NSRange],snapshot:SourceSnapshot,snippet:Bool)throws
    -> (primary:[SourceReplacement],shared:[SourceReplacement],ordered:[SourceReplacement]) {
    func refuse(_ message:String)->EngineError {EngineError(message:message)}
    guard let first=drafts.first,let caret=carets.first,drafts.count==carets.count,
          snapshot.writable,validCompletionCarets(carets,primary:caret,source:snapshot.text) else {
        throw refuse("The complete set of writable completion carets is unavailable.")
    }
    let source=snapshot.text
    func ordered(_ edits:[SourceReplacement])->[SourceReplacement] {edits.sorted {($0.from,$0.to)<($1.from,$1.to)}}
    let shared=ordered(first.additional)
    let primary=try zip(drafts,carets).map {draft,range in
        guard exactUTF8(draft.path,first.path),exactUTF8(draft.before,source),
              draft.continuing==nil,ordered(draft.additional)==shared,let edit=draft.primary,
              snippet || exactUTF8(edit.text,first.primary?.text), edit.from>=0,edit.from<=range.location,
              NSMaxRange(range)<=edit.to,edit.to<=source.utf16.count,
              languagePosition(source,edit.from)["line"]==languagePosition(source,range.location)["line"],
              languagePosition(source,edit.to)["line"]==languagePosition(source,range.location)["line"] else {
            throw refuse("The provider did not offer a compatible completion and the same additional edits at every caret.")
        }
        return edit
    }
    let edits=ordered(primary+shared)
    for (left,right) in zip(edits,edits.dropFirst()) where right.from<left.to || right.from==left.from {
        throw refuse("Completion edits overlap across carets or with an additional edit.")
    }
    return (primary,shared,edits)
}

func combineCompletionDrafts(_ drafts:[CompletionDraft],carets:[NSRange],snapshot:SourceSnapshot)throws->CompletionDraft {
    func refuse(_ message:String)->EngineError {EngineError(message:message)}
    guard let first=drafts.first else {throw refuse("The complete set of completion drafts is unavailable.")}
    let snippet = !first.fields.isEmpty
    guard drafts.allSatisfy({(!$0.fields.isEmpty)==snippet}) else {throw refuse("The completion group mixes plain edits with snippets.")}
    let (primary,shared,edits)=try completionGroupEdits(drafts,carets:carets,snapshot:snapshot,snippet:snippet)
    let source=snapshot.text
    let wire=edits.map {
        ["range":["start":languagePosition(source,$0.from),"end":languagePosition(source,$0.to)],"newText":$0.text] as [String:Any]
    }
    let uri=languageURI(first.path)
    let plan=try planWorkspaceEdit(first.plan.title,raw:["changes":[uri:wire]],sources:[first.path:snapshot],allowNoop:true)
    if snippet {
        let afterSource=plan.patches.first?.after ?? source
        func nativeEdit(_ edit:SourceReplacement)->NativeSourceEdit {
            NativeSourceEdit(range:NSRange(location:edit.from,length:edit.to-edit.from),text:edit.text)
        }
        let expansions=zip(drafts,primary).map {draft,edit in
            ParallelSnippetExpansion(edit:nativeEdit(edit),fields:draft.fields.map {field in
                var local=field
                local.from -= draft.insertionFrom;local.to -= draft.insertionFrom
                return local
            })
        }
        guard let prepared=ParallelSnippet.begin(source,expansions:expansions,primary:0,additional:shared.map(nativeEdit)),
              exactUTF8(prepared.state.source,afterSource) else {
            throw refuse("The complete snippet group could not be prepared as one draft edit.")
        }
        let group=prepared.state
        let selections=group.selections
        let selected=selections[group.primaryIndex]
        let after=[selected]+selections.enumerated().compactMap {$0.offset==group.primaryIndex ? nil : $0.element}
        guard validCompletionCarets(after,primary:selected,source:afterSource),
              after.allSatisfy({validSnippetContextSelection(afterSource,$0.location,NSMaxRange($0))}) else {
            throw refuse("Completion could not keep all resulting fields distinct.")
        }
        let session=group.peers.contains {$0.session != nil} ? group : nil
        return CompletionDraft(path:first.path,before:source,plan:plan,insertionFrom:group.peers[group.primaryIndex].range.location,
            selection:CompletionSelection(from:selected.location,to:NSMaxRange(selected)),caretBatch:CompletionCaretBatch(before:carets,after:after,parallel:session))
    }
    var ends:[Int:Int]=[:],delta=0
    for edit in edits {
        let count=edit.text.utf16.count
        ends[edit.from]=edit.from+delta+count
        delta += count-(edit.to-edit.from)
    }
    let after=primary.compactMap {edit in ends[edit.from].map {NSRange(location:$0,length:0)}}
    guard let selected=after.first,after.count==carets.count,Set(after).count==after.count else{
        throw refuse("Completion could not keep all resulting carets distinct.")
    }
    return CompletionDraft(path:first.path,before:source,plan:plan,insertionFrom:first.insertionFrom,
        selection:CompletionSelection(from:selected.location,to:selected.location),caretBatch:CompletionCaretBatch(before:carets,after:after))
}
