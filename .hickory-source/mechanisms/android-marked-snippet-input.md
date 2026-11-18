<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Commit linked Android fields after native marked input

The Android keyboard can revise the current candidate several times before
committing. Compose continues to show each candidate in the primary field.
One captured input connection owns the exact original selection and source.
After commit, the snippet model checks that the observed replacement stayed
within that field before planning all mirrors. Closing the input connection
retains native text and retires the mirror permission.

## Bind the final replacement and its undo trail

The model checks the unchanged prefix and suffix, then reuses the existing
snippet session or group. A bounded trail records observed native revisions.
Compose can combine native steps. A source-menu Undo or Redo accepts only a
sequence through the observed revisions in order. If replay reaches different
history, reverse the steps attempted and return control to ordinary Undo.
When a calculation refuses a confined candidate, the controller follows the
same witnessed trail back to the original text and selections. The editor's
Redo stops at the refused candidate. Earlier actions retain Undo and Redo.

<ui:copy id="android-marked-snippet-plan">
package com.hickorydocs.simple

import java.security.MessageDigest

internal data class MarkedSnippetResult(
    val source: String,
    val selection: CompletionSelection,
    val session: SnippetSession? = null,
    val parallel: ParallelSnippet? = null,
    val refused: Boolean = false,
)

/** Owned by one native input connection. Native marked text remains authoritative until commit. */
internal class MarkedSnippetInput(
    val path: String,
    val source: String,
    val selection: CompletionSelection,
    val range: CompletionSelection,
    val history: SourceUndoSnapshot<AndroidSnippetHistory?>,
    val intent: Int,
) {
    private val prefix = source.substring(0, range.from)
    private val suffix = source.substring(range.to)
    private val trail = mutableListOf(sourceIdentity(source))
    private var overflow = false
    private var splitSource = false

    private fun confined(text: String): Boolean =
        text.length >= prefix.length + suffix.length && text.startsWith(prefix) && text.endsWith(suffix)

    fun observe(text: String) {
        if (!confined(text)) {
            splitSource = true
            return
        }
        append(text)
    }

    private fun append(text: String) {
        if (overflow || splitSource) return
        val proof = sourceIdentity(text)
        if (trail.last() == proof) return
        if (trail.size >= 64) {
            overflow = true
            return
        }
        trail.add(proof)
    }

    fun replay(finalSource: String, mirrored: Boolean): MarkedSourceReplay? {
        if (mirrored) append(finalSource) else observe(finalSource)
        if (finalSource == source || overflow || splitSource || trail.size < 2 || trail.last() != sourceIdentity(finalSource)) return null
        return MarkedSourceReplay(trail.toList())
    }

    fun finish(observed: String, selected: CompletionSelection): MarkedSnippetResult? {
        if (overflow || splitSource || !confined(observed)) return null
        val end = observed.length - suffix.length
        val inserted = observed.substring(prefix.length, end)
        if (selected.from < range.from || selected.to > end || selected.from > selected.to) return null
        val refusal = MarkedSnippetResult(source, selection, refused = true)
        if (inserted.length > 1024 * 1024 || inserted.toByteArray(Charsets.UTF_8).size > 1024 * 1024) return refusal
        val offset = selected.from - end
        val length = selected.to - selected.from
        val context = history.context ?: return null
        val multiple = context.group
        if (multiple != null) {
            if (multiple.selections[multiple.primaryIndex] != range) return null
            val plan = multiple.editing(source, multiple.selections.map { NativeSourceEdit(it.from, it.to, inserted) }) ?: return refusal
            val primary = plan.state.primaryIndex
            val ranges = plan.state.selections.toMutableList()
            val start = ranges[primary].from + offset
            ranges[primary] = CompletionSelection(start, start + length)
            val next = plan.state.selecting(plan.state.source, ranges, primary) ?: return refusal
            return MarkedSnippetResult(next.source, ranges[primary], parallel = next)
        }
        val current = context.session ?: return null
        val plan = editSnippet(current, source, range.from, range.to, inserted) ?: return refusal
        val start = plan.selection.from + offset
        return MarkedSnippetResult(plan.session.source, CompletionSelection(start, start + length), session = plan.session)
    }
}

