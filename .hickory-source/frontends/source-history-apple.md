<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read an ordinary save and restore its text through native draft undo

For maintainers following macOS and iOS Local history from the shared menu into the retained source editor. History is a compact floating reading: opening it changes no source, saved baseline, selection or undo stack. Explicit restoration replaces only the captured current draft through the same coordinated native transaction used by completion. Historical construction text remains readable even when its compiler consequences are unknown.

## Retain request and evidence identity independently of the displayed text

Save retries retain their original request only while the exact submitted source and baseline remain the same. The engine decides whether its old confirmed outcome still belongs to the current physical source; a renderer cannot infer success from equal text. A failed request leaves the current draft intact.

<ui:copy id="source-history-apple-values">
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
</ui:copy>

## Lease the actual editor while reading and reviewing one source side

Bind source text, edit identity, selection, native editor UUID, workspace namespace and transport. Equal text after an edit/undo cannot revive the review. Closing the reading retires pending network replies. A restore rechecks saved identity and ownership on the engine, then checks this lease again before one local native undo action. Saving, deletion, composition or a new editor lifetime refuses while retaining all readable history.

<ui:copy id="source-history-apple-controller">
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
</ui:copy>

## Keep Save As on one no-clobber request and its captured construction inputs

Build the full source once; never create an empty destination and learn an identity from a later read. Retain the exact request across an uncertain reply only while the source and complete editor roster remain the same. A confirmed receipt supplies the initial token and bytes directly to the new buffer. Changing editors or construction inputs cannot make an old reply adopt a new namespace occupant.

<ui:copy id="source-history-apple-create">
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
</ui:copy>

## Keep before, saved after and the current draft visible in one compact reading

No toolbar or sidebar is added. The existing action search and main menu open this floating reading; native editor geometry stays fixed beneath it. Outcome labels describe recorded publication evidence, not compiler validity. Historical text is selectable even when restoration is unavailable.

<ui:copy id="source-history-apple-view">
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
</ui:copy>

## Verify the retained native path and refuse stale editor authority

The model witnesses use the actual macOS editor and UndoManager, not a replacement text model. The opt-in engine witness performs an ordinary save before reading its retained history and restoring into an independently edited draft. These checks do not substitute for physical input, VoiceOver or device IME acceptance.

## Verify the saved, draft and created-source paths through the retained editor

The focused canonical macOS run executed all thirteen selected history, affected-save and detached-copy cases with zero failures or skips. Its owned-engine witness exercised Local history, guarded restoration and native Undo/Redo, then File → Save As against the strict creation endpoint. The same Buffer, NSTextView, editor UUID and geometry survived; the new saved baseline and physical guard came directly from the confirmed reply, and the original saved file was unchanged. The created file's history retained an explicit missing before-side. This is programmatic native integration acceptance, not physical keyboard/pointer or VoiceOver acceptance; those remain unverified. The source-copy namespace transition retires old path-bound debugger/provider authority without replacing the editor or its local undo stack.

