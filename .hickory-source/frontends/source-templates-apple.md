<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preview reusable templates without replacing the native editor

For maintainers adding template providers while retaining native text, selection and coordinated snippet history. The engine owns the catalog and snippet expansion; this adapter never resolves an LSP item or invents a command/import fallback.

## Bind one source selection to one catalog

A native editor instance has a separate identity from its retained buffer. Capture the actual native text and selection, reject composition and multiple carets, and retain the workspace namespace, transport and source edit identity. Returning to equal text after an edit/undo does not revive an earlier preview.

Capture local time and random seed once when this reading opens, and send protocol 2 with that immutable variable context. Clipboard is initially unattempted: a verified engine request may demand it once, then the same source lease and clock/seed travel with either its captured text or explicit denial. Catalog identity does not change merely because clipboard was attempted. Apply repeats the reviewed context without reading the clipboard again. Reject additional effects and malformed or substituted context before preparing any draft; the existing editor and source revision checks remain authoritative after every asynchronous response.

<ui:copy id="source-templates-apple-values">
import Foundation
import Observation
import SwiftUI
struct NativeSourceEditorState {
    var identity:UUID;var source:String;var selection:NSRange;var composing:Bool;var editable:Bool;var attached:Bool;var selections:Int
}
struct SourceTemplateRange:Codable,Equatable {var from:Int;var to:Int
    init(_ selection:NSRange){from=selection.location;to=NSMaxRange(selection)}
    init(from:Int,to:Int){self.from=from;self.to=to}
    var body:[String:Int]{["from":from,"to":to]}
}
struct SourceTemplateEntry:Decodable,Identifiable {var id:String;var label:String;var description:String;var language:String;var origin:String;var available:Bool;var reason:String?;var exactIdentity:Data{Data(id.utf8)}}
private struct SourceTemplateCatalogStorage:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var available:Bool;var notices:[String];var entries:[SourceTemplateEntry];var variable_context:SnippetVariableSnapshot?=nil;var source_files_modified:Bool?=nil
}
struct SourceTemplateCatalog:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var available:Bool;var notices:[String];var entries:[SourceTemplateEntry];var variable_context:SnippetVariableSnapshot?=nil;var source_files_modified:Bool?=nil
    init(from decoder:Decoder)throws{try sourceTemplateNoExtraEffects(decoder);let value=try SourceTemplateCatalogStorage(from:decoder);version=value.version;template_protocol=value.template_protocol;path=value.path;selection=value.selection;source_hash=value.source_hash;workspace_id=value.workspace_id;language=value.language;catalog_fingerprint=value.catalog_fingerprint;available=value.available;notices=value.notices;entries=value.entries;variable_context=value.variable_context;source_files_modified=value.source_files_modified}
}
struct SourceTemplateField:Codable {
    var id:Int;var parent:Int?;var index:Int;var from:Int;var to:Int;var choices:[String];var transform:ResourceJSON?=nil
    var body:[String:Any]{get throws{var value:[String:Any]=["id":id,"parent":parent.map{$0 as Any} ?? NSNull(),"index":index,"from":from,"to":to,"choices":choices];if let transform{value["transform"]=try JSONSerialization.jsonObject(with:JSONEncoder().encode(transform),options:.fragmentsAllowed)};return value}}
    func same(_ other:Self)->Bool{id==other.id && parent==other.parent && index==other.index && from==other.from && to==other.to && choices.count==other.choices.count && zip(choices,other.choices).allSatisfy{exactUTF8($0.0,$0.1)} && sourceTemplateTransformSame(transform,other.transform)}
}
struct SourceTemplateSnippet:Decodable {
    var version:Int;var syntax:String;var text:String;var fields:[SourceTemplateField]
    var body:[String:Any]{get throws{["version":version,"syntax":syntax,"text":text,"fields":try fields.map{try $0.body}]}}
    func same(_ other:Self)->Bool{version==other.version && exactUTF8(syntax,other.syntax) && exactUTF8(text,other.text) && fields.count==other.fields.count && zip(fields,other.fields).allSatisfy{$0.0.same($0.1)}}
}
private func sourceTemplateTransformSame(_ a:ResourceJSON?,_ b:ResourceJSON?)->Bool{let encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];guard let left=try? encoder.encode(a),let right=try? encoder.encode(b) else{return false};return left==right}
private enum TemplateEffectKey:String,CodingKey{case command,additionalTextEdits,workspaceEdit,edit,context_request,variable_context}
private func sourceTemplateNoExtraEffects(_ decoder:Decoder,allowContextRequest:Bool=false)throws{
 let data=try decoder.container(keyedBy:TemplateEffectKey.self)
 guard ![TemplateEffectKey.command,.additionalTextEdits,.workspaceEdit,.edit].contains(where:{data.contains($0)}) else{throw EngineError(message:"Template contains unsupported additional effects. No edits were applied.")}
 func reject()throws->Never{throw EngineError(message:"Malformed template variable context. No edits were applied.")}
 if data.contains(.variable_context),try !data.decodeNil(forKey:.variable_context){
  guard case .object(let context)=try data.decode(ResourceJSON.self,forKey:.variable_context),Set(context.keys)==Set(["clock","random_hex","clipboard"]) else{try reject()}
  if let clock=context["clock"]{switch clock{case .null:break;case .object(let fields):guard Set(fields.keys)==Set(["unix_millis","utc_offset_minutes"]) else{try reject()};for value in fields.values{switch value{case .integer,.unsigned:break;default:try reject()}};default:try reject()}}
  if let random=context["random_hex"]{switch random{case .null,.string:break;default:try reject()}}
  if let clipboard=context["clipboard"]{switch clipboard{case .null:break;case .object(let fields):guard Set(fields.keys)==Set(["available","text"]),let available=fields["available"],case .bool=available else{try reject()};if let text=fields["text"]{switch text{case .null,.string:break;default:try reject()}};default:try reject()}}
 }
 if data.contains(.context_request){guard allowContextRequest,case .object(let request)=try data.decode(ResourceJSON.self,forKey:.context_request),Set(request.keys)==Set(["version","variables"]) else{try reject()}}
}
private struct SourceTemplatePlanStorage:Decodable{
 var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var snippet:SourceTemplateSnippet;var available:Bool?;var notices:[String]?;var variable_context:SnippetVariableSnapshot?;var source_files_modified:Bool?;var context_unavailable:[String]?
}
struct SourceTemplateContextRequest:Decodable{
 struct Requirement:Decodable{var version:Int;var variables:[String]}
 var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var available:Bool;var variable_context:SnippetVariableSnapshot;var source_files_modified:Bool;var context_request:Requirement
}
private enum SourceTemplateRead:Decodable{case plan(SourceTemplatePlan),context(SourceTemplateContextRequest)
 init(from decoder:Decoder)throws{try sourceTemplateNoExtraEffects(decoder,allowContextRequest:true);let data=try decoder.container(keyedBy:TemplateEffectKey.self);if data.contains(.context_request){self = .context(try SourceTemplateContextRequest(from:decoder))}else{self = .plan(try SourceTemplatePlan(from:decoder))}}
}
struct SourceTemplatePlan:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var snippet:SourceTemplateSnippet;var available:Bool?;var notices:[String]?;var variable_context:SnippetVariableSnapshot?=nil;var source_files_modified:Bool?=nil;var context_unavailable:[String]?=nil
    init(from decoder:Decoder)throws{try sourceTemplateNoExtraEffects(decoder);let value=try SourceTemplatePlanStorage(from:decoder);version=value.version;template_protocol=value.template_protocol;path=value.path;selection=value.selection;source_hash=value.source_hash;workspace_id=value.workspace_id;language=value.language;catalog_fingerprint=value.catalog_fingerprint;template_id=value.template_id;origin=value.origin;replacement=value.replacement;snippet=value.snippet;available=value.available;notices=value.notices;variable_context=value.variable_context;source_files_modified=value.source_files_modified;context_unavailable=value.context_unavailable}
    func same(_ other:Self)->Bool {
        let identity=zip([path,source_hash,workspace_id,language,catalog_fingerprint,template_id,origin],[other.path,other.source_hash,other.workspace_id,other.language,other.catalog_fingerprint,other.template_id,other.origin]).allSatisfy{exactUTF8($0.0,$0.1)}
        guard (notices==nil)==(other.notices==nil) else{return false}
        let a=notices ?? [],b=other.notices ?? []
        return variable_context==other.variable_context && source_files_modified==other.source_files_modified && context_unavailable==other.context_unavailable && identity && version==other.version && template_protocol==other.template_protocol && selection==other.selection && replacement==other.replacement && snippet.same(other.snippet) && available==other.available && a.count==b.count && zip(a,b).allSatisfy{exactUTF8($0.0,$0.1)}
    }
}
</ui:copy>

