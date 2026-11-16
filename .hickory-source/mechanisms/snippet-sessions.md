<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Derive linked snippet edits from explicit source-bound field state

For maintainers attaching template interaction to existing editors. Hosts retain immediate ordinary input, source permissions, composition and undo authority; this model prepares linked edits and traversal without UI or disk effects.

## Compose completion imports once around a linked replacement

The completion planner retains separate primary and additional edits. Mirror the primary through the current session, then combine imports in original source coordinates. Imports before the entire field forest shift its fields and intended selection; imports after it leave those positions unchanged. Prefix imports at the primary start precede its text. The source planner has already deduplicated provider additions. Refuse additions inside the forest, overlapping replacements, invalid scalar boundaries and excessive import text before any host mutation. This deliberately conservative boundary keeps an import from becoming a mirrored field edit. Nested insertion clones the inner occurrence forest under each outer mirror with distinct occurrence identities and fresh group indices. A bounded stack owns local traversal order and terminal occurrences. Editing a nested field retains and maps its terminal positions; leaving the inner scope restores the captured outer active occurrence at the inner exit, without saving or changing editor layout. Every edit retains the scope stack for native history context.


## Keep browser field editing independent of the view

A source-bound session owns ordered occurrences and active intent. Only an edit wholly inside a current active occurrence can produce linked replacements. Parent edits retire descendants; child edits update enclosing mirrors. Coincident empty mirrors produce ordered text for every occurrence, while explicit ancestry distinguishes them from nested fields. Validate scalar boundaries and fanout before producing a proposal. A stale or unsupported edit returns no linked plan so the host can retain ordinary input.

The pure capacity check counts field overhead and every alternative, including empty strings, before cloning choices. Nested proposals reserve the retained forest plus the complete inner metadata multiplied by the number of outer mirrors; an excessive proposal returns no edits or new session.

<ui:copy id="snippet-session-browser">
import type {SnippetField} from './SnippetExpansion';
export interface SnippetScope {order:number[];exitIndex:number;exitId:number;outerActive:number;outerCurrent:number}
export interface SnippetSession {source:string;fields:SnippetField[];active:number;current:number;scopes?:SnippetScope[]}
export interface SnippetEdit {from:number;to:number;text:string}
export interface SnippetInput {session:SnippetSession;edits:SnippetEdit[];selection:{from:number;to:number}}
const scalar=(text:string,at:number)=>Number.isSafeInteger(at)&&at>=0&&at<=text.length&&!(at>0&&at<text.length&&((text.charCodeAt(at-1)>=0xd800&&text.charCodeAt(at-1)<=0xdbff&&text.charCodeAt(at)>=0xdc00&&text.charCodeAt(at)<=0xdfff)||(text[at-1]==='\r'&&text[at]==='\n')));
function snippetMetadataCost(fields:readonly SnippetField[]):number|null {
 if(fields.length>4096)return null;let bytes=0;
 for(const field of fields){if(!field||!Array.isArray(field.choices)||field.choices.length>4096)return null;bytes+=64+field.choices.length*32;if(bytes>1048576)return null;for(const choice of field.choices){if(typeof choice!=='string'||choice.length>1048576-bytes)return null;bytes+=choice.length?new TextEncoder().encode(choice).length:0;if(bytes>1048576)return null;}}
 return bytes;
}
export function beginSnippet(source:string,fields:readonly SnippetField[]):SnippetSession|null {
 if(!fields.length||fields.length>4096||snippetMetadataCost(fields)===null||fields.some(field=>field.id===undefined||!Number.isSafeInteger(field.id)||field.id!<0||field.id!>=4096||!scalar(source,field.from)||!scalar(source,field.to)||field.from>field.to||!Number.isSafeInteger(field.index)||field.index<0||field.index>1004096||field.parent!=null&&!Number.isSafeInteger(field.parent)))return null;
 const ids=new Map(fields.map(field=>[field.id!,field]));if(ids.size!==fields.length||fields.filter(field=>field.index===0).length!==1||fields.some(field=>field.index===0&&field.from!==field.to))return null;
 for(const field of fields){const seen=new Set<number>();let next:SnippetField|undefined=field;while(next){if(seen.has(next.id!))return null;seen.add(next.id!);if(next.parent==null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)return null;next=parent;}}
 const active=Math.min(...fields.map(field=>field.index).filter(index=>index>0));
 const first=fields.find(field=>field.index===active);return first?{source,fields:fields.map(field=>({...field,choices:[...field.choices],parent:field.parent??null})),active,current:first.id!}:null;
}
export function advanceSnippet(session:SnippetSession,source:string,from:number,to:number,direction:1|-1):{session:SnippetSession|null;selection:{from:number;to:number}}|null {
 if(source!==session.source||from>to||!session.fields.some(field=>field.index===session.active&&field.from<=from&&field.to>=to))return null;
 const scope=session.scopes?.at(-1),order=scope?[...scope.order,scope.exitIndex]:[...new Set(session.fields.map(field=>field.index).filter(index=>index>0))].sort((a,b)=>a-b).concat(0);
 const at=order.indexOf(session.active),next=at+direction;if(at<0||next<0||next>=order.length)return null;
 const active=order[next];
 if(scope&&active===scope.exitIndex){const field=session.fields.find(field=>field.id===scope.exitId);if(!field)return null;return {session:{...session,fields:session.fields.filter(field=>!scope.order.includes(field.index)&&field.index!==scope.exitIndex),active:scope.outerActive,current:scope.outerCurrent,scopes:session.scopes!.slice(0,-1)},selection:{from:field.from,to:field.to}};}
 const field=session.fields.find(field=>field.index===active)!;
 return {session:active===0?null:{...session,active,current:field.id!},selection:{from:field.from,to:field.to}};
}

</ui:copy>

### Prepare one complete linked input proposal

Capture and numeric traversal above retain a source-bound intent, without editor or disk effects. For input, require the entire replacement to lie inside one current active occurrence, match the exact source and validate scalar boundaries. Compute all mirrors before returning anything. Explicit parents decide which descendants retire and which enclosing fields grow; a no-op keeps nested fields. Ordered coincident empty mirrors produce separate text and positions rather than collapsing to one insertion. Map source, fields and the originating caret together, and bound total UTF-8 mirror fanout before constructing output. A refused proposal leaves ordinary host input authoritative.

<ui:copy id="snippet-session-browser-input">
export function editSnippet(session:SnippetSession,source:string,from:number,to:number,text:string):SnippetInput|null {
 if(source!==session.source||from>to||!scalar(source,from)||!scalar(source,to)||session.active===0)return null;
 const terminal=(field:SnippetField)=>field.index===0||!!session.scopes?.some(scope=>scope.exitIndex===field.index);
 const fields=session.fields,ids=new Map(fields.map(field=>[field.id!,field]));
 const ancestors=(parent:number,child:number)=>{let current=ids.get(child);for(let steps=0;current&&steps<=fields.length;steps++){if(current.id===parent)return true;current=current.parent===null?undefined:ids.get(current.parent!);}return false;};
 const occurrences=fields.filter(field=>field.index===session.active),matches=occurrences.filter(field=>field.from<=from&&field.to>=to),origin=matches.find(field=>field.id===session.current)??matches[0];
 if(!origin||new TextEncoder().encode(text).length>1048576/Math.max(occurrences.length,1))return null;
 const before=source.slice(origin.from,origin.to),relativeFrom=from-origin.from,relativeTo=to-origin.from;
 if(occurrences.some(field=>source.slice(field.from,field.to)!==before))return null;
 const replacements:{from:number;to:number;text:string;owners:Set<number>}[]=[];
 for(const field of occurrences){const from=field.from+relativeFrom,to=field.from+relativeTo;if(!scalar(source,from)||!scalar(source,to))return null;const same=replacements.find(edit=>edit.from===from&&edit.to===to);if(same){if(from!==to)return null;same.text+=text;same.owners.add(field.id!);}else replacements.push({from,to,text,owners:new Set([field.id!])});}
 replacements.sort((a,b)=>a.from-b.from||a.to-b.to);
 if(replacements.some((edit,index)=>index>0&&replacements[index-1].to>edit.from))return null;
 let after=source;for(const edit of [...replacements].reverse())after=after.slice(0,edit.from)+edit.text+after.slice(edit.to);
 if(after===source)return {session:{...session,current:origin.id!},edits:replacements.map(({from,to,text})=>({from,to,text})),selection:{from:from+text.length,to:from+text.length}};
 const ordinals=new Map(fields.map((field,index)=>[field.id!,index]));
 const map=(field:SnippetField,position:number,end:boolean)=>{let delta=0;for(const edit of replacements){if(position<edit.from)break;if(position>edit.to){delta+=edit.text.length-(edit.to-edit.from);continue;}
   if(edit.from===edit.to){
    const owners=[...edit.owners],related=owners.map((id,index)=>({id,index})).filter(({id})=>ancestors(field.id!,id));
    if(related.length)return edit.from+delta+text.length*(end?related.at(-1)!.index+1:related[0].index);
    const enclosing=owners.map((id,index)=>({id,index})).filter(({id})=>terminal(field)&&ancestors(id,field.id!));
    if(enclosing.length)return edit.from+delta+text.length*(enclosing.at(-1)!.index+1);
    return edit.from+delta+text.length*owners.filter(id=>ordinals.get(id)!<ordinals.get(field.id!)!).length;
   }
   const owns=edit.owners.has(field.id!)||[...edit.owners].some(id=>ancestors(field.id!,id));
   const exit=terminal(field)&&[...edit.owners].some(id=>ancestors(id,field.id!));
   const right=owns?end:exit?true:position===edit.to;
   return edit.from+delta+(right?edit.text.length:0);
  }return position+delta;};
 const retained=fields.filter(field=>after===source||terminal(field)||field.index===session.active || !occurrences.some(parent=>ancestors(parent.id!,field.id!)));
 const retainedIds=new Set(retained.map(field=>field.id!));
 const updated=retained.map(field=>{let parent=field.parent;while(parent!==null&&parent!==undefined&&!retainedIds.has(parent))parent=ids.get(parent)?.parent;return {...field,parent:parent??null,from:map(field,field.from,false),to:map(field,field.to,true)};});
 if(updated.some(field=>field.from>field.to||!scalar(after,field.from)||!scalar(after,field.to)))return null;
 const groups=new Map<number,string>();for(const field of updated){const value=after.slice(field.from,field.to),previous=groups.get(field.index);if(previous!==undefined&&previous!==value)return null;groups.set(field.index,value);}
 const caret=map(origin,from,false)+text.length;
 return {session:{source:after,fields:updated,active:session.active,current:origin.id!,scopes:session.scopes},edits:replacements.map(({from,to,text})=>({from,to,text})),selection:{from:caret,to:caret}};
}
</ui:copy>

