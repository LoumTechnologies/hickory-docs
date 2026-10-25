<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/workspaceRibbonSources.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Answer In The Agent Pane Has Ribbons](../../../../../contract.md)

Captured SHA-256: `2e10323bc06a1529f3a41f45d9bbb7a74b9f1828b2caaf030c01f3812b41ba2b`.

<book:entry path="apps/web/src/views/workspaceRibbonSources.test.ts">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { expect, it } from "vitest";
import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { registerLens } from "../lib/lensSources";
import { registerSearchEditor } from "../lib/workspaceSearch";
import { workspaceRibbonSources } from "./workspaceRibbonSources";
import { SessionRegistry } from "./documentSession";

it("keeps conversation and open buffer sources when no saved document has focus", () => {
  const host = document.createElement("div"); document.body.appendChild(host);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "unsaved paragraph" }) });
  const unregisterFile = registerSearchEditor("note.md", view);
  const unregisterLens = registerLens({ path: "sessions/s.md", view, source: "conversation", links: [] });
  try {
    const sources = workspaceRibbonSources(undefined, new SessionRegistry(), path => ({ path }));
    expect(sources.map(s => s.docPath)).toEqual(["note.md", "sessions/s.md"]);
    expect(sources[0].docSource).toBe("unsaved paragraph");
    expect(sources[0].view).toBe(view);
  } finally {
    unregisterFile(); unregisterLens(); view.destroy(); host.remove();
  }
});
</book:entry>

</book:doc>
