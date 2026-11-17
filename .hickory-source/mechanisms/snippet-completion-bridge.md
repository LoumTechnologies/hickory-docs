<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Transport a resolved snippet plan to every simple editor

For maintainers changing snippet expansion independently of popup rendering. The engine expands the resolved item once in authored coordinates. Hosts validate the versioned payload, exact syntax identity, scalar positions and linked-field contents before preparing any draft. This bridge establishes initial field selection and retains positions; it does not yet establish Tab traversal, mirrored typing or full snippet syntax.

## Validate browser snippet fields

A resolved snippet must match its exact insertion syntax. Reject malformed versions, offsets splitting a scalar, missing or repeated exits and inconsistent mirror contents. Derive the first field from numeric order rather than trusting transport order; retain every occurrence and choice for the later editing session. Optional occurrence identities must be complete and unique, with a contained, acyclic parent forest. Legacy payloads without identities can still expand and select text, but do not establish a field-editing session. This capability writes neither source nor disk.

<ui:copy id="snippet-expansion-browser">
import {sourceChangeOffset} from './ChangePlan';
import {readSnippetTransform,applySnippetTransform,snippetTransformBytes,type SnippetTransform,type SnippetTransformBudget} from './SnippetTransform';
import {completionInsertion,type CompletionExpansion} from './CompletionPlan';
import {utf16ToPosition} from '../lsp/positions';
export interface SnippetField {index:number;from:number;to:number;choices:string[];id?:number;parent?:number|null;transform?:SnippetTransform}
export function completionExpansion(item:unknown):CompletionExpansion|undefined {
 const value=item as {insertTextFormat?:unknown;data?:{hickory_snippet?:unknown}};
 if(value?.insertTextFormat!==2)return undefined;
 const raw=value.data?.hickory_snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|undefined;
 if(!raw||![1,2].includes(raw.version as number)||raw.syntax!==completionInsertion(item)||typeof raw.text!=='string'||new TextEncoder().encode(raw.text).length>1048576||!Array.isArray(raw.fields)||!raw.fields.length||raw.fields.length>4096)throw new Error('Snippet expansion unavailable or out of date. Request completions again.');
 const text=raw.text;let metadataBytes=0;
 const number=(value:unknown)=>{if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw new Error('Malformed snippet field');return value;};
 const fields:SnippetField[]=raw.fields.map(value=>{
  if(!value||typeof value!=='object')throw new Error('Malformed snippet field');
  const field=value as Record<string,unknown>,index=number(field.index),from=number(field.from),to=number(field.to);
  if(index>1004096||from>to||to>text.length||!Array.isArray(field.choices))throw new Error('Malformed snippet field');
  if(field.choices.length>4096||metadataBytes+64+field.choices.length*32>1048576)throw new Error('Snippet metadata expansion limit exceeded');
  metadataBytes+=64;for(const choice of field.choices){if(typeof choice!=='string')throw new Error('Malformed snippet field');metadataBytes+=32+new TextEncoder().encode(choice).length;if(metadataBytes>1048576)throw new Error('Snippet metadata expansion limit exceeded');}
  sourceChangeOffset(text,utf16ToPosition(text,from));sourceChangeOffset(text,utf16ToPosition(text,to));
  const ancestry:Pick<SnippetField,'id'|'parent'>={};
  if(field.id!==undefined){ancestry.id=number(field.id);ancestry.parent=field.parent===null?null:number(field.parent);if(ancestry.id>=4096||ancestry.parent!==null&&ancestry.parent>=4096)throw new Error('Malformed snippet ancestry');}
  else if(field.parent!==undefined)throw new Error('Malformed snippet ancestry');
  const transform=field.transform===undefined||field.transform===null?undefined:readSnippetTransform(field.transform);
  if(transform){if(raw.version!==2||index===0||field.choices.length||ancestry.id===undefined)throw Error('Computed snippet metadata requires protocol 2 and an editable source group');metadataBytes+=snippetTransformBytes(transform);if(metadataBytes>1048576)throw Error('Snippet metadata expansion limit exceeded');}
  return {index,from,to,choices:field.choices as string[],...ancestry,...(transform?{transform}:{})};
 });
 if(fields.some(field=>field.id!==undefined)){
  const ids=new Map(fields.map(field=>[field.id,field]));
  if(ids.has(undefined)||ids.size!==fields.length)throw new Error('Malformed snippet ancestry');
  const visited=new Set<number>();
  for(const field of fields){const chain=new Set<number>();let next:SnippetField|undefined=field;while(next&&!visited.has(next.id!)){if(chain.has(next.id!))throw new Error('Cyclic snippet ancestry');chain.add(next.id!);if(next.parent===null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)throw new Error('Malformed snippet ancestry');next=parent;}for(const id of chain)visited.add(id);}
 }
 if(fields.filter(field=>field.index===0).length!==1)throw new Error('Malformed snippet exit');
 validateSnippetMirrors(text,fields);
 const active=Math.min(...fields.filter(field=>!field.transform&&field.index>0).map(field=>field.index),Infinity),selected=fields.find(field=>!field.transform&&field.index===(active===Infinity?0:active))!;
 return {text,fields,selection:{from:selected.from,to:selected.to}};
}