## Compose browser completion imports around the field forest

Compose the primary mirrored edit with imports once in original source coordinates. Imports outside the complete field forest may change source length; only imports before it shift field ranges and the intended caret. Merge coincident prefix insertions in provider order and reject interior or overlapping additions before returning any proposal.

<ui:copy id="snippet-session-browser-completion">
export function completeSnippet(session:SnippetSession,source:string,primary:SnippetEdit,additional:readonly SnippetEdit[]):SnippetInput|null {
 const input=editSnippet(session,source,primary.from,primary.to,primary.text);if(!input)return null;
 if(!additional.length)return input;
 const start=Math.min(...session.fields.map(field=>field.from)),end=Math.max(...session.fields.map(field=>field.to));
 let shift=0,bytes=0;const edits=input.edits.map(edit=>({...edit}));
 for(const edit of [...additional].reverse()){
  if(edit.from>edit.to||!scalar(source,edit.from)||!scalar(source,edit.to))return null;
  bytes+=new TextEncoder().encode(edit.text).length;if(bytes>1048576)return null;
  if(edit.to<=start)shift+=edit.text.length-(edit.to-edit.from);else if(edit.from<end)return null;
  const same=edits.find(value=>value.from===edit.from);
  if(same){if(edit.from!==edit.to||edit.to>start)return null;same.text=edit.text+same.text;}else edits.push({...edit});
 }
 edits.sort((a,b)=>a.from-b.from||a.to-b.to);
 if(edits.some((edit,index)=>index>0&&(edits[index-1].to>edit.from||edits[index-1].from===edit.from)))return null;
 let after=source;for(const edit of [...edits].reverse())after=after.slice(0,edit.from)+edit.text+after.slice(edit.to);
 const fields=input.session.fields.map(field=>({...field,from:field.from+shift,to:field.to+shift}));
 if(!beginSnippet(after,fields))return null;
 return {session:{...input.session,source:after,fields},edits,selection:{from:input.selection.from+shift,to:input.selection.to+shift}};
}
export function nestSnippet(session:SnippetSession,source:string,primary:SnippetEdit,additional:readonly SnippetEdit[],inner:readonly SnippetField[]):SnippetInput|null {
 if((session.scopes?.length??0)>=32||!inner.length||snippetMetadataCost(session.fields)===null)return null;
 const innerBytes=snippetMetadataCost(inner),exit=inner.find(field=>field.index===0);if(innerBytes===null||!exit)return null;
 const origin=session.fields.find(field=>field.id===session.current&&field.from<=primary.from&&field.to>=primary.to);if(!origin)return null;
 const input=completeSnippet(session,source,primary,additional);if(!input)return null;
 const parents=input.session.fields.filter(field=>field.index===session.active),current=parents.find(field=>field.id===input.session.current);if(!current)return null;
 const relative=primary.from-origin.from,order=[...new Set(inner.filter(field=>field.index>0).map(field=>field.index))].sort((a,b)=>a-b);
 if(order.length){const retainedBytes=snippetMetadataCost(input.session.fields);if(retainedBytes===null||retainedBytes+parents.length*innerBytes>1048576||input.session.fields.length+parents.length*inner.length>4096)return null;}
 // Reserve the complete mirrored forest before beginSnippet can clone inner choices.
 const parsed=beginSnippet(primary.text,inner);if(!parsed&&(inner.length!==1||exit.id===undefined||exit.parent!=null||exit.from!==exit.to||!scalar(primary.text,exit.from)))return null;
 if(!order.length){const caret=current.from+relative+exit.from;return {...input,selection:{from:caret,to:caret}};}
 let nextIndex=Math.max(...input.session.fields.map(field=>field.index))+1;const indices=new Map([...order,0].map(index=>[index,nextIndex++]));if(nextIndex>1004097)return null;
 const used=new Set(input.session.fields.map(field=>field.id!)),fields=[...input.session.fields];let nextId=0,exitId=-1,currentId=-1;
 for(const parent of parents){const ids=new Map<number,number>();for(const field of inner){while(used.has(nextId))nextId++;if(nextId>=4096)return null;ids.set(field.id!,nextId);used.add(nextId++);}
  for(const field of inner){const id=ids.get(field.id!)!;fields.push({...field,id,index:indices.get(field.index)!,parent:field.parent==null?parent.id!:ids.get(field.parent)!,from:parent.from+relative+field.from,to:parent.from+relative+field.to});if(parent.id===current.id){if(field.index===0)exitId=id;else if(field.id===parsed!.current)currentId=id;}}
 }
 if(snippetMetadataCost(fields)===null||!beginSnippet(input.session.source,fields))return null;
 const scope:SnippetScope={order:order.map(index=>indices.get(index)!),exitIndex:indices.get(0)!,exitId,outerActive:session.active,outerCurrent:input.session.current},active=scope.order[0],selected=fields.find(field=>field.id===currentId);if(!selected)return null;
 return {...input,session:{...input.session,fields,active,current:currentId,scopes:[...(input.session.scopes??[]),scope]},selection:{from:selected.from,to:selected.to}};
}
</ui:copy>

<ui:file path="apps/web/src/simple/SnippetSession.ts"><ui:paste select="#snippet-session-browser"/><ui:paste select="#snippet-session-browser-input"/><ui:paste select="#snippet-session-browser-completion"/></ui:file>

## Keep Apple field editing independent of the view

Use the same occurrence and revision model as the browser, with exact UTF-8 comparisons and UTF-16 editor coordinates. Parent/child scope is explicit, coincident insertions preserve every occurrence and no-op edits retain nested fields. Before retaining a forest, charge each field 64 bytes and each choice 32 bytes plus its UTF-8 payload, with at most 4096 choices per field and one MiB total. Nested insertion reserves all mirrored inner forests before constructing them. This pure model returns the complete native edit batch and final selection; it neither edits a view nor saves a file.

