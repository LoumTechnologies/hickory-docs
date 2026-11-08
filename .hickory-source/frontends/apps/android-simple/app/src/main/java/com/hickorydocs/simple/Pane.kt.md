<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Render tabs and recursive splits in Compose

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Tab strips are drag sources and targets. Their fixed height prevents content reflow. A remembered drop target consults the current workspace instead of capturing a stale layout snapshot.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Browser capabilities use the existing LSP client and its snapshot guards; native keyword and word-list previews deliberately do not claim semantic results. Native styling does not replace text, preserving the platform undo and selection model. Preview coloring skips very large files instead of delaying readable text indefinitely.

A custom tab MIME type keeps a center drop from being treated as text insertion by the editor. Content-edge nodes accept platform drags without installing pointer gestures over the editor. Left/right strips own the corners, matching the shared normalized edge rule. A half-pane preview is drawn behind the drop zones only while targeted. Docking revalidates current layout membership and focuses the newly created group only on success.

Tab and strip drop targets share one validator and insertion command. A tab supplies an insert-before anchor; the strip supplies no anchor and appends. Target callbacks draw an overlay marker without changing widths. RememberUpdatedState keeps platform drag callbacks attached to the current composition. Only internally labeled tab drags with a live tab identity are accepted.

Reserve a small blank slot at the end of the strip even when tabs overflow, so append remains reachable by horizontal scrolling. It is always present and does not resize during a drag.

<ui:copy id="simple-0d1e071614-0">
package com.hickorydocs.simple

import android.graphics.Paint
import android.graphics.Typeface
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
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
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.draganddrop.*
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
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

</ui:copy>

## Dock a tab at a pane edge

The four edge drop targets use the same twenty-percent regions as the shared classifier. The central region appends to the existing group without adding a split, and previews the whole pane. Their nodes handle drag events, without pointer gestures that would consume normal editing. The shaded half is a preview rather than a new child in the layout. On drop, validate the internally labeled tab against the current layout before creating or focusing the new group.

<ui:copy id="simple-android-pane-edges">
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

</ui:copy>

## Attach display capabilities to the retained text field

Compute keyword ranges on a background dispatcher and tag them with their source snapshot. OutputTransformation paints only matching results and never changes backing text. Document-word suggestions are an explicit local preview. Selection replacement uses TextFieldState.edit, while minimap navigation changes only its retained scroll state. Neither capability grants permission to edit generated files.

The left line-number canvas reads the text field's actual layout and scroll state, so wrapped continuations do not receive a second number. Draw at the measured baseline with the same font size and top padding. A stale layout is ignored until the current text is measured; the editable text remains available. The gutter reserves three digits and grows for larger documents without mounting a second scrolling view.

