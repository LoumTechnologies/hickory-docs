<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Derive a complete completion draft before accepting it

For maintainers changing completion acceptance independently of popup rendering.

A completion is a primary replacement plus a complete set of import edits and a resulting source selection. Source ownership, scalar boundaries and overlap are preconditions, not optional presentation details. A replaceable snippet expander supplies plain text and an initial selection; this planner never inserts literal snippet syntax when that capability is absent. A no-op completion may move the caret while retaining the captured source for an authoritative host check. Nothing here applies, saves or verifies source.

## Browser completion draft

Prepare the complete source change before touching a view. A completion explicitly names its primary edit, so additional insertions at its start precede that insertion in declared provider order. Deduplicate identical imports, validate all other replacements through the shared source planner, and map the final selection and every retained snippet field through earlier edits. Commands and unavailable snippet expansion refuse the whole proposal. Retain the primary replacement and deduplicated additions separately from their assembled patch. An active template can then mirror only the primary text and apply imports once; renderer code must not reverse-engineer those roles from a merged edit. The retained before snapshot also guards selection-only acceptance.

An ordinary completion group derives each primary edit from its own resolved
offer. The offers must agree on replacement text and ordered additional edits.
Each primary range contains its captured selection. Apply the shared additions
once, then use the complete source plan to locate every resulting caret.
When an ordinary offer supplies insertion text without a range, use the
editor's identifier prefix at each collapsed caret and compare those prefixes.

<ui:copy id="completion-draft-browser">
import {planWorkspaceEdit,sourceChangeOffset,type ChangePlan,type SourceReplacement,type SourceSnapshot} from './ChangePlan';
import {semanticPath} from './Intelligence';
import type {SnippetField} from './SnippetExpansion';
import type {SnippetSession} from './SnippetSession';
import {beginParallelSnippet,rebaseParallelSnippet,type ParallelSnippet} from './ParallelSnippet';
import {utf16ToPosition} from '../lsp/positions';
export interface CompletionExpansion {text:string;selection?:{from:number;to:number};fields?:SnippetField[]}
export interface CompletionDraft {path:string;before:string;plan:ChangePlan;insertionFrom:number;selection:{from:number;to:number};fields:SnippetField[];continuing?:SnippetSession;primary?:SourceReplacement;additional?:SourceReplacement[]}
export interface CapturedCompletion {item:Record<string, unknown>;from:number;to:number;prefixFrom?:number;expansion?:CompletionExpansion}
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
 const fields=(inserted.fields??[]).map(field=>{validateOffset(inserted.text,field.from);validateOffset(inserted.text,field.to);if(field.from>field.to)throw new Error('Invalid snippet field');return {...field,from:insertionFrom+field.from,to:insertionFrom+field.to};});
 return {path,before:source,plan,insertionFrom,fields,primary:{from,to,text:inserted.text},additional:unique,selection:{from:insertionFrom+selected.from,to:insertionFrom+selected.to}};
}
function prepareCompletionGroup(uri:string,snapshot:SourceSnapshot,captures:readonly CapturedCompletion[]) {
 const first = captures[0];
 if (!first || captures.length < 2 || captures.length > 16) throw new Error("Select 2 to 16 carets for a completion group.");
 const source = snapshot.text;
 const snippets = first.item.insertTextFormat === 2;
 if (captures.some(({item}) => item.textEdit === undefined)) {
  const prefix = first.prefixFrom === undefined ? undefined : source.slice(first.prefixFrom,first.to);
  if (prefix === undefined || captures.some(({from,to,prefixFrom}) =>
   from !== to || prefixFrom === undefined || source.slice(prefixFrom,to) !== prefix)) {
   throw new Error("Implicit completion ranges require the same identifier prefix at each caret.");
  }
 }
 const drafts = captures.map(({item,from,to,prefixFrom,expansion}) => {
  if ((item.insertTextFormat === 2) !== snippets || (!snippets && item.kind === 15)) {
   throw new Error("The carets must agree on the completion format.");
  }
  const draft = planCompletion(uri,item,snapshot,prefixFrom ?? from,to,expansion);
  const primary = draft.primary!;
  if (primary.from > from || primary.to < to) throw new Error("A completion range does not cover its captured selection.");
  return {primary,additional:draft.additional!,path:draft.path,fields:expansion?.fields ?? []};
 });
 const expected = drafts[0];
 const additional = JSON.stringify(expected.additional);
 if (drafts.some(draft => (!snippets && draft.primary.text !== expected.primary.text) || JSON.stringify(draft.additional) !== additional)) {
  throw new Error("The completions disagree about their replacement or shared edits.");
 }
 const byStart = new Map(drafts.map((draft,index) => [draft.primary.from,index]));
 if (byStart.size !== drafts.length) throw new Error("Completion ranges overlap.");
 const prefixes = new Map<number,string>();
 const edits = expected.additional.filter(edit => {
  if (edit.from !== edit.to || !byStart.has(edit.from)) return true;
  prefixes.set(edit.from,(prefixes.get(edit.from) ?? "") + edit.text);
  return false;
 });
 for (const {primary} of drafts) {
  const text = (prefixes.get(primary.from) ?? "") + primary.text;
  if (edits.some(edit => edit.from === primary.from)) throw new Error("Completion ranges overlap with a shared edit.");
  edits.push({...primary,text});
 }
 edits.sort((a,b) => a.from - b.from || a.to - b.to);
 const changes = edits.map(edit => ({
  range:{start:utf16ToPosition(source,edit.from),end:utf16ToPosition(source,edit.to)},newText:edit.text,
 }));
 const plan = planWorkspaceEdit(`Complete ${first.item.label}`,{changes:{[uri]:changes}},new Map([[expected.path,snapshot]]),true);
 const carets: number[] = [];
 const starts: number[] = [];
 let delta = 0;
 for (const edit of edits) {
  const index = byStart.get(edit.from);
  if (index !== undefined) {
   starts[index] = edit.from + delta + (prefixes.get(edit.from)?.length ?? 0);
   carets[index] = edit.from + delta + edit.text.length;
  }
  delta += edit.text.length - (edit.to - edit.from);
 }
 return {plan,carets,starts,drafts,snippets};
}

