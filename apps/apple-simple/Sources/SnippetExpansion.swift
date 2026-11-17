import Foundation
struct SnippetField {var index:Int;var from:Int;var to:Int;var choices:[String];var id:Int?=nil;var parent:Int?=nil;var transform:SnippetTransform?=nil}
func completionExpansion(_ item:[String:Any])throws->CompletionExpansion? {
    guard sourceChangeInteger(item["insertTextFormat"])==2 else{return nil}
    guard let raw=(item["data"] as? [String:Any])?["hickory_snippet"] as? [String:Any],let version=sourceChangeInteger(raw["version"]),[1,2].contains(version),let syntax=raw["syntax"] as? String,exactUTF8(syntax,try completionInsertion(item)),let text=raw["text"] as? String,text.utf8.count<=1048576,let values=raw["fields"] as? [[String:Any]],!values.isEmpty,values.count<=4096 else{throw EngineError(message:"Snippet expansion unavailable or out of date. Request completions again.")}
    var fields:[SnippetField]=[];var metadataBytes=0
    for value in values {
        guard let index=sourceChangeInteger(value["index"]),index>=0,index<=1004096,let from=sourceChangeInteger(value["from"]),let to=sourceChangeInteger(value["to"]),from>=0,from<=to,to<=text.utf16.count,let rawChoices=value["choices"] as? NSArray else{throw EngineError(message:"Malformed snippet field")}
        guard rawChoices.count<=4096,metadataBytes+64+rawChoices.count*32<=1048576 else{throw EngineError(message:"Snippet metadata expansion limit exceeded")}
        metadataBytes+=64
        for rawChoice in rawChoices {
            guard let choice=rawChoice as? String else{throw EngineError(message:"Malformed snippet field")}
            let bytes=choice.utf8.count
            guard bytes<=1048576-32,metadataBytes<=1048576-32-bytes else{throw EngineError(message:"Snippet metadata expansion limit exceeded")}
            metadataBytes+=32+bytes
        }
        guard let choices=rawChoices as? [String] else{throw EngineError(message:"Malformed snippet field")}
        _ = try sourceChangeOffset(text,languagePosition(text,from));_ = try sourceChangeOffset(text,languagePosition(text,to))
        var id:Int?=nil,parent:Int?=nil
        if value["id"] != nil {
            guard let identity=sourceChangeInteger(value["id"]),identity>=0,identity<4096 else{throw EngineError(message:"Malformed snippet ancestry")};id=identity
            if !(value["parent"] is NSNull){guard let identity=sourceChangeInteger(value["parent"]),identity>=0,identity<4096 else{throw EngineError(message:"Malformed snippet ancestry")};parent=identity}
        }else if value["parent"] != nil{throw EngineError(message:"Malformed snippet ancestry")}
        let transform:SnippetTransform?
        if let descriptor=value["transform"],!(descriptor is NSNull){guard version==2,index>0,choices.isEmpty,id != nil else{throw EngineError(message:"Computed snippet metadata requires protocol 2 and an editable source group")};transform=try readSnippetTransform(descriptor);guard transform!.bytes<=1048576-metadataBytes else{throw EngineError(message:"Snippet metadata expansion limit exceeded")};metadataBytes+=transform!.bytes}else{transform=nil}
        fields.append(SnippetField(index:index,from:from,to:to,choices:choices,id:id,parent:parent,transform:transform))
    }
    if fields.contains(where:{$0.id != nil}) {
        var ids:[Int:SnippetField]=[:]
        for field in fields{guard let id=field.id,ids[id]==nil else{throw EngineError(message:"Malformed snippet ancestry")};ids[id]=field}
        var visited:Set<Int>=[]
        for field in fields{var chain:Set<Int>=[],next:SnippetField?=field
            while let current=next,!visited.contains(current.id!){guard chain.insert(current.id!).inserted else{throw EngineError(message:"Cyclic snippet ancestry")};guard let identity=current.parent else{break};guard let parent=ids[identity],parent.from<=current.from,parent.to>=current.to else{throw EngineError(message:"Malformed snippet ancestry")};next=parent}
            visited.formUnion(chain)
        }
    }
    guard fields.filter({$0.index==0}).count==1 else{throw EngineError(message:"Malformed snippet exit")}
    _ = try validateSnippetMirrors(text,fields)
    let active=fields.filter{$0.transform==nil&&$0.index>0}.map(\.index).min() ?? 0,selected=fields.first{$0.transform==nil&&$0.index==active}!
    return CompletionExpansion(text:text,selection:CompletionSelection(from:selected.from,to:selected.to),fields:fields)
}

