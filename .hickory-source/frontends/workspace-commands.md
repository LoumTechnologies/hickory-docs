<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Name operations once for menus and action search

For maintainers changing workspace services independently of the editor renderer.

Menus and action search need one vocabulary and one availability decision. The same bundled descriptors name existing operations on every simple platform; platform adapters provide callbacks and current context. A missing adapter stays explicitly unavailable. Searching never grants authority: executing an enabled command still enters the existing source, save or change-preview guard. Keep the menu group and stable ID independent of a translated label or renderer. The ordinary eligibility function can be tested without mounting UI.

<ui:copy id="simple-workspace-command-descriptors">
[
  {
    "id": "file.connect",
    "label": "Connect…",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.open",
    "label": "Open File",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.directory",
    "label": "Open Directory",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.new",
    "label": "New Tab",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.window",
    "label": "New Window",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.save",
    "label": "Save",
    "group": "File",
    "requires": "save"
  },
  {
    "id": "file.saveAs",
    "label": "Save As",
    "group": "File",
    "requires": "editor"
  },
  {
    "id": "file.saveAll",
    "label": "Save All",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.refresh",
    "label": "Refresh",
    "group": "File",
    "requires": "always"
  },
  {
    "id": "file.close",
    "label": "Close tab",
    "group": "File",
    "requires": "tab"
  },
  {
    "id": "shell.search",
    "label": "Search Everywhere",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "edit.find",
    "label": "Find and replace",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "workspace.find",
    "label": "Find in workspace",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "symbol.rename",
    "label": "Rename symbol",
    "group": "Edit",
    "requires": "writable"
  },
  {
    "id": "symbol.actions",
    "label": "Refactor and quick fixes",
    "group": "Edit",
    "requires": "writable"
  },
  {
    "id": "document.format",
    "label": "Preview formatting",
    "group": "Edit",
    "requires": "writable"
  },
  {
    "id": "workspace.undo",
    "label": "Undo workspace change",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "workspace.redo",
    "label": "Redo workspace change",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "workspace.problems",
    "label": "Problems",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "workspace.coverage",
    "label": "Tests and coverage",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "workspace.tests",
    "label": "Source tests",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "workspace.analysis",
    "label": "Workspace analysis",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "problem.next",
    "label": "Next problem",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "problem.previous",
    "label": "Previous problem",
    "group": "Edit",
    "requires": "always"
  },
  {
    "id": "navigation.back",
    "label": "Navigate back",
    "group": "Edit",
    "requires": "back"
  },
  {
    "id": "navigation.forward",
    "label": "Navigate forward",
    "group": "Edit",
    "requires": "forward"
  },
  {
    "id": "semantic.definition",
    "label": "Go to definition",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.references",
    "label": "Find usages",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.typeDefinition",
    "label": "Go to type definition",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.implementation",
    "label": "Find implementations",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.declaration",
    "label": "Go to declaration",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.outline",
    "label": "File outline",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.symbols",
    "label": "Search workspace symbols",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.callers",
    "label": "Find callers",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.callees",
    "label": "Find callees",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.supertypes",
    "label": "Find supertypes",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "semantic.subtypes",
    "label": "Find subtypes",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "editor.undo",
    "label": "Undo text edit",
    "group": "Edit",
    "requires": "sourceUndo"
  },
  {
    "id": "editor.redo",
    "label": "Redo text edit",
    "group": "Edit",
    "requires": "sourceRedo"
  },
  {
    "id": "editor.tools",
    "label": "Language tools",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "editor.snippetNext",
    "label": "Next template field",
    "group": "Edit",
    "requires": "template"
  },
  {
    "id": "editor.snippetPrevious",
    "label": "Previous template field",
    "group": "Edit",
    "requires": "template"
  },
  {
    "id": "editor.snippetChoices",
    "label": "Choose template value",
    "group": "Edit",
    "requires": "templateChoice"
  },
  {
    "id": "editor.snippetFinish",
    "label": "Finish template editing",
    "group": "Edit",
    "requires": "template"
  },
  {
    "id": "editor.rectangle",
    "label": "Column selection",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "editor.cursorBelow",
    "label": "Add cursor below",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "editor.cursorAbove",
    "label": "Add cursor above",
    "group": "Edit",
    "requires": "editor"
  },
  {
    "id": "view.files",
    "label": "Files",
    "group": "View",
    "requires": "always"
  },
  {
    "id": "view.agent",
    "label": "Agent",
    "group": "View",
    "requires": "always"
  },
  {
    "id": "view.terminal",
    "label": "Terminal",
    "group": "View",
    "requires": "always"
  },
  {
    "id": "view.splitRight",
    "label": "Split right",
    "group": "View",
    "requires": "always"
  },
  {
    "id": "view.splitBelow",
    "label": "Split below",
    "group": "View",
    "requires": "always"
  },
  {
    "id": "view.move",
    "label": "Move tab to next group",
    "group": "View",
    "requires": "multi"
  },
  {
    "id": "workspace.review",
    "label": "Repository changes",
    "group": "View",
    "requires": "always"
  }
]
</ui:copy>

<ui:file path="apps/simple-contract/commands.json"><ui:paste select="#simple-workspace-command-descriptors"/></ui:file>
<ui:file path="apps/apple-simple/Sources/Resources/commands.json"><ui:paste select="#simple-workspace-command-descriptors"/></ui:file>
</ui:doc>