## Validate expansion before preparing a local draft

The provider may replace exactly the captured UTF-16 selection. Pass its expanded fields through the existing bounded snippet validator and completion planner. That planner already refuses split scalars, invalid ancestry, inconsistent mirrors and missing exits; the template path supplies no command or additional edit. Keep snippet syntax and expansion bytes exact, including canonical-distinct Unicode literals.

<ui:copy id="source-templates-apple-policy">
func sourceTemplateCatalogMatches(_ catalog:SourceTemplateCatalog,path:String,source:String,selection:NSRange,variables:SnippetVariableSnapshot?=nil)->Bool {
    catalog.version==1 && catalog.template_protocol==(variables==nil ? 1:2) && catalog.variable_context==variables && (catalog.template_protocol==1 ? catalog.source_files_modified != true:catalog.source_files_modified==false) && exactUTF8(catalog.path,path) && catalog.selection==SourceTemplateRange(selection) && exactUTF8(catalog.source_hash,nativeSourceHash(source)) && !catalog.workspace_id.isEmpty && !catalog.catalog_fingerprint.isEmpty && Set(catalog.entries.map{Data($0.id.utf8)}).count==catalog.entries.count && catalog.entries.allSatisfy{["builtin","workspace"].contains($0.origin)}
}
func sourceTemplatePlanMatches(_ plan:SourceTemplatePlan,catalog:SourceTemplateCatalog,entry:SourceTemplateEntry)->Bool {
    plan.version==1 && plan.template_protocol==catalog.template_protocol && plan.variable_context==catalog.variable_context && (plan.template_protocol==1 ? plan.source_files_modified != true:plan.source_files_modified==false) && (plan.template_protocol==1 ? plan.available != false:plan.available==true) && exactUTF8(plan.path,catalog.path) && plan.selection==catalog.selection && plan.replacement==catalog.selection && exactUTF8(plan.source_hash,catalog.source_hash) && exactUTF8(plan.workspace_id,catalog.workspace_id) && exactUTF8(plan.language,catalog.language) && exactUTF8(plan.catalog_fingerprint,catalog.catalog_fingerprint) && exactUTF8(plan.template_id,entry.id) && exactUTF8(plan.origin,entry.origin) && entry.available
}
func sourceTemplateDraft(_ plan:SourceTemplatePlan,source:String,label:String)throws->CompletionDraft {
    guard [1,2].contains(plan.snippet.version) else{throw EngineError(message:"Unsupported template snippet protocol. Refresh the template catalog.")}
    let item:[String:Any]=["label":label,"insertTextFormat":2,"insertText":plan.snippet.syntax,"data":["hickory_snippet":try plan.snippet.body]]
    return try planCompletion(languageURI(plan.path),item:item,snapshot:SourceSnapshot(text:source,writable:true),from:plan.replacement.from,to:plan.replacement.to,expansion:try completionExpansion(item))
}
</ui:copy>

