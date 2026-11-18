import Foundation
struct ResourceSource {var identity:String;var text:String}
struct ResourceEditorPatch {var path:String;var before:ResourceSource?;var after:ResourceSource?}
struct ResourceEditorCapture<Editor:AnyObject> {var editor:Editor;var source:String}
struct ResourceEditorRoute<Editor:AnyObject> {var identity:String;var from:String?;var to:String?;var before:String?;var after:String?;var editor:Editor?}
private struct ResourceEditorAddress {var identity:String;var path:String;var source:String}
func routeResourceEditors<Editor:AnyObject>(_ patches:[ResourceEditorPatch],live:[String:ResourceEditorCapture<Editor>],retained:[String:ResourceEditorCapture<Editor>]=[:]) throws->[ResourceEditorRoute<Editor>] {
 func key(_ text:String)->Data {Data(text.utf8)}
 func refuse(_ message:String)->NSError {NSError(domain:"ResourceEditors",code:1,userInfo:[NSLocalizedDescriptionKey:message])}
 guard !patches.isEmpty,patches.count<=1000 else{throw refuse("Choose a bounded complete resource namespace.")}
 var before:[Data:ResourceEditorAddress]=[:],after:[Data:ResourceEditorAddress]=[:],paths=Set<Data>(),order:[Data]=[]
 for patch in patches {
  guard !patch.path.isEmpty,paths.insert(key(patch.path)).inserted,patch.before != nil || patch.after != nil else{throw refuse("Resource paths must be unique and describe a source transition.")}
  if let side=patch.before {let id=key(side.identity);guard !id.isEmpty,before[id]==nil else{throw refuse("A source identity must have exactly one address per namespace.")};before[id]=ResourceEditorAddress(identity:side.identity,path:patch.path,source:side.text);order.append(id)}
  if let side=patch.after {let id=key(side.identity);guard !id.isEmpty,after[id]==nil else{throw refuse("A source identity must have exactly one address per namespace.")};after[id]=ResourceEditorAddress(identity:side.identity,path:patch.path,source:side.text)}
 }
 let captured=Dictionary(uniqueKeysWithValues:live.map{(key($0.key),$0.value)}),held=Dictionary(uniqueKeysWithValues:retained.map{(key($0.key),$0.value)})
 guard captured.keys.allSatisfy({before[$0] != nil}),held.keys.allSatisfy({before[$0]==nil && after[$0] != nil}) else{throw refuse("Editor captures must belong to this complete transition.")}
 var handles=Set<ObjectIdentifier>()
 for (id,capture) in Array(captured)+Array(held) {guard let expected=before[id] ?? after[id],key(capture.source)==key(expected.source),handles.insert(ObjectIdentifier(capture.editor)).inserted else{throw refuse("Source changed or one editor was assigned to multiple identities.")}}
 guard before.keys.allSatisfy({captured[$0] != nil}) else{throw refuse("Acquire every existing source editor before namespace adoption.")}
 for patch in patches {if let side=patch.after {let id=key(side.identity);if before[id]==nil{order.append(id)}}}
 return order.map{id in let a=before[id],b=after[id];return ResourceEditorRoute(identity:(a ?? b)!.identity,from:a?.path,to:b?.path,before:a?.source,after:b?.source,editor:(captured[id] ?? held[id])?.editor)}
}

struct ResourceSaved:Codable {
 var kind:String;var text:String?;var sha256:String?;var readonly:Bool?;var executable:Bool?;var mode:Int?
 func matches(_ other:ResourceSaved)->Bool {
  func exact(_ a:String?,_ b:String?)->Bool {a.map{Data($0.utf8)}==b.map{Data($0.utf8)}}
  return exact(kind,other.kind) && (kind=="missing" || kind=="file" && exact(text,other.text) && exact(sha256,other.sha256) && readonly==other.readonly && executable==other.executable && mode==other.mode)
 }
}
struct ResourcePublication<File:Decodable>:Decodable {
 struct Outcome:Decodable {var complete:Bool;var recovery:String;var applied:[String]}
 struct Entry:Decodable {var file:File;var saved:ResourceSaved;var hash:String;var save_guard:String? = nil}
 var version:Int;var request_id:String;var workspace:String;var context:String;var outcome:Outcome;var saved_sources:[String:ResourceSaved]?;var source_files:[Entry];var adoption_current:Bool=false;var adoption_reason:String?=nil;var entry_outcome:WorkspaceEntryPublication?=nil
}
func confirmedResourceFiles<File>(_ patches:[ResourceEditorPatch],receipt:ResourcePublication<File>,request:String,workspace:String,context:String,expected:[String:ResourceSaved],address:(File)->(path:String,authored:Bool)) throws->[Data:ResourcePublication<File>.Entry] {
 func key(_ value:String)->Data {Data(value.utf8)}
 func refuse()->NSError {NSError(domain:"ResourceEditors",code:2,userInfo:[NSLocalizedDescriptionKey:receipt.adoption_reason ?? "Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors."])}
 guard receipt.version==2,receipt.adoption_current,key(receipt.request_id)==key(request),key(receipt.workspace)==key(workspace),key(receipt.context)==key(context),receipt.outcome.complete,!receipt.outcome.recovery.isEmpty,let saved=receipt.saved_sources else{throw refuse()}
 let paths=Set(patches.map{key($0.path)}),savedByKey=Dictionary(uniqueKeysWithValues:saved.map{(key($0.key),$0.value)}),expectedByKey=Dictionary(uniqueKeysWithValues:expected.map{(key($0.key),$0.value)})
 guard paths.count==patches.count,Set(savedByKey.keys)==paths,Set(expectedByKey.keys)==paths,Set(receipt.outcome.applied.map{key($0)})==paths,receipt.outcome.applied.count==paths.count else{throw refuse()}
 for patch in patches {guard let state=savedByKey[key(patch.path)],let reviewed=expectedByKey[key(patch.path)],state.matches(reviewed),(state.kind=="file")==((patch.after) != nil) else{throw refuse()}}
 var files:[Data:ResourcePublication<File>.Entry]=[:]
 for entry in receipt.source_files {let target=address(entry.file),path=key(target.path);guard paths.contains(path),files[path]==nil,target.authored,let state=savedByKey[path],state.kind=="file",state.matches(entry.saved),state.text != nil else{throw refuse()};files[path]=entry}
 guard files.count==patches.filter({$0.after != nil}).count else{throw refuse()};return files
}