<ui:copy id="source-history-apple-tests">
import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
@MainActor final class SourceHistoryTests:XCTestCase {
    func saved(_ text:String)->[String:Any]{["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":420]}
    func reading()->[String:Any]{["version":1,"history_protocol":1,"workspace":"workspace","namespace":"/fixture","path":"a.txt","current":saved("saved"),"current_identity":"physical-current","entries":[["id":String(repeating:"1",count:32),"at":1,"outcome":"published","fingerprint":String(repeating:"a",count:64),"before":saved("old"),"after":saved("saved"),"available":true]],"notices":[]]}
    func draft(_ source:String)->[String:Any]{["version":1,"history_protocol":1,"workspace":"workspace","namespace":"/fixture","path":"a.txt","id":String(repeating:"1",count:32),"fingerprint":String(repeating:"a",count:64),"current_identity":"physical-current","current_hash":nativeSourceHash("saved"),"draft_hash":nativeSourceHash(source),"side":"before","source":"old","consequences":"unknown","notice":"Restored to draft; compiler consequences are unknown."]}
    func testHistoryEvidenceMustMatchExactPathsAndSavedHashes()throws {
        let bytes=try JSONSerialization.data(withJSONObject:reading()),value=try JSONDecoder().decode(SourceHistoryReading.self,from:bytes)
        XCTAssertThrowsError(try validateSourceTestJSON(Data(#"{"history_protocol":1e0}"#.utf8)));XCTAssertTrue(validSourceHistory(value,path:"a.txt"));XCTAssertFalse(validSourceHistory(value,path:"A.txt"))
        var corrupt=value;corrupt.entries[0].before.text="tampered";XCTAssertFalse(validSourceHistory(corrupt,path:"a.txt"));corrupt=value;corrupt.entries[0].outcome="failed";XCTAssertFalse(validSourceHistory(corrupt,path:"a.txt"))
        var malformed=value;malformed.entries[0].id=String(repeating:"😀",count:32);XCTAssertFalse(validSourceHistory(malformed,path:"a.txt"));malformed=value;malformed.entries[0].fingerprint=String(repeating:"A",count:64);XCTAssertFalse(validSourceHistory(malformed,path:"a.txt"))
        var alias=value;alias.path="é.txt";XCTAssertFalse(validSourceHistory(alias,path:"e\u{301}.txt"))
    }
    #if os(macOS)
    func native()->(Workspace,DocumentBuffer,MacEditor,NSWindow){
        _ = NSApplication.shared;let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try! Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),file=FileNode(name:"a.txt",path:"a.txt",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"draft",hash:"base")),editor=MacEditor(buffer)
        buffer.base="saved";space.engine=engine;space.sourceRootAddress="/fixture";space.buffers[file.path]=buffer;space.layout.activate(file.path,in:"document-group")
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;editor.text.setSelectedRange(NSRange(location:5,length:0));buffer.selection=editor.text.selectedRange();return(space,buffer,editor,window)
    }
    func install(_ source:String)throws{ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.resetRequests();ResponseProtocol.responsesByPath=["/api/source-history":try JSONSerialization.data(withJSONObject:reading()),"/api/source-history/restore-draft":try JSONSerialization.data(withJSONObject:draft(source))]}
    func testStrictCreationRetainsExactConsumedIntentAndDirectConfirmedBaseline()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        space.sourceWorkspaceID="runtime-workspace";buffer.savedGuard="captured-before";ResponseProtocol.resetRequests();ResponseProtocol.responsesByPath=[:];ResponseProtocol.status=500;ResponseProtocol.response=Data(#"{"error":"reply lost"}"#.utf8)
        do{_ = try await space.createSourceCopy(buffer,path:"copy.txt");XCTFail("Lost response cannot confirm creation")}catch{}
        let attempt=try XCTUnwrap(buffer.creationAttempt),original=try XCTUnwrap(ResponseProtocol.requestBodies("/api/source-history/create").last),identity=editor.text,selection=editor.text.selectedRange(),geometry=editor.scroll.frame
        XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.base,"saved");XCTAssertNil(space.buffers["copy.txt"])
        ResponseProtocol.status=200;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["version":1,"history_protocol":1,"workspace":"runtime-workspace","namespace":"/fixture","path":"copy.txt","source":buffer.text,"source_hash":nativeSourceHash(buffer.text),"hash":"confirmed-hash","request_id":attempt.id,"save_guard":"confirmed-copy","history":["available":true,"outcome":"published"]])
        let created=try await space.createSourceCopy(buffer,path:"copy.txt")
        XCTAssertEqual(try JSONSerialization.data(withJSONObject:original,options:.sortedKeys),try JSONSerialization.data(withJSONObject:try XCTUnwrap(ResponseProtocol.requestBodies("/api/source-history/create").last),options:.sortedKeys));XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0);XCTAssertEqual(ResponseProtocol.requestCount("/api/files/op"),0)
        XCTAssertEqual(created.base,"draft");XCTAssertEqual(created.savedGuard,"confirmed-copy");XCTAssertFalse(created.dirty);XCTAssertNil(space.buffers["a.txt"]);XCTAssertTrue(space.buffers["copy.txt"] === buffer);XCTAssertTrue(created === buffer);XCTAssertTrue(editor.text === identity);XCTAssertEqual(editor.text.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
    }
    func testCreationOwnershipCapturesDirtyConstructionAndRefusesUnboundBaseline()async throws {
        let(space,buffer,_,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        space.sourceWorkspaceID="runtime-workspace";let doc=DocumentBuffer(file:FileNode(name:"project.md",path:"project.md",dir:false,doc_id:"project"),text:FileText(path:"project.md",content:"saved construction",hash:"old"));doc.text="<h:file path=\"copy.txt\">draft output</h:file>";space.buffers["project.md"]=doc;ResponseProtocol.resetRequests()
        do{_ = try await space.createSourceCopy(buffer,path:"copy.txt");XCTFail("Unbound construction must refuse before effects")}catch{}
        XCTAssertEqual(ResponseProtocol.requestCount("/api/source-history/create"),0);XCTAssertNil(space.buffers["copy.txt"]);XCTAssertEqual(doc.text,"<h:file path=\"copy.txt\">draft output</h:file>")
    }
    func testOnlyReviewedDefinitiveFailureReleasesMatchingConsumedSaveAttempt()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        buffer.savedGuard="captured-before";let id=String(repeating:"1",count:32);buffer.saveHistoryAttempt=SourceSaveAttempt(id:id,source:buffer.text,base:buffer.base,hash:buffer.hash,guardToken:"captured-before",revision:buffer.sourceEditIdentity)
        var payload=reading();var entries=payload["entries"] as! [[String:Any]];entries[0]["outcome"]="failed";entries[0]["available"]=false;entries[0]["retry_allowed"]=true;payload["entries"]=entries
        ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=["/api/source-history":try JSONSerialization.data(withJSONObject:payload)];await space.sourceHistory.open(space)
        let chosen=try XCTUnwrap(space.sourceHistory.reading?.entries.first);space.sourceHistory.choose(chosen,side:"after");XCTAssertTrue(space.sourceHistory.allowNewSaveAttempt());XCTAssertNil(buffer.saveHistoryAttempt);XCTAssertEqual(buffer.text,"draft");XCTAssertEqual(buffer.base,"saved");XCTAssertEqual(buffer.savedGuard,"captured-before")
        buffer.saveHistoryAttempt=SourceSaveAttempt(id:id,source:buffer.text,base:buffer.base,hash:buffer.hash,guardToken:"captured-before",revision:buffer.sourceEditIdentity);var unknown=chosen;unknown.outcome="unknown";space.sourceHistory.choose(unknown,side:"after");XCTAssertFalse(space.sourceHistory.allowNewSaveAttempt());XCTAssertNotNil(buffer.saveHistoryAttempt);XCTAssertTrue(editor.text.string==buffer.text)
    }
    func testStrictSaveRetainsConsumedIntentAndNeverFallsBack()async throws {
        let(space,buffer,_,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        let engine=try XCTUnwrap(space.engine);buffer.savedGuard="captured-before";try install(buffer.text)
        ResponseProtocol.status=500;ResponseProtocol.response=Data(#"{"error":"response unavailable"}"#.utf8)
        await buffer.save(using:engine);let original=ResponseProtocol.captured
        XCTAssertEqual(buffer.base,"saved");XCTAssertEqual(buffer.savedGuard,"captured-before");XCTAssertTrue(buffer.dirty)
        ResponseProtocol.responsesByPath=[:];ResponseProtocol.status=200
        ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["version":1,"history_protocol":1,"path":"a.txt","request_id":original["request_id"] ?? "","source_hash":nativeSourceHash("draft"),"save_guard":"confirmed-after","hash":"after-hash","history":["available":true,"outcome":"published"]])
        await buffer.save(using:engine)
        XCTAssertEqual(ResponseProtocol.requestedURL?.path,"/api/source-history/file");XCTAssertEqual(ResponseProtocol.captured["request_id"] as? String,original["request_id"] as? String);XCTAssertEqual(ResponseProtocol.captured["save_guard"] as? String,"captured-before");XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0)
        XCTAssertEqual(try JSONSerialization.data(withJSONObject:ResponseProtocol.captured,options:.sortedKeys),try JSONSerialization.data(withJSONObject:original,options:.sortedKeys))
        XCTAssertEqual(buffer.base,"draft");XCTAssertEqual(buffer.savedGuard,"confirmed-after");XCTAssertFalse(buffer.dirty)
    }
    func testMissingReadGuardAndOldDaemonKeepReadableDraft()async throws {
        let(space,buffer,_,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        let engine=try XCTUnwrap(space.engine);ResponseProtocol.resetRequests();await buffer.save(using:engine)
        XCTAssertEqual(ResponseProtocol.requestCount("/api/source-history/file"),0);XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.text,"draft")
        buffer.savedGuard="captured";ResponseProtocol.status=404;ResponseProtocol.response=Data(#"{"error":"unsupported endpoint"}"#.utf8);ResponseProtocol.responsesByPath=[:]
        await buffer.save(using:engine);XCTAssertEqual(ResponseProtocol.requestedURL?.path,"/api/source-history/file");XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0);XCTAssertEqual(buffer.base,"saved");XCTAssertEqual(buffer.savedGuard,"captured");XCTAssertTrue(buffer.dirty)
    }
    func testHistoryPreviewAndRestoreRetainNativeDraftUndoAndEditor()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        editor.text.insertText("!",replacementRange:editor.text.selectedRange());let before=buffer.text,selection=editor.text.selectedRange(),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=editor.scroll.frame,manager=try XCTUnwrap(editor.text.undoManager)
        try install(before);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        XCTAssertEqual(buffer.text,before);XCTAssertEqual(editor.text.selectedRange(),selection);XCTAssertTrue(manager.canUndo)
        let applied=await space.sourceHistory.restore();XCTAssertTrue(applied,space.sourceHistory.notice);XCTAssertEqual(buffer.text,"old");XCTAssertEqual(buffer.base,"saved");XCTAssertTrue(buffer.dirty)
        manager.undo();XCTAssertEqual(buffer.text,before);XCTAssertEqual(editor.text.selectedRange(),selection);manager.redo();XCTAssertEqual(buffer.text,"old")
        XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(editor.scroll.frame,geometry)
    }
    func testClosedEditedAndReplacedEditorCannotRestoreHeldHistory()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        try install(buffer.text);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        editor.text.insertText("!",replacementRange:editor.text.selectedRange());let refused=await space.sourceHistory.restore();XCTAssertFalse(refused);XCTAssertEqual(buffer.text,"draft!")
        try install(buffer.text);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/source-history/restore-draft"{space.sourceHistory.close()}}
        let closed=await space.sourceHistory.restore();XCTAssertFalse(closed);XCTAssertEqual(buffer.text,"draft!")
        ResponseProtocol.beforeResponse=nil;try install(buffer.text);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before");buffer.resourceDeleted=true
        let deleted=await space.sourceHistory.restore();XCTAssertFalse(deleted);XCTAssertEqual(buffer.text,"draft!")
    }
    func testLiveNativeOrdinarySaveHistoryRestoreAndUndo()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_HISTORY_ORIGIN"] else{throw XCTSkip("Opt in with an isolated ordinary-history engine fixture.")}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular)
        guard let workspace=ProcessInfo.processInfo.environment["HICKORY_NATIVE_HISTORY_WORKSPACE"],let canonical=workspace.withCString({realpath($0,nil)}) else{throw EngineError(message:"Actual history writes require an owned workspace fixture.")};defer{free(canonical)}
        let engine=try Engine(address:origin),space=Workspace();space.engine=engine
        let marker=try await engine.file(FileNode(name:".hickory-native-history-fixture",path:".hickory-native-history-fixture",dir:false));XCTAssertEqual(marker.content,"owned-native-history-v1\n");guard marker.content=="owned-native-history-v1\n" else{return}
        let listing=try await engine.files();space.rootFiles=listing.tree;space.sourceRootAddress=listing.root_path;space.sourceWorkspaceID=listing.workspace_id;XCTAssertEqual(listing.root_path,String(cString:canonical));guard listing.root_path==String(cString:canonical) else{return}
        func find(_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path=="history.txt"{return node};if let child=find(node.children ?? []){return child}};return nil}
        let file=try XCTUnwrap(find(space.rootFiles)),initial=try await engine.file(file),buffer=try await space.loadBuffer(file);XCTAssertFalse(buffer.readOnly);space.layout.activate(file.path,in:"document-group")
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{space.sourceHistory.close();space.debugger?.close();space.sourceTestIndex?.stop();for item in space.buffers.values{item.language?.close()};window.orderOut(nil)}
        func editor(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        for _ in 0..<200{if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let native=try XCTUnwrap(editor(host)),scroll=try XCTUnwrap(native.enclosingScrollView),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host)
        native.setSelectedRange(NSRange(location:0,length:native.string.utf16.count));native.insertText("saved-after\n",replacementRange:native.selectedRange());await buffer.save(using:engine);XCTAssertTrue(buffer.error.isEmpty,buffer.error);XCTAssertFalse(buffer.dirty)
        native.setSelectedRange(NSRange(location:native.string.utf16.count,length:0));native.insertText("unsaved",replacementRange:native.selectedRange());let draft=buffer.text,selection=native.selectedRange(),manager=try XCTUnwrap(native.undoManager)
        try XCTUnwrap(space.commands().first(where:{$0.id=="source.history"})).run();for _ in 0..<200{if !space.sourceHistory.busy&&space.sourceHistory.reading != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let entry=try XCTUnwrap(space.sourceHistory.reading?.entries.first,space.sourceHistory.notice);XCTAssertEqual(entry.outcome,"published");XCTAssertTrue(entry.available);XCTAssertEqual(entry.before.text,initial.content);space.sourceHistory.choose(entry,side:"before");XCTAssertEqual(buffer.text,draft)
        let restored=await space.sourceHistory.restore();XCTAssertTrue(restored,space.sourceHistory.notice);XCTAssertEqual(buffer.text,initial.content);XCTAssertEqual(buffer.base,"saved-after\n");let disk=try await engine.file(file);XCTAssertEqual(disk.content,"saved-after\n")
        manager.undo();XCTAssertEqual(buffer.text,draft);XCTAssertEqual(native.selectedRange(),selection);manager.redo();XCTAssertEqual(buffer.text,initial.content)
        XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertTrue(editor(host) === native)
        space.sourceHistory.close();let copied=buffer.text;space.fileCommand="Save As";space.filePath="history-copy.txt";await space.performFileCommand()
        XCTAssertTrue(space.currentBuffer === buffer,space.status);XCTAssertNil(space.buffers["history.txt"]);XCTAssertEqual(buffer.file.path,"history-copy.txt");XCTAssertEqual(buffer.base,copied);XCTAssertNotNil(buffer.savedGuard);XCTAssertFalse(buffer.dirty)
        for _ in 0..<200{if editor(host) === native{break};try await Task.sleep(for:.milliseconds(10))}
        XCTAssertTrue(editor(host) === native);XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        manager.undo();XCTAssertEqual(buffer.text,draft);XCTAssertTrue(buffer.dirty);manager.redo();XCTAssertEqual(buffer.text,copied);XCTAssertFalse(buffer.dirty)
        let copiedDisk=try await engine.file(buffer.file),originalDisk=try await engine.file(file);XCTAssertEqual(copiedDisk.content,copied);XCTAssertEqual(originalDisk.content,"saved-after\n")
        await space.sourceHistory.open(space);let creation=try XCTUnwrap(space.sourceHistory.reading?.entries.first,space.sourceHistory.notice);XCTAssertEqual(creation.before.kind,"missing");XCTAssertEqual(creation.after.text,copied);XCTAssertTrue(creation.available)
    }
    #endif
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceHistoryTests.swift"><ui:paste select="#source-history-apple-tests"/></ui:file>

<ui:file path="apps/apple-simple/Sources/SourceHistory.swift"><ui:paste select="#source-history-apple-values"/><ui:paste select="#source-history-apple-controller"/><ui:paste select="#source-history-apple-create"/><ui:paste select="#source-history-apple-view"/></ui:file>
</ui:doc>
