package com.hickorydocs.simple

import android.content.ClipData
import android.content.ClipDescription
import androidx.compose.foundation.*
import androidx.compose.foundation.draganddrop.dragAndDropSource
import androidx.compose.foundation.draganddrop.dragAndDropTarget
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.geometry.Offset
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.draganddrop.*
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch

private const val TAB_MIME = "application/vnd.hickorydocs.tab"

@Composable private fun tabDropTarget(workspace: Workspace, group: String, before: String?, targeted: (Boolean) -> Unit): DragAndDropTarget {
    val latestTargeted by rememberUpdatedState(targeted)
    return remember(workspace, group, before) { object : DragAndDropTarget {
        override fun onEntered(event: DragAndDropEvent) { latestTargeted(true) }
        override fun onExited(event: DragAndDropEvent) { latestTargeted(false) }
        override fun onEnded(event: DragAndDropEvent) { latestTargeted(false) }
        override fun onDrop(event: DragAndDropEvent): Boolean {
            latestTargeted(false)
            val clip = event.toAndroidDragEvent().clipData ?: return false
            if (clip.itemCount != 1 || !clip.description.hasMimeType(TAB_MIME) || clip.description.label?.toString() != "hickory-tab") return false
            val tab = clip.getItemAt(0).text?.toString() ?: return false
            if (workspace.layout.groups().none { tab in it.tabs }) return false
            workspace.layout = workspace.layout.move(tab, group, before); workspace.focused = group; return true
        }
    } }
}
@OptIn(ExperimentalFoundationApi::class)
@Composable fun Pane(workspace: Workspace, id: String) {
    val node = workspace.layout.nodes[id] ?: return
    if (node.first != null && node.second != null) {
        BoxWithConstraints(Modifier.fillMaxSize()) {
            val horizontal = node.axis == "horizontal"
            val size = if (horizontal) maxWidth else maxHeight
            val firstSize = ((size - 5.dp) * node.ratio.toFloat()).coerceAtLeast(0.dp)
            val density = androidx.compose.ui.platform.LocalDensity.current
            val pixels = with(density) { size.toPx() }.coerceAtLeast(1f)
            val handle = Modifier.background(MaterialTheme.colorScheme.outlineVariant)
                .semantics { contentDescription = "Resize panes"; progressBarRangeInfo = ProgressBarRangeInfo(node.ratio.toFloat(), 0.1f.. 0.9f)
                    setProgress { value -> workspace.layout = workspace.layout.resize(id, value.toDouble()); true } }
                .pointerInput(id, pixels) { detectDragGestures { change, amount ->
                    change.consume(); val current = workspace.layout.nodes[id]?.ratio ?: 0.5
                    workspace.layout = workspace.layout.resize(id, current + (if (horizontal) amount.x else amount.y) / pixels)
                } }
            if (horizontal) Row(Modifier.fillMaxSize()) {
                Box(Modifier.width(firstSize).fillMaxHeight()) { Pane(workspace, node.first) }
                Box(handle.width(5.dp).fillMaxHeight())
                Box(Modifier.weight(1f).fillMaxHeight()) { Pane(workspace, node.second) }
            } else Column(Modifier.fillMaxSize()) {
                Box(Modifier.height(firstSize).fillMaxWidth()) { Pane(workspace, node.first) }
                Box(handle.height(5.dp).fillMaxWidth())
                Box(Modifier.weight(1f).fillMaxWidth()) { Pane(workspace, node.second) }
            }
        }
        return
    }
    var dropBefore by remember { mutableStateOf<String?>(null) }
    val target = tabDropTarget(workspace, id, null) { if (it) dropBefore = null }
    Column(Modifier.fillMaxSize().onFocusChanged { if (it.hasFocus) workspace.focused = id }.focusGroup()) {
        Row(Modifier.fillMaxWidth().height(29.dp).background(MaterialTheme.colorScheme.surfaceContainer)
            .dragAndDropTarget(shouldStartDragAndDrop = { TAB_MIME in it.mimeTypes() }, target = target).horizontalScroll(rememberScrollState())) {
            val activeColor = MaterialTheme.colorScheme.primary
            node.tabs.forEach { tab ->
                val tabTarget = tabDropTarget(workspace, id, tab) { targeted ->
                    if (targeted) dropBefore = tab else if (dropBefore == tab) dropBefore = null
                }
                Row(Modifier.background(if (node.active == tab) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainer)
                    .drawWithContent {
                        drawContent()
                        if (node.active == tab) drawRect(activeColor, size = Size(size.width, 2.dp.toPx()))
                        if (dropBefore == tab) drawRect(activeColor, size = Size(2.dp.toPx(), size.height))
                    }
                    .dragAndDropTarget(shouldStartDragAndDrop = { TAB_MIME in it.mimeTypes() }, target = tabTarget)
                    .dragAndDropSource { _ -> DragAndDropTransferData(ClipData(ClipDescription("hickory-tab", arrayOf(TAB_MIME)), ClipData.Item(tab))) }) {
                    Text(workspace.label(tab), fontSize = 13.sp, modifier = Modifier.clickable { workspace.layout = workspace.layout.activate(tab, id); workspace.focused = id }.padding(horizontal = 7.dp, vertical = 5.dp))
                    Text("×", modifier = Modifier.clickable { workspace.close(tab) }.semantics { contentDescription = "Close ${workspace.label(tab)}" }.padding(horizontal = 6.dp, vertical = 5.dp))
                }
            }
            Spacer(Modifier.width(20.dp).height(29.dp))
        }
        HorizontalDivider()
        Box(Modifier.weight(1f).fillMaxWidth()) {
            PaneContent(workspace, node.active, id)
            PaneEdges(workspace, id)
        }
    }
}

