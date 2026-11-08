<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Move native text views without rebuilding their buffers

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

When an empty pane closes, the surviving editor can move to a new position in the SwiftUI tree. Keep its native editor cached for selection, scroll and undo, but give each representable a fresh host. Reparent the editor inside that host; reusing the representable root lets teardown of the old tree location interfere with its new owner.

SwiftUI owns the surrounding workspace; AppKit and UIKit own editing. Cache native views by buffer identity so changing a tab parent does not reconstruct text, undo history, selection or scroll. Delegates update only that buffer.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Apple capabilities use the real backend LSP channel and retain their own snapshot guards. Native semantic styling does not replace text, preserving the platform undo and selection model.

<ui:copy id="simple-7fad175521-0">
import SwiftUI
#if os(macOS)
import AppKit
@MainActor private final class MacEditor: NSObject, NSTextViewDelegate {
    let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let text = NSTextView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let buffer: DocumentBuffer
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.isRichText = false; text.isAutomaticQuoteSubstitutionEnabled = false
        text.isAutomaticDashSubstitutionEnabled = false; text.isAutomaticTextReplacementEnabled = false
        text.isAutomaticSpellingCorrectionEnabled = false; text.isContinuousSpellCheckingEnabled = false
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        text.isVerticallyResizable = true; text.isHorizontallyResizable = false
        text.autoresizingMask = [.width]; text.textContainer?.widthTracksTextView = true
        text.textContainerInset = NSSize(width: 8, height: 8)
        text.string = buffer.text; text.isEditable = !buffer.readOnly; text.allowsUndo = true
        text.setAccessibilityLabel(buffer.file.path); text.delegate = self
        scroll.hasVerticalScroller = true; scroll.documentView = text
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            let source = self.text.string as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        buffer.renderLanguage = { [weak self] tokens, source in self?.paint(tokens, source: source) }
        buffer.applyLanguageCompletion = { [weak self] range, insert, source, selection in
            guard let self, !self.buffer.readOnly, !self.text.hasMarkedText(), self.text.string == source,
                  self.text.selectedRange() == selection, NSMaxRange(range) <= (source as NSString).length,
                  self.text.shouldChangeText(in: range, replacementString: insert) else { return false }
            self.text.textStorage?.replaceCharacters(in: range, with: insert)
            self.text.setSelectedRange(NSRange(location: range.location + (insert as NSString).length, length: 0))
            self.text.didChangeText(); return true
        }
        paint(buffer.language?.tokens ?? [], source: buffer.text)
    }
    func textDidChange(_ notification: Notification) { buffer.selection = text.selectedRange(); buffer.text = text.string }
    func textViewDidChangeSelection(_ notification: Notification) { buffer.selection = text.selectedRange() }
    func textView(_ textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        guard NSStringFromSelector(commandSelector) == "complete:" else { return false }
        Task { await buffer.language?.complete() }; return true
    }
    private func paint(_ tokens: [NativeSemanticToken], source: String) {
        guard text.string == source, !text.hasMarkedText(), let storage = text.textStorage else { return }
        storage.beginEditing()
        storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: NSRange(location: 0, length: storage.length))
        for token in tokens where NSMaxRange(token.range) <= storage.length {
            let color: NSColor
            switch token.kind {
            case "keyword", "type", "class", "struct": color = .systemPurple
            case "function", "method": color = .systemBlue
            case "string": color = .systemGreen
            case "number": color = .systemOrange
            case "comment": color = .secondaryLabelColor
            default: color = .systemTeal
            }
            storage.addAttribute(.foregroundColor, value: color, range: token.range)
        }
        storage.endEditing(); text.typingAttributes[.foregroundColor] = NSColor.textColor
    }
}
@MainActor private enum EditorCache { static var views: [UUID: MacEditor] = [:] }
// SwiftUI owns a fresh host at each tree location. The cached editor moves
// between hosts, so teardown of an old representable cannot detach the new one.
private final class MacEditorHost: NSView {
    let content: NSView
    init(_ content: NSView) { self.content = content; super.init(frame: .zero) }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    func attach() {
        guard window != nil else { return }
        if content.superview !== self { content.removeFromSuperview(); addSubview(content) }
        content.frame = bounds
    }
    override func viewDidMoveToWindow() { super.viewDidMoveToWindow(); attach() }
    override func layout() { super.layout(); attach() }
}
struct NativeEditor: NSViewRepresentable {
    let buffer: DocumentBuffer
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 600, height: proposal.height ?? 400)
    }
    func makeNSView(context: Context) -> NSView {
        let editor = EditorCache.views[buffer.id] ?? MacEditor(buffer)
        EditorCache.views[buffer.id] = editor
        return MacEditorHost(editor.scroll)
    }
    func updateNSView(_ view: NSView, context: Context) { (view as? MacEditorHost)?.attach(); view.needsLayout = true }
}
#else
import UIKit
@MainActor private final class MobileEditor: NSObject, UITextViewDelegate {
    let text = UITextView()
    let buffer: DocumentBuffer
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        text.autocorrectionType = .no; text.autocapitalizationType = .none
        text.smartQuotesType = .no; text.smartDashesType = .no; text.smartInsertDeleteType = .no
        text.text = buffer.text; text.isEditable = !buffer.readOnly; text.alwaysBounceVertical = true
        text.accessibilityLabel = buffer.file.path; text.delegate = self
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            let source = self.text.text as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        buffer.renderLanguage = { [weak self] tokens, source in self?.paint(tokens, source: source) }
        buffer.applyLanguageCompletion = { [weak self] range, insert, source, selection in
            guard let self, !self.buffer.readOnly, self.text.markedTextRange == nil, self.text.text == source,
                  self.text.selectedRange == selection,
                  let start = self.text.position(from: self.text.beginningOfDocument, offset: range.location),
                  let end = self.text.position(from: start, offset: range.length), let target = self.text.textRange(from: start, to: end) else { return false }
            self.text.replace(target, withText: insert)
            self.buffer.selection = self.text.selectedRange; self.buffer.text = self.text.text; return true
        }
        paint(buffer.language?.tokens ?? [], source: buffer.text)
    }
    func textViewDidChange(_ textView: UITextView) { buffer.selection = textView.selectedRange; buffer.text = textView.text }
    func textViewDidChangeSelection(_ textView: UITextView) { buffer.selection = textView.selectedRange }
    private func paint(_ tokens: [NativeSemanticToken], source: String) {
        guard text.text == source, text.markedTextRange == nil else { return }
        let storage = text.textStorage
        storage.beginEditing()
        storage.addAttribute(.foregroundColor, value: UIColor.label, range: NSRange(location: 0, length: storage.length))
        for token in tokens where NSMaxRange(token.range) <= storage.length {
            let color: UIColor
            switch token.kind {
            case "keyword", "type", "class", "struct": color = .systemPurple
            case "function", "method": color = .systemBlue
            case "string": color = .systemGreen
            case "number": color = .systemOrange
            case "comment": color = .secondaryLabel
            default: color = .systemTeal
            }
            storage.addAttribute(.foregroundColor, value: color, range: token.range)
        }
        storage.endEditing(); text.typingAttributes[.foregroundColor] = UIColor.label
    }
}
@MainActor private enum EditorCache { static var views: [UUID: MobileEditor] = [:] }
private final class MobileEditorHost: UIView {
    let content: UIView
    init(_ content: UIView) { self.content = content; super.init(frame: .zero) }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    func attach() {
        guard window != nil else { return }
        if content.superview !== self { content.removeFromSuperview(); addSubview(content) }
        content.frame = bounds
    }
    override func didMoveToWindow() { super.didMoveToWindow(); attach() }
    override func layoutSubviews() { super.layoutSubviews(); attach() }
}
struct NativeEditor: UIViewRepresentable {
    let buffer: DocumentBuffer
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: UIView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 600, height: proposal.height ?? 400)
    }
    func makeUIView(context: Context) -> UIView {
        let editor = EditorCache.views[buffer.id] ?? MobileEditor(buffer)
        EditorCache.views[buffer.id] = editor
        return MobileEditorHost(editor.text)
    }
    func updateUIView(_ view: UIView, context: Context) { (view as? MobileEditorHost)?.attach(); view.setNeedsLayout() }
}
#endif

