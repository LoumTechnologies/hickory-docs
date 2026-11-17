<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Attach the native debugger to a declared loopback process

For maintainers wiring the existing floating Debug chooser to an authored endpoint profile. The engine owns profile admission, endpoint arguments, source revision limits and disconnect policy. The host decodes the optional endpoint metadata, offers its explicit chooser, labels attachment clearly and sends only the current profile ID and catalog fingerprint. Older ordinary catalogs remain readable. A source hash captured in this editor does not establish the external process's loaded revision.

The existing [profile model and launch boundary](apps/apple-simple/Sources/Engine.swift.md) retain that distinction through session readings. The [floating debugger controls](apps/apple-simple/Sources/NativeEditor.swift.md) display the loopback port and external-source caveat without adding permanent controls or changing code geometry. There is no ordinary Run action for attachment; the engine independently refuses that unsupported operation.

## Check profile decoding and the existing chooser action

Endpoint ports and arguments must have their declared types. A clicked application keeps its established precedence. With no matching application, a declared default endpoint is used; the explicit chooser can always select another current profile without launching the clicked application. These fixtures observe the real native HTTP encoder and guarded Start construction; they do not open an endpoint.

<ui:copy id="debug-attach-apple-tests">
import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
@MainActor final class NativeDebugAttachTests:XCTestCase {
    func profile(_ attach:Any?=nil)->[String:Any]{var value:[String:Any]=["id":"external","name":"Owned external Python","program":NSNull(),"can_debug":true,"default":true];if let attach{value["attach"]=attach};return value}
    func testLegacyProfileAndTypedEndpointRemainDistinct()throws {
        let legacy=try JSONDecoder().decode(NativeDebugProfile.self,from:JSONSerialization.data(withJSONObject:profile()));XCTAssertNil(legacy.attach);XCTAssertNil(legacy.attachNotice)
        let endpoint=try JSONDecoder().decode(NativeDebugProfile.self,from:JSONSerialization.data(withJSONObject:profile(["port":5678,"arguments":["justMyCode":false]])))
        XCTAssertEqual(endpoint.attach?.port,5678);XCTAssertEqual(endpoint.default_profile,true);XCTAssertEqual(endpoint.actionLabel,"Attach Owned external Python");XCTAssertTrue(endpoint.attachNotice!.contains("loopback port 5678"));XCTAssertTrue(endpoint.attachNotice!.contains("external source revision unverified"));XCTAssertNotNil(endpoint.runUnavailableReason)
        for invalid:Any in [0,-1,65536,true,3.5,"5678"]{XCTAssertThrowsError(try JSONDecoder().decode(NativeDebugProfile.self,from:JSONSerialization.data(withJSONObject:profile(["port":invalid,"arguments":[:]]))))}
        XCTAssertThrowsError(try JSONDecoder().decode(NativeDebugProfile.self,from:JSONSerialization.data(withJSONObject:profile(["port":5678,"arguments":[]]))))
    }
    func testDefaultEndpointSelectionSendsOnlyCurrentProfileAuthority()async throws {
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self];let engine=try Engine(address:"http://attach.invalid",session:URLSession(configuration:configuration)),debug=NativeDebugger(engine:engine)
        defer{debug.close();ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        let external=profile(["port":5678,"arguments":["justMyCode":false]]),ordinary:[String:Any]=["id":"app:other.py","name":"Other file","program":"other.py","can_debug":true]
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.beforeResponse=nil;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["debug_protocol":3,"profiles":[ordinary,external],"fingerprint":String(repeating:"a",count:64),"viewed":"main.py"])
        let buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value = 41\n",hash:"loaded"));var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        await debug.prepare(buffer);let start=try XCTUnwrap(sent.first{$0["op"] as? String=="start"}),selection=try XCTUnwrap(start["profile"] as? [String:Any]);XCTAssertEqual(selection["id"] as? String,"external");XCTAssertEqual(selection["fingerprint"] as? String,String(repeating:"a",count:64));XCTAssertNil(start["attach"]);XCTAssertNil(start["arguments"]);XCTAssertNil(start["program"]);XCTAssertTrue(debug.attachReading!.contains("source revision unverified"));XCTAssertEqual(buffer.text,"value = 41\n")
    }
    func testClickedApplicationPrecedenceAndExplicitEndpointChooser()async throws{
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self];let engine=try Engine(address:"http://attach.invalid",session:URLSession(configuration:configuration)),debug=NativeDebugger(engine:engine)
        defer{debug.close();ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["debug_protocol":3,"profiles":[["id":"app:main.py","name":"Viewed file","program":"main.py","can_debug":true],profile(["port":5678,"arguments":[:]])],"fingerprint":String(repeating:"b",count:64),"viewed":"main.py"])
        let buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value = 41\n",hash:"loaded"));var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        await debug.prepare(buffer);XCTAssertEqual((sent.first?["profile"] as? [String:Any])?["id"] as? String,"app:main.py");XCTAssertNil(debug.attachReading)
        debug.close();sent=[];await debug.prepare(buffer,chooseProfile:true);XCTAssertTrue(sent.isEmpty);XCTAssertEqual(debug.profiles.count,2);debug.launch(try XCTUnwrap(debug.profiles.first{$0.attach != nil}));XCTAssertEqual((sent.first?["profile"] as? [String:Any])?["id"] as? String,"external");XCTAssertTrue(debug.attachReading?.contains("loopback port 5678")==true)
    }
    func testProtocolNumericAliasRefusesBeforeAnyStart()async throws {
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self];let debug=NativeDebugger(engine:try Engine(address:"http://attach.invalid",session:URLSession(configuration:configuration)))
        defer{debug.close();ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]};ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]
        ResponseProtocol.response=Data(#"{"debug_protocol":3.0,"profiles":[],"fingerprint":"current","viewed":"main.py"}"#.utf8);let buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value = 41\n",hash:"loaded"));var sent=0;debug.emitForTest={_ in sent+=1};await debug.prepare(buffer);XCTAssertEqual(sent,0);XCTAssertTrue(debug.message.contains("canonical integer"),debug.message);XCTAssertEqual(buffer.text,"value = 41\n")
    }
    #if os(macOS)
    func testLiveOwnedLoopbackAttachRetainsNativeEditorAndDetachesExternalTarget()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_ATTACH_ORIGIN"] else{throw XCTSkip("Opt in with scripts/test-debug-attach.py --serve-only and its owned endpoint descriptor.")}
        let environment=ProcessInfo.processInfo.environment,workspace=try XCTUnwrap(environment["HICKORY_NATIVE_ATTACH_WORKSPACE"]),endpoint=try XCTUnwrap(environment["HICKORY_NATIVE_ATTACH_ENDPOINT"].flatMap(UInt16.init)),target=try XCTUnwrap(environment["HICKORY_NATIVE_ATTACH_TARGET_PID"].flatMap(Int32.init));XCTAssertGreaterThan(target,0);guard target>0 else{return}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular)
        let engine=try Engine(address:origin),space=Workspace(),debug=NativeDebugger(engine:engine);space.engine=engine;space.debugger=debug
        guard let resolved=workspace.withCString({realpath($0,nil)}) else{throw EngineError(message:"Owned fixture root could not be verified.")};defer{free(resolved)};let canonical=String(cString:resolved)
        let listing=try await engine.files();guard exactUTF8(listing.root_path,canonical) else{throw EngineError(message:"Server namespace \(listing.root_path ?? "missing") differs from verified fixture \(canonical)")};space.rootFiles=listing.tree;space.files=listing.tree;space.sourceRootAddress=listing.root_path;space.sourceWorkspaceID=listing.workspace_id
        let file=FileNode(name:"main.py",path:environment["HICKORY_NATIVE_ATTACH_INPUT"] ?? "main.py",dir:false),buffer=try await space.loadBuffer(file),source=buffer.text;space.layout.activateSource(file.path,in:"document-group")
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
        var received:[[String:Any]]=[];debug.observeEventForTest={if received.count<64{received.append($0)}}
        defer{if debug.status=="failed"{print("Native attach failed state",debug.status,debug.message,"events",received)};if debug.active{debug.stop()};debug.close();space.sourceTestIndex?.stop();for b in space.buffers.values{b.language?.close()};window.orderOut(nil)}
        func find(_ view:NSView)->NSTextView?{if let text=view as? NSTextView,exactUTF8(text.string,buffer.text){return text};return view.subviews.lazy.compactMap{find($0)}.first}
        func settle(_ label:String,_ ready:@escaping ()->Bool)async throws{for _ in 0..<600{host.layoutSubtreeIfNeeded();if ready(){return};try await Task.sleep(for:.milliseconds(20))};print("Native attach diagnostic",label,debug.status,debug.message,debug.frames);throw EngineError(message:label+": "+debug.message+" · "+debug.status)}
        try await settle("Native editor did not mount"){find(host) != nil};let editor=try XCTUnwrap(find(host)),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=try XCTUnwrap(editor.enclosingScrollView).convert(editor.enclosingScrollView!.bounds,to:host)
        var sent:[[String:Any]]=[];debug.observeRequestForTest={sent.append($0)};await debug.prepare(buffer)
        if !debug.active,let selected=debug.profiles.first(where:{$0.attach?.port==endpoint}){debug.launch(selected)}
        try await settle("Owned endpoint did not pause"){debug.status=="paused"};XCTAssertEqual(kill(target,0),0);XCTAssertTrue(debug.attachReading?.contains("loopback port \(endpoint)")==true);XCTAssertTrue(debug.attachReading?.contains("source revision unverified")==true);XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertNil(debug.pausedLine)
        let starts=sent.filter{$0["op"] as? String=="start"};XCTAssertEqual(starts.count,1);XCTAssertEqual((starts.first?["profile"] as? [String:Any])?["id"] as? String,"external");XCTAssertNil(starts.first?["program"])
        editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("\n# retained unsaved native attach draft\n",replacementRange:editor.selectedRange());let draft=buffer.text,selection=editor.selectedRange();debug.evaluate("value",watch:true);try await settle("Attached watch did not resolve"){debug.values["value"]=="41"}
        let stop=try XCTUnwrap(debug.stoppedObservation),moves=sent.filter{$0["op"] as? String=="step"}.count;debug.step("over");XCTAssertEqual(sent.filter{$0["op"] as? String=="step"}.count,moves+1);try await settle("Attached thread did not step"){debug.stoppedObservation.map{$0>stop}==true};debug.evaluate("value",watch:true);try await settle("Watch did not resolve after step"){debug.values["value"]=="41"}
        XCTAssertTrue(find(host) === editor);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(editor.selectedRange(),selection);XCTAssertTrue(exactUTF8(buffer.text,draft));XCTAssertEqual(editor.enclosingScrollView!.convert(editor.enclosingScrollView!.bounds,to:host),geometry)
        debug.stop();try await settle("Attached session did not detach"){!debug.active};XCTAssertEqual(kill(target,0),0,"Disconnect must not terminate the externally owned target");let saved=try await engine.file(file);XCTAssertTrue(exactUTF8(saved.content,source));XCTAssertTrue(buffer.dirty)
        editor.undoManager?.undo();XCTAssertTrue(exactUTF8(buffer.text,source));editor.undoManager?.redo();XCTAssertTrue(exactUTF8(buffer.text,draft));XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity)
    }
    #endif
}
</ui:copy>

The opt-in witness uses a maintained owned debugpy endpoint. It goes through the current native profile preparation and real WebSocket, watches a runtime value, steps, retains the mounted editor and native undo, and confirms disconnect leaves the original external process alive. This is programmatic native acceptance; physical input, VoiceOver and device input remain unverified.

<ui:file path="apps/apple-simple/Tests/NativeDebugAttachTests.swift"><ui:paste select="#debug-attach-apple-tests"/></ui:file>
</ui:doc>
