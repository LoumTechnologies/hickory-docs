<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Render native commands independently of search

For maintainers changing workspace services independently of the editor renderer.

Menus and search consume the same command values, but the menu button does not belong to the search feature. This adapter applies current availability, an explanatory tooltip and established native keyboard shortcuts. Keeping it separate lets us remove or replace Search Everywhere without rewriting menu rendering or moving platform keyboard handling into workspace policy.

<ui:copy id="simple-workspace-command-button-apple">
import SwiftUI
struct WorkspaceCommandButton:View {
    var command:WorkspaceCommand
    var body:some View {
        let button=Button(command.label,action:command.run).disabled(command.reason != nil).help(command.reason ?? command.label)
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/WorkspaceCommandButton.swift"><ui:paste select="#simple-workspace-command-button-apple"/></ui:file>
</ui:doc>
