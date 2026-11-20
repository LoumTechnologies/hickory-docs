<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Share language transport across native source buffers

For maintainers changing workspace services independently of the editor renderer.

Language knowledge belongs to source buffers, while wire lifetime belongs to the workspace. One shared connection owns request IDs, pending replies, document snapshots and reconnection replay. Notifications route by source URI; capability announcements fan out and are replayed to tabs that join the open connection later. The semantic token legend belongs to that connection, so close and failure discard the cached announcement. Each URI has one source session in a workspace; views borrow it. Detaching a source cancels only its requests and closes its document. Duplicate sessions cannot overwrite the held source or borrow another owner's request authority. A failed connection invalidates every subscriber before replies resume, retaining readable source. The next read may reconnect and replay all held full-text documents. This ordinary transport object can be replaced without rewriting editor widgets.

Audit raw object keys before Foundation decodes a provider frame. Canonically equivalent contributor keys cannot merge into a partial edit; invalid identities retire only the optional language connection and pending proposals. Unrelated provider version metadata is not a source-test protocol version. Retained document URIs and notification routing use exact bytes, independently of their display spelling. Raw-key checking and Foundation parsing run on a worker; a changed connection epoch rejects the result before any listener or pending proposal sees it.

<ui:copy id="simple-apple-language-connection">
import Foundation
func nativeLanguageJSONObject(_ data:Data)throws->[String:Any]{
    try validateNativeJSON(data)
    guard let object=try JSONSerialization.jsonObject(with:data) as? [String:Any] else{throw EngineError(message:"Language provider sent a non-object message. Source text is retained.")}
    if let id=object["id"],!(id is NSNull){guard id is String || sourceChangeInteger(id) != nil else{throw EngineError(message:"Malformed language request identity refused. Source text is retained.")}}
    return object
}
// JSONSerialization produces immutable value containers here; only this private parser creates the cross-actor envelope.
private struct NativeLanguageJSONFrame:@unchecked Sendable {let object:[String:Any]}
@MainActor final class NativeLanguageConnection {
    private struct Subscriber { var uri: String; var receive: ([String:Any])->Void; var failed: (Error)->Void }
    private struct Document { var text: String; var version: Int }
    private struct Pending { var owner: UUID; var reply: CheckedContinuation<Any,Error> }
    private let engine: Engine
    private var subscribers: [UUID:Subscriber] = [:]
    private var documents: NativeSourceMap<Document> = [:]
    private var pending: [Int:Pending] = [:]
    private var nextID = 1
    private var epoch = UUID()
    private var capabilityAnnouncement: [String: Any]?
    private var retryAfter = Date.distantPast
    private var socket: URLSessionWebSocketTask?
    private var receiveTask: Task<Void,Never>?
    private var sendTask: Task<Void,Never>?
    init(engine: Engine) { self.engine=engine }
    deinit { receiveTask?.cancel();sendTask?.cancel();socket?.cancel(with:.goingAway,reason:nil) }
    var active: Bool { socket != nil }
    func attach(uri: String, source: String, version: Int, receive: @escaping ([String:Any])->Void, failed: @escaping (Error)->Void) -> UUID {
        let token=UUID(), existing=documents[uri] != nil
        guard !existing else {failed(EngineError(message:"Reuse the workspace language session for this source: \(uri)"));return token}
        subscribers[token]=Subscriber(uri:uri,receive:receive,failed:failed);documents[uri]=Document(text:source,version:version)
        do {
            let replayed=try startIfNeeded()
            if !replayed {
                sendDocument(uri,change:existing)
                if let capabilityAnnouncement { receive(capabilityAnnouncement) }
            }
        } catch { failed(error) }
        return token
    }
    func detach(_ token: UUID) {
        guard let entry=subscribers.removeValue(forKey:token) else {return}
        let abandoned=pending.filter {$0.value.owner==token};for(id,request) in abandoned {pending.removeValue(forKey:id);request.reply.resume(throwing:CancellationError())}
        if !subscribers.values.contains(where:{exactUTF8($0.uri,entry.uri)}) {
            documents.removeValue(forKey:entry.uri)
            if active {send(["jsonrpc":"2.0","method":"textDocument/didClose","params":["textDocument":["uri":entry.uri]]])}
        }
        if subscribers.isEmpty {close()}
    }
    func change(owner: UUID, source: String, version: Int) {
        guard let uri=subscribers[owner]?.uri,documents[uri] != nil else {return};documents[uri]=Document(text:source,version:version)
        do {if !(try startIfNeeded()) {sendDocument(uri,change:true)}} catch { /* Retain text for a later reconnect. */ }
    }
    func request(_ method: String, params: [String:Any], owner: UUID) async throws -> Any {
        guard let subscriber=subscribers[owner] else {throw EngineError(message:"Language source is closed. Retry language services.")}
        if let uri=(params["textDocument"] as? [String:Any])?["uri"] as? String,!exactUTF8(uri,subscriber.uri) {throw EngineError(message:"Language request source identity changed.")}
        _ = try startIfNeeded()
        let id=nextID;nextID += 1
        return try await withCheckedThrowingContinuation { reply in
            pending[id]=Pending(owner:owner,reply:reply)
            send(["jsonrpc":"2.0","id":id,"method":method,"params":params])
            Task { [weak self] in
                try? await Task.sleep(for:.seconds(12))
                self?.pending.removeValue(forKey:id)?.reply.resume(throwing:EngineError(message:"Language request timed out. Check the server installation and retry."))
            }
        }
    }
    func close() {
        capabilityAnnouncement = nil
        epoch=UUID();receiveTask?.cancel();sendTask?.cancel();receiveTask=nil;sendTask=nil
        socket?.cancel(with:.normalClosure,reason:nil);socket=nil;retryAfter = .distantPast
        let listeners=Array(subscribers.values);subscribers.removeAll();documents.removeAll()
        for listener in listeners {listener.failed(CancellationError())}
        let replies=Array(pending.values);pending.removeAll();for request in replies {request.reply.resume(throwing:CancellationError())}
    }
    private func startIfNeeded() throws -> Bool {
        if active {return false}
        guard Date() >= retryAfter else {throw EngineError(message:"Language connection unavailable. Retry language services shortly.")}
        epoch=UUID();let current=epoch
        var url=URLComponents(url:engine.origin,resolvingAgainstBaseURL:false)!
        url.scheme=engine.origin.scheme=="https" ? "wss":"ws";url.path="/api/ws";url.queryItems=[URLQueryItem(name:"doc",value:"workspace")]
        let socket=engine.session.webSocketTask(with:url.url!);self.socket=socket;socket.resume()
        receiveTask=Task { [weak self] in
            do {while !Task.isCancelled {
                let message=try await socket.receive()
                guard let self,self.epoch==current else {return}
                if case .data(let bytes)=message,bytes.first==2 {
                    let data=Data(bytes.dropFirst())
                    let frame=try await Task.detached(priority:.userInitiated){NativeLanguageJSONFrame(object:try nativeLanguageJSONObject(data))}.value
                    guard self.epoch==current,!Task.isCancelled else{return}
                    self.receive(frame.object)
                }
            }} catch {if let self,self.epoch==current,!Task.isCancelled {self.fail(error)}}
        }
        for uri in documents.keys.sorted() {sendDocument(uri,change:false)}
        return true
    }
    private func sendDocument(_ uri: String, change: Bool) {
        guard let document=documents[uri] else {return}
        if change {send(["jsonrpc":"2.0","method":"textDocument/didChange","params":["textDocument":["uri":uri,"version":document.version],"contentChanges":[["text":document.text]]]])}
        else {send(["jsonrpc":"2.0","method":"textDocument/didOpen","params":["textDocument":["uri":uri,"languageId":"hick","version":document.version,"text":document.text]]])}
    }
    private func send(_ object: [String:Any]) {
        guard let socket,let bytes=try? JSONSerialization.data(withJSONObject:object) else {return}
        let previous=sendTask,current=epoch
        sendTask=Task { [weak self] in
            await previous?.value;guard !Task.isCancelled else {return}
            do {try await socket.send(.data(Data([2])+bytes))}
            catch {if let self,self.epoch==current {self.fail(error)}}
        }
    }
    private func receive(_ object: [String:Any]) {
        if let id=sourceChangeInteger(object["id"]),object["method"]==nil,let request=pending.removeValue(forKey:id) {
            if let error=object["error"] as? [String:Any] {request.reply.resume(throwing:EngineError(message:error["message"] as? String ?? "Language request refused"))}
            else {request.reply.resume(returning:object["result"] ?? NSNull())};return
        }
        if let id=object["id"],object["method"] != nil {
            if object["method"] as? String == "workspace/applyEdit" {send(["jsonrpc":"2.0","id":id,"result":["applied":false,"failureReason":"A source preview is required."]])}
            else {send(["jsonrpc":"2.0","id":id,"error":["code":-32601,"message":"Client method unavailable"]])};return
        }
        if object["method"] as? String == "hick/serverCapabilities" { capabilityAnnouncement = object }
        let target = object["method"] as? String == "textDocument/publishDiagnostics" ? (object["params"] as? [String:Any])?["uri"] as? String : nil
        let listeners=Array(subscribers.values)
        for listener in listeners where target==nil || exactUTF8(target,listener.uri) {listener.receive(object)}
    }
    private func fail(_ error: Error) {
        capabilityAnnouncement = nil
        epoch=UUID();receiveTask?.cancel();sendTask?.cancel();socket?.cancel(with:.goingAway,reason:nil);socket=nil;retryAfter=Date().addingTimeInterval(1)
        // Source-bound captures retire before request continuations resume.
        for listener in Array(subscribers.values) {listener.failed(error)}
        let replies=Array(pending.values);pending.removeAll();for request in replies {request.reply.resume(throwing:error)}
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/LanguageConnection.swift"><ui:paste select="#simple-apple-language-connection"/></ui:file>
</ui:doc>
