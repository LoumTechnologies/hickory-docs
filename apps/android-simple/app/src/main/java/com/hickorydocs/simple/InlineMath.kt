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

@SuppressLint("SetJavaScriptEnabled")
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier, diagramRenderer: String? = null, onReady: (() -> Unit)? = null) {
    val ready = rememberUpdatedState(onReady)
    val dark = rememberUpdatedState(isSystemInDarkTheme())
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean = request.url.toString() != "file:///android_asset/math.html"
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") {
                        val rendered = view.tag as? String ?: return
                        view.evaluateJavascript("globalThis.setPreviewAppearance(${dark.value});" + rendered + ";!!document.querySelector('.katex')") { if (it == "true" && view.tag == rendered) ready.value?.invoke() }
                    }
                }
            }
            tag = if (diagramRenderer == "notebook") "globalThis.renderNotebookOutput(" + JSONObject.quote(source) + ")" else if (diagramRenderer == "terminal") "globalThis.renderTerminal(" + JSONObject.quote(source) + ")" else if (diagramRenderer == null) "globalThis.renderMath(" + JSONObject.quote(source) + ")" else "globalThis.renderDiagram(" + JSONObject.quote(source) + "," + JSONObject.quote(diagramRenderer) + ")"; loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        view.evaluateJavascript("globalThis.setPreviewAppearance?.(${dark.value})", null)
        val script = if (diagramRenderer == "notebook") "globalThis.renderNotebookOutput(" + JSONObject.quote(source) + ")" else if (diagramRenderer == "terminal") "globalThis.renderTerminal(" + JSONObject.quote(source) + ")" else if (diagramRenderer == null) "globalThis.renderMath(" + JSONObject.quote(source) + ")" else "globalThis.renderDiagram(" + JSONObject.quote(source) + "," + JSONObject.quote(diagramRenderer) + ")"
        if (view.tag != script) {
            view.tag = script
            view.evaluateJavascript("if(globalThis.renderDiagram){" + script + "};!!document.querySelector('.katex')") { if (it == "true" && view.tag == script) ready.value?.invoke() }
        }
    }, onRelease = { it.stopLoading(); it.destroy() })
}
