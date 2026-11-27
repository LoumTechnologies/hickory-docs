package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.JavascriptInterface
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import kotlinx.coroutines.delay
import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID

internal val DocumentBuffer.canAuthorElements: Boolean
    get() = !readOnly && !saving && !resourceDeleted &&
        (file.docID != null || file.path.substringAfterLast('.').lowercase() in listOf("md", "markdown"))

internal fun DocumentBuffer.openElementForm(line: Int? = null, top: Float = 0f): Boolean {
    if (!canAuthorElements || state.composition != null || (line == null && multiRanges.size > 1)) return false
    elementForm = NativeElementIntent(text, file.path, sourceRevision, state.selection, line, top)
    return true
}

internal data class NativeElementIntent(
    val source: String, val path: String, val revision: Long, val selection: TextRange,
    val line: Int?, val top: Float, val id: String = UUID.randomUUID().toString(),
) {
    fun current(buffer: DocumentBuffer): Boolean = buffer.canAuthorElements && buffer.text == source &&
        buffer.file.path == path && buffer.sourceRevision == revision && buffer.state.composition == null &&
        (line != null || (buffer.state.selection == selection && buffer.multiRanges.size < 2))
}

private const val localPage = "file:///android_asset/element-authoring.html"
private class ElementFormController(val buffer: DocumentBuffer) {
    var view: WebView? = null
    var isReady by mutableStateOf(false)
    var usable by mutableStateOf(true)
    var error by mutableStateOf("")
    private var loadedRevision by mutableStateOf<Long?>(null)
    private var sentIntent: String? = null

    fun read(revision: Long, source: String) {
        val args = JSONArray().put(revision.toString()).put(source)
        view?.evaluateJavascript("globalThis.hickoryElements.read(...$args)", null)
    }

    fun present(intent: NativeElementIntent?) {
        val web = view ?: return
        if (!isReady) return
        if (intent == null || !intent.current(buffer)) {
            if (sentIntent != null) web.evaluateJavascript("globalThis.hickoryElements.close()", null)
            sentIntent = null
            if (intent != null) error = "The editor changed. Close this form and open it again."
            return
        }
        if (intent.revision != loadedRevision || sentIntent == intent.id) return
        sentIntent = intent.id
        val request = JSONObject().put("id", intent.id).put("revision", intent.revision.toString())
            .put("from", intent.selection.min).put("to", intent.selection.max)
        intent.line?.let { request.put("line", it) }
        web.evaluateJavascript("globalThis.hickoryElements.open($request)", null)
    }

    fun unavailable() {
        isReady = false
        buffer.elementLines = null
        error = "Element forms are unavailable. Reopen the document to retry."
    }

    fun close() {
        buffer.elementForm = null
        if (isReady) error = ""
        buffer.focusSource?.invoke()
    }

    @JavascriptInterface fun postMessage(raw: String) {
        val web = view ?: return
        web.post {
            if (view !== web) return@post
            val message = runCatching { JSONObject(raw) }.getOrNull() ?: return@post
            when (message.optString("kind")) {
                "ready" -> isReady = true
                "lines" -> {
                    val revision = message.optString("revision").toLongOrNull() ?: return@post
                    if (revision != buffer.sourceRevision) return@post
                    val rows = message.optJSONArray("lines") ?: return@post
                    buffer.elementLines = revision to (0 until rows.length()).map { rows.optInt(it, -1) }.filter { it >= 0 }.toSet()
                    loadedRevision = revision
                    present(buffer.elementForm)
                }
                "error" -> error = message.optString("message", "Close the form and reopen the document to retry.")
                "cancel" -> if (message.optString("id") == buffer.elementForm?.id) close()
                "apply" -> {
                    val intent = buffer.elementForm ?: return@post
                    if (message.optString("id") != intent.id) return@post
                    val plan = message.optJSONObject("plan") ?: return@post
                    if (apply(plan, intent)) close()
                    else error = "The edit could not be applied. Close the form and try again in the current source."
                }
            }
        }
    }