<ui:copy id="simple-android-language-preview">
@Composable private fun LanguageEditor(buffer: DocumentBuffer, debug: NativeDebugger?, modifier: Modifier = Modifier) {
    val scroll = rememberScrollState()
    val scope = rememberCoroutineScope()
    var menu by remember { mutableStateOf(false) }
    val color = MaterialTheme.colorScheme.primary
    val source = buffer.text
    var textLayout by remember(buffer) { mutableStateOf<TextLayoutResult?>(null) }
    val paused = if (debug?.sourceMatches(buffer) == true) debug.line(buffer.file.path) else null
    val pauseColor = MaterialTheme.colorScheme.primary.copy(alpha = .14f)
    LaunchedEffect(buffer.debugReveal, textLayout) {
        val request = buffer.debugReveal ?: return@LaunchedEffect
        val layout = textLayout ?: return@LaunchedEffect
        val offset = debugLineOffset(source, request.first) ?: return@LaunchedEffect
        if (layout.layoutInput.text.text == source) scroll.scrollTo(layout.getLineTop(layout.getLineForOffset(offset)).toInt().coerceAtLeast(0))
    }
    val keywords = remember { Regex("\\b(func|fun|fn|let|val|var|const|class|struct|import|return|if|else|true|false|def)\\b") }
    val highlighting by produceState<Pair<String, List<IntRange>>>("" to emptyList(), source) {
        value = source to kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) {
            if (source.length <= 200_000) keywords.findAll(source).map { it.range }.toList() else emptyList()
        }
    }
    Box(modifier) {
        Row(Modifier.fillMaxSize()) {
            val numberColor = MaterialTheme.colorScheme.onSurfaceVariant
            val numberPaint = remember { Paint(Paint.ANTI_ALIAS_FLAG).apply {
                typeface = Typeface.MONOSPACE; textAlign = Paint.Align.RIGHT
            } }
            Canvas(Modifier.width((maxOf(3, (source.count { it == '\n' } + 1).toString().length) * 8 + 12).dp)
                .fillMaxHeight().clipToBounds()) {
                val layout = textLayout
                if (layout != null && layout.layoutInput.text.text == source) {
                    numberPaint.color = numberColor.toArgb(); numberPaint.textSize = 13.sp.toPx()
                    val first = layout.getLineForVerticalPosition((scroll.value - 8.dp.toPx()).coerceAtLeast(0f))
                    val last = layout.getLineForVerticalPosition(scroll.value + size.height)
                    // Wrapped rows share a logical number; only draw at the logical start.
                    for (visual in first..last) {
                        val offset = layout.getLineStart(visual)
                        if (offset == 0 || source[offset - 1] == '\n') {
                            val logical = source.take(offset).count { it == '\n' } + 1
                            val baseline = layout.getLineBaseline(visual) - scroll.value + 8.dp.toPx()
                            drawContext.canvas.nativeCanvas.drawText(logical.toString(), size.width - 6.dp.toPx(), baseline, numberPaint)
                        }
                    }
                }
            }
            BasicTextField(state = buffer.state, readOnly = buffer.readOnly,
                scrollState = scroll,
                onTextLayout = { getResult -> textLayout = getResult() },
                outputTransformation = OutputTransformation {
                    if (paused != null) {
                        val start = debugLineOffset(source, paused)
                        if (start != null && toString() == source) addStyle(SpanStyle(background = pauseColor), start, source.indexOf('\n', start).let { if (it < 0) source.length else it })
                    }
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
            if (debug != null) NativeDebugRail(buffer, debug, textLayout, scroll)
        }
        Box(Modifier.align(androidx.compose.ui.Alignment.TopEnd).padding(end = if (debug != null) 66.dp else 42.dp)) {
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
</ui:copy>

Compose text state is owned by the document buffer rather than remembered inside a tab. File folders expand in place. A failed save is visible in its document while the original text remains editable.

<ui:copy id="simple-0d1e071614-1">
@Composable private fun PaneContent(workspace: Workspace, tab: String?, group: String) {
    when (tab) {
        "files" -> Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).horizontalScroll(rememberScrollState()).padding(5.dp)) { FileRows(workspace.files, workspace) }
        "terminal" -> TerminalPane(workspace)
        "agent" -> AgentPane(workspace.engine, workspace.agent, workspace)
        null -> Text("Open a file or move a tab here", modifier = Modifier.padding(16.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
        else -> workspace.buffers[tab]?.let { buffer ->
            Column(Modifier.fillMaxSize()) {
                LanguageEditor(buffer, workspace.debugger, Modifier.weight(1f).fillMaxWidth().onFocusChanged { if (it.isFocused) workspace.focused = group })
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
</ui:copy>

## Measure the rail from the retained Compose editor

BasicTextField reports its real text layout after readable content exists. We use that result and the existing scroll state to position one action per visible logical line, ignoring wrapped continuation rows. A ⋯ popover holds a grid of debug controls instead of stacking icons over later lines. Paused highlighting is an output transformation and values float beside the rail; neither changes the TextFieldState, selection or undo. Source changes hide positional observations until the run and displayed source agree.

<ui:copy id="simple-android-debug-rail">
fun debugLineOffset(source: String, line: Int): Int? {
    if (line < 0) return null
    var offset = 0
    repeat(line) { val next = source.indexOf('\n', offset); if (next < 0) return null; offset = next + 1 }
    return offset
}
@OptIn(ExperimentalLayoutApi::class)
@Composable private fun NativeDebugRail(buffer: DocumentBuffer, debug: NativeDebugger, layout: TextLayoutResult?, scroll: ScrollState) {
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    var opened by remember(buffer) { mutableStateOf<Int?>(null) }
    var expression by remember(buffer) { mutableStateOf("") }
    var condition by remember(buffer) { mutableStateOf("") }
    val paused = if (debug.sourceMatches(buffer)) debug.line(buffer.file.path) else null
    BoxWithConstraints(Modifier.width(24.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surfaceContainer)) {
        val height = with(density) { maxHeight.toPx() }
        if (layout != null && layout.layoutInput.text.text == buffer.text) {
            val first = layout.getLineForVerticalPosition(scroll.value.toFloat())
            val last = layout.getLineForVerticalPosition(scroll.value + height)
            for (visual in first..last) {
                val offset = layout.getLineStart(visual)
                if (offset > 0 && buffer.text[offset - 1] != '\n') continue
                val line = buffer.text.take(offset).count { it == '\n' }
                val top = layout.getLineTop(visual) - scroll.value + with(density) { 8.dp.toPx() }
                if (top < 0 || top > height) continue
                val held = debug.breakpoints[buffer.file.path]?.firstOrNull { it.optInt("line", -1) == line }
                val glyph = when { line == paused -> "⋯"; line == 0 -> "▷"; held == null -> "·"; held.optString("state") == "refused" -> "○"; held.optString("state") == "bound" -> "●"; else -> "◌" }
                Box(Modifier.offset { IntOffset(0, top.toInt()) }) {
                    Text(glyph, fontSize = 13.sp, modifier = Modifier.width(24.dp).height(18.dp).clickable { opened = line }.semantics { contentDescription = "Debug actions on line ${line + 1}" })
                    DropdownMenu(expanded = opened == line, onDismissRequest = { opened = null }) {
                        Column(Modifier.width(300.dp).heightIn(max = 460.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
                            Text("Line ${line + 1} · ${debug.status}", fontSize = 12.sp)
                            if (!debug.sourceMatches(buffer)) Text("Save edited text before launching. Values belong to the source that ran.", fontSize = 12.sp)
                            FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                TextButton(enabled = !debug.active && !buffer.dirty && !buffer.saving, onClick = { scope.launch { debug.prepare(buffer, line) } }) { Text("Debug") }
                                TextButton(onClick = { debug.toggle(buffer.file.path, line) }) { Text("Breakpoint") }
                                if (debug.status == "paused") {
                                    listOf("Continue" to "continue", "Step over" to "over", "Step in" to "in", "Step out" to "out").forEach { (label, how) -> TextButton(onClick = { debug.step(how) }) { Text(label) } }
                                    if (debug.capabilities.optBoolean("step_back")) TextButton(onClick = { debug.step("back") }) { Text("Step back") }
                                }
                                if (debug.active) TextButton(onClick = { debug.stop() }) { Text("Stop") }
                            }
                            if (debug.owner == buffer.file.path) debug.targets.forEach { target -> TextButton(onClick = { scope.launch { debug.prepare(buffer, program = target.path) } }) { Text("Debug file ${target.path}") } }
                            debug.profiles.forEach { profile -> TextButton(onClick = { debug.launch(profile); opened = null }) { Text("Debug ${profile.optString("name")}") } }
                            if (debug.offer != null) TextButton(onClick = { scope.launch { debug.install() } }) { Text("Install debugger") }
                            if (debug.message.isNotBlank()) Text(debug.message, fontSize = 12.sp)
                            if (debug.capabilities.optBoolean("conditional_breakpoints")) {
                                OutlinedTextField(value = condition, onValueChange = { condition = it }, label = { Text("Breakpoint condition") })
                                TextButton(onClick = { debug.toggle(buffer.file.path, line, condition) }) { Text("Apply condition") }
                            }
                            if (debug.status == "paused") {
                                OutlinedTextField(value = expression, onValueChange = { expression = it }, label = { Text("Expression") })
                                Row { TextButton(onClick = { debug.evaluate(expression) }) { Text("Evaluate") }; TextButton(onClick = { debug.evaluate(expression, true) }) { Text("Watch") } }
                                debug.frames.forEach { frame -> TextButton(onClick = { debug.select(frame) }) { Text(frame.optString("name", "Frame")) } }
                                debug.variables.forEach { variable -> Text("${variable.optString("name")} = ${variable.optString("value")}", fontSize = 12.sp) }
                                debug.values.forEach { (key, value) -> Row { Text("$key = $value", modifier = Modifier.weight(1f)); if (key in debug.watches) TextButton(onClick = { debug.removeWatch(key) }) { Text("×") } } }
                            }
                        }
                    }
                    if (line == paused && debug.variables.isNotEmpty()) {
                        Text(debug.variables.take(3).joinToString("  ") { "${it.optString("name")} = ${it.optString("value")}" },
                            fontSize = 11.sp, fontFamily = FontFamily.Monospace, maxLines = 1,
                            modifier = Modifier.offset(x = (-224).dp).width(220.dp).background(MaterialTheme.colorScheme.surfaceContainer).padding(3.dp).semantics { contentDescription = "Paused values" })
                    }
                }
            }
        }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Pane.kt"><ui:paste select="#simple-0d1e071614-0"/><ui:paste select="#simple-android-pane-edges"/><ui:paste select="#simple-android-language-preview"/><ui:paste select="#simple-0d1e071614-1"/><ui:paste select="#simple-android-debug-rail"/></ui:file>

</ui:doc>
