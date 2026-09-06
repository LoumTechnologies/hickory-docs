// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md: the
// agent pane's history is the session document, drawn as cards with line
// numbers, and its links reach the overlay through the lens store.
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { installMockHandler } from "../api/client";
import { lensSources } from "../lib/lensSources";
import { SessionLens } from "./SessionLens";

const SOURCE = `<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:read file="data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
It slowed after the pool change; see data/latency.csv:3.
</hick:assistant>
<hick:wrote file="notes/today.hick" lines="72-75" hashes="a b c d"/>
</hick:session>
`;

afterEach(cleanup);

describe("the session lens", () => {
  it("draws the conversation's cards over the file, and registers its links", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path.startsWith("/api/sessions/view")) {
        return {
          path: "sessions/s.hick",
          source: SOURCE,
          view: { start: null, doc: null, turns: [] },
          blocks: [
            { kind: "session-user", turn: "t1", body: "Why is checkout slow?", span: [63, 111] },
            { kind: "session-read", file: "data/latency.csv", lines: "1-8", sha256: "ff", span: [112, 168] },
            { kind: "session-assistant", body: "It slowed after the pool change; see data/latency.csv:3.", span: [169, 185] },
            { kind: "session-wrote", file: "notes/today.hick", lines: "72-75", span: [260, 320] },
          ],
          links: [
            { family: "context", span: [112, 168], to: { path: "data/latency.csv", lines: [1, 8] }, title: "ctx", lines: [3, 3] },
            { family: "declared", span: [169, 185], to: { path: "data/latency.csv", lines: [3, 3] }, title: "dec", lines: [4, 6] },
            { family: "lineage", span: [260, 320], to: { path: "notes/today.hick", lines: [72, 75] }, title: "lin", lines: [7, 7] },
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
    // And the overlay can find the lens: its links, from the session's lines.
    const lens = lensSources().find((l) => l.path === "sessions/s.hick");
    expect(lens?.links.map((l) => [l.family, l.from.lines, l.to.path])).toEqual([
      ["context", [3, 3], "data/latency.csv"],
      ["declared", [4, 6], "data/latency.csv"],
      ["lineage", [7, 7], "notes/today.hick"],
    ]);
  });
});
