package com.hickorydocs.simple

import org.json.JSONObject
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.verticalScroll
import androidx.compose.ui.unit.dp

private fun templateNumber(value:Any?):Int = (value as? Number)?.toString()?.toIntOrNull()?.takeIf{it>=0} ?: error("Malformed template coordinate or protocol version")
private fun templateText(value:JSONObject,key:String)=value.opt(key) as? String ?: error("Missing template $key")
private fun templateStrings(value:JSONObject,key:String):List<String> {val array=value.optJSONArray(key) ?: error("Missing template $key");return (0 until array.length()).map{array.opt(it) as? String ?: error("Malformed template $key")}}

class SourceTemplateContinuation internal constructor(internal val group:ParallelSnippet)

data class SourceTemplateCapture(
    val workspace:Any,val engine:Any,val editor:Any,val namespace:Any,val revision:Long,
    val path:String,val source:String,val anchor:Int,val head:Int,val snippetIntent:Int,
    val variables:SnippetVariableSnapshot?=null,
    val readingContext:String?=null,val contributorsCurrent:()->Boolean={true},val entryCurrent:()->Boolean={true},
    val carets:String?=null,val continuation:SourceTemplateContinuation?=null,val multiSelectionRevision:Long=0,
) {
    init {
        check(anchor in 0..source.length&&head in 0..source.length){"Template selection is outside the source."}
        sourceChangeOffset(source,languagePosition(source,anchor));sourceChangeOffset(source,languagePosition(source,head))
    }
    val from get()=minOf(anchor,head)
    val to get()=maxOf(anchor,head)
    val protocol get()=if(variables==null)1 else 2
    fun body()=JSONObject().put("version",protocol).put("path",path).put("source",source).put("selection",JSONObject().put("from",from).put("to",to)).apply{variables?.let{put("variable_context",it.body())};readingContext?.let{put("reading_context",JSONObject(it));put("transform_program_versions",org.json.JSONArray(listOf(2)))};carets?.let{put("carets",JSONObject(it))}}
    fun matches(now:SourceTemplateCapture)=workspace===now.workspace&&engine===now.engine&&editor===now.editor&&namespace==now.namespace&&revision==now.revision&&path==now.path&&source==now.source&&anchor==now.anchor&&head==now.head&&snippetIntent==now.snippetIntent&&carets==now.carets&&continuation?.group===now.continuation?.group&&multiSelectionRevision==now.multiSelectionRevision
}
private fun templateJSONMatches(value:Any?,expected:Any?):Boolean {
 if(value==null||value===JSONObject.NULL)return expected==null||expected===JSONObject.NULL
 if(expected==null||expected===JSONObject.NULL)return false
 if(value is JSONObject&&expected is JSONObject){val keys=value.keys().asSequence().toSet();return keys==expected.keys().asSequence().toSet()&&keys.all{templateJSONMatches(value.opt(it),expected.opt(it))}}
 if(value is org.json.JSONArray&&expected is org.json.JSONArray)return value.length()==expected.length()&&(0 until value.length()).all{templateJSONMatches(value.opt(it),expected.opt(it))}
 if(value is Number||expected is Number)return value is Number&&expected is Number&&value.toString()==expected.toString()
 return value==expected
}
private fun templateSnapshotMatches(value:JSONObject?,expected:SnippetVariableSnapshot):Boolean {
 if(value==null||!templateJSONMatches(value,expected.body()))return false
 val clock=value.optJSONObject("clock")
 if(expected.unixMillis!=null&&expected.utcOffsetMinutes!=null){if(clock==null||(clock.opt("unix_millis") as? Number)?.toString()!=expected.unixMillis.toString()||(clock.opt("utc_offset_minutes") as? Number)?.toString()!=expected.utcOffsetMinutes.toString())return false}
 val clipboard=value.optJSONObject("clipboard")
 if(expected.clipboard!=null&&clipboard?.opt("available") !is Boolean)return false
 return true
}
private fun templateVariableIdentity(value:JSONObject,capture:SourceTemplateCapture){
 if(capture.carets!=null)check(templateNumber(value.opt("template_batch_protocol"))==1&&templateJSONMatches(value.opt("carets"),JSONObject(capture.carets))){"The original complete template selection roster changed."}
 else check(listOf("template_batch_protocol","carets","expansions","readings").none{value.has(it)}){"Unexpected multiple-selection template reply."}
 if(capture.readingContext!=null)check(templateJSONMatches(value.opt("transform_program_versions"),org.json.JSONArray(listOf(2)))){"Template transform consumer versions changed."}
 check(listOf("command","additionalTextEdits","workspaceEdit","edit").none{value.has(it)}){"Template contains unsupported additional effects. No edits were applied."}
 check(templateNumber(value.opt("version"))==1&&templateNumber(value.opt("template_protocol"))==capture.protocol){"This engine does not support the captured template protocol."}
 if(capture.protocol==2)check(value.opt("source_files_modified")==false&&templateSnapshotMatches(value.optJSONObject("variable_context"),capture.variables!!)){"Template variable context changed. Refresh before applying."}else check(value.opt("source_files_modified")!=true&&value.isNull("variable_context")){"Legacy template response carries unsupported context/effects"}
}
private fun templateSourceContext(value:JSONObject,capture:SourceTemplateCapture):String? {
 if(capture.readingContext==null)return null
 val context=value.optJSONObject("source_context") ?: error("Embedded source language context is unavailable")
 check(context.keys().asSequence().toSet()==setOf("version","scope","language","languages","targets","reason","project_entry","source_hashes","graph_fingerprint","fingerprint")){"Malformed embedded source language context"}
 fun digest(text:String)=text.matches(Regex("[0-9a-f]{64}"))
 check(templateNumber(context.opt("version"))==1&&templateText(context,"scope").isNotEmpty()&&digest(templateText(context,"fingerprint"))&&digest(templateText(context,"graph_fingerprint"))){"Malformed source language binding"}
 val languages=templateStrings(context,"languages");val targets=templateStrings(context,"targets");val language=context.opt("language");check(language===JSONObject.NULL||language is String){"Malformed effective language"}
 check(languages.size<=1024&&targets.size<=1024&&(language===JSONObject.NULL||languages==listOf(language))&&value.opt("language")==if(language===JSONObject.NULL)"unknown" else language){"Ambiguous source language"}
 val project=JSONObject(capture.readingContext).opt("project_entry");check(templateJSONMatches(context.opt("project_entry"),project)||(project===JSONObject.NULL&&context.opt("project_entry")==capture.path)){"Construction entry changed"}
 val hashes=context.optJSONObject("source_hashes") ?: error("Missing compiler source hashes");check(hashes.length()<=10000&&hashes.keys().asSequence().all{digest(hashes.opt(it) as? String ?: "")}&&hashes.opt(capture.path)==debugHash(capture.source)){"Original compiler source binding changed"}
 check(context.opt("reason")===JSONObject.NULL||context.opt("reason") is String){"Malformed source-language reason"}
 val encoded=context.toString();check(encoded.toByteArray(Charsets.UTF_8).size<=1024*1024){"Source language metadata exceeds its interactive budget"};return encoded
}