export function snippetTransformOrder(fields:readonly SnippetField[]):number[] {
 const ids=new Map(fields.map(field=>[field.id,field])),ordinary=new Set(fields.filter(field=>!field.transform).map(field=>field.index)),dependencies=new Map<number,Set<number>>([...ordinary].map(index=>[index,new Set()]));let work=0,edges=0;
 for(const field of fields.filter(field=>field.transform)){
  if(field.index===0||field.id===undefined||field.choices.length||!ordinary.has(field.index)||fields.some(child=>child.parent===field.id))throw Error('Malformed computed snippet occurrence');
  const seen=new Set<number>();let next=field.parent==null?undefined:ids.get(field.parent);
  while(next){if(++work>65536||next.id===undefined||seen.has(next.id))throw Error('Computed snippet ancestry limit exceeded');seen.add(next.id);if(next.index>0){if(next.index===field.index)throw Error('Cyclic computed snippet dependency');const values=dependencies.get(next.index);if(!values)throw Error('Computed snippet parent is not editable');if(!values.has(field.index)){if(++edges>4096)throw Error('Computed snippet dependency limit exceeded');values.add(field.index);}}next=next.parent==null?undefined:ids.get(next.parent);}
 }
 const parents=new Map<number,number[]>();for(const [parent,values] of dependencies)for(const child of values){const list=parents.get(child)??[];list.push(parent);parents.set(child,list);}
 const queue=[...dependencies].filter(([,values])=>!values.size).map(([index])=>index).sort((a,b)=>a-b),order:number[]=[];
 for(let at=0;at<queue.length;at++){const child=queue[at];order.push(child);for(const parent of parents.get(child)??[]){const values=dependencies.get(parent)!;values.delete(child);if(!values.size)queue.push(parent);}}
 if(order.length!==ordinary.size)throw Error('Cyclic computed snippet dependency');return order;
}
export function validateSnippetMirrors(text:string,fields:readonly SnippetField[],budget:SnippetTransformBudget={work:0}):number[] {
 const first=new Map<number,SnippetField>();for(const field of fields.filter(field=>!field.transform)){const previous=first.get(field.index);if(previous&&(text.slice(previous.from,previous.to)!==text.slice(field.from,field.to)||JSON.stringify(previous.choices)!==JSON.stringify(field.choices)))throw Error('Inconsistent snippet mirrors');first.set(field.index,field);}
 const order=snippetTransformOrder(fields);for(const field of fields.filter(field=>field.transform)){const source=first.get(field.index);if(!source||applySnippetTransform(field.transform!,text.slice(source.from,source.to),budget)!==text.slice(field.from,field.to))throw Error('Inconsistent computed snippet occurrence');}return order;
}
</ui:copy>

<ui:file path="apps/web/src/simple/SnippetExpansion.ts"><ui:paste select="#snippet-expansion-browser"/></ui:file>

Browser expansion bounds every choice, including empty alternatives: at most 4096 per field, with 32 bytes of choice overhead plus UTF-8 content charged to the shared 1 MiB metadata budget before copying field choices. Mirrored occurrences each consume their complete logical metadata cost.

## Validate Apple snippet fields

