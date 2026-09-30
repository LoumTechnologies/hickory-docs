import Foundation
import FSKit

final class WorkspaceItem: FSItem {
    var path: String
    let identifier: UInt64
    var handle: String?
    var opens = 0
    var writable = false
    init(path: String, identifier: UInt64) { self.path = path; self.identifier = identifier; super.init() }
}

final class WorkspaceVolume: FSVolume, FSVolume.Operations, FSVolume.ReadWriteOperations, FSVolume.OpenCloseOperations {
    let bridge: WorkspaceBridge
    let queue = DispatchQueue(label: "com.loumtechnologies.hickorydocs.workspace")
    let root = WorkspaceItem(path: "", identifier: 2)
    var items: [String: WorkspaceItem] = [:]
    var nextID: UInt64 = 3
    init(bridge: WorkspaceBridge) {
        self.bridge = bridge
        super.init(volumeID: FSVolume.Identifier(uuid: UUID()), volumeName: FSFileName(string: "Hickory Workspace"))
        items[""] = root
    }
    func item(_ raw: FSItem) throws -> WorkspaceItem { guard let item = raw as? WorkspaceItem else { throw POSIXError(.EINVAL) }; return item }
    func cached(_ path: String) -> WorkspaceItem {
        if let item = items[path] { return item }
        let item = WorkspaceItem(path: path, identifier: nextID); nextID += 1; items[path] = item; return item
    }
    func child(_ name: FSFileName, _ dir: FSItem) throws -> String {
        guard let name = name.string, !name.isEmpty, name != ".", name != "..", !name.contains("/"), !name.contains("\0") else { throw POSIXError(.EINVAL) }
        let dir = try item(dir)
        return dir.path.isEmpty ? name : "\(dir.path)/\(name)"
    }
    func attrs(_ item: WorkspaceItem) throws -> FSItem.Attributes {
        guard let stat = try bridge.call(["op":"stat", "path":item.path]) as? [String:Any] else { throw POSIXError(.EIO) }
        return attributes(stat, id: item.identifier)
    }
    func attributes(_ stat: [String:Any], id: UInt64) -> FSItem.Attributes {
        let a = FSItem.Attributes()
        a.type = (stat["directory"] as? Bool == true) ? .directory : .file
        a.size = (stat["size"] as? NSNumber)?.uint64Value ?? 0
        a.allocSize = a.size
        a.mode = (stat["mode"] as? NSNumber)?.uint32Value ?? 0o644
        a.uid = getuid(); a.gid = getgid(); a.linkCount = 1
        a.fileID = FSItem.Identifier(rawValue: id) ?? .invalid
        a.parentID = .rootDirectory
        return a
    }
    func activate(options: FSTaskOptions, replyHandler: @escaping (FSItem?, Error?) -> Void) { replyHandler(root, nil) }
    func deactivate(options: FSDeactivateOptions, replyHandler: @escaping (Error?) -> Void) {
        queue.async { do { _ = try self.bridge.call(["op":"sync"]); replyHandler(nil) } catch { replyHandler(error) } }
    }
    func mount(options: FSTaskOptions, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func unmount(replyHandler: @escaping () -> Void) { replyHandler() }
    func synchronize(flags: FSSyncFlags, replyHandler: @escaping (Error?) -> Void) {
        queue.async { do { _ = try self.bridge.call(["op":"sync"]); replyHandler(nil) } catch { replyHandler(error) } }
    }
    var maximumLinkCount: Int { 1 }
    var maximumNameLength: Int { 255 }
    var restrictsOwnershipChanges: Bool { true }
    var truncatesLongNames: Bool { false }
    var volumeStatistics: FSStatFSResult {
        let r = FSStatFSResult(fileSystemTypeName: "hickory"); r.blockSize = 4096; r.ioSize = 65536
        if let space = try? bridge.call(["op":"space"]) as? [String: NSNumber] {
            r.totalBlocks = (space["total"]?.uint64Value ?? 0) / 4096
            r.availableBlocks = (space["available"]?.uint64Value ?? 0) / 4096
            r.freeBlocks = (space["free"]?.uint64Value ?? 0) / 4096
            r.usedBlocks = r.totalBlocks - r.freeBlocks
        }
        return r
    }
    var supportedVolumeCapabilities: FSVolume.SupportedCapabilities { let c = FSVolume.SupportedCapabilities(); c.caseFormat = .sensitive; c.doesNotSupportSettingFilePermissions = true; return c }
    func getAttributes(_ request: FSItem.GetAttributesRequest, of item: FSItem, replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) {
        queue.async { do { replyHandler(try self.attrs(self.item(item)), nil) } catch { replyHandler(nil, error) } }
    }
    func setAttributes(_ request: FSItem.SetAttributesRequest, on item: FSItem, replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) {
        queue.async { do {
            let item = try self.item(item)
            if request.isValid(.size) {
                let handle = try self.open(item, write: true)
                _ = try self.bridge.call(["op":"truncate", "handle":handle,"length":request.size])
            }
            replyHandler(try self.attrs(item), nil)
        } catch { replyHandler(nil, error) } }
    }
    func lookupItem(named name: FSFileName, inDirectory directory: FSItem, replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        queue.async { do {
            let path = try self.child(name, directory)
            _ = try self.bridge.call(["op":"stat", "path":path])
            replyHandler(self.cached(path), name, nil)
        } catch { replyHandler(nil, nil, error) } }
    }
    func reclaimItem(_ item: FSItem, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func readSymbolicLink(_ item: FSItem, replyHandler: @escaping (FSFileName?, Error?) -> Void) { replyHandler(nil, POSIXError(.ENOTSUP)) }
    func createItem(named name: FSFileName, type: FSItem.ItemType, inDirectory directory: FSItem, attributes: FSItem.SetAttributesRequest, replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        queue.async { do {
            guard type == .file || type == .directory else { throw POSIXError(.ENOTSUP) }
            let path = try self.child(name, directory)
            _ = try self.bridge.call(["op":"create","path":path,"directory":type == .directory])
            replyHandler(self.cached(path), name, nil)
        } catch { replyHandler(nil, nil, error) } }
    }
    func createSymbolicLink(named name: FSFileName, inDirectory directory: FSItem, attributes: FSItem.SetAttributesRequest, linkContents: FSFileName, replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) { replyHandler(nil, nil, POSIXError(.ENOTSUP)) }
    func createLink(to item: FSItem, named name: FSFileName, inDirectory directory: FSItem, replyHandler: @escaping (FSFileName?, Error?) -> Void) { replyHandler(nil, POSIXError(.ENOTSUP)) }
    func removeItem(_ item: FSItem, named name: FSFileName, fromDirectory directory: FSItem, replyHandler: @escaping (Error?) -> Void) {
        queue.async { do { let path = try self.child(name,directory); _ = try self.bridge.call(["op":"remove","path":path]); self.items.removeValue(forKey:path); replyHandler(nil) } catch { replyHandler(error) } }
    }
    func renameItem(_ item: FSItem, inDirectory sourceDirectory: FSItem, named sourceName: FSFileName, to destinationName: FSFileName, inDirectory destinationDirectory: FSItem, overItem: FSItem?, replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        queue.async { do {
            let source = try self.child(sourceName, sourceDirectory), dest = try self.child(destinationName, destinationDirectory)
            _ = try self.bridge.call(["op":"rename","path":source,"to":dest])
            self.items.removeValue(forKey:source); let moved = try self.item(item); moved.path = dest; self.items[dest] = moved
            replyHandler(destinationName, nil)
        } catch { replyHandler(nil, error) } }
    }
    func enumerateDirectory(_ directory: FSItem, startingAt cookie: FSDirectoryCookie, verifier: FSDirectoryVerifier, attributes: FSItem.GetAttributesRequest?, packer: FSDirectoryEntryPacker, replyHandler: @escaping (FSDirectoryVerifier, Error?) -> Void) {
        queue.async { do {
            let dir = try self.item(directory)
            guard let entries = try self.bridge.call(["op":"list","path":dir.path]) as? [[String:Any]] else { throw POSIXError(.EIO) }
            for (index, entry) in entries.enumerated() where index >= Int(cookie.rawValue) {
                guard let path = entry["path"] as? String else { throw POSIXError(.EIO) }
                let item = self.cached(path)
                if !packer.packEntry(name: FSFileName(string: (path as NSString).lastPathComponent), itemType: (entry["directory"] as? Bool == true) ? .directory : .file, itemID: FSItem.Identifier(rawValue:item.identifier) ?? .invalid, nextCookie: FSDirectoryCookie(UInt64(index + 1)), attributes: attributes == nil ? nil : self.attributes(entry, id:item.identifier)) { break }
            }
            replyHandler(FSDirectoryVerifier(0), nil)
        } catch { replyHandler(FSDirectoryVerifier(0), error) } }
    }
    func open(_ item: WorkspaceItem, write: Bool) throws -> String {
        if let handle = item.handle {
            // FSKit identifies the item, rather than the calling descriptor.
            guard !write || item.writable else { throw POSIXError(.EBUSY) }
            return handle
        }
        guard let result = try bridge.call(["op":"open","path":item.path,"write":write]) as? [String:String], let handle = result["handle"] else { throw POSIXError(.EIO) }
        item.handle = handle; item.writable = write; return handle
    }
    func openItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        queue.async { do { let item = try self.item(item); if item !== self.root { _ = try self.open(item,write:modes.contains(.write)); item.opens += 1 }; replyHandler(nil) } catch { replyHandler(error) } }
    }
    func closeItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        queue.async { do {
            let item = try self.item(item); item.opens = max(0,item.opens - 1)
            if item.opens == 0, let handle = item.handle { _ = try self.bridge.call(["op":"close","handle":handle]); item.handle = nil; item.writable = false }
            replyHandler(nil)
        } catch { replyHandler(error) } }
    }
    func read(from item: FSItem, at offset: off_t, length: Int, into buffer: FSMutableFileDataBuffer, replyHandler: @escaping (Int, Error?) -> Void) {
        queue.async { do {
            let item = try self.item(item), handle = try self.open(item,write:false)
            guard let result = try self.bridge.call(["op":"read","handle":handle,"offset":offset,"length":length]) as? [String:String], let encoded = result["data"], let data = Data(base64Encoded:encoded) else { throw POSIXError(.EIO) }
            _ = buffer.withUnsafeMutableBytes { target in data.copyBytes(to:target) }
            replyHandler(data.count,nil)
        } catch { replyHandler(0,error) } }
    }
    func write(contents: Data, to item: FSItem, at offset: off_t, replyHandler: @escaping (Int, Error?) -> Void) {
        queue.async { do {
            let handle = try self.open(self.item(item),write:true)
            _ = try self.bridge.call(["op":"write","handle":handle,"offset":offset,"data":contents.base64EncodedString()])
            replyHandler(contents.count,nil)
        } catch { replyHandler(0,error) } }
    }
}
