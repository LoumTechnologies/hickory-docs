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

internal data class CapturedAndroidCompletion(val item:JSONObject,val from:Int,val to:Int,val prefixFrom:Int?)
internal data class AndroidCompletionGroup(val plan:ChangePlan,val carets:List<Int>,val snippet:ParallelSnippet?=null)

internal fun groupCompletionFormat(item:JSONObject):Int {
    val number=(if(!item.has("insertTextFormat"))1 else item.opt("insertTextFormat")) as? Number
    return when(number?.toDouble()){
        1.0 -> 1
        2.0 -> 2
        else -> error("Unsupported grouped completion format.")
    }
}

internal data class AndroidCompletionRole(val syntax:String,val from:Int,val to:Int,val additions:List<SourceReplacement>)

private data class ComposedAndroidCompletion(val group:AndroidCompletionGroup?,val roles:List<AndroidCompletionRole>)

internal fun preflightCompletionGroup(uri:String,snapshot:SourceSnapshot,captures:List<CapturedAndroidCompletion>,
                                      waitingForClipboard:Set<Int>):List<AndroidCompletionRole> =
    composeCompletionGroup(uri,snapshot,captures,clipboardPreflight=waitingForClipboard).roles

internal fun planCompletionGroup(uri:String,snapshot:SourceSnapshot,captures:List<CapturedAndroidCompletion>,
                                 preserving:List<AndroidCompletionRole>?=null):AndroidCompletionGroup =
    checkNotNull(composeCompletionGroup(uri,snapshot,captures,preserving=preserving).group)

private fun composeCompletionGroup(uri:String,snapshot:SourceSnapshot,captures:List<CapturedAndroidCompletion>,
                                   clipboardPreflight:Set<Int>?=null,preserving:List<AndroidCompletionRole>?=null):ComposedAndroidCompletion {
    require(captures.size in 2..16) { "Select 2 to 16 carets for a completion group." }
    val source = snapshot.text
    if (captures.any { !it.item.has("textEdit") }) {
        val prefixes = captures.map { (_,from,to,prefixFrom) ->
            require(from == to && prefixFrom != null && prefixFrom in 0..to && to <= source.length) {
                "Implicit completion ranges require the same identifier prefix at each caret."
            }
            source.substring(prefixFrom,to)
        }
        require(prefixes.distinct().size == 1) { "Implicit completion ranges require the same identifier prefix at each caret." }
    }
    val format=captures.map { groupCompletionFormat(it.item) }.distinct().singleOrNull()
        ?: error("The completions mix ordinary edits and templates.")
    require(format==2||captures.none {it.item.opt("kind")==15}){"A template did not return a snippet expansion."}
    val snippet=format==2
    val budget=SnippetTransformBudget()
    val expansions=captures.mapIndexed { index,capture ->
        when {
            !snippet -> null
            clipboardPreflight?.contains(index)==true -> CompletionExpansion("")
            else -> checkNotNull(completionExpansion(capture.item,budget))
        }
    }
    val drafts = captures.mapIndexed { index,(item,from,to,prefixFrom) ->
        planCompletion(uri,item,snapshot,prefixFrom ?: from,to,expansions[index]).also { draft ->
            val edit = checkNotNull(draft.primary)
            require(edit.from <= from && edit.to >= to) { "A completion range does not cover its captured selection." }
        }
    }
    val first = drafts.first()
    val primary = drafts.map { checkNotNull(it.primary) }
    val roles=drafts.mapIndexed { index,draft ->
        AndroidCompletionRole(completionInsertion(captures[index].item),primary[index].from,primary[index].to,draft.additional)
    }
    require(preserving==null||roles==preserving){
        "Completion edits changed after requesting context. Request completions again."
    }
    require(drafts.all { (snippet || it.primary?.text == primary.first().text) && it.additional == first.additional }) {
        "The completions disagree about their replacement or shared edits."
    }
    val byStart = primary.withIndex().associate { it.value.from to it.index }
    require(byStart.size == primary.size) { "Completion ranges overlap." }
    val prefixes = first.additional.filter { it.from == it.to && it.from in byStart }.groupBy { it.from }
    val edits = (first.additional.filter { it.from != it.to || it.from !in byStart } + primary.map { edit ->
        edit.copy(text = prefixes[edit.from].orEmpty().joinToString("") { it.text } + edit.text)
    }).sortedWith(compareBy({it.from},{it.to}))
    require(edits.zipWithNext().none { (left,right) -> left.from == right.from || left.to > right.from }) {
        "Completion ranges overlap with another completion or shared edit."
    }
    val raw = JSONArray()
    val insertionStarts=IntArray(primary.size)
    val carets = IntArray(primary.size)
    var delta = 0
    for (edit in edits) {
        raw.put(JSONObject().put("range",JSONObject().put("start",languagePosition(source,edit.from))
            .put("end",languagePosition(source,edit.to))).put("newText",edit.text))
        byStart[edit.from]?.let { index ->
            carets[index]=edit.from+delta+edit.text.length
            insertionStarts[index]=carets[index]-primary[index].text.length
        }
        delta += edit.text.length - (edit.to - edit.from)
    }

    val plan = planWorkspaceEdit("Complete ${captures.first().item.getString("label")}",
        JSONObject().put("changes",JSONObject().put(uri,raw)),mapOf(first.path to snapshot),allowNoop=true)
    if(clipboardPreflight!=null)return ComposedAndroidCompletion(null,roles)
    if(!snippet){
        require(carets.distinct().size==primary.size){"The completion would merge distinct carets."}
        return ComposedAndroidCompletion(AndroidCompletionGroup(plan,carets.toList()),roles)
    }
    val peers=primary.mapIndexed { index,edit ->
        ParallelSnippetExpansion(NativeSourceEdit(edit.from,edit.to,edit.text),checkNotNull(expansions[index]).fields)
    }
    val after=plan.patches.singleOrNull()?.after ?: source
    val group=ParallelSnippet.begin(source,peers,0)?.state?.placing(after,insertionStarts.toList())
        ?: error("The parallel snippet fields overlap or exceed their bounds.")
    require(group.selections.distinct().size==primary.size){"The completion would merge distinct template fields."}
    return ComposedAndroidCompletion(AndroidCompletionGroup(plan,carets.toList(),group),roles)
}