A resolved snippet must match its exact insertion syntax. Reject malformed versions, offsets splitting a scalar, missing or repeated exits and inconsistent mirror contents. Derive the first field from numeric order rather than trusting transport order; retain every occurrence and choice for the later editing session. Optional occurrence identities must be complete and unique, with a contained, acyclic parent forest. Legacy payloads without identities can still expand and select text, but do not establish a field-editing session. This capability writes neither source nor disk.

Reserve the choice-list floor before bridging it into retained Swift strings: at most 4096 alternatives per field, 64 bytes per field and 32 bytes plus UTF-8 content per alternative under 1 MiB. Empty alternatives consume overhead, and each mirrored occurrence pays its complete logical metadata cost.

<ui:copy id="snippet-expansion-apple">
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SnippetExpansion.swift"><ui:paste select="#snippet-expansion-apple"/></ui:file>

## Validate Android snippet fields

A resolved snippet must match its exact insertion syntax. Reject malformed versions, offsets splitting a scalar, missing or repeated exits and inconsistent mirror contents. Derive the first field from numeric order rather than trusting transport order; retain every occurrence and choice for the later editing session. Optional occurrence identities must be complete and unique, with a contained, acyclic parent forest. Legacy payloads without identities can still expand and select text, but do not establish a field-editing session. This capability writes neither source nor disk.