<ui:copy id="snippet-session-apple">
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
    let text=source as NSString
    guard !fields.isEmpty,snippetMetadataCost(fields) != nil,fields.allSatisfy({field in field.id != nil && field.id!>=0 && field.id!<4096 && field.index>=0 && field.index<=1004096 && field.from<=field.to && snippetScalar(text,field.from) && snippetScalar(text,field.to)}),fields.filter({$0.index==0}).count==1,!fields.contains(where:{$0.index==0&&$0.from != $0.to}) else{return nil}
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
    guard snippetMetadataCost(session.fields) != nil,exactUTF8(source,session.source),from<=to,(direction==1||direction == -1),session.fields.contains(where:{$0.index==session.active&&$0.from<=from&&$0.to>=to}) else{return nil}
    let scope=session.scopes.last,order=scope.map{$0.order+[$0.exitIndex]} ?? (Array(Set(session.fields.map{$0.index}.filter{$0>0})).sorted()+[0])
    guard let at=order.firstIndex(of:session.active),order.indices.contains(at+direction),let field=session.fields.first(where:{$0.index==order[at+direction]}) else{return nil}
    if let scope,field.index==scope.exitIndex{guard let exit=session.fields.first(where:{$0.id==scope.exitId}) else{return nil};var resumed=session;resumed.fields.removeAll{scope.order.contains($0.index)||$0.index==scope.exitIndex};resumed.active=scope.outerActive;resumed.current=scope.outerCurrent;resumed.scopes.removeLast();return SnippetAdvance(session:resumed,selection:CompletionSelection(from:exit.from,to:exit.to))}
    var next=session;next.active=field.index;next.current=field.id!
    return SnippetAdvance(session:field.index==0 ? nil:next,selection:CompletionSelection(from:field.from,to:field.to))
}

</ui:copy>

### Prepare one complete linked input proposal

Capture and numeric traversal above retain a source-bound intent, without editor or disk effects. For input, require the entire replacement to lie inside one current active occurrence, match the exact source and validate scalar boundaries. Compute all mirrors before returning anything. Explicit parents decide which descendants retire and which enclosing fields grow; a no-op keeps nested fields. Ordered coincident empty mirrors produce separate text and positions rather than collapsing to one insertion. Map source, fields and the originating caret together, and bound total UTF-8 mirror fanout before constructing output. A refused proposal leaves ordinary host input authoritative.