    private fun apply(plan: JSONObject, intent: NativeElementIntent): Boolean {
        if (!intent.current(buffer)) return false
        val from = plan.optInt("from", -1)
        val to = plan.optInt("to", -1)
        val text = plan.opt("insert") as? String ?: return false
        val start = plan.optInt("selectFrom", -1)
        val end = plan.optInt("selectTo", -1)
        if (from < 0 || to < from || to > intent.source.length || start < from || end < start || end > from + text.length) return false
        if (intent.line == null && (from != intent.selection.min || to != intent.selection.max)) return false
        return buffer.applySourceEdits(listOf(NativeSourceEdit(from, to, text)), intent.source, TextRange(start, end))
    }
}

@SuppressLint("SetJavaScriptEnabled")
@Composable internal fun NativeElementForm(buffer: DocumentBuffer) {
    val controller = remember(buffer) { ElementFormController(buffer) }
    var activated by remember(buffer) { mutableStateOf(false) }
    LaunchedEffect(controller) { delay(70); activated = true }
    DisposableEffect(controller) {
        onDispose { buffer.elementLines = null; buffer.elementForm = null }
    }
    val intent = buffer.elementForm
    val open = intent != null
    val revision = buffer.sourceRevision
    LaunchedEffect(revision, controller.isReady) {
        if (controller.isReady) {
            delay(70)
            controller.read(revision, buffer.text)
        }
    }
    LaunchedEffect(intent?.id, revision, controller.isReady) { controller.present(intent) }
    BackHandler(open) { controller.close() }
    BoxWithConstraints(Modifier.fillMaxSize(), contentAlignment = Alignment.TopEnd) {
        val width = (maxWidth - 20.dp).coerceIn(1.dp, 360.dp)
        val height = (maxHeight - 16.dp).coerceIn(1.dp, 490.dp)
        val top = (intent?.top ?: 0f).dp.coerceIn(8.dp, (maxHeight - height - 8.dp).coerceAtLeast(8.dp))
        Box(Modifier.offset(x = (-10).dp, y = top).size(if (open) width else 1.dp, if (open) height else 1.dp)
            .alpha(if (open) 1f else 0f)
            .then(if (open) Modifier.border(1.dp, MaterialTheme.colorScheme.outline) else Modifier.clearAndSetSemantics { hideFromAccessibility() })) {
            if (activated && controller.usable) AndroidView(modifier = Modifier.fillMaxSize(), factory = { context ->
                WebView(context).apply {
                    controller.view = this
                    settings.javaScriptEnabled = true
                    settings.allowFileAccess = false
                    settings.allowContentAccess = false
                    settings.blockNetworkLoads = true
                    addJavascriptInterface(controller, "HickoryElements")
                    webViewClient = object : WebViewClient() {
                        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) = request.url.toString() != localPage
                        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) { controller.unavailable() }
                        override fun onRenderProcessGone(view: WebView, detail: android.webkit.RenderProcessGoneDetail): Boolean {
                            controller.unavailable()
                            controller.usable = false
                            return true
                        }
                    }
                    loadUrl(localPage)
                }
            }, update = { web ->
                web.visibility = if (open) android.view.View.VISIBLE else android.view.View.INVISIBLE
            }, onRelease = { web ->
                controller.view = null
                web.removeJavascriptInterface("HickoryElements")
                web.stopLoading()
                web.destroy()
            })
            if (open && controller.error.isEmpty() && buffer.elementLines?.first != revision) {
                Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).padding(12.dp)) {
                    Text("Reading document elements…")
                    TextButton(onClick = controller::close) { Text("Cancel") }
                }
            }
            if (open && controller.error.isNotEmpty()) {
                Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).padding(12.dp)) {
                    Text(controller.error)
                    TextButton(onClick = controller::close) { Text("Close") }
                }
            }
        }
    }
}
