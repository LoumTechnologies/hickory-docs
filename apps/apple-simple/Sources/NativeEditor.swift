import SwiftUI
#if os(macOS)
import AppKit
@MainActor private final class MacEditor: NSObject, NSTextViewDelegate {
    let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let text = NSTextView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let buffer: DocumentBuffer
    private var highlighting: Task<Void, Never>?
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
        buffer.complete = { [weak self] word in
            guard let self, !self.buffer.readOnly else { return }
            let range = self.text.selectedRange()
            if self.text.shouldChangeText(in: range, replacementString: word) {
                self.text.textStorage?.replaceCharacters(in: range, with: word); self.text.didChangeText()
                self.text.setSelectedRange(NSRange(location: range.location + (word as NSString).length, length: 0))
            }
        }
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            let source = self.text.string as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        paint()
    }
    func textDidChange(_ notification: Notification) { buffer.text = text.string; paint() }
    private func paint() {
        highlighting?.cancel()
        let source = text.string
        highlighting = Task { [weak self] in
            let ranges = await Task.detached { previewKeywords(source) }.value
            guard !Task.isCancelled, let self, self.text.string == source,
                  let storage = self.text.textStorage else { return }
            storage.beginEditing()
            storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: NSRange(location: 0, length: storage.length))
            for range in ranges { storage.addAttribute(.foregroundColor, value: NSColor.systemPurple, range: range) }
            storage.endEditing()
            self.text.typingAttributes[.foregroundColor] = NSColor.textColor
        }
    }
}
@MainActor private enum EditorCache { static var views: [UUID: MacEditor] = [:] }
struct NativeEditor: NSViewRepresentable {
    let buffer: DocumentBuffer
    func makeNSView(context: Context) -> NSScrollView {
        if let current = EditorCache.views[buffer.id] { return current.scroll }
        let editor = MacEditor(buffer); EditorCache.views[buffer.id] = editor; return editor.scroll
    }
    func updateNSView(_ view: NSScrollView, context: Context) {}
}
#else
import UIKit
@MainActor private final class MobileEditor: NSObject, UITextViewDelegate {
    let text = UITextView()
    let buffer: DocumentBuffer
    private var highlighting: Task<Void, Never>?
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        text.autocorrectionType = .no; text.autocapitalizationType = .none
        text.smartQuotesType = .no; text.smartDashesType = .no; text.smartInsertDeleteType = .no
        text.text = buffer.text; text.isEditable = !buffer.readOnly; text.alwaysBounceVertical = true
        text.accessibilityLabel = buffer.file.path; text.delegate = self
        buffer.complete = { [weak self] word in
            guard let self, !self.buffer.readOnly else { return }
            self.text.insertText(word)
        }
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            let source = self.text.text as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        paint()
    }
    func textViewDidChange(_ textView: UITextView) { buffer.text = textView.text; paint() }
    private func paint() {
        highlighting?.cancel()
        let source = text.text ?? ""
        highlighting = Task { [weak self] in
            let ranges = await Task.detached { previewKeywords(source) }.value
            guard !Task.isCancelled, let self, self.text.text == source else { return }
            let storage = self.text.textStorage
            storage.beginEditing()
            storage.addAttribute(.foregroundColor, value: UIColor.label, range: NSRange(location: 0, length: storage.length))
            for range in ranges { storage.addAttribute(.foregroundColor, value: UIColor.systemPurple, range: range) }
            storage.endEditing()
            self.text.typingAttributes[.foregroundColor] = UIColor.label
        }
    }
}
@MainActor private enum EditorCache { static var views: [UUID: MobileEditor] = [:] }
struct NativeEditor: UIViewRepresentable {
    let buffer: DocumentBuffer
    func makeUIView(context: Context) -> UITextView {
        if let current = EditorCache.views[buffer.id] { return current.text }
        let editor = MobileEditor(buffer); EditorCache.views[buffer.id] = editor; return editor.text
    }
    func updateUIView(_ view: UITextView, context: Context) {}
}
#endif

// The preview deliberately recognizes a small keyword vocabulary, not language semantics.
func previewKeywords(_ source: String) -> [NSRange] {
    guard source.utf16.count <= 200_000,
          let regex = try? NSRegularExpression(pattern: "\\b(func|fun|fn|let|val|var|const|class|struct|import|return|if|else|true|false|def)\\b") else { return [] }
    return regex.matches(in: source, range: NSRange(source.startIndex..., in: source)).map(\.range)
}
struct LanguageEditor: View {
    var buffer: DocumentBuffer
    @State private var suggestions = false
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
            Button("⌘") { suggestions.toggle() }.accessibilityLabel("Language preview")
                .padding(4).background(.regularMaterial).padding(.trailing, 42)
                .popover(isPresented: $suggestions) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Language UI preview").font(.headline)
                        Text("Local keyword colors and document words. Native LSP transport is not connected.").font(.caption)
                        Text("Choose a word to insert at the selection.").font(.caption)
                        ScrollView {
                            VStack(alignment: .leading) {
                                ForEach(Array(Set(buffer.text.components(separatedBy: CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "_")).inverted).filter { $0.count > 2 })).sorted().prefix(30), id: \.self) { word in
                                    Button(word) { buffer.complete?(word); suggestions = false }.disabled(buffer.readOnly)
                                }
                            }
                        }.frame(maxHeight: 220)
                    }.padding().frame(width: 270)
                }
        }
    }
}
