import Foundation
struct CompletionSelection {var from:Int;var to:Int}
struct CompletionExpansion {var text:String;var selection:CompletionSelection?=nil;var fields:[SnippetField]=[]}
struct CompletionDraft {var path:String;var before:String;var plan:ChangePlan;var insertionFrom:Int;var selection:CompletionSelection;var fields:[SnippetField]=[]}
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
        return SnippetField(index:field.index,from:insertionFrom+field.from,to:insertionFrom+field.to,choices:field.choices,id:field.id,parent:field.parent)
    }
    return CompletionDraft(path:path,before:source,plan:plan,insertionFrom:insertionFrom,selection:CompletionSelection(from:insertionFrom+selection.from,to:insertionFrom+selection.to),fields:fields)
}
