import Foundation
struct SnippetScope {var order:[Int];var exitIndex:Int;var exitId:Int;var outerActive:Int;var outerCurrent:Int}
struct SnippetSession {var source:String;var fields:[SnippetField];var active:Int;var current:Int;var scopes:[SnippetScope]=[]}
struct SnippetInput {var session:SnippetSession;var edits:[NativeSourceEdit];var selection:CompletionSelection}
struct SnippetAdvance {var session:SnippetSession?;var selection:CompletionSelection}
// Admit retained metadata before constructing another field forest. Empty
// alternatives still occupy slots; payload accounting uses exact UTF-8 bytes.
func snippetMetadataCost(_ fields:[SnippetField])->Int? {
    guard fields.count<=4096 else{return nil};var bytes=0
    for field in fields {
        guard field.choices.count<=4096 else{return nil}
        let floor=64+32*field.choices.count
        guard floor<=1048576-bytes else{return nil};bytes+=floor
        if let transform=field.transform{guard transform.bytes>=0,transform.bytes<=1048576-bytes else{return nil};bytes+=transform.bytes}
        for choice in field.choices{let size=choice.utf8.count;guard size<=1048576-bytes else{return nil};bytes+=size}
    }
    return bytes
}
private func snippetScalar(_ text:NSString,_ at:Int)->Bool {
    guard at>=0,at<=text.length else{return false}
    if at>0,at<text.length{let before=text.character(at:at-1),after=text.character(at:at);if (before>=0xd800&&before<=0xdbff&&after>=0xdc00&&after<=0xdfff)||(before==13&&after==10){return false}}
    return true
}
func beginSnippet(_ source:String,fields:[SnippetField])->SnippetSession? {
    var budget=SnippetTransformBudget()
    return beginSnippet(source,fields:fields,budget:&budget)
}
func beginSnippet(_ source:String,fields:[SnippetField],budget:inout SnippetTransformBudget)->SnippetSession? {
    let text=source as NSString
    guard !fields.isEmpty,snippetMetadataCost(fields) != nil,fields.allSatisfy({field in field.id != nil && field.id!>=0 && field.id!<4096 && field.index>=0 && field.index<=1004096 && field.from<=field.to && snippetScalar(text,field.from) && snippetScalar(text,field.to)}),fields.filter({$0.index==0}).count==1,!fields.contains(where:{$0.index==0&&$0.from != $0.to}) else{return nil}
    var ids:[Int:SnippetField]=[:]
    for field in fields{guard ids[field.id!]==nil else{return nil};ids[field.id!]=field}
    var visited:Set<Int>=[]
    for field in fields{var chain:Set<Int>=[],next:SnippetField?=field
        while let value=next,!visited.contains(value.id!){guard chain.insert(value.id!).inserted else{return nil};guard let identity=value.parent else{break};guard let parent=ids[identity],parent.from<=value.from,parent.to>=value.to else{return nil};next=parent};visited.formUnion(chain)
    }
    if fields.contains(where:{$0.transform != nil}){guard (try? validateSnippetMirrors(source,fields,budget:&budget)) != nil else{return nil}}
    guard let active=fields.filter({$0.transform==nil}).map({$0.index}).filter({$0>0}).min(),let first=fields.first(where:{$0.transform==nil&&$0.index==active}) else{return nil}
    return SnippetSession(source:source,fields:fields,active:active,current:first.id!)
}
func advanceSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,direction:Int)->SnippetAdvance? {
    guard snippetMetadataCost(session.fields) != nil,exactUTF8(source,session.source),from<=to,(direction==1||direction == -1),session.fields.contains(where:{$0.transform==nil&&$0.index==session.active&&$0.from<=from&&$0.to>=to}) else{return nil}
    let scope=session.scopes.last,order=scope.map{$0.order+[$0.exitIndex]} ?? (Array(Set(session.fields.filter{$0.transform==nil}.map{$0.index}.filter{$0>0})).sorted()+[0])
    guard let at=order.firstIndex(of:session.active),order.indices.contains(at+direction),let field=session.fields.first(where:{$0.transform==nil&&$0.index==order[at+direction]}) else{return nil}
    if let scope,field.index==scope.exitIndex{guard let exit=session.fields.first(where:{$0.id==scope.exitId}) else{return nil};var resumed=session;resumed.fields.removeAll{scope.order.contains($0.index)||$0.index==scope.exitIndex};resumed.active=scope.outerActive;resumed.current=scope.outerCurrent;resumed.scopes.removeLast();return SnippetAdvance(session:resumed,selection:CompletionSelection(from:exit.from,to:exit.to))}
    var next=session;next.active=field.index;next.current=field.id!
    return SnippetAdvance(session:field.index==0 ? nil:next,selection:CompletionSelection(from:field.from,to:field.to))
}

