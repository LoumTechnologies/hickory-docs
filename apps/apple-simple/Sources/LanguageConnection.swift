import Foundation
@MainActor final class NativeLanguageConnection {
    private struct Subscriber { var uri: String; var receive: ([String:Any])->Void; var failed: (Error)->Void }
    private struct Document { var text: String; var version: Int }
    private struct Pending { var owner: UUID; var reply: CheckedContinuation<Any,Error> }
    private let engine: Engine
    private var subscribers: [UUID:Subscriber] = [:]
    private var documents: [String:Document] = [:]
    private var pending: [Int:Pending] = [:]
    private var nextID = 1
    private var epoch = UUID()
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
            if !replayed { sendDocument(uri,change:existing) }
        } catch { failed(error) }
        return token
    }
    func detach(_ token: UUID) {
        guard let entry=subscribers.removeValue(forKey:token) else {return}
        let abandoned=pending.filter {$0.value.owner==token};for(id,request) in abandoned {pending.removeValue(forKey:id);request.reply.resume(throwing:CancellationError())}
        if !subscribers.values.contains(where:{$0.uri==entry.uri}) {
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
        if let uri=(params["textDocument"] as? [String:Any])?["uri"] as? String,uri != subscriber.uri {throw EngineError(message:"Language request source identity changed.")}
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
                if case .data(let bytes)=message,bytes.first==2,let object=try JSONSerialization.jsonObject(with:bytes.dropFirst()) as? [String:Any] {self.receive(object)}
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
        if let id=object["id"] as? Int,object["method"]==nil,let request=pending.removeValue(forKey:id) {
            if let error=object["error"] as? [String:Any] {request.reply.resume(throwing:EngineError(message:error["message"] as? String ?? "Language request refused"))}
            else {request.reply.resume(returning:object["result"] ?? NSNull())};return
        }
        if let id=object["id"],object["method"] != nil {
            if object["method"] as? String == "workspace/applyEdit" {send(["jsonrpc":"2.0","id":id,"result":["applied":false,"failureReason":"A source preview is required."]])}
            else {send(["jsonrpc":"2.0","id":id,"error":["code":-32601,"message":"Client method unavailable"]])};return
        }
        let target = object["method"] as? String == "textDocument/publishDiagnostics" ? (object["params"] as? [String:Any])?["uri"] as? String : nil
        let listeners=Array(subscribers.values)
        for listener in listeners where target==nil || target==listener.uri {listener.receive(object)}
    }
    private func fail(_ error: Error) {
        epoch=UUID();receiveTask?.cancel();sendTask?.cancel();socket?.cancel(with:.goingAway,reason:nil);socket=nil;retryAfter=Date().addingTimeInterval(1)
        // Source-bound captures retire before request continuations resume.
        for listener in Array(subscribers.values) {listener.failed(error)}
        let replies=Array(pending.values);pending.removeAll();for request in replies {request.reply.resume(throwing:error)}
    }
}
