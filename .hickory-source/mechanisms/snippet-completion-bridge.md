<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Transport a resolved snippet plan to every simple editor

For maintainers changing snippet expansion independently of popup rendering. The engine expands the resolved item once in authored coordinates. Hosts validate the versioned payload, exact syntax identity, scalar positions and linked-field contents before preparing any draft. This bridge establishes initial field selection and retains positions; it does not yet establish Tab traversal, mirrored typing or full snippet syntax.

## Validate browser snippet fields

A resolved snippet must match its exact insertion syntax. Reject malformed versions, offsets splitting a scalar, missing or repeated exits and inconsistent mirror contents. Derive the first field from numeric order rather than trusting transport order; retain every occurrence and choice for the later editing session. Optional occurrence identities must be complete and unique, with a contained, acyclic parent forest. Legacy payloads without identities can still expand and select text, but do not establish a field-editing session. This capability writes neither source nor disk.

<ui:copy id="snippet-expansion-browser">
import {sourceChangeOffset} from './ChangePlan';
import {completionInsertion,type CompletionExpansion} from './CompletionPlan';
import {utf16ToPosition} from '../lsp/positions';
export interface SnippetField {index:number;from:number;to:number;choices:string[];id?:number;parent?:number|null}
export function completionExpansion(item:unknown):CompletionExpansion|undefined {
 const value=item as {insertTextFormat?:unknown;data?:{hickory_snippet?:unknown}};
 if(value?.insertTextFormat!==2)return undefined;
 const raw=value.data?.hickory_snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|undefined;
 if(!raw||raw.version!==1||raw.syntax!==completionInsertion(item)||typeof raw.text!=='string'||raw.text.length>1048576||!Array.isArray(raw.fields)||!raw.fields.length||raw.fields.length>4096)throw new Error('Snippet expansion unavailable or out of date. Request completions again.');
 const text=raw.text;
 const number=(value:unknown)=>{if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw new Error('Malformed snippet field');return value;};
 const fields:SnippetField[]=raw.fields.map(value=>{
  if(!value||typeof value!=='object')throw new Error('Malformed snippet field');
  const field=value as Record<string,unknown>,index=number(field.index),from=number(field.from),to=number(field.to);
  if(index>1004096||from>to||to>text.length||!Array.isArray(field.choices)||field.choices.some(value=>typeof value!=='string'))throw new Error('Malformed snippet field');
  sourceChangeOffset(text,utf16ToPosition(text,from));sourceChangeOffset(text,utf16ToPosition(text,to));
  const ancestry:Pick<SnippetField,'id'|'parent'>={};
  if(field.id!==undefined){ancestry.id=number(field.id);ancestry.parent=field.parent===null?null:number(field.parent);if(ancestry.id>=4096||ancestry.parent!==null&&ancestry.parent>=4096)throw new Error('Malformed snippet ancestry');}
  else if(field.parent!==undefined)throw new Error('Malformed snippet ancestry');
  return {index,from,to,choices:field.choices as string[],...ancestry};
 });
 if(fields.some(field=>field.id!==undefined)){
  const ids=new Map(fields.map(field=>[field.id,field]));
  if(ids.has(undefined)||ids.size!==fields.length)throw new Error('Malformed snippet ancestry');
  const visited=new Set<number>();
  for(const field of fields){const chain=new Set<number>();let next:SnippetField|undefined=field;while(next&&!visited.has(next.id!)){if(chain.has(next.id!))throw new Error('Cyclic snippet ancestry');chain.add(next.id!);if(next.parent===null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)throw new Error('Malformed snippet ancestry');next=parent;}for(const id of chain)visited.add(id);}
 }
 if(fields.filter(field=>field.index===0).length!==1)throw new Error('Malformed snippet exit');
 const first=new Map<number,SnippetField>();
 for(const field of fields){const previous=first.get(field.index);if(previous&&(text.slice(previous.from,previous.to)!==text.slice(field.from,field.to)||JSON.stringify(previous.choices)!==JSON.stringify(field.choices)))throw new Error('Inconsistent snippet mirrors');first.set(field.index,field);}
 const active=[...first.keys()].filter(index=>index>0).sort((a,b)=>a-b)[0]??0,selected=fields.find(field=>field.index===active)!;
 return {text,fields,selection:{from:selected.from,to:selected.to}};
}
</ui:copy>

<ui:file path="apps/web/src/simple/SnippetExpansion.ts"><ui:paste select="#snippet-expansion-browser"/></ui:file>

## Validate Apple snippet fields

A resolved snippet must match its exact insertion syntax. Reject malformed versions, offsets splitting a scalar, missing or repeated exits and inconsistent mirror contents. Derive the first field from numeric order rather than trusting transport order; retain every occurrence and choice for the later editing session. Optional occurrence identities must be complete and unique, with a contained, acyclic parent forest. Legacy payloads without identities can still expand and select text, but do not establish a field-editing session. This capability writes neither source nor disk.