private fun templateCaretReadings(value:JSONObject,capture:SourceTemplateCapture):String? {
 val carets=capture.carets?.let(::JSONObject) ?: return null
 val ranges=carets.getJSONArray("selections");val rows=value.optJSONArray("readings") ?: error("The complete caret language reading is missing")
 check(rows.length()==ranges.length()&&rows.length() in 2..256){"The complete template selection roster changed"}
 val global=value.optJSONObject("source_context") ?: error("Missing complete source context");var bytes=0L
 for(index in 0 until rows.length()){
  val row=rows.optJSONObject(index) ?: error("Malformed caret reading");check(row.keys().asSequence().toSet()==setOf("selection","context")&&templateJSONMatches(row.opt("selection"),ranges.opt(index))){"A caret language reading belongs to another source interval"}
  val facts=row.optJSONObject("context") ?: error("Missing caret language context")
  check(facts.keys().asSequence().toSet()==setOf("version","scope","language","languages","targets","reason","project_entry","graph_fingerprint","fingerprint")){"Malformed caret language context"}
  check(templateNumber(facts.opt("version"))==1&&templateText(facts,"scope").isNotEmpty()&&listOf("fingerprint","graph_fingerprint").all{templateText(facts,it).matches(Regex("[0-9a-f]{64}"))}){"Malformed caret language identity"}
  val language=facts.opt("language");val languages=templateStrings(facts,"languages");val targets=templateStrings(facts,"targets")
  check((language===JSONObject.NULL||language is String)&&languages.size<=1024&&targets.size<=1024&&(language===JSONObject.NULL||languages==listOf(language))&&(facts.opt("reason")===JSONObject.NULL||facts.opt("reason") is String)){"Malformed caret language values"}
  check(templateJSONMatches(facts.opt("project_entry"),global.opt("project_entry"))&&facts.opt("graph_fingerprint")==global.opt("graph_fingerprint")&&(global.opt("language")===JSONObject.NULL||language==global.opt("language"))&&targets.all{it in templateStrings(global,"targets")}){"A caret belongs to another compiler graph"}
  bytes+=facts.toString().toByteArray(Charsets.UTF_8).size;check(bytes<=1024*1024){"Complete caret language information exceeds its interactive budget"}
 }
 return rows.toString()
}