export function preflightSnippetGroup(uri:string,snapshot:SourceSnapshot,captures:readonly CapturedCompletion[]):void {
 // Only the provider edit roles are needed before the optional clipboard read.
 // Field expansion and final caret geometry are admitted against the full draft later.
 prepareCompletionGroup(uri,snapshot,captures.map(capture=>({...capture,expansion:{text:''}})));
}

export function planCompletionGroup(uri:string,snapshot:SourceSnapshot,captures:readonly CapturedCompletion[],primaryIndex=0): {plan:ChangePlan;carets:number[];parallel?:ParallelSnippet} {
 const {plan,carets,starts,drafts,snippets} = prepareCompletionGroup(uri,snapshot,captures);
 if (snippets) {
  const prepared = beginParallelSnippet(snapshot.text,drafts.map(({primary,fields}) => ({...primary,fields})),primaryIndex);
  const parallel = prepared && rebaseParallelSnippet(prepared.state,plan.patches[0]?.after ?? snapshot.text,starts);
  if (!parallel) throw new Error("The complete snippet group cannot retain its fields and selections.");
  return {plan,carets,parallel};
 }
 if (new Set(carets).size !== drafts.length) throw new Error("The completion would merge distinct carets.");
 return {plan,carets};
}
</ui:copy>

<ui:file path="apps/web/src/simple/CompletionPlan.ts"><ui:paste select="#completion-draft-browser"/></ui:file>

## Apple completion draft

Apple combines independently resolved completions against one exact source
snapshot. Each primary edit must cover its captured caret, and every proposal
must agree on the complete list of additional edits. A shared import appears
once in the source plan. Snippet text and fields can vary with each selection's
context. Each peer keeps its own field tree, with its placement adjusted for
earlier replacements and imports. Native Undo restores the original caret
ranges, and Redo restores the parallel field session when text changes. An
identical expansion may establish its selections and fields through the same
guarded editor transaction without adding a text Undo entry.