private struct SnippetReplacement {var from:Int;var to:Int;var text:String;var owners:[Int]}
func editSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,text:String)->SnippetInput? {
    var budget=SnippetTransformBudget()
    return editSnippet(session,source:source,from:from,to:to,text:text,budget:&budget)
}
func editSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,text:String,budget:inout SnippetTransformBudget)->SnippetInput? {
    if session.fields.contains(where:{$0.transform != nil}){return editDerivedSnippet(session,source:source,from:from,to:to,text:text,budget:&budget)}
    let original=source as NSString
    guard snippetMetadataCost(session.fields) != nil,exactUTF8(source,session.source),from<=to,snippetScalar(original,from),snippetScalar(original,to),session.active != 0 else{return nil}
    func terminal(_ field:SnippetField)->Bool{field.index==0 || session.scopes.contains(where:{$0.exitIndex==field.index})}
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
                let enclosing=edit.owners.indices.filter{terminal(field)&&ancestor(edit.owners[$0],field.id!)}
                if let last=enclosing.last{return edit.from+delta+text.utf16.count*(last+1)}
                return edit.from+delta+text.utf16.count*edit.owners.filter{ordinals[$0]!<ordinals[field.id!]!}.count
            }
            let owns=edit.owners.contains(field.id!)||edit.owners.contains(where:{ancestor(field.id!,$0)}),exit=terminal(field)&&edit.owners.contains(where:{ancestor($0,field.id!)})
            let right=owns ? end:exit ? true:position==edit.to
            return edit.from+delta+(right ? edit.text.utf16.count:0)
        }
        return position+delta
    }
    let retained=fields.filter{field in terminal(field)||field.index==session.active || !occurrences.contains(where:{ancestor($0.id!,field.id!)})},retainedIds=Set(retained.map{$0.id!})
    let updated=retained.map{field in var value=field;var parent=field.parent;while let identity=parent,!retainedIds.contains(identity){parent=ids[identity]?.parent};value.parent=parent;value.from=map(field,field.from,false);value.to=map(field,field.to,true);return value}
    let output=after as NSString;var groups:[Int:String]=[:]
    for field in updated{guard field.from<=field.to,snippetScalar(output,field.from),snippetScalar(output,field.to) else{return nil};let value=output.substring(with:NSRange(location:field.from,length:field.to-field.from));if let previous=groups[field.index],!exactUTF8(previous,value){return nil};groups[field.index]=value}
    let caret=map(origin,from,false)+text.utf16.count
    return SnippetInput(session:SnippetSession(source:after,fields:updated,active:session.active,current:origin.id!,scopes:session.scopes),edits:native,selection:CompletionSelection(from:caret,to:caret))
}
func completeSnippet(_ session:SnippetSession,source:String,primary:SourceReplacement,additional:[SourceReplacement])->SnippetInput? {
    var budget=SnippetTransformBudget()
    return completeSnippet(session,source:source,primary:primary,additional:additional,budget:&budget)
}
func completeSnippet(_ session:SnippetSession,source:String,primary:SourceReplacement,additional:[SourceReplacement],budget:inout SnippetTransformBudget)->SnippetInput? {
    guard let input=editSnippet(session,source:source,from:primary.from,to:primary.to,text:primary.text,budget:&budget) else{return nil}
    if additional.isEmpty{return input}
    let start=session.fields.map{$0.from}.min()!,end=session.fields.map{$0.to}.max()!,text=source as NSString
    var shift=0,bytes=0,edits=input.edits.map{SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)}
    for edit in additional.reversed() {
        guard edit.from<=edit.to,snippetScalar(text,edit.from),snippetScalar(text,edit.to) else{return nil}
        bytes+=edit.text.utf8.count;guard bytes<=1048576 else{return nil}
        if edit.to<=start{shift+=edit.text.utf16.count-(edit.to-edit.from)}else if edit.from<end{return nil}
        if let index=edits.firstIndex(where:{$0.from==edit.from}){guard edit.from==edit.to,edit.to<=start else{return nil};edits[index].text=edit.text+edits[index].text}else{edits.append(edit)}
    }
    edits.sort{$0.from==$1.from ? $0.to<$1.to:$0.from<$1.from}
    for index in edits.indices.dropFirst(){guard edits[index-1].to<=edits[index].from,edits[index-1].from != edits[index].from else{return nil}}
    let output=NSMutableString(string:source);for edit in edits.reversed(){output.replaceCharacters(in:NSRange(location:edit.from,length:edit.to-edit.from),with:edit.text)}
    let after=output as String,fields=input.session.fields.map{field in var value=field;value.from+=shift;value.to+=shift;return value}
    guard beginSnippet(after,fields:fields,budget:&budget) != nil else{return nil}
    var next=input.session;next.source=after;next.fields=fields
    return SnippetInput(session:next,edits:edits.map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selection:CompletionSelection(from:input.selection.from+shift,to:input.selection.to+shift))
}
func nestSnippet(_ session:SnippetSession,source:String,primary:SourceReplacement,additional:[SourceReplacement],inner:[SnippetField])->SnippetInput? {
    var budget=SnippetTransformBudget()
    return nestSnippet(session,source:source,primary:primary,additional:additional,inner:inner,budget:&budget)
}
func nestSnippet(_ session:SnippetSession,source:String,primary:SourceReplacement,additional:[SourceReplacement],inner:[SnippetField],budget:inout SnippetTransformBudget)->SnippetInput? {
    guard session.scopes.count<32,!inner.isEmpty,let innerCost=snippetMetadataCost(inner),let exit=inner.first(where:{$0.index==0}) else{return nil}
    let parsed=beginSnippet(primary.text,fields:inner,budget:&budget);guard parsed != nil || (inner.count==1 && exit.id != nil && exit.parent==nil && exit.from==exit.to && snippetScalar(primary.text as NSString,exit.from)) else{return nil}
    guard let origin=session.fields.first(where:{$0.id==session.current&&$0.from<=primary.from&&$0.to>=primary.to}),let input=completeSnippet(session,source:source,primary:primary,additional:additional,budget:&budget) else{return nil}
    let parents=input.session.fields.filter{$0.transform==nil&&$0.index==session.active};guard let current=parents.first(where:{$0.id==input.session.current}) else{return nil}
    let relative=primary.from-origin.from,order=Array(Set(inner.filter{$0.transform==nil&&$0.index>0}.map{$0.index})).sorted()
    if order.isEmpty{var result=input;let caret=current.from+relative+exit.from;result.selection=CompletionSelection(from:caret,to:caret);return result}
    // Reserve the complete mirror fanout before appending any nested fields.
    guard input.session.fields.count+parents.count*inner.count<=4096,let retainedCost=snippetMetadataCost(input.session.fields),parents.count<=(1048576-retainedCost)/innerCost else{return nil}
    var nextIndex=input.session.fields.map{$0.index}.max()!+1,indices:[Int:Int]=[:]
    for index in order+[0]{indices[index]=nextIndex;nextIndex+=1};guard nextIndex<=1004097 else{return nil}
    var used=Set(input.session.fields.map{$0.id!}),fields=input.session.fields,nextId=0,exitId = -1,currentId = -1
    for parent in parents {
        var ids:[Int:Int]=[:];for field in inner{while used.contains(nextId){nextId+=1};guard nextId<4096 else{return nil};ids[field.id!]=nextId;used.insert(nextId);nextId+=1}
        for field in inner{var value=field;let id=ids[field.id!]!;value.id=id;value.index=indices[field.index]!;value.parent=field.parent.map{ids[$0]!} ?? parent.id;value.from=parent.from+relative+field.from;value.to=parent.from+relative+field.to;fields.append(value)
            if parent.id==current.id{if field.index==0{exitId=id}else if field.id==parsed!.current{currentId=id}}
        }
    }
    guard beginSnippet(input.session.source,fields:fields,budget:&budget) != nil,let selected=fields.first(where:{$0.id==currentId}) else{return nil}
    let scope=SnippetScope(order:order.map{indices[$0]!},exitIndex:indices[0]!,exitId:exitId,outerActive:session.active,outerCurrent:input.session.current)
    var result=input;result.session.fields=fields;result.session.active=scope.order[0];result.session.current=currentId;result.session.scopes.append(scope);result.selection=CompletionSelection(from:selected.from,to:selected.to);return result
}
private struct DerivedSnippetPart {var from:Int;var to:Int;var text:String;var owner:Int}
private struct DerivedSnippetSpan {var owner:Int;var from:Int;var to:Int}
private struct DerivedSnippetReplacement {var from:Int;var to:Int;var text:String;var spans:[DerivedSnippetSpan]}
private func editDerivedSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,text:String,budget:inout SnippetTransformBudget)->SnippetInput? {
 let original=source as NSString
 guard exactUTF8(source,session.source),from<=to,snippetScalar(original,from),snippetScalar(original,to),session.active>0,snippetMetadataCost(session.fields) != nil else{return nil}
 do {
  _ = try validateSnippetMirrors(source,session.fields,budget:&budget)
  let fields=session.fields;var ids:[Int:SnippetField]=[:],ordinals:[Int:Int]=[:];for(index,field)in fields.enumerated(){guard let id=field.id else{return nil};ids[id]=field;ordinals[id]=index}
  func ancestors(_ parent:Int,_ child:Int)->Bool{var next=ids[child],steps=0;while let value=next,steps<=fields.count{if value.id==parent{return true};next=value.parent.flatMap{ids[$0]};steps+=1};return false}
  func terminal(_ field:SnippetField)->Bool{field.index==0 || session.scopes.contains{$0.exitIndex==field.index}}
  let occurrences=fields.filter{$0.transform==nil && $0.index==session.active},matches=occurrences.filter{$0.from<=from && $0.to>=to}
  guard let origin=matches.first(where:{$0.id==session.current}) ?? matches.first,text.utf8.count<=1048576/Swift.max(occurrences.count,1) else{return nil}
  let before=original.substring(with:NSRange(location:origin.from,length:origin.to-origin.from)),relativeFrom=from-origin.from,relativeTo=to-origin.from
  guard occurrences.allSatisfy({exactUTF8(original.substring(with:NSRange(location:$0.from,length:$0.to-$0.from)),before)}) else{return nil}
  let beforeText=before as NSString
  if exactUTF8(beforeText.substring(to:relativeFrom)+text+beforeText.substring(from:relativeTo),before){var next=session;next.current=origin.id!;return SnippetInput(session:next,edits:[],selection:CompletionSelection(from:from+text.utf16.count,to:from+text.utf16.count))}
  var retained=fields.filter{field in terminal(field) || field.index==session.active ||  !occurrences.contains{ancestors($0.id!,field.id!)}}
  let ordinary=Set(retained.filter{$0.transform==nil}.map(\.index));retained=retained.filter{$0.transform==nil || ordinary.contains($0.index)};let retainedIDs=Set(retained.map{$0.id!})
  retained=retained.map{field in var value=field,parent=field.parent;while let id=parent,!retainedIDs.contains(id){parent=ids[id]?.parent};value.parent=parent;return value}
  var parts=occurrences.map{DerivedSnippetPart(from:$0.from+relativeFrom,to:$0.from+relativeTo,text:text,owner:$0.id!)};var payload=parts.reduce(0){$0+$1.text.utf8.count}
  guard parts.allSatisfy({snippetScalar(original,$0.from) && snippetScalar(original,$0.to)}) else{return nil}
  func compare(_ a:DerivedSnippetPart,_ b:DerivedSnippetPart)->Bool{if a.from != b.from{return a.from<b.from};if a.to != b.to{return a.to<b.to};return ordinals[a.owner]!<ordinals[b.owner]!}
  func projected(_ field:SnippetField)throws->String{let edits=parts.filter{ancestors(field.id!,$0.owner)}.sorted(by:compare);guard edits.allSatisfy({$0.from>=field.from && $0.to<=field.to}) else{throw EngineError(message:"Computed source edit escapes its parent")};var value=original.substring(with:NSRange(location:field.from,length:field.to-field.from));for edit in edits.reversed(){let source=value as NSString;value=source.substring(to:edit.from-field.from)+edit.text+source.substring(from:edit.to-field.from)};return value}
  var expected:[Int:String]=[:]
  let dependencyOrder=try snippetTransformOrder(retained)
  for index in dependencyOrder {
   let mirrors:[SnippetField]=retained.filter{$0.transform==nil  &&  $0.index==index}
   let computed:[SnippetField]=retained.filter{$0.transform != nil  &&  $0.index==index}
   if computed.isEmpty{continue}
   let value=try projected(mirrors[0])
   for field in mirrors {let mirrored=try projected(field);guard exactUTF8(mirrored,value) else{return nil}}
   for field in computed {
    let result=try applySnippetTransform(field.transform!,value,budget:&budget);expected[field.id!]=result
    let oldValue=original.substring(with:NSRange(location:field.from,length:field.to-field.from))
    if !exactUTF8(result,oldValue){payload+=result.utf8.count;guard payload<=1048576 else{return nil};parts.append(DerivedSnippetPart(from:field.from,to:field.to,text:result,owner:field.id!))}
   }
  }
  var replacements:[DerivedSnippetReplacement]=[],at=0;let sorted=parts.sorted(by:compare)
  while at<sorted.count{let start=sorted[at].from;var group:[DerivedSnippetPart]=[];while at<sorted.count,sorted[at].from==start{group.append(sorted[at]);at+=1};guard group.filter({$0.to>$0.from}).count<=1 else{return nil};group.sort{ordinals[$0.owner]!<ordinals[$1.owner]!};var edit=DerivedSnippetReplacement(from:start,to:group.map(\.to).max()!,text:"",spans:[]);for part in group{let offset=edit.text.utf16.count;edit.text+=part.text;edit.spans.append(DerivedSnippetSpan(owner:part.owner,from:offset,to:edit.text.utf16.count))};guard replacements.last.map({$0.to<=start}) ?? true else{return nil};replacements.append(edit)}
  let changed=NSMutableString(string:source);for edit in replacements.reversed(){changed.replaceCharacters(in:NSRange(location:edit.from,length:edit.to-edit.from),with:edit.text)};let after=changed as String
  func map(_ field:SnippetField,_ position:Int,_ end:Bool)->Int{var delta=0;for edit in replacements{if position<edit.from{break};if position>edit.to || position==edit.to && edit.from<edit.to{delta+=edit.text.utf16.count-(edit.to-edit.from);continue};let related=edit.spans.filter{ancestors(field.id!,$0.owner)};if let first=related.first,let last=related.last{return edit.from+delta+(end ? last.to:first.from)};let enclosing=edit.spans.filter{terminal(field) && ancestors($0.owner,field.id!)};if let last=enclosing.last{return edit.from+delta+last.to};return edit.from+delta+edit.spans.filter{ordinals[$0.owner]!<ordinals[field.id!]!}.reduce(0){$0+$1.to-$1.from}};return position+delta}
  let updated=retained.map{field in var value=field;value.from=map(field,field.from,false);value.to=map(field,field.to,true);return value},afterText=after as NSString
  guard updated.allSatisfy({$0.from<=$0.to && snippetScalar(afterText,$0.from) && snippetScalar(afterText,$0.to)}) else{return nil}
  var groups:[Int:String]=[:];for field in updated{let value=afterText.substring(with:NSRange(location:field.from,length:field.to-field.from));if field.transform != nil{guard let wanted=expected[field.id!],exactUTF8(value,wanted) else{return nil}}else{if let previous=groups[field.index],!exactUTF8(previous,value){return nil};groups[field.index]=value}}
  let caret=map(origin,from,false)+text.utf16.count
  return SnippetInput(session:SnippetSession(source:after,fields:updated,active:session.active,current:origin.id!,scopes:session.scopes),edits:replacements.map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selection:CompletionSelection(from:caret,to:caret))
 }catch{return nil}
}
