package com.hickorydocs.simple
data class SnippetScope(val order:List<Int>,val exitIndex:Int,val exitId:Int,val outerActive:Int,val outerCurrent:Int)
data class SnippetSession(val source:String,val fields:List<SnippetField>,val active:Int,val current:Int,val scopes:List<SnippetScope> = emptyList())
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
 val scope=session.scopes.lastOrNull();val order=scope?.let{it.order+it.exitIndex} ?: (session.fields.map{it.index}.filter{it>0}.distinct().sorted()+0);val at=order.indexOf(session.active);val next=at+direction;if(at<0||next !in order.indices)return null
 if(scope!=null&&order[next]==scope.exitIndex){val field=session.fields.firstOrNull{it.id==scope.exitId} ?: return null;return SnippetAdvance(session.copy(fields=session.fields.filter{it.index !in scope.order&&it.index!=scope.exitIndex},active=scope.outerActive,current=scope.outerCurrent,scopes=session.scopes.dropLast(1)),CompletionSelection(field.from,field.to))}
 val field=session.fields.first{it.index==order[next]};return SnippetAdvance(if(field.index==0)null else session.copy(active=field.index,current=field.id!!),CompletionSelection(field.from,field.to))
}

private data class SnippetReplacement(val from:Int,val to:Int,var text:String,val owners:MutableList<Int>)
fun editSnippet(session:SnippetSession,source:String,from:Int,to:Int,text:String):SnippetInput? {
 if(source!=session.source||from>to||!snippetScalar(source,from)||!snippetScalar(source,to)||session.active==0)return null
 fun terminal(field:SnippetField)=field.index==0||session.scopes.any{it.exitIndex==field.index}
 val fields=session.fields;val ids=fields.associateBy{it.id!!}
 fun ancestor(parent:Int,child:Int):Boolean{var current=ids[child];repeat(fields.size+1){val value=current ?: return false;if(value.id==parent)return true;current=value.parent?.let{ids[it]}};return false}
 val occurrences=fields.filter{it.index==session.active};val matches=occurrences.filter{it.from<=from&&it.to>=to};val origin=matches.firstOrNull{it.id==session.current} ?: matches.firstOrNull() ?: return null
 if(text.toByteArray(Charsets.UTF_8).size>1048576/maxOf(occurrences.size,1))return null
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
    val enclosing=edit.owners.indices.filter{terminal(field)&&ancestor(edit.owners[it],field.id!!)};if(enclosing.isNotEmpty())return edit.from+delta+text.length*(enclosing.last()+1)
    return edit.from+delta+text.length*edit.owners.count{ordinals.getValue(it)<ordinals.getValue(field.id!!)}
   }
   val owns=edit.owners.contains(field.id!!)||edit.owners.any{ancestor(field.id!!,it)};val exit=terminal(field)&&edit.owners.any{ancestor(it,field.id!!)};val right=if(owns)end else if(exit)true else position==edit.to
   return edit.from+delta+if(right)edit.text.length else 0
  };return position+delta
 }
 val retained=fields.filter{field->terminal(field)||field.index==session.active||occurrences.none{ancestor(it.id!!,field.id!!)}};val retainedIds=retained.map{it.id!!}.toSet()
 val updated=retained.map{field->var parent=field.parent;while(parent!=null&&!retainedIds.contains(parent))parent=ids[parent]?.parent;field.copy(parent=parent,from=map(field,field.from,false),to=map(field,field.to,true))}
 val groups=mutableMapOf<Int,String>();for(field in updated){if(field.from>field.to||!snippetScalar(after,field.from)||!snippetScalar(after,field.to))return null;val value=after.substring(field.from,field.to);val previous=groups[field.index];if(previous!=null&&previous!=value)return null;groups[field.index]=value}
 val caret=map(origin,from,false)+text.length;return SnippetInput(SnippetSession(after,updated,session.active,origin.id!!,session.scopes),native,CompletionSelection(caret,caret))
}
fun completeSnippet(session:SnippetSession,source:String,primary:SourceReplacement,additional:List<SourceReplacement>):SnippetInput? {
 val input=editSnippet(session,source,primary.from,primary.to,primary.text) ?: return null
 if(additional.isEmpty())return input
 val start=session.fields.minOf{it.from};val end=session.fields.maxOf{it.to};var shift=0;var bytes=0L
 val edits=input.edits.map{SourceReplacement(it.from,it.to,it.text)}.toMutableList()
 for(edit in additional.asReversed()){
  if(edit.from>edit.to||!snippetScalar(source,edit.from)||!snippetScalar(source,edit.to))return null
  bytes+=edit.text.toByteArray(Charsets.UTF_8).size;if(bytes>1048576)return null
  if(edit.to<=start)shift+=edit.text.length-(edit.to-edit.from) else if(edit.from<end)return null
  val index=edits.indexOfFirst{it.from==edit.from}
  if(index>=0){if(edit.from!=edit.to||edit.to>start)return null;edits[index]=edits[index].copy(text=edit.text+edits[index].text)}else edits.add(edit)
 }
 edits.sortWith(compareBy({it.from},{it.to}));for(index in 1 until edits.size)if(edits[index-1].to>edits[index].from||edits[index-1].from==edits[index].from)return null
 var after=source;for(edit in edits.asReversed())after=after.replaceRange(edit.from,edit.to,edit.text)
 val fields=input.session.fields.map{it.copy(from=it.from+shift,to=it.to+shift)};if(beginSnippet(after,fields)==null)return null
 return SnippetInput(input.session.copy(source=after,fields=fields),edits.map{NativeSourceEdit(it.from,it.to,it.text)},CompletionSelection(input.selection.from+shift,input.selection.to+shift))
}
fun nestSnippet(session:SnippetSession,source:String,primary:SourceReplacement,additional:List<SourceReplacement>,inner:List<SnippetField>):SnippetInput? {
 if(session.scopes.size>=32||inner.isEmpty())return null
 val exit=inner.firstOrNull{it.index==0} ?: return null;val parsed=beginSnippet(primary.text,inner);if(parsed==null&&(inner.size!=1||exit.id==null||exit.parent!=null||exit.from!=exit.to||!snippetScalar(primary.text,exit.from)))return null
 val origin=session.fields.firstOrNull{it.id==session.current&&it.from<=primary.from&&it.to>=primary.to} ?: return null
 val input=completeSnippet(session,source,primary,additional) ?: return null;val parents=input.session.fields.filter{it.index==session.active};val current=parents.firstOrNull{it.id==input.session.current} ?: return null
 val relative=primary.from-origin.from;val order=inner.filter{it.index>0}.map{it.index}.distinct().sorted()
 if(order.isEmpty()){val caret=current.from+relative+exit.from;return input.copy(selection=CompletionSelection(caret,caret))}
 if(input.session.fields.size.toLong()+parents.size.toLong()*inner.size>4096)return null
 var nextIndex=input.session.fields.maxOf{it.index}+1;val indices=(order+0).associateWith{nextIndex++};if(nextIndex>1004097)return null
 val used=input.session.fields.map{it.id!!}.toMutableSet();val fields=input.session.fields.toMutableList();var nextId=0;var exitId = -1;var currentId = -1
 for(parent in parents){val ids=mutableMapOf<Int,Int>();for(field in inner){while(nextId in used)nextId++;if(nextId>=4096)return null;ids[field.id!!]=nextId;used.add(nextId++)}
  for(field in inner){val id=ids.getValue(field.id!!);fields.add(field.copy(id=id,index=indices.getValue(field.index),parent=field.parent?.let{ids.getValue(it)} ?: parent.id,from=parent.from+relative+field.from,to=parent.from+relative+field.to));if(parent.id==current.id){if(field.index==0)exitId=id else if(field.id==parsed!!.current)currentId=id}}
 }
 if(fields.sumOf{field->64L+field.choices.sumOf{it.toByteArray(Charsets.UTF_8).size.toLong()}}>1048576||beginSnippet(input.session.source,fields)==null)return null
 val selected=fields.firstOrNull{it.id==currentId} ?: return null;val scope=SnippetScope(order.map{indices.getValue(it)},indices.getValue(0),exitId,session.active,input.session.current)
 return input.copy(session=input.session.copy(fields=fields,active=scope.order[0],current=currentId,scopes=input.session.scopes+scope),selection=CompletionSelection(selected.from,selected.to))
}