Prepare the complete source change before touching a view. A completion explicitly names its primary edit, so additional insertions at its start precede that insertion in declared provider order. Deduplicate identical imports, validate all other replacements through the shared source planner, and map the final selection and every retained snippet field through earlier edits. Commands and unavailable snippet expansion refuse the whole proposal. Retain the primary replacement and deduplicated additions separately from their assembled patch. An active template can then mirror only the primary text and apply imports once; renderer code must not reverse-engineer those roles from a merged edit. The retained before snapshot also guards selection-only acceptance.

Offset rebasing preserves every field property, including its computed program. A derived occurrence must remain derived after prefix/import edits; dropping that program makes its different initial text look like an inconsistent ordinary mirror and silently prevents session installation. The connected witness below rebases a real program, starts a session from the resulting draft, then edits its source and requires the derived result.

<ui:copy id="completion-draft-apple">
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/CompletionPlan.swift"><ui:paste select="#completion-draft-apple"/></ui:file>

## Android completion draft

Prepare the complete source change before touching a view. A completion explicitly names its primary edit, so additional insertions at its start precede that insertion in declared provider order. Deduplicate identical imports, validate all other replacements through the shared source planner, and map the final selection and every retained snippet field through earlier edits. Commands and unavailable snippet expansion refuse the whole proposal. Retain the primary replacement and deduplicated additions separately from their assembled patch. An active template can then mirror only the primary text and apply imports once; renderer code must not reverse-engineer those roles from a merged edit. The retained before snapshot also guards selection-only acceptance.

Each caret supplies a separately resolved ordinary edit. The planner checks
that each replacement covers its original selection. If a reply omits a
range, all carets must be collapsed at identical identifier prefixes. The
group reuses the existing source planner after deduplicating shared imports.
It computes every resulting caret from the full edit order before the editor
can change.

Each snippet can have its own provider syntax, indentation, and expanded
context. The existing parallel planner validates the local fields together.
Their insertion starts place the peers after shared imports while keeping
the import text outside the editable fields. A pending clipboard request
uses a placeholder expansion in this same edit composer. Only the projected
edit roles survive that preflight. Retrying a token must preserve its
syntax, primary range and ordered shared edits.

<ui:copy id="completion-draft-android">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
data class CompletionSelection(val from:Int,val to:Int)
data class CompletionExpansion(val text:String,val selection:CompletionSelection?=null,val fields:List<SnippetField> = emptyList())
data class CompletionDraft(val path:String,val before:String,val plan:ChangePlan,val insertionFrom:Int,val selection:CompletionSelection,val fields:List<SnippetField> = emptyList(),val continuing:SnippetSession?=null,val primary:SourceReplacement?=null,val additional:List<SourceReplacement> = emptyList())
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
 val fields=inserted.fields.map {field->validateOffset(inserted.text,field.from);validateOffset(inserted.text,field.to);check(field.from<=field.to){"Invalid snippet field"};field.copy(from=insertionFrom+field.from,to=insertionFrom+field.to)}
 return CompletionDraft(path,source,plan,insertionFrom,CompletionSelection(insertionFrom+selection.from,insertionFrom+selection.to),fields,primary=SourceReplacement(from,to,inserted.text),additional=additions)
}

internal data class CapturedAndroidCompletion(val item:JSONObject,val from:Int,val to:Int,val prefixFrom:Int?)
internal data class AndroidCompletionGroup(val plan:ChangePlan,val carets:List<Int>,val snippet:ParallelSnippet?=null)

internal fun groupCompletionFormat(item:JSONObject):Int {
    val number=(if(!item.has("insertTextFormat"))1 else item.opt("insertTextFormat")) as? Number
    return when(number?.toDouble()){
        1.0 -> 1
        2.0 -> 2
        else -> error("Unsupported grouped completion format.")
    }
}

