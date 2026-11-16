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

data class SourceTemplateCapture(
    val workspace:Any,val engine:Any,val editor:Any,val namespace:Any,val revision:Long,
    val path:String,val source:String,val anchor:Int,val head:Int,val snippetIntent:Int,
) {
    init {
        check(anchor in 0..source.length&&head in 0..source.length){"Template selection is outside the source."}
        sourceChangeOffset(source,languagePosition(source,anchor));sourceChangeOffset(source,languagePosition(source,head))
    }
    val from get()=minOf(anchor,head)
    val to get()=maxOf(anchor,head)
    fun body()=JSONObject().put("version",1).put("path",path).put("source",source).put("selection",JSONObject().put("from",from).put("to",to))
    fun matches(now:SourceTemplateCapture)=workspace===now.workspace&&engine===now.engine&&editor===now.editor&&namespace==now.namespace&&revision==now.revision&&path==now.path&&source==now.source&&anchor==now.anchor&&head==now.head&&snippetIntent==now.snippetIntent
}
data class SourceTemplateEntry(val id:String,val label:String,val description:String,val language:String,val origin:String,val available:Boolean,val reason:String?)
class SourceTemplateCatalog private constructor(
    val capture:SourceTemplateCapture,val workspaceID:String,val language:String,val fingerprint:String,
    val available:Boolean,val reason:String?,val notices:List<String>,val entries:List<SourceTemplateEntry>,
) {
    companion object {
        fun decode(value:JSONObject,capture:SourceTemplateCapture):SourceTemplateCatalog {
            check(templateNumber(value.opt("version"))==1&&templateNumber(value.opt("template_protocol"))==1){"This engine does not support template protocol 1."}
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
            check(entries.map{it.id}.toSet().size==entries.size&&entries.all{it.id.isNotEmpty()}){"Ambiguous template identities"}
            return SourceTemplateCatalog(capture,workspace,language,fingerprint,available,value.opt("reason") as? String,java.util.Collections.unmodifiableList(templateStrings(value,"notices")),java.util.Collections.unmodifiableList(entries))
        }
    }
}
class SourceTemplatePlan private constructor(val catalog:SourceTemplateCatalog,val entry:SourceTemplateEntry,val syntax:String,val draft:CompletionDraft) {
    companion object {
        fun decode(value:JSONObject,catalog:SourceTemplateCatalog,id:String):SourceTemplatePlan {
            val entry=catalog.entries.singleOrNull{it.id==id} ?: error("Template entry no longer exists")
            check(catalog.available&&entry.available){entry.reason ?: "Template unavailable for this source."}
            val context=catalog.capture
            check(templateNumber(value.opt("version"))==1&&templateNumber(value.opt("template_protocol"))==1){"Unsupported template plan protocol"}
            val expected=mapOf("path" to context.path,"source_hash" to debugHash(context.source),"workspace_id" to catalog.workspaceID,"language" to catalog.language,"catalog_fingerprint" to catalog.fingerprint,"template_id" to entry.id,"origin" to entry.origin)
            check(expected.all{(key,text)->templateText(value,key)==text}){"The template plan does not match the captured catalog and source."}
            for(key in listOf("selection","replacement")){val range=value.optJSONObject(key) ?: error("Missing template $key");check(templateNumber(range.opt("from"))==context.from&&templateNumber(range.opt("to"))==context.to){"Template replacement must equal the captured selection."}}
            check(listOf("command","additionalTextEdits","workspaceEdit").none{value.has(it)}){"Template plan contains unsupported effects; no partial application is allowed."}
            val snippet=value.optJSONObject("snippet") ?: error("Missing template snippet")
            val syntax=templateText(snippet,"syntax")
            val item=JSONObject().put("label",entry.label).put("insertText",syntax).put("insertTextFormat",2).put("data",JSONObject().put("hickory_snippet",snippet))
            val expansion=completionExpansion(item) ?: error("Template expansion unavailable")
            val draft=planCompletion(languageURI(context.path),item,SourceSnapshot(context.source,true),context.from,context.to,expansion)
            return SourceTemplatePlan(catalog,entry,syntax,draft)
        }
    }
}
class SourceTemplatesModel {
    var catalog by androidx.compose.runtime.mutableStateOf<SourceTemplateCatalog?>(null);private set
    data class Review(val plan:SourceTemplatePlan,val draft:CompletionDraft)
    var reviewed by androidx.compose.runtime.mutableStateOf<Review?>(null);private set
    var busy by androidx.compose.runtime.mutableStateOf(false);private set
    var notice by androidx.compose.runtime.mutableStateOf("");private set
    private var ticket=java.util.UUID.randomUUID()
    fun close(){ticket=java.util.UUID.randomUUID();busy=false;reviewed=null;catalog=null}
    private fun capture(workspace:Workspace):SourceTemplateCapture {
        val engine=workspace.engine ?: error("Connect to this workspace before reading templates.")
        val buffer=workspace.currentBuffer ?: error("Open writable source before reading templates.")
        check(!buffer.readOnly&&!buffer.saving&&!buffer.resourceDeleted&&buffer.state.composition==null){"Finish composition and choose writable source that is not saving or deleted."}
        check(!(buffer.multiSource==buffer.text&&buffer.multiRanges.size>1)){"Choose one source selection before inserting a template."}
        val selected=buffer.state.selection
        return SourceTemplateCapture(workspace,engine,buffer,workspace.sourceNavigationEpoch,buffer.sourceRevision,buffer.file.path,buffer.text,selected.start,selected.end,buffer.snippets.intent)
    }
    fun current(workspace:Workspace,context:SourceTemplateCapture)=workspace.sourceTemplates===this&&workspace.showTemplates&&runCatching{context.matches(capture(workspace))}.getOrDefault(false)
    suspend fun refresh(workspace:Workspace){
        close();val id=ticket;busy=true;notice="Reading source templates…"
        try {
            val context=capture(workspace);val engine=context.engine as Engine
            val reply=engine.request("POST","/api/templates/catalog",context.body())
            if(ticket!=id)return
            check(current(workspace,context)){"Source or selection changed while reading templates. Refresh the catalog."}
            catalog=SourceTemplateCatalog.decode(reply,context);notice=if(catalog!!.available)"Choose a template to review its complete draft." else catalog!!.reason ?: "Templates are unavailable for this source."
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
            val request=catalog.capture.body().put("catalog_fingerprint",catalog.fingerprint).put("template_id",entry.id)
            val reply=(catalog.capture.engine as Engine).request("POST","/api/templates/plan",request)
            if(ticket!=id)return
            check(current(workspace,catalog.capture)){"Source, selection or snippet fields changed while preparing the template. Review again."}
            val plan=SourceTemplatePlan.decode(reply,catalog,entry.id)
            val draft=(catalog.capture.editor as DocumentBuffer).snippets.prepareCompletion(plan.draft)
            reviewed=Review(plan,draft);notice="Review the complete draft. Apply does not save or run a command."
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
            check(fresh.syntax==review.plan.syntax&&fresh.draft==review.plan.draft){"Template changed after preview. Review a fresh plan before applying."}
            check(current(workspace,capture)){"Source, selection, composition or editor changed while verifying the reviewed template. No edits were applied."}
            val buffer=capture.editor as DocumentBuffer;val draft=review.draft
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
            androidx.compose.foundation.text.selection.SelectionContainer{androidx.compose.material3.Text(review.draft.plan.patches.singleOrNull()?.after ?: review.draft.before)}
            androidx.compose.material3.TextButton(enabled=!model.busy&&model.current(workspace,review.plan.catalog.capture),onClick={workspace.runCommand{model.apply(workspace)}}){androidx.compose.material3.Text("Apply template to draft")}
        }
    }
}
