<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Derive a complete completion draft before accepting it

For maintainers changing completion acceptance independently of popup rendering.

A completion is a primary replacement plus a complete set of import edits and a resulting source selection. Source ownership, scalar boundaries and overlap are preconditions, not optional presentation details. A replaceable snippet expander supplies plain text and an initial selection; this planner never inserts literal snippet syntax when that capability is absent. A no-op completion may move the caret while retaining the captured source for an authoritative host check. Nothing here applies, saves or verifies source.

## Browser completion draft

Prepare the complete source change before touching a view. A completion explicitly names its primary edit, so additional insertions at its start precede that insertion in declared provider order. Deduplicate identical imports, validate all other replacements through the shared source planner, and map the final selection through earlier edits. Commands and unavailable snippet expansion refuse the whole proposal. The retained before snapshot also guards selection-only acceptance.

<ui:copy id="completion-draft-browser">
import {planWorkspaceEdit,sourceChangeOffset,type ChangePlan,type SourceReplacement,type SourceSnapshot} from './ChangePlan';
import {semanticPath} from './Intelligence';
import {utf16ToPosition} from '../lsp/positions';
export interface CompletionExpansion {text:string;selection?:{from:number;to:number}}
export interface CompletionDraft {path:string;before:string;plan:ChangePlan;insertionFrom:number;selection:{from:number;to:number}}
export function completionInsertion(item:unknown):string {
 const value=item as Record<string,unknown>|null;if(!value||typeof value!=='object'||typeof value.label!=='string')throw new Error('Malformed completion');
 const edit=value.textEdit as Record<string,unknown>|undefined;
 if(edit!==undefined&&(!edit||typeof edit!=="object"||Array.isArray(edit)))throw new Error("Malformed completion edit");
 const text=edit?edit.newText:value.insertText??value.label;
 if(typeof text!=='string')throw new Error('Missing completion insertion text');return text;
}
export function planCompletion(uri:string,item:unknown,snapshot:SourceSnapshot,from:number,to:number,expansion?:CompletionExpansion):CompletionDraft {
 const source=snapshot.text;if(!snapshot.writable)throw new Error("Completion source is generated, read-only or saving");
 const value=item as Record<string,unknown>|null,path=semanticPath(uri);if(!path||!value||typeof value!=='object')throw new Error('Unsupported completion source identity');
 if(value.command!==undefined&&value.command!==null)throw new Error('This completion requires a provider command; no edits were applied.');
 const format=value.insertTextFormat===undefined?1:value.insertTextFormat;if(format!==1&&format!==2)throw new Error('Unsupported completion insertion format');
 const rawText=completionInsertion(value);if(format===2&&!expansion)throw new Error('Snippet expansion unavailable; no edits were applied.');
 const inserted=format===2?expansion!:{text:rawText};
 const validateOffset=(text:string,offset:number)=>{if(!Number.isSafeInteger(offset)||offset<0||offset>text.length)throw new Error('Invalid completion selection');sourceChangeOffset(text,utf16ToPosition(text,offset));};
 validateOffset(source,from);validateOffset(source,to);if(from>to)throw new Error('Completion fallback runs backwards');
 const edit=value.textEdit as Record<string,unknown>|undefined;
 if(edit){if(edit.range!==undefined&&(edit.insert!==undefined||edit.replace!==undefined))throw new Error("Ambiguous completion edit");const range=(edit.range??edit.replace) as {start?:unknown;end?:unknown}|undefined;if(!range)throw new Error('Missing completion range');from=sourceChangeOffset(source,range.start);to=sourceChangeOffset(source,range.end);if(from>to)throw new Error('Completion range runs backwards');}
 let additions:SourceReplacement[]=[];
 if(value.additionalTextEdits!==undefined){if(!Array.isArray(value.additionalTextEdits))throw new Error('Malformed additional completion edits');
  additions=value.additionalTextEdits.map(raw=>{const edit=raw as {range?:{start?:unknown;end?:unknown};newText?:unknown}|null;if(!edit?.range||typeof edit.newText!=='string')throw new Error('Malformed additional completion edit');const change={from:sourceChangeOffset(source,edit.range.start),to:sourceChangeOffset(source,edit.range.end),text:edit.newText};if(change.from>change.to)throw new Error('Additional completion edit runs backwards');return change;});
 }
 const unique=additions.filter((edit,index)=>additions.findIndex(other=>other.from===edit.from&&other.to===edit.to&&other.text===edit.text)===index);
 // A completion names its primary edit explicitly. Imports inserted at its
 // start precede that insertion, in the provider's declared additional order.
 const prefix=unique.filter(edit=>edit.from===from&&edit.to===from).map(edit=>edit.text).join('');
 const others=unique.filter(edit=>edit.from!==from||edit.to!==from),primary={from,to,text:prefix+inserted.text};
 const rawEdits=[primary,...others].map(edit=>({range:{start:utf16ToPosition(source,edit.from),end:utf16ToPosition(source,edit.to)},newText:edit.text}));
 const plan=planWorkspaceEdit(`Complete ${String(value.label)}`,{changes:{[uri]:rawEdits}},new Map([[path,snapshot]]),true);
 const delta=others.filter(edit=>edit.from<from).reduce((sum,edit)=>sum+edit.text.length-(edit.to-edit.from),0),insertionFrom=from+delta+prefix.length;
 const selected=inserted.selection??{from:inserted.text.length,to:inserted.text.length};validateOffset(inserted.text,selected.from);validateOffset(inserted.text,selected.to);if(selected.from>selected.to)throw new Error('Completion selection runs backwards');
 return {path,before:source,plan,insertionFrom,selection:{from:insertionFrom+selected.from,to:insertionFrom+selected.to}};
}
</ui:copy>

