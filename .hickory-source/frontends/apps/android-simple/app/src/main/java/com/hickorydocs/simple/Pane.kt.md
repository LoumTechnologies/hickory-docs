<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Render tabs and recursive splits in Compose

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Tab strips are drag sources and targets. Their fixed height prevents content reflow. A remembered drop target consults the current workspace instead of capturing a stale layout snapshot.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Browser capabilities use the existing LSP client and its snapshot guards; native semantic results use the real backend channel, with a labeled local preview when unavailable. Native styling does not replace text, preserving the platform undo and selection model. Preview coloring skips very large files instead of delaying readable text indefinitely.

A custom tab MIME type keeps a center drop from being treated as text insertion by the editor. Content-edge nodes accept platform drags without installing pointer gestures over the editor. Left/right strips own the corners, matching the shared normalized edge rule. A half-pane preview is drawn behind the drop zones only while targeted. Docking revalidates current layout membership and focuses the newly created group only on success.

Tab and strip drop targets share one validator and insertion command. A tab supplies an insert-before anchor; the strip supplies no anchor and appends. Target callbacks draw an overlay marker without changing widths. RememberUpdatedState keeps platform drag callbacks attached to the current composition. Only internally labeled tab drags with a live tab identity are accepted.

Reserve a small blank slot at the end of the strip even when tabs overflow, so append remains reachable by horizontal scrolling. It is always present and does not resize during a drag.

Detect indentation from nonblank source lines before inserting Tab: prefer the observed whitespace kind, infer the space unit from its common divisor, and advance partial indentation to the next stop. The edit stays on the native permission, composition and undo path.

## Preserve whitespace through native input

Enter carries only the leading whitespace before the caret and preserves the local
LF or CRLF convention. It does not strip existing trailing whitespace or guess
language syntax from a brace that might be prose or a literal. Backspace in a
space-only prefix retreats to the previous inferred indentation stop; tabs, text,
selections, composition and multiple cursors retain their existing editing paths.
Use native edit transactions so undo remains the platform's responsibility.
Android applies this policy in the input transformation, covering software input
as well as hardware keys. Hidden-source checks precede these transformations.

Indentation inference samples at most 200 initial lines and 16,384 characters.
Keep that work bounded now that Backspace can request the same policy as Tab;
large-file keystrokes must not repeatedly split the entire document. The sample
is a heuristic, not authoritative formatter configuration.

## Move through identifiers without replacing native editing

Keep Ctrl and Alt/Option independent, persisted editor preferences. Platform default
leaves the key with the host; custom movement touches only the document selection,
including read-only documents. Whole identifiers include underscores; pieces stop
at underscore gaps, camel-case transitions, acronym endings and number runs. Shift
retains the anchor so reversing direction contracts a selection before extending it.
Only the adjacent logical line is scanned; no parser, language service or document
rebuild runs for these arrows. Composition and unrelated modifier combinations stay
with the native editor. This is logical source movement, not visual bidi navigation;
platform default remains available for native linguistic and visual navigation.
Settings live in existing menus, keeping readable text spatially stable.

Smart Enter delegates to the [shared indentation mechanism](../../../../../../../../../smart-enter.md), then
applies one ordinary edit and its proposed caret. The complete replacement range,
including whitespace before an existing closer, passes the same hidden-source,
composition and permission checks as other edits. Changing the persisted Enter
mode affects open editors; the platform retains its own undo transaction.

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
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.geometry.Offset
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.ui.text.input.TextFieldValue
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

Compute keyword ranges on a background dispatcher and tag them with their source snapshot. OutputTransformation paints only matching results and never changes backing text. Document-word suggestions are an explicit fallback preview; connected language tools use server results. Selection replacement uses TextFieldState.edit, while minimap navigation changes only its retained scroll state. Neither capability grants permission to edit generated files.

The left line-number canvas reads the text field's actual layout and scroll state, so wrapped continuations do not receive a second number. Draw at the measured baseline with the same font size and top padding. A stale layout is ignored until the current text is measured; the editable text remains available. The gutter reserves three digits and grows for larger documents without mounting a second scrolling view.

Native search runs against an immutable source/options request on Dispatchers.Default. A changed request cancels publication, while the floating panel leaves the editor mounted and stationary. Each result proposes source edits; DocumentBuffer validates source, permissions, composition and non-overlapping ranges and applies all replacements in one TextFieldState.edit undo transaction. Search navigation unfolds before selecting original UTF-16 offsets. Invalid patterns and more than 10,000 matches refuse replacement instead of using a partial list.

Compose retains source in TextFieldState and applies outermost folds through OutputTransformation; the displayed ellipsis never enters saved bytes. Measured text uses displayed offsets, so convert those offsets back to source for line numbers and debugger menus. Nested folded headers keep their state while a parent hides them. Source-revision equality retires stale positions after programmatic edits. InputTransformation reveals folds on typing and refuses an edit that intersects hidden bytes before it can be committed. A fixed-width left gutter shows − to fold and + to unfold. Map measured display rows back to source headers before placing controls; wrapped continuation rows and hidden headers receive no duplicate icons. Debugger popovers stay on the right.

## Keep search computation separate from Compose state

The scanner returns immutable proposed edits. UTF-16 offsets match TextFieldState, and capture replacements share the browser vocabulary without relying on Java's replacement-template rules. Invalid expressions or an over-limit result return an error with no edits. The [Kotlin search witnesses](../../../../../test/java/com/hickorydocs/simple/ParityTest.kt.md) check offsets, captures and one-transaction undo through the owning buffer.

<ui:copy id="simple-android-search-model">
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
</ui:copy>

## Publish only the current request

The floating panel below computes on a background dispatcher and checks its exact source/options snapshot before enabling navigation or replacement. The editable text field remains mounted throughout.

<ui:copy id="simple-android-language-preview">
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
private val simpleWholeWord = Regex("[\\p{L}\\p{N}_][\\p{L}\\p{M}\\p{N}_\\u200C\\u200D]*")
private val simpleWordPieces = Regex("(?:\\p{Lu}\\p{M}*)+(?=\\p{Lu}\\p{M}*\\p{Ll})|(?:\\p{Lu}\\p{M}*)?(?:\\p{Ll}\\p{M}*)+|(?:\\p{Lu}\\p{M}*)+|(?:\\p{N}\\p{M}*)+|\\p{L}[\\p{L}\\p{M}\\u200C\\u200D]*")
fun simpleWordOffset(source: String, offset: Int, forward: Boolean, pieces: Boolean): Int {
    val probe = if (!forward && offset > 0) offset - 1 else offset
    val start = if (probe == 0) 0 else source.lastIndexOf('\n', probe - 1) + 1
    val end = source.indexOf('\n', probe).let { if (it < 0) source.length else it + 1 }
    val matches = (if (pieces) simpleWordPieces else simpleWholeWord).findAll(source.substring(start, end))
    return if (forward) matches.map { start + it.range.last + 1 }.firstOrNull { it > offset } ?: end
    else matches.map { start + it.range.first }.lastOrNull { it < offset } ?: start
}

