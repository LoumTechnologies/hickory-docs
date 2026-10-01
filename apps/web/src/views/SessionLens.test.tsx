// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md: the
// agent pane's history is the session document, drawn as cards with line
// numbers, and its links reach the overlay through the lens store.
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { installMockHandler } from "../api/client";
import { lensSources } from "../lib/lensSources";
import { SessionLens } from "./SessionLens";

const SOURCE = `<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:usage turn="0" input="10" cache-write="0" cache-read="0" output="2"/>
<hick:read file="data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
<hick:reasoning>
Let me think about the pool.
</hick:reasoning>
It slowed after the pool change; see data/latency.csv:3.
</hick:assistant>
<hick:wrote file="notes/today.hick" lines="72-75" hashes="a b c d"/>
</hick:session>
`;

afterEach(cleanup);

/** Byte span of the element that opens with `open` and ends with `close`
 * (the source is ASCII, so bytes are characters). */
function span(open: string, close: string): [number, number] {
  const from = SOURCE.indexOf(open);
  const to = SOURCE.indexOf(close, from) + close.length;
  return [from, to];
}

describe("the session lens", () => {
  it("draws the conversation's cards over the file, and registers its links", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path.startsWith("/api/sessions/view")) {
        return {
          path: "sessions/s.hick",
          source: SOURCE,
          view: { start: null, doc: null, turns: [] },
          blocks: [
            { kind: "session-user", turn: "t1", body: "Why is checkout slow?", span: span('<hick:user', "</hick:user>") },
            { kind: "session-meta", element: "usage", span: span("<hick:usage", "/>") },
            { kind: "session-read", file: "data/latency.csv", lines: "1-8", sha256: "ff", span: span("<hick:read", "/>") },
            { kind: "session-assistant", body: "It slowed after the pool change; see data/latency.csv:3.", span: span("<hick:assistant>", "</hick:assistant>") },
            { kind: "session-reasoning", body: "Let me think about the pool.", span: span("<hick:reasoning>", "</hick:reasoning>") },
            { kind: "session-wrote", file: "notes/today.hick", lines: "72-75", span: span("<hick:wrote", "hashes=\"a b c d\"/>") },
          ],
          links: [
            { family: "context", span: span("<hick:read", "/>"), to: { path: "data/latency.csv", lines: [1, 8] }, title: "ctx", lines: [5, 5] },
            { family: "declared", span: span("<hick:assistant>", "</hick:assistant>"), to: { path: "data/latency.csv", lines: [3, 3] }, title: "dec", lines: [6, 11] },
            { family: "lineage", span: span("<hick:wrote", "hashes=\"a b c d\"/>"), to: { path: "notes/today.hick", lines: [72, 75] }, title: "lin", lines: [12, 12] },
          ],
        };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<SessionLens path="sessions/s.hick" />);
    await waitFor(() => expect(screen.getByText("Why is checkout slow?")).toBeTruthy());
    expect(screen.getByText(/It slowed after the pool change/)).toBeTruthy();
    expect(screen.getByText("data/latency.csv")).toBeTruthy();
    expect(screen.getByText("notes/today.hick")).toBeTruthy();
    // Line numbers are the file's.
    expect(document.querySelector(".cm-lineNumbers")).toBeTruthy();
    // The file's own chrome and the record's bookkeeping are not shown as
    // source; the reasoning is there, folded and labelled as such.
    const shown = document.querySelector(".session-lens")!.textContent ?? "";
    expect(shown).not.toContain("<?xml");
    expect(shown).not.toContain("hick:usage");
    expect(shown).not.toContain("</hick:session>");
    const reasoning = document.querySelector("details.chat-reasoning") as HTMLDetailsElement;
    expect(reasoning).toBeTruthy();
    expect(reasoning.open).toBe(false);
    expect(reasoning.querySelector("summary")?.textContent).toBe("reasoning");
    // And the overlay can find the lens: its links, from the session's lines.
    const lens = lensSources().find((l) => l.path === "sessions/s.hick");
    expect(lens?.links.map((l) => [l.family, l.from.lines, l.to.path])).toEqual([
      ["context", [5, 5], "data/latency.csv"],
      ["declared", [6, 11], "data/latency.csv"],
      ["lineage", [12, 12], "notes/today.hick"],
    ]);
  });
  it("removes a tool ribbon while collapsed and restores it when expanded or opted in", async () => {
    const source = '<hick:session>\n<hick:user>Read it</hick:user>\n<hick:context kind="acp-activity">{"kind":"read","title":"Read source","sessionUpdate":"tool_call"}</hick:context>\n<hick:assistant>Done</hick:assistant>\n</hick:session>';
    const tool: [number, number] = [source.indexOf('<hick:context'), source.indexOf('</hick:context>') + '</hick:context>'.length];
    const answer: [number, number] = [source.indexOf('<hick:assistant>'), source.indexOf('</hick:assistant>') + '</hick:assistant>'.length];
    installMockHandler(async () => ({ path: "sessions/fold.hick", source, view: { turns: [] },
      blocks: [{ kind: "session-context", context_kind: "acp-activity", body: '{"kind":"read","title":"Read source","sessionUpdate":"tool_call"}', span: tool },
        { kind: "session-assistant", body: "Done", span: answer }],
      links: [{ family: "context", span: tool, lines: [3, 3], to: { path: "source.hick" }, title: "read" }],
    }));
    const result = render(<SessionLens path="sessions/fold.hick" />);
    await screen.findByText("Read source");
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.hick")?.links).toEqual([]));
    const fold = screen.getByText("Read source").closest("details")!;
    fold.open = true; fireEvent(fold, new Event("toggle"));
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.hick")?.links).toHaveLength(1));
    fold.open = false; fireEvent(fold, new Event("toggle"));
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.hick")?.links).toEqual([]));
    result.rerender(<SessionLens path="sessions/fold.hick" showCollapsedLineage />);
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.hick")?.links).toHaveLength(1));
  });

});