internal data class AndroidCompletionRole(val syntax:String,val from:Int,val to:Int,val additions:List<SourceReplacement>)

private data class ComposedAndroidCompletion(val group:AndroidCompletionGroup?,val roles:List<AndroidCompletionRole>)

internal fun preflightCompletionGroup(uri:String,snapshot:SourceSnapshot,captures:List<CapturedAndroidCompletion>,
                                      waitingForClipboard:Set<Int>):List<AndroidCompletionRole> =
    composeCompletionGroup(uri,snapshot,captures,clipboardPreflight=waitingForClipboard).roles

internal fun planCompletionGroup(uri:String,snapshot:SourceSnapshot,captures:List<CapturedAndroidCompletion>,
                                 preserving:List<AndroidCompletionRole>?=null):AndroidCompletionGroup =
    checkNotNull(composeCompletionGroup(uri,snapshot,captures,preserving=preserving).group)

private fun composeCompletionGroup(uri:String,snapshot:SourceSnapshot,captures:List<CapturedAndroidCompletion>,
                                   clipboardPreflight:Set<Int>?=null,preserving:List<AndroidCompletionRole>?=null):ComposedAndroidCompletion {
    require(captures.size in 2..16) { "Select 2 to 16 carets for a completion group." }
    val source = snapshot.text
    if (captures.any { !it.item.has("textEdit") }) {
        val prefixes = captures.map { (_,from,to,prefixFrom) ->
            require(from == to && prefixFrom != null && prefixFrom in 0..to && to <= source.length) {
                "Implicit completion ranges require the same identifier prefix at each caret."
            }
            source.substring(prefixFrom,to)
        }
        require(prefixes.distinct().size == 1) { "Implicit completion ranges require the same identifier prefix at each caret." }
    }
    val format=captures.map { groupCompletionFormat(it.item) }.distinct().singleOrNull()
        ?: error("The completions mix ordinary edits and templates.")
    require(format==2||captures.none {it.item.opt("kind")==15}){"A template did not return a snippet expansion."}
    val snippet=format==2
    val budget=SnippetTransformBudget()
    val expansions=captures.mapIndexed { index,capture ->
        when {
            !snippet -> null
            clipboardPreflight?.contains(index)==true -> CompletionExpansion("")
            else -> checkNotNull(completionExpansion(capture.item,budget))
        }
    }
    val drafts = captures.mapIndexed { index,(item,from,to,prefixFrom) ->
        planCompletion(uri,item,snapshot,prefixFrom ?: from,to,expansions[index]).also { draft ->
            val edit = checkNotNull(draft.primary)
            require(edit.from <= from && edit.to >= to) { "A completion range does not cover its captured selection." }
        }
    }
    val first = drafts.first()
    val primary = drafts.map { checkNotNull(it.primary) }
    val roles=drafts.mapIndexed { index,draft ->
        AndroidCompletionRole(completionInsertion(captures[index].item),primary[index].from,primary[index].to,draft.additional)
    }
    require(preserving==null||roles==preserving){
        "Completion edits changed after requesting context. Request completions again."
    }
    require(drafts.all { (snippet || it.primary?.text == primary.first().text) && it.additional == first.additional }) {
        "The completions disagree about their replacement or shared edits."
    }
    val byStart = primary.withIndex().associate { it.value.from to it.index }
    require(byStart.size == primary.size) { "Completion ranges overlap." }
    val prefixes = first.additional.filter { it.from == it.to && it.from in byStart }.groupBy { it.from }
    val edits = (first.additional.filter { it.from != it.to || it.from !in byStart } + primary.map { edit ->
        edit.copy(text = prefixes[edit.from].orEmpty().joinToString("") { it.text } + edit.text)
    }).sortedWith(compareBy({it.from},{it.to}))
    require(edits.zipWithNext().none { (left,right) -> left.from == right.from || left.to > right.from }) {
        "Completion ranges overlap with another completion or shared edit."
    }
    val raw = JSONArray()
    val insertionStarts=IntArray(primary.size)
    val carets = IntArray(primary.size)
    var delta = 0
    for (edit in edits) {
        raw.put(JSONObject().put("range",JSONObject().put("start",languagePosition(source,edit.from))
            .put("end",languagePosition(source,edit.to))).put("newText",edit.text))
        byStart[edit.from]?.let { index ->
            carets[index]=edit.from+delta+edit.text.length
            insertionStarts[index]=carets[index]-primary[index].text.length
        }
        delta += edit.text.length - (edit.to - edit.from)
    }

    val plan = planWorkspaceEdit("Complete ${captures.first().item.getString("label")}",
        JSONObject().put("changes",JSONObject().put(uri,raw)),mapOf(first.path to snapshot),allowNoop=true)
    if(clipboardPreflight!=null)return ComposedAndroidCompletion(null,roles)
    if(!snippet){
        require(carets.distinct().size==primary.size){"The completion would merge distinct carets."}
        return ComposedAndroidCompletion(AndroidCompletionGroup(plan,carets.toList()),roles)
    }
    val peers=primary.mapIndexed { index,edit ->
        ParallelSnippetExpansion(NativeSourceEdit(edit.from,edit.to,edit.text),checkNotNull(expansions[index]).fields)
    }
    val after=plan.patches.singleOrNull()?.after ?: source
    val group=ParallelSnippet.begin(source,peers,0)?.state?.placing(after,insertionStarts.toList())
        ?: error("The parallel snippet fields overlap or exceed their bounds.")
    require(group.selections.distinct().size==primary.size){"The completion would merge distinct template fields."}
    return ComposedAndroidCompletion(AndroidCompletionGroup(plan,carets.toList(),group),roles)
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
import {planCompletion,planCompletionGroup,type CompletionExpansion} from './CompletionPlan';
import derived from '../../../simple-contract/snippet-derived-cases.json';
import {readSnippetTransform} from './SnippetTransform';
import {beginSnippet,editSnippet} from './SnippetSession';
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
it('preserves rebased computed programs through import edits and usable linked input',()=>{
 const transform=readSnippetTransform(derived.programs.upper),fields=[{index:1,from:0,to:3,choices:[],id:0,parent:null},{index:1,from:4,to:7,choices:[],id:1,parent:null,transform},{index:0,from:7,to:7,choices:[],id:2,parent:null}],draft=planCompletion('hick:///a.py',{label:'derived',insertTextFormat:2,insertText:'${1:foo} ${1/(.*)/${1:/upcase}/}$0',additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'import\n'}]},{text:'😀 old\n',writable:true},3,6,{text:'foo FOO',selection:{from:0,to:3},fields}),after=draft.plan.patches[0].after;
 expect(after).toBe('import\n😀 foo FOO\n');expect(draft.fields![1].from).toBe(14);expect(draft.fields![1].transform).toBe(transform);const session=beginSnippet(after,draft.fields!)!;expect(session).not.toBeNull();expect(editSnippet(session,after,draft.selection.from,draft.selection.to,'ßa')?.session.source).toBe('import\n😀 ßa SSA\n');
});
it('combines independently resolved identifier edits and a single shared import',()=>{
 const source='😀 va\nva\n';
 const item={label:'value',additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]};
 const first={item,from:5,to:5,prefixFrom:3},second={item,from:8,to:8,prefixFrom:6};
 const prepare=(captures:Parameters<typeof planCompletionGroup>[2])=>planCompletionGroup('hick:///a.py',{text:source,writable:true},captures);
 const result=prepare([first,second]);
 expect(result.plan.patches[0].after).toBe('# import\n😀 value\nvalue\n');
 expect(result.plan.patches[0].edits).toHaveLength(3);
 expect(result.carets).toEqual([17,23]);
 expect(()=>prepare([first,{...second,prefixFrom:7}])).toThrow('same identifier prefix');
 expect(()=>prepare([first,{...second,item:{...item,additionalTextEdits:[]}}])).toThrow('disagree');
 expect(()=>prepare([first,first])).toThrow('overlap');
});
it('keeps fields for a source-equal snippet group without a text patch',()=>{
 const source='va\nva\n',item={label:'value',insertTextFormat:2,insertText:'${1:va}$0'};
 const expansion={text:'va',fields:[
  {index:1,from:0,to:2,choices:[],id:0,parent:null},
  {index:0,from:2,to:2,choices:[],id:1,parent:null},
 ]};
 const draft=planCompletionGroup('hick:///a.py',{text:source,writable:true},[
  {item,from:2,to:2,prefixFrom:0,expansion},{item,from:5,to:5,prefixFrom:3,expansion},
 ],1);
 expect(draft.plan.patches).toEqual([]);
 expect(draft.parallel?.source).toBe(source);
 expect(draft.parallel?.primary).toBe(1);
 expect(draft.parallel?.peers.map(peer=>[peer.from,peer.selection.from,peer.selection.to])).toEqual([[0,0,2],[3,0,2]]);
});
</ui:copy>

