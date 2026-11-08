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
import androidx.compose.foundation.text.input.InputTransformation
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
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.Alignment
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.font.FontStyle
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

data class NativeSourceEdit(val from: Int, val to: Int, val text: String)
data class NativeSearchRequest(val source: String, val query: String, val replacement: String, val caseSensitive: Boolean, val regex: Boolean)
data class NativeSearchResult(val edits: List<NativeSourceEdit> = emptyList(), val error: String = "")
fun nativeSearchReplacement(template: String, groups: List<String>): String = Regex("\\$([$&]|\\d+)").replace(template) { token ->
    val key = token.groupValues[1]
    when (key) {
        "$" -> "$"
        "&" -> groups.firstOrNull() ?: ""
        else -> (key.length downTo 1).firstNotNullOfOrNull { length ->
            key.take(length).toIntOrNull()?.takeIf { it > 0 && it < groups.size }?.let { groups[it] + key.drop(length) }
        } ?: token.value
    }
}
fun nativeSearch(request: NativeSearchRequest): NativeSearchResult {
    if (request.query.isEmpty()) return NativeSearchResult()
    return try {
        val options = if (request.caseSensitive) setOf(RegexOption.MULTILINE) else setOf(RegexOption.MULTILINE, RegexOption.IGNORE_CASE)
        val expression = Regex(if (request.regex) request.query else Regex.escape(request.query), options)
        val edits = mutableListOf<NativeSourceEdit>()
        for (match in expression.findAll(request.source)) {
            if (match.value.isEmpty()) continue
            if (edits.size >= 10_000) return NativeSearchResult(error = "Too many matches. Narrow your search before replacing.")
            edits.add(NativeSourceEdit(match.range.first, match.range.last + 1, if (request.regex) nativeSearchReplacement(request.replacement, match.groupValues) else request.replacement))
        }
        NativeSearchResult(edits)
    } catch (_: java.util.regex.PatternSyntaxException) { NativeSearchResult(error = "Invalid regular expression. Check the pattern and try again.") }
}
fun validSourceEdits(edits: List<NativeSourceEdit>, length: Int): Boolean {
    var end = 0
    for (edit in edits.sortedBy { it.from }) {
        if (edit.from < end || edit.to < edit.from || edit.to > length) return false
        end = edit.to
    }
    return edits.isNotEmpty()
}
@Composable private fun NativeFindPanel(buffer: DocumentBuffer, modifier: Modifier = Modifier) {
    val request = NativeSearchRequest(buffer.text, buffer.findQuery, buffer.findReplacement, buffer.findCase, buffer.findRegex)
    var measured by remember { mutableStateOf<NativeSearchRequest?>(null) }
    var result by remember { mutableStateOf(NativeSearchResult()) }
    var notice by remember { mutableStateOf("") }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    LaunchedEffect(request) {
        measured = null; notice = ""
        kotlinx.coroutines.delay(100)
        val found = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) { nativeSearch(request) }
        result = found; measured = request
    }
    val current = measured == request && result.error.isEmpty()
    fun move(backwards: Boolean = false) {
        if (!current || result.edits.isEmpty()) return
        val selection = buffer.state.selection
        val match = if (backwards) result.edits.lastOrNull { it.to <= selection.min } ?: result.edits.last()
            else result.edits.firstOrNull { it.from >= selection.max } ?: result.edits.first()
        if (!buffer.selectSourceRange(match.from, match.to, request.source)) notice = "Text changed or input is composing. Finish typing and search again."
    }
    fun replace(all: Boolean) {
        if (!current || buffer.readOnly) return
        val selection = buffer.state.selection
        val edits = if (all) result.edits else result.edits.filter { it.from == selection.min && it.to == selection.max }
        if (edits.isEmpty()) { move(); return }
        if (!buffer.applySourceEdits(edits, request.source)) notice = "Replacement refused. Finish typing and search the current text again."
    }
    Surface(modifier.widthIn(max = 360.dp).heightIn(max = 360.dp), tonalElevation = 4.dp, shadowElevation = 4.dp) {
        Column(Modifier.verticalScroll(rememberScrollState()).padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Find and replace", modifier = Modifier.weight(1f), fontSize = 12.sp)
                TextButton(onClick = { buffer.closeFind() }) { Text("Close search") }
            }
            OutlinedTextField(buffer.findQuery, { buffer.findQuery = it }, label = { Text("Find") }, singleLine = true, modifier = Modifier.fillMaxWidth().focusRequester(focus))
            OutlinedTextField(buffer.findReplacement, { buffer.findReplacement = it }, label = { Text("Replace with") }, singleLine = true, enabled = !buffer.readOnly, modifier = Modifier.fillMaxWidth())
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(buffer.findCase, { buffer.findCase = it }); Text("Match case", fontSize = 12.sp)
                Checkbox(buffer.findRegex, { buffer.findRegex = it }); Text("Regex", fontSize = 12.sp)
            }
            Row {
                TextButton(enabled = current && result.edits.isNotEmpty(), onClick = { move(true) }) { Text("Previous") }
                TextButton(enabled = current && result.edits.isNotEmpty(), onClick = { move() }) { Text("Next") }
            }
            Row {
                TextButton(enabled = current && result.edits.isNotEmpty() && !buffer.readOnly, onClick = { replace(false) }) { Text("Replace") }
                TextButton(enabled = current && result.edits.isNotEmpty() && !buffer.readOnly, onClick = { replace(true) }) { Text("Replace all") }
            }
            TextButton(enabled = current && result.edits.size > 1, onClick = {
                if (buffer.selectMultiple(result.edits, request.source)) buffer.closeFind() else notice = "Select between 2 and 256 current matches."
            }) { Text("Select all matches") }
            Text(if (notice.isNotEmpty()) notice else if (measured != request) "Searching…" else if (result.error.isNotEmpty()) result.error else "${result.edits.size} matches", fontSize = 12.sp, modifier = Modifier.semantics { contentDescription = "Search status" })
            if (buffer.readOnly) Text("Read-only · searching is available", fontSize = 12.sp)
        }
    }
}

