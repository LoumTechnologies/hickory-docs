<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Prepare Android source patches without owning views

For maintainers changing workspace services independently of the editor renderer.

A workspace change is a set of exact before/after snapshots and UTF-16 replacements. The pure planner knows no views, sockets or saving. It refuses an unknown, generated, versioned-without-proof, overlapping or malformed target before producing any patches. Inverse replacements use offsets in the resulting source, so coordinated undo can check the complete after snapshot before restoring anything. Adjacent edits are allowed; duplicate identical edits collapse, while two different insertions at the same offset are ambiguous and refuse. Scalar boundaries protect surrogate pairs.

A rename also checks coherence with a fresh provider reference read: every authored source identity that the provider reports must occur in the proposed source patches. Missing identities refuse the preview. This does not prove that the provider indexed the whole project, and an empty reference answer is not proof that no usages exist.

Discovery and planning share the same workspace-edit decoder. It validates source identities, representation, edit lists and unsupported resource/version forms before loading any target. Decoding paths is discovery only: ranges and replacements still require exact captured text in the planner.

Completion planning reuses the scalar-aware offset decoder. Only an explicit selection-only caller may allow an empty patch set after all targets and edits have still been validated; ordinary workspace proposals continue to refuse no-change results. Hosts must retain the before snapshot and recheck source and permissions even for an empty patch set.

