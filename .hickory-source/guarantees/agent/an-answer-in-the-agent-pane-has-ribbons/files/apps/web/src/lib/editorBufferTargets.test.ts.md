<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/editorBufferTargets.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Answer In The Agent Pane Has Ribbons](../../../../../contract.md)

Captured SHA-256: `7b0973fa53b2e1118444a34adb13cbaf90bcd7d0578dcb852f6260dc3df53468`.

<book:entry path="apps/web/src/lib/editorBufferTargets.test.ts">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { bufferTarget, openBufferTarget, registerBufferEditor } from "./editorBufferTargets";
import { workspaceRibbonSources } from "../views/workspaceRibbonSources";
import { SessionRegistry } from "../views/documentSession";
import { initialWorkspace, openUntitledTab } from "../views/workspaceState";
import { panes } from "../shell/layout";

it("registers an unsaved editor as a ribbon target and reveals its cited paragraph without opening a file", () => {
  const host = document.createElement("div"); document.body.appendChild(host);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "Short.\n\nThe longest paragraph. 📝" }) });
  const layout = openUntitledTab(initialWorkspace());
  const tab = panes(layout.root).flatMap(p => p.tabs).find(t => t.kind === "untitled")!;
  const unregister = registerBufferEditor(tab.id, view);
  const notice = vi.fn(), setLayout = vi.fn();
  try {
    const sources = workspaceRibbonSources(null, new SessionRegistry(), path => ({ path }));
    expect(sources.find(s => s.docPath === bufferTarget(tab.id))?.view).toBe(view);
    expect(openBufferTarget(bufferTarget(tab.id), 3, layout, setLayout, notice)).toBe(true);
    expect(view.state.selection.main.head).toBe(view.state.doc.line(3).from);
    expect(setLayout).toHaveBeenCalledOnce();
    expect(notice).not.toHaveBeenCalled();
    expect(openBufferTarget(bufferTarget("closed"), 3, layout, setLayout, notice)).toBe(true);
    expect(notice).toHaveBeenCalledOnce();
    // A previous window can reuse the short tab id without reusing its evidence.
    expect(openBufferTarget(`hickory-buffer://prior-window/${tab.id}`, 3, layout, setLayout, notice)).toBe(true);
    expect(notice).toHaveBeenCalledTimes(2);
    expect(setLayout).toHaveBeenCalledOnce();
    expect(openBufferTarget("note.md", 3, layout, setLayout, notice)).toBe(false);
  } finally { unregister(); view.destroy(); host.remove(); }
});
</book:entry>

</book:doc>
