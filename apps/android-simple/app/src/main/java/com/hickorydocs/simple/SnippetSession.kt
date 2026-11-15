package com.hickorydocs.simple
data class SnippetSession(val source:String,val fields:List<SnippetField>,val active:Int,val current:Int)
data class SnippetInput(val session:SnippetSession,val edits:List<NativeSourceEdit>,val selection:CompletionSelection)
data class SnippetAdvance(val session:SnippetSession?,val selection:CompletionSelection)
private fun snippetScalar(text:String,at:Int):Boolean=at in 0..text.length&&!(at>0&&at<text.length&&(Character.isHighSurrogate(text[at-1])&&Character.isLowSurrogate(text[at])||text[at-1]=='\r'&&text[at]=='\n'))
fun beginSnippet(source:String,fields:List<SnippetField>):SnippetSession? {
 if(fields.isEmpty()||fields.size>4096||fields.any{it.id==null||it.id !in 0..4095||it.index !in 0..1004096||it.from>it.to||!snippetScalar(source,it.from)||!snippetScalar(source,it.to)}||fields.count{it.index==0}!=1||fields.any{it.index==0&&it.from!=it.to})return null
 val ids=fields.associateBy{it.id!!};if(ids.size!=fields.size)return null;val visited=mutableSetOf<Int>()
 for(field in fields){val chain=mutableSetOf<Int>();var next:SnippetField?=field
  while(next!=null&&!visited.contains(next.id!!)){val value=next;if(!chain.add(value.id!!))return null;val identity=value.parent ?: break;val parent=ids[identity] ?: return null;if(parent.from>value.from||parent.to<value.to)return null;next=parent};visited.addAll(chain)
 }
 val active=fields.map{it.index}.filter{it>0}.minOrNull() ?: return null;return SnippetSession(source,fields.map{it.copy(choices=it.choices.toList())},active,fields.first{it.index==active}.id!!)
}
fun advanceSnippet(session:SnippetSession,source:String,from:Int,to:Int,direction:Int):SnippetAdvance? {
 if(source!=session.source||from>to||direction!=1&&direction != -1||session.fields.none{it.index==session.active&&it.from<=from&&it.to>=to})return null
 val order=session.fields.map{it.index}.filter{it>0}.distinct().sorted()+0;val at=order.indexOf(session.active);val next=at+direction;if(at<0||next !in order.indices)return null
 val field=session.fields.first{it.index==order[next]};return SnippetAdvance(if(field.index==0)null else session.copy(active=field.index,current=field.id!!),CompletionSelection(field.from,field.to))
}
private data class SnippetReplacement(val from:Int,val to:Int,var text:String,val owners:MutableList<Int>)
fun editSnippet(session:SnippetSession,source:String,from:Int,to:Int,text:String):SnippetInput? {
 if(source!=session.source||from>to||!snippetScalar(source,from)||!snippetScalar(source,to)||session.active==0)return null
 val fields=session.fields;val ids=fields.associateBy{it.id!!}
 fun ancestor(parent:Int,child:Int):Boolean{var current=ids[child];repeat(fields.size+1){val value=current ?: return false;if(value.id==parent)return true;current=value.parent?.let{ids[it]}};return false}
 val occurrences=fields.filter{it.index==session.active};val matches=occurrences.filter{it.from<=from&&it.to>=to};val origin=matches.firstOrNull{it.id==session.current} ?: matches.firstOrNull() ?: return null
 if(text.length>1048576/maxOf(occurrences.size,1))return null
 val before=source.substring(origin.from,origin.to);val relativeFrom=from-origin.from;val relativeTo=to-origin.from
 if(occurrences.any{source.substring(it.from,it.to)!=before})return null
 val edits=mutableListOf<SnippetReplacement>()
 for(field in occurrences){val from=field.from+relativeFrom;val to=field.from+relativeTo;if(!snippetScalar(source,from)||!snippetScalar(source,to))return null
  val same=edits.firstOrNull{it.from==from&&it.to==to};if(same!=null){if(from!=to)return null;same.text+=text;same.owners.add(field.id!!)}else edits.add(SnippetReplacement(from,to,text,mutableListOf(field.id!!)))
 }
 edits.sortWith(compareBy({it.from},{it.to}));for(index in 1 until edits.size)if(edits[index-1].to>edits[index].from)return null
 var after=source;for(edit in edits.asReversed())after=after.replaceRange(edit.from,edit.to,edit.text)
 val native=edits.map{NativeSourceEdit(it.from,it.to,it.text)}
 if(after==source)return SnippetInput(session.copy(current=origin.id!!),native,CompletionSelection(from+text.length,from+text.length))
 val ordinals=fields.mapIndexed{index,field->field.id!! to index}.toMap()
 fun map(field:SnippetField,position:Int,end:Boolean):Int {
  var delta=0;for(edit in edits){if(position<edit.from)break;if(position>edit.to){delta+=edit.text.length-(edit.to-edit.from);continue}
   if(edit.from==edit.to){val related=edit.owners.indices.filter{ancestor(field.id!!,edit.owners[it])};if(related.isNotEmpty())return edit.from+delta+text.length*(if(end)related.last()+1 else related.first())
    val enclosing=edit.owners.indices.filter{field.index==0&&ancestor(edit.owners[it],field.id!!)};if(enclosing.isNotEmpty())return edit.from+delta+text.length*(enclosing.last()+1)
    return edit.from+delta+text.length*edit.owners.count{ordinals.getValue(it)<ordinals.getValue(field.id!!)}
   }
   val owns=edit.owners.contains(field.id!!)||edit.owners.any{ancestor(field.id!!,it)};val exit=field.index==0&&edit.owners.any{ancestor(it,field.id!!)};val right=if(owns)end else if(exit)true else position==edit.to
   return edit.from+delta+if(right)edit.text.length else 0
  };return position+delta
 }
 val retained=fields.filter{field->field.index==0||field.index==session.active||occurrences.none{ancestor(it.id!!,field.id!!)}};val retainedIds=retained.map{it.id!!}.toSet()
 val updated=retained.map{field->var parent=field.parent;while(parent!=null&&!retainedIds.contains(parent))parent=ids[parent]?.parent;field.copy(parent=parent,from=map(field,field.from,false),to=map(field,field.to,true))}
 val groups=mutableMapOf<Int,String>();for(field in updated){if(field.from>field.to||!snippetScalar(after,field.from)||!snippetScalar(after,field.to))return null;val value=after.substring(field.from,field.to);val previous=groups[field.index];if(previous!=null&&previous!=value)return null;groups[field.index]=value}
 val caret=map(origin,from,false)+text.length;return SnippetInput(SnippetSession(after,updated,session.active,origin.id!!),native,CompletionSelection(caret,caret))
}