@Composable private fun PaneEdges(workspace: Workspace, group: String) {
    var selected by remember { mutableStateOf<String?>(null) }
    val color = MaterialTheme.colorScheme.primary
    Box(Modifier.fillMaxSize()) {
        selected?.let { edge ->
            val preview = when (edge) {
                "left" -> Modifier.align(androidx.compose.ui.Alignment.CenterStart).fillMaxWidth(.5f).fillMaxHeight()
                "right" -> Modifier.align(androidx.compose.ui.Alignment.CenterEnd).fillMaxWidth(.5f).fillMaxHeight()
                "top" -> Modifier.align(androidx.compose.ui.Alignment.TopCenter).fillMaxWidth().fillMaxHeight(.5f)
                "bottom" -> Modifier.align(androidx.compose.ui.Alignment.BottomCenter).fillMaxWidth().fillMaxHeight(.5f)
                else -> Modifier.fillMaxSize()
            }
            Box(preview.background(color.copy(alpha = .18f)).border(2.dp, color))
        }
        listOf("left", "right", "top", "bottom", "center").forEach { edge ->
            val target = remember(workspace, group, edge) { object : DragAndDropTarget {
                override fun onEntered(event: DragAndDropEvent) { selected = edge }
                override fun onExited(event: DragAndDropEvent) { if (selected == edge) selected = null }
                override fun onEnded(event: DragAndDropEvent) { selected = null }
                override fun onDrop(event: DragAndDropEvent): Boolean {
                    selected = null
                    val clip = event.toAndroidDragEvent().clipData ?: return false
                    if (clip.itemCount != 1 || !clip.description.hasMimeType(TAB_MIME) || clip.description.label?.toString() != "hickory-tab") return false
                    val tab = clip.getItemAt(0).text?.toString() ?: return false
                    if (workspace.layout.groups().none { it.id == group } || workspace.layout.groups().none { tab in it.tabs }) return false
                    val id = if (edge == "center") group else java.util.UUID.randomUUID().toString()
                    val next = if (edge == "center") workspace.layout.move(tab, group) else workspace.layout.dock(tab, group, edge, id)
                    if (next === workspace.layout) return false
                    workspace.layout = next; workspace.focused = id; return true
                }
            } }
            val zone = when (edge) {
                "left" -> Modifier.align(androidx.compose.ui.Alignment.CenterStart).fillMaxWidth(.2f).fillMaxHeight()
                "right" -> Modifier.align(androidx.compose.ui.Alignment.CenterEnd).fillMaxWidth(.2f).fillMaxHeight()
                "top" -> Modifier.align(androidx.compose.ui.Alignment.TopCenter).fillMaxWidth(.6f).fillMaxHeight(.2f)
                "bottom" -> Modifier.align(androidx.compose.ui.Alignment.BottomCenter).fillMaxWidth(.6f).fillMaxHeight(.2f)
                else -> Modifier.align(androidx.compose.ui.Alignment.Center).fillMaxWidth(.6f).fillMaxHeight(.6f)
            }
            // These nodes handle platform drags only; normal editor pointer input passes through.
            Box(zone.dragAndDropTarget(shouldStartDragAndDrop = { TAB_MIME in it.mimeTypes() }, target = target))
        }
    }
}

