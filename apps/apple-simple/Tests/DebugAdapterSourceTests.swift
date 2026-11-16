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