<ui:file path="apps/web/src/simple/CompletionPlan.test.ts"><ui:paste select="#completion-draft-browser-witness"/></ui:file>

## Run the Apple model against the same vectors

Foundation string normalization must not collapse distinct source or import bytes. The same fixture also checks source-position and selection scalar boundaries and inverse source recovery.

<ui:copy id="completion-draft-apple-witness">
import XCTest
@testable import HickorySimple
final class CompletionPlanTests:XCTestCase {
    func testOrdinaryCaretDraftsCheckCompleteRangesAndSharedEdits() throws {
        let uri="hick:///a.py",source="😀\nva\nva\n"
        let snapshot=SourceSnapshot(text:source,writable:true)
        let carets=[3,6].map{NSRange(location:$0,length:2)}
        func edit(_ line:Int,_ start:Int,_ end:Int,_ text:String)->[String:Any] {
            ["range":["start":["line":line,"character":start],"end":["line":line,"character":end]],"newText":text]
        }
        func proposal(_ line:Int,_ from:Int=0,extra:String="import x\n")throws->CompletionDraft {
            try planCompletion(uri,item:["label":"value","textEdit":edit(line,from,2,"value"),
                "additionalTextEdits":[edit(0,0,0,extra)]],snapshot:snapshot,from:0,to:0)
        }
        let first=try proposal(1),second=try proposal(2)
        let batch=try combineCompletionDrafts([first,second],carets:carets,snapshot:snapshot)
        XCTAssertEqual(batch.plan.patches.first?.after,"import x\n😀\nvalue\nvalue\n")
        XCTAssertEqual(batch.plan.patches.first?.edits.count,3)
        XCTAssertEqual(batch.caretBatch?.before,carets)
        XCTAssertEqual(batch.caretBatch?.after,[17,23].map{NSRange(location:$0,length:0)})
        XCTAssertThrowsError(try combineCompletionDrafts([first,proposal(2,1)],carets:carets,snapshot:snapshot))
        XCTAssertThrowsError(try combineCompletionDrafts([first,proposal(2,extra:"import different\n")],carets:carets,snapshot:snapshot))
        XCTAssertFalse(validCompletionCarets([NSRange(location:1,length:0),carets[1]],
                                           primary:NSRange(location:1,length:0),source:source))

        let overlapping=SourceSnapshot(text:"va  va",writable:true)
        let left=try planCompletion(uri,item:["label":"value","textEdit":edit(0,0,4,"value")],snapshot:overlapping,from:0,to:0)
        let right=try planCompletion(uri,item:["label":"value","textEdit":edit(0,3,6,"value")],snapshot:overlapping,from:0,to:0)
        XCTAssertThrowsError(try combineCompletionDrafts([left,right],carets:[2,6].map{NSRange(location:$0,length:0)},snapshot:overlapping))
    }

