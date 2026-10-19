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
