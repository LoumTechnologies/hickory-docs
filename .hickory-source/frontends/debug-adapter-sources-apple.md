<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read adapter-owned source inside the existing native debugger inspector

For macOS and iOS maintainers following a positive stopped-frame source reference. Adapter text is a literal, read-only observation, never a workspace path, source replacement or breakpoint authority. Require explicit source_content capability and exact session, stopped generation, thread, frame, positive signed32 reference, inspector lifetime and latest request. Movement, frame/thread changes and inspector close retire pending reads while retaining captured historical text. Optional refusal remains local to this reading; watches, paused frames and Stop remain usable.

## Validate the observation before attaching bounded literal pages

The worker validates UTF8 size, MIME metadata, scalar page boundaries and SHA256. After that await the main actor checks the same pending request and current stopped binding again. Pages contain at most four thousand Unicode scalars and never split surrogate pairs. Missing capability sends no request; positive references never trigger a filesystem fallback.

<ui:copy id="debug-adapter-sources-apple-model">
import Foundation
import Observation
import CoreFoundation
import SwiftUI
struct NativeAdapterSourceBinding:Equatable,Sendable {
    static func ==(a:Self,b:Self)->Bool{exactUTF8(a.session,b.session)&&a.stop==b.stop&&a.thread==b.thread&&a.frame==b.frame&&a.reference==b.reference}
    var session:String;var stop:UInt64;var thread:Int64;var frame:Int;var reference:Int
    var request:[String:Any]{["op":"source","session":session,"stop":stop,"thread":thread,"frame":frame,"reference":reference]}
    func matches(_ event:[String:Any])->Bool {exactUTF8(event["session"] as? String,session)&&adapterSourceStop(event["stop"])==stop&&adapterSourceID(event["thread"]).map(Int64.init)==thread&&adapterSourceID(event["frame"])==frame&&adapterSourceID(event["reference"])==reference}
}
func adapterSourceID(_ raw:Any?)->Int? {guard let number=raw as? NSNumber,CFGetTypeID(number) != CFBooleanGetTypeID() else{return nil};let value=number.doubleValue;guard value.isFinite,value.rounded()==value,value>=Double(Int32.min),value<=Double(Int32.max) else{return nil};return Int(value)}
private func adapterSourceStop(_ raw:Any?)->UInt64? {guard let number=raw as? NSNumber,CFGetTypeID(number) != CFBooleanGetTypeID() else{return nil};let value=number.stringValue;guard !value.isEmpty,value.utf8.allSatisfy({(48...57).contains($0)}) else{return nil};return UInt64(value)}
struct NativeAdapterSourceFacts:Sendable {var hash:String;var scalars:Int;var starts:[Int]}
func nativeAdapterSourceFacts(_ content:String,mime:String?=nil,budget:Int=8*1024*1024)throws->NativeAdapterSourceFacts {
    guard budget>=0,content.utf8.count<=budget else{throw EngineError(message:"Adapter source exceeds its bounded UTF8 reading capacity.")}
    if let mime {guard mime.utf8.count<=256,!mime.unicodeScalars.contains(where:{CharacterSet.controlCharacters.contains($0)}) else{throw EngineError(message:"Adapter source MIME metadata is malformed or exceeds its capacity.")}}
    var scalars=0,offset=0,starts=[0]
    for scalar in content.unicodeScalars {if scalars>0&&scalars%4000==0{starts.append(offset)};offset += scalar.value>0xffff ? 2:1;scalars += 1}
    return NativeAdapterSourceFacts(hash:nativeSourceHash(content),scalars:scalars,starts:starts)
}
struct NativeAdapterSourceReading {var binding:NativeAdapterSourceBinding;var request:String;var content:String;var mime:String?;var facts:NativeAdapterSourceFacts
    func page(_ index:Int)->String{guard facts.starts.indices.contains(index) else{return ""};let start=facts.starts[index],end=index+1<facts.starts.count ? facts.starts[index+1]:content.utf16.count;return (content as NSString).substring(with:NSRange(location:start,length:end-start))}
}
@MainActor @Observable final class NativeAdapterSourceModel {
    private struct Pending {var binding:NativeAdapterSourceBinding;var request:String;var inspection:UUID;var generation:UUID;var processing=false}
    private var inspection:UUID?;private var generation=UUID();private var pending:Pending?
    private(set) var reading:NativeAdapterSourceReading?;private(set) var busy=false;private(set) var current=false;private(set) var notice=""
    @ObservationIgnored var validate:@Sendable(String,String?)async throws->NativeAdapterSourceFacts={text,mime in try await Task.detached(priority:.userInitiated){try nativeAdapterSourceFacts(text,mime:mime)}.value}
    func open()->UUID{retire("Previous adapter text is historical until explicitly refreshed.");let token=UUID();inspection=token;return token}
    func close(_ token:UUID){guard inspection==token else{return};inspection=nil;retire("Adapter-source inspector closed. Captured text remains historical.")}
    func retire(_ reason:String="Paused source observation changed. Captured adapter text is historical."){generation=UUID();pending=nil;busy=false;current=false;if reading != nil || !notice.isEmpty{notice=reason}}
    func cancel(_ token:UUID){if inspection==token{retire("Pending source reading cancelled locally. Late replies cannot replace captured text.")}}
    func begin(_ binding:NativeAdapterSourceBinding?,capability:Any?,inspection token:UUID)->[String:Any]? {
        guard inspection==token else{notice="Open the current debugger inspector before reading adapter source.";return nil}
        guard let number=capability as? NSNumber,CFGetTypeID(number)==CFBooleanGetTypeID(),number.boolValue else{notice="This engine/provider does not advertise adapter source content. No workspace text was substituted.";return nil}
        guard let binding,binding.reference>0,binding.reference<=Int(Int32.max) else{notice="Choose a current paused frame with a positive adapter source reference. No disk fallback is used.";return nil}
        let id=UUID().uuidString;generation=UUID();pending=Pending(binding:binding,request:id,inspection:token,generation:generation);busy=true;current=false;notice="Reading selected frame adapter-owned source…";var request=binding.request;request["request_id"]=id;return request
    }
    private func owns(_ value:Pending)->Bool{pending?.request==value.request&&inspection==value.inspection&&generation==value.generation}
    @discardableResult func receive(_ event:[String:Any],binding:()->NativeAdapterSourceBinding?)async->Bool {
        guard var held=pending,exactUTF8(event["request_id"] as? String,held.request),owns(held),!held.processing else{return false}
        guard event["event"] as? String=="adapter_source",held.binding.matches(event),binding()==held.binding,let content=event["content"] as? String,let hash=event["source_hash"] as? String,hash.utf8.count==64&&hash.utf8.allSatisfy({(48...57).contains($0)||(97...102).contains($0)}) else{return reject(held.request,"Adapter source reply is malformed or belongs to an obsolete stopped frame.")}
        let mime:String?;if let raw=event["mime_type"],!(raw is NSNull){guard let text=raw as? String else{return reject(held.request,"Adapter source MIME metadata is malformed.")};mime=text}else{mime=nil}
        held.processing=true;pending=held
        let facts:NativeAdapterSourceFacts;do{facts=try await validate(content,mime)}catch{return reject(held.request,error.localizedDescription)}
        guard owns(held) else{return false};guard binding()==held.binding else{retire();return false}
        guard exactUTF8(facts.hash,hash) else{return reject(held.request,"Adapter source bytes do not match the reported hash.")}
        reading=NativeAdapterSourceReading(binding:held.binding,request:held.request,content:content,mime:mime,facts:facts);pending=nil;busy=false;current=true;notice=content.isEmpty ? "Adapter reported empty literal source content.":"Adapter-owned literal text only. No workspace edit, source position or breakpoint authority is implied.";return true
    }
    private func reject(_ request:String,_ message:String)->Bool{_ = refuse(request,message);return false}
    @discardableResult func refuse(_ request:String,_ message:String)->Bool{guard let held=pending,exactUTF8(held.request,request),owns(held) else{return false};pending=nil;busy=false;current=false;notice=message+" Captured text is retained; refresh explicitly.";return true}
    func matches(_ binding:NativeAdapterSourceBinding?)->Bool{current&&reading?.binding==binding}
}
</ui:copy>