</ui:copy>

## Show real language results beside the retained editor

AppKit and UIKit keep their own text views above. The language session supplies semantic ranges only for the unchanged source snapshot. The shell reserves a narrow minimap and anchors a server completion popover beside it. Insertion calls back into the cached native view, checking source, selection, read-only state and marked-text composition before its normal edit API. This preserves native undo instead of replacing the entire buffer from SwiftUI.

<ui:copy id="simple-apple-language-preview">
struct LanguageEditor: View {
    var buffer: DocumentBuffer
    var body: some View {
        HStack(spacing: 0) {
            NativeEditor(buffer: buffer)
            GeometryReader { geometry in
                Canvas { context, size in
                    let lines = buffer.text.components(separatedBy: "\n")
                    let rows = min(lines.count, max(1, Int(size.height / 2)))
                    for row in 0..<rows {
                        let line = lines[row * lines.count / rows]
                        context.fill(Path(CGRect(x: 2, y: Double(row) * size.height / Double(rows), width: min(34, Double(line.count) * 0.65), height: 1)), with: .color(.secondary.opacity(0.5)))
                    }
                }.contentShape(Rectangle())
                    .gesture(DragGesture(minimumDistance: 0).onChanged { value in buffer.navigate?(min(1, max(0, value.location.y / max(1, geometry.size.height)))) })
                    .accessibilityLabel("Document minimap")
                    .accessibilityAdjustableAction { direction in buffer.navigate?(direction == .increment ? 1 : 0) }
            }.frame(width: 40)
        }.overlay(alignment: .topTrailing) {
            Button("⌁") {
                #if os(macOS)
                NSApp.activate()
                #endif
                Task { await buffer.language?.complete() }
            }.accessibilityLabel("Language completions")
                .help(buffer.language?.status ?? "Language services not connected")
                .padding(4).background(.regularMaterial).padding(.trailing, 42)
                .popover(isPresented: Binding(get: { buffer.language?.showingSuggestions ?? false }, set: { buffer.language?.showingSuggestions = $0 })) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Language server completions").font(.headline)
                        Text(buffer.language?.status ?? "Language services not connected").font(.caption).textSelection(.enabled)
                        if let language = buffer.language {
                            ScrollView {
                                VStack(alignment: .leading, spacing: 4) {
                                    ForEach(language.suggestions) { item in
                                        Button { language.accept(item) } label: {
                                            VStack(alignment: .leading) { Text(item.label).font(.system(.body, design: .monospaced)); Text(item.detail).font(.caption).foregroundStyle(.secondary) }
                                        }.disabled(buffer.readOnly)
                                    }
                                }.frame(maxWidth: .infinity, alignment: .leading)
                            }.frame(height: 180)
                            Button("Retry language services") { language.start() }
                            Text("Python needs basedpyright installed in this workspace. Snippets and multi-file completion edits are not applied here.").font(.caption)
                        }
                    }.padding().frame(width: 330)
                }
        }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/NativeEditor.swift"><ui:paste select="#simple-7fad175521-0"/><ui:paste select="#simple-apple-language-preview"/></ui:file>

</ui:doc>
