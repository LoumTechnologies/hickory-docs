// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { afterEach, describe, expect, it } from "vitest";
import { conversationAnchor, loadCollapsedLineage, saveCollapsedLineage } from "./conversationLineage";
afterEach(() => { localStorage.clear(); });
describe("conversation lineage", () => {
  it("defaults to expanded items, with a persisted option for collapsed items", () => {
    expect(loadCollapsedLineage()).toBe(false);
    saveCollapsedLineage(true);
    expect(loadCollapsedLineage()).toBe(true);
  });
  it("anchors an expanded answer but hides closed work and responds to opening it", () => {
    const host = document.createElement("div");
    host.innerHTML = '<div data-session-from="0" data-session-to="100"><p>answer</p><div data-session-from="20" data-session-to="40"><details><summary>Read file</summary><pre>evidence</pre></details></div></div>';
    const fold = host.querySelector("details")!;
    expect(conversationAnchor(host, 0, 100, false)?.textContent).toContain("answer");
    expect(conversationAnchor(host, 20, 40, false)).toBeNull();
    expect(conversationAnchor(host, 20, 40, true)).toBe(fold.querySelector("summary"));
    fold.open = true;
    expect(conversationAnchor(host, 20, 40, false)).toBe(fold.parentElement);
  });
});
