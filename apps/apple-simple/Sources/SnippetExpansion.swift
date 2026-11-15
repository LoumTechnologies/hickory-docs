import Foundation
struct SnippetField {var index:Int;var from:Int;var to:Int;var choices:[String]}
func completionExpansion(_ item:[String:Any])throws->CompletionExpansion? {
    guard sourceChangeInteger(item["insertTextFormat"])==2 else{return nil}
    guard let raw=(item["data"] as? [String:Any])?["hickory_snippet"] as? [String:Any],sourceChangeInteger(raw["version"])==1,let syntax=raw["syntax"] as? String,exactUTF8(syntax,try completionInsertion(item)),let text=raw["text"] as? String,text.utf16.count<=1048576,let values=raw["fields"] as? [[String:Any]],!values.isEmpty,values.count<=4096 else{throw EngineError(message:"Snippet expansion unavailable or out of date. Request completions again.")}
    var fields:[SnippetField]=[]
    for value in values {
        guard let index=sourceChangeInteger(value["index"]),index>=0,index<=1004096,let from=sourceChangeInteger(value["from"]),let to=sourceChangeInteger(value["to"]),from>=0,from<=to,to<=text.utf16.count,let choices=value["choices"] as? [String] else{throw EngineError(message:"Malformed snippet field")}
        _ = try sourceChangeOffset(text,languagePosition(text,from));_ = try sourceChangeOffset(text,languagePosition(text,to))
        fields.append(SnippetField(index:index,from:from,to:to,choices:choices))
    }
    guard fields.filter({$0.index==0}).count==1 else{throw EngineError(message:"Malformed snippet exit")}
    var first:[Int:SnippetField]=[:];let source=text as NSString
    for field in fields {
        if let previous=first[field.index] {
            let before=source.substring(with:NSRange(location:previous.from,length:previous.to-previous.from)),after=source.substring(with:NSRange(location:field.from,length:field.to-field.from))
            guard exactUTF8(before,after),previous.choices.count==field.choices.count,zip(previous.choices,field.choices).allSatisfy({exactUTF8($0.0,$0.1)}) else{throw EngineError(message:"Inconsistent snippet mirrors")}
        }
        first[field.index]=field
    }
    let active=first.keys.filter{$0>0}.min() ?? 0,selected=fields.first{$0.index==active}!
    return CompletionExpansion(text:text,selection:CompletionSelection(from:selected.from,to:selected.to),fields:fields)
}