<ui:copy id="snippet-expansion-android">
package com.hickorydocs.simple
import org.json.JSONObject
data class SnippetField(val index:Int,val from:Int,val to:Int,val choices:List<String>,val id:Int?=null,val parent:Int?=null,val transform:SnippetTransform?=null)
fun completionExpansion(item:JSONObject):CompletionExpansion? {
 if((item.opt("insertTextFormat") as? Number)?.toDouble()!=2.0)return null
 val raw=item.optJSONObject("data")?.optJSONObject("hickory_snippet") ?: error("Snippet expansion unavailable or out of date. Request completions again.")
 fun number(value:Any?):Int {check(value is Number&&value.toDouble().isFinite()&&value.toDouble()==value.toInt().toDouble()&&value.toInt()>=0){"Malformed snippet field"};return value.toInt()}
 val version=number(raw.opt("version"));check(version in listOf(1,2)&&raw.opt("syntax")==completionInsertion(item)){"Snippet expansion unavailable or out of date. Request completions again."}
 val text=raw.opt("text") as? String ?: error("Malformed snippet text");check(text.toByteArray(Charsets.UTF_8).size<=1048576){"Snippet expansion limit exceeded"}
 val values=raw.optJSONArray("fields") ?: error("Malformed snippet fields");check(values.length() in 1..4096){"Malformed snippet fields"}
 var metadataBytes=0L
 val fields=(0 until values.length()).map {offset->
  val value=values.optJSONObject(offset) ?: error("Malformed snippet field");val index=number(value.opt("index"));val from=number(value.opt("from"));val to=number(value.opt("to"));check(index<=1004096&&from<=to&&to<=text.length){"Malformed snippet field"}
  val choices=value.optJSONArray("choices") ?: error("Malformed snippet choices")
  check(choices.length()<=4096){"Snippet choice count limit exceeded"}
  check(64L+32L*choices.length()<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"}
  metadataBytes+=64L+32L*choices.length()
  for(choice in 0 until choices.length()){val option=choices.opt(choice) as? String ?: error("Malformed snippet choices");check(option.length.toLong()<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"};val bytes=option.toByteArray(Charsets.UTF_8).size.toLong();check(bytes<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"};metadataBytes+=bytes}
  // Materialize a separate retained vector only after the whole field's quota
  // has been admitted, including empty alternatives and their slots.
  val options=(0 until choices.length()).map{choices.getString(it)}
  sourceChangeOffset(text,languagePosition(text,from));sourceChangeOffset(text,languagePosition(text,to))
  val id=if(value.has("id"))number(value.opt("id")) else null;val parent=if(value.has("id")&&!value.isNull("parent"))number(value.opt("parent")) else null
  check((id==null&&!value.has("parent")||id!=null&&id<4096&&value.has("parent"))&&(parent==null||parent<4096)){"Malformed snippet ancestry"}
  val transform=if(value.has("transform")&&!value.isNull("transform")){check(version==2&&index>0&&options.isEmpty()&&id!=null){"Computed snippet metadata requires protocol 2 and an editable source group"};readSnippetTransform(value.opt("transform"))}else null
  if(transform!=null){check(transform.bytes.toLong()<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"};metadataBytes+=transform.bytes}
  SnippetField(index,from,to,options,id,parent,transform)
 }
 if(fields.any{it.id!=null}){
  val ids=fields.associateBy{it.id};check(!ids.containsKey(null)&&ids.size==fields.size){"Malformed snippet ancestry"};val visited=mutableSetOf<Int>()
  for(field in fields){val chain=mutableSetOf<Int>();var next:SnippetField?=field
   while(next!=null&&!visited.contains(next.id!!)){val current=next;check(chain.add(current.id!!)){"Cyclic snippet ancestry"};val identity=current.parent ?: break;val parent=ids[identity] ?: error("Malformed snippet ancestry");check(parent.from<=current.from&&parent.to>=current.to){"Malformed snippet ancestry"};next=parent};visited.addAll(chain)
  }
 }
 check(fields.count{it.index==0}==1){"Malformed snippet exit"}
 validateSnippetMirrors(text,fields)
 val active=fields.filter{it.transform==null&&it.index>0}.map{it.index}.minOrNull() ?: 0;val selected=fields.first{it.transform==null&&it.index==active}
 return CompletionExpansion(text,CompletionSelection(selected.from,selected.to),fields)
}

fun snippetTransformOrder(fields:List<SnippetField>):List<Int> {
 val ids=fields.filter{it.id!=null}.associateBy{it.id!!};val ordinary=fields.filter{it.transform==null}.map{it.index}.toSet();val dependencies=ordinary.associateWith{mutableSetOf<Int>()};var work=0;var edges=0
 for(field in fields.filter{it.transform!=null}){
  check(field.index>0&&field.id!=null&&field.choices.isEmpty()&&field.index in ordinary&&fields.none{it.parent==field.id}){"Malformed computed snippet occurrence"}
  val seen=mutableSetOf<Int>();var next=field.parent?.let{ids[it]}
  while(next!=null){val value=next;work++;check(work<=65536&&value.id!=null&&seen.add(value.id)){"Computed snippet ancestry limit exceeded"};if(value.index>0){check(value.index!=field.index){"Cyclic computed snippet dependency"};val values=dependencies[value.index] ?: error("Computed snippet parent is not editable");if(values.add(field.index)){edges++;check(edges<=4096){"Computed snippet dependency limit exceeded"}}};next=value.parent?.let{ids[it]}}
 }
 val parents=mutableMapOf<Int,MutableList<Int>>();for(parent in dependencies.keys.sorted())for(child in dependencies.getValue(parent).sorted())parents.getOrPut(child){mutableListOf()}.add(parent)
 val queue=dependencies.keys.filter{dependencies.getValue(it).isEmpty()}.sorted().toMutableList();val order=mutableListOf<Int>();var at=0
 while(at<queue.size){val child=queue[at++];order+=child;for(parent in parents[child].orEmpty()){val values=dependencies.getValue(parent);values.remove(child);if(values.isEmpty())queue+=parent}}
 check(order.size==ordinary.size){"Cyclic computed snippet dependency"};return order
}
fun validateSnippetMirrors(text:String,fields:List<SnippetField>):List<Int> = validateSnippetMirrors(text,fields,SnippetTransformBudget())
internal fun validateSnippetMirrors(text:String,fields:List<SnippetField>,budget:SnippetTransformBudget):List<Int> {
 val first=mutableMapOf<Int,SnippetField>();for(field in fields.filter{it.transform==null}){val previous=first[field.index];check(previous==null||text.substring(previous.from,previous.to)==text.substring(field.from,field.to)&&previous.choices==field.choices){"Inconsistent snippet mirrors"};first[field.index]=field}
 val order=snippetTransformOrder(fields);for(field in fields.filter{it.transform!=null}){val value=first[field.index] ?: error("Computed snippet source is unavailable");check(applySnippetTransform(field.transform!!,text.substring(value.from,value.to),budget)==text.substring(field.from,field.to)){"Inconsistent computed snippet occurrence"}};return order
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SnippetExpansion.kt"><ui:paste select="#snippet-expansion-android"/></ui:file>

## Specify authoritative wire and import outcomes

These shared vectors independently state the complete expanded text, initial field selection and final source-relative positions after imports. Malformed transport and scalar boundaries, stale syntax identity, incompatible mirrors and invalid exits refuse the whole proposal.

<ui:copy id="snippet-expansion-shared-vectors">
{
  "cases": [
    {
      "name": "mirrors and exit follow imports in UTF16",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "expected": {
        "text": "😀 + 😀",
        "selection": {
          "from": 0,
          "to": 2
        },
        "after": "#\n😀 + 😀",
        "fields": [
          {
            "index": 1,
            "from": 0,
            "to": 2,
            "choices": [],
            "sourceFrom": 2,
            "sourceTo": 4
          },
          {
            "index": 1,
            "from": 5,
            "to": 7,
            "choices": [],
            "sourceFrom": 7,
            "sourceTo": 9
          },
          {
            "index": 0,
            "from": 7,
            "to": 7,
            "choices": [],
            "sourceFrom": 9,
            "sourceTo": 9
          }
        ]
      }
    },
    {
      "name": "different exact syntax",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "other",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "boolean version",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": true,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "boolean field index",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": true,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "fractional field offset",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 1.5,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "split surrogate",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 1,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "negative range",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": -1,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "outside text",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 100,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "backwards range",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 2,
                "to": 0,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "different mirrored text",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + hi",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "different mirrored choices",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [
                  "x"
                ]
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "no exit",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "repeated exit",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "nonstring choice",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [
                  true
                ]
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "empty fields",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": []
          }
        }
      },
      "error": true
    },
    {
      "name": "unsupported version",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 3,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": []
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": []
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "absent expansion",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ]
      },
      "error": true
    },
    {
      "name": "plain item needs no expansion",
      "item": {
        "label": "plain",
        "insertTextFormat": 1
      },
      "plain": true
    },
    {
      "name": "explicit occurrence forest survives import mapping",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": null
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "expected": {
        "text": "😀 + 😀",
        "selection": {
          "from": 0,
          "to": 2
        },
        "after": "#\n😀 + 😀",
        "fields": [
          {
            "index": 1,
            "from": 0,
            "to": 2,
            "choices": [],
            "sourceFrom": 2,
            "sourceTo": 4,
            "id": 0,
            "parent": null
          },
          {
            "index": 1,
            "from": 5,
            "to": 7,
            "choices": [],
            "sourceFrom": 7,
            "sourceTo": 9,
            "id": 1,
            "parent": null
          },
          {
            "index": 0,
            "from": 7,
            "to": 7,
            "choices": [],
            "sourceFrom": 9,
            "sourceTo": 9,
            "id": 2,
            "parent": null
          }
        ]
      }
    },
    {
      "name": "duplicate occurrence id",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": null
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 0,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "missing occurrence parent",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "nonexistent occurrence parent",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": 99
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "parent does not contain child",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": 1
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "self ancestry cycle",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": 0
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "partially identified forest",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": null
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": []
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "boolean parent identity",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 0,
                "parent": true
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "oversized occurrence identity",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:😀} + $1$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:😀} + $1$0",
            "text": "😀 + 😀",
            "fields": [
              {
                "index": 1,
                "from": 0,
                "to": 2,
                "choices": [],
                "id": 4096,
                "parent": null
              },
              {
                "index": 1,
                "from": 5,
                "to": 7,
                "choices": [],
                "id": 1,
                "parent": null
              },
              {
                "index": 0,
                "from": 7,
                "to": 7,
                "choices": [],
                "id": 2,
                "parent": null
              }
            ]
          }
        }
      },
      "error": true
    },
    {
      "name": "nested and adjacent empty fields retain distinct parents",
      "item": {
        "label": "snippet",
        "insertTextFormat": 2,
        "textEdit": {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "${1:${2:}}$3$0"
        },
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            },
            "newText": "#\n"
          }
        ],
        "data": {
          "hickory_snippet": {
            "version": 1,
            "syntax": "${1:${2:}}$3$0",
            "text": "",
            "fields": [
              {
                "index": 2,
                "id": 1,
                "parent": 0,
                "from": 0,
                "to": 0,
                "choices": []
              },
              {
                "index": 1,
                "id": 0,
                "parent": null,
                "from": 0,
                "to": 0,
                "choices": []
              },
              {
                "index": 3,
                "id": 2,
                "parent": null,
                "from": 0,
                "to": 0,
                "choices": []
              },
              {
                "index": 0,
                "id": 3,
                "parent": null,
                "from": 0,
                "to": 0,
                "choices": []
              }
            ]
          }
        }
      },
      "expected": {
        "text": "",
        "selection": {
          "from": 0,
          "to": 0
        },
        "after": "#\n",
        "fields": [
          {
            "index": 2,
            "id": 1,
            "parent": 0,
            "from": 0,
            "to": 0,
            "choices": [],
            "sourceFrom": 2,
            "sourceTo": 2
          },
          {
            "index": 1,
            "id": 0,
            "parent": null,
            "from": 0,
            "to": 0,
            "choices": [],
            "sourceFrom": 2,
            "sourceTo": 2
          },
          {
            "index": 3,
            "id": 2,
            "parent": null,
            "from": 0,
            "to": 0,
            "choices": [],
            "sourceFrom": 2,
            "sourceTo": 2
          },
          {
            "index": 0,
            "id": 3,
            "parent": null,
            "from": 0,
            "to": 0,
            "choices": [],
            "sourceFrom": 2,
            "sourceTo": 2
          }
        ]
      }
    }
  ]
}
</ui:copy>