@Composable private fun LanguageEditor(buffer: DocumentBuffer, modifier: Modifier = Modifier) {
    val scroll = rememberScrollState()
    val scope = rememberCoroutineScope()
    var menu by remember { mutableStateOf(false) }
    val color = MaterialTheme.colorScheme.primary
    val source = buffer.text
    val keywords = remember { Regex("\\b(func|fun|fn|let|val|var|const|class|struct|import|return|if|else|true|false|def)\\b") }
    val highlighting by produceState<Pair<String, List<IntRange>>>("" to emptyList(), source) {
        value = source to kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) {
            if (source.length <= 200_000) keywords.findAll(source).map { it.range }.toList() else emptyList()
        }
    }
    Box(modifier) {
        Row(Modifier.fillMaxSize()) {
            BasicTextField(state = buffer.state, readOnly = buffer.readOnly,
                scrollState = scroll,
                outputTransformation = OutputTransformation {
                    if (highlighting.first == toString()) highlighting.second.forEach { range ->
                        addStyle(SpanStyle(color = color), range.first, range.last + 1)
                    }
                },
                modifier = Modifier.weight(1f).fillMaxHeight().padding(8.dp).semantics { contentDescription = buffer.file.path },
                textStyle = TextStyle(fontFamily = FontFamily.Monospace, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurface),
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false))
            Canvas(Modifier.width(40.dp).fillMaxHeight().semantics {
                contentDescription = "Document minimap"
                progressBarRangeInfo = ProgressBarRangeInfo(if (scroll.maxValue > 0) scroll.value.toFloat() / scroll.maxValue else 0f, 0f..1f)
                setProgress { fraction -> scope.launch { scroll.scrollTo((fraction.coerceIn(0f, 1f) * scroll.maxValue).toInt()) }; true }
            }.pointerInput(scroll) { detectTapGestures { point ->
                scope.launch { scroll.scrollTo((point.y / size.height.coerceAtLeast(1) * scroll.maxValue).toInt().coerceIn(0, scroll.maxValue)) }
            } }) {
                val lines = source.lineSequence().toList()
                val rows = minOf(lines.size, maxOf(1, (size.height / 2).toInt()))
                repeat(rows) { row ->
                    drawLine(color.copy(alpha = .5f), Offset(2f, row * size.height / rows), Offset(minOf(size.width - 2, 2 + lines[row * lines.size / rows].length * 1.3f), row * size.height / rows))
                }
                val total = scroll.maxValue.toFloat() + scroll.viewportSize
                if (total > 0) drawRect(color.copy(alpha = .15f), Offset(0f, scroll.value / total * size.height), Size(size.width, maxOf(4f, scroll.viewportSize / total * size.height)))
            }
        }
        Box(Modifier.align(androidx.compose.ui.Alignment.TopEnd).padding(end = 42.dp)) {
            Text("⌘", modifier = Modifier.background(MaterialTheme.colorScheme.surface).clickable { menu = true }.padding(4.dp).semantics { contentDescription = "Language preview" })
            DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                Text("Language UI preview", modifier = Modifier.padding(8.dp))
                Text("Local colors and document words; native LSP is not connected.", fontSize = 11.sp, modifier = Modifier.width(250.dp).padding(8.dp))
                source.split(Regex("[^A-Za-z0-9_]+")).asSequence().filter { it.length > 2 }.distinct().sorted().take(30).forEach { word ->
                    DropdownMenuItem(text = { Text(word) }, enabled = !buffer.readOnly, onClick = {
                        buffer.state.edit { replace(selection.min, selection.max, word) }; menu = false
                    })
                }
            }
        }
    }
}
@Composable private fun PaneContent(workspace: Workspace, tab: String?, group: String) {
    when (tab) {
        "files" -> Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).horizontalScroll(rememberScrollState()).padding(5.dp)) { FileRows(workspace.files, workspace) }
        "terminal" -> TerminalPane(workspace)
        "agent" -> AgentPane(workspace.engine, workspace.agent, workspace)
        null -> Text("Open a file or move a tab here", modifier = Modifier.padding(16.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
        else -> workspace.buffers[tab]?.let { buffer ->
            Column(Modifier.fillMaxSize()) {
                LanguageEditor(buffer, Modifier.weight(1f).fillMaxWidth().onFocusChanged { if (it.isFocused) workspace.focused = group })
                if (buffer.error.isNotEmpty()) Text(buffer.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp, modifier = Modifier.padding(6.dp))
            }
        }
    }
}
@Composable private fun FileRows(nodes: List<FileNode>, workspace: Workspace) {
    val scope = rememberCoroutineScope()
    nodes.forEach { node ->
        key(node.path) {
            var expanded by remember { mutableStateOf(false) }
            if (node.dir) {
                Text((if (expanded) "▾ " else "▸ ") + node.name, fontSize = 13.sp, modifier = Modifier.clickable { expanded = !expanded }.padding(vertical = 4.dp))
                if (expanded) Column(Modifier.padding(start = 12.dp)) { FileRows(node.children, workspace) }
            } else Text(node.name, fontSize = 13.sp, modifier = Modifier.clickable { scope.launch { workspace.open(node) } }.padding(vertical = 4.dp))
        }
    }
}
