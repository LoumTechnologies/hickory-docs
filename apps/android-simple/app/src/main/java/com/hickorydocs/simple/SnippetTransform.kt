@file:Suppress("UNCHECKED_CAST")
package com.hickorydocs.simple
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale
@ConsistentCopyVisibility
data class SnippetTransform internal constructor(val regex:List<Any?>,val format:List<Any?>,val flags:String,val groups:Int,val bytes:Int)
internal data class SnippetTransformBudget(var work:Int=0)
private fun scalarText(value:String):Boolean {var at=0;while(at<value.length){val c=value[at++];if(c.isHighSurrogate()){if(at==value.length||!value[at++].isLowSurrogate())return false}else if(c.isLowSurrogate())return false};return true}
internal fun readSnippetTransform(raw:Any?):SnippetTransform {
 var count=0;var bytes=0
 fun tick(n:Int=1){count+=n;check(count<=512){"Transform program limit exceeded"}}
 fun integer(raw:Any?,max:Int=1114111):Int {val value=(raw as? Number)?.toString()?.toLongOrNull();check(value!=null&&value in 0..max.toLong()){"Malformed transform integer"};return value.toInt()}
 fun text(raw:Any?):String {check(raw is String&&scalarText(raw)){"Malformed transform text"};bytes+=raw.toByteArray(Charsets.UTF_8).size;check(bytes<=16384){"Transform metadata limit exceeded"};return raw}
 fun bool(raw:Any?):Boolean {check(raw is Boolean){"Malformed transform Boolean"};return raw}
 check(raw is JSONObject&&integer(raw.opt("version"))==2&&raw.opt("case_map")==snippetCaseVersion){"Unsupported snippet transform version"};val groups=integer(raw.opt("groups"),32);val flags=text(raw.opt("flags"));check(flags.all{it in "gimsu"}&&flags.toSet().size==flags.length){"Unsupported transform flags"}
 fun node(raw:Any?,depth:Int):List<Any?> {tick();check(depth<=64&&raw is JSONArray&&raw.opt(0) is String){"Malformed transform node"};val v=raw as JSONArray;val op=v.getString(0)
  return when(op){
   "seq","alt"->{val children=v.optJSONArray(1);check(v.length()==2&&children!=null&&children.length()<=512){"Malformed transform branches"};listOf(op,(0 until children!!.length()).map{node(children.get(it),depth+1)})}
   "capture"->{check(v.length()==3){"Malformed transform capture"};listOf(op,integer(v.opt(1),groups),node(v.opt(2),depth+1))}
   "repeat"->{check(v.length()==5){"Malformed transform repetition"};val min=integer(v.opt(2),256);val max=if(v.isNull(3))null else integer(v.opt(3),256);check(max==null||max>=min){"Backwards transform repetition"};listOf(op,node(v.opt(1),depth+1),min,max,bool(v.opt(4)))}
   "class"->{val ranges=v.optJSONArray(1);check(v.length()==3&&ranges!=null){"Malformed transform class"};val copied=(0 until ranges!!.length()).map{tick();val range=ranges.optJSONArray(it);check(range!=null&&range.length()==2){"Malformed transform range"};val low=integer(range!!.opt(0));val high=integer(range.opt(1));check(low<=high){"Backwards transform range"};listOf(low,high)};listOf(op,copied,bool(v.opt(2)))}
   "dot"->{check(v.length()==1){"Malformed transform dot"};listOf(op)}
   "assert"->{check(v.length()==2&&v.opt(1) in listOf("start","end","word","nonword")){"Malformed transform assertion"};listOf(op,v.getString(1))}
   else->error("Unsupported transform operation")
  }
 }
 fun format(raw:Any?,depth:Int):List<Any?> {check(depth<=32&&raw is JSONArray&&raw.length()<=512){"Malformed transform format"};return (0 until raw.length()).map{index->tick();val value=raw.get(index);if(value is String)text(value) else {check(value is JSONArray){"Malformed transform format"};when{value.opt(0)=="capture"&&value.length()==3&&value.opt(2) in listOf("","upcase","downcase","capitalize","camelcase","pascalcase")->listOf("capture",integer(value.opt(1),32),value.getString(2));value.opt(0)=="if"&&value.length()==4->listOf("if",integer(value.opt(1),32),format(value.opt(2),depth+1),format(value.opt(3),depth+1));else->error("Unsupported transform format")}}}}
 val regex=node(raw.opt("regex"),0);val format=format(raw.opt("format"),0);val normalized=JSONObject(mapOf("version" to 2,"case_map" to snippetCaseVersion,"regex" to regex,"format" to format,"flags" to flags,"groups" to groups));return SnippetTransform(regex,format,flags,groups,normalized.toString().toByteArray(Charsets.UTF_8).size)
}
private data class TransformCapture(val from:Int,val to:Int)
private data class TransformMatch(val at:Int,val captures:List<TransformCapture?>)
private class TransformEvaluator(val text:IntArray,val flags:String,var work:Int) {
 fun tick(n:Int=1){check(n>=0&&work>=0&&work<=100000-n){"Transform work limit exceeded"};work+=n}
 fun newline(c:Int?)=c in listOf(10,13,0x2028,0x2029)
 fun word(c:Int?)=c!=null&&(c in 48..57||c in 65..90||c==95||c in 97..122||'i' in flags&&'u' in flags&&(c==383||c==8490))
 fun string(from:Int,to:Int)=buildString{for(i in from until to)appendCodePoint(text[i])}
 fun matches(node:List<Any?>,state:TransformMatch,depth:Int):List<TransformMatch>{tick();check(depth<=64){"Transform evaluation nesting limit exceeded"};val op=node[0] as String
  val result:List<TransformMatch> = when(op){
   "seq"->{var states=listOf(state);for(child in node[1] as List<List<Any?>>){val next=mutableListOf<TransformMatch>();for(value in states){next+=matches(child,value,depth+1);check(next.size<=1024){"Transform state limit exceeded"}};states=next;if(states.isEmpty())break};states}
   "alt"->{val states=mutableListOf<TransformMatch>();for(child in node[1] as List<List<Any?>>){states+=matches(child,state,depth+1);check(states.size<=1024){"Transform state limit exceeded"}};states}
   "capture"->matches(node[2] as List<Any?>,state,depth+1).map{value->val captures=value.captures.toMutableList();captures[node[1] as Int]=TransformCapture(state.at,value.at);value.copy(captures=captures.toList())}
   "repeat"->{val min=node[2] as Int;val max=minOf(node[3] as? Int ?: text.size+1,4096);val lazy=node[4] as Boolean;val levels=mutableListOf(listOf(state));var total=1;for(i in 0 until max){val next=mutableListOf<TransformMatch>();for(value in levels.last()){for(found in matches(node[1] as List<Any?>,value,depth+1)){if(found.at!=value.at||levels.size<=min)next+=found};check(next.size<=1024){"Transform state limit exceeded"}};if(next.isEmpty())break;levels+=next.toList();total+=next.size;check(total<=1024){"Transform state limit exceeded"}};val indexes=(min until levels.size).toList();(if(lazy)indexes else indexes.reversed()).flatMap{levels[it]}}
   "assert"->{val before=text.getOrNull(state.at-1);val next=text.getOrNull(state.at);val boundary=word(before)!=word(next);val yes=when(node[1]){"start"->state.at==0||'m' in flags&&newline(before);"end"->state.at==text.size||newline(next)&&('m' in flags||state.at+1==text.size||state.at+2==text.size&&next==13&&text.lastOrNull()==10);"word"->boundary;else->!boundary};if(yes)listOf(state) else emptyList()}
   "class","dot"->{val ch=text.getOrNull(state.at);if(ch==null)emptyList() else {val yes=if(op=="dot")'s' in flags||!newline(ch) else {val ranges=node[1] as List<List<Int>>;tick(ranges.size)
    fun contains(c:Int)=ranges.any{c in it[0]..it[1]}
    val found=contains(ch);found!=(node[2] as Boolean)};if(yes)listOf(state.copy(at=state.at+1)) else emptyList()}}
   else->error("Unsupported transform operation")
  };tick(result.size);return result
 }
 fun format(ops:List<Any?>,captures:List<TransformCapture?>,depth:Int):String{check(depth<=32){"Transform format nesting limit exceeded"};var output="";for(raw in ops){tick();val value=if(raw is String)raw else {val op=raw as List<Any?>;val range=captures.getOrNull(op[1] as Int);val captured=range?.let{string(it.from,it.to)}?:"";if(op[0]=="if")format(op[if(captured.isEmpty())3 else 2] as List<Any?>,captures,depth+1) else when(op[2]){
   "upcase"->snippetCase(captured,"upper")
   "downcase"->snippetCase(captured,"lower")
   "capitalize"->if(captured.isEmpty())"" else {val first=captured.codePointAt(0);snippetCase(String(Character.toChars(first)),"upper")+captured.substring(Character.charCount(first))}
   "camelcase","pascalcase"->{val words=snippetCaseWords(captured);if(words.isEmpty())captured else words.mapIndexed{index,word->val c=word.codePointAt(0);val first=String(Character.toChars(c));(if(index==0&&op[2]=="camelcase")snippetCase(first,"lower") else snippetCase(first,"upper"))+word.substring(Character.charCount(c))}.joinToString("")}
   else->captured
  }};check(value.toByteArray(Charsets.UTF_8).size<=1048576-output.toByteArray(Charsets.UTF_8).size){"Transform output limit exceeded"};output+=value};return output}
}
internal fun applySnippetTransform(program:SnippetTransform,input:String,budget:SnippetTransformBudget=SnippetTransformBudget()):String {
 check(scalarText(input)&&input.toByteArray(Charsets.UTF_8).size<=65536){"Transform input limit exceeded"};val evaluator=TransformEvaluator(input.codePoints().toArray(),program.flags,budget.work);var output="";var cursor=0;var search=0;var matched=false;val count=evaluator.text.size
 while(search<=count){var found:Pair<Int,TransformMatch>?=null;for(start in search..count){val initial=TransformMatch(start,List(program.groups+1){null});val state=evaluator.matches(program.regex,initial,0).firstOrNull();if(state!=null){val captures=state.captures.toMutableList();captures[0]=TransformCapture(start,state.at);found=start to state.copy(captures=captures.toList());break}};val selected=found?:break;matched=true;val (start,state)=selected;val prefix=evaluator.string(cursor,start);val replacement=evaluator.format(program.format,state.captures,0);check((output+prefix+replacement).toByteArray(Charsets.UTF_8).size<=1048576){"Transform output limit exceeded"};output+=prefix+replacement;cursor=state.at;if('g' !in program.flags)break;search=if(state.at==start)state.at+1 else state.at}
 if(!matched&&program.format.any{it is List<*>&&it[0]=="if"&&(it[3] as List<*>).isNotEmpty()}){val value=evaluator.format(program.format,List(program.groups+1){null},0);budget.work=evaluator.work;return value};val suffix=evaluator.string(cursor,count);check((output+suffix).toByteArray(Charsets.UTF_8).size<=1048576){"Transform output limit exceeded"};budget.work=evaluator.work;return output+suffix
}
