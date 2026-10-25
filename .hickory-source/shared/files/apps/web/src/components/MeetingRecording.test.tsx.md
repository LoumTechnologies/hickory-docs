<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/MeetingRecording.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Meeting recordings remain recoverable local evidence](../../../../../../guarantees/authoring/recordings-stay-local/contract.md)

Captured SHA-256: `ba7206bcf4fbb4292fb0e89682600de498c823908fe548c2059957ea551a4152`.

<book:entry path="apps/web/src/components/MeetingRecording.test.tsx">
// Guarantee: docs/guarantees/authoring/recordings-stay-local.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { request } from "../api/client";
import { MeetingRecording } from "./MeetingRecording";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.unstubAllGlobals(); });
const catalogue = { directory: "/local/application-data/recordings", recordings: [{ id: "record", bytes: 10, complete: true, sha256: "audio-hash", created: "today" }], adapter: { mode: "remote", command: "", args: [], endpoint: "https://adapter.example/transcribe", model: "chosen-model" }, retention: "Local retention" };
it("requires explicit consent before remote transcription and preserves evidence on failure", async () => {
  vi.mocked(request).mockResolvedValue(catalogue);
  render(<MeetingRecording />);
  fireEvent.click(screen.getByText("Record a meeting"));
  const transcribe = await screen.findByText("Transcribe / retry");
  expect((transcribe as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByLabelText(/I authorize uploading/));
  expect((transcribe as HTMLButtonElement).disabled).toBe(false);
  vi.mocked(request).mockRejectedValueOnce(new Error("provider unavailable; audio retained"));
  fireEvent.click(transcribe);
  await screen.findByRole("alert");
  expect(request).toHaveBeenLastCalledWith("POST", "/api/meetings/transcribe", expect.objectContaining({ id: "record", remote_consent: true, expected_adapter: catalogue.adapter }));
  expect(screen.getByText("Save retained audio")).toBeTruthy();
  expect((transcribe as HTMLButtonElement).disabled).toBe(true);
});
it("shows capture refusal without creating a recording or uploading audio", async () => {
  vi.mocked(request).mockResolvedValue(catalogue);
  vi.stubGlobal("MediaRecorder", class { static isTypeSupported() { return true; } });
  const denied = vi.fn().mockRejectedValue(new DOMException("Permission denied", "NotAllowedError"));
  Object.defineProperty(navigator, "mediaDevices", { configurable: true, value: { getUserMedia: denied } });
  render(<MeetingRecording />);
  fireEvent.click(screen.getByText("Record a meeting"));
  await screen.findByText("Save retained audio");
  fireEvent.click(screen.getByText("Start microphone recording"));
  await screen.findByText("Capture refused or unavailable");
  expect(denied).toHaveBeenCalledWith({ audio: true, video: false });
  expect(vi.mocked(request).mock.calls.every(call => call[0] === "GET")).toBe(true);
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("exposes microphone and remote transcription as human requests even with cached consent", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { act } = await import("@testing-library/react");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(request).mockResolvedValue(catalogue);
  render(<UiActionContext.Provider value={registry}><MeetingRecording /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("meeting-recording:"))!; let n = 0;
  async function invoke(label: string, args: Record<string, string | boolean>) {
    const action = registry.observe(parent).actions.find(a => a.label === label)!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); });
    return result;
  }
  expect(registry.observe(parent).actions).toHaveLength(1);
  await invoke("Set meeting recording visibility", { open: true });
  expect(await invoke("Start microphone recording", {})).toMatchObject({ outcome: { status: "requires_input" } });
  expect(await invoke("Set remote transcription consent", { checked: true })).toMatchObject({ outcome: { status: "requires_input" } });
  fireEvent.click(screen.getByLabelText(/I authorize uploading/));
  expect((screen.getByLabelText(/I authorize uploading/) as HTMLInputElement).checked).toBe(true);
  expect(await invoke("Transcribe retained meeting audio", { id: "record" })).toMatchObject({ outcome: { status: "requires_input", request: expect.stringContaining("record:audio-hash") } });
  expect(vi.mocked(request).mock.calls.every(call => call[0] === "GET")).toBe(true);
  expect(JSON.stringify(registry.observe(parent))).not.toContain(catalogue.directory);
});
it("allows status and cancellation while an actual local transcription request is pending", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { act } = await import("@testing-library/react");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let reject!: (reason: Error) => void;
  const pendingHost = new Promise((_resolve, reject_) => { reject = reject_; });
  vi.mocked(request).mockImplementation(async (_method, path) => path === "/api/meetings/transcribe" ? pendingHost : path.endsWith("/cancel") ? { cancelled: "record", coverage: "Actual host cancellation response" } : { ...catalogue, adapter: { ...catalogue.adapter, mode: "local" } });
  render(<UiActionContext.Provider value={registry}><MeetingRecording /></UiActionContext.Provider>);
  fireEvent.click(screen.getByText("Record a meeting")); await screen.findByText("Transcribe / retry");
  const parent = registry.groups().groups.find(g => g.startsWith("meeting-recording:"))!;
  const transcribe = registry.observe(parent).actions.find(a => a.label === "Transcribe retained meeting audio")!;
  let operation!: ReturnType<typeof registry.act>;
  await act(async () => { operation = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: transcribe.handle, arguments: { id: "record" }, request_id: "transcribe" }); await Promise.resolve(); });
  const read = registry.observe(parent).actions.find(a => a.label === "Read meeting recording state bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected live status reading");
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).transcribing).toBe("record");
  for (const [open, id] of [[false, "close"], [true, "reopen"]] as const) {
    const visibility = registry.observe(parent).actions.find(a => a.label === "Set meeting recording visibility")!;
    await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: visibility.handle, arguments: { open }, request_id: id })).toMatchObject({ status: "result" }); });
  }
  const cancel = registry.observe(parent).actions.find(a => a.label === "Cancel meeting transcription")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: cancel.handle, arguments: {}, request_id: "cancel" })).toMatchObject({ outcome: { value: { cancellation_requested: "record", host: { coverage: "Actual host cancellation response" } } } }); });
  expect(request).toHaveBeenCalledWith("POST", "/api/meetings/recordings/record/cancel", {});
  await act(async () => { reject(Error("Local transcription cancelled; audio retained")); await operation; });
  expect((await screen.findByRole("alert")).textContent).toContain("audio retained");
});
</book:entry>

</book:doc>