## Revalidate the reviewed provider immediately before Apply

Catalog and preview are read-only. A newer choice or closed popover retires held responses. Apply reads the same plan again and requires exact equality with the immutable preview, so a removed workspace declaration or changed deterministic expansion cannot silently reroll the reviewed code. Only the retained native editor commits, through its existing coordinated completion/snippet undo path. Nothing is saved to disk.

<ui:copy id="source-templates-apple-controller">
@MainActor @Observable final class SourceTemplatesModel {
    @ObservationIgnored var captureVariables:()->SnippetVariableSnapshot=SnippetVariableSnapshot.capture
    @ObservationIgnored var clipboardReader:(()->String?)?=nil
    var showing=false;private(set) var reading=false;private(set) var applying=false;private(set) var notice=""
    private(set) var catalog:SourceTemplateCatalog?;private(set) var preview:SourceTemplatePlan?;private(set) var selected:SourceTemplateEntry?
    private var ticket=UUID()
    private struct Context {
        weak var workspace:Workspace?;weak var buffer:DocumentBuffer?;weak var model:SourceTemplatesModel?;var engine:Engine;var namespace:UUID;var bufferID:UUID;var editorID:UUID;var sourceID:UUID;var source:String;var path:String;var selection:NSRange;var selectionID:UUID;var snippetIntent:Int;var variables:SnippetVariableSnapshot
        var body:[String:Any]{["version":2,"path":path,"source":source,"selection":SourceTemplateRange(selection).body,"variable_context":variables.body]}
        @MainActor func current()->Bool {
            guard let workspace,let buffer,workspace.sourceTemplates === model,!workspace.connecting,workspace.currentBuffer === buffer,workspace.sourceNavigationEpoch==namespace,workspace.engine?.origin==engine.origin,workspace.engine?.session === engine.session,buffer.id==bufferID,buffer.sourceEditIdentity==sourceID,buffer.selectionIdentity==selectionID,!buffer.resourceDeleted,!buffer.readOnly,!buffer.saving,exactUTF8(buffer.file.path,path),exactUTF8(buffer.text,source),buffer.selection==selection,buffer.snippets.intent==snippetIntent,let editor=buffer.sourceEditorState?() else{return false}
            return editor.identity==editorID && editor.attached && editor.editable && !editor.composing && editor.selections==1 && exactUTF8(editor.source,source) && editor.selection==selection && buffer.multiRanges.count<2
        }
    }
    private var context:Context?
    func current()->Bool {context?.current()==true}
    func close(){ticket=UUID();reading=false;applying=false;showing=false;preview=nil;selected=nil;context=nil}
    func sourceClosed(_ buffer:DocumentBuffer){if context?.bufferID==buffer.id{close()}}
    func open(_ workspace:Workspace) async {
        close();showing=true;catalog=nil;notice=""
        guard !workspace.connecting,let buffer=workspace.currentBuffer,let engine=workspace.engine,!buffer.resourceDeleted,!buffer.readOnly,!buffer.saving,buffer.multiRanges.count<2,let editor=buffer.sourceEditorState?(),editor.attached,editor.editable,!editor.composing,editor.selections==1,exactUTF8(editor.source,buffer.text),editor.selection==buffer.selection else{notice="Open one writable source selection and finish composition before choosing a reusable template.";return}
        buffer.showSnippetTools=false;buffer.language?.cancelCompletion()
        let held=Context(workspace:workspace,buffer:buffer,model:self,engine:engine,namespace:workspace.sourceNavigationEpoch,bufferID:buffer.id,editorID:editor.identity,sourceID:buffer.sourceEditIdentity,source:buffer.text,path:buffer.file.path,selection:editor.selection,selectionID:buffer.selectionIdentity,snippetIntent:buffer.snippets.intent,variables:captureVariables())
        context=held;let id=UUID();ticket=id;reading=true;notice="Reading reusable templates…"
        defer{if ticket==id{reading=false}}
        do {
            let value:SourceTemplateCatalog=try await engine.request("POST","/api/templates/catalog",body:held.body)
            guard ticket==id,showing else{return};guard held.current() else{throw EngineError(message:"Source editor or selection changed. Read templates again.")}
            guard sourceTemplateCatalogMatches(value,path:held.path,source:held.source,selection:held.selection,variables:held.variables) else{throw EngineError(message:"Template catalog identity or protocol did not match this source selection.")}
            catalog=value;notice=value.available ? "Choose a template to preview its expanded source.":"Reusable templates are unavailable for this source."
        }catch{if ticket==id{notice=error.localizedDescription}}
    }
    func choose(_ entry:SourceTemplateEntry) async {
        guard !applying,let context,let catalog,context.current(),catalog.available,catalog.entries.contains(where:{exactUTF8($0.id,entry.id) && exactUTF8($0.origin,entry.origin)}) else{notice="Template catalog is stale. Read it again.";return}
        guard entry.available else{notice=entry.reason ?? "This template provider is unavailable.";return}
        let id=UUID();ticket=id;reading=true;preview=nil;selected=entry;notice="Preparing template preview…"
        defer{if ticket==id{reading=false}}
        do {
            var captured=context,readingCatalog=catalog
            for attempt in 0..<2{
                guard ticket==id,showing else{return}
                var body=captured.body;body["catalog_fingerprint"]=readingCatalog.catalog_fingerprint;body["template_id"]=entry.id
                let reply:SourceTemplateRead=try await captured.engine.request("POST","/api/templates/plan",body:body)
                guard ticket==id,showing else{return};guard captured.current() else{throw EngineError(message:"Source editor or selection changed. Read templates again.")}
                switch reply {
                case .context(let request):
                    guard attempt==0,captured.variables.clipboard==nil,request.version==1,request.template_protocol==2,!request.available,!request.source_files_modified,request.context_request.version==1,request.context_request.variables==["CLIPBOARD"],request.variable_context==captured.variables,request.selection==readingCatalog.selection,request.replacement==readingCatalog.selection,zip([request.path,request.source_hash,request.workspace_id,request.language,request.catalog_fingerprint,request.template_id,request.origin],[readingCatalog.path,readingCatalog.source_hash,readingCatalog.workspace_id,readingCatalog.language,readingCatalog.catalog_fingerprint,entry.id,entry.origin]).allSatisfy({exactUTF8($0.0,$0.1)}) else{throw EngineError(message:"Unsupported, foreign or repeated template context request.")}
                    captured.variables.clipboard=captureSnippetClipboard(read:clipboardReader);guard ticket==id,showing else{return};guard captured.current() else{throw EngineError(message:"Source or selection changed while reading clipboard context.")};readingCatalog.variable_context=captured.variables;self.context=captured;self.catalog=readingCatalog
                case .plan(let value):
                    guard sourceTemplatePlanMatches(value,catalog:readingCatalog,entry:entry) else{throw EngineError(message:"Template plan no longer identifies the selected catalog entry.")};_ = try sourceTemplateDraft(value,source:captured.source,label:entry.label)
                    preview=value;let unavailable=try snippetContextNotice(["data":["hickory_context_unavailable":value.context_unavailable ?? []]]);notice="Preview only · Apply rechecks the provider and changes this draft through native undo."+(unavailable.isEmpty ? "":"\n"+unavailable);return
                }
            }
            throw EngineError(message:"Template context is unavailable. No edits were applied.")
        }catch{if ticket==id{notice=error.localizedDescription}}
    }
    @discardableResult func apply() async->Bool {
        guard !applying,!reading,let context,let catalog,let selected,let reviewed=preview,context.current() else{notice="Template preview is stale. Read and preview it again.";return false}
        let id=UUID();ticket=id;applying=true;preview=nil;notice="Rechecking reviewed template…"
        defer{if ticket==id{applying=false}}
        do {
            var body=context.body;body["catalog_fingerprint"]=catalog.catalog_fingerprint;body["template_id"]=selected.id
            let confirmed:SourceTemplatePlan=try await context.engine.request("POST","/api/templates/plan",body:body)
            guard ticket==id,showing else{return false};guard context.current() else{throw EngineError(message:"Source editor or selection changed. No template was applied.")}
            guard sourceTemplatePlanMatches(confirmed,catalog:catalog,entry:selected),reviewed.same(confirmed) else{throw EngineError(message:"Template provider changed or was revoked. Read a new catalog before applying.")}
            guard let buffer=context.buffer else{throw EngineError(message:"Source editor closed. No template was applied.")}
            let draft=try buffer.snippets.prepareCompletion(sourceTemplateDraft(confirmed,source:context.source,label:selected.label))
            guard buffer.applyCompletionDraft?(draft,context.selection)==true else{throw EngineError(message:"Native editor refused the template. Finish composition and preview again.")}
            close();notice="Template applied · unsaved draft";buffer.focusSource?();return true
        }catch{if ticket==id{notice=error.localizedDescription};return false}
    }
}
</ui:copy>

