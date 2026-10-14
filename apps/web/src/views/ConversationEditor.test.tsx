// @vitest-environment jsdom
// Guarantees: agent/the-agent-pane-is-a-live-document.md; agent/an-answer-in-the-agent-pane-has-ribbons.md.
import { useState } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { undo } from "@codemirror/commands";
import { ConversationEditor, focusConversationResponse } from "./ConversationEditor";
import { responseStart } from "../editor/protectedPrefix";
import { installMockHandler } from "../api/client";
import { lensSources } from "../lib/lensSources";
import type { AgentTurn, SessionViewResponse } from "../api/types";
import { conversationReading } from "./conversationReading";
import { charToByte } from "../lib/offsets";
import { foldedRanges, unfoldEffect } from "@codemirror/language";
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
it("opens at the editable response and can return there without rewriting the conversation", async () => {
  const props = { branch: [], path: null, running: null, stream: "", reasoning: "", draft: "My draft", onDraft: () => {}, onSend: () => {}, canSend: true, stamp: "" };
  const rendered = render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(view.state.selection.main.head).toBe(view.state.doc.length));
  expect(view.hasFocus).toBe(true);
  const before = view.state.doc.toString();
  act(() => view.dispatch({ selection: { anchor: 0 } }));
  act(() => focusConversationResponse(rendered.container));
  expect(view.state.selection.main.head).toBe(view.state.doc.length);
  expect(view.state.doc.toString()).toBe(before);
  rendered.rerender(<ConversationEditor {...props} stream="Update" />);
  expect(view.state.doc.sliceString(view.state.field(responseStart))).toBe("My draft");
});
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
it("places Send after the multiline draft without adding bytes, and disables an empty draft", async () => {
  const send = vi.fn();
  const props = { branch: [], path: null, running: null, stream: "", reasoning: "", draft: "Hello\nthere", onDraft: () => {}, onSend: send, canSend: true, stamp: "" };
  const rendered = render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  const button = await screen.findByRole("button", { name: "Send" });
  expect(button.closest(".cm-line")?.textContent).toBe("thereSend");
  expect(view.state.doc.sliceString(view.state.field(responseStart))).toBe("Hello\nthere");
  fireEvent.click(button);
  expect(send).toHaveBeenCalledTimes(1);
  rendered.rerender(<ConversationEditor {...props} draft="" />);
  await waitFor(() => expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(true));
});
it("follows growing responses, pauses when scrolling upward, and resumes on sending", async () => {
  const props = { branch: [live], path: null, running: "b", stream: "Writing", reasoning: "", draft: "", onDraft: () => {}, onSend: () => {}, canSend: false, stamp: "" };
  const rendered = render(<div className="chat-log"><ConversationEditor {...props} /></div>);
  const log = rendered.container.firstElementChild as HTMLElement;
  Object.defineProperties(log, { scrollHeight: { configurable: true, value: 1000 }, clientHeight: { value: 200 } });
  await waitFor(() => expect(log.scrollTop).toBe(1000));
  log.scrollTop = 800; fireEvent.scroll(log);
  log.scrollTop = 400; fireEvent.scroll(log);
  Object.defineProperty(log, "scrollHeight", { value: 1200 });
  rendered.rerender(<div className="chat-log"><ConversationEditor {...props} stream="Writing more" /></div>);
  await act(async () => { await new Promise(resolve => requestAnimationFrame(resolve)); });
  expect(log.scrollTop).toBe(400);
  rendered.rerender(<div className="chat-log"><ConversationEditor {...props} sending /></div>);
  await waitFor(() => expect(log.scrollTop).toBe(1200));
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

it("folds newly arriving context with native editor folds and refreshes its receipts on unfold", async () => {
  const contextSource = '<hick:session>\n<hick:user turn="a">Request a</hick:user>\n<hick:context kind="editor-buffers">\nSnapshot 📝\n</hick:context>\n<hick:assistant>Answer a</hick:assistant>\n</hick:session>\n';
  const from = contextSource.indexOf("<hick:context"), to = contextSource.indexOf("</hick:context>") + 15;
  const contextData = { ...data, source: contextSource, links: [{ ...data.links[0], span: [charToByte(contextSource, from), charToByte(contextSource, to)] as [number, number], lines: [3,5] as [number,number] }] };
  installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? contextData : {});
  const props = { branch: [first], path: data.path, stamp: "a", running: null, stream: "", reasoning: "", draft: "", onDraft: () => {}, onSend: () => {}, canSend: false };
  const rendered = render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(foldedRanges(view.state).size).toBeGreaterThan(0));
  expect(host.querySelector("details")).toBeNull();
  expect(view.state.doc.toString()).toContain("Snapshot 📝");
  expect(host.textContent).not.toContain("Snapshot 📝");
  await waitFor(() => expect(lensSources().find(lens => lens.path === data.path)?.links).toHaveLength(0));
  rendered.rerender(<ConversationEditor {...props} showCollapsedLineage />);
  await waitFor(() => expect(lensSources().find(lens => lens.path === data.path)?.links[0]?.from.lines).toEqual([3,3]));
  const folds: { from: number; to: number }[] = [];
  foldedRanges(view.state).between(0, view.state.doc.length, (from, to) => { folds.push({ from, to }); });
  act(() => view.dispatch({ effects: folds.map(f => unfoldEffect.of(f)) }));
  await waitFor(() => expect(host.textContent).toContain("Snapshot 📝"));
  rendered.rerender(<ConversationEditor {...props} draft="new response" />);
  await waitFor(() => expect(view.state.doc.toString()).toContain("new response"));
  expect(foldedRanges(view.state).size).toBe(0);
  expect(lensSources().find(lens => lens.path === data.path)?.links[0]?.from.lines).toEqual([3,5]);
  installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? data : {});
});