data class SourceTemplateEntry(val id:String,val label:String,val description:String,val language:String,val origin:String,val available:Boolean,val reason:String?)
class SourceTemplateCatalog private constructor(
    val capture:SourceTemplateCapture,val workspaceID:String,val language:String,val fingerprint:String,
    val available:Boolean,val reason:String?,val notices:List<String>,val entries:List<SourceTemplateEntry>,val sourceContext:String?=null,val readings:String?=null,
) {
    fun withCapture(next:SourceTemplateCapture):SourceTemplateCatalog {check(capture.matches(next)){"Template source capture changed"};return SourceTemplateCatalog(next,workspaceID,language,fingerprint,available,reason,notices,entries,sourceContext,readings)}
    companion object {
        fun decode(value:JSONObject,capture:SourceTemplateCapture):SourceTemplateCatalog {
            check(!value.has("context_request")){"Unexpected template catalog context request"}
            templateVariableIdentity(value,capture)
            check(templateText(value,"path")==capture.path&&templateText(value,"source_hash")==debugHash(capture.source)){"Template source changed or reply belongs to another source."}
            val selection=value.optJSONObject("selection") ?: error("Missing template selection")
            check(templateNumber(selection.opt("from"))==capture.from&&templateNumber(selection.opt("to"))==capture.to){"Template cursor changed."}
            val available=value.opt("available") as? Boolean ?: error("Malformed template availability")
            val workspace=templateText(value,"workspace_id");val language=templateText(value,"language");val fingerprint=templateText(value,"catalog_fingerprint")
            check(workspace.isNotEmpty()&&fingerprint.isNotEmpty()){ "Template catalog identity is missing." }
            val array=value.optJSONArray("entries") ?: error("Missing template entries");check(array.length()<=4096){"Template catalog exceeds the supported reading."}
            val entries=(0 until array.length()).map{index->val row=array.optJSONObject(index) ?: error("Malformed template entry")
                val origin=templateText(row,"origin");check(origin in listOf("builtin","workspace")){"Unsupported template origin"}
                SourceTemplateEntry(templateText(row,"id"),templateText(row,"label"),templateText(row,"description"),templateText(row,"language"),origin,row.opt("available") as? Boolean ?: error("Malformed template availability"),row.opt("reason") as? String)
            }
            check(entries.map{it.id}.toSet().size==entries.size&&entries.all{it.id.isNotEmpty()&&it.language==language}){"Ambiguous template identities"}
            return SourceTemplateCatalog(capture,workspace,language,fingerprint,available,value.opt("reason") as? String,java.util.Collections.unmodifiableList(templateStrings(value,"notices")),java.util.Collections.unmodifiableList(entries),templateSourceContext(value,capture),templateCaretReadings(value,capture))
        }
    }
}
private fun templatePlanIdentity(value:JSONObject,catalog:SourceTemplateCatalog,id:String):SourceTemplateEntry{
            val entry=catalog.entries.singleOrNull{it.id==id} ?: error("Template entry no longer exists")
            check(catalog.available&&entry.available){entry.reason ?: "Template unavailable for this source."}
            val context=catalog.capture
            templateVariableIdentity(value,context)
            check(templateSourceContext(value,context)?.let{fresh->catalog.sourceContext?.let{templateJSONMatches(JSONObject(fresh),JSONObject(it))}} ?: (catalog.sourceContext==null)){"The original compiler language context changed"}
            val expected=mapOf("path" to context.path,"source_hash" to debugHash(context.source),"workspace_id" to catalog.workspaceID,"language" to catalog.language,"catalog_fingerprint" to catalog.fingerprint,"template_id" to entry.id,"origin" to entry.origin)
            check(expected.all{(key,text)->templateText(value,key)==text}){"The template plan does not match the captured catalog and source."}
            check(templateCaretReadings(value,context)?.let{fresh->catalog.readings?.let{templateJSONMatches(org.json.JSONArray(fresh),org.json.JSONArray(it))}} ?: (catalog.readings==null)){"A caret compiler context changed"}
            if(context.carets!=null)check(!value.has("replacement")&&!value.has("snippet")){"A batch cannot substitute a single template edit"}
            for(key in if(context.carets==null)listOf("selection","replacement") else listOf("selection")){val range=value.optJSONObject(key) ?: error("Missing template $key");check(templateNumber(range.opt("from"))==context.from&&templateNumber(range.opt("to"))==context.to){"Template replacement must equal the captured selection."}}
            check(listOf("command","additionalTextEdits","workspaceEdit","edit").none{value.has(it)}){"Template plan contains unsupported effects; no partial application is allowed."}
 return entry
}
class SourceTemplatePlan internal constructor(val catalog:SourceTemplateCatalog,val entry:SourceTemplateEntry,val syntax:String,val draft:CompletionDraft,val notices:List<String>,val unavailable:List<String>,internal val batch:SourceTemplateBatch?=null) {
    internal val completeSource:String? get()=batch?.plan?.state?.source
    companion object {
        fun decode(value:JSONObject,catalog:SourceTemplateCatalog,id:String):SourceTemplatePlan {
            check(!value.has("context_request")){"Unexpected template plan context request"}
            val entry=templatePlanIdentity(value,catalog,id);val context=catalog.capture;check(value.opt("available")==true){"Template is unavailable for this captured context."}
            if(context.carets!=null)return decodeTemplateBatch(value,catalog,entry)
            val snippet=value.optJSONObject("snippet") ?: error("Missing template snippet")
            templateTransformPrograms(snippet)
            val syntax=templateText(snippet,"syntax")
            val item=JSONObject().put("label",entry.label).put("insertText",syntax).put("insertTextFormat",2).put("data",JSONObject().put("hickory_snippet",snippet))
            val expansion=completionExpansion(item) ?: error("Template expansion unavailable")
            val draft=planCompletion(languageURI(context.path),item,SourceSnapshot(context.source,true),context.from,context.to,expansion)
            val unavailable=if(value.has("context_unavailable"))templateStrings(value,"context_unavailable") else emptyList()
            check(unavailable.size<=40&&unavailable.all{it.length<=64}){"Malformed template context availability"}
            return SourceTemplatePlan(catalog,entry,syntax,draft,if(value.isNull("notices")&&context.protocol==1)emptyList() else java.util.Collections.unmodifiableList(templateStrings(value,"notices")),java.util.Collections.unmodifiableList(unavailable))
        }
    }
}
internal data class SourceTemplateBatch(val expansions:List<ParallelSnippetExpansion>,val plan:ParallelSnippetChange)
private fun templateTransformPrograms(snippet:JSONObject) {
 val fields=snippet.optJSONArray("fields") ?: error("Missing template fields")
 for(index in 0 until fields.length()){val field=fields.optJSONObject(index) ?: error("Malformed template field");if(field.has("transform")&&!field.isNull("transform"))check(templateNumber(field.getJSONObject("transform").opt("version"))==2){"Template transform was not negotiated by this captured consumer"}}
}
private fun templateBatchSnippets(value:JSONObject,capture:SourceTemplateCapture):List<JSONObject>{
 val ranges=JSONObject(capture.carets!!).getJSONArray("selections");val rows=value.optJSONArray("expansions") ?: error("Missing complete template expansions")
 check(rows.length()==ranges.length()&&rows.length() in 2..256){"A template expansion is missing from the reviewed roster"}
 var output=0L;var syntax=0L;var fields=0;var metadata=0L
 for(index in 0 until rows.length()){
  val row=rows.optJSONObject(index) ?: error("Malformed template expansion")
  check(row.keys().asSequence().toSet()==setOf("selection","replacement","snippet")&&templateJSONMatches(row.opt("selection"),ranges.opt(index))&&templateJSONMatches(row.opt("replacement"),ranges.opt(index))){"A template expansion belongs to another source selection"}
  val snippet=row.optJSONObject("snippet") ?: error("Missing template expansion");templateTransformPrograms(snippet)
  output+=templateText(snippet,"text").toByteArray(Charsets.UTF_8).size;syntax+=templateText(snippet,"syntax").toByteArray(Charsets.UTF_8).size
  val raw=snippet.getJSONArray("fields");fields+=raw.length();check(output<=1024*1024&&syntax<=8*1024*1024&&fields<=4096){"The complete template expansion exceeds its interactive budget"}
  for(at in 0 until raw.length()){
   val field=raw.getJSONObject(at);val choices=field.optJSONArray("choices") ?: error("Missing template choices");metadata+=64L+32L*choices.length();check(metadata<=1024*1024){"The complete template metadata exceeds its interactive budget"}
   for(choice in 0 until choices.length()){metadata+=(choices.opt(choice) as? String ?: error("Malformed template choice")).toByteArray(Charsets.UTF_8).size;check(metadata<=1024*1024){"The complete template metadata exceeds its interactive budget"}}
   if(field.has("transform")&&!field.isNull("transform")){metadata+=field.getJSONObject("transform").toString().toByteArray(Charsets.UTF_8).size;check(metadata<=1024*1024){"The complete template transform metadata exceeds its interactive budget"}}
  }
 }
 return (0 until rows.length()).map{rows.getJSONObject(it).getJSONObject("snippet")}
}
private fun decodeTemplateBatch(value:JSONObject,catalog:SourceTemplateCatalog,entry:SourceTemplateEntry):SourceTemplatePlan {
 val capture=catalog.capture;val carets=JSONObject(capture.carets!!);val ranges=carets.getJSONArray("selections");val primary=templateNumber(carets.opt("primary"));check(primary in 0 until ranges.length()){"Missing original primary template selection"}
 val snippets=templateBatchSnippets(value,capture);val budget=SnippetTransformBudget();var primaryDraft:CompletionDraft?=null
 val expansions=snippets.mapIndexed{index,snippet->
  val item=JSONObject().put("label",entry.label).put("insertText",templateText(snippet,"syntax")).put("insertTextFormat",2).put("data",JSONObject().put("hickory_snippet",snippet))
  val expansion=completionExpansion(item,budget) ?: error("Template expansion unavailable");val range=ranges.getJSONObject(index);val from=templateNumber(range.opt("from"));val to=templateNumber(range.opt("to"))
  if(index==primary)primaryDraft=planCompletion(languageURI(capture.path),item,SourceSnapshot(capture.source,true),from,to,expansion)
  ParallelSnippetExpansion(NativeSourceEdit(from,to,expansion.text),expansion.fields)
 }
 val planned=ParallelSnippet.plan(capture.source,expansions,primary,capture.continuation?.group) ?: error("The complete template group cannot be inserted at these original selections")
 val unavailable=if(value.has("context_unavailable"))templateStrings(value,"context_unavailable") else emptyList();check(unavailable.size<=40&&unavailable.all{it.length<=64}){"Malformed template context availability"}
 return SourceTemplatePlan(catalog,entry,value.getJSONArray("expansions").toString(),primaryDraft!!,java.util.Collections.unmodifiableList(templateStrings(value,"notices")),java.util.Collections.unmodifiableList(unavailable),SourceTemplateBatch(expansions,planned))
}

