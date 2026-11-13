<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Match the explicit workspace agent conversation in Compose

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Polling and prompt state are confined to the agent pane. Cancelling its composition cancels polling; it never cancels or retries a dispatched agent turn implicitly. A failed provider leaves the editor operational.

<ui:copy id="simple-716e739687-0">
package com.hickorydocs.simple

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import androidx.lifecycle.viewModelScope

class AgentSession {
    var turns by mutableStateOf<List<AgentTurn>>(emptyList())
    var provider by mutableStateOf("codex-subscription")
    var model by mutableStateOf("")
    var effort by mutableStateOf("")
    val draft = DocumentBuffer(FileNode("Message", "conversation/draft.md", false, emptyList(), null, null), FileText("", ""))
    var prompt: String
        get() = draft.text
        set(value) { draft.state.edit { replace(0, length, value) } }
    private val documents = mutableMapOf<String, DocumentBuffer>()
    fun document(id: String, text: String): DocumentBuffer {
        val held = documents[id]
        if (held != null && held.text == text) return held
        val path = "conversation/$id.md"
        return DocumentBuffer(FileNode(id, path, false, emptyList(), null, "conversation"), FileText(text, "")).also { documents[id] = it }
    }
    var sending by mutableStateOf(false)
    var error by mutableStateOf("")
    var mutation = 0
    val running get() = sending || turns.any { it.status == "running" }
}
</ui:copy>

## Reuse the document renderer inside message cards

LanguageEditor accepts the same Markdown buffer for previous messages and the
composer, with no language or debugger host. A read-only history buffer protects
inline edits as well as typing. Flow the provider and effort controls on narrow
screens, then keep Send beneath the draft. The session outlives tab composition;
polling cancels when hidden, while an acknowledged send continues independently.

<ui:copy id="simple-android-agent-pane">
@OptIn(ExperimentalLayoutApi::class)
@Composable fun AgentPane(engine: Engine?, session: AgentSession, workspace: Workspace) {
    val scope = workspace.viewModelScope
    val running = session.running
    LaunchedEffect(engine) {
        if (engine == null) { session.error = "Connect to a workspace server to use the agent."; return@LaunchedEffect }
        while (isActive) {
            try { val mutation = session.mutation; val latest = engine.turns(); if (mutation == session.mutation && !session.sending) session.turns = latest }
            catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { session.error = "${failure.message} Check subscription sign-in on the workspace server." }
            delay(1000)
        }
    }
    Column(Modifier.fillMaxSize()) {
        Text("Workspace agent · Hickory Python only", style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(12.dp))
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(10.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (session.turns.isEmpty()) item {
                Text("What shall we work on?", style = MaterialTheme.typography.titleLarge)
                Text("Write a literate document below. Code and explanations belong together.")
            }
            items(session.turns, key = { it.id }) { turn -> Column {
                Text("You", style = MaterialTheme.typography.labelMedium)
                Box(Modifier.fillMaxWidth().padding(start = 24.dp)) {
                    Surface(shape = androidx.compose.foundation.shape.RoundedCornerShape(12.dp), color = MaterialTheme.colorScheme.surfaceContainer) {
                        AgentDocument(session.document("${turn.id}-user", turn.prompt))
                    }
                }
                Text("Agent · ${turn.status}", style = MaterialTheme.typography.labelMedium)
                AgentDocument(session.document("${turn.id}-agent", turn.answer ?: turn.error ?: if (turn.status == "running") "Working…" else ""))
            } }
        }
        if (session.error.isNotEmpty()) Text(session.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp, modifier = Modifier.padding(6.dp))
        HorizontalDivider()
        Column(Modifier.fillMaxWidth().padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("Your message · literate document", style = MaterialTheme.typography.labelMedium)
            LanguageEditor(session.draft, null, Modifier.fillMaxWidth().height(140.dp))
            FlowRow(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                AgentChoice("Provider", session.provider, listOf("codex-subscription" to "ChatGPT subscription", "claude-subscription" to "Claude Code subscription", "grok-subscription" to "Grok subscription"), !running) {
                    session.provider = it; session.model = ""; session.effort = ""
                }
                AgentChoice("Effort", session.effort, listOf("" to "Default") + listOf("low", "medium", "high", "xhigh", "max").map { it to it }, !running && session.provider != "grok-subscription") { session.effort = it }
            }
            OutlinedTextField(session.model, { session.model = it }, enabled = !running, label = { Text("Model · provider default") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            TextButton(enabled = engine != null && !running && session.prompt.isNotBlank(), onClick = {
                val submitted = session.prompt
                session.sending = true; session.error = ""; session.mutation++
                scope.launch {
                    try {
                        val id = engine!!.send(submitted, session.turns.lastOrNull()?.id, session.provider, session.model, session.effort)
                        session.turns = session.turns.filter { it.id != id } + AgentTurn(id, submitted, null, null, "running")
                        if (session.prompt == submitted) session.prompt = ""
                    } catch (cancelled: CancellationException) { throw cancelled }
                    catch (failure: Exception) { session.error = "${failure.message} The request was not retried. Refresh history before sending again." }
                    finally { session.sending = false; session.mutation++ }
                }
            }) { Text("Send") }
            TextButton(enabled = running, onClick = { scope.launch {
                try { engine?.stop() } catch (cancelled: CancellationException) { throw cancelled }
                catch (failure: Exception) { session.error = "${failure.message} Refresh history and try Stop again." }
            } }) { Text("Stop") }
            }
            Text("Sign in on the workspace server with " + when (session.provider) { "claude-subscription" -> "claude auth login"; "grok-subscription" -> "grok login"; else -> "codex login" } + ". Uses your subscription; no API fallback.", style = MaterialTheme.typography.labelSmall)
        }
    }
}

</ui:copy>

## Bound long messages without changing their source

A long message scrolls within its document card; shorter messages retain compact
heights. Provider and effort choices use floating menus, so opening one does not
move the conversation. Changing a selection retains the draft buffer.

<ui:copy id="simple-android-agent-message-controls">
@Composable private fun AgentDocument(buffer: DocumentBuffer) {
    key(buffer) { LanguageEditor(buffer, null, Modifier.fillMaxWidth().height((buffer.text.lines().size * 22 + 36).coerceIn(100, 420).dp)) }
}
@Composable private fun AgentChoice(label: String, selected: String, choices: List<Pair<String, String>>, enabled: Boolean, change: (String) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        OutlinedButton(enabled = enabled, onClick = { expanded = true }) { Text("$label: ${choices.firstOrNull { it.first == selected }?.second ?: selected}") }
        DropdownMenu(expanded, { expanded = false }) { choices.forEach { (value, title) ->
            DropdownMenuItem(text = { Text(title) }, onClick = { expanded = false; change(value) })
        } }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/AgentPane.kt"><ui:paste select="#simple-716e739687-0"/><ui:paste select="#simple-android-agent-pane"/><ui:paste select="#simple-android-agent-message-controls"/></ui:file>

</ui:doc>