<ui:copy id="snippet-session-apple-input">
private struct SnippetReplacement {var from:Int;var to:Int;var text:String;var owners:[Int]}
func editSnippet(_ session:SnippetSession,source:String,from:Int,to:Int,text:String)->SnippetInput? {
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
</ui:copy>

## Compose apple completion imports around the field forest

Compose the primary mirrored edit with imports once in original source coordinates. Imports outside the complete field forest may change source length; only imports before it shift field ranges and the intended caret. Merge coincident prefix insertions in provider order and reject interior or overlapping additions before returning any proposal.

<ui:copy id="snippet-session-apple-completion">
func completeSnippet(_ session:SnippetSession,source:String,primary:SourceReplacement,additional:[SourceReplacement])->SnippetInput? {
    guard let input=editSnippet(session,source:source,from:primary.from,to:primary.to,text:primary.text) else{return nil}
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
    guard beginSnippet(after,fields:fields) != nil else{return nil}
    var next=input.session;next.source=after;next.fields=fields
    return SnippetInput(session:next,edits:edits.map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selection:CompletionSelection(from:input.selection.from+shift,to:input.selection.to+shift))
}
func nestSnippet(_ session:SnippetSession,source:String,primary:SourceReplacement,additional:[SourceReplacement],inner:[SnippetField])->SnippetInput? {
    guard session.scopes.count<32,!inner.isEmpty,let innerCost=snippetMetadataCost(inner),let exit=inner.first(where:{$0.index==0}) else{return nil}
    let parsed=beginSnippet(primary.text,fields:inner);guard parsed != nil || (inner.count==1 && exit.id != nil && exit.parent==nil && exit.from==exit.to && snippetScalar(primary.text as NSString,exit.from)) else{return nil}
    guard let origin=session.fields.first(where:{$0.id==session.current&&$0.from<=primary.from&&$0.to>=primary.to}),let input=completeSnippet(session,source:source,primary:primary,additional:additional) else{return nil}
    let parents=input.session.fields.filter{$0.index==session.active};guard let current=parents.first(where:{$0.id==input.session.current}) else{return nil}
    let relative=primary.from-origin.from,order=Array(Set(inner.filter{$0.index>0}.map{$0.index})).sorted()
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
    guard beginSnippet(input.session.source,fields:fields) != nil,let selected=fields.first(where:{$0.id==currentId}) else{return nil}
    let scope=SnippetScope(order:order.map{indices[$0]!},exitIndex:indices[0]!,exitId:exitId,outerActive:session.active,outerCurrent:input.session.current)
    var result=input;result.session.fields=fields;result.session.active=scope.order[0];result.session.current=currentId;result.session.scopes.append(scope);result.selection=CompletionSelection(from:selected.from,to:selected.to);return result
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SnippetSession.swift"><ui:paste select="#snippet-session-apple"/><ui:paste select="#snippet-session-apple-input"/><ui:paste select="#snippet-session-apple-completion"/></ui:file>

## Keep Android field editing independent of the view

Keep source, active intent and field relationships in one immutable plan. Linked replacements, field retirement, parent mapping, numeric traversal and exit selection follow the shared vectors. Bound mirror fanout before constructing text. Rejected intent produces no partial mirrored edits and does not authorize a source write.

<ui:copy id="snippet-session-android">
package com.hickorydocs.simple
data class SnippetScope(val order:List<Int>,val exitIndex:Int,val exitId:Int,val outerActive:Int,val outerCurrent:Int)
data class SnippetSession(val source:String,val fields:List<SnippetField>,val active:Int,val current:Int,val scopes:List<SnippetScope> = emptyList())
data class SnippetInput(val session:SnippetSession,val edits:List<NativeSourceEdit>,val selection:CompletionSelection)
data class SnippetAdvance(val session:SnippetSession?,val selection:CompletionSelection)
private fun snippetScalar(text:String,at:Int):Boolean=at in 0..text.length&&!(at>0&&at<text.length&&(Character.isHighSurrogate(text[at-1])&&Character.isLowSurrogate(text[at])||text[at-1]=='\r'&&text[at]=='\n'))
// A shared forest admission check runs before any retained choices vector is
// copied. Empty alternatives still occupy slots; UTF-8 payloads add to them.
internal fun snippetMetadataCost(fields:List<SnippetField>,budget:Long=1048576L):Long? {
 if(fields.size>4096)return null;var bytes=0L
 for(field in fields){if(field.choices.size>4096)return null;val floor=64L+32L*field.choices.size;if(floor>budget-bytes)return null;bytes+=floor
  for(choice in field.choices){if(choice.length.toLong()>budget-bytes)return null;val payload=choice.toByteArray(Charsets.UTF_8).size.toLong();if(payload>budget-bytes)return null;bytes+=payload}
 }
 return bytes
}
fun beginSnippet(source:String,fields:List<SnippetField>):SnippetSession? {
 if(fields.isEmpty()||snippetMetadataCost(fields)==null||fields.any{it.id==null||it.id !in 0..4095||it.index !in 0..1004096||it.from>it.to||!snippetScalar(source,it.from)||!snippetScalar(source,it.to)}||fields.count{it.index==0}!=1||fields.any{it.index==0&&it.from!=it.to})return null
 val ids=fields.associateBy{it.id!!};if(ids.size!=fields.size)return null;val visited=mutableSetOf<Int>()
 for(field in fields){val chain=mutableSetOf<Int>();var next:SnippetField?=field
  while(next!=null&&!visited.contains(next.id!!)){val value=next;if(!chain.add(value.id!!))return null;val identity=value.parent ?: break;val parent=ids[identity] ?: return null;if(parent.from>value.from||parent.to<value.to)return null;next=parent};visited.addAll(chain)
 }
 val active=fields.map{it.index}.filter{it>0}.minOrNull() ?: return null;return SnippetSession(source,fields.map{it.copy(choices=it.choices.toList())},active,fields.first{it.index==active}.id!!)
}
fun advanceSnippet(session:SnippetSession,source:String,from:Int,to:Int,direction:Int):SnippetAdvance? {
 if(source!=session.source||from>to||direction!=1&&direction != -1||session.fields.none{it.index==session.active&&it.from<=from&&it.to>=to})return null
 val scope=session.scopes.lastOrNull();val order=scope?.let{it.order+it.exitIndex} ?: (session.fields.map{it.index}.filter{it>0}.distinct().sorted()+0);val at=order.indexOf(session.active);val next=at+direction;if(at<0||next !in order.indices)return null
 if(scope!=null&&order[next]==scope.exitIndex){val field=session.fields.firstOrNull{it.id==scope.exitId} ?: return null;return SnippetAdvance(session.copy(fields=session.fields.filter{it.index !in scope.order&&it.index!=scope.exitIndex},active=scope.outerActive,current=scope.outerCurrent,scopes=session.scopes.dropLast(1)),CompletionSelection(field.from,field.to))}
 val field=session.fields.first{it.index==order[next]};return SnippetAdvance(if(field.index==0)null else session.copy(active=field.index,current=field.id!!),CompletionSelection(field.from,field.to))
}

</ui:copy>

### Prepare one complete linked input proposal

Capture and numeric traversal above retain a source-bound intent, without editor or disk effects. For input, require the entire replacement to lie inside one current active occurrence, match the exact source and validate scalar boundaries. Compute all mirrors before returning anything. Explicit parents decide which descendants retire and which enclosing fields grow; a no-op keeps nested fields. Ordered coincident empty mirrors produce separate text and positions rather than collapsing to one insertion. Map source, fields and the originating caret together, and bound total UTF-8 mirror fanout before constructing output. A refused proposal leaves ordinary host input authoritative.

<ui:copy id="snippet-session-android-input">
private data class SnippetReplacement(val from:Int,val to:Int,var text:String,val owners:MutableList<Int>)
fun editSnippet(session:SnippetSession,source:String,from:Int,to:Int,text:String):SnippetInput? {
 if(source!=session.source||from>to||!snippetScalar(source,from)||!snippetScalar(source,to)||session.active==0)return null
 fun terminal(field:SnippetField)=field.index==0||session.scopes.any{it.exitIndex==field.index}
 val fields=session.fields;val ids=fields.associateBy{it.id!!}
 fun ancestor(parent:Int,child:Int):Boolean{var current=ids[child];repeat(fields.size+1){val value=current ?: return false;if(value.id==parent)return true;current=value.parent?.let{ids[it]}};return false}
 val occurrences=fields.filter{it.index==session.active};val matches=occurrences.filter{it.from<=from&&it.to>=to};val origin=matches.firstOrNull{it.id==session.current} ?: matches.firstOrNull() ?: return null
 if(text.toByteArray(Charsets.UTF_8).size>1048576/maxOf(occurrences.size,1))return null
 val before=source.substring(origin.from,origin.to);val relativeFrom=from-origin.from;val relativeTo=to-origin.from
 if(occurrences.any{source.substring(it.from,it.to)!=before})return null
 val edits=mutableListOf<SnippetReplacement>()
 for(field in occurrences){val from=field.from+relativeFrom;val to=field.from+relativeTo;if(!snippetScalar(source,from)||!snippetScalar(source,to))return null
  val same=edits.firstOrNull{it.from==from&&it.to==to};if(same!=null){if(from!=to)return null;same.text+=text;same.owners.add(field.id!!)}else edits.add(SnippetReplacement(from,to,text,mutableListOf(field.id!!)))
 }
 edits.sortWith(compareBy({it.from},{it.to}));for(index in 1 until edits.size)if(edits[index-1].to>edits[index].from)return null
 var after=source;for(edit in edits.asReversed())after=after.replaceRange(edit.from,edit.to,edit.text)
 val native=edits.map{NativeSourceEdit(it.from,it.to,it.text)}
 if(after==source)return SnippetInput(session.copy(current=origin.id!!),native,CompletionSelection(from+text.length,from+text.length))
 val ordinals=fields.mapIndexed{index,field->field.id!! to index}.toMap()
 fun map(field:SnippetField,position:Int,end:Boolean):Int {
  var delta=0;for(edit in edits){if(position<edit.from)break;if(position>edit.to){delta+=edit.text.length-(edit.to-edit.from);continue}
   if(edit.from==edit.to){val related=edit.owners.indices.filter{ancestor(field.id!!,edit.owners[it])};if(related.isNotEmpty())return edit.from+delta+text.length*(if(end)related.last()+1 else related.first())
    val enclosing=edit.owners.indices.filter{terminal(field)&&ancestor(edit.owners[it],field.id!!)};if(enclosing.isNotEmpty())return edit.from+delta+text.length*(enclosing.last()+1)
    return edit.from+delta+text.length*edit.owners.count{ordinals.getValue(it)<ordinals.getValue(field.id!!)}
   }
   val owns=edit.owners.contains(field.id!!)||edit.owners.any{ancestor(field.id!!,it)};val exit=terminal(field)&&edit.owners.any{ancestor(it,field.id!!)};val right=if(owns)end else if(exit)true else position==edit.to
   return edit.from+delta+if(right)edit.text.length else 0
  };return position+delta
 }
 val retained=fields.filter{field->terminal(field)||field.index==session.active||occurrences.none{ancestor(it.id!!,field.id!!)}};val retainedIds=retained.map{it.id!!}.toSet()
 val updated=retained.map{field->var parent=field.parent;while(parent!=null&&!retainedIds.contains(parent))parent=ids[parent]?.parent;field.copy(parent=parent,from=map(field,field.from,false),to=map(field,field.to,true))}
 val groups=mutableMapOf<Int,String>();for(field in updated){if(field.from>field.to||!snippetScalar(after,field.from)||!snippetScalar(after,field.to))return null;val value=after.substring(field.from,field.to);val previous=groups[field.index];if(previous!=null&&previous!=value)return null;groups[field.index]=value}
 val caret=map(origin,from,false)+text.length;return SnippetInput(SnippetSession(after,updated,session.active,origin.id!!,session.scopes),native,CompletionSelection(caret,caret))
}
</ui:copy>

## Compose android completion imports around the field forest

Compose the primary mirrored edit with imports once in original source coordinates. Imports outside the complete field forest may change source length; only imports before it shift field ranges and the intended caret. Merge coincident prefix insertions in provider order and reject interior or overlapping additions before returning any proposal.

<ui:copy id="snippet-session-android-completion">
fun completeSnippet(session:SnippetSession,source:String,primary:SourceReplacement,additional:List<SourceReplacement>):SnippetInput? {
 val input=editSnippet(session,source,primary.from,primary.to,primary.text) ?: return null
 if(additional.isEmpty())return input
 val start=session.fields.minOf{it.from};val end=session.fields.maxOf{it.to};var shift=0;var bytes=0L
 val edits=input.edits.map{SourceReplacement(it.from,it.to,it.text)}.toMutableList()
 for(edit in additional.asReversed()){
  if(edit.from>edit.to||!snippetScalar(source,edit.from)||!snippetScalar(source,edit.to))return null
  bytes+=edit.text.toByteArray(Charsets.UTF_8).size;if(bytes>1048576)return null
  if(edit.to<=start)shift+=edit.text.length-(edit.to-edit.from) else if(edit.from<end)return null
  val index=edits.indexOfFirst{it.from==edit.from}
  if(index>=0){if(edit.from!=edit.to||edit.to>start)return null;edits[index]=edits[index].copy(text=edit.text+edits[index].text)}else edits.add(edit)
 }
 edits.sortWith(compareBy({it.from},{it.to}));for(index in 1 until edits.size)if(edits[index-1].to>edits[index].from||edits[index-1].from==edits[index].from)return null
 var after=source;for(edit in edits.asReversed())after=after.replaceRange(edit.from,edit.to,edit.text)
 val fields=input.session.fields.map{it.copy(from=it.from+shift,to=it.to+shift)};if(beginSnippet(after,fields)==null)return null
 return SnippetInput(input.session.copy(source=after,fields=fields),edits.map{NativeSourceEdit(it.from,it.to,it.text)},CompletionSelection(input.selection.from+shift,input.selection.to+shift))
}
fun nestSnippet(session:SnippetSession,source:String,primary:SourceReplacement,additional:List<SourceReplacement>,inner:List<SnippetField>):SnippetInput? {
 if(session.scopes.size>=32||inner.isEmpty()||snippetMetadataCost(session.fields)==null)return null
 val innerBytes=snippetMetadataCost(inner) ?: return null
 val exit=inner.firstOrNull{it.index==0} ?: return null
 val origin=session.fields.firstOrNull{it.id==session.current&&it.from<=primary.from&&it.to>=primary.to} ?: return null
 val input=completeSnippet(session,source,primary,additional) ?: return null;val parents=input.session.fields.filter{it.index==session.active};val current=parents.firstOrNull{it.id==input.session.current} ?: return null
 val relative=primary.from-origin.from;val order=inner.filter{it.index>0}.map{it.index}.distinct().sorted()
 if(order.isNotEmpty()){
  val retainedBytes=snippetMetadataCost(input.session.fields) ?: return null
  if(parents.isEmpty()||innerBytes>(1048576L-retainedBytes)/parents.size||input.session.fields.size.toLong()+parents.size.toLong()*inner.size>4096)return null
 }
 // Only after the entire mirrored proposal fits may validation clone an inner
 // vector. A rejected nested proposal publishes no session or text edits.
 val parsed=beginSnippet(primary.text,inner);if(parsed==null&&(inner.size!=1||exit.id==null||exit.parent!=null||exit.from!=exit.to||!snippetScalar(primary.text,exit.from)))return null
 if(order.isEmpty()){val caret=current.from+relative+exit.from;return input.copy(selection=CompletionSelection(caret,caret))}
 var nextIndex=input.session.fields.maxOf{it.index}+1;val indices=(order+0).associateWith{nextIndex++};if(nextIndex>1004097)return null
 val used=input.session.fields.map{it.id!!}.toMutableSet();val fields=input.session.fields.toMutableList();var nextId=0;var exitId = -1;var currentId = -1
 for(parent in parents){val ids=mutableMapOf<Int,Int>();for(field in inner){while(nextId in used)nextId++;if(nextId>=4096)return null;ids[field.id!!]=nextId;used.add(nextId++)}
  for(field in inner){val id=ids.getValue(field.id!!);fields.add(field.copy(id=id,index=indices.getValue(field.index),parent=field.parent?.let{ids.getValue(it)} ?: parent.id,from=parent.from+relative+field.from,to=parent.from+relative+field.to));if(parent.id==current.id){if(field.index==0)exitId=id else if(field.id==parsed!!.current)currentId=id}}
 }
 if(snippetMetadataCost(fields)==null||beginSnippet(input.session.source,fields)==null)return null
 val selected=fields.firstOrNull{it.id==currentId} ?: return null;val scope=SnippetScope(order.map{indices.getValue(it)},indices.getValue(0),exitId,session.active,input.session.current)
 return input.copy(session=input.session.copy(fields=fields,active=scope.order[0],current=currentId,scopes=input.session.scopes+scope),selection=CompletionSelection(selected.from,selected.to))
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SnippetSession.kt"><ui:paste select="#snippet-session-android"/><ui:paste select="#snippet-session-android-input"/><ui:paste select="#snippet-session-android-completion"/></ui:file>

## Specify linked input and source selection outcomes

These independently written cases state source text, every retained field occurrence and selection after edits and traversal. They distinguish parent replacement, child editing, no-op preservation, coincident empty mirrors, numeric versus source order, Unicode and nested exits. Stale/outside/scalar-invalid input and excessive fanout yield no linked proposal. These are model witnesses, not native IME acceptance.

<ui:copy id="snippet-session-vectors">
{
  "cases": [
    {
      "name": "linked typing and final exit",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "value",
          "after": "value + value",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 5,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 8,
              "to": 13,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 13,
              "to": 13,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 5,
            "to": 5
          }
        },
        {
          "action": "edit",
          "from": 5,
          "to": 5,
          "text": "!",
          "after": "value! + value!",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 6,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 9,
              "to": 15,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 15,
              "to": 15,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 6,
            "to": 6
          }
        },
        {
          "action": "next",
          "from": 6,
          "to": 6,
          "active": null,
          "selection": {
            "from": 15,
            "to": 15
          }
        }
      ]
    },
    {
      "name": "editing another mirror keeps its caret",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 4,
          "to": 5,
          "text": "value",
          "after": "value + value",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 5,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 8,
              "to": 13,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 13,
              "to": 13,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 13,
            "to": 13
          }
        }
      ]
    },
    {
      "name": "nested child updates mirrored parents",
      "source": "foo bar = foo bar",
      "fields": [
        {
          "id": 1,
          "index": 2,
          "from": 4,
          "to": 7,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 7,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 2,
          "from": 14,
          "to": 17,
          "choices": [],
          "parent": 2
        },
        {
          "id": 2,
          "index": 1,
          "from": 10,
          "to": 17,
          "choices": [],
          "parent": null
        },
        {
          "id": 4,
          "index": 0,
          "from": 17,
          "to": 17,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "next",
          "from": 0,
          "to": 7,
          "active": 2,
          "selection": {
            "from": 4,
            "to": 7
          }
        },
        {
          "action": "edit",
          "from": 4,
          "to": 7,
          "text": "x",
          "after": "foo x = foo x",
          "fields": [
            {
              "id": 1,
              "index": 2,
              "from": 4,
              "to": 5,
              "choices": [],
              "parent": 0
            },
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 5,
              "choices": [],
              "parent": null
            },
            {
              "id": 3,
              "index": 2,
              "from": 12,
              "to": 13,
              "choices": [],
              "parent": 2
            },
            {
              "id": 2,
              "index": 1,
              "from": 8,
              "to": 13,
              "choices": [],
              "parent": null
            },
            {
              "id": 4,
              "index": 0,
              "from": 13,
              "to": 13,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 5,
            "to": 5
          }
        },
        {
          "action": "prev",
          "from": 5,
          "to": 5,
          "active": 1,
          "selection": {
            "from": 0,
            "to": 5
          }
        },
        {
          "action": "edit",
          "from": 0,
          "to": 5,
          "text": "v",
          "after": "v = v",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 1,
              "from": 4,
              "to": 5,
              "choices": [],
              "parent": null
            },
            {
              "id": 4,
              "index": 0,
              "from": 5,
              "to": 5,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "editing parent retires child fields",
      "source": "foo bar = foo bar",
      "fields": [
        {
          "id": 1,
          "index": 2,
          "from": 4,
          "to": 7,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 7,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 2,
          "from": 14,
          "to": 17,
          "choices": [],
          "parent": 2
        },
        {
          "id": 2,
          "index": 1,
          "from": 10,
          "to": 17,
          "choices": [],
          "parent": null
        },
        {
          "id": 4,
          "index": 0,
          "from": 17,
          "to": 17,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 7,
          "text": "value",
          "after": "value = value",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 5,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 1,
              "from": 8,
              "to": 13,
              "choices": [],
              "parent": null
            },
            {
              "id": 4,
              "index": 0,
              "from": 13,
              "to": 13,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 5,
            "to": 5
          }
        }
      ]
    },
    {
      "name": "no-op parent retains child fields",
      "source": "foo bar = foo bar",
      "fields": [
        {
          "id": 1,
          "index": 2,
          "from": 4,
          "to": 7,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 7,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 2,
          "from": 14,
          "to": 17,
          "choices": [],
          "parent": 2
        },
        {
          "id": 2,
          "index": 1,
          "from": 10,
          "to": 17,
          "choices": [],
          "parent": null
        },
        {
          "id": 4,
          "index": 0,
          "from": 17,
          "to": 17,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 7,
          "text": "foo bar",
          "after": "foo bar = foo bar",
          "fields": [
            {
              "id": 1,
              "index": 2,
              "from": 4,
              "to": 7,
              "choices": [],
              "parent": 0
            },
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 7,
              "choices": [],
              "parent": null
            },
            {
              "id": 3,
              "index": 2,
              "from": 14,
              "to": 17,
              "choices": [],
              "parent": 2
            },
            {
              "id": 2,
              "index": 1,
              "from": 10,
              "to": 17,
              "choices": [],
              "parent": null
            },
            {
              "id": 4,
              "index": 0,
              "from": 17,
              "to": 17,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 7,
            "to": 7
          }
        }
      ]
    },
    {
      "name": "partial parent editing retires child fields",
      "source": "foo bar = foo bar",
      "fields": [
        {
          "id": 1,
          "index": 2,
          "from": 4,
          "to": 7,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 7,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 2,
          "from": 14,
          "to": 17,
          "choices": [],
          "parent": 2
        },
        {
          "id": 2,
          "index": 1,
          "from": 10,
          "to": 17,
          "choices": [],
          "parent": null
        },
        {
          "id": 4,
          "index": 0,
          "from": 17,
          "to": 17,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "F",
          "after": "Foo bar = Foo bar",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 7,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 1,
              "from": 10,
              "to": 17,
              "choices": [],
              "parent": null
            },
            {
              "id": 4,
              "index": 0,
              "from": 17,
              "to": 17,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "nested empty field is not an adjacent sibling",
      "source": "",
      "fields": [
        {
          "id": 1,
          "index": 2,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 3,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 0,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 0,
          "text": "a",
          "after": "a",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 3,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 3,
              "index": 0,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "editing nested empty child extends its parent",
      "source": "",
      "fields": [
        {
          "id": 1,
          "index": 2,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 3,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 0,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "next",
          "from": 0,
          "to": 0,
          "active": 2,
          "selection": {
            "from": 0,
            "to": 0
          }
        },
        {
          "action": "edit",
          "from": 0,
          "to": 0,
          "text": "a",
          "after": "a",
          "fields": [
            {
              "id": 1,
              "index": 2,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": 0
            },
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 3,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 3,
              "index": 0,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "active": 3,
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "coincident empty mirrors keep distinct occurrences",
      "source": "",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 2,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 1,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 3,
          "index": 0,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 0,
          "text": "x",
          "after": "xx",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 2,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 1,
              "from": 1,
              "to": 2,
              "choices": [],
              "parent": null
            },
            {
              "id": 3,
              "index": 0,
              "from": 2,
              "to": 2,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "active": 2,
          "selection": {
            "from": 1,
            "to": 1
          }
        },
        {
          "action": "edit",
          "from": 1,
          "to": 1,
          "text": "y",
          "after": "xyx",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 2,
              "from": 1,
              "to": 2,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 1,
              "from": 2,
              "to": 3,
              "choices": [],
              "parent": null
            },
            {
              "id": 3,
              "index": 0,
              "from": 3,
              "to": 3,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 2,
            "to": 2
          }
        }
      ]
    },
    {
      "name": "numeric order differs from source order",
      "source": "",
      "fields": [
        {
          "id": 0,
          "index": 2,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 0,
          "to": 0,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 0,
          "text": "x",
          "after": "x",
          "fields": [
            {
              "id": 0,
              "index": 2,
              "from": 0,
              "to": 0,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "active": 2,
          "selection": {
            "from": 0,
            "to": 0
          }
        },
        {
          "action": "edit",
          "from": 0,
          "to": 0,
          "text": "y",
          "after": "yx",
          "fields": [
            {
              "id": 0,
              "index": 2,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 1,
              "to": 2,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 2,
              "to": 2,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "Unicode forms remain byte distinct",
      "source": "😀 = 😀",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 2,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 5,
          "to": 7,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 7,
          "to": 7,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 2,
          "text": "é",
          "after": "é = é",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 2,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 5,
              "to": 7,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 7,
              "to": 7,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 2,
            "to": 2
          }
        },
        {
          "action": "edit",
          "from": 0,
          "to": 2,
          "text": "é",
          "after": "é = é",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 4,
              "to": 5,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 5,
              "to": 5,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "multiline linked input maps UTF16 ranges",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 1,
          "to": 1,
          "text": "\r\nb",
          "after": "a\r\nb + a\r\nb",
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 4,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 7,
              "to": 11,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 11,
              "to": 11,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 4,
            "to": 4
          }
        },
        {
          "action": "edit",
          "from": 2,
          "to": 2,
          "text": "x",
          "refuse": true
        }
      ]
    },
    {
      "name": "nested exit follows replaced parent",
      "source": "foo suffix",
      "fields": [
        {
          "id": 1,
          "index": 0,
          "from": 2,
          "to": 2,
          "choices": [],
          "parent": 0
        },
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 3,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 3,
          "text": "x",
          "after": "x suffix",
          "fields": [
            {
              "id": 1,
              "index": 0,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": 0
            },
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            }
          ],
          "selection": {
            "from": 1,
            "to": 1
          }
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "active": null,
          "selection": {
            "from": 1,
            "to": 1
          }
        }
      ]
    },
    {
      "name": "stale source ends editing intent",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "x",
          "currentSource": "changed",
          "refuse": true
        }
      ]
    },
    {
      "name": "typing outside a field stays ordinary input",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 2,
          "to": 3,
          "text": "x",
          "refuse": true
        }
      ]
    },
    {
      "name": "split Unicode scalar refuses mirroring",
      "source": "😀 = 😀",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 2,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 5,
          "to": 7,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 7,
          "to": 7,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 1,
          "to": 2,
          "text": "x",
          "refuse": true
        }
      ]
    },
    {
      "name": "different mirror contents refuse whole linked edit",
      "source": "a + b",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "x",
          "refuse": true
        }
      ]
    },
    {
      "name": "bounded mirror fanout",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "x",
          "repeat": 600000,
          "refuse": true
        }
      ]
    },
    {
      "name": "previous before first field is unavailable",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "prev",
          "from": 0,
          "to": 1,
          "refuse": true
        }
      ]
    },
    {
      "name": "navigation outside field is unavailable",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "next",
          "from": 2,
          "to": 2,
          "refuse": true
        }
      ]
    },
    {
      "name": "legacy fields do not imply ancestry",
      "source": "a",
      "fields": [
        {
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": []
        },
        {
          "index": 0,
          "from": 1,
          "to": 1,
          "choices": []
        }
      ],
      "invalidBegin": true
    },
    {
      "name": "duplicate occurrence identities",
      "source": "a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 0,
          "index": 0,
          "from": 1,
          "to": 1,
          "choices": [],
          "parent": null
        }
      ],
      "invalidBegin": true
    },
    {
      "name": "cyclic occurrence ancestry",
      "source": "a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": 0
        },
        {
          "id": 1,
          "index": 0,
          "from": 1,
          "to": 1,
          "choices": [],
          "parent": null
        }
      ],
      "invalidBegin": true
    },
    {
      "name": "nonempty exit is not a field cursor",
      "source": "a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 0,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        }
      ],
      "invalidBegin": true
    },
    {
      "name": "UTF8 mirror budget is identical on every host",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "😀",
          "repeat": 150000,
          "refuse": true
        }
      ]
    },
    {
      "name": "prefix and suffix imports stay single around all mirrors",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "complete",
          "from": 0,
          "to": 1,
          "text": "long",
          "additional": [
            {
              "from": 0,
              "to": 0,
              "text": "# import\n"
            },
            {
              "from": 5,
              "to": 5,
              "text": "\n# end"
            }
          ],
          "after": "# import\nlong + long\n# end",
          "selection": {
            "from": 13,
            "to": 13
          },
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 9,
              "to": 13,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 16,
              "to": 20,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 20,
              "to": 20,
              "choices": [],
              "parent": null
            }
          ],
          "active": 1
        }
      ]
    },
    {
      "name": "coincident prefix imports retain provider order",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "complete",
          "from": 0,
          "to": 1,
          "text": "long",
          "additional": [
            {
              "from": 0,
              "to": 0,
              "text": "first\n"
            },
            {
              "from": 0,
              "to": 0,
              "text": "second\n"
            }
          ],
          "after": "first\nsecond\nlong + long",
          "selection": {
            "from": 17,
            "to": 17
          },
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 13,
              "to": 17,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 20,
              "to": 24,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 24,
              "to": 24,
              "choices": [],
              "parent": null
            }
          ],
          "active": 1
        }
      ]
    },
    {
      "name": "an interior additional edit refuses the complete proposal",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "complete",
          "from": 0,
          "to": 1,
          "text": "long",
          "additional": [
            {
              "from": 2,
              "to": 2,
              "text": "import"
            }
          ],
          "refuse": true
        }
      ]
    },
    {
      "name": "an overlapping additional replacement refuses the complete proposal",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "complete",
          "from": 0,
          "to": 1,
          "text": "long",
          "additional": [
            {
              "from": 0,
              "to": 2,
              "text": "import"
            }
          ],
          "refuse": true
        }
      ]
    },
    {
      "name": "stale source refuses primary and import edits",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "complete",
          "from": 0,
          "to": 1,
          "text": "long",
          "additional": [
            {
              "from": 0,
              "to": 0,
              "text": "import"
            }
          ],
          "refuse": true,
          "currentSource": "b + b"
        }
      ]
    },
    {
      "name": "nested fields mirror through outer occurrences then return to outer scope",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "x:x",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 2,
              "to": 3,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 3,
              "to": 3,
              "choices": [],
              "parent": null
            }
          ],
          "after": "x:x + x:x",
          "selection": {
            "from": 0,
            "to": 1
          },
          "active": 2
        },
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "long",
          "after": "long:long + long:long",
          "selection": {
            "from": 4,
            "to": 4
          },
          "active": 2
        },
        {
          "action": "next",
          "from": 4,
          "to": 4,
          "selection": {
            "from": 9,
            "to": 9
          },
          "active": 1,
          "fields": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 9,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 12,
              "to": 21,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 21,
              "to": 21,
              "choices": [],
              "parent": null
            }
          ]
        },
        {
          "action": "next",
          "from": 9,
          "to": 9,
          "selection": {
            "from": 21,
            "to": 21
          },
          "active": null
        }
      ]
    },
    {
      "name": "nested scope stack returns through two levels",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "x:x",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 2,
              "to": 3,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 3,
              "to": 3,
              "choices": [],
              "parent": null
            }
          ],
          "after": "x:x + x:x",
          "selection": {
            "from": 0,
            "to": 1
          },
          "active": 2
        },
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "z",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 0,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            }
          ],
          "after": "z:z + z:z",
          "selection": {
            "from": 0,
            "to": 1
          },
          "active": 4
        },
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "Q",
          "after": "Q:Q + Q:Q",
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 4
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 2
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "selection": {
            "from": 3,
            "to": 3
          },
          "active": 1
        }
      ]
    },
    {
      "name": "exit-only nested template retains outer scope at its explicit exit",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "xy",
          "inner": [
            {
              "id": 0,
              "index": 0,
              "from": 1,
              "to": 1,
              "choices": [],
              "parent": null
            }
          ],
          "after": "xy + xy",
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 1
        }
      ]
    },
    {
      "name": "malformed nested field forest refuses insertion",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "xy",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": 42
            },
            {
              "id": 1,
              "index": 0,
              "from": 2,
              "to": 2,
              "choices": [],
              "parent": null
            }
          ],
          "refuse": true
        }
      ]
    },
    {
      "name": "empty nested fields retain local exits while typing",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 0,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 0,
              "from": 0,
              "to": 0,
              "choices": [],
              "parent": null
            }
          ],
          "after": " + ",
          "selection": {
            "from": 0,
            "to": 0
          },
          "active": 2
        },
        {
          "action": "edit",
          "from": 0,
          "to": 0,
          "text": "x",
          "after": "x + x",
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 2
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 1
        }
      ]
    },
    {
      "name": "Unicode nested fields use scalar-safe UTF16 ranges",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "nest",
          "from": 0,
          "to": 1,
          "text": "😀",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 2,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 0,
              "from": 2,
              "to": 2,
              "choices": [],
              "parent": null
            }
          ],
          "after": "😀 + 😀",
          "selection": {
            "from": 0,
            "to": 2
          },
          "active": 2
        },
        {
          "action": "edit",
          "from": 0,
          "to": 1,
          "text": "x",
          "refuse": true
        },
        {
          "action": "edit",
          "from": 0,
          "to": 2,
          "text": "é",
          "after": "é + é",
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 2
        },
        {
          "action": "next",
          "from": 1,
          "to": 1,
          "selection": {
            "from": 1,
            "to": 1
          },
          "active": 1
        }
      ]
    },
    {
      "name": "nested insertion retains the selected outer mirror identity",
      "source": "a + a",
      "fields": [
        {
          "id": 0,
          "index": 1,
          "from": 0,
          "to": 1,
          "choices": [],
          "parent": null
        },
        {
          "id": 1,
          "index": 1,
          "from": 4,
          "to": 5,
          "choices": [],
          "parent": null
        },
        {
          "id": 2,
          "index": 0,
          "from": 5,
          "to": 5,
          "choices": [],
          "parent": null
        }
      ],
      "steps": [
        {
          "action": "edit",
          "from": 4,
          "to": 5,
          "text": "a",
          "after": "a + a",
          "selection": {
            "from": 5,
            "to": 5
          },
          "active": 1
        },
        {
          "action": "nest",
          "from": 4,
          "to": 5,
          "text": "x:x",
          "inner": [
            {
              "id": 0,
              "index": 1,
              "from": 0,
              "to": 1,
              "choices": [],
              "parent": null
            },
            {
              "id": 1,
              "index": 1,
              "from": 2,
              "to": 3,
              "choices": [],
              "parent": null
            },
            {
              "id": 2,
              "index": 0,
              "from": 3,
              "to": 3,
              "choices": [],
              "parent": null
            }
          ],
          "after": "x:x + x:x",
          "selection": {
            "from": 6,
            "to": 7
          },
          "active": 2
        },
        {
          "action": "next",
          "from": 6,
          "to": 7,
          "selection": {
            "from": 9,
            "to": 9
          },
          "active": 1
        }
      ]
    }
  ]
}
</ui:copy>

