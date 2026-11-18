package com.hickorydocs.simple
import org.json.JSONObject
data class SnippetField(val index:Int,val from:Int,val to:Int,val choices:List<String>,val id:Int?=null,val parent:Int?=null,val transform:SnippetTransform?=null)
fun completionExpansion(item:JSONObject):CompletionExpansion? = completionExpansion(item,SnippetTransformBudget())
internal fun completionExpansion(item:JSONObject,budget:SnippetTransformBudget):CompletionExpansion? {
 if((item.opt("insertTextFormat") as? Number)?.toDouble()!=2.0)return null
 val raw=item.optJSONObject("data")?.optJSONObject("hickory_snippet") ?: error("Snippet expansion unavailable or out of date. Request completions again.")
 fun number(value:Any?):Int {check(value is Number&&value.toDouble().isFinite()&&value.toDouble()==value.toInt().toDouble()&&value.toInt()>=0){"Malformed snippet field"};return value.toInt()}
 val version=number(raw.opt("version"));check(version in listOf(1,2)&&raw.opt("syntax")==completionInsertion(item)){"Snippet expansion unavailable or out of date. Request completions again."}
 val text=raw.opt("text") as? String ?: error("Malformed snippet text");check(text.toByteArray(Charsets.UTF_8).size<=1048576){"Snippet expansion limit exceeded"}
 val values=raw.optJSONArray("fields") ?: error("Malformed snippet fields");check(values.length() in 1..4096){"Malformed snippet fields"}
 var metadataBytes=0L
 val fields=(0 until values.length()).map {offset->
  val value=values.optJSONObject(offset) ?: error("Malformed snippet field");val index=number(value.opt("index"));val from=number(value.opt("from"));val to=number(value.opt("to"));check(index<=1004096&&from<=to&&to<=text.length){"Malformed snippet field"}
  val choices=value.optJSONArray("choices") ?: error("Malformed snippet choices")
  check(choices.length()<=4096){"Snippet choice count limit exceeded"}
  check(64L+32L*choices.length()<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"}
  metadataBytes+=64L+32L*choices.length()
  for(choice in 0 until choices.length()){val option=choices.opt(choice) as? String ?: error("Malformed snippet choices");check(option.length.toLong()<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"};val bytes=option.toByteArray(Charsets.UTF_8).size.toLong();check(bytes<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"};metadataBytes+=bytes}
  // Materialize a separate retained vector only after the whole field's quota
  // has been admitted, including empty alternatives and their slots.
  val options=(0 until choices.length()).map{choices.getString(it)}
  sourceChangeOffset(text,languagePosition(text,from));sourceChangeOffset(text,languagePosition(text,to))
  val id=if(value.has("id"))number(value.opt("id")) else null;val parent=if(value.has("id")&&!value.isNull("parent"))number(value.opt("parent")) else null
  check((id==null&&!value.has("parent")||id!=null&&id<4096&&value.has("parent"))&&(parent==null||parent<4096)){"Malformed snippet ancestry"}
  val transform=if(value.has("transform")&&!value.isNull("transform")){check(version==2&&index>0&&options.isEmpty()&&id!=null){"Computed snippet metadata requires protocol 2 and an editable source group"};readSnippetTransform(value.opt("transform"))}else null
  if(transform!=null){check(transform.bytes.toLong()<=1048576L-metadataBytes){"Snippet metadata expansion limit exceeded"};metadataBytes+=transform.bytes}
  SnippetField(index,from,to,options,id,parent,transform)
 }
 if(fields.any{it.id!=null}){
  val ids=fields.associateBy{it.id};check(!ids.containsKey(null)&&ids.size==fields.size){"Malformed snippet ancestry"};val visited=mutableSetOf<Int>()
  for(field in fields){val chain=mutableSetOf<Int>();var next:SnippetField?=field
   while(next!=null&&!visited.contains(next.id!!)){val current=next;check(chain.add(current.id!!)){"Cyclic snippet ancestry"};val identity=current.parent ?: break;val parent=ids[identity] ?: error("Malformed snippet ancestry");check(parent.from<=current.from&&parent.to>=current.to){"Malformed snippet ancestry"};next=parent};visited.addAll(chain)
  }
 }
 check(fields.count{it.index==0}==1){"Malformed snippet exit"}
 validateSnippetMirrors(text,fields,budget)
 val active=fields.filter{it.transform==null&&it.index>0}.map{it.index}.minOrNull() ?: 0;val selected=fields.first{it.transform==null&&it.index==active}
 return CompletionExpansion(text,CompletionSelection(selected.from,selected.to),fields)
}

fun snippetTransformOrder(fields:List<SnippetField>):List<Int> {
 val ids=fields.filter{it.id!=null}.associateBy{it.id!!};val ordinary=fields.filter{it.transform==null}.map{it.index}.toSet();val dependencies=ordinary.associateWith{mutableSetOf<Int>()};var work=0;var edges=0
 for(field in fields.filter{it.transform!=null}){
  check(field.index>0&&field.id!=null&&field.choices.isEmpty()&&field.index in ordinary&&fields.none{it.parent==field.id}){"Malformed computed snippet occurrence"}
  val seen=mutableSetOf<Int>();var next=field.parent?.let{ids[it]}
  while(next!=null){val value=next;work++;check(work<=65536&&value.id!=null&&seen.add(value.id)){"Computed snippet ancestry limit exceeded"};if(value.index>0){check(value.index!=field.index){"Cyclic computed snippet dependency"};val values=dependencies[value.index] ?: error("Computed snippet parent is not editable");if(values.add(field.index)){edges++;check(edges<=4096){"Computed snippet dependency limit exceeded"}}};next=value.parent?.let{ids[it]}}
 }
 val parents=mutableMapOf<Int,MutableList<Int>>();for(parent in dependencies.keys.sorted())for(child in dependencies.getValue(parent).sorted())parents.getOrPut(child){mutableListOf()}.add(parent)
 val queue=dependencies.keys.filter{dependencies.getValue(it).isEmpty()}.sorted().toMutableList();val order=mutableListOf<Int>();var at=0
 while(at<queue.size){val child=queue[at++];order+=child;for(parent in parents[child].orEmpty()){val values=dependencies.getValue(parent);values.remove(child);if(values.isEmpty())queue+=parent}}
 check(order.size==ordinary.size){"Cyclic computed snippet dependency"};return order
}
fun validateSnippetMirrors(text:String,fields:List<SnippetField>):List<Int> = validateSnippetMirrors(text,fields,SnippetTransformBudget())
internal fun validateSnippetMirrors(text:String,fields:List<SnippetField>,budget:SnippetTransformBudget):List<Int> {
 val first=mutableMapOf<Int,SnippetField>();for(field in fields.filter{it.transform==null}){val previous=first[field.index];check(previous==null||text.substring(previous.from,previous.to)==text.substring(field.from,field.to)&&previous.choices==field.choices){"Inconsistent snippet mirrors"};first[field.index]=field}
 val order=snippetTransformOrder(fields);for(field in fields.filter{it.transform!=null}){val value=first[field.index] ?: error("Computed snippet source is unavailable");check(applySnippetTransform(field.transform!!,text.substring(value.from,value.to),budget)==text.substring(field.from,field.to)){"Inconsistent computed snippet occurrence"}};return order
}
