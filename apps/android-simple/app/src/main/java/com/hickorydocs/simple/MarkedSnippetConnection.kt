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