data class SimpleFold(val line: Int, val from: Int, val to: Int)
fun simpleFoldRanges(source: String): List<SimpleFold> {
    data class Header(val line: Int, val indent: Int, val end: Int)
    val rows = source.split('\n'); val stack = mutableListOf<Header>(); val result = mutableListOf<SimpleFold>()
    var offset = 0; var last = -1
    for (line in 0..rows.size) {
        val row = rows.getOrNull(line)
        if (row != null && row.isBlank()) { offset += row.length + 1; continue }
        val indent = row?.takeWhile { it == ' ' || it == '\t' }?.sumOf { if (it == '\t') 4 else 1 } ?: -1
        while (stack.isNotEmpty() && stack.last().indent >= indent) {
            val header = stack.removeAt(stack.lastIndex)
            if (last > header.end) result.add(SimpleFold(header.line, header.end, last))
        }
        if (row != null) { last = offset + row.length; stack.add(Header(line, indent, last)); offset = last + 1 }
    }
    return result.sortedBy { it.from }
}
// Only outermost folds change displayed offsets; nested state survives unfolding its parent.
fun outerFolds(ranges: List<SimpleFold>): List<SimpleFold> {
    var end = -1
    return ranges.sortedBy { it.from }.filter { if (it.from >= end) { end = it.to; true } else false }
}
fun foldDisplay(source: String, ranges: List<SimpleFold>): String {
    if (ranges.isEmpty()) return source
    val result = StringBuilder(source)
    outerFolds(ranges).asReversed().forEach { result.replace(it.from, it.to, " …") }
    return result.toString()
}
fun foldSourceOffset(displayOffset: Int, ranges: List<SimpleFold>): Int {
    var removed = 0
    for (range in outerFolds(ranges)) {
        val start = range.from - removed
        if (displayOffset <= start) break
        if (displayOffset < start + 2) return range.from
        removed += range.to - range.from - 2
    }
    return displayOffset + removed
}