fun simpleIndent(source: String): String {
    val prefixes = source.take(16_384).lineSequence().take(200).filter { it.isNotBlank() }.map { it.takeWhile { c -> c == ' ' || c == '\t' } }.toList()
    if (prefixes.count { it.startsWith("\t") } > prefixes.count { it.startsWith(" ") }) return "\t"
    fun gcd(a: Int, b: Int): Int = if (b == 0) a else gcd(b, a % b)
    val width = prefixes.filter { it.isNotEmpty() && it.all { c -> c == ' ' } }.fold(0) { width, prefix -> gcd(width, prefix.length) }
    return " ".repeat(if (width in 1..8) width else 4)
}
fun simpleNewlineText(source: String, offset: Int): String {
    val start = if (offset == 0) 0 else source.lastIndexOf('\n', offset - 1) + 1
    val end = source.indexOf('\n', offset)
    val crlf = if (end >= 0) end > 0 && source[end - 1] == '\r' else start >= 2 && source[start - 2] == '\r'
    return (if (crlf) "\r\n" else "\n") + source.substring(start, offset).takeWhile { it == ' ' || it == '\t' }
}
fun simpleBackspaceStart(source: String, offset: Int): Int? {
    val start = if (offset == 0) 0 else source.lastIndexOf('\n', offset - 1) + 1
    val before = source.substring(start, offset)
    if (before.isEmpty() || before.any { it != ' ' }) return null
    val unit = simpleIndent(source)
    if (unit == "\t") return null
    return offset - ((before.length - 1) % unit.length + 1)
}
fun simpleTabText(source: String, offset: Int): String {
    val unit = simpleIndent(source)
    if (unit == "\t") return unit
    val column = offset - (if (offset == 0) 0 else source.lastIndexOf('\n', offset - 1) + 1)
    return " ".repeat(unit.length - column % unit.length)
}
fun simpleTabEdits(source: String, from: Int, to: Int, outdent: Boolean = false): List<NativeSourceEdit> {
    if (from == to && !outdent) return listOf(NativeSourceEdit(from, to, simpleTabText(source, from)))
    val unit = simpleIndent(source)
    var offset = if (from == 0) 0 else source.lastIndexOf('\n', from - 1) + 1
    val edits = mutableListOf<NativeSourceEdit>()
    do {
        if (outdent) {
            val tail = source.substring(offset)
            val count = if (tail.startsWith("\t")) 1 else minOf(if (unit == "\t") 4 else unit.length, tail.takeWhile { it == ' ' }.length)
            if (count > 0) edits.add(NativeSourceEdit(offset, offset + count, ""))
        } else edits.add(NativeSourceEdit(offset, offset, unit))
        val next = source.indexOf('\n', offset)
        if (next < 0) break
        offset = next + 1
    } while (offset < to)
    return edits
}
fun simpleFoldRanges(source: String, markdown: Boolean = false): List<SimpleFold> {
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
    if (markdown) {
        data class Section(val line: Int, val level: Int, val end: Int)
        val headings = mutableListOf<Section>(); val sections = mutableListOf<Section>(); val starts = mutableListOf<Int>()
        var position = 0; var fence = ""; var payload = ""; var previous: Section? = null
        for ((line, row) in rows.withIndex()) {
            starts.add(position); val end = position + row.length; position = end + 1
            val trimmed = row.trim()
            if (payload.isNotEmpty()) { if (trimmed.contains(":" + payload + ">") && trimmed.contains("</")) payload = ""; previous = null; continue }
            if (fence.isNotEmpty()) { if (trimmed.length >= fence.length && trimmed.all { it == fence[0] }) fence = ""; previous = null; continue }
            val marker = Regex("""^ {0,3}(`{3,}|~{3,})""").find(row)
            if (marker != null) { fence = marker.groupValues[1]; previous = null; continue }
            val literal = Regex("""^ {0,3}<[\w.-]+:(file|copy|cut|exec|patch|entry|diagram|math|table)\b[^>]*>""").find(row)
            if (literal != null) { if (!trimmed.contains("</") && !trimmed.endsWith("/>")) payload = literal.groupValues[1]; previous = null; continue }
            val heading = Regex("""^ {0,3}(#{1,6})(?:[ \t]+|$)""").find(row)
            if (heading != null) { headings.add(Section(line, heading.groupValues[1].length, end)); previous = null }
            else if (previous != null && Regex("""^ {0,3}(?:=+|-+)\s*$""").matches(row)) { headings.add(previous.copy(level = if (trimmed[0] == '=') 1 else 2)); previous = null }
            else previous = if (trimmed.isNotEmpty() && !row.startsWith("    ") && !row.startsWith('\t')) Section(line, 0, end) else null
        }
        for (heading in headings + Section(rows.size, 0, source.length)) {
            while (sections.isNotEmpty() && sections.last().level >= heading.level) {
                val top = sections.removeAt(sections.lastIndex)
                val end = if (heading.line < rows.size) starts[heading.line] - 1 else source.length
                if (end > top.end) { result.removeAll { it.line == top.line }; result.add(SimpleFold(top.line, top.end, end)) }
            }
            sections.add(heading)
        }
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

</ui:copy>

Keep a preferred x across vertical moves, including empty and shorter cells. User edits, caret motion and non-consuming pointer observation retire that goal. Navigation assigns selection directly, so its programmatic focus/selection is not mistaken for user input.

Arrow keys carry the current layout’s caret x into the target field and set a
collapsed TextFieldValue selection after that target has layout. Interior left/right arrows and selected-text/modifier gestures remain native editing operations; only a collapsed caret at the matching boundary exits. Left enters at
the end and right at the start. Request serials prevent later typing/layout
passes from reapplying an obsolete caret. Scroll an unmounted destination into
view before its own effect requests focus.

Arrow keys commit the current cell through the snapshot guard, then request
focus by row/column. Cell identity must not depend on its source offset because
editing an earlier cell moves later offsets. A bordered source button reveals
only this table; the return button restores grids without replacing the field.
CSV uses its delimiter-aware serializer so quoting survives an edit.

## Give a table one spreadsheet-shaped place in the editor

The output transformation replaces only displayed bytes with a bounded set of placeholder rows. The original TextFieldState remains the save and undo authority. One ordered replacement list drives both directions of offset mapping, line numbers, folds, debugger icons and widget placement; placeholder continuations never acquire extra line actions. The [projection witness](../../../../../test/java/com/hickorydocs/simple/ParityTest.kt.md) checks Unicode, CRLF, literal code and the following prose's source position.

The grid scrolls inside its reserved viewport, labels rows and columns, and applies a cell on Return/Done or focus loss. A stale snapshot or read-only owner cannot authorize a write. Source reveal removes the presentation transformation; it does not recreate the field or its undo stack.

<ui:copy id="simple-android-table-grid">
// One projection drives the text field, line numbers, debugger rail and grid anchors.
// Its offsets always map back to the retained source, including Unicode and folds.
data class EditorReplacement(val from: Int, val to: Int, val text: String)
fun editorReplacements(folds: List<SimpleFold>, tables: List<InlineTable>, source: String = "", diagrams: List<InlineDiagram> = emptyList(), equations: List<InlineEquation> = emptyList()): List<EditorReplacement> {
    val tabs = mutableListOf<EditorReplacement>()
    var column = 0
    source.forEachIndexed { offset, char ->
        if (char == '\t') { val width = 4 - column % 4; tabs.add(EditorReplacement(offset, offset + 1, " ".repeat(width))); column += width }
        else if (char == '\n') column = 0 else column++
    }
    // Reserve read-only preview whitespace after the source paragraph. The
    // projection maps every preview offset back to its unchanged source anchor.
    val math = equations.groupBy { it.line }.values.map { values ->
        val end = source.indexOf('\n', values.first().to).let { if (it < 0) source.length else it }
        EditorReplacement(end, end, "\n \n \n ")
    }
    val candidates = math + diagrams.map { EditorReplacement(it.from, it.to, " \n".repeat(8) + " ") } + tabs + folds.map { EditorReplacement(it.from, it.to, " …") } + tables.map { EditorReplacement(it.from, it.to, " \n".repeat(it.displayRows - 1) + " ") }
    val result = mutableListOf<EditorReplacement>()
    for (item in candidates.sortedWith(compareBy<EditorReplacement> { it.from }.thenByDescending { it.to })) {
        if (result.lastOrNull()?.to?.let { it > item.from } != true) result.add(item)
    }
    return result
}
fun editorDisplay(source: String, replacements: List<EditorReplacement>): String {
    val result = StringBuilder(source)
    replacements.asReversed().forEach { result.replace(it.from, it.to, it.text) }
    return result.toString()
}
fun editorSourceOffset(offset: Int, replacements: List<EditorReplacement>): Int {
    var removed = 0
    for (item in replacements) {
        val start = item.from - removed
        if (offset <= start) break
        if (offset < start + item.text.length) return item.from
        removed += item.to - item.from - item.text.length
    }
    return offset + removed
}
fun editorDisplayOffset(offset: Int, replacements: List<EditorReplacement>): Int {
    var removed = 0
    for (item in replacements) {
        if (offset <= item.from) break
        if (offset < item.to) return item.from - removed
        removed += item.to - item.from - item.text.length
    }
    return offset - removed
}
fun tableColumnName(index: Int): String {
    var value = index + 1; var name = ""
    while (value > 0) { value--; name = ('A' + value % 26) + name; value /= 26 }
    return name
}
// Native layout supplies the hit-test offset; policy always returns a collapsed,
// bounded selection so focus cannot turn navigation into select-all replacement.
fun gridHorizontalExit(selection: TextRange, length: Int, direction: Int): Boolean =
    selection.collapsed && selection.start in 0..length &&
        (if (direction < 0) selection.start == 0 else direction > 0 && selection.end == length)
fun gridCaretSelection(text: String, x: Float, edge: Int, offsetAtX: (Float) -> Int): TextRange {
    val offset = if (edge < 0) text.length else if (edge > 0) 0 else offsetAtX(x)
    return TextRange(offset.coerceIn(0, text.length))
}
class GridVerticalCaretGoal {
    private var x: Float? = null
    fun position(measured: Float): Float = (x ?: measured).also { x = it }
    fun reset() { x = null }
}
private data class GridCaretRequest(val row: Int, val column: Int, val x: Float, val edge: Int, val serial: Int)
@Composable private fun InlineTableGrid(table: InlineTable, source: String, buffer: DocumentBuffer, modifier: Modifier = Modifier) {
    var navigation by remember { mutableStateOf<GridCaretRequest?>(null) }
    var navigationSerial by remember { mutableStateOf(0) }
    val verticalGoal = remember { GridVerticalCaretGoal() }
    var caretTarget by remember { mutableStateOf<Pair<Int, Int>?>(null) }
    val list = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val focus = remember(table.rows.map { it.size }) { table.rows.map { row -> row.map { FocusRequester() } } }
    Column(modifier.background(MaterialTheme.colorScheme.surface).border(1.dp, MaterialTheme.colorScheme.outlineVariant).semantics { contentDescription = "Inline table grid" }) {
        Row(Modifier.fillMaxWidth().height(28.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("Table", fontSize = 12.sp, modifier = Modifier.weight(1f).padding(start = 6.dp))
            OutlinedButton(onClick = { buffer.sourceTable = table.from }) { Text("Edit source text", fontSize = 12.sp) }
        }
        Box(Modifier.weight(1f).horizontalScroll(rememberScrollState())) {
          LazyColumn(Modifier.width((32 + 140 * table.rows.maxOf { it.size }).dp).fillMaxHeight(), state = list) {
            item { Row {
                Text("#", modifier = Modifier.width(32.dp).height(28.dp))
                repeat(table.rows.maxOf { it.size }) { column -> Text(tableColumnName(column), fontSize = 12.sp, modifier = Modifier.width(140.dp).height(28.dp).border(0.5.dp, MaterialTheme.colorScheme.outlineVariant).padding(4.dp)) }
            } }
            itemsIndexed(table.rows) { row, cells ->
                Row {
                    Text("${row + 1}", fontSize = 12.sp, modifier = Modifier.width(32.dp).height(28.dp).padding(4.dp))
                    cells.forEach { cell -> key(cell.column) {
                        var draft by remember { mutableStateOf(TextFieldValue(cell.value)) }
                        var layout by remember { mutableStateOf<TextLayoutResult?>(null) }
                        var applied by remember { mutableStateOf<Int?>(null) }
                        LaunchedEffect(cell.value) { if (draft.text != cell.value) draft = TextFieldValue(cell.value, TextRange(draft.selection.end.coerceAtMost(cell.value.length))) }
                        LaunchedEffect(navigation, layout) {
                            val request = navigation
                            val measured = layout
                            if (request != null && request.row == row && request.column == cell.column && applied != request.serial && measured != null && measured.layoutInput.text.text == draft.text) {
                                val caret = gridCaretSelection(draft.text, request.x, request.edge) { measured.getOffsetForPosition(Offset(it, 0f)) }
                                draft = draft.copy(selection = caret); applied = request.serial
                                focus[row][cell.column].requestFocus()
                                if (navigation?.serial == request.serial) navigation = null
                            }
                        }
                        var notice by remember { mutableStateOf("") }
                        var focused by remember { mutableStateOf(false) }
                        val commit = { if (draft.text != cell.value && !buffer.applySourceEdits(listOf(NativeSourceEdit(cell.from, cell.to, inlineCellText(draft.text, cell.format, cell.delimiter))), source)) notice = "Table changed. Reopen the cell and try again." }
                        val latestCommit by rememberUpdatedState(commit)
                        DisposableEffect(Unit) { onDispose { latestCommit() } }
                        BasicTextField(value = draft, onValueChange = { if (it.text != draft.text || it.selection != draft.selection) verticalGoal.reset(); draft = it }, readOnly = buffer.readOnly, singleLine = true, onTextLayout = { layout = it },
                            keyboardActions = androidx.compose.foundation.text.KeyboardActions(onDone = { commit() }), keyboardOptions = KeyboardOptions(imeAction = androidx.compose.ui.text.input.ImeAction.Done),
                            textStyle = TextStyle(fontSize = 13.sp, fontWeight = if (row == 0) FontWeight.SemiBold else FontWeight.Normal, color = MaterialTheme.colorScheme.onSurface),
                            modifier = Modifier.width(140.dp).height(28.dp).border(0.5.dp, MaterialTheme.colorScheme.outlineVariant).padding(4.dp)
                                .focusRequester(focus[row][cell.column])
                                .pointerInput(buffer) { awaitEachGesture { awaitFirstDown(requireUnconsumed = false); verticalGoal.reset(); waitForUpOrCancellation() } }
                                .onPreviewKeyEvent { event ->
                                    if (event.type == KeyEventType.KeyDown && (event.key in listOf(Key.DirectionLeft, Key.DirectionRight, Key.MoveHome, Key.MoveEnd) || (event.key in listOf(Key.DirectionUp, Key.DirectionDown) && (event.isShiftPressed || event.isCtrlPressed || event.isAltPressed || event.isMetaPressed)))) verticalGoal.reset()
                                    if (event.type != KeyEventType.KeyDown || draft.composition != null || event.isShiftPressed || event.isCtrlPressed || event.isAltPressed || event.isMetaPressed) false else {
                                        val delta = when (event.key) { Key.DirectionUp -> -1 to 0; Key.DirectionDown -> 1 to 0; Key.DirectionLeft -> 0 to -1; Key.DirectionRight -> 0 to 1; else -> null }
                                        if (delta == null || delta.second != 0 && !gridHorizontalExit(draft.selection, draft.text.length, delta.second)) false else {
                                            val measured = layout
                                            val actualX = measured?.getCursorRect(draft.selection.end.coerceAtMost(measured.layoutInput.text.length))?.left ?: 0f
                                            val x = if (delta.second == 0) verticalGoal.position(actualX) else actualX
                                            commit()
                                            val nextRow = (row + delta.first).coerceIn(table.rows.indices)
                                            val nextColumn = (cell.column + delta.second).coerceIn(table.rows[nextRow].indices)
                                            if (nextRow != row || nextColumn != cell.column) {
                                                navigationSerial += 1
                                                val request = GridCaretRequest(nextRow, nextColumn, x, delta.second, navigationSerial)
                                                caretTarget = nextRow to nextColumn
                                                navigation = request
                                                if (list.layoutInfo.visibleItemsInfo.none { it.index == nextRow + 1 }) scope.launch { if (navigation?.serial == request.serial) list.scrollToItem(nextRow + 1) }
                                            }
                                            true
                                        }
                                    }
                                }
                                .onFocusChanged { state ->
                                    val wasFocused = focused; focused = state.isFocused
                                    if (focused && caretTarget != (row to cell.column)) { verticalGoal.reset(); caretTarget = row to cell.column }
                                    if (focused && navigation?.let { it.row != row || it.column != cell.column } == true) navigation = null
                                    if (wasFocused && !focused) commit()
                                }
                                .semantics { contentDescription = "Row ${row + 1}, column ${tableColumnName(cell.column)}"; if (notice.isNotEmpty()) error(notice) })
                    } }
                }
            }
          }
        }
    }
}

</ui:copy>

Attach the grid only to an inline snapshot that still matches the field. Measure its anchor and height from the transformed text's own layout, with scrolling applied once. Retiring a fold or an obsolete scan cannot leave an offset from another display revision behind. Edits into hidden source are refused before source is revealed.

<ui:copy id="simple-android-table-editor">
@OptIn(ExperimentalFoundationApi::class)
@Composable private fun LanguageEditor(buffer: DocumentBuffer, debug: NativeDebugger?, modifier: Modifier = Modifier) {
    val context = androidx.compose.ui.platform.LocalContext.current
    val editorPreferences = remember(context) { context.getSharedPreferences("editor", android.content.Context.MODE_PRIVATE) }
    val enterRules = remember(context) { runCatching { SimpleEnterRules(context.assets.open("enter-rules.json").bufferedReader().use { it.readText() }) }.getOrNull() }
    var enterIndent by remember { mutableStateOf(editorPreferences.getString("enterIndent", "smart") ?: "smart") }
    var controlArrow by remember { mutableStateOf(editorPreferences.getString("controlArrow", "native") ?: "native") }
    var altArrow by remember { mutableStateOf(editorPreferences.getString("altArrow", "native") ?: "native") }
    DisposableEffect(editorPreferences) {
        val listener = android.content.SharedPreferences.OnSharedPreferenceChangeListener { prefs, _ ->
            enterIndent = prefs.getString("enterIndent", "smart") ?: "smart"
            controlArrow = prefs.getString("controlArrow", "native") ?: "native"
            altArrow = prefs.getString("altArrow", "native") ?: "native"
        }
        editorPreferences.registerOnSharedPreferenceChangeListener(listener)
        onDispose { editorPreferences.unregisterOnSharedPreferenceChangeListener(listener) }
    }
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
    // Compose can observe typing before the service watcher runs. Rebase here too
    // so that frame uses the cache instead of briefly showing uncolored text.
    val semantic = service?.let { rebaseNativeTokens(it.tokens, it.tokenSource, source) }.orEmpty()
    val inline = buffer.inlineMarkup
    LaunchedEffect(buffer, source) {
        val result = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) { computeInlineMarkup(source, buffer.file.path) }
        if (buffer.text == source) buffer.inlineMarkup = result
    }
    val selection = buffer.state.selection
    val folds = buffer.activeFolds()
    val presentation = tablePresentation(inline, source)
    val tables = if (!buffer.tableSourceVisible) inlineTables(presentation).filter { it.from != buffer.sourceTable } else emptyList()
    val diagrams = if (!buffer.tableSourceVisible) presentation.diagrams.filter { it.from != buffer.sourceTable } else emptyList()
    val hiddenRanges = editorReplacements(folds, tables, diagrams = diagrams)
    val replacements = editorReplacements(folds, tables, source, diagrams, if (buffer.inlineMarkup.source == source) buffer.inlineMarkup.equations else emptyList())
    val display = editorDisplay(source, replacements)
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
    val lexical = rebaseNativeTokens(highlighting.second.map { NativeLanguageSpan(it.first, it.last + 1, "keyword") }, highlighting.first, source)
    val wheelScale = with(LocalDensity.current) { 64.dp.toPx() }
    val nestedWheelRanges by rememberUpdatedState((tables.map { it.from to it.to } + diagrams.map { it.from to it.to }).mapNotNull { (from, to) ->
        val layout = textLayout ?: return@mapNotNull null
        if (layout.layoutInput.text.text != display) return@mapNotNull null
        val first = layout.getLineForOffset(editorDisplayOffset(from, replacements))
        val last = layout.getLineForOffset(editorDisplayOffset(to, replacements).coerceAtMost(display.length))
        (layout.getLineTop(first) - scroll.value) to (layout.getLineBottom(last) - scroll.value)
    })
    // Mouse wheels retain their recipient until physical pointer movement;
    // touch drags keep their native nested-scroll behavior.
    Box(modifier.pointerInput(buffer) {
        var pointer: Offset? = null
        var documentOwns: Boolean? = null
        awaitPointerEventScope {
            while (true) {
                val event = awaitPointerEvent(androidx.compose.ui.input.pointer.PointerEventPass.Initial)
                val change = event.changes.firstOrNull() ?: continue
                if (pointer != change.position) { documentOwns = null; pointer = change.position }
                if (event.type == androidx.compose.ui.input.pointer.PointerEventType.Scroll) {
                    if (documentOwns == null) documentOwns = nestedWheelRanges.none { change.position.y >= it.first && change.position.y <= it.second }
                    if (documentOwns == true) {
                        val delta = change.scrollDelta.y * wheelScale
                        event.changes.forEach { it.consume() }
                        scope.launch { scroll.scrollTo((scroll.value + delta).toInt().coerceIn(0, scroll.maxValue)) }
                    }
                }
            }
        }
    }) {
        Row(Modifier.fillMaxSize()) {
            BoxWithConstraints(Modifier.width(24.dp).fillMaxHeight().clipToBounds()) {
                val density = LocalDensity.current
                val layout = textLayout
                val headers = simpleFoldRanges(source, buffer.file.path.substringAfterLast('.').lowercase() in listOf("md", "markdown")).associateBy { it.line }
                val height = with(density) { maxHeight.toPx() }
                if (layout != null && layout.layoutInput.text.text == display) {
                    val first = layout.getLineForVerticalPosition(scroll.value.toFloat())
                    val last = layout.getLineForVerticalPosition(scroll.value + height)
                    for (visual in first..last) {
                        val offset = editorSourceOffset(layout.getLineStart(visual), replacements)
                        if (layout.getLineStart(visual) != editorDisplayOffset(offset, replacements)) continue
                        if (offset > 0 && source[offset - 1] != '\n') continue
                        val line = source.take(offset).count { it == '\n' }
                        if (line !in headers) continue
                        val top = layout.getLineTop(visual) - scroll.value + with(density) { 8.dp.toPx() }
                        if (top < 0 || top > height) continue
                        val folded = line in buffer.foldedLines
                        Text(if (folded) "+" else "−", fontSize = 13.sp,
                            modifier = Modifier.offset { IntOffset(0, top.toInt()) }.width(24.dp).height(18.dp)
                                .clickable { buffer.toggleFold(line) }.semantics { contentDescription = "${if (folded) "Unfold" else "Fold"} line ${line + 1}" })
                    }
                }
            }
            val numberColor = MaterialTheme.colorScheme.onSurfaceVariant
            val numberPaint = remember { Paint(Paint.ANTI_ALIAS_FLAG).apply {
                typeface = Typeface.MONOSPACE; textAlign = Paint.Align.RIGHT
            } }
            Canvas(Modifier.width((maxOf(3, (source.count { it == '\n' } + 1).toString().length) * 8 + 12).dp)
                .fillMaxHeight().clipToBounds().pointerInput(source, textLayout, replacements) {
                    detectTapGestures { point ->
                        val current = textLayout
                        if (current != null && current.layoutInput.text.text == display) {
                            val visual = current.getLineForVerticalPosition(point.y + scroll.value - with(density) { 8.dp.toPx() })
                            val offset = editorSourceOffset(current.getLineStart(visual), replacements)
                            val line = inlineLineAt(offset, inline.lineStarts)
                            if (inlineDebugPath(source, buffer.file.path, line) != null) debug?.toggle(buffer.file.path, line)
                        }
                    }
                }) {
                val layout = textLayout
                if (layout != null && layout.layoutInput.text.text == display) {
                    numberPaint.color = numberColor.toArgb(); numberPaint.textSize = 13.sp.toPx()
                    val first = layout.getLineForVerticalPosition((scroll.value - 8.dp.toPx()).coerceAtLeast(0f))
                    val last = layout.getLineForVerticalPosition(scroll.value + size.height)
                    // Wrapped rows share a logical number; only draw at the logical start.
                    for (visual in first..last) {
                        val offset = editorSourceOffset(layout.getLineStart(visual), replacements)
                        if (layout.getLineStart(visual) != editorDisplayOffset(offset, replacements)) continue
                        if (offset == 0 || source[offset - 1] == '\n') {
                            val logical = (if (inline.source == source) inlineLineAt(offset, inline.lineStarts) else source.take(offset).count { it == '\n' }) + 1
                            val baseline = layout.getLineBaseline(visual) - scroll.value + 8.dp.toPx()
                            drawContext.canvas.nativeCanvas.drawText(logical.toString(), size.width - 6.dp.toPx(), baseline, numberPaint)
                            val mark = debug?.breakpoints?.get(buffer.file.path)?.firstOrNull { it.optInt("line", -1) == logical - 1 }
                            if (mark != null) {
                                numberPaint.color = android.graphics.Color.RED
                                val glyph = when (mark.optString("state")) { "refused" -> "○"; "bound" -> "●"; else -> "◌" }
                                drawContext.canvas.nativeCanvas.drawText(glyph, 12.dp.toPx(), baseline, numberPaint)
                                numberPaint.color = numberColor.toArgb()
                            }
                        }
                    }
                }
            }
            Box(Modifier.weight(1f).fillMaxHeight().clipToBounds()) {
            BasicTextField(state = buffer.state, readOnly = buffer.readOnly,
                scrollState = scroll,
                inputTransformation = InputTransformation {
                    val hiddenEdit = (0 until changes.changeCount).any { index ->
                        val changed = changes.getOriginalRange(index)
                        hiddenRanges.any { range -> changed.min < range.to && changed.max > range.from || changed.collapsed && changed.min in range.from until range.to }
                    }
                    if (hiddenEdit) { revertAllChanges(); scope.launch { buffer.tableSourceVisible = true } }
                    else if (changes.changeCount == 1 && buffer.state.composition == null && !buffer.readOnly && buffer.multiRanges.size < 2) {
                        val original = changes.getOriginalRange(0)
                        val changed = changes.getRange(0)
                        val inserted = asCharSequence().subSequence(changed.min, changed.max).toString()
                        if (inserted == "\n" && original == originalSelection) {
                            val source = originalText.toString()
                            val plan = smartEnter(source, original.min, original.max, buffer.file.path, simpleIndent(source), enterIndent, enterRules)
                            if (hiddenRanges.none { plan.from < it.to && plan.to > it.from || plan.from == plan.to && plan.from in it.from until it.to }) {
                                revertAllChanges(); replace(plan.from, plan.to, plan.text); this.selection = TextRange(plan.caret)
                            } else revertAllChanges()
                        } else if (inserted.isEmpty() && originalSelection.collapsed && original.length == 1 && original.max == originalSelection.end) {
                            val start = simpleBackspaceStart(originalText.toString(), original.max)
                            if (start != null && start < original.min && hiddenRanges.none { start < it.to && original.max > it.from }) {
                                replace(start, changed.min, "")
                            }
                        }
                    }
                    // Retire display state after the input transaction, outside its receiver.
                    scope.launch { buffer.clearFolds(); if (!buffer.applyingMulti) buffer.clearMulti() }
                },
                onTextLayout = { getResult -> textLayout = getResult() },
                outputTransformation = OutputTransformation {
                    if (paused != null) {
                        val start = debugLineOffset(source, paused)
                        if (start != null && toString() == source) addStyle(SpanStyle(background = pauseColor), start, source.indexOf('\n', start).let { if (it < 0) source.length else it })
                    }
                    if (source == toString()) lexical.forEach { token ->
                        addStyle(SpanStyle(color = color), token.from, token.to)
                    }
                    if (source == toString()) semantic.forEach { token -> addStyle(SpanStyle(color = color), token.from, token.to) }
                    if (inline.source == toString()) inline.marks.forEach { mark ->
                        if (mark.kind.startsWith("syntax-") && semantic.any { it.from < mark.to && it.to > mark.from }) return@forEach
                        val style = when (mark.kind) {
                            "heading" -> SpanStyle(fontSize = maxOf(14, 25 - mark.level * 2).sp, fontWeight = FontWeight.SemiBold)
                            "syntax-keyword" -> SpanStyle(color = androidx.compose.ui.graphics.Color(0xff9955cc))
                            "syntax-string" -> SpanStyle(color = androidx.compose.ui.graphics.Color(0xff338855))
                            "syntax-number" -> SpanStyle(color = androidx.compose.ui.graphics.Color(0xffbb7722))
                            "syntax-comment" -> SpanStyle(color = androidx.compose.ui.graphics.Color(0xff888888))
                            "strong" -> SpanStyle(fontWeight = FontWeight.Bold)
                            "diff-add" -> SpanStyle(background = androidx.compose.ui.graphics.Color(0x2233aa55))
                            "diff-remove" -> SpanStyle(background = androidx.compose.ui.graphics.Color(0x22dd3344))
                            "em", "math" -> SpanStyle(fontStyle = FontStyle.Italic)
                            else -> SpanStyle(fontFamily = FontFamily.Monospace)
                        }
                        addStyle(style, mark.from, mark.to)
                    }
                    if (toString() == source) {
                        replacements.asReversed().forEach { replace(it.from, it.to, it.text) }
                        var removed = 0
                        for (item in replacements) {
                            if (tables.any { it.from == item.from && it.to == item.to } || diagrams.any { it.from == item.from && it.to == item.to }) addStyle(SpanStyle(fontSize = 28.sp), item.from - removed, item.from - removed + item.text.length)
                            removed += item.to - item.from - item.text.length
                        }
                    }
                },
                modifier = Modifier.fillMaxSize().padding(8.dp).focusRequester(editorFocus).onPreviewKeyEvent { event ->
                    val key = event.nativeKeyEvent
                    val selected = buffer.state.selection
                    val hiddenTyping = hiddenRanges.any { selected.min < it.to && selected.max > it.from || selected.collapsed && selected.min in it.from until it.to }
                    val navigationMode = if (event.isCtrlPressed && !event.isAltPressed) controlArrow else if (event.isAltPressed && !event.isCtrlPressed) altArrow else "native"
                    if (event.type == KeyEventType.KeyDown && !event.isMetaPressed && (event.key == Key.DirectionLeft || event.key == Key.DirectionRight) && navigationMode in listOf("word", "pieces") && buffer.state.composition == null) {
                        val forward = event.key == Key.DirectionRight
                        val selection = buffer.state.selection
                        val head = if (!event.isShiftPressed && !selection.collapsed) {
                            if (forward) selection.max else selection.min
                        } else simpleWordOffset(source, selection.end, forward, navigationMode == "pieces")
                        buffer.clearFolds(); buffer.clearMulti()
                        buffer.state.edit { this.selection = TextRange(if (event.isShiftPressed) selection.start else head, head) }
                        true
                    } else if (event.type == KeyEventType.KeyDown && (event.key == Key.Tab || event.key == Key.Enter) && hiddenTyping) {
                        buffer.tableSourceVisible = true; buffer.clearFolds(); true
                    } else if (event.type == KeyEventType.KeyDown && event.key == Key.Tab && !event.isCtrlPressed && !event.isMetaPressed && !event.isAltPressed && !buffer.readOnly && buffer.state.composition == null) {
                        val selection = buffer.state.selection
                        buffer.applySourceEdits(simpleTabEdits(source, selection.min, selection.max, event.isShiftPressed), source); true
                    } else if (event.type == KeyEventType.KeyDown && event.key == Key.Spacebar && event.isCtrlPressed) {
                        menu = true; scope.launch { service?.inspect() }; scope.launch { service?.complete() }; true
                    } else if (buffer.multiSource == source && buffer.multiRanges.size > 1 && event.type == KeyEventType.KeyDown && !event.isCtrlPressed && !event.isMetaPressed && !event.isAltPressed && key.device?.isVirtual == false && key.unicodeChar > 0 && buffer.state.composition == null) {
                        buffer.replaceMultiple(String(Character.toChars(key.unicodeChar))); true
                    } else false
                }.drawWithContent {
                    drawContent()
                    val layout = textLayout
                    if (buffer.multiSource == source && layout?.layoutInput?.text?.text == display) buffer.multiRanges.drop(1).forEach { sourceRange ->
                        val range = TextRange(editorDisplayOffset(sourceRange.from, replacements), editorDisplayOffset(sourceRange.to, replacements))
                        if (range.start == range.end) { val rect = layout.getCursorRect(range.start); drawLine(color, Offset(rect.left, rect.top - scroll.value), Offset(rect.left, rect.bottom - scroll.value), 2f) }
                        else { val path = layout.getPathForRange(range.min, range.max); path.translate(Offset(0f, -scroll.value.toFloat())); drawPath(path, color.copy(alpha = .2f)) }
                    }
                }.clipToBounds().semantics { contentDescription = buffer.file.path },
                textStyle = TextStyle(fontFamily = FontFamily.Monospace, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurface),
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false))
            val density = LocalDensity.current
            val layout = textLayout
            // Values were evaluated in this stopped frame. Keep them out of
            // the program bytes and use the layout's actual end-of-line caret.
            if (paused != null && debug != null && layout?.layoutInput?.text?.text == display) {
                val start = debugLineOffset(source, paused)
                if (start != null) {
                    val end = source.indexOf('\n', start).let { if (it < 0) source.length else it }
                    val rect = layout.getCursorRect(editorDisplayOffset(end, replacements))
                    val values = debug.watches.mapNotNull { expression -> debug.values[expression]?.let { "$expression = $it" } }.joinToString("  ")
                    if (values.isNotEmpty()) Text(values.take(240), color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = .65f), fontSize = 11.sp, fontFamily = FontFamily.Monospace, maxLines = 1,
                        modifier = Modifier.offset { IntOffset(rect.right.toInt() + 12, rect.top.toInt() - scroll.value + with(density) { 8.dp.roundToPx() }) }.semantics { contentDescription = "Watched expressions at evaluation line" })
                }
            }
            if (layout != null && layout.layoutInput.text.text == display) tables.forEach { table ->
                val replacement = replacements.firstOrNull { it.from == table.from && it.to == table.to } ?: return@forEach
                val anchor = editorDisplayOffset(table.from, replacements)
                val first = layout.getLineForOffset(anchor)
                val last = layout.getLineForOffset(anchor + replacement.text.length - 1)
                val top = layout.getLineTop(first) - scroll.value
                val height = layout.getLineBottom(last) - layout.getLineTop(first)
                InlineTableGrid(table, source, buffer, Modifier.offset { IntOffset(0, top.toInt() + with(density) { 8.dp.roundToPx() }) }.padding(horizontal = 8.dp).fillMaxWidth().height(with(density) { height.toDp() }))
            }
            if (layout != null && layout.layoutInput.text.text == display && inline.source == source) inline.equations.groupBy { it.line }.values.forEach { equations ->
                val end = source.indexOf('\n', equations.first().to).let { if (it < 0) source.length else it }
                val anchor = editorDisplayOffset(end, replacements)
                val row = layout.getLineForOffset(anchor)
                val top = layout.getLineBottom(row) - scroll.value
                NativeMathView(equations.joinToString(" \\qquad ") { it.body }, Modifier.offset { IntOffset(0, top.toInt() + with(density) { 8.dp.roundToPx() }) }.padding(horizontal = 8.dp).fillMaxWidth().height(80.dp))
            }
            if (layout != null && layout.layoutInput.text.text == display) diagrams.forEach { diagram ->
                val replacement = replacements.firstOrNull { it.from == diagram.from && it.to == diagram.to } ?: return@forEach
                val anchor = editorDisplayOffset(diagram.from, replacements)
                val first = layout.getLineForOffset(anchor)
                val last = layout.getLineForOffset(anchor + replacement.text.length - 1)
                val top = layout.getLineTop(first) - scroll.value
                val height = layout.getLineBottom(last) - layout.getLineTop(first)
                Column(Modifier.offset { IntOffset(0, top.toInt() + with(density) { 8.dp.roundToPx() }) }.padding(horizontal = 8.dp).fillMaxWidth().height(with(density) { height.toDp() }).background(MaterialTheme.colorScheme.surface)) {
                    OutlinedButton(onClick = { buffer.sourceTable = diagram.from }) { Text("Edit source text", fontSize = 12.sp) }
                    NativeMathView(diagram.body, Modifier.fillMaxWidth().weight(1f), diagramRenderer = diagram.renderer)
                }
            }
            if (inline.diagrams.any { it.from == buffer.sourceTable }) OutlinedButton(onClick = { buffer.sourceTable = null }) { Text("Render diagram", fontSize = 12.sp) }
            if ((buffer.tableSourceVisible || buffer.sourceTable != null) && inlineTables(inline).isNotEmpty()) OutlinedButton(onClick = { buffer.sourceTable = null; buffer.tableSourceVisible = false }) { Text("Edit tables as grids", fontSize = 12.sp) }
            }
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
            Text("⌁", modifier = Modifier.background(MaterialTheme.colorScheme.surface).clickable {
                menu = true; scope.launch { service?.inspect() }; scope.launch { service?.complete() }
            }.padding(4.dp).semantics { contentDescription = "Language tools" })
            DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                Text("Editor settings", modifier = Modifier.padding(8.dp))
                Text("Enter indentation", modifier = Modifier.padding(horizontal = 8.dp), fontSize = 12.sp)
                for ((value, title) in listOf("smart" to "Smart (language rules)", "copy" to "Match current line", "none" to "None")) {
                    DropdownMenuItem(text = { Text((if (enterIndent == value) "✓ " else "") + title) }, onClick = { editorPreferences.edit().putString("enterIndent", value).apply() })
                }
                for ((label, key, current) in listOf(Triple("Ctrl + Left/Right", "controlArrow", controlArrow), Triple("Alt + Left/Right", "altArrow", altArrow))) {
                    Text(label, modifier = Modifier.padding(horizontal = 8.dp), fontSize = 12.sp)
                    for ((value, title) in listOf("native" to "Platform default", "word" to "Whole identifier", "pieces" to "Word pieces")) {
                        DropdownMenuItem(text = { Text((if (current == value) "✓ " else "") + title) }, onClick = {
                            editorPreferences.edit().putString(key, value).apply()
                        })
                    }
                }
                HorizontalDivider()
                Column(Modifier.width(320.dp).heightIn(max = 460.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
                    Text("Language tools", fontSize = 14.sp)
                    Text(service?.status ?: "Language services not connected. Reconnect the workspace.", fontSize = 11.sp)
                    if (service != null) {
                        Text("Last received information; refresh for current position.", fontSize = 11.sp)
                        if (service.hover.isNotEmpty()) Text(service.hover, fontSize = 12.sp)
                        if (service.signature.isNotEmpty()) Text(service.signature, fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                        service.inlays.take(50).forEach { Text(it, fontSize = 11.sp) }
                    }
                    if (service != null) service.diagnostics.forEach { item ->
                        if (item.from != null && item.to != null) TextButton(enabled = service.diagnosticSource == source, onClick = { buffer.selectSourceRange(item.from, item.to, service.diagnosticSource) }) { Text(item.message) }
                        else Text(item.message + " (server message; source version unavailable)", fontSize = 12.sp)
                    }
                    service?.suggestions?.forEach { item ->
                        TextButton(enabled = !buffer.readOnly && item.source == source && item.selection == buffer.state.selection, onClick = { if (service.accept(item)) menu = false }) {
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

Only executable declarations, equations, receipts and the paused line create
right-side actions. The left number gutter toggles provisional code breakpoints;
DAP binds, relocates or refuses them. Keep their dot visible beside the number.
The right divider changes the number of columns. Free rows below an action are
usable until the next populated row; only then reserve an overflow slot.
Overflow opens just an icon grid. Equation and debugger details have their own
explicit actions and do not become overflow content.

## Measure the rail from the retained Compose editor

BasicTextField reports its real text layout after readable content exists. We use that result and the existing scroll state to position one action per visible logical line, ignoring wrapped continuation rows. A ⋯ popover holds a grid of debug controls instead of stacking icons over later lines. Paused highlighting is an output transformation and values float beside the rail; neither changes the TextFieldState, selection or undo. Source changes hide positional observations until the run and displayed source agree.

A successful gutter launch leaves the detail popover closed: the clicked line already supplied the target. Open details only when preparation needs a choice or reports a refusal; inspection controls remain explicitly available beside the line.

<ui:copy id="simple-android-debug-rail">
fun debugLineOffset(source: String, line: Int): Int? {
    if (line < 0) return null
    var offset = 0
    repeat(line) { val next = source.indexOf('\n', offset); if (next < 0) return null; offset = next + 1 }
    return offset
}
// The tooltip floats over the document; its anchor keeps the icon's slot.
@OptIn(ExperimentalMaterial3Api::class)
@Composable private fun GutterHint(label: String, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    TooltipBox(modifier = modifier, positionProvider = TooltipDefaults.rememberPlainTooltipPositionProvider(), tooltip = { PlainTooltip { Text(label) } }, state = rememberTooltipState(), content = content)
}
@OptIn(ExperimentalLayoutApi::class)
@Composable private fun NativeDebugRail(buffer: DocumentBuffer, debug: NativeDebugger, layout: TextLayoutResult?, scroll: ScrollState) {
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    var overflow by remember(buffer) { mutableStateOf<Int?>(null) }
    var railWidth by remember(buffer) { mutableStateOf(24f) }
    var opened by remember(buffer) { mutableStateOf<Int?>(null) }
    var expression by remember(buffer) { mutableStateOf("") }
    var condition by remember(buffer) { mutableStateOf("") }
    val source = buffer.text
    val starts = buffer.inlineMarkup.takeIf { it.source == source }?.lineStarts
    val folds = buffer.activeFolds()
    val tables = if (buffer.inlineMarkup.source == source && !buffer.tableSourceVisible) inlineTables(buffer.inlineMarkup).filter { it.from != buffer.sourceTable } else emptyList()
    val diagrams = if (!buffer.tableSourceVisible) tablePresentation(buffer.inlineMarkup, source).diagrams.filter { it.from != buffer.sourceTable } else emptyList()
    val replacements = editorReplacements(folds, tables, source, diagrams, if (buffer.inlineMarkup.source == source) buffer.inlineMarkup.equations else emptyList())
    val display = editorDisplay(source, replacements)
    val paused = if (debug.sourceMatches(buffer)) debug.line(buffer.file.path) else null
    Box(Modifier.width(4.dp).fillMaxHeight().background(MaterialTheme.colorScheme.outlineVariant).pointerInput(buffer) {
        detectDragGestures { change, amount -> change.consume(); railWidth = (railWidth - amount.x / density.density).coerceIn(24f, 220f) }
    }.semantics { contentDescription = "Resize right gutter" })
    LaunchedEffect(buffer.base) { buffer.refreshReceipts() }
    BoxWithConstraints(Modifier.width(railWidth.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surfaceContainer)) {
        val height = with(density) { maxHeight.toPx() }
        if (layout != null && layout.layoutInput.text.text == display) {
            val first = layout.getLineForVerticalPosition(scroll.value.toFloat())
            val last = layout.getLineForVerticalPosition(scroll.value + height)
            for (visual in first..last) {
                val offset = editorSourceOffset(layout.getLineStart(visual), replacements)
                        if (layout.getLineStart(visual) != editorDisplayOffset(offset, replacements)) continue
                if (offset > 0 && buffer.text[offset - 1] != '\n') continue
                val line = starts?.let { inlineLineAt(offset, it) } ?: source.take(offset).count { it == '\n' }
                val top = layout.getLineTop(visual) - scroll.value + with(density) { 8.dp.toPx() }
                if (top < 0 || top > height) continue
                val held = debug.breakpoints[buffer.file.path]?.firstOrNull { it.optInt("line", -1) == line }
                val actions = mutableListOf<Triple<String, String, () -> Unit>>()
                if (buffer.inlineMarkup.source == source && buffer.inlineMarkup.debug.launch[line] != null) actions.add(Triple("▷", "Debug", { scope.launch { debug.prepare(buffer, line); opened = if (debug.active) null else line } }))
                if (buffer.inlineMarkup.source == source && buffer.inlineMarkup.equations.any { it.line == line }) actions.add(Triple("ƒ", "Edit equation", { opened = line }))
                if (buffer.receiptsOn(line).isNotEmpty()) actions.add(Triple("▤", "Recorded output", { opened = line }))
                if (line == paused) {
                    listOf("▷" to "continue", "↳" to "over", "↓" to "in", "↑" to "out").forEach { (glyph, how) -> actions.add(Triple(glyph, how, { debug.step(how) })) }
                    actions.add(Triple("■", "Stop", { debug.stop() })); actions.add(Triple("◎", "Inspect", { opened = line }))
                }
                if (actions.isEmpty()) continue
                val columns = maxOf(1, (railWidth / 22).toInt())
                // Only another populated action row limits wrapping; prose leaves room.
                val next = (visual + 1..last).firstOrNull { nextVisual ->
                    val nextOffset = editorSourceOffset(layout.getLineStart(nextVisual), replacements)
                    val nextLine = starts?.let { inlineLineAt(nextOffset, it) } ?: source.take(nextOffset).count { it == '\n' }
                    buffer.inlineMarkup.debug.launch[nextLine] != null || nextLine == paused || buffer.inlineMarkup.equations.any { it.line == nextLine } || buffer.receiptsOn(nextLine).isNotEmpty()
                }
                val room = if (next != null) layout.getLineTop(next) - layout.getLineTop(visual) else height - top
                val capacity = maxOf(1, (room / with(density) { 20.dp.toPx() }).toInt()) * columns
                val visible = if (actions.size <= capacity) actions.size else maxOf(0, capacity - 1)
                Box(Modifier.offset { IntOffset(0, top.toInt()) }) {
                    actions.take(visible).forEachIndexed { index, (glyph, label, run) ->
                        GutterHint(label, Modifier.offset((index % columns * 22).dp, (index / columns * 20).dp)) {
                            Text(glyph, fontSize = 13.sp, modifier = Modifier.width(22.dp).height(18.dp).clickable(onClick = run).semantics { contentDescription = "$label on line ${line + 1}" })
                        }
                    }
                    if (visible < actions.size) Box(Modifier.offset((visible % columns * 22).dp, (visible / columns * 20).dp)) {
                        GutterHint("More line actions") { Text("⋯", modifier = Modifier.width(22.dp).height(18.dp).clickable { overflow = line }.semantics { contentDescription = "More gutter icons on line ${line + 1}" }) }
                        DropdownMenu(expanded = overflow == line, onDismissRequest = { overflow = null }) {
                            Column(Modifier.padding(4.dp)) {
                                actions.drop(visible).chunked(4).forEach { row -> Row { row.forEach { (glyph, label, run) ->
                                    GutterHint(label) { Text(glyph, modifier = Modifier.size(24.dp).clickable { overflow = null; run() }.semantics { contentDescription = label }) }
                                } } }
                            }
                        }
                    }
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
                                val output = if (receipt.has("recorded_output")) receipt.optString("recorded_output") else (0 until transcript.length()).map { transcript.getJSONObject(it) }.filter { it.optString("kind") in listOf("out", "err") }.joinToString("") { it.optString("data") }
                                Text(output.take(50_000), fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                                if (output.length > 50_000) Text("Output truncated in this control.", fontSize = 11.sp)
                            }
                            if (buffer.receiptNotice.isNotEmpty()) Text(buffer.receiptNotice, fontSize = 12.sp)
                            if (buffer.file.docID != null) TextButton(onClick = { scope.launch { buffer.refreshReceipts() } }) { Text("↻ Refresh recorded output") }
                            if (inlineDebugPath(source, buffer.file.path, line, true) != null || inlineDebugPath(source, buffer.file.path, line) != null) {
                            if (buffer.dirty) Text("Save edited text before launching. Values belong to the source that ran.", fontSize = 12.sp)
                            FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                TextButton(enabled = !debug.active && !buffer.dirty && !buffer.saving, onClick = { scope.launch { debug.prepare(buffer, line) } }) { Text("▷ Debug") }
                                if (inlineDebugPath(source, buffer.file.path, line) != null) TextButton(onClick = { debug.toggle(buffer.file.path, line) }) { Text("● Breakpoint") }
                                if (paused == line) {
                                    listOf("Continue" to "continue", "Step over" to "over", "Step in" to "in", "Step out" to "out").forEach { (label, how) -> TextButton(onClick = { debug.step(how) }) { Text(label) } }
                                    if (debug.capabilities.optBoolean("step_back")) TextButton(onClick = { debug.step("back") }) { Text("↶ Step back") }
                                }
                                if (debug.active) TextButton(onClick = { debug.stop() }) { Text("■ Stop") }
                            }
                            if (debug.owner == buffer.file.path) debug.targets.forEach { target -> TextButton(onClick = { scope.launch { debug.prepare(buffer, program = target.path) } }) { Text("Debug file ${target.path}") } }
                            debug.profiles.forEach { profile -> TextButton(onClick = { debug.launch(profile); opened = null }) { Text("Debug ${profile.optString("name")}") } }
                            if (debug.offer != null) TextButton(onClick = { scope.launch { debug.install() } }) { Text("⇩ Install debugger") }
                            if (debug.message.isNotBlank()) Text(debug.message, fontSize = 12.sp)
                            if (debug.capabilities.optBoolean("conditional_breakpoints")) {
                                OutlinedTextField(value = condition, onValueChange = { condition = it }, label = { Text("Breakpoint condition") })
                                TextButton(onClick = { debug.toggle(buffer.file.path, line, condition) }) { Text("✓ Apply condition") }
                            }
                            if (paused == line) {
                                OutlinedTextField(value = expression, onValueChange = { expression = it }, label = { Text("Expression") })
                                Row { TextButton(onClick = { debug.evaluate(expression) }) { Text("= Evaluate") }; TextButton(onClick = { debug.evaluate(expression, true) }) { Text("◉ Watch") } }
                                debug.frames.forEach { frame -> TextButton(onClick = { debug.select(frame) }) { Text(frame.optString("name", "Frame")) } }
                                debug.variables.forEach { variable -> Text("${variable.optString("name")} = ${variable.optString("value")}", fontSize = 12.sp) }
                                debug.values.forEach { (key, value) -> Row { Text("$key = $value", modifier = Modifier.weight(1f)); if (key in debug.watches) TextButton(onClick = { debug.removeWatch(key) }) { Text("×") } } }
                            }
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

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Pane.kt"><ui:paste select="#simple-0d1e071614-0"/><ui:paste select="#simple-android-pane-edges"/><ui:paste select="#simple-android-search-model"/><ui:paste select="#simple-android-language-preview"/><ui:paste select="#simple-android-table-grid"/><ui:paste select="#simple-android-table-editor"/><ui:paste select="#simple-0d1e071614-1"/><ui:paste select="#simple-android-debug-rail"/></ui:file>

</ui:doc>
