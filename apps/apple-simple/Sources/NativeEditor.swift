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
    }
    func textDidChange(_ notification: Notification) { buffer.text = text.string }
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
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        text.autocorrectionType = .no; text.autocapitalizationType = .none
        text.smartQuotesType = .no; text.smartDashesType = .no; text.smartInsertDeleteType = .no
        text.text = buffer.text; text.isEditable = !buffer.readOnly; text.alwaysBounceVertical = true
        text.accessibilityLabel = buffer.file.path; text.delegate = self
    }
    func textViewDidChange(_ textView: UITextView) { buffer.text = textView.text }
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
