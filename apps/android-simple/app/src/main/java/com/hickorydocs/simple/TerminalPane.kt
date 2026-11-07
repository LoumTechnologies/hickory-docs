package com.hickorydocs.simple

import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class TerminalSession {
    var id by mutableStateOf<String?>(null)
    var opening by mutableStateOf(false)
    var input by mutableStateOf("")
    var screen by mutableStateOf("")
    var error by mutableStateOf("")
    var sending by mutableStateOf(false)
    suspend fun open(engine: Engine) {
        if (id != null || opening) return
        opening = true; error = ""
        try { id = engine.openTerminal() }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { error = "${failure.message} Check the terminal backend, then choose Terminal again." }
        finally { opening = false }
    }
    suspend fun send(engine: Engine, data: String? = null) {
        val current = id ?: return
        if (sending) return
        val submitted = input; sending = true; error = ""
        try { engine.terminalInput(current, data ?: submitted + "\r"); if (data == null && input == submitted) input = "" }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { error = "${failure.message} Input was not retried. Check the screen before sending again." }
        finally { sending = false }
    }
}
@Composable fun TerminalPane(workspace: Workspace) {
    val session = workspace.terminal; val engine = workspace.engine
    val scope = workspace.viewModelScope
    var size by remember { mutableStateOf(IntSize.Zero) }
    val density = LocalDensity.current.density
    LaunchedEffect(session.id, engine) {
        val id = session.id ?: return@LaunchedEffect
        if (engine == null) return@LaunchedEffect
        while (isActive) {
            try { val screen = engine.terminalScreen(id); if (session.id == id) session.screen = screen }
            catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { session.error = "${failure.message} Reopen Terminal to reconnect to this session." }
            delay(250)
        }
    }
    LaunchedEffect(size, session.id) {
        val id = session.id ?: return@LaunchedEffect
        if (engine == null || size.width == 0 || size.height == 0) return@LaunchedEffect
        try { engine.terminalResize(id, (size.height / density / 17).toInt().coerceIn(1, 60), (size.width / density / 8).toInt().coerceIn(1, 160)) }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { session.error = "${failure.message} Resize the pane to retry its terminal size." }
    }
    Column(Modifier.fillMaxSize()) {
        Box(Modifier.weight(1f).fillMaxWidth().onSizeChanged { size = it }.verticalScroll(rememberScrollState()).horizontalScroll(rememberScrollState()).padding(6.dp)) {
            Text(session.screen, fontFamily = FontFamily.Monospace, fontSize = 13.sp)
        }
        if (session.error.isNotEmpty()) Text(session.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp)
        HorizontalDivider()
        Row(Modifier.fillMaxWidth().padding(6.dp)) {
            OutlinedTextField(session.input, { session.input = it }, label = { Text("Terminal input") }, modifier = Modifier.weight(1f), singleLine = true)
            TextButton(enabled = session.id != null && !session.sending, onClick = { if (engine != null) scope.launch { session.send(engine) } }) { Text("Send") }
            TextButton(enabled = session.id != null && !session.sending, onClick = { if (engine != null) scope.launch { session.send(engine, "\u0003") } }) { Text("Interrupt") }
            TextButton(enabled = session.id != null && !session.sending, onClick = { if (engine != null) scope.launch { session.send(engine, "\u0004") } }) { Text("EOF") }
        }
    }
}
