package com.hickorydocs.simple
import java.util.Collections
import java.util.IdentityHashMap
class SourceAcquisitionLease(private val probe:(String?)->Boolean,private val adopt:(String,DocumentBuffer)->Unit) {
 fun current(allowing:String?=null)=probe(allowing)
 fun retain(path:String,buffer:DocumentBuffer)=adopt(path,buffer)
}
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

data class ResourceSaved(val kind:String,val text:String?=null,val sha256:String?=null,val readonly:Boolean?=null,val executable:Boolean?=null,val mode:Int?=null)
data class ResourcePublishedFile<File>(val file:File,val saved:ResourceSaved,val hash:String,val saveGuard:String?=null)
data class ResourcePublication<File>(val version:Int,val request:String,val workspace:String,val context:String,val complete:Boolean,val recovery:String,val applied:List<String>,val saved:Map<String,ResourceSaved>?,val files:List<ResourcePublishedFile<File>>,val adoptionCurrent:Boolean=false,val adoptionReason:String?=null)
fun <File> confirmedResourceFiles(patches:List<ResourceEditorPatch>,receipt:ResourcePublication<File>,request:String,workspace:String,context:String,expected:Map<String,ResourceSaved>,address:(File)->Pair<String,Boolean>):Map<String,ResourcePublishedFile<File>> {
 fun admit(ok:Boolean){require(ok){"Resource publication is not completely confirmed for this review. Inspect Source recovery before changing editors."}}
 admit(receipt.version==2&&receipt.request==request&&receipt.workspace==workspace&&receipt.context==context&&receipt.complete&&receipt.adoptionCurrent&&receipt.recovery.isNotEmpty())
 val paths=patches.map{it.path}.toSet();val saved=receipt.saved
 admit(paths.size==patches.size&&saved!=null&&saved.keys==paths&&expected.keys==paths&&receipt.applied.size==paths.size&&receipt.applied.toSet()==paths)
 for(patch in patches){val state=saved!!.getValue(patch.path);admit(state==expected[patch.path]&&(state.kind=="file")== (patch.after!=null)&&state.kind in listOf("file","missing"))}
 val files=linkedMapOf<String,ResourcePublishedFile<File>>()
 for(entry in receipt.files){val (path,authored)=address(entry.file);val state=saved!![path];admit(path in paths&&path !in files&&authored&&state?.kind=="file"&&state==entry.saved&&state.text!=null&&(entry.saveGuard==null||entry.saveGuard.isNotEmpty()));files[path]=entry}
 admit(files.size==patches.count{it.after!=null});return files
}