@OptIn(ExperimentalFoundationApi::class)
@Composable private fun LanguageEditor(buffer: DocumentBuffer, debug: NativeDebugger?, modifier: Modifier = Modifier) {
    val editorFocus = remember(buffer) { FocusRequester() }
    DisposableEffect(buffer, editorFocus) {
        val focus: () -> Unit = { editorFocus.requestFocus(); Unit }
        buffer.focusSource = focus
        onDispose { if (buffer.focusSource === focus) buffer.focusSource = null }
    }
    val scroll = rememberScrollState()
    val scope = rememberCoroutineScope()
    var menu by remember { mutableStateOf(false) }
    val color = MaterialTheme.colorScheme.primary
    val source = buffer.text
    val service = buffer.language
    val semantic = service?.tokens.orEmpty()
    val semanticSource = service?.tokenSource
    val inline = buffer.inlineMarkup
    LaunchedEffect(buffer, source) {
        val result = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) { computeInlineMarkup(source, buffer.file.path) }
        if (buffer.text == source) buffer.inlineMarkup = result
    }
    val selection = buffer.state.selection
    val folds = buffer.activeFolds()
    val display = foldDisplay(source, folds)
    var textLayout by remember(buffer) { mutableStateOf<TextLayoutResult?>(null) }
    val paused = if (debug?.sourceMatches(buffer) == true) debug.line(buffer.file.path) else null
    val pauseColor = MaterialTheme.colorScheme.primary.copy(alpha = .14f)
    LaunchedEffect(buffer.debugReveal) { if (buffer.debugReveal != null) buffer.clearFolds() }
    LaunchedEffect(buffer.debugReveal, textLayout) {
        val request = buffer.debugReveal ?: return@LaunchedEffect
        val layout = textLayout ?: return@LaunchedEffect
        val offset = debugLineOffset(source, request.first) ?: return@LaunchedEffect
        if (layout.layoutInput.text.text == source && buffer.text == source) {
            scroll.scrollTo(layout.getLineTop(layout.getLineForOffset(offset)).toInt().coerceAtLeast(0))
            if (buffer.debugReveal == request) buffer.debugReveal = null
        }
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
                if (layout != null && layout.layoutInput.text.text == display) {
                    numberPaint.color = numberColor.toArgb(); numberPaint.textSize = 13.sp.toPx()
                    val first = layout.getLineForVerticalPosition((scroll.value - 8.dp.toPx()).coerceAtLeast(0f))
                    val last = layout.getLineForVerticalPosition(scroll.value + size.height)
                    // Wrapped rows share a logical number; only draw at the logical start.
                    for (visual in first..last) {
                        val offset = foldSourceOffset(layout.getLineStart(visual), folds)
                        if (offset == 0 || source[offset - 1] == '\n') {
                            val logical = (if (inline.source == source) inlineLineAt(offset, inline.lineStarts) else source.take(offset).count { it == '\n' }) + 1
                            val baseline = layout.getLineBaseline(visual) - scroll.value + 8.dp.toPx()
                            drawContext.canvas.nativeCanvas.drawText(logical.toString(), size.width - 6.dp.toPx(), baseline, numberPaint)
                        }
                    }
                }
            }
            BasicTextField(state = buffer.state, readOnly = buffer.readOnly,
                scrollState = scroll,
                inputTransformation = InputTransformation {
                    val hiddenEdit = (0 until changes.changeCount).any { index ->
                        val changed = changes.getOriginalRange(index)
                        folds.any { range -> changed.min < range.to && changed.max > range.from || changed.collapsed && changed.min in range.from until range.to }
                    }
                    if (hiddenEdit) revertAllChanges()
                    // Retire display state after the input transaction, outside its receiver.
                    scope.launch { buffer.clearFolds(); if (!buffer.applyingMulti) buffer.clearMulti() }
                },
                onTextLayout = { getResult -> textLayout = getResult() },
                outputTransformation = OutputTransformation {
                    if (paused != null) {
                        val start = debugLineOffset(source, paused)
                        if (start != null && toString() == source) addStyle(SpanStyle(background = pauseColor), start, source.indexOf('\n', start).let { if (it < 0) source.length else it })
                    }
                    if (highlighting.first == toString()) highlighting.second.forEach { range ->
                        addStyle(SpanStyle(color = color), range.first, range.last + 1)
                    }
                    if (semanticSource == toString()) semantic.forEach { token -> addStyle(SpanStyle(color = color), token.from, token.to) }
                    if (inline.source == toString()) inline.marks.forEach { mark ->
                        val style = when (mark.kind) {
                            "heading" -> SpanStyle(fontSize = maxOf(14, 25 - mark.level * 2).sp, fontWeight = FontWeight.SemiBold)
                            "strong" -> SpanStyle(fontWeight = FontWeight.Bold)
                            "em", "math" -> SpanStyle(fontStyle = FontStyle.Italic)
                            else -> SpanStyle(fontFamily = FontFamily.Monospace)
                        }
                        addStyle(style, mark.from, mark.to)
                    }
                    if (toString() == source) outerFolds(folds).asReversed().forEach { replace(it.from, it.to, " …") }
                },
                modifier = Modifier.weight(1f).fillMaxHeight().padding(8.dp).focusRequester(editorFocus).onPreviewKeyEvent { event ->
                    val key = event.nativeKeyEvent
                    if (event.type == KeyEventType.KeyDown && event.key == Key.Spacebar && event.isCtrlPressed) {
                        menu = true; scope.launch { service?.inspect() }; scope.launch { service?.complete() }; true
                    } else if (buffer.multiSource == source && buffer.multiRanges.size > 1 && event.type == KeyEventType.KeyDown && !event.isCtrlPressed && !event.isMetaPressed && !event.isAltPressed && key.device?.isVirtual == false && key.unicodeChar > 0 && buffer.state.composition == null) {
                        buffer.replaceMultiple(String(Character.toChars(key.unicodeChar))); true
                    } else false
                }.drawWithContent {
                    drawContent()
                    val layout = textLayout
                    if (buffer.multiSource == source && layout?.layoutInput?.text?.text == source) buffer.multiRanges.drop(1).forEach { range ->
                        if (range.from == range.to) { val rect = layout.getCursorRect(range.from); drawLine(color, Offset(rect.left, rect.top - scroll.value), Offset(rect.left, rect.bottom - scroll.value), 2f) }
                        else { val path = layout.getPathForRange(range.from, range.to); path.translate(Offset(0f, -scroll.value.toFloat())); drawPath(path, color.copy(alpha = .2f)) }
                    }
                }.clipToBounds().semantics { contentDescription = buffer.file.path },
                textStyle = TextStyle(fontFamily = FontFamily.Monospace, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurface),
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false))
            Canvas(Modifier.width(40.dp).fillMaxHeight().semantics {
                contentDescription = "Document minimap"
                progressBarRangeInfo = ProgressBarRangeInfo(if (scroll.maxValue > 0) scroll.value.toFloat() / scroll.maxValue else 0f, 0f..1f)
                setProgress { fraction -> buffer.clearFolds(); scope.launch { scroll.scrollTo((fraction.coerceIn(0f, 1f) * scroll.maxValue).toInt()) }; true }
            }.pointerInput(scroll) { detectTapGestures { point ->
                buffer.clearFolds()
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
            Text("⌘", modifier = Modifier.background(MaterialTheme.colorScheme.surface).clickable {
                menu = true; scope.launch { service?.inspect() }; scope.launch { service?.complete() }
            }.padding(4.dp).semantics { contentDescription = "Language tools" })
            DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                Column(Modifier.width(320.dp).heightIn(max = 460.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
                    Text("Language tools", fontSize = 14.sp)
                    Text(service?.status ?: "Language services not connected. Reconnect the workspace.", fontSize = 11.sp)
                    if (service?.inspectionSource == source && service.inspectionSelection == buffer.state.selection) {
                        if (service.hover.isNotEmpty()) Text(service.hover, fontSize = 12.sp)
                        if (service.signature.isNotEmpty()) Text(service.signature, fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                        service.inlays.take(50).forEach { Text(it, fontSize = 11.sp) }
                    }
                    if (service?.diagnosticSource == source) service.diagnostics.forEach { item ->
                        if (item.from != null && item.to != null) TextButton(onClick = { buffer.selectSourceRange(item.from, item.to, source) }) { Text(item.message) }
                        else Text(item.message + " (server message; source version unavailable)", fontSize = 12.sp)
                    }
                    service?.suggestions?.forEach { item ->
                        TextButton(enabled = !buffer.readOnly, onClick = { if (service.accept(item)) menu = false }) {
                            Column { Text(item.label, fontFamily = FontFamily.Monospace); Text(item.detail, fontSize = 11.sp) }
                        }
                    }
                    TextButton(onClick = { scope.launch { service?.inspect() }; scope.launch { service?.complete() } }) { Text("Refresh language tools") }
                    TextButton(onClick = { service?.start() }) { Text("Retry language services") }
                }
            }
        }
        if (buffer.multiSource == source && buffer.multiRanges.size > 1) Row(Modifier.align(Alignment.TopStart).background(MaterialTheme.colorScheme.surface)) {
            TextButton(onClick = { buffer.showMulti = true }) { Text("${buffer.multiRanges.size} selections") }
            TextButton(onClick = { buffer.clearMulti() }) { Text("Clear selections") }
        }
        if (buffer.showMulti) AlertDialog(onDismissRequest = { buffer.showMulti = false }, title = { Text("Edit selections") }, text = {
            OutlinedTextField(buffer.multiText, { buffer.multiText = it }, label = { Text("Insert at selections") })
        }, confirmButton = { TextButton(enabled = !buffer.readOnly, onClick = { buffer.replaceMultiple(buffer.multiText); buffer.showMulti = false; buffer.focusSource?.invoke() }) { Text("Apply to selections") } }, dismissButton = { TextButton(onClick = { buffer.showMulti = false }) { Text("Cancel") } })
        if (buffer.showFind) NativeFindPanel(buffer, Modifier.align(Alignment.TopEnd).padding(4.dp).widthIn(max = 360.dp).fillMaxWidth())
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
    val source = buffer.text
    val starts = buffer.inlineMarkup.takeIf { it.source == source }?.lineStarts
    val folds = buffer.activeFolds()
    val display = foldDisplay(buffer.text, folds)
    val candidates = simpleFoldRanges(buffer.text).associateBy { it.line }
    val paused = if (debug.sourceMatches(buffer)) debug.line(buffer.file.path) else null
    BoxWithConstraints(Modifier.width(24.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surfaceContainer)) {
        val height = with(density) { maxHeight.toPx() }
        if (layout != null && layout.layoutInput.text.text == display) {
            val first = layout.getLineForVerticalPosition(scroll.value.toFloat())
            val last = layout.getLineForVerticalPosition(scroll.value + height)
            for (visual in first..last) {
                val offset = foldSourceOffset(layout.getLineStart(visual), folds)
                if (offset > 0 && buffer.text[offset - 1] != '\n') continue
                val line = starts?.let { inlineLineAt(offset, it) } ?: source.take(offset).count { it == '\n' }
                val top = layout.getLineTop(visual) - scroll.value + with(density) { 8.dp.toPx() }
                if (top < 0 || top > height) continue
                val held = debug.breakpoints[buffer.file.path]?.firstOrNull { it.optInt("line", -1) == line }
                val glyph = when { line in candidates || buffer.inlineMarkup.source == buffer.text && (buffer.inlineMarkup.cells.any { it.line == line } || buffer.inlineMarkup.equations.any { it.line == line }) -> "⋯"; line == paused -> "⋯"; line == 0 -> "▷"; held == null -> "·"; held.optString("state") == "refused" -> "○"; held.optString("state") == "bound" -> "●"; else -> "◌" }
                Box(Modifier.offset { IntOffset(0, top.toInt()) }) {
                    Text(glyph, fontSize = 13.sp, modifier = Modifier.width(24.dp).height(18.dp).clickable { opened = line; scope.launch { buffer.refreshReceipts() } }.semantics { contentDescription = "Debug actions on line ${line + 1}" })
                    DropdownMenu(expanded = opened == line, onDismissRequest = { opened = null }) {
                        Column(Modifier.width(300.dp).heightIn(max = 460.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
                            Text("Line ${line + 1} · ${debug.status}", fontSize = 12.sp)
                            if (buffer.inlineMarkup.source == buffer.text) buffer.inlineMarkup.equations.filter { it.line == line }.take(4).forEach { equation ->
                                key(buffer.text, equation.from) {
                                    var draft by remember { mutableStateOf(equation.body) }
                                    NativeMathView(draft, Modifier.fillMaxWidth().height(120.dp))
                                    OutlinedTextField(draft, { draft = it }, label = { Text("LaTeX") }, enabled = !buffer.readOnly)
                                    TextButton(enabled = !buffer.readOnly && draft != equation.body, onClick = {
                                        buffer.applySourceEdits(listOf(NativeSourceEdit(equation.from, equation.to, draft)), buffer.inlineMarkup.source)
                                    }) { Text("Apply equation") }
                                }
                            }
                            if (buffer.inlineMarkup.source == buffer.text && (buffer.inlineMarkup.cells.count { it.line == line } > 32 || buffer.inlineMarkup.equations.count { it.line == line } > 4)) Text("More items remain editable in source.", fontSize = 12.sp)
                            if (buffer.inlineMarkup.source == buffer.text) buffer.inlineMarkup.cells.filter { it.line == line }.take(32).forEach { cell ->
                                key(buffer.text, cell.from) {
                                    var draft by remember { mutableStateOf(cell.value) }
                                    var error by remember { mutableStateOf("") }
                                    OutlinedTextField(draft, { draft = it }, label = { Text("Column ${cell.column + 1}") }, enabled = !buffer.readOnly)
                                    TextButton(enabled = !buffer.readOnly && draft != cell.value, onClick = {
                                        if (!buffer.applySourceEdits(listOf(NativeSourceEdit(cell.from, cell.to, inlineCellText(draft, cell.format, cell.delimiter))), buffer.inlineMarkup.source)) error = "Table changed. Reopen this row and try again."
                                    }) { Text("Apply cell") }
                                    if (error.isNotEmpty()) Text(error, fontSize = 12.sp)
                                }
                            }
                            buffer.receiptsOn(line).forEach { receipt ->
                                Text("Recorded output · " + receipt.optString("status", "unrecorded"), fontSize = 12.sp)
                                Text(receipt.optString("command"), fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                                val transcript = receipt.optJSONArray("transcript") ?: org.json.JSONArray()
                                val output = (0 until transcript.length()).joinToString("") { transcript.getJSONObject(it).optString("data") }
                                Text(output.take(50_000), fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                                if (output.length > 50_000) Text("Output truncated in this control.", fontSize = 11.sp)
                            }
                            if (buffer.receiptNotice.isNotEmpty()) Text(buffer.receiptNotice, fontSize = 12.sp)
                            if (buffer.file.docID != null) TextButton(onClick = { scope.launch { buffer.refreshReceipts() } }) { Text("Refresh recorded output") }
                            if (!debug.sourceMatches(buffer)) Text("Save edited text before launching. Values belong to the source that ran.", fontSize = 12.sp)
                            FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                if (line in candidates) TextButton(onClick = { buffer.toggleFold(line); opened = null }) { Text(if (buffer.foldedLines.contains(line)) "Unfold" else "Fold") }
                                if (folds.isNotEmpty()) TextButton(onClick = { buffer.clearFolds(); opened = null }) { Text("Unfold all") }
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
