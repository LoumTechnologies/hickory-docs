import Foundation
struct CommandDescriptor: Decodable {var id:String;var label:String;var group:String;var requires:String}
struct CommandContext {var editor:Bool;var writable:Bool;var dirty:Bool;var saving:Bool;var back:Bool;var forward:Bool;var tab:Bool;var groups:Int;var snippet:Bool=false;var snippetChoices:Bool=false;var sourceUndo:Bool=false;var sourceRedo:Bool=false}
struct WorkspaceCommand: Identifiable {var id:String;var label:String;var group:String;var reason:String?;var run:()->Void}
func commandReason(_ requires:String,_ context:CommandContext)->String? {
    guard ["always","editor","writable","save","back","forward","tab","multi","template","templateChoice","sourceUndo","sourceRedo"].contains(requires) else {return "Command unavailable. Reinstall the app."}
    if ["editor","writable","save","template","templateChoice","sourceUndo","sourceRedo"].contains(requires) && !context.editor {return "Open a source file first."}
    if ["writable","save","templateChoice","sourceUndo","sourceRedo"].contains(requires) && !context.writable {return "Choose an authored, editable source file."}
    if ["writable","save","templateChoice"].contains(requires) && context.saving {return "Wait for this file to finish saving."}
    if ["template","templateChoice"].contains(requires) && !context.snippet {return "Start a template or snippet first."}
    if requires=="templateChoice" && !context.snippetChoices {return "This field has no offered values."}
    if requires=="sourceUndo" && !context.sourceUndo || requires=="sourceRedo" && !context.sourceRedo {return "No text history in this direction."}
    if requires=="save" && !context.dirty {return "No unsaved changes."}
    if requires=="back" && !context.back || requires=="forward" && !context.forward {return "No navigation history in this direction."}
    if requires=="tab" && !context.tab {return "Open a tab first."}
    if requires=="multi" && (!context.tab || context.groups<2) {return "Open a tab and split the workspace first."}
    return nil
}
private class CommandResourceMarker {}
enum SimpleCommandCatalog {
    static let descriptors: [CommandDescriptor] = {
        let roots=[Bundle.main.resourceURL,Bundle.main.bundleURL,Bundle(for:CommandResourceMarker.self).bundleURL.deletingLastPathComponent(),URL(fileURLWithPath:CommandLine.arguments[0]).deletingLastPathComponent()].compactMap {$0}
        let bundles=[Bundle.main]+roots.compactMap {Bundle(url:$0.appendingPathComponent("HickorySimple_HickorySimple.bundle"))}
        for bundle in bundles {
            let url=bundle.url(forResource:"commands",withExtension:"json",subdirectory:"HickoryAssets/Resources") ?? bundle.url(forResource:"commands",withExtension:"json",subdirectory:"Resources") ?? bundle.url(forResource:"commands",withExtension:"json")
            if let url,let data=try? Data(contentsOf:url),let descriptors=try? JSONDecoder().decode([CommandDescriptor].self,from:data),Set(descriptors.map(\.id)).count==descriptors.count {return descriptors}
        }
        return []
    }()
}
@MainActor func workspaceCommands(_ context:@escaping ()->CommandContext,handlers:[String:()->Void],descriptors:[CommandDescriptor]=SimpleCommandCatalog.descriptors)->[WorkspaceCommand] {
    descriptors.map {item in WorkspaceCommand(id:item.id,label:item.label,group:item.group,reason:handlers[item.id] == nil ? "Unavailable on this platform.":commandReason(item.requires,context()),run:{if commandReason(item.requires,context())==nil {handlers[item.id]?()}})}
}
