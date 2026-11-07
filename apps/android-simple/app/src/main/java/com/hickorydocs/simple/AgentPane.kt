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
    var prompt by mutableStateOf("")
    var sending by mutableStateOf(false)
    var error by mutableStateOf("")
    var mutation = 0
    val running get() = sending || turns.any { it.status == "running" }
}
@Composable fun AgentPane(engine: Engine?, session: AgentSession, workspace: Workspace) {
    val scope = workspace.viewModelScope
    val running = session.running
    LaunchedEffect(engine) {
        if (engine == null) { session.error = "Connect to a workspace server to use the agent."; return@LaunchedEffect }
        while (isActive) {
            try { val mutation = session.mutation; val latest = engine.turns(); if (mutation == session.mutation && !session.sending) session.turns = latest }
            catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { session.error = "${failure.message} Check agent configuration in the existing frontend." }
            delay(1000)
        }
    }
    Column(Modifier.fillMaxSize()) {
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(10.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(session.turns, key = { it.id }) { turn -> Column {
                Text("You", style = MaterialTheme.typography.labelMedium); Text(turn.prompt)
                Text("Agent · ${turn.status}", style = MaterialTheme.typography.labelMedium)
                Text(turn.answer ?: turn.error ?: if (turn.status == "running") "Working…" else "")
            } }
        }
        if (session.error.isNotEmpty()) Text(session.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp, modifier = Modifier.padding(6.dp))
        HorizontalDivider()
        Row(Modifier.fillMaxWidth().padding(6.dp)) {
            OutlinedTextField(session.prompt, { session.prompt = it }, modifier = Modifier.weight(1f), placeholder = { Text("Message the workspace agent") }, maxLines = 5)
            TextButton(enabled = engine != null && !running && session.prompt.isNotBlank(), onClick = {
                val submitted = session.prompt
                session.sending = true; session.error = ""; session.mutation++
                scope.launch {
                    try {
                        val id = engine!!.send(submitted, session.turns.lastOrNull()?.id)
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
    }
}