## Keep preview controls floating

The existing Edit menu and action search open this popover. Entry availability and provider notices remain readable; only explicit Apply changes source. Field navigation and choices then use the existing snippet commands and native Tab/Shift-Tab/Escape behavior.

<ui:copy id="source-templates-apple-view">
struct SourceTemplatesView:View {
    var workspace:Workspace
    private var model:SourceTemplatesModel{workspace.sourceTemplates}
    var body:some View {
        ScrollView{VStack(alignment:.leading,spacing:8){
            HStack{Text("Reusable templates").font(.headline);Spacer();Button("Close"){model.close();workspace.currentBuffer?.focusSource?()}}
            Button(model.reading ? "Reading templates…":"Read templates again"){Task{await model.open(workspace)}}.disabled(model.applying)
            if let catalog=model.catalog {
                Text("\(catalog.path) · \(catalog.language) · \(model.current() ? "current selection":"stale selection — read again")").font(.caption)
                ForEach(catalog.notices,id:\.self){Text($0).font(.caption).textSelection(.enabled)}
                ForEach(catalog.entries,id:\.exactIdentity){entry in VStack(alignment:.leading){Button("Preview \(entry.label)"){Task{await model.choose(entry)}}.disabled(!catalog.available || !entry.available || !model.current() || model.applying);Text(entry.description+" · "+entry.origin).font(.caption);if let reason=entry.reason{Text(reason).font(.caption).textSelection(.enabled)}}}
            }
            if !model.notice.isEmpty{Text(model.notice).font(.caption).textSelection(.enabled).accessibilityLabel("Template notice")}
            if let preview=model.preview,let selected=model.selected {
                Divider();Text("Preview · "+selected.label).font(.headline)
                Text("Replaces only the captured selection. Apply leaves an unsaved draft; native undo retains field editing.").font(.caption)
                Text(preview.snippet.text).font(.system(.body,design:.monospaced)).textSelection(.enabled)
                Button(model.applying ? "Applying…":"Apply template"){Task{_ = await model.apply()}}.disabled(model.applying || model.reading || !model.current()).accessibilityLabel("Apply reviewed template \(selected.label)")
            }
        }.padding()}.frame(minWidth:320,idealWidth:560,maxWidth:760,minHeight:220,idealHeight:500,maxHeight:720)
    }
}
</ui:copy>

## Witness native history and refusal before mutation

Focused fixtures exercise the actual native completion commit. A preview alone leaves text and undo unchanged; Apply and linked typing retain one editor and coordinated field history. Native emoji/combining-text input preserves exact bytes, UTF-16 mirror ranges and undo/redo field state. A changed provider under the same fingerprint, a revoked plan, changed editor/source/selection and composition all refuse without falling back to language commands.