<ui:copy id="snippet-expansion-apple">
import Foundation
struct SnippetField {var index:Int;var from:Int;var to:Int;var choices:[String];var id:Int?=nil;var parent:Int?=nil}
func completionExpansion(_ item:[String:Any])throws->CompletionExpansion? {
    guard sourceChangeInteger(item["insertTextFormat"])==2 else{return nil}
    guard let raw=(item["data"] as? [String:Any])?["hickory_snippet"] as? [String:Any],sourceChangeInteger(raw["version"])==1,let syntax=raw["syntax"] as? String,exactUTF8(syntax,try completionInsertion(item)),let text=raw["text"] as? String,text.utf16.count<=1048576,let values=raw["fields"] as? [[String:Any]],!values.isEmpty,values.count<=4096 else{throw EngineError(message:"Snippet expansion unavailable or out of date. Request completions again.")}
    var fields:[SnippetField]=[]
    for value in values {
        guard let index=sourceChangeInteger(value["index"]),index>=0,index<=1004096,let from=sourceChangeInteger(value["from"]),let to=sourceChangeInteger(value["to"]),from>=0,from<=to,to<=text.utf16.count,let choices=value["choices"] as? [String] else{throw EngineError(message:"Malformed snippet field")}
        _ = try sourceChangeOffset(text,languagePosition(text,from));_ = try sourceChangeOffset(text,languagePosition(text,to))
        var id:Int?=nil,parent:Int?=nil
        if value["id"] != nil {
            guard let identity=sourceChangeInteger(value["id"]),identity>=0,identity<4096 else{throw EngineError(message:"Malformed snippet ancestry")};id=identity
            if !(value["parent"] is NSNull){guard let identity=sourceChangeInteger(value["parent"]),identity>=0,identity<4096 else{throw EngineError(message:"Malformed snippet ancestry")};parent=identity}
        }else if value["parent"] != nil{throw EngineError(message:"Malformed snippet ancestry")}
        fields.append(SnippetField(index:index,from:from,to:to,choices:choices,id:id,parent:parent))
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SnippetExpansion.swift"><ui:paste select="#snippet-expansion-apple"/></ui:file>

## Validate Android snippet fields

A resolved snippet must match its exact insertion syntax. Reject malformed versions, offsets splitting a scalar, missing or repeated exits and inconsistent mirror contents. Derive the first field from numeric order rather than trusting transport order; retain every occurrence and choice for the later editing session. Optional occurrence identities must be complete and unique, with a contained, acyclic parent forest. Legacy payloads without identities can still expand and select text, but do not establish a field-editing session. This capability writes neither source nor disk.

<ui:copy id="snippet-expansion-android">
package com.hickorydocs.simple
import org.json.JSONObject
data class SnippetField(val index:Int,val from:Int,val to:Int,val choices:List<String>,val id:Int?=null,val parent:Int?=null)
fun completionExpansion(item:JSONObject):CompletionExpansion? {
 if((item.opt("insertTextFormat") as? Number)?.toDouble()!=2.0)return null
 val raw=item.optJSONObject("data")?.optJSONObject("hickory_snippet") ?: error("Snippet expansion unavailable or out of date. Request completions again.")
 fun number(value:Any?):Int {check(value is Number&&value.toDouble().isFinite()&&value.toDouble()==value.toInt().toDouble()&&value.toInt()>=0){"Malformed snippet field"};return value.toInt()}
 check(number(raw.opt("version"))==1&&raw.opt("syntax")==completionInsertion(item)){"Snippet expansion unavailable or out of date. Request completions again."}
 val text=raw.opt("text") as? String ?: error("Malformed snippet text");check(text.length<=1048576){"Snippet expansion limit exceeded"}
 val values=raw.optJSONArray("fields") ?: error("Malformed snippet fields");check(values.length() in 1..4096){"Malformed snippet fields"}
 val fields=(0 until values.length()).map {offset->
  val value=values.optJSONObject(offset) ?: error("Malformed snippet field");val index=number(value.opt("index"));val from=number(value.opt("from"));val to=number(value.opt("to"));check(index<=1004096&&from<=to&&to<=text.length){"Malformed snippet field"}
  val choices=value.optJSONArray("choices") ?: error("Malformed snippet choices");val options=(0 until choices.length()).map{choices.opt(it) as? String ?: error("Malformed snippet choices")}
  sourceChangeOffset(text,languagePosition(text,from));sourceChangeOffset(text,languagePosition(text,to))
  val id=if(value.has("id"))number(value.opt("id")) else null;val parent=if(value.has("id")&&!value.isNull("parent"))number(value.opt("parent")) else null
  check((id==null&&!value.has("parent")||id!=null&&id<4096&&value.has("parent"))&&(parent==null||parent<4096)){"Malformed snippet ancestry"}
  SnippetField(index,from,to,options,id,parent)
 }
 if(fields.any{it.id!=null}){
  val ids=fields.associateBy{it.id};check(!ids.containsKey(null)&&ids.size==fields.size){"Malformed snippet ancestry"};val visited=mutableSetOf<Int>()
  for(field in fields){val chain=mutableSetOf<Int>();var next:SnippetField?=field
   while(next!=null&&!visited.contains(next.id!!)){val current=next;check(chain.add(current.id!!)){"Cyclic snippet ancestry"};val identity=current.parent ?: break;val parent=ids[identity] ?: error("Malformed snippet ancestry");check(parent.from<=current.from&&parent.to>=current.to){"Malformed snippet ancestry"};next=parent};visited.addAll(chain)
  }
 }
 check(fields.count{it.index==0}==1){"Malformed snippet exit"}
 val first=mutableMapOf<Int,SnippetField>();for(field in fields){val previous=first[field.index];check(previous==null||text.substring(previous.from,previous.to)==text.substring(field.from,field.to)&&previous.choices==field.choices){"Inconsistent snippet mirrors"};first[field.index]=field}
 val active=first.keys.filter{it>0}.minOrNull() ?: 0;val selected=fields.first{it.index==active}
 return CompletionExpansion(text,CompletionSelection(selected.from,selected.to),fields)
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
            "version": 2,
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
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetExpansionTest.kt"><ui:paste select="#snippet-expansion-android-tests"/></ui:file>

</ui:doc>
