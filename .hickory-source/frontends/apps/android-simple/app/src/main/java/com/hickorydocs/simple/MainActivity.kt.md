<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Launch a native Compose workspace

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The activity retains a ViewModel across recreation. A development launch supplies `hickory_api_origin` in its Intent; use it only for an untouched, disconnected workspace, then retain the session across recomposition. Failed startup connections leave the manual Connect dialog available. System back refuses to discard unsaved buffers; missing server or agent configuration remains independent of editing.

<ui:copy id="simple-d8c991adb6-0">
package com.hickorydocs.simple

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.*
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState); enableEdgeToEdge()
        setContent { MaterialTheme { Surface { SimpleWorkspace(viewModel(), intent.getStringExtra("hickory_api_origin")) } } }
    }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable fun SimpleWorkspace(workspace: Workspace, initialAddress: String? = null) {
    var connection by remember { mutableStateOf(workspace.engine == null) }
    val scope = rememberCoroutineScope()
    LaunchedEffect(initialAddress) {
        if (workspace.engine == null && workspace.address.isEmpty() && !initialAddress.isNullOrEmpty()) {
            workspace.address = initialAddress
            connection = !workspace.connect()
        }
    }
    BackHandler(workspace.hasUnsavedChanges) { workspace.status = "Save your documents before leaving this workspace." }
    Column(Modifier.fillMaxSize().safeDrawingPadding().imePadding().onPreviewKeyEvent {
        if (it.type == KeyEventType.KeyDown && it.key == Key.S && (it.isCtrlPressed || it.isMetaPressed)) {
            scope.launch { workspace.save() }; true
        } else if (it.type == KeyEventType.KeyDown && it.key == Key.F && (it.isCtrlPressed || it.isMetaPressed)) {
            workspace.currentBuffer?.showFind = true; workspace.currentBuffer != null
        } else if (it.type == KeyEventType.KeyDown && it.key == Key.Escape && workspace.currentBuffer?.showFind == true) {
            workspace.currentBuffer?.closeFind(); true
        } else false
    }) {
        Row(Modifier.fillMaxWidth().height(32.dp)) {
            CommandMenu("File", listOf(
                "Connect…" to { connection = true },
                "Refresh files" to { scope.launch { workspace.refresh() }; Unit },
                "Save" to { scope.launch { workspace.save() }; Unit },
                "Close tab" to { workspace.active?.let(workspace::close); Unit }
            ))
            CommandMenu("Edit", listOf("Find and replace" to { workspace.currentBuffer?.showFind = true; Unit },
                "Column selection" to { workspace.currentBuffer?.selectRectangle(); Unit },
                "Add cursor below" to { workspace.currentBuffer?.addCursor(1); Unit }, "Add cursor above" to { workspace.currentBuffer?.addCursor(-1); Unit }))
            CommandMenu("View", listOf(
                "Files" to { workspace.show("files") }, "Agent" to { workspace.show("agent") },
                "Terminal" to { scope.launch { workspace.openTerminal() }; Unit },
                "Split right" to { workspace.split("horizontal") }, "Split below" to { workspace.split("vertical") },
                "Move tab to next group" to { workspace.moveNext() }
            ))
        }
        HorizontalDivider()
        Box(Modifier.weight(1f)) { Pane(workspace, workspace.layout.rootID) }
        HorizontalDivider()
        Row(Modifier.fillMaxWidth().height(24.dp).padding(horizontal = 8.dp)) {
            Text(workspace.status, fontSize = 11.sp, maxLines = 1, modifier = Modifier.weight(1f))
            workspace.currentBuffer?.let { buffer -> Text(if (buffer.readOnly) "Read-only" else if (buffer.saving) "Saving…" else if (buffer.dirty) "Modified" else "Saved", fontSize = 11.sp) }
        }
    }
    if (connection) AlertDialog(onDismissRequest = { connection = false }, title = { Text("Workspace server") }, text = {
        Column {
            OutlinedTextField(workspace.address, { workspace.address = it }, label = { Text("Workspace server") }, singleLine = true)
            Text("Use a host reachable from this device. For an emulator, adb reverse can forward the backend port.", fontSize = 12.sp)
            Text(workspace.status, fontSize = 12.sp)
        }
    }, confirmButton = { TextButton(enabled = !workspace.connecting, onClick = { scope.launch { if (workspace.connect()) connection = false } }) { Text("Connect") } },
        dismissButton = { TextButton(onClick = { connection = false }) { Text("Cancel") } })
}
@Composable private fun CommandMenu(label: String, commands: List<Pair<String, () -> Unit>>) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        TextButton(onClick = { expanded = true }, contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp)) { Text(label, fontSize = 13.sp) }
        DropdownMenu(expanded, { expanded = false }) {
            commands.forEach { (name, action) -> DropdownMenuItem(text = { Text(name) }, onClick = { expanded = false; action() }) }
        }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/MainActivity.kt"><ui:paste select="#simple-d8c991adb6-0"/></ui:file>

</ui:doc>
