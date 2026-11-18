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
struct SourceTemplateLanguageContext:Codable {
 var version:Int;var scope:String;var language:String?;var languages:[String];var targets:[String];var reason:String?;var project_entry:String?;var source_hashes:[String:String];var graph_fingerprint:String;var fingerprint:String
 func same(_ other:Self)->Bool{let encoder=JSONEncoder();encoder.outputFormatting=[.sortedKeys];return (try? encoder.encode(self))==(try? encoder.encode(other))}
 func valid(path:String,source:String,project:String?,language actual:String)->Bool{
  func digest(_ value:String)->Bool{value.utf8.count==64 && value.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
  guard version==1,!scope.isEmpty,digest(fingerprint),digest(graph_fingerprint),languages.count<=1024,targets.count<=1024,source_hashes.count<=10000,source_hashes.values.allSatisfy(digest),source_hashes[path].map({exactUTF8($0,nativeSourceHash(source))})==true,exactUTF8(actual,language ?? "unknown"),language==nil || (languages.count==1 && exactUTF8(languages[0],language!)),exactUTF8(project_entry,project) || (project==nil && exactUTF8(project_entry,path)),let encoded=try? JSONEncoder().encode(self),encoded.count<=1024*1024 else{return false};return true
 }
}
struct SourceTemplateReadingRequest:Decodable {var version:Int;var kind:String}
struct SourceTemplateEntry:Decodable,Identifiable {var id:String;var label:String;var description:String;var language:String;var origin:String;var available:Bool;var reason:String?;var exactIdentity:Data{Data(id.utf8)}}
private struct SourceTemplateCatalogStorage:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var available:Bool;var notices:[String];var entries:[SourceTemplateEntry];var variable_context:SnippetVariableSnapshot?=nil;var source_files_modified:Bool?=nil;var source_context:SourceTemplateLanguageContext?=nil;var transform_program_versions:[Int]?=nil;var reading_context_request:SourceTemplateReadingRequest?=nil
}
struct SourceTemplateCatalog:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var available:Bool;var notices:[String];var entries:[SourceTemplateEntry];var variable_context:SnippetVariableSnapshot?=nil;var source_files_modified:Bool?=nil;var source_context:SourceTemplateLanguageContext?=nil;var transform_program_versions:[Int]?=nil;var reading_context_request:SourceTemplateReadingRequest?=nil
    init(from decoder:Decoder)throws{try sourceTemplateNoExtraEffects(decoder,allowReadingRequest:true);let value=try SourceTemplateCatalogStorage(from:decoder);version=value.version;template_protocol=value.template_protocol;path=value.path;selection=value.selection;source_hash=value.source_hash;workspace_id=value.workspace_id;language=value.language;catalog_fingerprint=value.catalog_fingerprint;available=value.available;notices=value.notices;entries=value.entries;variable_context=value.variable_context;source_files_modified=value.source_files_modified;source_context=value.source_context;transform_program_versions=value.transform_program_versions;reading_context_request=value.reading_context_request}
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
private enum TemplateEffectKey:String,CodingKey{case command,additionalTextEdits,workspaceEdit,edit,context_request,variable_context,reading_context_request,source_context}
private func sourceTemplateNoExtraEffects(_ decoder:Decoder,allowContextRequest:Bool=false,allowReadingRequest:Bool=false)throws{
 let data=try decoder.container(keyedBy:TemplateEffectKey.self)
 guard ![TemplateEffectKey.command,.additionalTextEdits,.workspaceEdit,.edit].contains(where:{data.contains($0)}) else{throw EngineError(message:"Template contains unsupported additional effects. No edits were applied.")}
 func reject()throws->Never{throw EngineError(message:"Malformed template variable context. No edits were applied.")}
 if data.contains(.variable_context),try !data.decodeNil(forKey:.variable_context){
  guard case .object(let context)=try data.decode(ResourceJSON.self,forKey:.variable_context),Set(context.keys)==Set(["clock","random_hex","clipboard"]) else{try reject()}
  if let clock=context["clock"]{switch clock{case .null:break;case .object(let fields):guard Set(fields.keys)==Set(["unix_millis","utc_offset_minutes"]) else{try reject()};for value in fields.values{switch value{case .integer,.unsigned:break;default:try reject()}};default:try reject()}}
  if let random=context["random_hex"]{switch random{case .null,.string:break;default:try reject()}}
  if let clipboard=context["clipboard"]{switch clipboard{case .null:break;case .object(let fields):guard Set(fields.keys)==Set(["available","text"]),let available=fields["available"],case .bool=available else{try reject()};if let text=fields["text"]{switch text{case .null,.string:break;default:try reject()}};default:try reject()}}
 }
 if data.contains(.reading_context_request),try !data.decodeNil(forKey:.reading_context_request){guard allowReadingRequest,case .object(let request)=try data.decode(ResourceJSON.self,forKey:.reading_context_request),Set(request.keys)==Set(["version","kind"]),case .integer(1)?=request["version"],case .string("construction_entry")?=request["kind"] else{try reject()}}
 if data.contains(.source_context),try !data.decodeNil(forKey:.source_context){guard case .object(let context)=try data.decode(ResourceJSON.self,forKey:.source_context),Set(context.keys)==Set(["version","scope","language","languages","targets","reason","project_entry","source_hashes","graph_fingerprint","fingerprint"]) else{try reject()}}
 if data.contains(.context_request){guard allowContextRequest,case .object(let request)=try data.decode(ResourceJSON.self,forKey:.context_request),Set(request.keys)==Set(["version","variables"]) else{try reject()}}
}
private struct SourceTemplatePlanStorage:Decodable{
 var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var snippet:SourceTemplateSnippet;var available:Bool?;var notices:[String]?;var variable_context:SnippetVariableSnapshot?;var source_files_modified:Bool?;var source_context:SourceTemplateLanguageContext?;var transform_program_versions:[Int]?;var context_unavailable:[String]?
}
struct SourceTemplateContextRequest:Decodable{
 struct Requirement:Decodable{var version:Int;var variables:[String]}
 var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var available:Bool;var variable_context:SnippetVariableSnapshot;var source_files_modified:Bool;var source_context:SourceTemplateLanguageContext?;var transform_program_versions:[Int]?;var context_request:Requirement
}
private enum SourceTemplateRead:Decodable{case plan(SourceTemplatePlan),context(SourceTemplateContextRequest)
 init(from decoder:Decoder)throws{try sourceTemplateNoExtraEffects(decoder,allowContextRequest:true);let data=try decoder.container(keyedBy:TemplateEffectKey.self);if data.contains(.context_request){self = .context(try SourceTemplateContextRequest(from:decoder))}else{self = .plan(try SourceTemplatePlan(from:decoder))}}
}
struct SourceTemplatePlan:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var snippet:SourceTemplateSnippet;var available:Bool?;var notices:[String]?;var variable_context:SnippetVariableSnapshot?=nil;var source_files_modified:Bool?=nil;var source_context:SourceTemplateLanguageContext?=nil;var transform_program_versions:[Int]?=nil;var reading_context_request:SourceTemplateReadingRequest?=nil;var context_unavailable:[String]?=nil
    init(from decoder:Decoder)throws{try sourceTemplateNoExtraEffects(decoder);let value=try SourceTemplatePlanStorage(from:decoder);version=value.version;template_protocol=value.template_protocol;path=value.path;selection=value.selection;source_hash=value.source_hash;workspace_id=value.workspace_id;language=value.language;catalog_fingerprint=value.catalog_fingerprint;template_id=value.template_id;origin=value.origin;replacement=value.replacement;snippet=value.snippet;available=value.available;notices=value.notices;variable_context=value.variable_context;source_files_modified=value.source_files_modified;source_context=value.source_context;transform_program_versions=value.transform_program_versions;context_unavailable=value.context_unavailable}
    func same(_ other:Self)->Bool {
        let identity=zip([path,source_hash,workspace_id,language,catalog_fingerprint,template_id,origin],[other.path,other.source_hash,other.workspace_id,other.language,other.catalog_fingerprint,other.template_id,other.origin]).allSatisfy{exactUTF8($0.0,$0.1)}
        guard (notices==nil)==(other.notices==nil) else{return false}
        let a=notices ?? [],b=other.notices ?? []
        return ((source_context==nil && other.source_context==nil) || (source_context.flatMap{left in other.source_context.map{left.same($0)}}==true)) && transform_program_versions==other.transform_program_versions && variable_context==other.variable_context && source_files_modified==other.source_files_modified && context_unavailable==other.context_unavailable && identity && version==other.version && template_protocol==other.template_protocol && selection==other.selection && replacement==other.replacement && snippet.same(other.snippet) && available==other.available && a.count==b.count && zip(a,b).allSatisfy{exactUTF8($0.0,$0.1)}
    }
}
func sourceTemplateCatalogMatches(_ catalog:SourceTemplateCatalog,path:String,source:String,selection:NSRange,variables:SnippetVariableSnapshot?=nil)->Bool {
    catalog.version==1 && catalog.template_protocol==(variables==nil ? 1:2) && catalog.variable_context==variables && (catalog.template_protocol==1 ? catalog.source_files_modified != true:catalog.source_files_modified==false) && exactUTF8(catalog.path,path) && catalog.selection==SourceTemplateRange(selection) && exactUTF8(catalog.source_hash,nativeSourceHash(source)) && !catalog.workspace_id.isEmpty && !catalog.catalog_fingerprint.isEmpty && Set(catalog.entries.map{Data($0.id.utf8)}).count==catalog.entries.count && catalog.entries.allSatisfy{["builtin","workspace"].contains($0.origin) && exactUTF8($0.language,catalog.language)}
}
func sourceTemplatePlanMatches(_ plan:SourceTemplatePlan,catalog:SourceTemplateCatalog,entry:SourceTemplateEntry)->Bool {
    ((plan.source_context==nil && catalog.source_context==nil) || (plan.source_context.flatMap{left in catalog.source_context.map{left.same($0)}}==true)) && plan.transform_program_versions==catalog.transform_program_versions && plan.version==1 && plan.template_protocol==catalog.template_protocol && plan.variable_context==catalog.variable_context && (plan.template_protocol==1 ? plan.source_files_modified != true:plan.source_files_modified==false) && (plan.template_protocol==1 ? plan.available != false:plan.available==true) && exactUTF8(plan.path,catalog.path) && plan.selection==catalog.selection && plan.replacement==catalog.selection && exactUTF8(plan.source_hash,catalog.source_hash) && exactUTF8(plan.workspace_id,catalog.workspace_id) && exactUTF8(plan.language,catalog.language) && exactUTF8(plan.catalog_fingerprint,catalog.catalog_fingerprint) && exactUTF8(plan.template_id,entry.id) && exactUTF8(plan.origin,entry.origin) && entry.available
}
func sourceTemplateDraft(_ plan:SourceTemplatePlan,source:String,label:String)throws->CompletionDraft {
    guard [1,2].contains(plan.snippet.version) else{throw EngineError(message:"Unsupported template snippet protocol. Refresh the template catalog.")}
    let item:[String:Any]=["label":label,"insertTextFormat":2,"insertText":plan.snippet.syntax,"data":["hickory_snippet":try plan.snippet.body]]
    return try planCompletion(languageURI(plan.path),item:item,snapshot:SourceSnapshot(text:source,writable:true),from:plan.replacement.from,to:plan.replacement.to,expansion:try completionExpansion(item))
}
@MainActor private func captureSourceTemplateContributors(_ workspace:Workspace)throws->(body:[String:Any],current:()->Bool){
 let lease=ResourceAcquisitionLease(workspace,current:{true}),all=Array(workspace.buffers.values)+Array(workspace.retainedResourceEditors.values),unique=Dictionary(grouping:all,by:{ObjectIdentifier($0)}).values.compactMap{$0.first};var sources:[String:String]=[:],unaddressed:[String:String]=[:],missing:Set<String>=[];var bytes=0
 guard unique.count<=4096 else{throw EngineError(message:"Too many retained template contributors.")}
 for buffer in unique {let size=buffer.text.utf8.count;guard size<=8*1024*1024,size<=32*1024*1024-bytes else{throw EngineError(message:"Template contributors exceed the interactive text budget.")};bytes+=size
  if buffer.resourceDeleted {if !buffer.file.path.hasPrefix("/"){missing.insert(buffer.file.path)};continue}
  if buffer.isUntitledDraft || buffer.file.path.hasPrefix("/"){unaddressed[buffer.id.uuidString]=buffer.text;continue}
  guard sources[buffer.file.path]==nil else{throw EngineError(message:"Template contributor addresses are ambiguous.")};sources[buffer.file.path]=buffer.text
 }
 for path in sources.keys{missing.remove(path)}
 return(["version":1,"project_entry":NSNull(),"sources":sources,"unaddressed_sources":unaddressed,"missing_sources":Array(missing).sorted()],{lease.current()})
}
@MainActor @Observable final class SourceTemplatesModel {
    @ObservationIgnored var captureVariables:()->SnippetVariableSnapshot=SnippetVariableSnapshot.capture
    @ObservationIgnored var clipboardReader:(()->String?)?=nil
    var showing=false;private(set) var reading=false;private(set) var applying=false;private(set) var notice=""
    private(set) var catalog:SourceTemplateCatalog?;private(set) var preview:SourceTemplatePlan?;private(set) var selected:SourceTemplateEntry?
    private var ticket=UUID()
    private struct Context {
        weak var workspace:Workspace?;weak var buffer:DocumentBuffer?;weak var model:SourceTemplatesModel?;var engine:Engine;var namespace:UUID;var bufferID:UUID;var editorID:UUID;var sourceID:UUID;var source:String;var path:String;var selection:NSRange;var selectionID:UUID;var snippetIntent:Int;var variables:SnippetVariableSnapshot;var readingContext:[String:Any]?=nil;var contributorsCurrent:()->Bool={true};var entryCurrent:()->Bool={true}
        var body:[String:Any]{var value:[String:Any]=["version":2,"path":path,"source":source,"selection":SourceTemplateRange(selection).body,"variable_context":variables.body];if let readingContext{value["reading_context"]=readingContext;value["transform_program_versions"]=[2]};return value}
        @MainActor func current()->Bool {
            guard contributorsCurrent(),entryCurrent(),let workspace,let buffer,workspace.sourceTemplates === model,!workspace.connecting,workspace.currentBuffer === buffer,workspace.sourceNavigationEpoch==namespace,workspace.engine?.origin==engine.origin,workspace.engine?.session === engine.session,buffer.id==bufferID,buffer.sourceEditIdentity==sourceID,buffer.selectionIdentity==selectionID,!buffer.resourceDeleted,!buffer.readOnly,!buffer.saving,exactUTF8(buffer.file.path,path),exactUTF8(buffer.text,source),buffer.selection==selection,buffer.snippets.intent==snippetIntent,let editor=buffer.sourceEditorState?() else{return false}
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
        var held=Context(workspace:workspace,buffer:buffer,model:self,engine:engine,namespace:workspace.sourceNavigationEpoch,bufferID:buffer.id,editorID:editor.identity,sourceID:buffer.sourceEditIdentity,source:buffer.text,path:buffer.file.path,selection:editor.selection,selectionID:buffer.selectionIdentity,snippetIntent:buffer.snippets.intent,variables:captureVariables())
        do{let graph=try captureSourceTemplateContributors(workspace);held.readingContext=graph.body;held.contributorsCurrent=graph.current}catch{notice=error.localizedDescription;return}
        context=held;let id=UUID();ticket=id;reading=true;notice="Reading reusable templates…"
        defer{if ticket==id{reading=false}}
        do {
            for attempt in 0..<2 {
                let value:SourceTemplateCatalog=try await engine.request("POST","/api/templates/catalog",body:held.body)
                guard ticket==id,showing else{return};guard held.current() else{throw EngineError(message:"Source editor, selection or contributors changed. Read templates again.")}
                let project=held.readingContext?["project_entry"] as? String
                guard sourceTemplateCatalogMatches(value,path:held.path,source:held.source,selection:held.selection,variables:held.variables),value.transform_program_versions==[2],value.source_context?.valid(path:held.path,source:held.source,project:project,language:value.language)==true else{throw EngineError(message:"Template catalog language context did not match this source selection.")}
                if let request=value.reading_context_request {
                    guard attempt==0,request.version==1,request.kind=="construction_entry",!value.available,value.entries.isEmpty,held.readingContext?["entry_selection"]==nil,let runtime=workspace.sourceWorkspaceID,let namespace=workspace.sourceRootAddress else{throw EngineError(message:"Original construction entry is unavailable. Source remains readable.")}
                    let original=held
                    let entry=try await WorkspaceEntrySelections.read(load:{let value:ResourceJSON=try await engine.request("GET","/api/workspace/entry");return try JSONSerialization.jsonObject(with:JSONEncoder().encode(value))},workspace:runtime,namespace:namespace,current:{original.current()})
                    guard ticket==id,showing,original.current(),entry.current() else{throw EngineError(message:"Construction entry or source contributors changed.")}
                    held.readingContext?["project_entry"]=entry.entryPath.map{$0 as Any} ?? NSNull();held.readingContext?["entry_selection"]=try entry.proof.body();held.entryCurrent=entry.current;context=held;continue
                }
                catalog=value;notice=value.available ? "Choose a template to preview its expanded source.":"Reusable templates are unavailable for this source.";return
            }
            throw EngineError(message:"Repeated construction entry demand. Source remains readable.")
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
                    guard attempt==0,captured.variables.clipboard==nil,request.version==1,request.template_protocol==2,!request.available,!request.source_files_modified,request.context_request.version==1,request.context_request.variables==["CLIPBOARD"],request.variable_context==captured.variables,request.transform_program_versions==readingCatalog.transform_program_versions,((request.source_context==nil && readingCatalog.source_context==nil) || (request.source_context.flatMap{left in readingCatalog.source_context.map{left.same($0)}}==true)),request.selection==readingCatalog.selection,request.replacement==readingCatalog.selection,zip([request.path,request.source_hash,request.workspace_id,request.language,request.catalog_fingerprint,request.template_id,request.origin],[readingCatalog.path,readingCatalog.source_hash,readingCatalog.workspace_id,readingCatalog.language,readingCatalog.catalog_fingerprint,entry.id,entry.origin]).allSatisfy({exactUTF8($0.0,$0.1)}) else{throw EngineError(message:"Unsupported, foreign or repeated template context request.")}
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
