package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
data class CompletionSelection(val from:Int,val to:Int)
data class CompletionExpansion(val text:String,val selection:CompletionSelection?=null,val fields:List<SnippetField> = emptyList())
data class CompletionDraft(val path:String,val before:String,val plan:ChangePlan,val insertionFrom:Int,val selection:CompletionSelection,val fields:List<SnippetField> = emptyList(),val continuing:SnippetSession?=null,val primary:SourceReplacement?=null,val additional:List<SourceReplacement> = emptyList())
fun completionInsertion(item:JSONObject):String {
 val label=item.opt("label") as? String ?: error("Malformed completion")
 if(item.has("textEdit"))return item.optJSONObject("textEdit")?.opt("newText") as? String ?: error("Missing completion insertion text")
 if(item.has("insertText"))return item.opt("insertText") as? String ?: error("Missing completion insertion text")
 return label
}
fun planCompletion(uri:String,item:JSONObject,snapshot:SourceSnapshot,initialFrom:Int,initialTo:Int,expansion:CompletionExpansion?=null):CompletionDraft {
 val source=snapshot.text;check(snapshot.writable){"Completion source is generated, read-only or saving"}
 val path=SemanticLocation(uri,0,0).path ?: error("Unsupported completion source identity")
 check(item.isNull("command")){"This completion requires a provider command; no edits were applied."}
 val format=if(item.has("insertTextFormat")){val value=item.opt("insertTextFormat") as? Number ?: error("Unsupported completion insertion format");check(value.toDouble()==value.toInt().toDouble()){ "Unsupported completion insertion format" };value.toInt()}else 1
 check(format==1||format==2){"Unsupported completion insertion format"}
 val rawText=completionInsertion(item);check(format!=2||expansion!=null){"Snippet expansion unavailable; no edits were applied."}
 val inserted=if(format==2)expansion!! else CompletionExpansion(rawText)
 fun validateOffset(text:String,offset:Int){check(offset in 0..text.length){"Invalid completion selection"};sourceChangeOffset(text,languagePosition(text,offset))}
 validateOffset(source,initialFrom);validateOffset(source,initialTo);check(initialFrom<=initialTo){"Completion fallback runs backwards"}
 var from=initialFrom;var to=initialTo
 item.optJSONObject("textEdit")?.let {edit->
  check(!edit.has("range")||!edit.has("insert")&&!edit.has("replace")){"Ambiguous completion edit"}
  val range=edit.optJSONObject("range") ?: edit.optJSONObject("replace") ?: error("Missing completion range")
  from=sourceChangeOffset(source,range.opt("start"));to=sourceChangeOffset(source,range.opt("end"));check(from<=to){"Completion range runs backwards"}
 }
 val additions=mutableListOf<SourceReplacement>()
 if(item.has("additionalTextEdits")) {
  val items=item.optJSONArray("additionalTextEdits") ?: error("Malformed additional completion edits")
  for(index in 0 until items.length()) {
   val edit=items.optJSONObject(index) ?: error("Malformed additional completion edit");val range=edit.optJSONObject("range") ?: error("Malformed additional completion edit");val text=edit.opt("newText") as? String ?: error("Malformed additional completion edit")
   val change=SourceReplacement(sourceChangeOffset(source,range.opt("start")),sourceChangeOffset(source,range.opt("end")),text);check(change.from<=change.to){"Additional completion edit runs backwards"};if(change !in additions)additions.add(change)
  }
 }
 val prefix=additions.filter{it.from==from&&it.to==from}.joinToString(""){it.text}
 val others=additions.filter{it.from!=from||it.to!=from};val primary=SourceReplacement(from,to,prefix+inserted.text)
 val edits=JSONArray();(listOf(primary)+others).forEach {edit->edits.put(JSONObject().put("range",JSONObject().put("start",languagePosition(source,edit.from)).put("end",languagePosition(source,edit.to))).put("newText",edit.text))}
 val plan=planWorkspaceEdit("Complete ${item.optString("label")}",JSONObject().put("changes",JSONObject().put(uri,edits)),mapOf(path to snapshot),allowNoop=true)
 val delta=others.filter{it.from<from}.sumOf{it.text.length-(it.to-it.from)};val insertionFrom=from+delta+prefix.length
 val selection=inserted.selection ?: CompletionSelection(inserted.text.length,inserted.text.length);validateOffset(inserted.text,selection.from);validateOffset(inserted.text,selection.to);check(selection.from<=selection.to){"Completion selection runs backwards"}
 val fields=inserted.fields.map {field->validateOffset(inserted.text,field.from);validateOffset(inserted.text,field.to);check(field.from<=field.to){"Invalid snippet field"};field.copy(from=insertionFrom+field.from,to=insertionFrom+field.to)}
 return CompletionDraft(path,source,plan,insertionFrom,CompletionSelection(insertionFrom+selection.from,insertionFrom+selection.to),fields,primary=SourceReplacement(from,to,inserted.text),additional=additions)
}
