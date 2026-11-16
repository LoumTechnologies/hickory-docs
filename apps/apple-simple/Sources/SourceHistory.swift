import Foundation
import Observation
import SwiftUI
struct SourceSaveAttempt {var id:String=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased();var source:String;var base:String;var hash:String;var guardToken:String;var revision:UUID
    func matches(source:String,base:String,hash:String,guardToken:String,revision:UUID)->Bool{exactUTF8(self.source,source)&&exactUTF8(self.base,base)&&exactUTF8(self.hash,hash)&&exactUTF8(self.guardToken,guardToken)&&self.revision==revision}
}
struct SourceGuardedSave:Decodable {
    struct Status:Decodable{var available:Bool;var outcome:String;var reason:String?;var id:String?}
    var version:Int;var history_protocol:Int;var path:String;var request_id:String;var source_hash:String;var save_guard:String;var hash:String?;var history:Status
}
extension Engine {
    @MainActor func saveSourceWithHistory(_ buffer:DocumentBuffer,text:String,base:String,hash:String,requestID:String,guardToken:String) async throws->String {
        let path=buffer.file.path
        var body:[String:Any]=["version":1,"request_id":requestID,"save_guard":guardToken]
        let route:String
        if let id=buffer.file.doc_id{route="/api/source-history/docs/"+id;body["source"]=text;body["expected"]=["source":base,"disk":base]}
        else{route="/api/source-history/file";body["path"]=path;body["content"]=text;body["base_hash"]=hash}
        let value:SourceGuardedSave=try await request("PUT",route,body:body)
        guard value.version==1,value.history_protocol==1,exactUTF8(value.path,path),exactUTF8(value.request_id,requestID),exactUTF8(value.source_hash,nativeSourceHash(text)),!value.save_guard.isEmpty,value.history.outcome=="published",exactUTF8(buffer.file.path,path),buffer.savedGuard.map({exactUTF8($0,guardToken)})==true else{throw EngineError(message:"Save response does not confirm this exact source, request and physical baseline. Preserve the draft and inspect Local history.")}
        buffer.savedGuard=value.save_guard;buffer.saveGuardReason=nil;buffer.savedHistoryNotice=value.history.available ? "":value.history.reason ?? "Saved bytes were confirmed; local history is unavailable for this provider."
        return value.hash ?? hash
    }
}
struct SourceHistoryEntry:Decodable,Identifiable {var id:String;var at:UInt64;var outcome:String;var fingerprint:String;var before:ResourceSaved;var after:ResourceSaved;var available:Bool;var reason:String?;var retry_allowed:Bool?;var diff:String?}
struct SourceHistoryReading:Decodable {var version:Int;var history_protocol:Int;var workspace:String;var namespace:String;var path:String;var current:ResourceSaved;var current_identity:String;var entries:[SourceHistoryEntry];var notices:[String]}
struct SourceHistoryDraft:Decodable {var version:Int;var history_protocol:Int;var workspace:String;var namespace:String;var path:String;var id:String;var fingerprint:String;var current_identity:String;var current_hash:String;var draft_hash:String;var side:String;var source:String;var consequences:String;var notice:String}
func sourceHistoryHex(_ value:String,_ count:Int)->Bool{value.utf8.count==count && value.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
func validSourceHistory(_ value:SourceHistoryReading,path:String)->Bool {
    value.version==1&&value.history_protocol==1&&exactUTF8(value.path,path) && !value.workspace.isEmpty && !value.current_identity.isEmpty&&value.current.kind=="file"&&value.current.text.map{nativeSourceHash($0)==value.current.sha256}==true&&Set(value.entries.map{Data($0.id.utf8)}).count==value.entries.count&&value.entries.allSatisfy{entry in
        sourceHistoryHex(entry.id,32)&&sourceHistoryHex(entry.fingerprint,64)&&["prepared","published","failed","unknown"].contains(entry.outcome)&&[entry.before,entry.after].allSatisfy{saved in saved.kind=="missing" || saved.kind=="file" && saved.text.map{nativeSourceHash($0)==saved.sha256}==true}&&(!entry.available||entry.outcome=="published")
    }
}
@MainActor @Observable final class SourceHistoryModel {
    var showing=false;private(set) var busy=false;private(set) var notice="";private(set) var reading:SourceHistoryReading?;private(set) var selected:SourceHistoryEntry?;private(set) var side="before"
    private var ticket=UUID()
    private struct Context {
        weak var workspace:Workspace?;weak var buffer:DocumentBuffer?;weak var model:SourceHistoryModel?;var engine:Engine;var namespace:UUID;var address:String;var sourceID:UUID;var editorID:UUID;var bufferID:UUID;var path:String;var source:String;var selection:NSRange;var snippetIntent:Int
        @MainActor func current()->Bool {
            guard let workspace,let buffer,workspace.sourceHistory === model,workspace.currentBuffer === buffer,!workspace.connecting,workspace.engine?.origin==engine.origin,workspace.engine?.session === engine.session,workspace.sourceNavigationEpoch==namespace,workspace.sourceRootAddress.map({exactUTF8($0,address)})==true,buffer.id==bufferID,buffer.sourceEditIdentity==sourceID,exactUTF8(buffer.file.path,path),exactUTF8(buffer.text,source),!buffer.readOnly,!buffer.saving,buffer.selection==selection,buffer.snippets.intent==snippetIntent,let editor=buffer.sourceEditorState?() else{return false}
            return editor.identity==editorID&&editor.attached&&editor.editable && !editor.composing&&editor.selections==1&&buffer.multiRanges.count<2&&exactUTF8(editor.source,source)&&editor.selection==selection
        }
    }
    private var context:Context?
    func current()->Bool{context?.current()==true}
    func close(){ticket=UUID();showing=false;busy=false;context=nil;selected=nil}
    func sourceClosed(_ buffer:DocumentBuffer){if context?.bufferID==buffer.id{close()}}
    func open(_ workspace:Workspace) async {
        close();showing=true;reading=nil;notice="Reading local history…"
        guard !workspace.connecting,let buffer=workspace.currentBuffer,let engine=workspace.engine,let address=workspace.sourceRootAddress,!buffer.resourceDeleted,!buffer.saving,let editor=buffer.sourceEditorState?(),editor.attached,exactUTF8(editor.source,buffer.text),editor.selection==buffer.selection else{notice="Open an attached source editor before reading local history.";return}
        let held=Context(workspace:workspace,buffer:buffer,model:self,engine:engine,namespace:workspace.sourceNavigationEpoch,address:address,sourceID:buffer.sourceEditIdentity,editorID:editor.identity,bufferID:buffer.id,path:buffer.file.path,source:buffer.text,selection:editor.selection,snippetIntent:buffer.snippets.intent);context=held
        let id=UUID();ticket=id;busy=true;defer{if ticket==id{busy=false}}
        do {let value:SourceHistoryReading=try await engine.request("GET","/api/source-history",query:[URLQueryItem(name:"path",value:held.path)])
            guard ticket==id,showing else{return};guard workspace.currentBuffer === buffer,buffer.id==held.bufferID,buffer.sourceEditIdentity==held.sourceID,exactUTF8(buffer.file.path,held.path),workspace.sourceNavigationEpoch==held.namespace,workspace.engine?.session === engine.session,workspace.engine?.origin==engine.origin,exactUTF8(value.namespace,held.address),validSourceHistory(value,path:held.path) else{throw EngineError(message:"Local history belongs to a changed source or unsupported protocol. Read it again.")}
            reading=value;notice="Local history is retained on this machine. Choose a save to compare; restoration changes only the current draft."
        }catch{if ticket==id{notice=error.localizedDescription}}
    }
    func choose(_ entry:SourceHistoryEntry,side:String){guard !busy,let reading,reading.entries.contains(where:{exactUTF8($0.id,entry.id)&&exactUTF8($0.fingerprint,entry.fingerprint)}),["before","after"].contains(side) else{return};selected=entry;self.side=side;notice=entry.reason ?? "Preview only · current unsaved text is retained until explicit Restore to draft."}
    @discardableResult func allowNewSaveAttempt()->Bool {
        guard let context,context.current(),let selected,selected.outcome=="failed",selected.retry_allowed==true,let buffer=context.buffer,let pending=buffer.saveHistoryAttempt,exactUTF8(pending.id,selected.id),pending.matches(source:buffer.text,base:buffer.base,hash:buffer.hash,guardToken:buffer.savedGuard ?? "",revision:buffer.sourceEditIdentity) else{notice="No matching definitive failed save is available for a new attempt.";return false}
        buffer.saveHistoryAttempt=nil;notice="A new explicit Save may now be prepared. No text or saved file changed.";return true
    }
    var preview:String?{guard let selected else{return nil};return side=="before" ? selected.before.text:selected.after.text}
    @discardableResult func restore() async->Bool {
        guard !busy,let context,context.current(),let reading,let selected,selected.available,let source=preview,let currentHash=reading.current.sha256 else{notice="History review or editor is stale or read-only. Current draft is retained.";return false}
        let id=UUID();ticket=id;busy=true;self.selected=nil;defer{if ticket==id{busy=false}}
        let chosenSide=side
        do {
            let value:SourceHistoryDraft=try await context.engine.request("POST","/api/source-history/restore-draft",body:["version":1,"path":context.path,"id":selected.id,"fingerprint":selected.fingerprint,"workspace":reading.workspace,"namespace":reading.namespace,"current_identity":reading.current_identity,"current_hash":currentHash,"draft_hash":nativeSourceHash(context.source),"side":chosenSide])
            guard ticket==id,showing,context.current(),value.version==1,value.history_protocol==1,exactUTF8(value.workspace,reading.workspace),exactUTF8(value.namespace,context.address),exactUTF8(value.path,context.path),exactUTF8(value.id,selected.id),exactUTF8(value.fingerprint,selected.fingerprint),exactUTF8(value.current_identity,reading.current_identity),exactUTF8(value.current_hash,currentHash),exactUTF8(value.draft_hash,nativeSourceHash(context.source)),exactUTF8(value.side,chosenSide),exactUTF8(value.source,source),value.consequences=="unknown",let buffer=context.buffer else{throw EngineError(message:"History authority or source editor changed. No draft was replaced.")}
            let edit=SourceReplacement(from:0,to:context.source.utf16.count,text:source),plan=ChangePlan(title:"Restore local history to draft",patches:[SourcePatch(path:context.path,before:context.source,after:source,edits:[edit])])
            let draft=CompletionDraft(path:context.path,before:context.source,plan:plan,insertionFrom:0,selection:CompletionSelection(from:0,to:0))
            guard buffer.applyCompletionDraft?(draft,context.selection)==true else{throw EngineError(message:"Native editor refused the coordinated draft restoration. Current text is retained.")}
            notice=value.notice;self.context=nil;buffer.focusSource?();return true
        }catch{if ticket==id{notice=error.localizedDescription};return false}
    }
}
struct SourceCreated:Decodable {var version:Int;var history_protocol:Int;var workspace:String;var namespace:String;var path:String;var source:String;var source_hash:String;var hash:String;var request_id:String;var save_guard:String;var doc_id:String?;var history:SourceGuardedSave.Status}
@MainActor struct SourceCreationAttempt {
    struct Input {weak var buffer:DocumentBuffer?;var identity:ObjectIdentifier;var path:String;var sourceID:UUID;var base:String;var guardToken:Data?;var readOnly:Bool;var deleted:Bool}
    var id:String;var path:String;var source:String;var revision:UUID;var workspace:String;var namespace:String;var epoch:UUID;var origin:URL;var session:ObjectIdentifier;var inputs:[Input];var body:[String:Any]
    func current(_ owner:Workspace,_ sourceBuffer:DocumentBuffer)->Bool {
        guard owner.sourceNavigationEpoch==epoch,owner.engine?.origin==origin,owner.engine.map({ObjectIdentifier($0.session)})==session,owner.sourceWorkspaceID.map({exactUTF8($0,workspace)})==true,owner.sourceRootAddress.map({exactUTF8($0,namespace)})==true,sourceBuffer.sourceEditIdentity==revision,exactUTF8(sourceBuffer.text,source) else{return false}
        let live=Array(owner.buffers.values)+Array(owner.retainedResourceEditors.values),keys=Set(live.map{ObjectIdentifier($0)})
        guard keys==Set(inputs.map{$0.identity}) else{return false}
        return inputs.allSatisfy{value in guard let buffer=value.buffer else{return false};return exactUTF8(buffer.file.path,value.path)&&buffer.sourceEditIdentity==value.sourceID&&exactUTF8(buffer.base,value.base)&&buffer.savedGuard.map({Data($0.utf8)})==value.guardToken&&buffer.readOnly==value.readOnly&&buffer.resourceDeleted==value.deleted}
    }
}
extension Workspace {
    func createSourceCopy(_ source:DocumentBuffer,path:String)async throws->DocumentBuffer {
        guard let engine,let workspace=sourceWorkspaceID,let namespace=sourceRootAddress else{throw EngineError(message:"Workspace creation authority is still loading. Retain the draft and refresh the workspace before Save As.")}
        if buffers.keys.contains(where:{$0==path && !exactUTF8($0,path)}){throw EngineError(message:"The destination aliases another native editor name. Preserve the source and choose an unambiguous filename.")}
        if let editor=source.sourceEditorState?(),editor.composing || !exactUTF8(editor.source,source.text){throw EngineError(message:"Finish source composition before Save As. Original text is retained.")}
        let attempt:SourceCreationAttempt
        if let previous=source.creationAttempt,exactUTF8(previous.path,path),previous.current(self,source){attempt=previous}
        else{
            let all=Array(buffers.values)+Array(retainedResourceEditors.values),unique=Dictionary(grouping:all,by:{ObjectIdentifier($0)}).values.compactMap{$0.first}
            let inputs=unique.map{SourceCreationAttempt.Input(buffer:$0,identity:ObjectIdentifier($0),path:$0.file.path,sourceID:$0.sourceEditIdentity,base:$0.base,guardToken:$0.savedGuard.map{Data($0.utf8)},readOnly:$0.readOnly,deleted:$0.resourceDeleted)}
            var ownership:[String:String]=[:],guards:[String:String]=[:],unaddressed:[String:String]=[:];var exact:Set<Data>=[]
            for buffer in unique where buffer !== source && buffer.file.generated_by==nil && (buffer.resourceDeleted || buffer.file.path.hasPrefix("Untitled-") || buffer.file.path.hasPrefix("/")){unaddressed[buffer.id.uuidString]=buffer.text}
            for buffer in unique where !buffer.resourceDeleted && !buffer.readOnly && !buffer.file.path.hasPrefix("Untitled-") && !buffer.file.path.hasPrefix("/") {
                guard exact.insert(Data(buffer.file.path.utf8)).inserted else{continue}
                if let state=buffer.sourceEditorState?(),state.composing || !exactUTF8(state.source,buffer.text){throw EngineError(message:"Finish source composition before capturing creation ownership.")}
                guard ownership[buffer.file.path]==nil else{throw EngineError(message:"Retained source names have ambiguous Unicode spellings. Their text is retained; creation ownership cannot be merged.")}
                ownership[buffer.file.path]=buffer.text;if let guardToken=buffer.savedGuard{guards[buffer.file.path]=guardToken}
            }
            let id=UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased()
            let capture:[String:Any]=["context":id,"edit":["changes":[:]],"drafts":[:],"project_entry":NSNull(),"ownership_sources":ownership]
            let body:[String:Any]=["version":1,"workspace":workspace,"path":path,"source":source.text,"request_id":id,"ownership":capture,"saved_guards":guards,"unaddressed_sources":unaddressed]
            attempt=SourceCreationAttempt(id:id,path:path,source:source.text,revision:source.sourceEditIdentity,workspace:workspace,namespace:namespace,epoch:sourceNavigationEpoch,origin:engine.origin,session:ObjectIdentifier(engine.session),inputs:inputs,body:body);source.creationAttempt=attempt
        }
        let value:SourceCreated=try await engine.request("POST","/api/source-history/create",body:attempt.body)
        guard attempt.current(self,source),value.version==1,value.history_protocol==1,exactUTF8(value.workspace,attempt.workspace),exactUTF8(value.namespace,attempt.namespace),exactUTF8(value.path,path),exactUTF8(value.request_id,attempt.id),exactUTF8(value.source_hash,nativeSourceHash(attempt.source)),exactUTF8(value.source,attempt.source),!value.save_guard.isEmpty,value.history.outcome=="published" else{throw EngineError(message:"Creation reply does not confirm this exact source and editor namespace. Original text is retained; inspect the destination and Local history.")}
        guard buffers[path]==nil else{throw EngineError(message:"A different editor now owns the destination. The original source is retained.")}
        let file=FileNode(name:(path as NSString).lastPathComponent,path:path,dir:false,doc_id:value.doc_id)
        return try adoptCreatedSource(source,file:file,text:FileText(path:path,content:value.source,hash:value.hash,save_guard:value.save_guard),engine:engine)
    }
}
struct SourceHistoryView:View {
    @Bindable var workspace:Workspace
    var body:some View {
        @Bindable var model=workspace.sourceHistory
        VStack(alignment:.leading,spacing:8){
            HStack{Text("Local history").font(.headline);Spacer();Button("Close"){model.close();workspace.currentBuffer?.focusSource?()}}
            Text(model.notice).font(.caption).textSelection(.enabled)
            if let reading=model.reading {
                ForEach(reading.notices,id:\.self){Text($0).font(.caption)}
                ScrollView{VStack(alignment:.leading,spacing:6){ForEach(reading.entries){entry in
                    HStack{Text(entry.outcome.capitalized).font(.caption);Text(Date(timeIntervalSince1970:Double(entry.at)/1000),style:.time).font(.caption);Spacer();Button("Before"){model.choose(entry,side:"before")}.accessibilityLabel("Preview source before save \(entry.id)");Button("Saved after"){model.choose(entry,side:"after")}.accessibilityLabel("Preview intended saved source \(entry.id)")}
                }
                if reading.entries.isEmpty{Text("No ordinary saves are retained for this source.").font(.caption)}
                if let diff=model.selected?.diff{Text("Recorded save diff").font(.caption);Text(diff).font(.system(.caption,design:.monospaced)).textSelection(.enabled)}
                if model.selected != nil,model.preview==nil{Text("No source existed on this historical side. Deletion requires the separate resource recovery workflow.").font(.caption)}
                if let source=model.preview{Text(model.side=="before" ? "Before save":"Intended saved after").font(.caption);Text(source).font(.system(.caption,design:.monospaced)).textSelection(.enabled).frame(maxWidth:.infinity,alignment:.leading);Divider();Text("Current draft").font(.caption);Text(workspace.currentBuffer?.text ?? "").font(.system(.caption,design:.monospaced)).textSelection(.enabled).frame(maxWidth:.infinity,alignment:.leading)}
                }}
                if model.selected?.retry_allowed==true{Button("Allow a new Save attempt"){_ = model.allowNewSaveAttempt()}.disabled(model.busy || !model.current())}
                Button("Restore selected text to draft"){Task{_ = await model.restore()}}.disabled(model.busy||model.selected?.available != true || model.preview==nil || !model.current()).help("Replace this draft through native undo. Saved files and generated outputs are unchanged.")
            }
        }.padding(12).frame(minWidth:340,idealWidth:560,maxWidth:720,minHeight:180,idealHeight:440,maxHeight:640)
    }
}
