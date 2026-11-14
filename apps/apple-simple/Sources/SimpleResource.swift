import Foundation
private class SimpleResourceMarker {}
/** Resolve only bundled capability assets; no source workspace is searched. */
func simpleResource(_ name:String,withExtension ext:String)->URL? {
    guard !name.isEmpty,!ext.isEmpty,!name.contains("/"),!name.contains("\\"),name != ".",name != "..",!ext.contains("/"),!ext.contains("\\") else {return nil}
    let owner=Bundle(for:SimpleResourceMarker.self)
    let roots=[Bundle.main.resourceURL,Bundle.main.bundleURL,owner.bundleURL.deletingLastPathComponent(),URL(fileURLWithPath:CommandLine.arguments[0]).deletingLastPathComponent()].compactMap {$0}
    var bundles=[Bundle.main,owner]+roots.compactMap {Bundle(url:$0.appendingPathComponent("HickorySimple_HickorySimple.bundle"))}
    #if SWIFT_PACKAGE
    bundles.append(Bundle.module)
    #endif
    for bundle in bundles {
        if let url=bundle.url(forResource:name,withExtension:ext,subdirectory:"HickoryAssets/Resources") ?? bundle.url(forResource:name,withExtension:ext,subdirectory:"Resources") ?? bundle.url(forResource:name,withExtension:ext),url.isFileURL {return url}
    }
    return nil
}
