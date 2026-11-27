package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.WebResourceRequest
import android.webkit.RenderProcessGoneDetail
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONObject

private data class MathPreview(val revision: Int, val script: String)

private fun mathPreviewScript(source: String, renderer: String?, revision: Int): String {
    val escaped = JSONObject.quote(source)
    return when (renderer) {
        "notebook" -> "globalThis.renderNotebookOutput($escaped)"
        "terminal" -> "globalThis.renderTerminal($escaped)"
        "source-output" -> "globalThis.renderSourceOutput($escaped,$revision)"
        null -> "globalThis.renderMath($escaped)"
        else -> "globalThis.renderDiagram($escaped,${JSONObject.quote(renderer)},$revision)"
    }
}

@SuppressLint("SetJavaScriptEnabled")
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier, diagramRenderer: String? = null, onReady: (() -> Unit)? = null, onLink: ((String) -> Unit)? = null) {
    val ready = rememberUpdatedState(onReady)
    val activateLink = rememberUpdatedState(onLink)
    val dark = rememberUpdatedState(isSystemInDarkTheme())
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
                    val url = request.url
                    val preview = view.tag as? MathPreview
                    if (preview != null && request.isForMainFrame && url.scheme == "hickory-diagram" && url.host == "navigate" &&
                        url.getQueryParameter("revision") == preview.revision.toString()) {
                        url.pathSegments.singleOrNull()?.let { activateLink.value?.invoke(it) }
                    }
                    return url.toString() != "file:///android_asset/math.html"
                }
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") {
                        val preview = view.tag as? MathPreview ?: return
                        view.evaluateJavascript("globalThis.hickoryAndroidDiagram=true;globalThis.setPreviewAppearance(${dark.value});" + preview.script + ";!!document.querySelector('.katex')") {
                            if (it == "true" && view.tag === preview) ready.value?.invoke()
                        }
                    }
                }
            }
            tag = MathPreview(1, mathPreviewScript(source, diagramRenderer, 1))
            loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        view.evaluateJavascript("globalThis.setPreviewAppearance?.(${dark.value})", null)
        val current = view.tag as? MathPreview ?: return@AndroidView
        if (current.script != mathPreviewScript(source, diagramRenderer, current.revision)) {
            val revision = current.revision + 1
            val preview = MathPreview(revision, mathPreviewScript(source, diagramRenderer, revision))
            view.tag = preview
            view.evaluateJavascript("if(globalThis.renderDiagram){" + preview.script + "};!!document.querySelector('.katex')") {
                if (it == "true" && view.tag === preview) ready.value?.invoke()
            }
        }
    }, onRelease = { it.stopLoading(); it.destroy() })
}
