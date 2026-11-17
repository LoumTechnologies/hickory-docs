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
struct SourceTemplateCatalog:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var available:Bool;var notices:[String];var entries:[SourceTemplateEntry]
}
struct SourceTemplateField:Codable {
    var id:Int;var parent:Int?;var index:Int;var from:Int;var to:Int;var choices:[String]
    var body:[String:Any]{["id":id,"parent":parent.map{$0 as Any} ?? NSNull(),"index":index,"from":from,"to":to,"choices":choices]}
    func same(_ other:Self)->Bool{id==other.id && parent==other.parent && index==other.index && from==other.from && to==other.to && choices.count==other.choices.count && zip(choices,other.choices).allSatisfy{exactUTF8($0.0,$0.1)}}
}
struct SourceTemplateSnippet:Decodable {
    var version:Int;var syntax:String;var text:String;var fields:[SourceTemplateField]
    var body:[String:Any]{["version":version,"syntax":syntax,"text":text,"fields":fields.map(\.body)]}
    func same(_ other:Self)->Bool{version==other.version && exactUTF8(syntax,other.syntax) && exactUTF8(text,other.text) && fields.count==other.fields.count && zip(fields,other.fields).allSatisfy{$0.0.same($0.1)}}
}
struct SourceTemplatePlan:Decodable {
    var version:Int;var template_protocol:Int;var path:String;var selection:SourceTemplateRange;var source_hash:String;var workspace_id:String;var language:String;var catalog_fingerprint:String;var template_id:String;var origin:String;var replacement:SourceTemplateRange;var snippet:SourceTemplateSnippet;var available:Bool?;var notices:[String]?
    func same(_ other:Self)->Bool {
        let identity=zip([path,source_hash,workspace_id,language,catalog_fingerprint,template_id,origin],[other.path,other.source_hash,other.workspace_id,other.language,other.catalog_fingerprint,other.template_id,other.origin]).allSatisfy{exactUTF8($0.0,$0.1)}
        guard (notices==nil)==(other.notices==nil) else{return false}
        let a=notices ?? [],b=other.notices ?? []
        return identity && version==other.version && template_protocol==other.template_protocol && selection==other.selection && replacement==other.replacement && snippet.same(other.snippet) && available==other.available && a.count==b.count && zip(a,b).allSatisfy{exactUTF8($0.0,$0.1)}
    }
}
func sourceTemplateCatalogMatches(_ catalog:SourceTemplateCatalog,path:String,source:String,selection:NSRange)->Bool {
    catalog.version==1 && catalog.template_protocol==1 && exactUTF8(catalog.path,path) && catalog.selection==SourceTemplateRange(selection) && exactUTF8(catalog.source_hash,nativeSourceHash(source)) && !catalog.workspace_id.isEmpty && !catalog.catalog_fingerprint.isEmpty && Set(catalog.entries.map{Data($0.id.utf8)}).count==catalog.entries.count && catalog.entries.allSatisfy{["builtin","workspace"].contains($0.origin)}
}
func sourceTemplatePlanMatches(_ plan:SourceTemplatePlan,catalog:SourceTemplateCatalog,entry:SourceTemplateEntry)->Bool {
    plan.version==1 && plan.template_protocol==1 && plan.available != false && exactUTF8(plan.path,catalog.path) && plan.selection==catalog.selection && plan.replacement==catalog.selection && exactUTF8(plan.source_hash,catalog.source_hash) && exactUTF8(plan.workspace_id,catalog.workspace_id) && exactUTF8(plan.language,catalog.language) && exactUTF8(plan.catalog_fingerprint,catalog.catalog_fingerprint) && exactUTF8(plan.template_id,entry.id) && exactUTF8(plan.origin,entry.origin) && entry.available
}
func sourceTemplateDraft(_ plan:SourceTemplatePlan,source:String,label:String)throws->CompletionDraft {
    guard plan.snippet.version==1 else{throw EngineError(message:"Unsupported template snippet protocol. Refresh the template catalog.")}
    let item:[String:Any]=["label":label,"insertTextFormat":2,"insertText":plan.snippet.syntax,"data":["hickory_snippet":plan.snippet.body]]
    return try planCompletion(languageURI(plan.path),item:item,snapshot:SourceSnapshot(text:source,writable:true),from:plan.replacement.from,to:plan.replacement.to,expansion:try completionExpansion(item))
}
@MainActor @Observable final class SourceTemplatesModel {
    var showing=false;private(set) var reading=false;private(set) var applying=false;private(set) var notice=""
    private(set) var catalog:SourceTemplateCatalog?;private(set) var preview:SourceTemplatePlan?;private(set) var selected:SourceTemplateEntry?
    private var ticket=UUID()
    private struct Context {
        weak var workspace:Workspace?;weak var buffer:DocumentBuffer?;weak var model:SourceTemplatesModel?;var engine:Engine;var namespace:UUID;var bufferID:UUID;var editorID:UUID;var sourceID:UUID;var source:String;var path:String;var selection:NSRange;var selectionID:UUID;var snippetIntent:Int
        var body:[String:Any]{["version":1,"path":path,"source":source,"selection":SourceTemplateRange(selection).body]}
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
        let held=Context(workspace:workspace,buffer:buffer,model:self,engine:engine,namespace:workspace.sourceNavigationEpoch,bufferID:buffer.id,editorID:editor.identity,sourceID:buffer.sourceEditIdentity,source:buffer.text,path:buffer.file.path,selection:editor.selection,selectionID:buffer.selectionIdentity,snippetIntent:buffer.snippets.intent)
        context=held;let id=UUID();ticket=id;reading=true;notice="Reading reusable templates…"
        defer{if ticket==id{reading=false}}
        do {
            let value:SourceTemplateCatalog=try await engine.request("POST","/api/templates/catalog",body:held.body)
            guard ticket==id,showing else{return};guard held.current() else{throw EngineError(message:"Source editor or selection changed. Read templates again.")}
            guard sourceTemplateCatalogMatches(value,path:held.path,source:held.source,selection:held.selection) else{throw EngineError(message:"Template catalog identity or protocol did not match this source selection.")}
            catalog=value;notice=value.available ? "Choose a template to preview its expanded source.":"Reusable templates are unavailable for this source."
        }catch{if ticket==id{notice=error.localizedDescription}}
    }
    func choose(_ entry:SourceTemplateEntry) async {
        guard !applying,let context,let catalog,context.current(),catalog.available,catalog.entries.contains(where:{exactUTF8($0.id,entry.id) && exactUTF8($0.origin,entry.origin)}) else{notice="Template catalog is stale. Read it again.";return}
        guard entry.available else{notice=entry.reason ?? "This template provider is unavailable.";return}
        let id=UUID();ticket=id;reading=true;preview=nil;selected=entry;notice="Preparing template preview…"
        defer{if ticket==id{reading=false}}
        do {
            var body=context.body;body["catalog_fingerprint"]=catalog.catalog_fingerprint;body["template_id"]=entry.id
            let value:SourceTemplatePlan=try await context.engine.request("POST","/api/templates/plan",body:body)
            guard ticket==id,showing else{return};guard context.current() else{throw EngineError(message:"Source editor or selection changed. Read templates again.")}
            guard sourceTemplatePlanMatches(value,catalog:catalog,entry:entry) else{throw EngineError(message:"Template plan no longer identifies the selected catalog entry.")}
            _ = try sourceTemplateDraft(value,source:context.source,label:entry.label)
            preview=value;notice="Preview only · Apply rechecks the provider and changes this draft through native undo."
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