<ui:file path="apps/simple-contract/snippet-session-cases.json"><ui:paste select="#snippet-session-vectors"/></ui:file>

## Run the browser field-session contract

Run the same independently specified source, scope and selection outcomes as the other hosts. The tests cover complete linked edits and refuse stale or unsupported intent without mutating session state. They do not stand in for actual editor, IME or accessibility acceptance.

<ui:copy id="snippet-session-browser-tests">
import {expect,it} from 'vitest';
import vectors from '../../../simple-contract/snippet-session-cases.json';
import type {SnippetField} from './SnippetExpansion';
import {beginSnippet,editSnippet,advanceSnippet,completeSnippet,nestSnippet,type SnippetSession} from './SnippetSession';
interface Step {inner?:SnippetField[];additional?:{from:number;to:number;text:string}[];action:string;from:number;to:number;text?:string;repeat?:number;currentSource?:string;refuse?:boolean;after?:string;fields?:SnippetField[];selection?:{from:number;to:number};active?:number|null}
interface Case {name:string;source:string;fields:SnippetField[];invalidBegin?:boolean;steps?:Step[]}
it('derives complete linked source edits, field scope and traversal from shared vectors',()=>{
 for(const value of vectors.cases as unknown as Case[]){
  let session:SnippetSession|null=beginSnippet(value.source,value.fields),source=value.source;
  if(value.invalidBegin){expect(session,value.name).toBeNull();continue;}expect(session,value.name).not.toBeNull();
  for(const step of value.steps!){const before=JSON.stringify(session),current=session!,input=step.currentSource??source;
   const result=step.action==='nest'?nestSnippet(current,input,{from:step.from,to:step.to,text:step.text??''},step.additional??[],step.inner??[]):step.action==='complete'?completeSnippet(current,input,{from:step.from,to:step.to,text:step.text??''},step.additional??[]):step.action==='edit'?editSnippet(current,input,step.from,step.to,(step.text??'').repeat(step.repeat??1)):advanceSnippet(current,input,step.from,step.to,step.action==='next'?1:-1);
   expect(JSON.stringify(session),value.name).toBe(before);if(step.refuse){expect(result,value.name).toBeNull();continue;}expect(result,value.name).not.toBeNull();
   expect(result!.selection,value.name).toEqual(step.selection);
   if(step.action==='edit'||step.action==='complete'||step.action==='nest'){const edited=result as NonNullable<ReturnType<typeof editSnippet>>;expect(edited.session.source,value.name).toBe(step.after);source=edited.session.source;}
   session=result!.session;
   if(step.fields)expect(session?.fields,value.name).toEqual(step.fields);
   if('active' in step)expect(session?.active??null,value.name).toBe(step.active);
  }
 }
});
it('rejects excessive begin metadata and predicts nested choice fanout before cloning',()=>{
 const field=(id:number,index:number,choices:string[]=[])=>({id,parent:null,index,from:0,to:0,choices});
 expect(beginSnippet('',[field(0,1,Array<string>(4097).fill('')),field(1,0)])).toBeNull();
 const outer=beginSnippet('',[...Array.from({length:8},(_,id)=>field(id,1)),field(8,0)])!,before=JSON.stringify(outer);let reads=0;
 const choices=new Proxy(Array<string>(4096).fill(''),{get(target,key,receiver){if(typeof key==='string'&&/^[0-9]+$/.test(key))reads++;return Reflect.get(target,key,receiver);}}),inner=[field(0,1,choices),field(1,0)];
 expect(nestSnippet(outer,'',{from:0,to:0,text:''},[],inner)).toBeNull();expect(reads).toBe(4096);expect(JSON.stringify(outer)).toBe(before);
 const small=nestSnippet(outer,'',{from:0,to:0,text:''},[],[field(0,1,['','']),field(1,0)]);expect(small?.session.scopes).toHaveLength(1);
});
</ui:copy>