<ui:file path="apps/simple-contract/snippet-expansion-cases.json"><ui:paste select="#snippet-expansion-shared-vectors"/></ui:file>

## Run browser expansion vectors

Use the same transport and source-position cases as the native hosts; test the complete planner after decoding rather than a second copy of the decoder.

<ui:copy id="snippet-expansion-browser-tests">
import {expect,it} from 'vitest';
import vectors from '../../../simple-contract/snippet-expansion-cases.json';
import {completionExpansion} from './SnippetExpansion';
import {planCompletion} from './CompletionPlan';
it('validates shared snippet payloads and maps fields through imports',()=>{
 for(const value of vectors.cases){
  const expand=()=>completionExpansion(value.item);
  if('error' in value){expect(expand,value.name).toThrow();continue;}
  if('plain' in value){expect(expand(),value.name).toBeUndefined();continue;}
  const expansion=expand()!,expected=value.expected!;expect(expansion.text,value.name).toBe(expected.text);expect(expansion.selection,value.name).toEqual(expected.selection);
  const draft=planCompletion('hick:///a.py',value.item,{text:'va',writable:true},0,2,expansion);
  expect(draft.plan.patches[0].after,value.name).toBe(expected.after);for(const [index,field] of expected.fields.entries()){if('id' in field&&'parent' in field){expect(draft.fields[index].id,value.name).toBe(field.id);expect(draft.fields[index].parent,value.name).toBe(field.parent);}}expect(draft.fields.map(field=>({index:field.index,from:field.from,to:field.to})),value.name).toEqual(expected.fields.map(field=>({index:field.index,from:field.sourceFrom,to:field.sourceTo})));
 }
});
it('refuses empty text that amplifies choice metadata beyond the shared budget',()=>{
 const alternative='x'.repeat(1024),syntax='${1|,'+alternative+'|}'+'$1'.repeat(1029)+'$0';
 const fields=Array.from({length:1030},(_,id)=>({id,parent:null,index:1,from:0,to:0,choices:['',alternative]}));fields.push({id:1030,parent:null,index:0,from:0,to:0,choices:[]});
 expect(()=>completionExpansion({label:'large choices',insertTextFormat:2,insertText:syntax,data:{hickory_snippet:{version:1,syntax,text:'',fields}}})).toThrow(/metadata/);
});
it('counts every empty alternative and refuses cumulative mirror metadata before expansion',()=>{
 const item=(count:number,mirrors=1)=>{const choices=Array<string>(count).fill(''),syntax='${1|'+','.repeat(count-1)+'|}'+'$1'.repeat(mirrors-1)+'$0',fields=Array.from({length:mirrors},(_,id)=>({id,parent:null,index:1,from:0,to:0,choices}));fields.push({id:mirrors,parent:null,index:0,from:0,to:0,choices:[]});return {label:'empty alternatives',insertTextFormat:2,insertText:syntax,data:{hickory_snippet:{version:1,syntax,text:'',fields}}};};
 expect(completionExpansion(item(4096))?.fields?.[0].choices).toHaveLength(4096);
 expect(()=>completionExpansion(item(4097))).toThrow(/metadata/);
 expect(()=>completionExpansion(item(4096,8))).toThrow(/metadata/);
});
</ui:copy>