    func testRebasedComputedProgramsRemainUsableAfterImportEdits()throws {
      let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent();let raw=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-derived-cases.json"))) as! [String:Any];let program=try readSnippetTransform((raw["programs"] as! [String:Any])["upper"]!)
      let fields=[SnippetField(index:1,from:0,to:3,choices:[],id:0),SnippetField(index:1,from:4,to:7,choices:[],id:1,transform:program),SnippetField(index:0,from:7,to:7,choices:[],id:2)]
      let item:[String:Any]=["label":"derived","insertTextFormat":2,"insertText":"${1:foo} ${1/(.*)/${1:/upcase}/}$0","additionalTextEdits":[["range":["start":["line":0,"character":0],"end":["line":0,"character":0]],"newText":"import\n"]]]
      let draft=try planCompletion("hick:///a.py",item:item,snapshot:SourceSnapshot(text:"😀 old\n",writable:true),from:3,to:6,expansion:CompletionExpansion(text:"foo FOO",selection:CompletionSelection(from:0,to:3),fields:fields));let after=try XCTUnwrap(draft.plan.patches.first?.after)
      XCTAssertEqual(after,"import\n😀 foo FOO\n");XCTAssertEqual(draft.fields[1].from,14);XCTAssertNotNil(draft.fields[1].transform);let session=try XCTUnwrap(beginSnippet(after,fields:draft.fields));let edited=try XCTUnwrap(editSnippet(session,source:after,from:draft.selection.from,to:draft.selection.to,text:"ßa"));XCTAssertEqual(edited.session.source,"import\n😀 ßa SSA\n")
    }
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
 @Test fun rebasedComputedProgramsRemainUsableAfterImportEdits(){
  val programs=JSONObject(javaClass.classLoader!!.getResource("snippet-derived-cases.json")!!.readText()).getJSONObject("programs");val transform=readSnippetTransform(programs.get("upper"));val fields=listOf(SnippetField(1,0,3,emptyList(),0),SnippetField(1,4,7,emptyList(),1,transform=transform),SnippetField(0,7,7,emptyList(),2))
  val item=JSONObject().put("label","derived").put("insertTextFormat",2).put("insertText","derived").put("additionalTextEdits",org.json.JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",0))).put("newText","import\n")))
  val draft=planCompletion("hick:///a.py",item,SourceSnapshot("😀 old\n",true),3,6,CompletionExpansion("foo FOO",CompletionSelection(0,3),fields));val after=draft.plan.patches.first().after;assertEquals("import\n😀 foo FOO\n",after);assertEquals(14,draft.fields[1].from);assertSame(transform,draft.fields[1].transform);val session=beginSnippet(after,draft.fields);assertNotNull(session);assertEquals("import\n😀 ßa SSA\n",editSnippet(session!!,after,draft.selection.from,draft.selection.to,"ßa")!!.session.source)
 }
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
 @Test fun ordinaryGroupChecksExactImplicitPrefixesAndRetainsEveryMappedCaret() {
  val uri="hick:///a.py"
  fun item()=JSONObject().put("label","value").put("insertText","value")
  fun group(source:String,secondFrom:Int,secondTo:Int,secondPrefix:Int?)=planCompletionGroup(uri,SourceSnapshot(source,true),listOf(
   CapturedAndroidCompletion(item(),2,2,0),CapturedAndroidCompletion(item(),secondFrom,secondTo,secondPrefix)))
  val accepted=group("va\nva\n",5,5,3)
  assertEquals("value\nvalue\n",accepted.plan.patches.single().after)
  assertEquals(listOf(5,11),accepted.carets)
  assertTrue(runCatching{group("va\nvo\n",5,5,3)}.isFailure)
  assertTrue(runCatching{group("va\nva\n",3,5,null)}.isFailure)
  assertTrue(runCatching{group("va\nva\n",2,2,0)}.isFailure)
 }

}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CompletionPlanTest.kt"><ui:paste select="#completion-draft-android-witness"/></ui:file>

</ui:doc>
