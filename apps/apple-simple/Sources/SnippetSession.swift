import Foundation
struct SnippetSession {var source:String;var fields:[SnippetField];var active:Int;var current:Int}
struct SnippetInput {var session:SnippetSession;var edits:[NativeSourceEdit];var selection:CompletionSelection}
struct SnippetAdvance {var session:SnippetSession?;var selection:CompletionSelection}
private func snippetScalar(_ text:NSString,_ at:Int)->Bool {
    guard at>=0,at<=text.length else{return false}
    if at>0,at<text.length{let before=text.character(at:at-1),after=text.character(at:at);if (before>=0xd800&&before<=0xdbff&&after>=0xdc00&&after<=0xdfff)||(before==13&&after==10){return false}}
    return true
}
func beginSnippet(_ source:String,fields:[SnippetField])->SnippetSession? {
    let text=source as NSString
    guard !fields.isEmpty,fields.count<=4096,fields.allSatisfy({field in field.id != nil && field.id!>=0 && field.id!<4096 && field.index>=0 && field.index<=1004096 && field.from<=field.to && snippetScalar(text,field.from) && snippetScalar(text,field.to)}),fields.filter({$0.index==0}).count==1,!fields.contains(where:{$0.index==0&&$0.from != $0.to}) else{return nil}
    var ids:[Int:SnippetField]=[:]
    for field in fields{guard ids[field.id!]==nil else{return nil};ids[field.id!]=field}
    var visited:Set<Int>=[]
    for field in fields{var chain:Set<Int>=[],next:SnippetField?=field
        while let value=next,!visited.contains(value.id!){guard chain.insert(value.id!).inserted else{return nil};guard let identity=value.parent else{break};guard let parent=ids[identity],parent.from<=value.from,parent.to>=value.to else{return nil};next=parent};visited.formUnion(chain)
    }
    guard let active=fields.map({$0.index}).filter({$0>0}).min(),let first=fields.first(where:{$0.index==active}) else{return nil}
    return SnippetSession(source:source,fields:fields,active:active,current:first.id!)
}
func advanceSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,direction:Int)->SnippetAdvance? {
    guard exactUTF8(source,session.source),from<=to,(direction==1||direction == -1),session.fields.contains(where:{$0.index==session.active&&$0.from<=from&&$0.to>=to}) else{return nil}
    let order=Array(Set(session.fields.map{$0.index}.filter{$0>0})).sorted()+[0]
    guard let at=order.firstIndex(of:session.active),order.indices.contains(at+direction),let field=session.fields.first(where:{$0.index==order[at+direction]}) else{return nil}
    var next=session;next.active=field.index;next.current=field.id!
    return SnippetAdvance(session:field.index==0 ? nil:next,selection:CompletionSelection(from:field.from,to:field.to))
}
private struct SnippetReplacement {var from:Int;var to:Int;var text:String;var owners:[Int]}
func editSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,text:String)->SnippetInput? {
    let original=source as NSString
    guard exactUTF8(source,session.source),from<=to,snippetScalar(original,from),snippetScalar(original,to),session.active != 0 else{return nil}
    let fields=session.fields,ids=Dictionary(uniqueKeysWithValues:fields.map{($0.id!,$0)})
    func ancestor(_ parent:Int,_ child:Int)->Bool{var current=ids[child];for _ in 0...fields.count{guard let value=current else{return false};if value.id==parent{return true};current=value.parent.flatMap{ids[$0]}};return false}
    let occurrences=fields.filter{$0.index==session.active},matches=occurrences.filter{$0.from<=from&&$0.to>=to}
    guard let origin=matches.first(where:{$0.id==session.current}) ?? matches.first,text.utf8.count<=1048576/max(occurrences.count,1) else{return nil}
    let before=original.substring(with:NSRange(location:origin.from,length:origin.to-origin.from)),relativeFrom=from-origin.from,relativeTo=to-origin.from
    guard occurrences.allSatisfy({exactUTF8(original.substring(with:NSRange(location:$0.from,length:$0.to-$0.from)),before)}) else{return nil}
    var edits:[SnippetReplacement]=[]
    for field in occurrences{let from=field.from+relativeFrom,to=field.from+relativeTo;guard snippetScalar(original,from),snippetScalar(original,to) else{return nil}
        if let existing=edits.firstIndex(where:{$0.from==from&&$0.to==to}){guard from==to else{return nil};edits[existing].text+=text;edits[existing].owners.append(field.id!)}else{edits.append(SnippetReplacement(from:from,to:to,text:text,owners:[field.id!]))}
    }
    edits.sort{($0.from,$0.to)<($1.from,$1.to)}
    for index in edits.indices where index>0{guard edits[index-1].to<=edits[index].from else{return nil}}
    let result=NSMutableString(string:source);for edit in edits.reversed(){result.replaceCharacters(in:NSRange(location:edit.from,length:edit.to-edit.from),with:edit.text)};let after=result as String
    let native=edits.map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)}
    if exactUTF8(after,source){var next=session;next.current=origin.id!;return SnippetInput(session:next,edits:native,selection:CompletionSelection(from:from+text.utf16.count,to:from+text.utf16.count))}
    let ordinals=Dictionary(uniqueKeysWithValues:fields.enumerated().map{($0.element.id!,$0.offset)})
    func map(_ field:SnippetField,_ position:Int,_ end:Bool)->Int {
        var delta=0
        for edit in edits{
            if position<edit.from{break};if position>edit.to{delta+=edit.text.utf16.count-(edit.to-edit.from);continue}
            if edit.from==edit.to{
                let related=edit.owners.indices.filter{ancestor(field.id!,edit.owners[$0])}
                if let first=related.first,let last=related.last{return edit.from+delta+text.utf16.count*(end ? last+1:first)}
                let enclosing=edit.owners.indices.filter{field.index==0&&ancestor(edit.owners[$0],field.id!)}
                if let last=enclosing.last{return edit.from+delta+text.utf16.count*(last+1)}
                return edit.from+delta+text.utf16.count*edit.owners.filter{ordinals[$0]!<ordinals[field.id!]!}.count
            }
            let owns=edit.owners.contains(field.id!)||edit.owners.contains(where:{ancestor(field.id!,$0)}),exit=field.index==0&&edit.owners.contains(where:{ancestor($0,field.id!)})
            let right=owns ? end:exit ? true:position==edit.to
            return edit.from+delta+(right ? edit.text.utf16.count:0)
        }
        return position+delta
    }
    let retained=fields.filter{field in field.index==0||field.index==session.active || !occurrences.contains(where:{ancestor($0.id!,field.id!)})},retainedIds=Set(retained.map{$0.id!})
    let updated=retained.map{field in var value=field;var parent=field.parent;while let identity=parent,!retainedIds.contains(identity){parent=ids[identity]?.parent};value.parent=parent;value.from=map(field,field.from,false);value.to=map(field,field.to,true);return value}
    let output=after as NSString;var groups:[Int:String]=[:]
    for field in updated{guard field.from<=field.to,snippetScalar(output,field.from),snippetScalar(output,field.to) else{return nil};let value=output.substring(with:NSRange(location:field.from,length:field.to-field.from));if let previous=groups[field.index],!exactUTF8(previous,value){return nil};groups[field.index]=value}
    let caret=map(origin,from,false)+text.utf16.count
    return SnippetInput(session:SnippetSession(source:after,fields:updated,active:session.active,current:origin.id!),edits:native,selection:CompletionSelection(from:caret,to:caret))
}
