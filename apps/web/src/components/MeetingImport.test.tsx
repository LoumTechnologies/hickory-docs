import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiValue, type UiResult } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { MeetingImport } from "./MeetingImport";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
vi.mock("./MeetingRecording", () => ({ MeetingRecording: () => null }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses import, retains real parser result and navigates exact utterance evidence", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const onOpen = vi.fn();
  vi.mocked(request).mockResolvedValue({ doc_id: "note", path: "note.md", source: "original.vtt", duplicate: false, format: "vtt", source_sha256: "abc", utterances: [{ id: "u1", speaker: null, at: null, text: "秘密 qualification", start_utf8: 0, end_utf8: 20, line: 3 }] });
  render(<UiActionContext.Provider value={registry}><MeetingImport onOpen={onOpen} /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(group => group.startsWith("meeting-import:"))!;
  let n = 0;
  async function invoke(label: string, args: Record<string, UiValue>) {
    const descriptor = registry.observe(parent).actions.find(action => action.label === label)!;
    let result: UiResult | undefined;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: descriptor.handle, arguments: args, request_id: String(++n) }); });
    return result;
  }
  expect(registry.observe(parent).actions.map(a => a.label)).toEqual(["Set transcript import visibility"]);
  await invoke("Set transcript import visibility", { open: true });
  await invoke("Import transcript text", { name: "meeting.vtt", text: "WEBVTT\n\n00:00.000 --> 00:01.000\n秘密 qualification" });
  expect(request).toHaveBeenCalledWith("POST", "/api/meetings/import", { name: "meeting.vtt", text: expect.stringContaining("秘密") });
  expect(screen.getByText("Transcript imported.")).toBeTruthy();
  expect(JSON.stringify(registry.observe(parent))).not.toContain("秘密");
  const reading = await invoke("Read transcript import evidence", { offset: 0, limit: 8192 }) as unknown as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(reading.outcome.value.bytes))).toContain("秘密 qualification");
  await invoke("Open transcript utterance source", { id: "u1" });
  expect(onOpen).toHaveBeenCalledWith("original.vtt", 3);
  await invoke("Set transcript import visibility", { open: false });
  expect(registry.observe(parent).actions).toHaveLength(1);
});
it("rejects invalid format and multibyte oversize before any host write", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><MeetingImport onOpen={() => {}} /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(group => group.startsWith("meeting-import:"))!;
  let n = 0;
  async function invoke(label: string, args: Record<string, UiValue>) {
    const descriptor = registry.observe(parent).actions.find(action => action.label === label)!;
    await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: descriptor.handle, arguments: args, request_id: String(++n) }); });
  }
  await invoke("Set transcript import visibility", { open: true });
  await invoke("Import transcript text", { name: "audio.mp3", text: "test" });
  expect(screen.getByRole("alert").textContent).toContain("Transcribe audio/video first");
  await invoke("Import transcript text", { name: "large.txt", text: "界".repeat(400000) });
  expect(screen.getByRole("alert").textContent).toContain("smaller than 1 MB");
  expect(request).not.toHaveBeenCalled();
});