<ui:file path="apps/web/src/simple/SnippetExpansion.test.ts"><ui:paste select="#snippet-expansion-browser-tests"/></ui:file>

## Run Apple expansion vectors

Run the shared transport, Unicode and import-position vectors through the production Apple decoder and source planner. Exact text comparison retains byte distinctions.

<ui:copy id="snippet-expansion-apple-tests">
import XCTest
@testable import HickorySimple
final class SnippetExpansionTests:XCTestCase {
    func testSharedWireAndImportPositions()throws {
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-expansion-cases.json"))) as! [String:Any]
        for value in vectors["cases"] as! [[String:Any]] {
            let name=value["name"] as! String,item=value["item"] as! [String:Any]
            if value["error"] as? Bool==true{XCTAssertThrowsError(try completionExpansion(item),name);continue}
            if value["plain"] as? Bool==true{XCTAssertNil(try completionExpansion(item),name);continue}
            let expansion=try XCTUnwrap(try completionExpansion(item)),expected=value["expected"] as! [String:Any],selection=expected["selection"] as! [String:Int]
            XCTAssertTrue(exactUTF8(expansion.text,expected["text"] as! String),name);XCTAssertEqual(expansion.selection?.from,selection["from"],name);XCTAssertEqual(expansion.selection?.to,selection["to"],name)
            let draft=try planCompletion("hick:///a.py",item:item,snapshot:SourceSnapshot(text:"va",writable:true),from:0,to:2,expansion:expansion)
            XCTAssertTrue(exactUTF8(draft.plan.patches[0].after,expected["after"] as! String),name)
            let fields=expected["fields"] as! [[String:Any]];XCTAssertEqual(draft.fields.count,fields.count,name)
            for (actual,field) in zip(draft.fields,fields){XCTAssertEqual(actual.index,field["index"] as? Int,name);XCTAssertEqual(actual.from,field["sourceFrom"] as? Int,name);XCTAssertEqual(actual.to,field["sourceTo"] as? Int,name);if field["id"] != nil{XCTAssertEqual(actual.id,field["id"] as? Int,name);XCTAssertEqual(actual.parent,field["parent"] as? Int,name)}}
        }
    }
    func testChoiceMetadataAmplificationRefusesBeforePreparingSource()throws {
        let alternative=String(repeating:"x",count:1024),syntax="${1|,"+String(repeating:"x",count:1024)+"|}"+String(repeating:"$1",count:1029)+"$0"
        var fields:[[String:Any]]=(0..<1030).map{["id":$0,"parent":NSNull(),"index":1,"from":0,"to":0,"choices":["",alternative]]}
        fields.append(["id":1030,"parent":NSNull(),"index":0,"from":0,"to":0,"choices":[]])
        XCTAssertThrowsError(try completionExpansion(["label":"large choices","insertTextFormat":2,"insertText":syntax,"data":["hickory_snippet":["version":1,"syntax":syntax,"text":"","fields":fields]]]))
    }
    func testEmptyChoicesReserveOverheadBeforeCopyingMetadata()throws {
        func item(_ count:Int,_ mirrors:Int=1)->[String:Any] {
            let choices=Array(repeating:"",count:count),syntax="${1|"+String(repeating:",",count:count-1)+"|}"+String(repeating:"$1",count:mirrors-1)+"$0"
            var fields:[[String:Any]]=(0..<mirrors).map{["id":$0,"parent":NSNull(),"index":1,"from":0,"to":0,"choices":choices]}
            fields.append(["id":mirrors,"parent":NSNull(),"index":0,"from":0,"to":0,"choices":[]])
            return ["label":"empty alternatives","insertTextFormat":2,"insertText":syntax,"data":["hickory_snippet":["version":1,"syntax":syntax,"text":"","fields":fields]]]
        }
        XCTAssertEqual(try completionExpansion(item(4096))?.fields.first?.choices.count,4096)
        XCTAssertThrowsError(try completionExpansion(item(4097)))
        XCTAssertThrowsError(try completionExpansion(item(4096,8)))
    }

}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SnippetExpansionTests.swift"><ui:paste select="#snippet-expansion-apple-tests"/></ui:file>

