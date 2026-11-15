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
