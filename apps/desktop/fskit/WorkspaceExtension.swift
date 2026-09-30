import Foundation
import FSKit

@main
struct HickoryWorkspaceExtension: UnaryFileSystemExtension {
    typealias FileSystem = FSUnaryFileSystem & FSUnaryFileSystemOperations
    var fileSystem: FileSystem { WorkspaceFileSystem() }
}

final class WorkspaceFileSystem: FSUnaryFileSystem, FSUnaryFileSystemOperations {
    private var resource: FSPathURLResource?
    func probeResource(resource: FSResource, replyHandler: @escaping (FSProbeResult?, Error?) -> Void) {
        guard let r = resource as? FSPathURLResource else { return replyHandler(nil, POSIXError(.ENODEV)) }
        guard r.url.startAccessingSecurityScopedResource() else { return replyHandler(nil, POSIXError(.EACCES)) }
        defer { r.url.stopAccessingSecurityScopedResource() }
        guard FileManager.default.fileExists(atPath: r.url.appendingPathComponent("connection.json").path) else { return replyHandler(nil, POSIXError(.ENODEV)) }
        replyHandler(.usable(name: "Hickory Workspace", containerID: FSContainerIdentifier(uuid: UUID())), nil)
    }
    func loadResource(resource: FSResource, options: FSTaskOptions, replyHandler: @escaping (FSVolume?, Error?) -> Void) {
        guard let r = resource as? FSPathURLResource, r.url.startAccessingSecurityScopedResource() else { return replyHandler(nil, POSIXError(.EACCES)) }
        do {
            let bridge = try WorkspaceBridge(resource: r.url)
            _ = try bridge.call(["op": "stat", "path": ""])
            self.resource = r
            containerStatus = .ready
            replyHandler(WorkspaceVolume(bridge: bridge), nil)
        } catch { r.url.stopAccessingSecurityScopedResource(); replyHandler(nil, error) }
    }
    func unloadResource(resource: FSResource, options: FSTaskOptions, replyHandler: @escaping (Error?) -> Void) {
        self.resource?.url.stopAccessingSecurityScopedResource()
        self.resource = nil
        replyHandler(nil)
    }
}

/// Only the mount descriptor is accessible to the extension. Actual repository
/// I/O belongs to the Rust engine, so this contains no passthrough file writes.
final class WorkspaceBridge {
    let endpoint: URL
    init(resource: URL) throws {
        let data = try Data(contentsOf: resource.appendingPathComponent("connection.json"))
        let value = try JSONSerialization.jsonObject(with: data) as? [String: String]
        guard let raw = value?["url"], let url = URL(string: raw), url.scheme == "http", url.host == "127.0.0.1", url.user == nil else { throw POSIXError(.EINVAL) }
        endpoint = url
    }
    func call(_ value: [String: Any]) throws -> Any {
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONSerialization.data(withJSONObject: value)
        request.timeoutInterval = 30
        let done = DispatchSemaphore(value: 0)
        var result: Result<Any, Error> = .failure(POSIXError(.EIO))
        let task = URLSession.shared.dataTask(with: request) { data, response, error in
            defer { done.signal() }
            do {
                if let error = error { throw error }
                guard let response = response as? HTTPURLResponse, response.statusCode == 200, let data = data else { throw POSIXError(.EIO) }
                guard let reply = try JSONSerialization.jsonObject(with: data) as? [String: Any] else { throw POSIXError(.EIO) }
                if let message = reply["error"] as? String { throw NSError(domain: NSPOSIXErrorDomain, code: (reply["errno"] as? NSNumber)?.intValue ?? Int(EPERM), userInfo: [NSLocalizedDescriptionKey: message]) }
                result = .success(reply["result"] ?? [:])
            } catch { result = .failure(error) }
        }
        task.resume()
        guard done.wait(timeout: .now() + 35) == .success else { task.cancel(); throw POSIXError(.ETIMEDOUT) }
        return try result.get()
    }
}