<ui:copy id="simple-android-change-plans">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONArray
data class SourceSnapshot(val text: String, val writable: Boolean)
data class SourceReplacement(val from: Int, val to: Int, val text: String)
data class SourcePatch(val path: String, val before: String, val after: String, val edits: List<SourceReplacement>)
data class ChangePlan(val title: String, val patches: List<SourcePatch>)
fun sourceChangeOffset(source: String, raw: Any?): Int {
    val point = raw as? JSONObject ?: error("Invalid UTF-16 source position")
    for (key in listOf("line","character")) { val value = point.opt(key) as? Number ?: error("Invalid UTF-16 source position"); require(value.toDouble() == value.toInt().toDouble() && value.toInt() >= 0) { "Invalid UTF-16 source position" } }
    val offset = languageOffset(source,point) ?: error("Source edit lies outside its captured source")
    require(offset == 0 || offset == source.length || !Character.isHighSurrogate(source[offset-1]) || !Character.isLowSurrogate(source[offset])) { "Source edit splits a Unicode scalar" }
    return offset
}
data class WorkspaceEditTarget(val path:String,val items:JSONArray)
fun workspaceEditTargets(raw:Any?):List<WorkspaceEditTarget> {
    val edit=raw as? JSONObject ?: error("No workspace edit to preview")
    val targets=mutableListOf<WorkspaceEditTarget>()
    fun add(rawURI:Any?,rawEdits:Any?,version:Any?=null) {
        val uri=rawURI as? String ?: error("Malformed workspace edit target")
        val items=rawEdits as? JSONArray ?: error("Malformed source edit list")
        require(version==null || version==JSONObject.NULL){"Authored source version could not be established. Refresh the proposal."}
        val path=SemanticLocation(uri,0,0).path ?: error("Unsupported source identity: $uri")
        targets.add(WorkspaceEditTarget(path,items))
    }
    if (edit.has("changes")) {
        val changes = edit.optJSONObject("changes") ?: error("Malformed workspace changes")
        for (uri in changes.keys()) add(uri,changes.opt(uri))
    }
    if (edit.has("documentChanges")) {
        require(targets.isEmpty()) { "Ambiguous workspace change representation" }
        val changes = edit.optJSONArray("documentChanges") ?: error("Malformed documentChanges")
        for (index in 0 until changes.length()) {
            val change = changes.optJSONObject(index) ?: error("Malformed document change")
            require(!change.has("kind")) { "Resource create, move and delete require a filesystem proposal" }
            val document = change.optJSONObject("textDocument") ?: error("Missing source edit URI")
            add(document.opt("uri"),change.opt("edits"),document.opt("version"))
        }
    }
    return targets
}
fun providerActionPaths(action:JSONObject):List<String> {
    val data=action.optJSONObject("data")
    if(data==null || !data.has("hickory_acquire"))return workspaceEditTargets(action.opt("edit")).map{it.path}
    val items=data.optJSONArray("hickory_acquire") ?: error("Malformed provider source acquisition")
    require(items.length() in 1..10000){"Malformed provider source acquisition"}
    return (0 until items.length()).map{val uri=items.optJSONObject(it)?.opt("uri") as? String ?: error("Unsupported provider acquisition source");SemanticLocation(uri,0,0).path ?: error("Unsupported provider acquisition source")}
}
fun planWorkspaceEdit(title: String, raw: Any?, sources: Map<String,SourceSnapshot>,allowNoop:Boolean=false): ChangePlan {
    val groups = sortedMapOf<String,MutableList<SourceReplacement>>()
    for(target in workspaceEditTargets(raw)) {
        val path=target.path;val items=target.items
        val source=sources[path] ?: error("Acquire the affected source before preparing this proposal: $path")
        require(source.writable) { "Source is generated, read-only or saving: $path" }
        val replacements = groups.getOrPut(path) { mutableListOf() }
        for (index in 0 until items.length()) {
            val item = items.optJSONObject(index) ?: error("Malformed source replacement")
            val range = item.optJSONObject("range") ?: error("Missing source range")
            val text = item.opt("newText") as? String ?: error("Missing replacement text")
            val replacement = SourceReplacement(sourceChangeOffset(source.text,range.opt("start")),sourceChangeOffset(source.text,range.opt("end")),text)
            require(replacement.from <= replacement.to) { "Source replacement runs backwards" }
            if (replacement !in replacements) replacements.add(replacement)
            require(replacements.size <= 10000) { "Proposal exceeds 10,000 replacements in one source" }
        }
    }
    val patches = groups.mapNotNull { (path,replacements) ->
        val edits = replacements.sortedWith(compareBy({it.from},{it.to}))
        for ((left,right) in edits.zipWithNext()) require(right.from >= left.to && right.from != left.from) { "Overlapping source edits: $path" }
        val before = sources.getValue(path).text; val after = StringBuilder(before)
        for (item in edits.asReversed()) after.replace(item.from,item.to,item.text)
        if (before == after.toString()) null else SourcePatch(path,before,after.toString(),edits)
    }
    require(patches.isNotEmpty()||allowNoop) { "The proposal contains no source changes" }
    return ChangePlan(title,patches)
}
fun invertChangePlan(plan: ChangePlan): ChangePlan = ChangePlan("Undo ${plan.title}",plan.patches.map { patch ->
    var delta = 0
    val edits = patch.edits.map { edit -> val from = edit.from+delta; delta += edit.text.length-(edit.to-edit.from); SourceReplacement(from,from+edit.text.length,patch.before.substring(edit.from,edit.to)) }
    SourcePatch(patch.path,patch.after,patch.before,edits)
})
fun validateChangePlan(plan: ChangePlan, sources: Map<String,SourceSnapshot>): String? {
    for (patch in plan.patches) if (sources[patch.path]?.let { it.writable && it.text == patch.before } != true) return "Source changed, is unavailable, read-only or saving: ${patch.path}. Prepare a fresh proposal."
    return null
}
fun validateReferencedTargets(plan: ChangePlan, referencePaths: List<String>) {
    val missing = referencePaths.toSet()-plan.patches.map { it.path }.toSet()
    require(missing.isEmpty()) { "Rename omitted reported reference source targets: ${missing.sorted().joinToString(", ")}. Refresh project analysis and prepare again. No edits were applied." }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ChangePlan.kt"><ui:paste select="#simple-android-change-plans"/></ui:file>
</ui:doc>