<ui:file path="apps/web/src/simple/CompletionPlan.ts"><ui:paste select="#completion-draft-browser"/></ui:file>

## Apple completion draft

Prepare the complete source change before touching a view. A completion explicitly names its primary edit, so additional insertions at its start precede that insertion in declared provider order. Deduplicate identical imports, validate all other replacements through the shared source planner, and map the final selection through earlier edits. Commands and unavailable snippet expansion refuse the whole proposal. The retained before snapshot also guards selection-only acceptance.

<ui:copy id="completion-draft-apple">
import Foundation
struct CompletionSelection {var from:Int;var to:Int}
struct CompletionExpansion {var text:String;var selection:CompletionSelection?=nil}
struct CompletionDraft {var path:String;var before:String;var plan:ChangePlan;var insertionFrom:Int;var selection:CompletionSelection}
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
    return CompletionDraft(path:path,before:source,plan:plan,insertionFrom:insertionFrom,selection:CompletionSelection(from:insertionFrom+selection.from,to:insertionFrom+selection.to))
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/CompletionPlan.swift"><ui:paste select="#completion-draft-apple"/></ui:file>

## Android completion draft

Prepare the complete source change before touching a view. A completion explicitly names its primary edit, so additional insertions at its start precede that insertion in declared provider order. Deduplicate identical imports, validate all other replacements through the shared source planner, and map the final selection through earlier edits. Commands and unavailable snippet expansion refuse the whole proposal. The retained before snapshot also guards selection-only acceptance.

