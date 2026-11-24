import SwiftUI

extension View {
    func disabled(reason: String?, fallbackHelp: String = "") -> some View {
        let control = disabled(reason != nil).help(reason ?? fallbackHelp)
        #if os(iOS)
        return control.modifier(DisabledControlHelp(reason: reason))
        #else
        return control
        #endif
    }
}

#if os(iOS)
private struct DisabledControlHelp: ViewModifier {
    var reason: String?
    @State private var isHovered = false
    @State private var shownReason: String?

    func body(content: Content) -> some View {
        let hoveredReason = isHovered ? reason : nil
        let show = hoveredReason != nil && hoveredReason == shownReason
        content.overlay {
            if reason != nil {
                Color.clear.contentShape(Rectangle())
                    .onHover { isHovered = $0 }
                    .accessibilityHidden(true)
                    .task(id: hoveredReason) {
                        shownReason = nil
                        guard let hoveredReason else { return }
                        do {
                            try await Task.sleep(for: .milliseconds(450))
                            if !Task.isCancelled { shownReason = hoveredReason }
                        } catch {
                            // Leaving the control or changing its reason cancels the delay.
                        }
                    }
            }
        }
        .overlay(alignment: .bottom) {
            if show, let hoveredReason {
                Text(hoveredReason)
                    .font(.caption)
                    .padding(6)
                    .frame(maxWidth: 260)
                    .fixedSize(horizontal: true, vertical: true)
                    .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 6))
                    .alignmentGuide(.bottom) { $0[.top] - 4 }
                    .allowsHitTesting(false)
                    .accessibilityHidden(true)
            }
        }
        .zIndex(show ? 1 : 0)
        .onChange(of: reason) { _, newReason in
            if newReason == nil {
                isHovered = false
                shownReason = nil
            }
        }
    }
}
#endif

struct WorkspaceCommandButton:View {
    var command:WorkspaceCommand
    var body:some View {
        let button=Button(command.label,action:command.run).disabled(reason: command.reason)
        switch command.id {
        case "file.open":button.keyboardShortcut("o")
        case "file.directory":button.keyboardShortcut("o",modifiers:[.command,.shift])
        case "file.new":button.keyboardShortcut("n")
        case "file.window":button.keyboardShortcut("n",modifiers:[.command,.shift])
        case "file.save":button.keyboardShortcut("s")
        case "file.saveAs":button.keyboardShortcut("s",modifiers:[.command,.shift])
        case "file.close":button.keyboardShortcut("w")
        case "edit.find":button.keyboardShortcut("f")
        case "shell.search":button.keyboardShortcut("p")
        case "editor.tools":button.keyboardShortcut(.space,modifiers:.control)
        case "editor.cursorBelow":button.keyboardShortcut(.downArrow,modifiers:[.command,.option])
        case "editor.cursorAbove":button.keyboardShortcut(.upArrow,modifiers:[.command,.option])
        default:button
        }
    }
}