func snippetTransformOrder(_ fields:[SnippetField])throws->[Int] {
 var ids:[Int:SnippetField]=[:];for field in fields{if let id=field.id{ids[id]=field}}
 let ordinary=Set(fields.filter{$0.transform==nil}.map(\.index));var dependencies=Dictionary(uniqueKeysWithValues:ordinary.map{($0,Set<Int>())}),work=0,edges=0
 for field in fields where field.transform != nil {
  guard field.index>0,let identity=field.id,field.choices.isEmpty,ordinary.contains(field.index),!fields.contains(where:{$0.parent==identity}) else{throw EngineError(message:"Malformed computed snippet occurrence")}
  var seen:Set<Int>=[],next=field.parent.flatMap{ids[$0]}
  while let value=next{work+=1;guard work<=65536,let id=value.id,seen.insert(id).inserted else{throw EngineError(message:"Computed snippet ancestry limit exceeded")};if value.index>0{guard value.index != field.index,var values=dependencies[value.index] else{throw EngineError(message:"Cyclic or noneditable computed snippet parent")};if values.insert(field.index).inserted{edges+=1;guard edges<=4096 else{throw EngineError(message:"Computed snippet dependency limit exceeded")};dependencies[value.index]=values}};next=value.parent.flatMap{ids[$0]}}
 }
 var parents:[Int:[Int]]=[:];for parent in dependencies.keys.sorted(){for child in dependencies[parent]!.sorted(){parents[child,default:[]].append(parent)}}
 var queue=dependencies.keys.filter{dependencies[$0]!.isEmpty}.sorted(),order:[Int]=[],at=0
 while at<queue.count{let child=queue[at];at+=1;order.append(child);for parent in parents[child] ?? []{dependencies[parent]!.remove(child);if dependencies[parent]!.isEmpty{queue.append(parent)}}}
 guard order.count==ordinary.count else{throw EngineError(message:"Cyclic computed snippet dependency")};return order
}
func validateSnippetMirrors(_ text:String,_ fields:[SnippetField])throws->[Int] {var budget=SnippetTransformBudget();return try validateSnippetMirrors(text,fields,budget:&budget)}
func validateSnippetMirrors(_ text:String,_ fields:[SnippetField],budget:inout SnippetTransformBudget)throws->[Int] {
 var first:[Int:SnippetField]=[:];let source=text as NSString
 for field in fields where field.transform==nil {if let previous=first[field.index]{guard exactUTF8(source.substring(with:NSRange(location:previous.from,length:previous.to-previous.from)),source.substring(with:NSRange(location:field.from,length:field.to-field.from))),previous.choices.count==field.choices.count,zip(previous.choices,field.choices).allSatisfy({exactUTF8($0.0,$0.1)}) else{throw EngineError(message:"Inconsistent snippet mirrors")}};first[field.index]=field}
 let order=try snippetTransformOrder(fields)
 for field in fields where field.transform != nil {guard let value=first[field.index] else{throw EngineError(message:"Computed snippet source is unavailable")};let before=source.substring(with:NSRange(location:value.from,length:value.to-value.from)),after=source.substring(with:NSRange(location:field.from,length:field.to-field.from));guard exactUTF8(try applySnippetTransform(field.transform!,before,budget:&budget),after) else{throw EngineError(message:"Inconsistent computed snippet occurrence")}}
 return order
}
