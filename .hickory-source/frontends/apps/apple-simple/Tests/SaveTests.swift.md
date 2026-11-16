<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Witness native save refusal and revision payloads

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

A local URLProtocol fixture exercises the actual Swift HTTP encoder and buffer behavior without sending an agent request or writing a workspace. This is a protocol safety test, not a substitute for device UI checks.

<ui:copy id="simple-d0dada29d6-0">
// Contract: .hickory-source/frontends/contract.md
import XCTest
@testable import HickorySimple

final class ResponseProtocol: URLProtocol {
    static var requestedURL: URL?
    static var beforeResponse: ((URLRequest) -> Void)?
    private static let requestLock = NSLock()
    private static var requests: [URL] = []
    static func resetRequests() { requestLock.lock(); defer { requestLock.unlock() }; requests = [] }
    static func requestCount(_ path: String) -> Int { requestLock.lock(); defer { requestLock.unlock() }; return requests.filter { $0.path == path }.count }
    static var status = 200
    static var response = Data()
    static var responsesByPath: [String: Data] = [:]
    static var captured: [String: Any] = [:]
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        Self.requestedURL = request.url
        if let url=request.url { Self.requestLock.lock(); Self.requests.append(url); Self.requestLock.unlock() }
        Self.beforeResponse?(request)
        var body = request.httpBody ?? Data()
        if let stream = request.httpBodyStream {
            stream.open(); defer { stream.close() }
            var bytes = [UInt8](repeating: 0, count: 4096)
            while stream.hasBytesAvailable {
                let count = stream.read(&bytes, maxLength: bytes.count)
                if count <= 0 { break }
                body.append(contentsOf: bytes.prefix(count))
            }
        }
        Self.captured = (try? JSONSerialization.jsonObject(with: body)) as? [String: Any] ?? [:]
        let response = HTTPURLResponse(url: request.url!, statusCode: Self.status, httpVersion: nil, headerFields: ["Content-Type": "application/json"])!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: Self.responsesByPath[request.url!.path] ?? Self.response)
        client?.urlProtocolDidFinishLoading(self)
    }
    override func stopLoading() {}
}
final class SaveTests: XCTestCase {
    @MainActor func testSharedLanguageWireRoutesRepliesDetachesOneSourceAndReplaysAfterFailure() async throws {
        // Contract: .hickory-source/frontends/contract.md. Own protocol fixture, real native WebSocket.
        #if os(macOS)
        let root=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let process=Process(),pipe=Pipe();process.executableURL=URL(fileURLWithPath:"/usr/bin/env");process.arguments=["python3",root.appendingPathComponent("scripts/simple-language-wire-fixture.py").path];process.standardOutput=pipe;process.standardError=pipe
        try process.run();defer {if process.isRunning {process.terminate()};process.waitUntilExit()}
        let greeting=try XCTUnwrap(JSONSerialization.jsonObject(with:pipe.fileHandleForReading.availableData) as? [String:Any])
        let engine=try Engine(address:try XCTUnwrap(greeting["origin"] as? String)),connection=NativeLanguageConnection(engine:engine)
        defer {connection.close()}
        struct Metrics: Decodable {var connections:Int;var ids:[Int];var documents:[String:String]}
        func metrics() async throws -> Metrics {try await engine.request("GET","/state")}
        func params(_ uri:String)->[String:Any] {["textDocument":["uri":uri]]}
        let a="hick:///a.py",b="hick:///b.py",c="hick:///c.py"
        var reportsA:[String]=[],reportsB:[String]=[],reportsC:[String]=[],failuresA=0,failuresB=0,failuresC=0
        let ownerA=connection.attach(uri:a,source:"first",version:1,receive:{if let uri=($0["params"] as? [String:Any])?["uri"] as? String {reportsA.append(uri)}},failed:{_ in failuresA += 1})
        let ownerB=connection.attach(uri:b,source:"second",version:1,receive:{if let uri=($0["params"] as? [String:Any])?["uri"] as? String {reportsB.append(uri)}},failed:{_ in failuresB += 1})
        _=connection.attach(uri:c,source:"third",version:1,receive:{if let uri=($0["params"] as? [String:Any])?["uri"] as? String {reportsC.append(uri)}},failed:{_ in failuresC += 1})
        async let slow=connection.request("fixture/slow",params:params(a),owner:ownerA)
        async let fast=connection.request("fixture/fast",params:params(b),owner:ownerB)
        let (one,two)=try await(slow,fast)
        XCTAssertEqual((one as? [String:Any])?["uri"] as? String,a);XCTAssertEqual((two as? [String:Any])?["text"] as? String,"second")
        XCTAssertEqual(reportsA,[a]);XCTAssertEqual(reportsB,[b]);XCTAssertEqual(reportsC,[c]);let initial=try await metrics();XCTAssertEqual(initial.connections,1)
        var rejected=false
        let duplicate=connection.attach(uri:a,source:"conflicting draft",version:2,receive:{_ in XCTFail("Duplicate source received a notification")},failed:{_ in rejected=true})
        XCTAssertTrue(rejected);connection.change(owner:duplicate,source:"corrupt",version:3)
        do {_=try await connection.request("fixture/fast",params:params(a),owner:duplicate);XCTFail("Duplicate source borrowed request authority")} catch {}
        let refused=try await connection.request("fixture/askApply",params:params(b),owner:ownerB);XCTAssertEqual((refused as? [String:Any])?["applied"] as? Bool,false)
        let preceding=try await metrics().ids.count
        let blocking=Task {try await connection.request("fixture/block",params:params(a),owner:ownerA)}
        _=try await connection.request("fixture/fast",params:params(b),owner:ownerB)
        let deadline=Date().addingTimeInterval(2)
        while try await metrics().ids.count < preceding+2,Date()<deadline {try await Task.sleep(for:.milliseconds(10))}
        connection.detach(ownerA)
        do {_=try await blocking.value;XCTFail("Detached owner retained a pending request")} catch {}
        let stillAlive=try await connection.request("fixture/fast",params:params(b),owner:ownerB)
        XCTAssertEqual((stillAlive as? [String:Any])?["text"] as? String,"second")
        do {_=try await connection.request("fixture/drop",params:params(b),owner:ownerB);XCTFail("Dropped wire returned a successful reply")} catch {}
        XCTAssertEqual(failuresA,0);XCTAssertEqual(failuresB,1);XCTAssertEqual(failuresC,1)
        connection.change(owner:ownerB,source:"new second",version:2)
        try await Task.sleep(for:.milliseconds(1100))
        let reopened=try await connection.request("fixture/fast",params:params(b),owner:ownerB)
        XCTAssertEqual((reopened as? [String:Any])?["text"] as? String,"new second")
        let final=try await metrics();XCTAssertEqual(final.connections,2);XCTAssertNil(final.documents[a]);XCTAssertEqual(final.documents[b],"new second");XCTAssertEqual(final.documents[c],"third");XCTAssertEqual(Set(final.ids).count,final.ids.count)
        let file=FileNode(name:"outline.py",path:"outline.py",dir:false,language:"python")
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"source",hash:"base"))
        let service=NativeLanguage(engine:engine,buffer:buffer,connection:connection);service.start();defer {service.close()}
        let reading=Task {try await service.readOutline(source:"source")}
        try await Task.sleep(for:.milliseconds(10));buffer.selection=NSRange(location:3,length:0)
        let outline=try await reading.value
        XCTAssertEqual(outline.first?.name,"background");XCTAssertEqual(buffer.selection.location,3);XCTAssertFalse(service.showingSuggestions)
        await service.navigate(.supertypes);XCTAssertEqual(service.locations.first?.name,"supertype")
        await service.navigate(.subtypes);XCTAssertEqual(service.locations.first?.name,"subtype")
        let hierarchy=Task {await service.navigate(.supertypes)};try await Task.sleep(for:.milliseconds(10));buffer.selection=NSRange(location:4,length:0);await hierarchy.value
        XCTAssertTrue(service.locations.isEmpty);XCTAssertTrue(service.locationStatus.contains("changed"));buffer.selection=NSRange(location:3,length:0)
        let stale=Task {try await service.readOutline(source:"source")}
        try await Task.sleep(for:.milliseconds(10));buffer.text="draft"
        do {_=try await stale.value;XCTFail("Source changed during outline read")} catch {}
        XCTAssertEqual(buffer.text,"draft")
        #endif
    }

    @MainActor func testBackgroundSourceAcquisitionSharesReadWithoutActivatingPane() async throws {
        // Contract: .hickory-source/frontends/contract.md
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.beforeResponse=nil;ResponseProtocol.resetRequests()
        ResponseProtocol.response=Data(#"{"path":"background.py","content":"value = 1","hash":"disk"}"#.utf8)
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let session=URLSession(configuration:config),workspace=Workspace();workspace.engine=try Engine(address:"http://fixture.invalid",session:session)
        let file=FileNode(name:"background.py",path:"background.py",dir:false,language:"python")
        defer { workspace.buffers.values.forEach {$0.language?.close()};session.invalidateAndCancel() }
        async let first=workspace.loadBuffer(file);async let second=workspace.loadBuffer(file)
        let (a,b)=try await(first,second)
        XCTAssertTrue(a === b);XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),1)
        XCTAssertEqual(a.text,"value = 1");XCTAssertNil(workspace.active);XCTAssertNotNil(a.language)
        let other=try await workspace.loadBuffer(FileNode(name:"other.py",path:"other.py",dir:false,language:"python"))
        XCTAssertTrue(a.language?.connection === other.language?.connection);XCTAssertNil(workspace.active)
    }
    @MainActor func testDelayedBackgroundSourceCannotReplaceAvailableDraft() async throws {
        // Contract: .hickory-source/frontends/contract.md
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests()
        ResponseProtocol.response=Data(#"{"path":"background.py","content":"late disk text","hash":"disk"}"#.utf8)
        let arrived=expectation(description:"raw file request"),gate=DispatchSemaphore(value:0)
        ResponseProtocol.beforeResponse={request in if request.url?.path == "/api/file" {arrived.fulfill();_ = gate.wait(timeout:.now()+5)}}
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let session=URLSession(configuration:config),workspace=Workspace();workspace.engine=try Engine(address:"http://fixture.invalid",session:session)
        let file=FileNode(name:"background.py",path:"background.py",dir:false,language:"python")
        defer {gate.signal();ResponseProtocol.beforeResponse=nil;session.invalidateAndCancel()}
        let read=Task {try await workspace.loadBuffer(file)}
        await fulfillment(of:[arrived],timeout:3)
        let draft=DocumentBuffer(file:file,text:FileText(path:file.path,content:"base",hash:"base"));draft.text="retained draft";workspace.buffers[file.path]=draft
        gate.signal();let loaded=try await read.value
        XCTAssertTrue(loaded === draft);XCTAssertEqual(draft.text,"retained draft");XCTAssertTrue(draft.dirty);XCTAssertNil(workspace.active)
    }

    @MainActor func testAgentRequestUsesSubscriptionAndPythonBackend() async throws {
        let (engine, _) = try fixture()
        ResponseProtocol.status = 200; ResponseProtocol.responsesByPath = [:]
        ResponseProtocol.response = Data("{\"session_id\":\"turn\"}".utf8)
        _ = try await engine.send("# Task", parent: nil, provider: "claude-subscription", model: "sonnet", effort: "high")
        XCTAssertEqual(ResponseProtocol.captured["backend"] as? String, "python")
        XCTAssertEqual(ResponseProtocol.captured["provider"] as? String, "claude-subscription")
        XCTAssertEqual(ResponseProtocol.captured["model"] as? String, "sonnet")
        XCTAssertEqual(ResponseProtocol.captured["effort"] as? String, "high")
        XCTAssertEqual(ResponseProtocol.captured["prompt"] as? String, "# Task")
    }

    @MainActor private func fixture(doc: String? = nil, generated: String? = nil) throws -> (Engine, DocumentBuffer) {
        let configuration = URLSessionConfiguration.ephemeral; configuration.protocolClasses = [ResponseProtocol.self]
        let engine = try Engine(address: "http://fixture.invalid", session: URLSession(configuration: configuration))
        let file = FileNode(name: "notes.txt", path: "notes.txt", dir: false, doc_id: doc, generated_by: generated)
        return (engine, DocumentBuffer(file: file, text: FileText(path: file.path, content: "before", hash: "hash-before")))
    }
    @MainActor func testManagedMarkdownOpensThroughDocumentSource() async throws {
        let (engine, buffer) = try fixture(doc: "managed")
        ResponseProtocol.status = 200
        ResponseProtocol.response = Data(##"{"source":"# Readable source"}"##.utf8)
        let text = try await engine.file(buffer.file)
        XCTAssertEqual(ResponseProtocol.requestedURL?.path, "/api/docs/managed")
        XCTAssertEqual(text.content, "# Readable source")
    }
    @MainActor func testPlainTextOpensWithFileHash() async throws {
        let (engine, buffer) = try fixture()
        ResponseProtocol.status = 200
        ResponseProtocol.response = Data(#"{"path":"notes.txt","content":"Plain text","hash":"plain-hash"}"#.utf8)
        let text = try await engine.file(buffer.file)
        XCTAssertEqual(ResponseProtocol.requestedURL?.path, "/api/file")
        XCTAssertEqual(text.content, "Plain text"); XCTAssertEqual(text.hash, "plain-hash")
    }
    @MainActor func testConflictRetainsEditsAndOriginalBaseline() async throws {
        let (engine, buffer) = try fixture()
        ResponseProtocol.status = 409; ResponseProtocol.response = Data(#"{"error":"changed on disk"}"#.utf8)
        buffer.text = "mine"; await buffer.save(using: engine)
        XCTAssertEqual(ResponseProtocol.captured["base_hash"] as? String, "hash-before")
        XCTAssertEqual(buffer.text, "mine"); XCTAssertEqual(buffer.base, "before"); XCTAssertTrue(buffer.dirty)
        XCTAssertTrue(buffer.error.contains("edits are retained"))
    }
    @MainActor func testLiterateSaveUsesBothExpectations() async throws {
        let (engine, buffer) = try fixture(doc: "doc")
        ResponseProtocol.status = 200; ResponseProtocol.response = Data(#"{"source":"next"}"#.utf8)
        buffer.text = "next"; await buffer.save(using: engine)
        let expected = ResponseProtocol.captured["expected"] as? [String: String]
        XCTAssertEqual(expected, ["source": "before", "disk": "before"]); XCTAssertFalse(buffer.dirty)
    }
    @MainActor func testGeneratedOutputDoesNotDispatchSave() async throws {
        let (engine, buffer) = try fixture(generated: "owner")
        ResponseProtocol.captured = [:]; buffer.text = "replacement"; await buffer.save(using: engine)
        XCTAssertTrue(ResponseProtocol.captured.isEmpty); XCTAssertTrue(buffer.readOnly)
    }
    // Contract: .hickory-source/frontends/contract.md — reopening retains the PTY identity.
    @MainActor func testTerminalReopenKeepsExistingSession() async throws {
        let (engine, _) = try fixture()
        ResponseProtocol.status = 200; ResponseProtocol.response = Data(#"{"id":"pty"}"#.utf8)
        let session = TerminalSession()
        await session.open(using: engine)
        ResponseProtocol.status = 409; ResponseProtocol.response = Data(#"{"error":"must not open twice"}"#.utf8)
        await session.open(using: engine)
        XCTAssertEqual(session.id, "pty"); XCTAssertTrue(session.error.isEmpty)
    }

}
</ui:copy>

## Check session identity, launch bytes and control boundaries

These focused witnesses drive the native controller with explicit protocol events, inspect outbound verbs and check pending-stop races. The HTTP witness verifies that literate launch resolution carries the selected generated path plus the entire displayed document. The optional live witnesses require a real backend and adapter; they skip without HICKORY_NATIVE_DEBUG_ORIGIN and are not native physical-input acceptance receipts. The thread fixture also needs an active Python worker and a `# thread-stop` breakpoint marker. Its native client selects the named `hickory-worker` thread and `worker` frame, requests selected-frame locals, evaluates `worker_value` as 41, returns to the original thread and evaluates `main_value` as 7, then stops while leaving source and caret unchanged. Distinct frame-local values make wrong-frame evaluation observable. The AppKit renderer witness opens the real gutter overflow before Inspect; it must preserve the cached text view and geometry when an unmapped thread retires the original paused row. It uses programmatic accessibility actions and native text insertion; physical input and VoiceOver remain separate acceptance. A worker blocked indefinitely inside a provider’s native wait may report a truthful evaluation/locals refusal; that response must preserve the paused session and the controls needed to select another thread.

<ui:copy id="simple-apple-debug-witnesses">
#if os(macOS)
import AppKit
import SwiftUI
#endif
final class NativeDebugTests: XCTestCase {
    @MainActor func testLegacyCatalogRefusesDebugBeforeStartAndRetainsSource()async throws {
        let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),debug=NativeDebugger(engine:engine)
        let source="print(1)\n",file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"saved"))
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};defer{debug.close();ResponseProtocol.responsesByPath=[:]}
        let protocols:[Int?]=[nil,1]
        for version in protocols {
            var catalog:[String:Any]=["profiles":[["id":"python","name":"Python","can_debug":true,"program":"main.py"]],"fingerprint":"current"]
            if let version{catalog["debug_protocol"]=version}
            ResponseProtocol.status=200;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:catalog)
            await debug.prepare(buffer);XCTAssertTrue(requests.isEmpty);XCTAssertEqual(debug.status,"unavailable");XCTAssertTrue(debug.message.contains("No Start command was sent"));XCTAssertEqual(buffer.text,source);XCTAssertFalse(buffer.dirty)
        }
    }

    @MainActor func testCanonicalDistinctExpressionsKeepExactRequestsValuesAndWatchRows()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";let launch=debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};debug.receive(["event":"started","request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":11,"name":"main"]],"variables":[]])
        let first="'é'",second="'e\u{301}'"
        XCTAssertEqual(first,second,"Swift linguistic equality must not become debugger expression identity.")
        debug.evaluate(first,watch:true);let one=requests.last!;debug.evaluate(second,watch:true);let two=requests.last!
        XCTAssertEqual(debug.watches.count,2);XCTAssertEqual(Set(debug.watches.map{Data($0.utf8)}).count,2)
        func value(_ request:[String:Any],_ expression:String,_ text:String)->[String:Any] {["event":"value","request_id":request["request_id"]!,"session":"ours","stop":1,"thread":1,"frame":11,"expression":expression,"value":text]}
        debug.receive(value(one,first,"first result"));XCTAssertEqual(debug.values[first],"first result");XCTAssertNil(debug.values[second])
        debug.receive(value(two,second,"second result"));XCTAssertEqual(debug.values[first],"first result");XCTAssertEqual(debug.values[second],"second result");XCTAssertEqual(Set(debug.values.readings.map{$0.id}).count,2)
        debug.evaluate(first,watch:true);let newer=requests.last!;debug.receive(value(two,first,"wrong canonical row"));XCTAssertEqual(debug.values[first],"first result");debug.receive(value(newer,first,"updated first"))
        debug.removeWatch(first);XCTAssertFalse(debug.isWatched(first));XCTAssertTrue(debug.isWatched(second));XCTAssertNil(debug.values[first]);XCTAssertEqual(debug.values[second],"second result");XCTAssertEqual(debug.values.readings.count,1);debug.close()
    }

    @MainActor func testZeroNegativeAndMalformedRuntimeIDsStayDistinct()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";debug.launchSource="value\n";let launch=debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)};debug.receive(["event":"started","request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int,_ thread:Any,_ frame:Any)->[String:Any] {["event":"stopped","session":"ours","stop":stop,"thread":thread,"frames":[["id":frame,"name":"frame","source":"main.py","source_line":0]],"variables":[]]}
        debug.receive(stopped(1,-1,-7));XCTAssertEqual(debug.selectedThread,-1);XCTAssertEqual(debug.selectedFrame,-7);XCTAssertEqual(debug.line(in:"main.py"),0)
        debug.readFrameLocals();let locals=requests.last!;XCTAssertEqual(locals["thread"] as? Int64,-1);XCTAssertEqual(locals["frame"] as? Int,-7)
        debug.receive(["event":"frame_variables","request_id":locals["request_id"]!,"session":"ours","stop":1,"thread":-1,"frame":-7,"variables":[]]);XCTAssertTrue(debug.frameVariablesCurrent)
        debug.receive(stopped(2,0,0));XCTAssertEqual(debug.selectedThread,0);XCTAssertEqual(debug.selectedFrame,0);debug.evaluate("zero");let zero=requests.last!
        debug.receive(["event":"value","request_id":zero["request_id"]!,"session":"ours","stop":2,"thread":0,"frame":0,"expression":"zero","value":"accepted"]);XCTAssertEqual(debug.values["zero"],"accepted")
        var current=2
        let malformed:[Any]=[true,1.5,Int64(Int32.max)+1]
        for bad in malformed {
            debug.readThreads();debug.receive(["event":"threads","request_id":requests.last!["request_id"]!,"session":"ours","stop":current,"selected":0,"threads":[["id":bad,"name":"malformed"]]])
            XCTAssertTrue(debug.threads.isEmpty);XCTAssertFalse(debug.threadBusy)
            debug.receive(stopped(current+1,bad,0));XCTAssertFalse(debug.canResume);XCTAssertFalse(debug.canReadThreads);XCTAssertNil(debug.line(in:"main.py"))
            debug.receive(stopped(current+2,0,0));current+=2;XCTAssertTrue(debug.canResume)
            debug.receive(stopped(current+1,0,bad));XCTAssertFalse(debug.canInspectFrame);XCTAssertFalse(debug.frameVariablesCurrent);XCTAssertNil(debug.line(in:"main.py"))
            debug.receive(stopped(current+2,0,0));current+=2
        }
        debug.close()
    }
    @MainActor func testAdapterSourceReferenceCannotBorrowPrimaryDiskProof()async throws {
        let engine=try Engine(address:"http://fixture.invalid"),space=Workspace(),debug=NativeDebugger(engine:engine);space.engine=engine;space.debugger=debug
        let text="value = 1\n",file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:text,hash:"saved"))
        space.buffers[file.path]=buffer;space.rootFiles=[file];debug.owner=file.path;debug.launchSource=text;let launch=debug.beginLaunchRequest();var location:NativeDebugLocation?,reveals=0
        debug.onLocation={location=$0};buffer.revealDebugLine={_ in reveals+=1;return true};debug.receive(["event":"started","request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int,_ reference:Int)->[String:Any] {["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":11,"name":"adapter source","source":"main.py","source_line":0,"source_hash":nativeSourceHash(text),"source_reference":reference]],"variables":[]]}
        debug.receive(stopped(1,17));XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertNil(debug.line(in:"main.py"));let adapter=try XCTUnwrap(location);XCTAssertNil(adapter.sourceHash);XCTAssertEqual(adapter.sourceReference,17)
        await space.revealDebug(adapter,debug:debug);XCTAssertEqual(space.active,"main.py");XCTAssertEqual(reveals,0);XCTAssertTrue(space.status.contains("adapter-provided source reference"));XCTAssertEqual(buffer.text,text)
        debug.receive(stopped(2,0));let disk=try XCTUnwrap(location);await space.revealDebug(disk,debug:debug);XCTAssertEqual(reveals,1);XCTAssertTrue(debug.sourceMatches(buffer));debug.close()
    }

    @MainActor func testRealInspectorSurvivesRetiredGutterRowAndDirtyEditor()async throws {
        #if os(macOS)
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
        let workspace=Workspace(),engine=try Engine(address:"http://fixture.invalid"),debug=NativeDebugger(engine:engine)
        let source="value = 1\nprint(value)\n",file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"saved"))
        workspace.buffers[file.path]=buffer;workspace.rootFiles=[file];workspace.debugger=debug;workspace.show(file.path)
        debug.owner=file.path;debug.launchSource=source;let launch=debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","request_id":launch,"doc":"hick:///main.py","session":"ours","breakpoints":[]])
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[["id":11,"name":"main frame","source":"main.py","source_line":1]],"variables":[]])
        let host=NSHostingView(rootView:LanguageEditor(buffer:buffer,debugger:debug)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded();defer{debug.close();window.orderOut(nil)}
        func editor(_ view:NSView)->NSTextView? {if let text=view as? NSTextView,text.string==buffer.text{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        func children(_ object:NSObject)->[Any] {((object as? any NSAccessibilityProtocol)?.accessibilityChildren() ?? []) + (object.accessibilityAttributeValue(.children) as? [Any] ?? [])}
        func role(_ object:NSObject)->String {((object as? any NSAccessibilityProtocol)?.accessibilityRole()?.rawValue) ?? object.accessibilityAttributeValue(.role) as? String ?? ""}
        func label(_ object:NSObject)->String {((object as? any NSAccessibilityProtocol)?.accessibilityLabel()) ?? ((object as? any NSAccessibilityProtocol)?.accessibilityTitle()) ?? object.accessibilityAttributeValue(.description) as? String ?? object.accessibilityAttributeValue(.title) as? String ?? ""}
        func press(_ object:NSObject)->Bool {if let node=object as? any NSAccessibilityProtocol{return node.accessibilityPerformPress()};guard object.accessibilityActionNames().contains(.press) else{return false};object.accessibilityPerformAction(.press);return true}
        func elements()->[NSObject] {
            var pending:[Any]=[NSApp,host]+NSApp.windows.filter{$0.isVisible},seen=Set<ObjectIdentifier>(),result:[NSObject]=[],visits=0
            while !pending.isEmpty,visits<10000 {visits+=1;let item=pending.removeLast();guard let object=item as? NSObject,seen.insert(ObjectIdentifier(object)).inserted else{continue};if let window=object as? NSWindow,let content=window.contentView{pending.append(content)};if let view=object as? NSView{pending += view.subviews};result.append(object);pending += children(object)}
            return result
        }
        func pumpNativeEvents(){host.layoutSubtreeIfNeeded();RunLoop.current.run(until:Date().addingTimeInterval(0.01));NSAccessibility.post(element:host,notification:.layoutChanged)}
        func button(_ name:String)->NSObject? {elements().first{role($0)==NSAccessibility.Role.button.rawValue && label($0).contains(name)}}
        func waitButton(_ name:String)async throws->NSObject {for _ in 0..<200 {pumpNativeEvents();if let found=button(name){return found};try await Task.sleep(for:.milliseconds(10))};throw NSError(domain:"NativeDebugView",code:1,userInfo:[NSLocalizedDescriptionKey:"Debugger button not reachable: "+name+" · "+debug.status+" · bound "+String(debug.sourceMatches(buffer))+" · rows "+String(describing:buffer.visibleDebugLines.map{$0.line})+" · tree "+elements().prefix(40).map{String(describing:type(of:$0))+":"+role($0)+":"+label($0)}.joined(separator:" | ")])}
        // An independently mounted stock SwiftUI button distinguishes an
        // unavailable host AX bridge from a missing product control.
        let baselineHost=NSHostingView(rootView:Button("Native debug AX baseline"){}),baselineWindow=NSWindow(contentRect:NSRect(x:720,y:0,width:220,height:80),styleMask:[.titled],backing:.buffered,defer:false)
        baselineWindow.contentView=baselineHost;baselineWindow.makeKeyAndOrderFront(nil);baselineHost.layoutSubtreeIfNeeded();defer{baselineWindow.orderOut(nil)}
        for _ in 0..<100 {pumpNativeEvents();baselineHost.layoutSubtreeIfNeeded();if button("Native debug AX baseline") != nil{break};try await Task.sleep(for:.milliseconds(10))}
        guard button("Native debug AX baseline") != nil else{throw XCTSkip("This XCTest host exposes native text/window AX but no independently mounted stock SwiftUI Button through modern or legacy AX children. Renderer/AX reachability acceptance remains unverified.")}
        baselineWindow.orderOut(nil);window.makeKeyAndOrderFront(nil)
        for _ in 0..<200 {if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let text=try XCTUnwrap(editor(host)),scroll=try XCTUnwrap(text.enclosingScrollView);text.setSelectedRange(NSRange(location:2,length:0));let geometry=scroll.convert(scroll.bounds,to:host)
        let overflow=try await waitButton("More gutter icons on line 2");XCTAssertTrue(press(overflow))
        let inspect=try await waitButton("Inspect on line 2");XCTAssertTrue(press(inspect))
        let read=try await waitButton("Read threads");XCTAssertTrue(press(read))
        for _ in 0..<200 {if requests.last?["op"] as? String=="threads"{break};try await Task.sleep(for:.milliseconds(10))}
        let readID=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"threads","request_id":readID,"session":"ours","stop":1,"selected":1,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        let worker=try await waitButton("worker (2)");XCTAssertTrue(press(worker))
        for _ in 0..<200 {if requests.last?["op"] as? String=="select_thread"{break};try await Task.sleep(for:.milliseconds(10))}
        let selected=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[["id":22,"name":"external worker frame","source":"/external/runtime.py","source_line":4]],"variables":[]])
        debug.receive(["event":"threads","request_id":selected,"session":"ours","stop":2,"selected":2,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        XCTAssertNil(debug.line(in:"main.py"));_ = try await waitButton("Read threads");_ = try await waitButton("external worker frame");_ = try await waitButton("Read selected-frame variables");_ = try await waitButton("Stop")
        XCTAssertTrue(editor(host) === text);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(text.selectedRange(),NSRange(location:2,length:0))
        text.insertText("# retained draft\n",replacementRange:NSRange(location:text.string.utf16.count,length:0));let draft=text.string,caret=text.selectedRange(),undo=try XCTUnwrap(text.undoManager)
        XCTAssertTrue(undo.canUndo);XCTAssertTrue(buffer.dirty)
        let locals=try await waitButton("Read selected-frame variables");XCTAssertTrue(press(locals))
        for _ in 0..<200 {if requests.last?["op"] as? String=="state"{break};try await Task.sleep(for:.milliseconds(10))}
        let localID=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"frame_variables","request_id":localID,"session":"ours","stop":2,"thread":2,"frame":22,"variables":[["name":"alive","value":"true","variables_reference":0]]])
        _ = try await waitButton("main (1)");XCTAssertTrue(editor(host) === text);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(text.selectedRange(),caret);XCTAssertTrue(undo.canUndo);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        // With no mapped pause, the same launch slot must still expose the
        // retained session after the original inspection popover is dismissed.
        let escape=try XCTUnwrap(NSEvent.keyEvent(with:.keyDown,location:.zero,modifierFlags:[],timestamp:ProcessInfo.processInfo.systemUptime,windowNumber:NSApp.keyWindow?.windowNumber ?? window.windowNumber,context:nil,characters:"\u{1b}",charactersIgnoringModifiers:"\u{1b}",isARepeat:false,keyCode:53))
        NSApp.keyWindow?.sendEvent(escape);try await Task.sleep(for:.milliseconds(50))
        let launchSlot=try await waitButton("Inspect debugger on line 1");XCTAssertTrue(press(launchSlot));_ = try await waitButton("Read threads");_ = try await waitButton("Stop")
        XCTAssertTrue(editor(host) === text);XCTAssertEqual(buffer.text,draft);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        #endif
    }

    @MainActor func testRetiredSameDocumentStartupCannotAdoptNewLaunchOrStopForeignSession()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py"
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        let old=debug.beginLaunchRequest();debug.stop();debug.close();let current=debug.beginLaunchRequest()
        debug.receive(["event":"started","request_id":old,"doc":"hick:///main.py","session":"retired","breakpoints":[]])
        XCTAssertEqual(requests.last?["op"] as? String,"stop");XCTAssertEqual(requests.last?["session"] as? String,"retired");XCTAssertEqual(debug.status,"starting");XCTAssertNil(debug.sessionID)
        let count=requests.count
        debug.receive(["event":"started","request_id":"foreign","doc":"hick:///main.py","session":"foreign-session","breakpoints":[]]);XCTAssertEqual(requests.count,count)
        debug.receive(["event":"build","request_id":old,"doc":"hick:///main.py","events":[["data":"old build"]]])
        debug.receive(["event":"failed","request_id":old,"doc":"hick:///main.py","session":NSNull(),"about":"start","message":"old failure"])
        XCTAssertTrue(debug.message.isEmpty);XCTAssertEqual(debug.status,"starting")
        debug.receive(["event":"started","request_id":current,"doc":"hick:///main.py","session":"current","breakpoints":[]]);XCTAssertEqual(debug.sessionID,"current");XCTAssertEqual(debug.status,"running")
        debug.receive(["event":"build","request_id":old,"doc":"hick:///main.py","events":[["data":"late old build"]]]);XCTAssertTrue(debug.message.isEmpty)
        debug.close();let forgotten=debug.beginLaunchRequest();for _ in 0..<65 {_ = debug.beginLaunchRequest()}
        let bounded=requests.count;debug.receive(["event":"started","request_id":forgotten,"doc":"hick:///main.py","session":"no-longer-owned","breakpoints":[]]);XCTAssertEqual(requests.count,bounded);XCTAssertEqual(debug.status,"starting");debug.close()
    }

    @MainActor func testDebugSourceReadsFullRootButNeverLearnsPostStopDiskAuthority()async throws {
        let engine=try Engine(address:"http://fixture.invalid"),space=Workspace(),debug=NativeDebugger(engine:engine)
        space.engine=engine;space.debugger=debug;debug.owner="main.py";debug.launchSource="primary\n";_ = debug.beginLaunchRequest()
        let text="helper\nsecond\n",file=FileNode(name:"helper.py",path:"folder/helper.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:text,hash:"saved"))
        buffer.selection=NSRange(location:2,length:0);space.buffers[file.path]=buffer;space.rootFiles=[FileNode(name:"folder",path:"folder",dir:true,children:[file])];space.files=[]
        var locations:[NativeDebugLocation]=[],revealed:[Int]=[];debug.onLocation={locations.append($0)};buffer.revealDebugLine={revealed.append($0);return true}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int,_ hash:String?=nil)->[String:Any] {var frame:[String:Any]=["id":11,"name":"helper","source":file.path,"source_line":1];if let hash{frame["source_hash"]=hash};return ["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[frame],"variables":[]]}
        debug.receive(stopped(1));let unbound=try XCTUnwrap(locations.last);await space.revealDebug(unbound,debug:debug)
        XCTAssertEqual(space.active,file.path);XCTAssertTrue(revealed.isEmpty);XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertTrue(space.status.contains("no captured source revision"));XCTAssertEqual(buffer.selection,NSRange(location:2,length:0))
        debug.receive(stopped(2,nativeSourceHash(text)));let bound=try XCTUnwrap(locations.last);await space.revealDebug(bound,debug:debug)
        XCTAssertEqual(revealed,[1]);XCTAssertTrue(debug.sourceMatches(buffer));XCTAssertEqual(buffer.text,text)
        debug.receive(stopped(3));XCTAssertFalse(debug.sourceMatches(buffer));XCTAssertNil(locations.last?.sourceHash,"An unknown frame must not borrow an earlier frame’s captured hash.")
        debug.receive(stopped(4,nativeSourceHash(text)));let current=try XCTUnwrap(locations.last)
        buffer.text += "# newer unsaved text\n";await space.revealDebug(current,debug:debug);XCTAssertEqual(revealed,[1]);XCTAssertTrue(buffer.text.hasSuffix("# newer unsaved text\n"));XCTAssertTrue(space.status.contains("without a position"))
        debug.close()
    }
    @MainActor func testDelayedDebugRevealRejectsRetiredStopAndUserNavigation()async throws {
        let engine=try Engine(address:"http://fixture.invalid"),space=Workspace(),debug=NativeDebugger(engine:engine);space.engine=engine;space.debugger=debug
        debug.owner="main.py";debug.launchSource="first\nsecond\n";_ = debug.beginLaunchRequest()
        let file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:debug.launchSource,hash:"saved"))
        space.buffers[file.path]=buffer;space.rootFiles=[file];space.files=[]
        var location:NativeDebugLocation?,revealed=0;debug.onLocation={location=$0}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        func stopped(_ stop:Int)->[String:Any] {["event":"stopped","session":"ours","stop":stop,"thread":1,"frames":[["id":11,"name":"main","source":"main.py","source_line":1]],"variables":[]]}
        debug.receive(stopped(1));let old=try XCTUnwrap(location)
        let waiting=Task {await space.revealDebug(old,debug:debug)};for _ in 0..<200 {if space.active=="main.py"{break};await Task.yield()};XCTAssertEqual(space.active,"main.py")
        debug.receive(stopped(2));buffer.revealDebugLine={_ in revealed+=1;return true};await waiting.value;XCTAssertEqual(revealed,0);XCTAssertEqual(space.active,"main.py")
        buffer.revealDebugLine=nil;space.show("files");let next=try XCTUnwrap(location),userNavigation=Task {await space.revealDebug(next,debug:debug)}
        for _ in 0..<200 {if space.active=="main.py"{break};await Task.yield()};XCTAssertEqual(space.active,"main.py");space.show("agent");buffer.revealDebugLine={_ in revealed+=1;return true};await userNavigation.value;XCTAssertEqual(revealed,0);XCTAssertEqual(space.active,"agent")
        await space.revealDebug(old,debug:debug);XCTAssertEqual(space.active,"agent");debug.close()
    }

    @MainActor func testSelectedFrameLocalsRequireLatestFrameScopeAndPreserveFailure()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let first:[String:Any]=["id":11,"name":"main"],second:[String:Any]=["id":12,"name":"caller"]
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":7,"frames":[first,second],"variables":[["name":"top","value":"old"]]])
        func locals(_ request:[String:Any],_ frame:Int,_ value:String,_ stop:Int=1)->[String:Any] {["event":"frame_variables","session":"ours","stop":stop,"thread":7,"frame":frame,"request_id":request["request_id"]!,"variables":[["name":"local","value":value,"variables_reference":0]]]}
        debug.select(second,snapshot:debug.frameSnapshot);let old=requests.last!;XCTAssertEqual(old["op"] as? String,"state");XCTAssertEqual(old["frame"] as? Int,12);XCTAssertTrue(debug.frameBusy)
        debug.select(first,snapshot:debug.frameSnapshot);let current=requests.last!;debug.receive(locals(old,12,"wrong caller"));XCTAssertTrue(debug.frameBusy);XCTAssertEqual(debug.variables.first?["name"] as? String,"top")
        debug.receive(locals(current,11,"fresh top"));XCTAssertFalse(debug.frameBusy);XCTAssertTrue(debug.frameVariablesCurrent);XCTAssertEqual(debug.variables.first?["value"] as? String,"fresh top")
        debug.select(second,snapshot:debug.frameSnapshot);let refused=requests.last!;debug.receive(["event":"failed","session":"ours","about":"state","request_id":refused["request_id"]!,"message":"Provider cannot materialize this frame"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.count,2);XCTAssertEqual(debug.selectedFrame,12);XCTAssertFalse(debug.frameBusy);XCTAssertFalse(debug.frameVariablesCurrent);XCTAssertTrue(debug.frameNotice.contains("selected stack and source are retained"))
        debug.readFrameLocals();let held=requests.last!;debug.receive(["event":"stopped","session":"ours","stop":2,"thread":7,"frames":[first],"variables":[["name":"new stop","value":"2"]]])
        debug.receive(locals(held,12,"stale reply"));XCTAssertEqual(debug.variables.first?["name"] as? String,"new stop");XCTAssertFalse(debug.frameBusy);debug.close()
    }
    @MainActor func testStackAndVariablesErrorsStayExplicitAndAllowThreadEscape()throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[],locations:[String]=[];debug.emitForTest={requests.append($0)};debug.onLocation={location in locations.append(location.path)}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let frame:[String:Any]=["id":11,"name":"main","source":"main.py","source_line":2]
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[frame],"variables":[["name":"value","value":"1"]]])
        let count=locations.count
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[],"variables":[],"stack_error":"Adapter stack unavailable","variables_error":"No materialized frame"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.count,1);XCTAssertEqual(locations.count,count);XCTAssertNil(debug.line(in:"main.py"));XCTAssertFalse(debug.canInspectFrame);XCTAssertFalse(debug.canEvaluate);XCTAssertFalse(debug.frameVariablesCurrent)
        XCTAssertTrue(debug.stackNotice.contains("Earlier frames"));XCTAssertTrue(debug.frameNotice.contains("Provider variables read refused"));XCTAssertTrue(debug.canReadThreads)
        debug.readThreads();debug.receive(["event":"threads","session":"ours","request_id":requests.last!["request_id"]!,"stop":1,"selected":1,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        debug.selectThread(2,snapshot:debug.threadSnapshot);XCTAssertEqual(requests.last?["op"] as? String,"select_thread")
        let selected=requests.last!["request_id"]!
        debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[["id":22,"name":"worker"]],"variables":[]])
        debug.receive(["event":"threads","session":"ours","request_id":selected,"stop":2,"selected":2,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]])
        XCTAssertEqual(debug.selectedThread,2);XCTAssertTrue(debug.stackNotice.isEmpty);XCTAssertTrue(debug.canInspectFrame);XCTAssertTrue(debug.frameVariablesCurrent);debug.close()
    }

    @MainActor func testThreadReadsAndSelectionBindStopRequestAndRenderedSnapshot() throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let first:[String:Any]=["id":11,"name":"main","source":"main.py","source_line":0]
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[first],"variables":[]])
        debug.readThreads();let request=try XCTUnwrap(requests.last?["request_id"] as? String)
        func threads(_ id:String,_ stop:Int=1,_ selected:Int=1)->[String:Any] {["event":"threads","session":"ours","request_id":id,"stop":stop,"selected":selected,"threads":[["id":1,"name":"main"],["id":2,"name":"worker"]]]}
        debug.receive(threads("retired"));XCTAssertTrue(debug.threads.isEmpty);XCTAssertTrue(debug.threadBusy)
        debug.receive(threads(request));XCTAssertEqual(debug.threads.map{$0.id},[1,2]);XCTAssertFalse(debug.threadBusy)
        let stale=debug.threadSnapshot;debug.readThreads();let newer=try XCTUnwrap(requests.last?["request_id"] as? String);debug.receive(threads(request));XCTAssertTrue(debug.threadBusy);debug.receive(threads(newer))
        let count=requests.count;debug.selectThread(2,snapshot:stale);XCTAssertEqual(requests.count,count)
        debug.selectThread(2,snapshot:debug.threadSnapshot);let selection=try XCTUnwrap(requests.last?["request_id"] as? String)
        XCTAssertEqual(requests.last?["op"] as? String,"select_thread");XCTAssertEqual(requests.last?["stop"] as? UInt64,1);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11);XCTAssertEqual(debug.selectedThread,1)
        debug.receive(["event":"failed","session":"ours","about":"select_thread","request_id":"retired","message":"old failure"]);XCTAssertTrue(debug.threadBusy)
        debug.receive(["event":"failed","session":"ours","about":"select_thread","request_id":selection,"message":"Thread is running"]);XCTAssertFalse(debug.threadBusy);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11)
        debug.selectThread(2,snapshot:debug.threadSnapshot);let confirmed=try XCTUnwrap(requests.last?["request_id"] as? String)
        debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[["id":22,"name":"worker","source":"worker.py","source_line":3]],"variables":[]])
        XCTAssertTrue(debug.threadBusy);debug.receive(threads(confirmed,2,2));XCTAssertEqual(debug.selectedThread,2);XCTAssertEqual(debug.threadStop,2);XCTAssertFalse(debug.threadBusy)
        debug.receive(["event":"stopped","session":"ours","stop":1,"thread":1,"frames":[first],"variables":[]]);XCTAssertEqual(debug.frames.first?["id"] as? Int,22)
        debug.step("over");XCTAssertEqual(debug.status,"running");XCTAssertTrue(debug.threads.isEmpty);debug.receive(["event":"stopped","session":"ours","stop":2,"thread":2,"frames":[first],"variables":[]]);XCTAssertEqual(debug.status,"running");debug.receive(threads(confirmed,2,2));XCTAssertTrue(debug.threads.isEmpty)
        debug.close()
    }
    @MainActor func testEvaluationRequiresCurrentStopThreadFrameAndLatestExpressionRequest() throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]])
        let first:[String:Any]=["id":11,"name":"main"],second:[String:Any]=["id":12,"name":"caller"]
        debug.receive(["event":"stopped","session":"ours","stop":10,"thread":7,"frames":[first,second],"variables":[]])
        func response(_ request:[String:Any],_ text:String,_ stop:Int=10,_ thread:Int=7,_ frame:Int=11)->[String:Any] {["event":"value","session":"ours","expression":"value","value":text,"stop":stop,"thread":thread,"frame":frame,"request_id":request["request_id"]!]}
        debug.evaluate("value");let old=requests.last!;debug.evaluate("value");let current=requests.last!
        XCTAssertEqual(current["stop"] as? UInt64,10);XCTAssertEqual(current["thread"] as? Int64,7);XCTAssertEqual(current["frame"] as? Int,11)
        debug.receive(response(old,"old"));XCTAssertNil(debug.values["value"]);debug.receive(response(current,"current"));XCTAssertEqual(debug.values["value"],"current")
        debug.evaluate("unsupported");let failure=requests.last!;debug.receive(["event":"failed","session":"ours","about":"eval","request_id":failure["request_id"]!,"message":"Provider cannot evaluate this frame"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.selectedFrame,11);XCTAssertEqual(debug.frames.count,2);XCTAssertTrue(debug.evaluationNotice.contains("current stopped frame is retained"))
        debug.evaluate("value");let formerFrame=requests.last!,snapshot=debug.frameSnapshot;debug.select(second,snapshot:snapshot);debug.receive(response(formerFrame,"wrong frame"));XCTAssertNil(debug.values["value"])
        debug.evaluate("value");let selected=requests.last!;debug.receive(response(selected,"wrong stop",9,7,12));XCTAssertNil(debug.values["value"]);debug.receive(response(selected,"caller value",10,7,12));XCTAssertEqual(debug.values["value"],"caller value")
        debug.receive(["event":"stopped","session":"ours","stop":11,"thread":8,"frames":[second],"variables":[]]);debug.select(first,snapshot:snapshot);XCTAssertEqual(debug.selectedFrame,12);XCTAssertTrue(debug.values.isEmpty)
        debug.receive(response(selected,"late",10,7,12));XCTAssertTrue(debug.values.isEmpty);debug.close()
    }
    @MainActor func testThreadPagingAndMalformedReadKeepPausedFrames() throws {
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"));debug.owner="main.py";_ = debug.beginLaunchRequest()
        var requests:[[String:Any]]=[];debug.emitForTest={requests.append($0)}
        debug.receive(["event":"started","request_id":debug.launchRequestID ?? "missing","doc":"hick:///main.py","session":"ours","breakpoints":[]]);debug.receive(["event":"stopped","session":"ours","stop":1,"thread":0,"frames":[["id":11,"name":"main"]],"variables":[]])
        debug.readThreads();debug.receive(["event":"threads","session":"ours","request_id":requests.last!["request_id"]!,"stop":1,"selected":0,"threads":(0..<201).map{["id":$0,"name":"Thread \($0)"]}])
        XCTAssertEqual(debug.threadPages,3);XCTAssertEqual(debug.visibleThreads.count,100);debug.threadPage=2;XCTAssertEqual(debug.visibleThreads.map{$0.id},[200])
        debug.readThreads();debug.receive(["event":"threads","session":"ours","request_id":requests.last!["request_id"]!,"stop":1,"selected":0,"threads":[["id":1,"name":"one"],["id":1,"name":"duplicate"]]])
        XCTAssertEqual(debug.threads.count,201);XCTAssertFalse(debug.threadBusy);XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11);XCTAssertTrue(debug.threadNotice.contains("duplicate"))
        debug.readThreads();debug.receive(["event":"failed","session":"ours","request_id":requests.last!["request_id"]!,"about":"threads","message":"Adapter unavailable"])
        XCTAssertEqual(debug.status,"paused");XCTAssertEqual(debug.frames.first?["id"] as? Int,11);XCTAssertEqual(debug.threads.count,201);XCTAssertFalse(debug.threadBusy);debug.close()
    }

    @MainActor func testOutputIsReadOnlySourceAnchoredAndRevisionBound() throws {
        // Contract: .hickory-source/frontends/contract.md
        let source = "value = 1\nprint(value)\n"
        let buffer = DocumentBuffer(file: FileNode(name: "main.py", path: "main.py", dir: false), text: FileText(path: "main.py", content: source, hash: "base"))
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.launchSource = source; _ = debug.beginLaunchRequest()
        debug.receive(["event": "started", "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///main.py", "session": "ours", "breakpoints": []])
        debug.receive(["event": "run_output", "session": "old", "events": [["text": "wrong"]]])
        XCTAssertTrue(debug.output.isEmpty)
        debug.receive(["event": "run_output", "session": "ours", "events": [["category": "stdout", "text": "1\n"], ["category": "stderr", "text": "warning\n", "source": "main.py", "line": 1]]])
        XCTAssertEqual(debug.outputPlacements(buffer)[1], "1\nwarning\n")
        XCTAssertEqual(debug.output.last?.source, "main.py"); XCTAssertEqual(debug.output.last?.line, 1)
        XCTAssertEqual(buffer.text, source); XCTAssertFalse(buffer.dirty)
        buffer.text += "# edited"
        XCTAssertTrue(debug.outputPlacements(buffer).isEmpty)
        let document = "<h:file path=\"main.py\">\nprint(1)\n\nprint(2)\n</h:file>\n"
        let book = DocumentBuffer(file: FileNode(name: "workbook.md", path: "workbook.md", dir: false), text: FileText(path: "workbook.md", content: document, hash: "base"))
        debug.owner = "workbook.md"; debug.launchSource = document; _ = debug.beginLaunchRequest()
        debug.receive(["event": "started", "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///workbook.md", "session": "document", "breakpoints": []])
        debug.receive(["event": "run_output", "session": "document", "events": [["category": "stdout", "text": "1\n", "source": "workbook.md", "line": 1]]])
        XCTAssertNotNil(debug.outputPlacements(book)[4], "A blank line must not split the producing file block")
    }
    @MainActor func testRecordedOutputUsesByteSpanAfterUnicodeBlock() {
        let source = "<h:exec>\necho café\n</h:exec>\n"
        let buffer = DocumentBuffer(file: FileNode(name: "workbook.md", path: "workbook.md", dir: false), text: FileText(path: "workbook.md", content: source, hash: "base"))
        buffer.receipts = [InlineReceipt(kind: "exec", span: [0, source.utf8.count - 1], command: nil, status: nil, transcript: [InlineReceiptEvent(kind: "out", data: "café\n")], recorded_output: nil)]
        buffer.receiptSource = source
        XCTAssertEqual(buffer.recordedOutputPlacements(), [2: "café\n"])
        XCTAssertEqual(buffer.text, source)
        buffer.text += "changed"
        XCTAssertTrue(buffer.recordedOutputPlacements().isEmpty)
    }

    @MainActor func testNativeDebugRejectsOtherSessionAndCarriesBoundCondition() throws {
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; debug.launchSource = "value = 1\nprint(value)\n"; _ = debug.beginLaunchRequest()
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        debug.toggle("main.py", line: 0, condition: "value > 0")
        debug.receive(["event": "started", "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///other.py", "session": "wrong"])
        XCTAssertNil(debug.sessionID)
        debug.receive(["event": "started", "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///main.py", "session": "ours", "breakpoints": [["line": 0, "moved_to": 1, "state": "bound"]]])
        XCTAssertEqual(debug.breakpoints["main.py"]?.first?["condition"] as? String, "value > 0")
        XCTAssertEqual(debug.breakpoints["main.py"]?.first?["line"] as? Int, 1)
        var revealed = ""; debug.onLocation = {location in revealed="\(location.path):\(location.line)"}
        debug.receive(["event": "stopped", "session": "old", "line": 0]); XCTAssertEqual(debug.status, "running")
        debug.receive(["event": "stopped", "session": "ours", "stop":1,"thread":1,"line": NSNull(), "frames": [["id": 7, "name": "helper", "source": "helper.py", "source_line": 2,"source_hash":nativeSourceHash("before")]], "variables": [["name": "value", "value": "1"]]])
        XCTAssertEqual(revealed, "helper.py:2"); XCTAssertEqual(debug.line(in: "helper.py"), 2)
        let helper = DocumentBuffer(file: FileNode(name: "helper.py", path: "helper.py", dir: false), text: FileText(path: "helper.py", content: "before", hash: "base"))
        XCTAssertTrue(debug.sourceMatches(helper))
        helper.text = "saved replacement"; helper.base = helper.text
        XCTAssertFalse(debug.sourceMatches(helper), "Saving changed callee text must not reattach old positional values")
        debug.evaluate("value", watch: true); XCTAssertEqual(requests.last?["frame"] as? Int, 7)
        debug.step("over"); XCTAssertEqual(requests.last?["how"] as? String, "over"); XCTAssertTrue(debug.variables.isEmpty)
        debug.receive(["event": "value", "session": "ours", "expression": "value", "value": "old"]); XCTAssertTrue(debug.values.isEmpty)
        debug.receive(["event": "finished", "session": "ours"]); debug.step("over"); XCTAssertNil(debug.sessionID); XCTAssertEqual(debug.status, "finished")
    }
    @MainActor func testStopWhileStartingStopsTheAcknowledgedSession() throws {
        let debug = NativeDebugger(engine: try Engine(address: "http://fixture.invalid"))
        debug.owner = "main.py"; _ = debug.beginLaunchRequest()
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        debug.stop(); XCTAssertEqual(debug.status, "stopping"); XCTAssertTrue(requests.isEmpty)
        debug.receive(["event": "started", "request_id":debug.launchRequestID ?? "missing", "doc": "hick:///main.py", "session": "late", "breakpoints": []])
        XCTAssertEqual(requests.last?["op"] as? String, "stop"); XCTAssertEqual(requests.last?["session"] as? String, "late")
        debug.receive(["event": "ended", "session": "late"]); XCTAssertEqual(debug.status, "finished")
    }
    @MainActor func testLiterateLaunchUsesDocumentExpectationAndRejectsUnsavedText() async throws {
        let config = URLSessionConfiguration.ephemeral; config.protocolClasses = [ResponseProtocol.self]
        let engine = try Engine(address: "http://fixture.invalid", session: URLSession(configuration: config))
        let debug = NativeDebugger(engine: engine)
        let source = "# Prose\n<h:file path=\"main.py\">\nprint(1)\n</h:file>\n"
        let file = FileNode(name: "source.md", path: "source.md", dir: false, doc_id: "managed")
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        ResponseProtocol.status = 200
        ResponseProtocol.response = Data(#"{"debug_protocol":2,"profiles":[{"id":"other","name":"Other","can_debug":true,"program":"other.py"},{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}"#.utf8)
        defer { ResponseProtocol.responsesByPath = [:] }
        ResponseProtocol.responsesByPath["/api/docs/managed/render"] = try JSONSerialization.data(withJSONObject: ["blocks": [["kind": "file", "path": "main.py", "span": [8, source.utf8.count]]]])
        var requests: [[String: Any]] = []; debug.emitForTest = { requests.append($0) }
        defer { debug.close() }
        await debug.prepare(buffer, line: 2)
        XCTAssertEqual(ResponseProtocol.captured["program"] as? String, "main.py")
        XCTAssertEqual(ResponseProtocol.captured["expected_document"] as? String, source)
        XCTAssertNil(ResponseProtocol.captured["expected_source"])
        XCTAssertEqual(debug.status, "starting"); XCTAssertTrue(debug.profiles.isEmpty)
        XCTAssertEqual((requests.last?["profile"] as? [String: Any])?["id"] as? String, "python")
        debug.close()
        buffer.text += "edited"; await debug.prepare(buffer)
        XCTAssertTrue(debug.message.contains("Save this file")); XCTAssertFalse(debug.sourceMatches(buffer))
    }
    @MainActor func testLiveNativeDebugChannel() async throws {
        guard let origin = ProcessInfo.processInfo.environment["HICKORY_NATIVE_DEBUG_ORIGIN"] else { throw XCTSkip("Set HICKORY_NATIVE_DEBUG_ORIGIN for a real backend with debugpy installed") }
        let engine = try Engine(address: origin), file = FileNode(name: "debug.py", path: "debug.py", dir: false)
        let loaded = try await engine.file(file), buffer = DocumentBuffer(file: file, text: loaded)
        let debug = NativeDebugger(engine: engine); defer { debug.close() }
        debug.toggle(file.path, line: 1); await debug.prepare(buffer)
        for _ in 0..<200 { if debug.status == "paused" || debug.status == "failed" { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertEqual(debug.status, "paused", debug.message)
        guard debug.status == "paused" else { return }
        debug.readThreads();for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertFalse(debug.threads.isEmpty,debug.threadNotice)
        debug.step("over")
        for _ in 0..<200 { if debug.status == "paused" || debug.status == "failed" { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertEqual(debug.status, "paused", debug.message); XCTAssertFalse(debug.variables.isEmpty)
        debug.evaluate("value", watch: true)
        for _ in 0..<100 { if debug.values["value"] != nil { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertNotNil(debug.values["value"]); XCTAssertEqual(buffer.text, loaded.content); XCTAssertFalse(buffer.dirty)
        debug.stop()
        for _ in 0..<100 { if !debug.active { break }; try await Task.sleep(for: .milliseconds(50)) }
        XCTAssertFalse(debug.active, debug.message)
    }
    @MainActor func testLiveNativeThreadSelectionAndScopedEvaluation()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_DEBUG_ORIGIN"] else{throw XCTSkip("Set HICKORY_NATIVE_DEBUG_ORIGIN for the real multi-thread debugger fixture.")}
        let engine=try Engine(address:origin),file=FileNode(name:"debug.py",path:"debug.py",dir:false),loaded=try await engine.file(file)
        guard let line=loaded.content.components(separatedBy:"\n").firstIndex(where:{$0.contains("# thread-stop")}) else{throw XCTSkip("Use a debug.py fixture with a worker and a # thread-stop breakpoint.")}
        let buffer=DocumentBuffer(file:file,text:loaded),debug=NativeDebugger(engine:engine);defer{debug.close()}
        buffer.selection=NSRange(location:2,length:0);debug.toggle(file.path,line:line);await debug.prepare(buffer)
        for _ in 0..<400 {if debug.status=="paused" || debug.status=="failed"{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",debug.message);guard debug.status=="paused" else{return}
        debug.readThreads();for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        let original=try XCTUnwrap(debug.selectedThread),worker=try XCTUnwrap(debug.threads.first(where:{$0.name == "hickory-worker"}),debug.threadNotice)
        debug.selectThread(worker.id,snapshot:debug.threadSnapshot)
        for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.selectedThread,worker.id,debug.threadNotice);XCTAssertFalse(debug.frames.isEmpty)
        let workerFrame=try XCTUnwrap(debug.frames.first(where:{$0["name"] as? String == "worker"}),"Worker frame absent")
        debug.select(workerFrame,snapshot:debug.frameSnapshot)
        debug.readFrameLocals();for _ in 0..<200 {if !debug.frameBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertTrue(debug.frameVariablesCurrent,debug.frameNotice)
        debug.evaluate("worker_value");for _ in 0..<200 {if debug.values["worker_value"] != nil{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.values["worker_value"],"41",debug.message+debug.evaluationNotice)
        debug.selectThread(original,snapshot:debug.threadSnapshot);for _ in 0..<200 {if !debug.threadBusy{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.selectedThread,original,debug.threadNotice)
        debug.evaluate("main_value");for _ in 0..<200 {if debug.values["main_value"] != nil{break};try await Task.sleep(for:.milliseconds(50))}
        XCTAssertEqual(debug.values["main_value"],"7",debug.message+debug.evaluationNotice)
        XCTAssertEqual(buffer.text,loaded.content);XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertFalse(buffer.dirty)
        debug.stop();for _ in 0..<200 {if !debug.active{break};try await Task.sleep(for:.milliseconds(50))};XCTAssertFalse(debug.active,debug.message)
    }

}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Tests/SaveTests.swift"><ui:paste select="#simple-d0dada29d6-0"/><ui:paste select="#simple-apple-debug-witnesses"/></ui:file>

</ui:doc>
