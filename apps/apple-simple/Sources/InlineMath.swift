import SwiftUI
import WebKit
@MainActor final class MathCoordinator: NSObject, WKNavigationDelegate {
    var source = ""
    var diagramRenderer: String?
    var ready = false
    func display(_ view: WKWebView) {
        guard ready, let bytes = try? JSONSerialization.data(withJSONObject: source, options: .fragmentsAllowed), let json = String(data: bytes, encoding: .utf8) else { return }
        if let renderer = diagramRenderer, let data = try? JSONSerialization.data(withJSONObject: renderer, options: .fragmentsAllowed), let engine = String(data: data, encoding: .utf8) {
            view.evaluateJavaScript("globalThis.renderDiagram(" + json + "," + engine + ")", completionHandler: nil)
        } else { view.evaluateJavaScript("globalThis.renderMath(" + json + ")", completionHandler: nil) }
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { ready = true; display(webView) }
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url
        decisionHandler(url?.isFileURL == true || url?.scheme == "about" ? .allow : .cancel)
    }
}
// Inspect bundles without touching SwiftPM's fatal Bundle.module accessor.
func nativeMathResource(roots: [URL], bundles: [Bundle] = [.main]) -> URL? {
    let candidates = bundles + roots.compactMap { Bundle(url: $0.appendingPathComponent("HickorySimple_HickorySimple.bundle")) }
    return candidates.lazy.compactMap { bundle in
        bundle.url(forResource: "math", withExtension: "html", subdirectory: "Resources") ?? bundle.url(forResource: "math", withExtension: "html")
    }.first
}
@MainActor func makeMathView(_ coordinator: MathCoordinator) -> WKWebView {
    let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent(); configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
    let view = WKWebView(frame: .zero, configuration: configuration); view.navigationDelegate = coordinator
    let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL,
        URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
    let url = nativeMathResource(roots: roots)
    if let url { view.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent()) }
    else { view.loadHTMLString("<body><p>Math resource missing; source stays editable.</p><div id='math'></div><script>globalThis.renderMath=s=>document.getElementById('math').textContent=s</script></body>", baseURL: nil) }
    return view
}
#if os(macOS)
struct NativeMathView: NSViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; return makeMathView(context.coordinator) }
    func updateNSView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.display(view) }
}
#else
struct NativeMathView: UIViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; return makeMathView(context.coordinator) }
    func updateUIView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.display(view) }
}
#endif
struct NativeDiagramPreview: View {
    var diagram: InlineDiagram
    var buffer: DocumentBuffer
    var body: some View {
        VStack(spacing: 0) {
            HStack { Text("Diagram").font(.caption); Spacer(); Button("Edit source text") { buffer.sourceTable = diagram.id }.buttonStyle(.bordered) }.padding(4)
            NativeMathView(source: diagram.body, diagramRenderer: diagram.renderer)
        }.background(.background)
    }
}
struct InlineEquationControl: View {
    var equation: InlineEquation
    var source: String
    var buffer: DocumentBuffer
    @State private var draft = ""
    @State private var notice = ""
    var body: some View {
        VStack(alignment: .leading) {
            NativeMathView(source: draft).frame(height: 120)
            TextEditor(text: $draft).font(.system(.caption, design: .monospaced)).frame(height: 64).disabled(buffer.readOnly)
            Button("Apply equation") {
                if buffer.applySourceEdits?([NativeSourceEdit(range: equation.range, text: draft)], source) != true { notice = "Equation changed. Reopen this line and try again." }
            }.disabled(buffer.readOnly || draft == equation.body)
            if !notice.isEmpty { Text(notice).font(.caption) }
        }.onAppear { draft = equation.body }
    }
}