## Reuse the floating inspector without changing editor geometry

The existing debugger popover owns one inspection token. Its plain selectable Text never interprets MIME as HTML or markup. Explicit paging, refresh and cancellation remain keyboard accessible. Captured bytes stay readable after scope retirement; historical labels do not imply current runtime or workspace authority.

<ui:copy id="debug-adapter-sources-apple-view">
struct NativeAdapterSourceView:View {
    var debug:NativeDebugger;var inspection:UUID?
    @State private var page=0
    var body:some View {
        let model=debug.adapterSources,binding=debug.adapterSourceBinding
        if binding != nil || model.reading != nil || !model.notice.isEmpty {
            VStack(alignment:.leading,spacing:5){
                Text("Adapter-owned source").font(.caption.bold())
                Button("Read selected frame adapter source"){if let inspection{debug.readAdapterSource(inspection)}}.disabled(inspection==nil || binding==nil || !debug.canReadAdapterSource || model.busy)
                if binding != nil && !debug.canReadAdapterSource{Text("Source content capability is unavailable. Workspace or disk text is not a fallback.").font(.caption)}
                if model.busy{Button("Cancel pending source reading"){if let inspection{model.cancel(inspection)}}}
                if !model.notice.isEmpty{Text(model.notice).font(.caption).textSelection(.enabled)}
                if let reading=model.reading {
                    let index=min(max(0,page),reading.facts.starts.count-1)
                    Text("\(model.matches(binding) ? "Current":"Historical") adapter text · pause \(reading.binding.stop) · thread \(reading.binding.thread) · frame \(reading.binding.frame) · reference \(reading.binding.reference)").font(.caption)
                    Text("\(reading.mime ?? "MIME unavailable") · SHA256 \(reading.facts.hash)").font(.caption).textSelection(.enabled)
                    Text(reading.page(index)).font(.system(.caption,design:.monospaced)).textSelection(.enabled).accessibilityLabel("Read-only literal adapter source page \(index+1)")
                    Text("Page \(index+1)/\(reading.facts.starts.count) · \(reading.facts.scalars) Unicode scalars. No workspace mapping is implied.").font(.caption)
                    HStack{Button("Previous adapter source page"){page=index-1}.disabled(index==0);Button("Next adapter source page"){page=index+1}.disabled(index+1>=reading.facts.starts.count)}
                }
            }.onChange(of:model.reading?.request){_,_ in page=0}
        }
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/DebugAdapterSources.swift"><ui:paste select="#debug-adapter-sources-apple-model"/><ui:paste select="#debug-adapter-sources-apple-view"/></ui:file>
<ui:copy id="debug-adapter-sources-apple-tests">
import XCTest
#if os(macOS)
import AppKit
import SwiftUI
#endif
@testable import HickorySimple
private actor AdapterSourceGate {
    private var continuation:CheckedContinuation<Void,Never>?;private(set) var waiting=false
    func hold()async{waiting=true;await withCheckedContinuation{continuation=$0}}
    func release(){continuation?.resume();continuation=nil}
}
@MainActor final class DebugAdapterSourceTests:XCTestCase {
    func binding(_ stop:UInt64=1)->NativeAdapterSourceBinding{NativeAdapterSourceBinding(session:"session",stop:stop,thread:0,frame:-3,reference:123)}
    func reply(_ request:[String:Any],content:String="<b>literal source</b>",mime:Any?="text/html")->[String:Any]{var event=request;event["op"]=nil;event["event"]="adapter_source";event["content"]=content;event["source_hash"]=nativeSourceHash(content);event["mime_type"]=mime;return event}
    func testCapabilityAbsentOrNumericAliasNeverSendsSource()throws {
        let model=NativeAdapterSourceModel(),token=model.open()
        for capability:Any? in [nil,false,1,"true"]{XCTAssertNil(model.begin(binding(),capability:capability,inspection:token))}
        XCTAssertNil(model.begin(nil,capability:true,inspection:token));XCTAssertNil(model.begin(NativeAdapterSourceBinding(session:"session",stop:1,thread:0,frame:-3,reference:0),capability:true,inspection:token));XCTAssertFalse(model.busy)
    }
    func testLiteralUnicodePagesHashAndCapacityAreExact()throws {
        let text=String(repeating:"😀",count:4000)+"e\u{301}<b>literal</b>",facts=try nativeAdapterSourceFacts(text,mime:"text/html"),reading=NativeAdapterSourceReading(binding:binding(),request:"read",content:text,mime:"text/html",facts:facts)
        XCTAssertEqual(facts.starts,[0,8000]);XCTAssertEqual(reading.page(0),String(repeating:"😀",count:4000));XCTAssertEqual(reading.page(1),"e\u{301}<b>literal</b>");XCTAssertNotEqual(try nativeAdapterSourceFacts("é").hash,try nativeAdapterSourceFacts("e\u{301}").hash)
        XCTAssertThrowsError(try nativeAdapterSourceFacts("😀",budget:3));XCTAssertThrowsError(try nativeAdapterSourceFacts("",mime:"text/html\n"));XCTAssertThrowsError(try nativeAdapterSourceFacts("",mime:String(repeating:"λ",count:129)))
        XCTAssertEqual(try nativeAdapterSourceFacts("").starts,[0]);XCTAssertEqual(try nativeAdapterSourceFacts("").scalars,0)
    }
    func testLatestScopedReplyAndProviderRefusalPreserveHistoricalBytes()async throws {
        let model=NativeAdapterSourceModel(),token=model.open(),scope=binding();let first=try XCTUnwrap(model.begin(scope,capability:true,inspection:token)),second=try XCTUnwrap(model.begin(scope,capability:true,inspection:token))
        let old=await model.receive(reply(first),binding:{scope});XCTAssertFalse(old);XCTAssertTrue(model.busy)
        let accepted=await model.receive(reply(second),binding:{scope});XCTAssertTrue(accepted);XCTAssertEqual(model.reading?.content,"<b>literal source</b>");XCTAssertTrue(model.matches(scope))
        let third=try XCTUnwrap(model.begin(scope,capability:true,inspection:token));XCTAssertTrue(model.refuse(third["request_id"] as! String,"Provider unavailable"));XCTAssertFalse(model.current);XCTAssertFalse(model.busy);XCTAssertEqual(model.reading?.content,"<b>literal source</b>")
        let fourth=try XCTUnwrap(model.begin(scope,capability:true,inspection:token));var wrong=reply(fourth);wrong["stop"]=2;let rejected=await model.receive(wrong,binding:{scope});XCTAssertFalse(rejected);XCTAssertEqual(model.reading?.request,second["request_id"] as? String)
    }
    func testHashMismatchAndWrongSignedIdentityNeverAdopt()async throws {
        let model=NativeAdapterSourceModel(),token=model.open(),scope=binding();let request=try XCTUnwrap(model.begin(scope,capability:true,inspection:token));var event=reply(request);event["source_hash"]=nativeSourceHash("different");let rejected=await model.receive(event,binding:{scope});XCTAssertFalse(rejected);XCTAssertNil(model.reading)
        for bad:Any in [true,0.5,Int64(Int32.max)+1]{let request=try XCTUnwrap(model.begin(scope,capability:true,inspection:token));var event=reply(request);event["frame"]=bad;let rejected=await model.receive(event,binding:{scope});XCTAssertFalse(rejected);XCTAssertNil(model.reading)}
    }
    func testHeldValidationCannotReviveClosedInspectorOrReusedFrameReference()async throws {
        let model=NativeAdapterSourceModel(),gate=AdapterSourceGate(),token=model.open(),scope=binding();model.validate={text,mime in await gate.hold();return try nativeAdapterSourceFacts(text,mime:mime)};let request=try XCTUnwrap(model.begin(scope,capability:true,inspection:token));var current:NativeAdapterSourceBinding?=scope
        let task=Task{await model.receive(reply(request),binding:{current})};for _ in 0..<100{if await gate.waiting{break};try await Task.sleep(for:.milliseconds(5))};let waiting=await gate.waiting;XCTAssertTrue(waiting)
        model.close(token);current=binding(2);_ = model.open();await gate.release();let accepted=await task.value;XCTAssertFalse(accepted);XCTAssertNil(model.reading);XCTAssertFalse(model.busy)
    }
    func debugFixture()throws->(NativeDebugger,()->[[String:Any]]){
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest();var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","request_id":launch,"doc":"hick:///main.py","session":"session","source_hash":nativeSourceHash(debug.launchSource),"capabilities":["source_content":true],"breakpoints":[]]);debug.receive(["event":"stopped","session":"session","stop":1,"thread":0,"frames":[["id":-3,"name":"adapter frame","source":"main.py","source_reference":123]],"variables":[]]);return(debug,{requests})
    }
    func testOptionalSourceFailureKeepsPauseEvaluationAndStopUsable()throws {
        let(debug,requests)=try debugFixture();defer{debug.close()};let inspection=debug.adapterSources.open();debug.readAdapterSource(inspection);let request=try XCTUnwrap(requests().last);XCTAssertEqual(request["op"] as? String,"source");XCTAssertEqual(request["frame"] as? Int,-3);XCTAssertNil(request["path"])
        debug.receive(["event":"failed","session":"session","about":"source","request_id":request["request_id"]!,"message":"Provider source unavailable"]);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.count,1);XCTAssertTrue(debug.canEvaluate);XCTAssertFalse(debug.adapterSources.busy);debug.evaluate("marker",watch:true);XCTAssertEqual(requests().last?["op"] as? String,"eval");debug.stop();XCTAssertEqual(requests().last?["op"] as? String,"stop")
    }
    func testMalformedPauseRetiresPendingAndClosedRuntimeKeepsCapturedReadingReachable()async throws {
        let(debug,requests)=try debugFixture();let token=debug.adapterSources.open();debug.readAdapterSource(token);let request=try XCTUnwrap(requests().last),binding=try XCTUnwrap(debug.adapterSourceBinding)
        let accepted=await debug.adapterSources.receive(reply(request),binding:{debug.adapterSourceBinding});XCTAssertTrue(accepted)
        debug.readAdapterSource(token);XCTAssertTrue(debug.adapterSources.busy);debug.receive(["event":"stopped","session":"session","stop":2,"thread":true,"frames":[],"variables":[]]);XCTAssertFalse(debug.adapterSources.busy);XCTAssertFalse(debug.adapterSources.matches(binding));XCTAssertNotNil(debug.adapterSources.reading)
        debug.close();XCTAssertTrue(debug.hasSessionReading);XCTAssertNil(debug.adapterSourceBinding);XCTAssertEqual(debug.adapterSources.reading?.content,"<b>literal source</b>")
    }
    #if os(macOS)
    func testLiveNativeAdapterSourceLiteralScopeAndEditorRetention()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_ADAPTER_SOURCE_ORIGIN"],let workspace=ProcessInfo.processInfo.environment["HICKORY_NATIVE_ADAPTER_SOURCE_WORKSPACE"] else{throw XCTSkip("Opt in with the isolated owned adapter-source provider fixture.")}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular)
        let engine=try Engine(address:origin),space=Workspace();space.engine=engine
        let marker=try await engine.file(FileNode(name:".hickory-owned-adapter-source-fixture",path:".hickory-owned-adapter-source-fixture",dir:false));guard exactUTF8(marker.content,"adapter-source-fixture-v1\n") else{throw EngineError(message:"Adapter source integration writes require the owned fixture marker.")}
        let listing=try await engine.files();space.rootFiles=listing.tree;space.sourceRootAddress=listing.root_path;space.sourceWorkspaceID=listing.workspace_id
        let root=URL(fileURLWithPath:workspace).resolvingSymlinksInPath();guard let listed=listing.root_path,URL(fileURLWithPath:listed).resolvingSymlinksInPath()==root else{throw EngineError(message:"Adapter source fixture workspace changed.")}
        let file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=try await space.loadBuffer(file),initial=buffer.text,debug=NativeDebugger(engine:engine);space.debugger=debug;space.layout.activate(file.path,in:"document-group");debug.sourceBuffers={Array(space.buffers.values)}
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        let popover=NSPopover();defer{popover.close();debug.close();space.sourceTestIndex?.stop();for item in space.buffers.values{item.language?.close()};window.orderOut(nil)}
        func textView(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{textView($0)}.first}
        func wait(_ reason:String,_ predicate:()->Bool)async throws{for _ in 0..<400{if predicate(){return};try await Task.sleep(for:.milliseconds(10))};XCTFail(reason)}
        try await wait("Native source editor did not attach"){textView(host) != nil};let native=try XCTUnwrap(textView(host)),scroll=try XCTUnwrap(native.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
        await debug.prepare(buffer);try await wait("Owned adapter did not stop: \(debug.message)"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.adapterSourceBinding?.reference,123);XCTAssertTrue(debug.canReadAdapterSource)
        native.setSelectedRange(NSRange(location:native.string.utf16.count,length:0));native.insertText("# unsaved native text\n",replacementRange:native.selectedRange());let draft=buffer.text,selection=native.selectedRange(),undo=try XCTUnwrap(native.undoManager)
        let inspection=debug.adapterSources.open();popover.contentViewController=NSHostingController(rootView:NativeAdapterSourceView(debug:debug,inspection:inspection));popover.contentSize=NSSize(width:330,height:350);popover.show(relativeTo:NSRect(x:0,y:0,width:1,height:1),of:native,preferredEdge:.maxX)
        var requests:[[String:Any]]=[];debug.observeRequestForTest={requests.append($0)};debug.readAdapterSource(inspection);try await wait("Literal adapter source did not arrive"){!debug.adapterSources.busy};let reading=try XCTUnwrap(debug.adapterSources.reading,debug.adapterSources.notice)
        XCTAssertEqual(reading.content,"<b>adapter-owned literal source</b>\nmarker = 41\n");XCTAssertEqual(reading.mime,"text/html");XCTAssertEqual(reading.page(0),reading.content);XCTAssertNotEqual(reading.content,initial);XCTAssertNil(debug.line(in:file.path));XCTAssertEqual(requests.last?["op"] as? String,"source");XCTAssertNil(requests.last?["path"])
        try Data().write(to:root.appendingPathComponent("source-error"));debug.readAdapterSource(inspection);try await wait("Optional source refusal did not arrive"){!debug.adapterSources.busy};XCTAssertEqual(debug.status,"paused");XCTAssertTrue(debug.canEvaluate);XCTAssertEqual(debug.adapterSources.reading?.request,reading.request);try FileManager.default.removeItem(at:root.appendingPathComponent("source-error"))
        try Data().write(to:root.appendingPathComponent("hold-source"));debug.readAdapterSource(inspection);try await wait("Provider source operation was not held"){FileManager.default.fileExists(atPath:root.appendingPathComponent("source-held").path)}
        let stop=try XCTUnwrap(debug.adapterSourceBinding?.stop);debug.step("over");try await wait("Movement was blocked by optional source IO"){debug.status=="paused"&&(debug.adapterSourceBinding?.stop ?? 0)>stop};XCTAssertFalse(debug.adapterSources.busy);XCTAssertFalse(debug.adapterSources.current)
        try Data().write(to:root.appendingPathComponent("release-source"));try await Task.sleep(for:.milliseconds(100));XCTAssertEqual(debug.adapterSources.reading?.request,reading.request);XCTAssertFalse(debug.adapterSources.current)
        debug.evaluate("marker",watch:true);try await wait("Watch unavailable after optional source refusal"){debug.values["marker"]=="41"};XCTAssertEqual(buffer.text,draft);XCTAssertEqual(native.selectedRange(),selection);XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertTrue(textView(host) === native)
        popover.close();debug.adapterSources.close(inspection);undo.undo();XCTAssertEqual(buffer.text,initial);undo.redo();XCTAssertEqual(buffer.text,draft);debug.stop();try await wait("Stop unavailable after optional source reading"){debug.status=="finished"||debug.status=="failed"};XCTAssertEqual(debug.status,"finished",debug.message);XCTAssertEqual(debug.adapterSources.reading?.content,reading.content)
    }
    #endif

}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/DebugAdapterSourceTests.swift"><ui:paste select="#debug-adapter-sources-apple-tests"/></ui:file>

</ui:doc>
