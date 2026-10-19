// @vitest-environment jsdom
// Guarantee: docs/guarantees/authoring/meeting-import-preserves-source.md
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { MeetingImport } from "./MeetingImport";
const request = vi.fn();
vi.mock("../api/client", () => ({ request: (...args: unknown[]) => request(...args) }));
afterEach(() => { cleanup(); request.mockReset(); });
function select(name: string, text = "Sam: hi") {
  const file = new File([text], name);
  Object.defineProperty(file, "arrayBuffer", { value: async () => new TextEncoder().encode(text).buffer });
  fireEvent.change(screen.getByLabelText("Transcript file"), { target: { files: [file] } });
}
it("imports exact bytes, preserves unknown speakers and navigates retained evidence", async () => {
  request.mockResolvedValue({ path: "meeting.hick", source: "inbox/ingested/meeting.vtt", duplicate: false,
    format: "vtt", utterances: [{ id: "u1", speaker: null, at: "00:01", text: "hi", start_utf8: 0, end_utf8: 2, line: 4 }] });
  const open = vi.fn(); render(<MeetingImport onOpen={open} />);
  fireEvent.click(screen.getByText("Import meeting transcript"));
  select("meeting.vtt", "Sam: hi\r\n");
  fireEvent.click(screen.getByRole("button", { name: "Import transcript" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/meetings/import", { name: "meeting.vtt", text: "Sam: hi\r\n" }));
  await screen.findByText(/Unknown speaker/);
  fireEvent.click(screen.getByRole("button", { name: "Open source for u1" }));
  expect(open).toHaveBeenCalledWith("inbox/ingested/meeting.vtt", 4);
});
it("cancels before import and rejects audio without calling the server", async () => {
  render(<MeetingImport onOpen={vi.fn()} />);
  fireEvent.click(screen.getByText("Import meeting transcript"));
  select("meeting.txt");
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(request).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText("Import meeting transcript"));
  select("meeting.mp4");
  fireEvent.click(screen.getByRole("button", { name: "Import transcript" }));
  await screen.findByRole("alert");
  expect(request).not.toHaveBeenCalled();
});
it("reports existing note on duplicate and surfaces server failures", async () => {
  request.mockRejectedValueOnce(new Error("Malformed transcript"));
  render(<MeetingImport onOpen={vi.fn()} />);
  fireEvent.click(screen.getByText("Import meeting transcript"));
  select("meeting.txt");
  fireEvent.click(screen.getByRole("button", { name: "Import transcript" }));
  await screen.findByText(/Malformed transcript/);
  request.mockResolvedValueOnce({ path: "meeting.hick", source: "inbox/meeting.txt", duplicate: true, format: "lines", utterances: [] });
  fireEvent.click(screen.getByRole("button", { name: "Import transcript" }));
  await screen.findByText(/Already imported/);
});
