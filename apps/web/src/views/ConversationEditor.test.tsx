// @vitest-environment jsdom
// Guarantees: agent/the-agent-pane-is-a-live-document.md; agent/an-answer-in-the-agent-pane-has-ribbons.md.
import { useState } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { undo } from "@codemirror/commands";
import { ConversationEditor } from "./ConversationEditor";
import { responseStart } from "../editor/protectedPrefix";
import { installMockHandler } from "../api/client";
import { lensSources } from "../lib/lensSources";
import type { AgentTurn, SessionViewResponse } from "../api/types";
import { conversationReading } from "./conversationReading";
import { charToByte } from "../lib/offsets";
afterEach(cleanup);
const turn = (id: string, parent: string | null = null): AgentTurn => ({ id, parent_id: parent, prompt: `Request ${id}`, answer: `Answer ${id}`, status: "ok", error: null, provider: "acp:codex", model: "", created_at: "", usage: null });
const first = turn("a"), live = { ...turn("b", "a"), status: "running" as const, answer: null };
const source = `<hick:session>\n<hick:user turn="a">Request a</hick:user>\n<hick:read file="note.md" lines="1-2"/>\n<hick:assistant>Answer a 📝</hick:assistant>\n</hick:session>\n`;
const start = source.indexOf("<hick:read"), end = source.indexOf("/>", start) + 2;
const data: SessionViewResponse = { path: "sessions/s.md", source, blocks: [], view: { start: null, doc: null, turns: [{ id: "a", parent: null, prompt: first.prompt, answer: first.answer, provider: first.provider, model: "", steps: [], usage: null, session_line: 2 }] },
  links: [{ family: "context", span: [charToByte(source, start), charToByte(source, end)], lines: [3, 3], to: { path: "note.md", lines: [1, 2] }, title: "read" }] };
installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? data : {});
function Harness({ stream = "Writing", reasoning = "Thinking" }: { stream?: string; reasoning?: string }) {
  const [draft, setDraft] = useState("");
  return <ConversationEditor branch={[first, live]} path="sessions/s.md" stamp="a,b" running="b" stream={stream} reasoning={reasoning} draft={draft} onDraft={setDraft} onSend={() => {}} canSend={false} />;
}
it("protects recorded and live content while preserving a response across streaming updates", async () => {
  const rendered = render(<Harness />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(view.state.doc.toString()).toContain("Answer a 📝"));
  const before = view.state.doc.toString();
  act(() => view.dispatch({ changes: { from: 0, to: 5, insert: "erase" }, userEvent: "input.type" }));
  expect(view.state.doc.toString()).toBe(before);
  // A widget's programmatic edit is protected too, not just keyboard events.
  act(() => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "replace everything" } }));
  expect(view.state.doc.toString()).toBe(before);
  act(() => view.dispatch({ changes: { from: view.state.field(responseStart), insert: "My response\nsecond line" }, userEvent: "input.type" }));
  expect(view.state.doc.toString()).toContain("My response\nsecond line");
  rendered.rerender(<Harness stream="Writing more now" />);
  await waitFor(() => expect(view.state.doc.toString()).toContain("Writing more now"));
  expect(view.state.doc.sliceString(view.state.field(responseStart))).toBe("My response\nsecond line");
  expect(view.state.doc.toString()).toContain("Thinking");
  expect(document.querySelector(".cm-turn-assistant")).toBeTruthy();
  expect(document.querySelector(".cm-turn-reasoning")).toBeTruthy();
  expect(lensSources().find(lens => lens.path === data.path)?.links.some(link => link.to.path === "note.md")).toBe(true);
  const protectedText = view.state.doc.sliceString(0, view.state.field(responseStart));
  act(() => { undo(view); });
  expect(view.state.doc.sliceString(0, view.state.field(responseStart))).toBe(protectedText);
});
it("submits only with the send shortcut; ordinary Enter remains multiline editing", async () => {
  const send = vi.fn();
  const props = { branch: [], path: null, running: null, stream: "", reasoning: "", draft: "Hello", onDraft: () => {}, onSend: send, canSend: true, stamp: "" };
  render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  act(() => view.dispatch({ selection: { anchor: view.state.doc.length } }));
  view.focus();
  fireEvent.keyDown(host, { key: "Enter", code: "Enter" });
  expect(send).not.toHaveBeenCalled();
  fireEvent.keyDown(host, { key: "Enter", code: "Enter", ctrlKey: true });
  expect(send).toHaveBeenCalledTimes(1);
});
it("reads only the selected branch and remaps recorded Unicode links without claiming live evidence", () => {
  const other = turn("alternative", "a");
  const fixture = { ...data, source: data.source.replace("</hick:session>", '<hick:user turn="alternative">OTHER BRANCH</hick:user>\n<hick:assistant>other</hick:assistant>\n</hick:session>'),
    view: { ...data.view, turns: [...data.view.turns, { ...data.view.turns[0], id: other.id, session_line: 5 }] } };
  const reading = conversationReading(fixture, [first, live], "b", "Live words", "");
  expect(reading.source).not.toContain("OTHER BRANCH");
  expect(reading.source).toContain("Live words");
  expect(reading.links).toHaveLength(1);
  expect(reading.source.slice(...reading.links[0].span)).toBe(source.slice(start, end));
});
