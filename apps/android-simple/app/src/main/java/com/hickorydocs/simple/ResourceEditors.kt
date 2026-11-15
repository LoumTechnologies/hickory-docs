package com.hickorydocs.simple
import java.util.Collections
import java.util.IdentityHashMap
data class ResourceSource(val identity:String,val text:String)
data class ResourceEditorPatch(val path:String,val before:ResourceSource?,val after:ResourceSource?)
data class ResourceEditorCapture<T:Any>(val editor:T,val source:String)
data class ResourceEditorRoute<T:Any>(val identity:String,val from:String?,val to:String?,val before:String?,val after:String?,val editor:T?)
fun <T:Any> routeResourceEditors(patches:List<ResourceEditorPatch>,live:Map<String,ResourceEditorCapture<T>>,retained:Map<String,ResourceEditorCapture<T>> = emptyMap()):List<ResourceEditorRoute<T>> {
 data class Address(val path:String,val source:String)
 val before=linkedMapOf<String,Address>();val after=linkedMapOf<String,Address>();val paths=mutableSetOf<String>()
 require(patches.isNotEmpty()&&patches.size<=1000){"Choose a bounded complete resource namespace."}
 for(patch in patches){
  require(patch.path.isNotEmpty()&&paths.add(patch.path)&&(patch.before!=null||patch.after!=null)){"Resource paths must be unique and describe a source transition."}
  for((side,target) in listOf(patch.before to before,patch.after to after))if(side!=null){require(side.identity.isNotEmpty()&&!target.containsKey(side.identity)){"A source identity must have exactly one address per namespace."};target[side.identity]=Address(patch.path,side.text)}
 }
 require(live.keys.all{it in before}&&retained.keys.all{it !in before&&it in after}){"Editor captures must belong to this complete transition."}
 val handles=Collections.newSetFromMap(IdentityHashMap<T,Boolean>())
 for((id,capture) in live.toList()+retained.toList()){
  val expected=before[id] ?: after.getValue(id)
  require(capture.source==expected.source&&handles.add(capture.editor)){"Source changed or one editor was assigned to multiple identities."}
 }
 require(before.keys.all{it in live}){"Acquire every existing source editor before namespace adoption."}
 return (before.keys+after.keys).map{id->ResourceEditorRoute(id,before[id]?.path,after[id]?.path,before[id]?.source,after[id]?.source,(live[id] ?: retained[id])?.editor)}
}