<ui:file path="apps/web/src/simple/SnippetSession.test.ts"><ui:paste select="#snippet-session-browser-tests"/></ui:file>

## Run the Apple field-session contract

Run the same independently specified source, scope and selection outcomes as the other hosts. The tests cover complete linked edits and refuse stale or unsupported intent without mutating session state. They do not stand in for actual editor, IME or accessibility acceptance.

<ui:copy id="snippet-session-apple-tests">
import XCTest
@testable import HickorySimple
final class SnippetSessionTests:XCTestCase {
    func fields(_ values:[[String:Any]])->[SnippetField]{values.map{SnippetField(index:$0["index"] as! Int,from:$0["from"] as! Int,to:$0["to"] as! Int,choices:$0["choices"] as! [String],id:$0["id"] as? Int,parent:$0["parent"] as? Int)}}
    func testNestedEmptyChoicesReserveAllMirrorsBeforePublication()throws {
        let source="aaaaaaaa",outer=(0..<8).map{SnippetField(index:1,from:$0,to:$0+1,choices:[],id:$0,parent:nil)}+[SnippetField(index:0,from:8,to:8,choices:[],id:8,parent:nil)]
        let session=try XCTUnwrap(beginSnippet(source,fields:outer))
        let inner=[SnippetField(index:1,from:0,to:0,choices:Array(repeating:"",count:4096),id:0,parent:nil),SnippetField(index:0,from:0,to:0,choices:[],id:1,parent:nil)]
        XCTAssertNotNil(beginSnippet("",fields:inner))
        XCTAssertNil(nestSnippet(session,source:source,primary:SourceReplacement(from:0,to:1,text:""),additional:[],inner:inner))
        XCTAssertTrue(exactUTF8(session.source,source));XCTAssertEqual(session.fields.count,9)
        var excessive=inner;excessive[0].choices.append("")
        XCTAssertNil(beginSnippet("",fields:excessive))
        let amplified=(0..<8).map{SnippetField(index:1,from:0,to:0,choices:inner[0].choices,id:$0,parent:nil)}+[SnippetField(index:0,from:0,to:0,choices:[],id:8,parent:nil)]
        XCTAssertNil(beginSnippet("",fields:amplified))
        let ordinary=[SnippetField(index:1,from:0,to:1,choices:["x","y"],id:0,parent:nil),SnippetField(index:0,from:1,to:1,choices:[],id:1,parent:nil)]
        XCTAssertNotNil(nestSnippet(session,source:source,primary:SourceReplacement(from:0,to:1,text:"x"),additional:[],inner:ordinary))
    }
    func testSharedLinkedEditsScopeAndTraversal()throws {
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-session-cases.json"))) as! [String:Any]
        for value in vectors["cases"] as! [[String:Any]] {
            let name=value["name"] as! String;var source=value["source"] as! String,session=beginSnippet(value["source"] as! String,fields:fields(value["fields"] as! [[String:Any]]))
            if value["invalidBegin"] as? Bool==true{XCTAssertNil(session,name);continue};XCTAssertNotNil(session,name)
            for step in value["steps"] as! [[String:Any]] {
                let current=try XCTUnwrap(session),input=step["currentSource"] as? String ?? source,from=step["from"] as! Int,to=step["to"] as! Int,action=step["action"] as! String
                let selected:CompletionSelection?,next:SnippetSession?
                if action=="edit" || action=="complete" || action=="nest" {
                    let additional=(step["additional"] as? [[String:Any]] ?? []).map{SourceReplacement(from:$0["from"] as! Int,to:$0["to"] as! Int,text:$0["text"] as! String)}
                    let result=action=="nest" ? nestSnippet(current,source:input,primary:SourceReplacement(from:from,to:to,text:step["text"] as? String ?? ""),additional:additional,inner:fields(step["inner"] as? [[String:Any]] ?? [])):action=="complete" ? completeSnippet(current,source:input,primary:SourceReplacement(from:from,to:to,text:step["text"] as? String ?? ""),additional:additional):editSnippet(current,source:input,from:from,to:to,text:String(repeating:step["text"] as? String ?? "",count:step["repeat"] as? Int ?? 1))
                    if step["refuse"] as? Bool==true{XCTAssertNil(result,name);continue}
                    let applied=try XCTUnwrap(result);XCTAssertTrue(exactUTF8(applied.session.source,step["after"] as! String),name);source=applied.session.source;selected=applied.selection;next=applied.session
                }else{
                    let result=advanceSnippet(current,source:input,from:from,to:to,direction:action=="next" ? 1:-1)
                    if step["refuse"] as? Bool==true{XCTAssertNil(result,name);continue};let applied=try XCTUnwrap(result);selected=applied.selection;next=applied.session
                }
                session=next;let expected=step["selection"] as! [String:Int];XCTAssertEqual(selected?.from,expected["from"],name);XCTAssertEqual(selected?.to,expected["to"],name)
                if let expected=step["fields"] as? [[String:Any]]{let actual=try XCTUnwrap(next).fields;XCTAssertEqual(actual.count,expected.count,name);for(field,value)in zip(actual,expected){XCTAssertEqual(field.id,value["id"] as? Int,name);XCTAssertEqual(field.parent,value["parent"] as? Int,name);XCTAssertEqual(field.index,value["index"] as? Int,name);XCTAssertEqual(field.from,value["from"] as? Int,name);XCTAssertEqual(field.to,value["to"] as? Int,name)}}
                if step["active"] != nil{XCTAssertEqual(next?.active,step["active"] as? Int,name)}
            }
        }
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SnippetSessionTests.swift"><ui:paste select="#snippet-session-apple-tests"/></ui:file>

## Run the Android field-session contract

Run the same independently specified source, scope and selection outcomes as the other hosts. The tests cover complete linked edits and refuse stale or unsupported intent without mutating session state. They do not stand in for actual editor, IME or accessibility acceptance.

<ui:copy id="snippet-session-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
class SnippetSessionTest {
 private fun fields(values:JSONArray)=(0 until values.length()).map{index->val value=values.getJSONObject(index);val choices=value.getJSONArray("choices");SnippetField(value.getInt("index"),value.getInt("from"),value.getInt("to"),(0 until choices.length()).map{choices.getString(it)},if(value.isNull("id"))null else value.getInt("id"),if(value.isNull("parent"))null else value.getInt("parent"))}
 @Test fun sharedLinkedEditsScopeAndTraversal() {
  val cases=JSONObject(javaClass.classLoader!!.getResource("snippet-session-cases.json")!!.readText()).getJSONArray("cases")
  for(index in 0 until cases.length()){
   val value=cases.getJSONObject(index);val name=value.getString("name");var source=value.getString("source");var session=beginSnippet(source,fields(value.getJSONArray("fields")))
   if(value.optBoolean("invalidBegin")){assertNull(name,session);continue};assertNotNull(name,session)
   val steps=value.getJSONArray("steps");for(offset in 0 until steps.length()){
    val step=steps.getJSONObject(offset);val current=session!!;val input=step.optString("currentSource",source);val from=step.getInt("from");val to=step.getInt("to");val action=step.getString("action")
    val selected:CompletionSelection;val next:SnippetSession?
    if(action=="edit"||action=="complete"||action=="nest"){
     val additions=step.optJSONArray("additional") ?: JSONArray();val additional=(0 until additions.length()).map{val edit=additions.getJSONObject(it);SourceReplacement(edit.getInt("from"),edit.getInt("to"),edit.getString("text"))}
     val result=if(action=="nest")nestSnippet(current,input,SourceReplacement(from,to,step.optString("text","")),additional,fields(step.optJSONArray("inner") ?: JSONArray()))else if(action=="complete")completeSnippet(current,input,SourceReplacement(from,to,step.optString("text","")),additional)else editSnippet(current,input,from,to,step.optString("text","").repeat(step.optInt("repeat",1)))
     if(step.optBoolean("refuse")){assertNull(name,result);continue};assertNotNull(name,result);assertEquals(name,step.getString("after"),result!!.session.source);source=result.session.source;selected=result.selection;next=result.session
    }else{
     val result=advanceSnippet(current,input,from,to,if(action=="next")1 else -1)
     if(step.optBoolean("refuse")){assertNull(name,result);continue};assertNotNull(name,result);selected=result!!.selection;next=result.session
    }
    session=next;val expected=step.getJSONObject("selection");assertEquals(name,expected.getInt("from"),selected.from);assertEquals(name,expected.getInt("to"),selected.to)
    if(step.has("fields"))assertEquals(name,fields(step.getJSONArray("fields")),next!!.fields)
    if(step.has("active"))assertEquals(name,if(step.isNull("active"))null else step.getInt("active"),next?.active)
   }
  }
 }
 @Test fun nestedEmptyChoiceAmplificationIsRefusedBeforeInnerClone(){
  val source="aaaaaaaa";val outer=(0 until 8).map{SnippetField(1,it,it+1,emptyList(),it,null)}+SnippetField(0,8,8,emptyList(),8,null)
  val session=beginSnippet(source,outer)!!;var reads=0
  val emptyChoices=object:AbstractList<String>(){override val size=4096;override fun get(index:Int):String{reads++;return ""}}
  val inner=listOf(SnippetField(1,0,0,emptyChoices,0,null),SnippetField(0,0,0,emptyList(),1,null))
  assertNull(nestSnippet(session,source,SourceReplacement(0,1,""),emptyList(),inner));assertEquals(4096,reads)
  assertNull(beginSnippet("",listOf(SnippetField(1,0,0,List(4097){""},0,null),SnippetField(0,0,0,emptyList(),1,null))))
  assertNotNull(beginSnippet("",listOf(SnippetField(1,0,0,List(4096){""},0,null),SnippetField(0,0,0,emptyList(),1,null))))
 }

}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetSessionTest.kt"><ui:paste select="#snippet-session-android-tests"/></ui:file>


</ui:doc>