The protocol 2 witnesses below extend the earlier executed protocol 1 evidence. They require exact variable-context structure and typed false effects, test one demanded clipboard capture including explicit denial, refuse repeated or stale requests, and retain the same snapshot through Apply. The actual-engine case also exercises captured filename/time/random values, derived uppercase mirrors, native Undo/Redo, unused clipboard defaults and source-lease retirement. On 2026-11-17 complete Swift source/test compilation and 12 focused checks passed: these 7 template checks, three shared context-handshake checks and two connected completion-plan checks. The exact real-engine template case then passed with zero skips through `python3 scripts/test-native-templates.py apple --engine target/source-edit-focus/debug/examples/engine_client`. Its distinct provider-completion context case also passed through `--provider-context`; the [native provider reading](native-snippet-context-provider-witness.md) records that scope. Durable native receipts and logs live under `target/native-template-smoke/`. Physical pointer, IME, operating-system clipboard prompts and accessibility acceptance remain unverified; these counts do not establish iOS runtime acceptance.

<ui:copy id="source-templates-apple-witness">
import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
@MainActor final class SourceTemplateTests:XCTestCase {
    func reply(_ source:String="value = 'old'\n")->([String:Any],[String:Any]) {
        let entry:[String:Any]=["id":"workspace.wrap","label":"Wrap selection","description":"Mirrored selected text","language":"python","origin":"workspace","available":true]
        let catalog:[String:Any]=["version":1,"template_protocol":1,"path":"main.py","selection":["from":9,"to":12],"source_hash":nativeSourceHash(source),"workspace_id":"fixture-workspace","language":"python","catalog_fingerprint":"catalog","available":true,"notices":[],"entries":[entry]]
        var plan=catalog;plan.removeValue(forKey:"entries");plan["template_id"]="workspace.wrap";plan["origin"]="workspace";plan["replacement"]=["from":9,"to":12]
        plan["snippet"]=["version":1,"syntax":"${1:${TM_SELECTED_TEXT}} + $1$0","text":"old + old","fields":[["id":0,"parent":NSNull(),"index":1,"from":0,"to":3,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":6,"to":9,"choices":[]],["id":2,"parent":NSNull(),"index":0,"from":9,"to":9,"choices":[]]]]
        return(catalog,plan)
    }
    static let variables=SnippetVariableSnapshot(clock:SnippetVariableClock(unix_millis:1704067200000,utc_offset_minutes:0),random_hex:"0123456789abcdef0123456789abcdef")
    func install(_ originalCatalog:[String:Any],_ originalPlan:[String:Any])throws {
        var catalog=originalCatalog,plan=originalPlan
        catalog["template_protocol"]=2;plan["template_protocol"]=2;catalog["variable_context"]=Self.variables.body;plan["variable_context"]=Self.variables.body;catalog["source_files_modified"]=false;plan["source_files_modified"]=false
        ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.resetRequests()
        ResponseProtocol.responsesByPath=["/api/templates/catalog":try JSONSerialization.data(withJSONObject:catalog),"/api/templates/plan":try JSONSerialization.data(withJSONObject:plan)]
    }
    func decode<T:Decodable>(_ value:[String:Any],as type:T.Type)throws->T {try JSONDecoder().decode(type,from:JSONSerialization.data(withJSONObject:value))}
    func testPlanBytesAndExactSelectionAreRequired()throws {
        let (raw,body)=reply(),catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplatePlan.self),entry=try XCTUnwrap(catalog.entries.first)
        XCTAssertTrue(sourceTemplateCatalogMatches(catalog,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3)));XCTAssertTrue(sourceTemplatePlanMatches(plan,catalog:catalog,entry:entry))
        var changed=plan;changed.replacement.to=13;XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:entry));changed=plan;changed.origin="builtin";XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:entry))
        var composed=plan;composed.snippet.syntax="$0é";composed.snippet.text="é";composed.snippet.fields=[SourceTemplateField(id:0,parent:nil,index:0,from:0,to:0,choices:[])]
        var decomposed=composed;decomposed.snippet.text="e\u{301}";XCTAssertFalse(composed.same(decomposed));decomposed=composed;decomposed.snippet.syntax="$0e\u{301}";XCTAssertFalse(composed.same(decomposed))
        XCTAssertThrowsError(try validateSourceTestJSON(Data(#"{"template_protocol":1e0}"#.utf8)))
        changed=plan;changed.snippet.fields[0].to=99;XCTAssertThrowsError(try sourceTemplateDraft(changed,source:"value = 'old'\n",label:"Bad field"))
    }
    func testVariableContextRequiresExactStructureAndTypedNoEffects()throws {
        let (original,originalPlan)=reply();var raw=original,body=originalPlan
        for key in ["template_protocol","variable_context","source_files_modified"]{raw[key]=key=="template_protocol" ? 2 as Any:key=="variable_context" ? Self.variables.body as Any:false as Any;body[key]=raw[key]}
        let catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplatePlan.self)
        XCTAssertTrue(sourceTemplateCatalogMatches(catalog,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3),variables:Self.variables));XCTAssertTrue(sourceTemplatePlanMatches(plan,catalog:catalog,entry:catalog.entries[0]))
        for effect in ["command","edit","workspaceEdit","additionalTextEdits"]{var changed=raw;changed[effect]=NSNull();XCTAssertThrowsError(try decode(changed,as:SourceTemplateCatalog.self));changed=body;changed[effect]=NSNull();XCTAssertThrowsError(try decode(changed,as:SourceTemplatePlan.self))}
        for value in [0 as Any,"false" as Any,true as Any]{var changed=raw;changed["source_files_modified"]=value;if let parsed=try? decode(changed,as:SourceTemplateCatalog.self){XCTAssertFalse(sourceTemplateCatalogMatches(parsed,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3),variables:Self.variables))}}
        var context=Self.variables.body;context["extra"]=true;raw["variable_context"]=context;XCTAssertThrowsError(try decode(raw,as:SourceTemplateCatalog.self))
        context=Self.variables.body;var clock=context["clock"] as! [String:Any];clock["extra"]=true;context["clock"]=clock;body["variable_context"]=context;XCTAssertThrowsError(try decode(body,as:SourceTemplatePlan.self))
    }
    #if os(macOS)
    func native()->(Workspace,DocumentBuffer,MacEditor,NSWindow) {
        _ = NSApplication.shared
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try! Engine(address:"http://fixture.invalid",session:URLSession(configuration:configuration)),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"value = 'old'\n",hash:"base")),editor=MacEditor(buffer)
        space.engine=engine;space.buffers[file.path]=buffer;space.layout.activate(file.path,in:"document-group");space.sourceTemplates.captureVariables={Self.variables}
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        editor.text.setSelectedRange(NSRange(location:9,length:3));buffer.selection=editor.text.selectedRange()
        return(space,buffer,editor,window)
    }
    func testDemandedClipboardIsCapturedOnceDeniedExplicitlyAndCannotOutliveSourceLease()async throws {
        for mode in ["available","denied","stale","repeated"] {
            let (space,buffer,editor,window)=native(),(catalog,basePlan)=reply();try install(catalog,basePlan)
            defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
            let model=space.sourceTemplates;var reads=0
            model.clipboardReader={reads+=1;if mode=="stale"{editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange()};return mode=="denied" ? nil:"copied"}
            var requested=basePlan;requested.removeValue(forKey:"snippet");requested["template_protocol"]=2;requested["source_files_modified"]=false;requested["variable_context"]=Self.variables.body;requested["available"]=false;requested["context_request"]=["version":1,"variables":["CLIPBOARD"]]
            var resolved=basePlan;resolved["template_protocol"]=2;resolved["source_files_modified"]=false
            var snapshot=Self.variables;snapshot.clipboard=SnippetClipboard(available:mode != "denied",text:mode=="denied" ? nil:"copied");resolved["variable_context"]=snapshot.body;resolved["context_unavailable"]=mode=="denied" ? ["CLIPBOARD"]:[]
            ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/templates/plan"{let value=ResponseProtocol.requestCount("/api/templates/plan")==1||mode=="repeated" ? requested:resolved;ResponseProtocol.responsesByPath["/api/templates/plan"]=try! JSONSerialization.data(withJSONObject:value)}}
            await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first));XCTAssertEqual(reads,1)
            if mode=="stale"||mode=="repeated"{XCTAssertNil(model.preview);XCTAssertEqual(buffer.text,"value = 'old'\n");continue}
            XCTAssertNotNil(model.preview,model.notice);let applied=await model.apply();XCTAssertTrue(applied,model.notice);XCTAssertEqual(reads,1);XCTAssertEqual(buffer.base,"value = 'old'\n")
            let requests=ResponseProtocol.requestBodies("/api/templates/plan");XCTAssertEqual(requests.count,3)
            let initial=requests[0]["variable_context"] as! [String:Any],retry=requests[1]["variable_context"] as! [String:Any]
            XCTAssertTrue(initial["clipboard"] is NSNull);XCTAssertEqual((retry["clipboard"] as? [String:Any])?["available"] as? Bool,mode != "denied")
            XCTAssertEqual(try JSONSerialization.data(withJSONObject:retry,options:.sortedKeys),try JSONSerialization.data(withJSONObject:requests[2]["variable_context"]!,options:.sortedKeys))
        }
    }
    func testPreviewApplyAndLinkedInputRetainActualNativeHistory()async throws {
        let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan);defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        let native=editor.text,manager=try XCTUnwrap(native.undoManager),source=buffer.text,identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
        await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview,space.sourceTemplates.notice);XCTAssertEqual(buffer.text,source);XCTAssertFalse(manager.canUndo)
        let applied=await space.sourceTemplates.apply();XCTAssertTrue(applied,space.sourceTemplates.notice);XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertEqual(native.selectedRange(),NSRange(location:9,length:3));XCTAssertNotNil(buffer.snippets.active)
        native.insertText("new",replacementRange:native.selectedRange());XCTAssertEqual(buffer.text,"value = 'new + new'\n");XCTAssertEqual(buffer.base,source)
        manager.undo();XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertNotNil(buffer.snippets.active);manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertNil(buffer.snippets.active)
        manager.redo();XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertNotNil(buffer.snippets.active);manager.redo();XCTAssertEqual(buffer.text,"value = 'new + new'\n")
        XCTAssertTrue(editor.text === native);XCTAssertTrue(native.undoManager === manager);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(ResponseProtocol.requestCount("/api/templates/plan"),2)
    }
    func testActualNativeUnicodeMirrorsRetainUTF16FieldsThroughHistory()async throws {
        let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan);defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));let accepted=await space.sourceTemplates.apply();XCTAssertTrue(accepted,space.sourceTemplates.notice)
        let value="😀e\u{301}",manager=try XCTUnwrap(editor.text.undoManager)
        editor.text.insertText(value,replacementRange:editor.text.selectedRange());XCTAssertTrue(exactUTF8(buffer.text,"value = '\(value) + \(value)'\n"));XCTAssertEqual(editor.text.selectedRange(),NSRange(location:13,length:0));XCTAssertEqual(buffer.snippets.active?.fields.map{[$0.from,$0.to]},[[9,13],[16,20],[20,20]])
        manager.undo();XCTAssertTrue(exactUTF8(buffer.text,"value = 'old + old'\n"));XCTAssertEqual(buffer.snippets.active?.fields.map{[$0.from,$0.to]},[[9,12],[15,18],[18,18]])
        manager.redo();XCTAssertTrue(exactUTF8(buffer.text,"value = '\(value) + \(value)'\n"));XCTAssertEqual(buffer.snippets.active?.fields.map{[$0.from,$0.to]},[[9,13],[16,20],[20,20]])
    }
    func testSourceEditUndoSelectionAndCompositionRetirePreview()async throws {
        for change in ["editUndo","selection","selectionUndo","deleted","composition","editor","model","closed"] {
            let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan)
            await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview)
            let source=buffer.text,original=try XCTUnwrap(buffer.sourceEditorState),state=try XCTUnwrap(original()),reviewed=space.sourceTemplates
            switch change {
            case "editUndo":buffer.text="changed";buffer.text=source
            case "selection":editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange()
            case "selectionUndo":let original=buffer.selection;editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange();editor.text.setSelectedRange(original);buffer.selection=editor.text.selectedRange()
            case "deleted":buffer.resourceDeleted=true
            case "composition":buffer.sourceEditorState={var value=state;value.composing=true;return value}
            case "editor":buffer.sourceEditorState={var value=state;value.identity=UUID();return value}
            case "model":space.sourceTemplates=SourceTemplatesModel()
            default:space.close("main.py");space.show("main.py")
            }
            let before=ResponseProtocol.requestCount("/api/templates/plan"),accepted=await reviewed.apply();XCTAssertFalse(accepted,change);XCTAssertEqual(buffer.text,source,change);XCTAssertEqual(ResponseProtocol.requestCount("/api/templates/plan"),before,change);window.contentView=nil
        }
        ResponseProtocol.responsesByPath=[:]
    }
    func testProviderRequeryCannotRerollReviewedUnicodeOrApplyRevokedPlan()async throws {
        for revoked in [false,true] {
            let (space,buffer,editor,window)=native(),(catalog,raw)=reply();defer{_ = editor};var plan=raw
            plan["snippet"]=["version":1,"syntax":"$0é","text":"é","fields":[["id":0,"parent":NSNull(),"index":0,"from":0,"to":0,"choices":[]]]];try install(catalog,plan)
            await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview)
            if revoked{ResponseProtocol.status=409;ResponseProtocol.responsesByPath["/api/templates/plan"]=Data(#"{"error":"Template provider revoked"}"#.utf8)}else{var snippet=plan["snippet"] as! [String:Any];snippet["text"]="e\u{301}";plan["snippet"]=snippet;ResponseProtocol.responsesByPath["/api/templates/plan"]=try JSONSerialization.data(withJSONObject:plan)}
            let accepted=await space.sourceTemplates.apply();XCTAssertFalse(accepted);XCTAssertEqual(buffer.text,"value = 'old'\n");XCTAssertNil(buffer.snippets.active);window.contentView=nil
        }
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:]
    }
    func testLiveNativeSourceTemplateCatalogAndHistory()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Set HICKORY_NATIVE_TEMPLATE_ORIGIN/WORKSPACE to the maintained isolated native-template fixture.")}
        func canonicalPath(_ path:String)throws->String{let pointer=try XCTUnwrap(realpath(path,nil));defer{free(pointer)};return String(cString:pointer)}
        let canonicalWorkspace=try canonicalPath(directory),owned=URL(fileURLWithPath:directory),marker=owned.appendingPathComponent(".hickory-native-template-fixture")
        XCTAssertTrue(FileManager.default.fileExists(atPath:marker.path));guard FileManager.default.fileExists(atPath:marker.path) else{return}
        let manifest=owned.appendingPathComponent(".hickory/templates.json"),canonicalManifest=try canonicalPath(manifest.path);XCTAssertTrue(canonicalManifest.hasPrefix(canonicalWorkspace+"/"));guard canonicalManifest.hasPrefix(canonicalWorkspace+"/") else{return}
        let originalManifest=try Data(contentsOf:manifest);defer{try? originalManifest.write(to:manifest,options:.atomic)}
        let engine=try Engine(address:origin),space=Workspace(),path=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_INPUT"] ?? "main.py",tree=try await engine.files()
        func find(_ files:[FileNode],_ path:String)->FileNode?{for file in files{if exactUTF8(file.path,path){return file};if let child=find(file.children ?? [],path){return child}};return nil}
        let file=try XCTUnwrap(find(tree.tree,path));space.engine=engine;space.rootFiles=tree.tree;space.debugger=NativeDebugger(engine:engine)
        let buffer=try await space.loadBuffer(file);XCTAssertFalse(buffer.readOnly);space.layout.activate(file.path,in:"document-group")
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{space.sourceTemplates.close();space.debugger?.close();space.sourceTestIndex?.stop();for item in space.buffers.values{item.language?.close()};window.orderOut(nil)}
        func editor(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        for _ in 0..<200{if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let native=try XCTUnwrap(editor(host)),manager=try XCTUnwrap(native.undoManager),scroll=try XCTUnwrap(native.enclosingScrollView),source=buffer.text
        XCTAssertEqual(source,"value = 'old'\n");native.insertText("# prior history\n",replacementRange:NSRange(location:native.string.utf16.count,length:0));manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo)
        native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();let geometry=scroll.convert(scroll.bounds,to:host),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
        let command=try XCTUnwrap(space.commands().first(where:{$0.id=="source.templates"}));XCTAssertNil(command.reason);command.run()
        let model=space.sourceTemplates;for _ in 0..<400{if model.catalog != nil{break};try await Task.sleep(for:.milliseconds(25))}
        let catalog=try XCTUnwrap(model.catalog,model.notice),entry=try XCTUnwrap(catalog.entries.first(where:{exactUTF8($0.id,"workspace.wrap")}),model.notice)
        XCTAssertEqual(catalog.workspace_id,nativeSourceHash(canonicalWorkspace),"Owned fixture workspace must match the provider catalog");guard exactUTF8(catalog.workspace_id,nativeSourceHash(canonicalWorkspace)) else{return}
        let unsupported=try XCTUnwrap(catalog.entries.first(where:{exactUTF8($0.id,"workspace.unsupported")}));XCTAssertFalse(unsupported.available);XCTAssertFalse(unsupported.reason?.isEmpty ?? true)
        await model.choose(entry);XCTAssertEqual(model.preview?.snippet.text,"old + old",model.notice);XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo);XCTAssertEqual(native.selectedRange(),NSRange(location:9,length:3))
        let accepted=await model.apply();XCTAssertTrue(accepted,model.notice);guard accepted else{return}
        XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertEqual(native.selectedRange(),NSRange(location:9,length:3));XCTAssertNotNil(buffer.snippets.active)
        native.insertText("new",replacementRange:native.selectedRange());XCTAssertEqual(buffer.text,"value = 'new + new'\n")
        manager.undo();XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertNotNil(buffer.snippets.active);manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertNil(buffer.snippets.active)
        manager.redo();manager.redo();XCTAssertEqual(buffer.text,"value = 'new + new'\n");XCTAssertNotNil(buffer.snippets.active)
        XCTAssertTrue(editor(host) === native);XCTAssertTrue(native.undoManager === manager);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(buffer.base,source)
        manager.undo();manager.undo();native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange()
        model.captureVariables={Self.variables};var clipboardReads=0;model.clipboardReader={clipboardReads+=1;return "copied"}
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.context"})));XCTAssertEqual(model.preview?.snippet.text,"main.py|2024|012345|old|old|OLD",model.notice);XCTAssertEqual(clipboardReads,0)
        let contextApplied=await model.apply();XCTAssertTrue(contextApplied,model.notice);XCTAssertEqual(buffer.text,"value = 'main.py|2024|012345|old|old|OLD'\n")
        let installed=try XCTUnwrap(buffer.snippets.active,"Derived template metadata must install an active native session");XCTAssertEqual(installed.fields.filter{$0.transform != nil}.count,1);XCTAssertNotNil(buffer.snippets.commit)
        let selectedField=try XCTUnwrap(installed.fields.first(where:{$0.id==installed.current}));XCTAssertNil(selectedField.transform);XCTAssertEqual(native.selectedRange(),NSRange(location:selectedField.from,length:selectedField.to-selectedField.from))
        native.insertText("new",replacementRange:native.selectedRange());XCTAssertEqual(buffer.text,"value = 'main.py|2024|012345|new|new|NEW'\n")
        manager.undo();manager.undo();XCTAssertEqual(buffer.text,source);manager.redo();manager.redo();XCTAssertEqual(buffer.text,"value = 'main.py|2024|012345|new|new|NEW'\n");manager.undo();manager.undo()
        for denied in [false,true]{
            native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();let before=clipboardReads
            model.clipboardReader={clipboardReads+=1;return denied ? nil:"copied"}
            await model.open(space);XCTAssertEqual(clipboardReads,before);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.clipboard"})))
            XCTAssertEqual(clipboardReads,before+1);XCTAssertEqual(model.preview?.snippet.text,denied ? "fallback":"copied",model.notice);XCTAssertEqual(model.preview?.context_unavailable ?? [],denied ? ["CLIPBOARD"]:[])
            let applied=await model.apply();XCTAssertTrue(applied,model.notice);XCTAssertEqual(clipboardReads,before+1);XCTAssertEqual(buffer.text,"value = '\(denied ? "fallback":"copied")'\n");manager.undo();XCTAssertEqual(buffer.text,source);manager.redo();XCTAssertEqual(buffer.text,"value = '\(denied ? "fallback":"copied")'\n");manager.undo()
        }
        native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();let beforeUnused=clipboardReads
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.unused-clipboard"})));XCTAssertEqual(model.preview?.snippet.text,"old",model.notice);XCTAssertEqual(clipboardReads,beforeUnused);model.close()
        model.clipboardReader={clipboardReads+=1;native.setSelectedRange(NSRange(location:0,length:0));buffer.selection=native.selectedRange();return "stale"}
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.clipboard"})));XCTAssertNil(model.preview);XCTAssertEqual(buffer.text,source)
        native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();XCTAssertTrue(editor(host) === native);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(buffer.base,source)
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{exactUTF8($0.id,"workspace.wrap")})));XCTAssertNotNil(model.preview,model.notice)
        try Data(#"{"version":1,"templates":[]}"#.utf8).write(to:manifest,options:.atomic)
        let revoked=await model.apply();XCTAssertFalse(revoked);XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo);XCTAssertNil(buffer.snippets.active)
        try originalManifest.write(to:manifest,options:.atomic)
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{exactUTF8($0.id,"workspace.wrap")})));XCTAssertNotNil(model.preview,model.notice)
        native.setSelectedRange(NSRange(location:0,length:0));buffer.selection=native.selectedRange();let stale=await model.apply();XCTAssertFalse(stale);XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo)
        XCTAssertTrue(editor(host) === native);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);model.close()
        let disk=try await engine.file(file);XCTAssertEqual(disk.content,source,"Template application is a local draft, not a save")
        if let generated=find((try await engine.files()).tree,"generated.py") {await space.open(generated);let readonly=try XCTUnwrap(space.currentBuffer);XCTAssertTrue(readonly.readOnly);let before=readonly.text;XCTAssertNotNil(space.commands().first(where:{$0.id=="source.templates"})?.reason);await model.open(space);XCTAssertNil(model.catalog);XCTAssertEqual(readonly.text,before)}else{XCTFail("Fixture must include generated.py for readonly refusal")}
    }
    #endif
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceTemplates.swift"><ui:paste select="#source-templates-apple-values"/><ui:paste select="#source-templates-apple-policy"/><ui:paste select="#source-templates-apple-controller"/><ui:paste select="#source-templates-apple-view"/></ui:file>
<ui:file path="apps/apple-simple/Tests/SourceTemplateTests.swift"><ui:paste select="#source-templates-apple-witness"/></ui:file>
</ui:doc>