<ui:copy id="completion-draft-android">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
data class CompletionSelection(val from:Int,val to:Int)
data class CompletionExpansion(val text:String,val selection:CompletionSelection?=null)
data class CompletionDraft(val path:String,val before:String,val plan:ChangePlan,val insertionFrom:Int,val selection:CompletionSelection)
fun completionInsertion(item:JSONObject):String {
 val label=item.opt("label") as? String ?: error("Malformed completion")
 if(item.has("textEdit"))return item.optJSONObject("textEdit")?.opt("newText") as? String ?: error("Missing completion insertion text")
 if(item.has("insertText"))return item.opt("insertText") as? String ?: error("Missing completion insertion text")
 return label
}
fun planCompletion(uri:String,item:JSONObject,snapshot:SourceSnapshot,initialFrom:Int,initialTo:Int,expansion:CompletionExpansion?=null):CompletionDraft {
 val source=snapshot.text;check(snapshot.writable){"Completion source is generated, read-only or saving"}
 val path=SemanticLocation(uri,0,0).path ?: error("Unsupported completion source identity")
 check(item.isNull("command")){"This completion requires a provider command; no edits were applied."}
 val format=if(item.has("insertTextFormat")){val value=item.opt("insertTextFormat") as? Number ?: error("Unsupported completion insertion format");check(value.toDouble()==value.toInt().toDouble()){ "Unsupported completion insertion format" };value.toInt()}else 1
 check(format==1||format==2){"Unsupported completion insertion format"}
 val rawText=completionInsertion(item);check(format!=2||expansion!=null){"Snippet expansion unavailable; no edits were applied."}
 val inserted=if(format==2)expansion!! else CompletionExpansion(rawText)
 fun validateOffset(text:String,offset:Int){check(offset in 0..text.length){"Invalid completion selection"};sourceChangeOffset(text,languagePosition(text,offset))}
 validateOffset(source,initialFrom);validateOffset(source,initialTo);check(initialFrom<=initialTo){"Completion fallback runs backwards"}
 var from=initialFrom;var to=initialTo
 item.optJSONObject("textEdit")?.let {edit->
  check(!edit.has("range")||!edit.has("insert")&&!edit.has("replace")){"Ambiguous completion edit"}
  val range=edit.optJSONObject("range") ?: edit.optJSONObject("replace") ?: error("Missing completion range")
  from=sourceChangeOffset(source,range.opt("start"));to=sourceChangeOffset(source,range.opt("end"));check(from<=to){"Completion range runs backwards"}
 }
 val additions=mutableListOf<SourceReplacement>()
 if(item.has("additionalTextEdits")) {
  val items=item.optJSONArray("additionalTextEdits") ?: error("Malformed additional completion edits")
  for(index in 0 until items.length()) {
   val edit=items.optJSONObject(index) ?: error("Malformed additional completion edit");val range=edit.optJSONObject("range") ?: error("Malformed additional completion edit");val text=edit.opt("newText") as? String ?: error("Malformed additional completion edit")
   val change=SourceReplacement(sourceChangeOffset(source,range.opt("start")),sourceChangeOffset(source,range.opt("end")),text);check(change.from<=change.to){"Additional completion edit runs backwards"};if(change !in additions)additions.add(change)
  }
 }
 val prefix=additions.filter{it.from==from&&it.to==from}.joinToString(""){it.text}
 val others=additions.filter{it.from!=from||it.to!=from};val primary=SourceReplacement(from,to,prefix+inserted.text)
 val edits=JSONArray();(listOf(primary)+others).forEach {edit->edits.put(JSONObject().put("range",JSONObject().put("start",languagePosition(source,edit.from)).put("end",languagePosition(source,edit.to))).put("newText",edit.text))}
 val plan=planWorkspaceEdit("Complete ${item.optString("label")}",JSONObject().put("changes",JSONObject().put(uri,edits)),mapOf(path to snapshot),allowNoop=true)
 val delta=others.filter{it.from<from}.sumOf{it.text.length-(it.to-it.from)};val insertionFrom=from+delta+prefix.length
 val selection=inserted.selection ?: CompletionSelection(inserted.text.length,inserted.text.length);validateOffset(inserted.text,selection.from);validateOffset(inserted.text,selection.to);check(selection.from<=selection.to){"Completion selection runs backwards"}
 return CompletionDraft(path,source,plan,insertionFrom,CompletionSelection(insertionFrom+selection.from,insertionFrom+selection.to))
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/CompletionPlan.kt"><ui:paste select="#completion-draft-android"/></ui:file>

## Specify the complete result independently of a renderer

These vectors name observable source bytes, caret/selection positions and whole-proposal refusals. They cover imports at the primary insertion, provider order, duplicate imports, distinct normalized Unicode byte sequences, earlier replacements, insert/replace form, no-op authority and an injected snippet expansion. Malformed positions, overlaps, commands, ownership and scalar-splitting selections refuse.

<ui:copy id="completion-draft-shared-vectors">
{
  "cases": [
    {
      "name": "plain fallback",
      "source": "va",
      "item": {
        "label": "value"
      },
      "from": 0,
      "to": 2,
      "after": "value",
      "selection": {
        "from": 5,
        "to": 5
      },
      "insertionFrom": 0
    },
    {
      "name": "import at primary start",
      "source": "va\n",
      "item": {
        "label": "value",
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
          "newText": "value"
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
            "newText": "# import\n"
          }
        ]
      },
      "from": 0,
      "to": 2,
      "after": "# import\nvalue\n",
      "selection": {
        "from": 14,
        "to": 14
      },
      "insertionFrom": 9
    },
    {
      "name": "ordered imports and identical duplicate",
      "source": "va",
      "item": {
        "label": "value",
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
            "newText": "# first\n"
          },
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
            "newText": "# second\n"
          },
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
            "newText": "# first\n"
          }
        ]
      },
      "from": 0,
      "to": 2,
      "after": "# first\n# second\nvalue",
      "selection": {
        "from": 22,
        "to": 22
      },
      "insertionFrom": 17
    },
    {
      "name": "distinct Unicode import bytes",
      "source": "va",
      "item": {
        "label": "value",
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
            "newText": "#é\n"
          },
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
            "newText": "#é\n"
          }
        ]
      },
      "from": 0,
      "to": 2,
      "after": "#é\n#é\nvalue",
      "selection": {
        "from": 12,
        "to": 12
      },
      "insertionFrom": 7
    },
    {
      "name": "earlier replacement moves primary caret",
      "source": "header\nva\n",
      "item": {
        "label": "value",
        "textEdit": {
          "range": {
            "start": {
              "line": 1,
              "character": 0
            },
            "end": {
              "line": 1,
              "character": 2
            }
          },
          "newText": "value"
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
                "character": 6
              }
            },
            "newText": "longheader"
          }
        ]
      },
      "from": 7,
      "to": 9,
      "after": "longheader\nvalue\n",
      "selection": {
        "from": 16,
        "to": 16
      },
      "insertionFrom": 11
    },
    {
      "name": "replace form consumes the remaining suffix",
      "source": "valu",
      "item": {
        "label": "value",
        "textEdit": {
          "insert": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "replace": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 4
            }
          },
          "newText": "value"
        }
      },
      "from": 0,
      "to": 2,
      "after": "value",
      "selection": {
        "from": 5,
        "to": 5
      },
      "insertionFrom": 0
    },
    {
      "name": "no-op still has a source-bound caret",
      "source": "value",
      "item": {
        "label": "value"
      },
      "from": 0,
      "to": 5,
      "patchCount": 0,
      "after": "value",
      "selection": {
        "from": 5,
        "to": 5
      },
      "insertionFrom": 0
    },
    {
      "name": "expanded selection follows imports in UTF16",
      "source": "va",
      "item": {
        "label": "snippet",
        "insertText": "${1:😀}",
        "insertTextFormat": 2,
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
            "newText": "#é\n"
          }
        ]
      },
      "from": 0,
      "to": 2,
      "expansion": {
        "text": "😀",
        "selection": {
          "from": 0,
          "to": 2
        }
      },
      "after": "#é\n😀",
      "selection": {
        "from": 3,
        "to": 5
      },
      "insertionFrom": 3
    },
    {
      "name": "readonly cannot become selection authority",
      "source": "value",
      "item": {
        "label": "value"
      },
      "from": 0,
      "to": 5,
      "writable": false,
      "error": true
    },
    {
      "name": "command refuses its edit portion",
      "source": "va",
      "item": {
        "label": "value",
        "command": {
          "title": "Effect",
          "command": "effect"
        }
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "snippet requires an expander",
      "source": "va",
      "item": {
        "label": "value",
        "insertText": "$1",
        "insertTextFormat": 2
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "additional overlap refuses whole proposal",
      "source": "va",
      "item": {
        "label": "value",
        "additionalTextEdits": [
          {
            "range": {
              "start": {
                "line": 0,
                "character": 1
              },
              "end": {
                "line": 0,
                "character": 2
              }
            },
            "newText": "bad"
          }
        ]
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "malformed additional list",
      "source": "va",
      "item": {
        "label": "value",
        "additionalTextEdits": "bad"
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "boolean source position",
      "source": "va",
      "item": {
        "label": "value",
        "textEdit": {
          "range": {
            "start": {
              "line": false,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          },
          "newText": "value"
        }
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "boolean format",
      "source": "va",
      "item": {
        "label": "value",
        "insertTextFormat": true
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "ambiguous edit representation",
      "source": "va",
      "item": {
        "label": "value",
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
          "newText": "value",
          "replace": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 0,
              "character": 2
            }
          }
        }
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "selection cannot split expanded scalar",
      "source": "va",
      "item": {
        "label": "value",
        "insertText": "$1",
        "insertTextFormat": 2
      },
      "from": 0,
      "to": 2,
      "expansion": {
        "text": "😀",
        "selection": {
          "from": 1,
          "to": 2
        }
      },
      "error": true
    },
    {
      "name": "fallback cannot split source scalar",
      "source": "😀",
      "item": {
        "label": "value"
      },
      "from": 1,
      "to": 2,
      "error": true
    },
    {
      "name": "foreign source URI",
      "source": "va",
      "item": {
        "label": "value"
      },
      "from": 0,
      "to": 2,
      "uri": "file:///etc/passwd",
      "error": true
    },
    {
      "name": "null edit is malformed",
      "source": "va",
      "item": {
        "label": "value",
        "textEdit": null
      },
      "from": 0,
      "to": 2,
      "error": true
    },
    {
      "name": "null insertion format is malformed",
      "source": "va",
      "item": {
        "label": "value",
        "insertTextFormat": null
      },
      "from": 0,
      "to": 2,
      "error": true
    }
  ]
}
</ui:copy>

<ui:file path="apps/simple-contract/completion-plan-cases.json"><ui:paste select="#completion-draft-shared-vectors"/></ui:file>

## Run the browser model against the shared vectors

Require exact before/after source and UTF-16 selection, verify the inverse draft, and refuse every negative vector without a change proposal. A no-op still retains the before snapshot.

<ui:copy id="completion-draft-browser-witness">
import {expect,it} from 'vitest';
import cases from '../../../simple-contract/completion-plan-cases.json';
import {planCompletion,type CompletionExpansion} from './CompletionPlan';
import {invertChangePlan,validateChangePlan} from './ChangePlan';
it('prepares source, import order, UTF16 selection and refusal from shared completion vectors',()=>{
 for(const item of cases.cases){
  const prepare=()=>planCompletion('uri' in item?item.uri!:'hick:///a.py',item.item,{text:item.source,writable:'writable' in item?item.writable!==false:true},item.from,item.to,'expansion' in item?item.expansion as CompletionExpansion:undefined);
  if('error' in item&&item.error){expect(prepare,item.name).toThrow();continue;}
  const value=prepare();expect(value.before,item.name).toBe(item.source);expect(value.plan.patches[0]?.after??item.source,item.name).toBe('after' in item?item.after:undefined);expect(value.selection,item.name).toEqual('selection' in item?item.selection:undefined);expect(value.insertionFrom,item.name).toBe('insertionFrom' in item?item.insertionFrom:undefined);
  if('patchCount' in item)expect(value.plan.patches).toHaveLength(item.patchCount!);
  if(value.plan.patches.length){const inverse=invertChangePlan(value.plan);expect(inverse.patches[0].after).toBe(item.source);expect(validateChangePlan(inverse,new Map([['a.py',{text:value.plan.patches[0].after,writable:true}]]))).toBeNull();}
 }
});
</ui:copy>

<ui:file path="apps/web/src/simple/CompletionPlan.test.ts"><ui:paste select="#completion-draft-browser-witness"/></ui:file>

## Run the Apple model against the same vectors

Foundation string normalization must not collapse distinct source or import bytes. The same fixture also checks source-position and selection scalar boundaries and inverse source recovery.

<ui:copy id="completion-draft-apple-witness">
import XCTest
@testable import HickorySimple
final class CompletionPlanTests:XCTestCase {
    func testSharedSourceImportSelectionAndRefusalVectors()throws {
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let raw=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/completion-plan-cases.json"))) as! [String:Any]
        for item in raw["cases"] as! [[String:Any]] {
            let name=item["name"] as! String,source=item["source"] as! String
            let expanded=item["expansion"] as? [String:Any],selected=expanded?["selection"] as? [String:Int]
            let expansion=expanded.map{CompletionExpansion(text:$0["text"] as! String,selection:selected.map{CompletionSelection(from:$0["from"]!,to:$0["to"]!)})}
            let prepare={try planCompletion(item["uri"] as? String ?? "hick:///a.py",item:item["item"] as! [String:Any],snapshot:SourceSnapshot(text:source,writable:item["writable"] as? Bool ?? true),from:item["from"] as! Int,to:item["to"] as! Int,expansion:expansion)}
            if item["error"] as? Bool==true{XCTAssertThrowsError(try prepare(),name);continue}
            let value=try prepare(),selection=item["selection"] as! [String:Int]
            XCTAssertTrue(exactUTF8(value.before,source),name);XCTAssertTrue(exactUTF8(value.plan.patches.first?.after ?? source,item["after"] as! String),name)
            XCTAssertEqual(value.selection.from,selection["from"],name);XCTAssertEqual(value.selection.to,selection["to"],name);XCTAssertEqual(value.insertionFrom,item["insertionFrom"] as? Int,name)
            if let count=item["patchCount"] as? Int{XCTAssertEqual(value.plan.patches.count,count,name)}
            if let patch=value.plan.patches.first{let inverse=invertChangePlan(value.plan);XCTAssertTrue(exactUTF8(inverse.patches[0].after,source),name);XCTAssertNil(validateChangePlan(inverse,sources:["a.py":SourceSnapshot(text:patch.after,writable:true)]),name)}
        }
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/CompletionPlanTests.swift"><ui:paste select="#completion-draft-apple-witness"/></ui:file>

## Run the Android model against the same vectors

Require the same complete source, source-bound caret, inverse and refusal outcomes independently of Compose or a live provider.

<ui:copy id="completion-draft-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class CompletionPlanTest {
 @Test fun sharedSourceImportSelectionAndRefusalVectors() {
  val cases=JSONObject(javaClass.classLoader!!.getResource("completion-plan-cases.json")!!.readText()).getJSONArray("cases")
  for(index in 0 until cases.length()) {
   val item=cases.getJSONObject(index);val name=item.getString("name");val source=item.getString("source")
   val expanded=item.optJSONObject("expansion");val selected=expanded?.optJSONObject("selection");val expansion=expanded?.let{CompletionExpansion(it.getString("text"),selected?.let{CompletionSelection(it.getInt("from"),it.getInt("to"))})}
   val value=runCatching{planCompletion(item.optString("uri","hick:///a.py"),item.getJSONObject("item"),SourceSnapshot(source,item.optBoolean("writable",true)),item.getInt("from"),item.getInt("to"),expansion)}
   if(item.optBoolean("error")){assertTrue(name,value.isFailure);continue}
   val draft=value.getOrThrow();val selection=item.getJSONObject("selection");assertEquals(name,source,draft.before);assertEquals(name,item.getString("after"),draft.plan.patches.firstOrNull()?.after ?: source);assertEquals(name,selection.getInt("from"),draft.selection.from);assertEquals(name,selection.getInt("to"),draft.selection.to);assertEquals(name,item.getInt("insertionFrom"),draft.insertionFrom)
   if(item.has("patchCount"))assertEquals(name,item.getInt("patchCount"),draft.plan.patches.size)
   draft.plan.patches.firstOrNull()?.let{patch->val inverse=invertChangePlan(draft.plan);assertEquals(name,source,inverse.patches.first().after);assertNull(name,validateChangePlan(inverse,mapOf("a.py" to SourceSnapshot(patch.after,true))))}
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CompletionPlanTest.kt"><ui:paste select="#completion-draft-android-witness"/></ui:file>

</ui:doc>