## Run Android expansion vectors

The actual Android decoder and planner consume the same vectors and assert final field positions after imports, independently of any popup or renderer.

<ui:copy id="snippet-expansion-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class SnippetExpansionTest {
 @Test fun sharedWireAndImportPositions() {
  val cases=JSONObject(javaClass.classLoader!!.getResource("snippet-expansion-cases.json")!!.readText()).getJSONArray("cases")
  for(index in 0 until cases.length()) {
   val value=cases.getJSONObject(index);val name=value.getString("name");val item=value.getJSONObject("item");val result=runCatching{completionExpansion(item)}
   if(value.optBoolean("error")){assertTrue(name,result.isFailure);continue}
   if(value.optBoolean("plain")){assertNull(name,result.getOrThrow());continue}
   val expansion=result.getOrThrow()!!;val expected=value.getJSONObject("expected");val selection=expected.getJSONObject("selection");assertEquals(name,expected.getString("text"),expansion.text);assertEquals(name,selection.getInt("from"),expansion.selection!!.from);assertEquals(name,selection.getInt("to"),expansion.selection.to)
   val draft=planCompletion("hick:///a.py",item,SourceSnapshot("va",true),0,2,expansion);assertEquals(name,expected.getString("after"),draft.plan.patches[0].after)
   val fields=expected.getJSONArray("fields");assertEquals(name,fields.length(),draft.fields.size)
   draft.fields.forEachIndexed {i,field->val expectedField=fields.getJSONObject(i);assertEquals(name,expectedField.getInt("index"),field.index);assertEquals(name,expectedField.getInt("sourceFrom"),field.from);assertEquals(name,expectedField.getInt("sourceTo"),field.to);if(expectedField.has("id")){assertEquals(name,expectedField.getInt("id"),field.id);assertEquals(name,if(expectedField.isNull("parent"))null else expectedField.getInt("parent"),field.parent)}}
  }
 }
 @Test fun choiceMetadataAmplificationRefusesBeforePreparingSource(){
  val alternative="x".repeat(1024);val syntax="$"+"{1|,"+alternative+"|}"+"$1".repeat(1029)+"$0";val fields=org.json.JSONArray()
  for(id in 0 until 1030)fields.put(JSONObject().put("id",id).put("parent",JSONObject.NULL).put("index",1).put("from",0).put("to",0).put("choices",org.json.JSONArray().put("").put(alternative)))
  fields.put(JSONObject().put("id",1030).put("parent",JSONObject.NULL).put("index",0).put("from",0).put("to",0).put("choices",org.json.JSONArray()))
  val item=JSONObject().put("label","large choices").put("insertTextFormat",2).put("insertText",syntax).put("data",JSONObject().put("hickory_snippet",JSONObject().put("version",1).put("syntax",syntax).put("text","").put("fields",fields)))
  assertTrue(runCatching{completionExpansion(item)}.isFailure)
 }

 @Test fun emptyAlternativesHaveCountAndSlotBudgetsBeforeCopy(){
  fun item(count:Int,mirrors:Int):JSONObject {
   val alternatives=org.json.JSONArray();repeat(count){alternatives.put("")};val fields=org.json.JSONArray()
   repeat(mirrors){fields.put(JSONObject().put("id",it).put("parent",JSONObject.NULL).put("index",1).put("from",0).put("to",0).put("choices",alternatives))}
   fields.put(JSONObject().put("id",mirrors).put("parent",JSONObject.NULL).put("index",0).put("from",0).put("to",0).put("choices",org.json.JSONArray()))
   return JSONObject().put("label","empty choices").put("insertTextFormat",2).put("insertText","syntax").put("data",JSONObject().put("hickory_snippet",JSONObject().put("version",1).put("syntax","syntax").put("text","").put("fields",fields)))
  }
  assertEquals(4096,completionExpansion(item(4096,1))!!.fields.first().choices.size)
  assertTrue(runCatching{completionExpansion(item(4097,1))}.isFailure)
  assertTrue(runCatching{completionExpansion(item(4096,8))}.isFailure)
 }

}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetExpansionTest.kt"><ui:paste select="#snippet-expansion-android-tests"/></ui:file>

</ui:doc>