private fun sourceIdentity(source: String): String {
    val digest = MessageDigest.getInstance("SHA-256")
    val bytes = ByteArray(4096)
    var at = 0
    for (unit in source) {
        bytes[at++] = (unit.code ushr 8).toByte()
        bytes[at++] = unit.code.toByte()
        if (at == bytes.size) {
            digest.update(bytes)
            at = 0
        }
    }
    digest.update(bytes, 0, at)
    return digest.digest().joinToString("") { "%02x".format(it) }
}

/** Native Compose may merge proposals. Accept a monotone subsequence of the
 * observed revisions. If a replay steps into unrelated history, reverse just
 * the performed steps before handing ordinary Undo or Redo back to the caller.
 */
internal class MarkedSourceReplay(private val revisions: List<String>) {
    enum class Result { Finished, Ordinary, Interrupted }
    val retainedBytes: Long get() = 128L + 160L * revisions.size

    fun run(
        backwards: Boolean,
        read: () -> String,
        move: (Boolean) -> Boolean,
    ): Result {
        val order = if (backwards) revisions.asReversed() else revisions
        if (sourceIdentity(read()) != order.first()) return Result.Ordinary
        var at = 0
        var performed = 0
        while (at < order.lastIndex && performed < order.lastIndex) {
            if (!move(backwards)) break
            performed++
            val key = sourceIdentity(read())
            val next = (at + 1..order.lastIndex).firstOrNull { order[it] == key } ?: break
            at = next
        }
        if (at == order.lastIndex) return Result.Finished
        repeat(performed) {
            if (!move(!backwards)) return Result.Interrupted
        }
        return if (sourceIdentity(read()) == order.first()) Result.Ordinary else Result.Interrupted
    }
}