private fun captureSourceTemplateContributors(workspace:Workspace):Pair<String,()->Boolean>{
 val lease=workspace.sourceAcquisitionLease(protectSelection=false);val all=(workspace.buffers.values+workspace.retainedResourceEditors.values+workspace.retainedResourceTabs.values).distinct();check(all.size<=4096){"Too many retained template contributors"}
 val sources=JSONObject();val unaddressed=JSONObject();val missing=linkedSetOf<String>();var bytes=0L
 for(buffer in all){val size=buffer.text.toByteArray(Charsets.UTF_8).size;check(size<=8*1024*1024&&size<=32*1024*1024-bytes){"Template contributor text exceeds the interactive budget"};bytes+=size
  if(buffer.resourceDeleted){if(!buffer.file.path.startsWith("/"))missing.add(buffer.file.path);continue}
  if(buffer.isUntitledDraft||buffer.file.path.startsWith("/")){unaddressed.put(java.util.UUID.randomUUID().toString(),buffer.text);continue}
  check(!sources.has(buffer.file.path)){"Template contributor addresses are ambiguous"};sources.put(buffer.file.path,buffer.text)
 }
 for(path in sources.keys())missing.remove(path)
 return JSONObject().put("version",1).put("project_entry",JSONObject.NULL).put("sources",sources).put("unaddressed_sources",unaddressed).put("missing_sources",org.json.JSONArray(missing.sorted())).toString() to {lease.current()}
}
class SourceTemplatesModel {
    var captureVariables:()->SnippetVariableSnapshot={SnippetVariableSnapshot.capture()}
    var catalog by androidx.compose.runtime.mutableStateOf<SourceTemplateCatalog?>(null);private set
    data class Review(val plan:SourceTemplatePlan,val draft:CompletionDraft)
    var reviewed by androidx.compose.runtime.mutableStateOf<Review?>(null);private set
    var busy by androidx.compose.runtime.mutableStateOf(false);private set
    var notice by androidx.compose.runtime.mutableStateOf("");private set
    private var ticket=java.util.UUID.randomUUID()
    fun close(){ticket=java.util.UUID.randomUUID();busy=false;reviewed=null;catalog=null}
    private fun capture(workspace:Workspace,variables:SnippetVariableSnapshot?=null):SourceTemplateCapture {
        val engine=workspace.engine ?: error("Connect to this workspace before reading templates.")
        val buffer=workspace.currentBuffer ?: error("Open writable source before reading templates.")
        check(!buffer.readOnly&&!buffer.saving&&!buffer.resourceDeleted&&buffer.state.composition==null){"Finish composition and choose writable source that is not saving or deleted."}
        val parallel=buffer.snippets.parallel
        val logical=parallel?.selections?.map{NativeSourceEdit(it.from,it.to,"")} ?: buffer.multiRanges.takeIf{buffer.multiSource==buffer.text}.orEmpty()
        val carets=if(logical.size<2)null else {check(logical.size<=256){"Choose at most 256 source selections"};JSONObject().put("version",1).put("selections",org.json.JSONArray(logical.map{JSONObject().put("from",it.from).put("to",it.to)})).put("primary",parallel?.primaryIndex ?: 0).put("continuing",parallel!=null).toString()}
        val selected=buffer.state.selection
        return SourceTemplateCapture(workspace,engine,buffer,workspace.sourceNavigationEpoch,buffer.sourceRevision,buffer.file.path,buffer.text,selected.start,selected.end,buffer.snippets.intent,variables,carets=carets,continuation=parallel?.let(::SourceTemplateContinuation),multiSelectionRevision=buffer.multiSelectionRevision)
    }
    fun current(workspace:Workspace,context:SourceTemplateCapture)=workspace.sourceTemplates===this&&workspace.showTemplates&&runCatching{context.contributorsCurrent()&&context.entryCurrent()&&context.matches(capture(workspace))}.getOrDefault(false)
    suspend fun refresh(workspace:Workspace){
        close();val id=ticket;busy=true;notice="Reading source templates…"
        try {
            val origin=capture(workspace,captureVariables());val graph=captureSourceTemplateContributors(workspace);var context=origin.copy(readingContext=graph.first,contributorsCurrent=graph.second);val engine=context.engine as Engine
            for(attempt in 0 until 2){
                val reply=engine.request("POST","/api/templates/catalog",context.body())
                if(ticket!=id)return
                check(current(workspace,context)){"Source, selection or contributors changed while reading templates. Refresh the catalog."}
                val value=SourceTemplateCatalog.decode(reply,context)
                if(reply.has("reading_context_request")){
                    val required=reply.optJSONObject("reading_context_request") ?: error("Malformed construction entry request")
                    check(attempt==0&&required.length()==2&&templateNumber(required.opt("version"))==1&&required.opt("kind")=="construction_entry"&&!value.available&&value.entries.isEmpty()&&!JSONObject(context.readingContext!!).has("entry_selection")){"Unsupported or repeated construction entry request"}
                    val original=context;val runtime=engine.sourceWorkspaceID ?: error("Original workspace runtime is unavailable");val namespace=engine.sourceRootAddress ?: error("Original workspace root is unavailable")
                    val entry=WorkspaceEntrySelections.read(load={engine.request("GET","/api/workspace/entry")},workspace=runtime,namespace=namespace,current={current(workspace,original)})
                    if(ticket!=id)return;check(current(workspace,original)&&entry.current()){"Construction entry or original contributors changed"}
                    val reading=JSONObject(context.readingContext!!).put("project_entry",entry.entryPath ?: JSONObject.NULL).put("entry_selection",entry.proof.body());context=context.copy(readingContext=reading.toString(),entryCurrent=entry.current);continue
                }
                catalog=value;notice=if(value.available)"Choose a template to review its complete draft." else value.reason ?: "Templates are unavailable for this source.";return
            }
            error("Repeated construction entry demand. Source remains readable.")
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}
        finally{if(ticket==id)busy=false}
    }
    suspend fun prepare(workspace:Workspace,entry:SourceTemplateEntry){
        val catalog=catalog ?: return
        val id=java.util.UUID.randomUUID();ticket=id;reviewed=null;busy=true;notice="Preparing template preview…"
        try {
            check(current(workspace,catalog.capture)){"Source context changed. Refresh templates."}
            check(catalog.entries.any{it===entry}&&entry.available){entry.reason ?: "Template unavailable."}
            var reading=catalog
            for(attempt in 0 until 2){
                if(ticket!=id)return
                val capture=reading.capture;val request=capture.body().put("catalog_fingerprint",reading.fingerprint).put("template_id",entry.id);val reply=(capture.engine as Engine).request("POST","/api/templates/plan",request)
                if(ticket!=id)return;check(current(workspace,capture)){"Source, selection or snippet fields changed while preparing the template. Review again."}
                templatePlanIdentity(reply,reading,entry.id)
                if(reply.has("context_request")){val required=reply.optJSONObject("context_request");val names=required?.optJSONArray("variables");check(attempt==0&&capture.variables!=null&&capture.variables.clipboard==null&&reply.opt("available")==false&&required!=null&&required.length()==2&&(required.opt("version") as? Number)?.toString()=="1"&&names?.length()==1&&names.opt(0)=="CLIPBOARD"&&(capture.carets==null||!reply.has("expansions"))){"Unsupported or repeated template context request"}
                    val clipboard=captureSnippetClipboard((capture.engine as Engine).snippetClipboardReader);if(ticket!=id)return;check(current(workspace,capture)){"Source or selection changed while reading clipboard context."};reading=reading.withCapture(capture.copy(variables=capture.variables.copy(clipboard=clipboard)));this.catalog=reading;continue
                }
                val plan=SourceTemplatePlan.decode(reply,reading,entry.id);val draft=if(plan.batch==null)(capture.editor as DocumentBuffer).snippets.prepareCompletion(plan.draft) else plan.draft;reviewed=Review(plan,draft);notice="Review the complete draft. Apply does not save or run a command."+plan.notices.filter{it !in reading.notices}.joinToString("\n",prefix=if(plan.notices.any{it !in reading.notices})"\n" else "");return
            }
            error("Template context is unavailable. No edits were applied.")
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(ticket==id)notice=error.message.orEmpty()}
        finally{if(ticket==id)busy=false}
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    suspend fun apply(workspace:Workspace):Boolean {
        if(busy)return false
        val review=reviewed ?: return false
        val id=java.util.UUID.randomUUID();reviewed=null;ticket=id;busy=true
        try {
            check(current(workspace,review.plan.catalog.capture)){"Source, selection or editor changed. Review a fresh template before applying."}
            val capture=review.plan.catalog.capture
            val request=capture.body().put("catalog_fingerprint",review.plan.catalog.fingerprint).put("template_id",review.plan.entry.id)
            val reply=(capture.engine as Engine).request("POST","/api/templates/plan",request)
            if(ticket!=id)return false
            val fresh=SourceTemplatePlan.decode(reply,review.plan.catalog,review.plan.entry.id)
            check(fresh.syntax==review.plan.syntax&&fresh.draft==review.plan.draft&&fresh.notices==review.plan.notices&&fresh.unavailable==review.plan.unavailable){"Template changed after preview. Review a fresh plan before applying."}
            check(current(workspace,capture)){"Source, selection, composition or editor changed while verifying the reviewed template. No edits were applied."}
            val buffer=capture.editor as DocumentBuffer
            if(review.plan.batch!=null){
                val original=review.plan.batch;val checked=fresh.batch ?: error("The complete template group changed")
                check(original==checked){"The reviewed complete template group changed"}
                val primary=templateNumber(JSONObject(capture.carets!!).opt("primary"))
                check(buffer.snippets.applyParallel(capture.source,checked.expansions,primary,capture.snippetIntent){ticket==id&&current(workspace,capture)}){"The editor refused the complete reviewed template group"}
                check(buffer.text==original.plan.state.source){"The applied template differs from the reviewed complete group"}
                notice="Templates applied · draft. Existing source Undo/Redo retains the complete group.";catalog=null;workspace.showTemplates=false;buffer.focusSource?.invoke();return true
            }
            val draft=review.draft
            val before=buffer.snippets.captureBefore(draft.before,buffer.state.selection,true)!!;val serial=buffer.snippets.recordSerial
            check(draft.plan.patches.size<=1&&draft.plan.patches.all{it.path==review.plan.catalog.capture.path}){"Template preview contains unsupported additional source effects."}
            val edits=draft.plan.patches.singleOrNull()?.edits.orEmpty().map{NativeSourceEdit(it.from,it.to,it.text)}
            check(buffer.applySourceEdits(edits,draft.before,androidx.compose.ui.text.TextRange(draft.selection.from,draft.selection.to))){"Editor refused the complete template. No replacement request was sent; review again."}
            buffer.snippets.attachCompletionContext(before,draft.continuing ?: beginSnippet(buffer.text,draft.fields),serial)
            notice="Template applied · draft. Existing source Undo/Redo retains template fields.";catalog=null;workspace.showTemplates=false;buffer.focusSource?.invoke();return true
        }catch(cancelled:kotlinx.coroutines.CancellationException){throw cancelled}
        catch(error:Exception){if(ticket==id)notice=error.message.orEmpty();return false}
        finally{if(ticket==id)busy=false}
    }
}
@androidx.compose.runtime.Composable fun SourceTemplatesView(workspace:Workspace) {
    val model=workspace.sourceTemplates
    androidx.compose.foundation.layout.Column(androidx.compose.ui.Modifier.widthIn(min=280.dp,max=760.dp).heightIn(max=720.dp).verticalScroll(androidx.compose.foundation.rememberScrollState()).padding(12.dp)) {
        androidx.compose.material3.Text("Source templates")
        androidx.compose.material3.TextButton(onClick={workspace.showTemplates=false;workspace.currentBuffer?.focusSource?.invoke()}){androidx.compose.material3.Text("Close")}
        androidx.compose.material3.TextButton(enabled=!model.busy,onClick={workspace.runCommand{model.refresh(workspace)}}){androidx.compose.material3.Text("Refresh templates")}
        androidx.compose.material3.Text(model.notice)
        model.catalog?.let {catalog->
            catalog.notices.forEach{androidx.compose.material3.Text(it)}
            catalog.entries.forEach{entry->
                androidx.compose.material3.TextButton(enabled=catalog.available&&entry.available&&!model.busy&&model.current(workspace,catalog.capture),onClick={workspace.runCommand{model.prepare(workspace,entry)}}){androidx.compose.material3.Text("${entry.label} · ${entry.origin}")}
                androidx.compose.material3.Text(entry.description)
                entry.reason?.let{androidx.compose.material3.Text(it)}
            }
        }
        model.reviewed?.let{review->
            androidx.compose.material3.Text("Reviewed complete draft · ${review.plan.entry.label}")
            androidx.compose.foundation.text.selection.SelectionContainer{androidx.compose.material3.Text(review.plan.batch?.plan?.state?.source ?: review.draft.plan.patches.singleOrNull()?.after ?: review.draft.before)}
            androidx.compose.material3.TextButton(enabled=!model.busy&&model.current(workspace,review.plan.catalog.capture),onClick={workspace.runCommand{model.apply(workspace)}}){androidx.compose.material3.Text("Apply template to draft")}
        }
    }
}
