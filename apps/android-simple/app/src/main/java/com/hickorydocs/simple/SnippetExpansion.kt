package com.hickorydocs.simple
import org.json.JSONObject
data class SnippetField(val index:Int,val from:Int,val to:Int,val choices:List<String>)
fun completionExpansion(item:JSONObject):CompletionExpansion? {
 if((item.opt("insertTextFormat") as? Number)?.toDouble()!=2.0)return null
 val raw=item.optJSONObject("data")?.optJSONObject("hickory_snippet") ?: error("Snippet expansion unavailable or out of date. Request completions again.")
 fun number(value:Any?):Int {check(value is Number&&value.toDouble().isFinite()&&value.toDouble()==value.toInt().toDouble()&&value.toInt()>=0){"Malformed snippet field"};return value.toInt()}
 check(number(raw.opt("version"))==1&&raw.opt("syntax")==completionInsertion(item)){"Snippet expansion unavailable or out of date. Request completions again."}
 val text=raw.opt("text") as? String ?: error("Malformed snippet text");check(text.length<=1048576){"Snippet expansion limit exceeded"}
 val values=raw.optJSONArray("fields") ?: error("Malformed snippet fields");check(values.length() in 1..4096){"Malformed snippet fields"}
 val fields=(0 until values.length()).map {offset->
  val value=values.optJSONObject(offset) ?: error("Malformed snippet field");val index=number(value.opt("index"));val from=number(value.opt("from"));val to=number(value.opt("to"));check(index<=1004096&&from<=to&&to<=text.length){"Malformed snippet field"}
  val choices=value.optJSONArray("choices") ?: error("Malformed snippet choices");val options=(0 until choices.length()).map{choices.opt(it) as? String ?: error("Malformed snippet choices")}
  sourceChangeOffset(text,languagePosition(text,from));sourceChangeOffset(text,languagePosition(text,to));SnippetField(index,from,to,options)
 }
 check(fields.count{it.index==0}==1){"Malformed snippet exit"}
 val first=mutableMapOf<Int,SnippetField>();for(field in fields){val previous=first[field.index];check(previous==null||text.substring(previous.from,previous.to)==text.substring(field.from,field.to)&&previous.choices==field.choices){"Inconsistent snippet mirrors"};first[field.index]=field}
 val active=first.keys.filter{it>0}.minOrNull() ?: 0;val selected=fields.first{it.index==active}
 return CompletionExpansion(text,CompletionSelection(selected.from,selected.to),fields)
}