internal fun changedMarkedSource(before: String, after: String): NativeSourceEdit? {
    if (before == after) return null
    fun boundary(text: String, offset: Int): Boolean {
        if (offset <= 0 || offset >= text.length) return true
        val left = text[offset - 1]
        val right = text[offset]
        return !(left == '\r' && right == '\n' || left.isHighSurrogate() && right.isLowSurrogate())
    }
    val limit = minOf(before.length, after.length)
    var first = 0
    while (first < limit && before[first] == after[first]) first++
    while (!boundary(before, first) || !boundary(after, first)) first--
    var tail = 0
    while (tail < limit - first && before[before.lastIndex - tail] == after[after.lastIndex - tail]) tail++
    while (!boundary(before, before.length - tail) || !boundary(after, after.length - tail)) tail--
    return NativeSourceEdit(first, before.length - tail, after.substring(first, after.length - tail))
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/MarkedSnippetInput.kt"><ui:paste select="#android-marked-snippet-plan"/></ui:file>


## Observe the native input connection

Keep one stable Compose interceptor with the existing text field. It observes
mark, commit and close events, including nested input batches. Ordinary input
still flows through the existing transformation. The interceptor accepts no
provider commands and owns no source authority beyond the captured draft.

<ui:copy id="android-marked-snippet-connection">
package com.hickorydocs.simple

import android.os.Looper
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputConnectionWrapper
import android.view.inputmethod.TextAttribute
import androidx.annotation.RequiresApi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.platform.InterceptPlatformTextInput
import androidx.compose.ui.platform.PlatformTextInputInterceptor
import androidx.compose.ui.platform.PlatformTextInputMethodRequest

@OptIn(ExperimentalComposeUiApi::class)
@Composable
internal fun SourceSnippetInputSession(buffer: DocumentBuffer, content: @Composable () -> Unit) {
    // A stable interceptor keeps Compose's input session attached when field
    // context, diagnostics or the rest of the editor recomposes.
    val interceptor = remember(buffer) {
        PlatformTextInputInterceptor { original, next ->
            val request = PlatformTextInputMethodRequest { attributes ->
                MarkedSnippetConnection(original.createInputConnection(attributes), buffer.snippets)
            }
            next.startInputMethod(request)
        }
    }
    InterceptPlatformTextInput(interceptor, content)
}

private class MarkedSnippetConnection(
    target: InputConnection,
    private val snippets: SourceSnippetController,
) : InputConnectionWrapper(target, false) {
    private val identity = Any()
    private var batchDepth = 0
    private var markedBatch = false

    // Binder requests are marshalled to the view thread by the platform.
    // If a host delegates a call elsewhere, let the native editor own it.
    private inline fun native(
        markedEvent: Boolean = false,
        before: () -> Unit = {},
        body: () -> Boolean,
    ): Boolean {
        if (Looper.myLooper() != Looper.getMainLooper()) return body()
        if (markedEvent && batchDepth > 0) markedBatch = true
        if (markedEvent) snippets.enteringMarkedEvent()
        val accepted = try {
            before()
            body()
        } finally {
            if (markedEvent) snippets.leavingMarkedEvent()
        }
        if (batchDepth == 0) snippets.settleMarkedInput(identity)
        return accepted
    }

    private fun starts() = snippets.beginMarkedInput(identity)
    private fun startsRegion(start: Int, end: Int) =
        snippets.beginMarkedInput(identity, CompletionSelection(minOf(start, end), maxOf(start, end)))

    override fun setComposingText(text: CharSequence?, newCursorPosition: Int): Boolean =
        native(markedEvent = true, before = ::starts) { super.setComposingText(text, newCursorPosition) }

    @RequiresApi(33)
    override fun setComposingText(text: CharSequence, newCursorPosition: Int, textAttribute: TextAttribute?): Boolean =
        native(markedEvent = true, before = ::starts) { super.setComposingText(text, newCursorPosition, textAttribute) }

    override fun setComposingRegion(start: Int, end: Int): Boolean =
        native(markedEvent = true, before = { startsRegion(start, end) }) { super.setComposingRegion(start, end) }

    @RequiresApi(33)
    override fun setComposingRegion(start: Int, end: Int, textAttribute: TextAttribute?): Boolean =
        native(markedEvent = true, before = { startsRegion(start, end) }) { super.setComposingRegion(start, end, textAttribute) }

    override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean =
        native(before = { snippets.markCommitRequested(identity) }) { super.commitText(text, newCursorPosition) }

    @RequiresApi(33)
    override fun commitText(text: CharSequence, newCursorPosition: Int, textAttribute: TextAttribute?): Boolean =
        native(before = { snippets.markCommitRequested(identity) }) { super.commitText(text, newCursorPosition, textAttribute) }

    override fun finishComposingText(): Boolean =
        native(before = { snippets.markCommitRequested(identity) }) { super.finishComposingText() }

    override fun beginBatchEdit(): Boolean {
        val accepted = super.beginBatchEdit()
        if (accepted && Looper.myLooper() == Looper.getMainLooper()) batchDepth++
        return accepted
    }

    override fun endBatchEdit(): Boolean {
        if (Looper.myLooper() != Looper.getMainLooper()) return super.endBatchEdit()
        if (batchDepth > 0) batchDepth--
        val marked = markedBatch
        if (batchDepth == 0) markedBatch = false
        return native(markedEvent = marked) { super.endBatchEdit() }
    }

    override fun closeConnection() {
        super.closeConnection()
        if (Looper.myLooper() == Looper.getMainLooper()) snippets.settleMarkedInput(identity, closing = true)
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/MarkedSnippetConnection.kt"><ui:paste select="#android-marked-snippet-connection"/></ui:file>

</ui:doc>
