// Unsaved work coming back, and what happens when the file moved on while the
// app was closed.
//
// Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
//
// Driven through the pane against a mocked API client, because the decision
// being tested — restore quietly, drop a stale draft, or open a merge — is
// made from what those two requests answer.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";

import { PlainFilePane } from "./PlainFilePane";
import { api } from "../api/client";
import type { WorkspaceDraft } from "../api/types";
import { setSharedRealtime, type Realtime } from "../api/realtime";
import { createLspChannel, encodeLspFrame, type JsonRpcMessage } from "../lsp/channel";
import { resetWorkspaceLsp } from "../lsp/useLsp";
import { resetWorkspaceDebugger } from "../debug/useDebugger";
import { publishPausedElsewhere, resetPausedElsewhere } from "../lib/pausedElsewhere";
import { PlainDebugHosts, resetPlainDebugHosts } from "../debug/plainDebugHosts";
import { allFileProblems, resetFileProblems } from "../lib/fileProblems";

const ON_DISK = "one\ntwo\nthree\n";

function serve({ content, drafts }: { content: string; drafts: WorkspaceDraft[] }) {
  vi.spyOn(api, "file").mockResolvedValue({
    path: "notes.md",
    language: "markdown",
    content,
    hash: `h:${content}`,
  });
  vi.spyOn(api, "drafts").mockResolvedValue({ drafts });
  vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "saveFile").mockResolvedValue({ path: "notes.md", hash: "h:saved" });
  vi.spyOn(api, "files").mockResolvedValue({
    root: "notebook",
    tree: [{ name: "notes.md", path: "notes.md", dir: false }],
  });
}

const draft = (over: Partial<WorkspaceDraft> = {}): WorkspaceDraft => ({
  path: "notes.md",
  contents: "one\nMINE\nthree\n",
  base: ON_DISK,
  saved_at: 1,
  ...over,
});

beforeEach(() => {
  vi.restoreAllMocks();
  resetWorkspaceLsp();
  resetWorkspaceDebugger();
  resetPausedElsewhere();
  resetPlainDebugHosts();
  resetFileProblems();
});

afterEach(() => {
  cleanup();
});

describe("reopening a file that had unsaved changes", () => {
  it("puts the unsaved text back, without asking", () => {
    // The common case by a wide margin. A dialog here would train people to
    // dismiss dialogs, and the work is theirs — nothing was lost or decided.
    serve({ content: ON_DISK, drafts: [draft()] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("MINE");
      expect(screen.queryByTestId("merge-view")).toBeNull();
    });
  });

  it("throws away a draft the file already agrees with", () => {
    // Somebody saved the same text from elsewhere; there is nothing unsaved.
    const discard = vi.fn().mockResolvedValue({ ok: true });
    serve({ content: ON_DISK, drafts: [draft({ contents: ON_DISK, base: "older\n" })] });
    vi.spyOn(api, "discardDraft").mockImplementation(discard);
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => expect(discard).toHaveBeenCalledWith("notes.md"));
  });

  it("opens a merge when the file moved on and the buffer did too", () => {
    serve({
      content: "one\nTHEIRS\nthree\n",
      drafts: [draft({ contents: "one\nMINE\nthree\n", base: ON_DISK })],
    });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      const merge = screen.getByTestId("merge-view");
      expect(merge.textContent).toMatch(/changed while you were away/i);
      expect(merge.textContent).toMatch(/still needs you/i);
    });
  });

  it("opens normally when there is no draft for this file", () => {
    serve({ content: ON_DISK, drafts: [draft({ path: "somewhere/else.md" })] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("two");
      expect(screen.queryByTestId("merge-view")).toBeNull();
    });
  });

  it("opens normally when the draft store is unreachable", () => {
    // A machine with no data directory, or a read-only home. The file opens
    // as it is on disk, which is what would have happened anyway.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "drafts").mockRejectedValue(new Error("no store"));
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("two");
    });
  });
});

describe("a file that is already the output of a document", () => {
  it("is not offered a button to make it literate", () => {
    // It already is. Offering the verb tells the reader their document is not
    // what it plainly is.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "files").mockResolvedValue({
      root: "notebook",
      tree: [{ name: "notes.md", path: "notes.md", dir: false, generated_by: "d1" }],
    });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(screen.queryByRole("button", { name: /make literate/i })).toBeNull();
      expect(screen.getByText(/already literate/i)).toBeTruthy();
    });
  });

  it("keeps the button for a file nobody writes", () => {
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() =>
      expect(screen.getByRole("button", { name: /make literate/i })).toBeTruthy(),
    );
  });

  it("keeps the button when the listing is unavailable", () => {
    // The adopt route's own refusal is the backstop; a failed listing must
    // not remove a verb that might be valid.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "files").mockRejectedValue(new Error("no listing"));
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() =>
      expect(screen.getByRole("button", { name: /make literate/i })).toBeTruthy(),
    );
  });
});

// Protects docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md
describe("a plain file and the language server", () => {
  /** A realtime whose language channel is a loopback wire this test holds
   * both ends of. Not server-authoritative, which is how the workspace
   * connection is told to reuse it rather than open a socket. */
  function wire() {
    const outbound: JsonRpcMessage[] = [];
    const channel = createLspChannel({
      send: (frame) => {
        outbound.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
      },
    });
    const realtime: Realtime = {
      serverAuthoritative: false,
      whenSynced: () => Promise.resolve(),
      bindDoc: () => {},
      onRunEvent: () => () => {},
      lsp: () => channel,
      close: () => {},
      reopen: () => {},
    };
    setSharedRealtime(realtime);
    const inject = (msg: JsonRpcMessage) => channel.handleFrame(encodeLspFrame(msg));
    return { outbound, inject };
  }

  it("opens the file with the language server at its own path, once loaded", async () => {
    const { outbound } = wire();
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    await waitFor(() => {
      const opened = outbound.find((m) => "method" in m && m.method === "textDocument/didOpen");
      expect(opened).toBeDefined();
      // The file's own path, and the text the screen shows — never an empty
      // stand-in sent before the load landed.
      expect(opened).toMatchObject({
        params: { textDocument: { uri: "hick:///notes.md", text: ON_DISK } },
      });
    });
  });

  it("puts what the server says is wrong into the file-problems store", async () => {
    const { outbound, inject } = wire();
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    await waitFor(() =>
      expect(outbound.some((m) => "method" in m && m.method === "textDocument/didOpen")).toBe(true),
    );
    inject({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: {
        uri: "hick:///notes.md",
        diagnostics: [
          {
            range: { start: { line: 1, character: 0 }, end: { line: 1, character: 3 } },
            severity: 1,
            message: "two is not a number",
          },
        ],
      },
    });
    await waitFor(() => {
      expect(allFileProblems()).toEqual([
        expect.objectContaining({
          path: "notes.md",
          diagnostics: [expect.objectContaining({ message: "two is not a number" })],
        }),
      ]);
      // And drawn in the buffer, where the reader is.
      expect(document.querySelector(".cm-lsp-diagnostic, .cm-lsp-error, [class*=\"cm-lsp\"]")).not.toBeNull();
    });
  });
});

// Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
describe("closing the pane", () => {
  it("never writes a draft of nothing for a buffer that was never edited", async () => {
    // The final flush runs after the view is destroyed. Reading "" from the
    // dead view used to record an empty draft, which the next mount restored
    // and saved — a 159-line file emptied on disk by switching tabs.
    serve({ content: ON_DISK, drafts: [] });
    const saveDraft = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
    const view = render(<PlainFilePane path="notes.md" />);
    await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
    view.unmount();
    expect(saveDraft).not.toHaveBeenCalled();
    expect(saveDraft.mock.calls.every((call) => call[0].contents !== "")).toBe(true);
  });
});

// Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
describe("a plain file and the debugger", () => {
  const APP = "import os\n\ndef main():\n    print(os.getcwd())\n\nmain()\n";

  /** A realtime whose debug channel this test holds both ends of. */
  function debugWire() {
    const sent: Record<string, unknown>[] = [];
    let handler: ((frame: Uint8Array) => boolean) | null = null;
    const realtime: Realtime = {
      serverAuthoritative: false,
      whenSynced: () => Promise.resolve(),
      bindDoc: () => {},
      onRunEvent: () => () => {},
      lsp: () => null,
      close: () => {},
      reopen: () => {},
      debug: () => ({
        send: (frame) => {
          sent.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
        },
      }),
      onDebugFrame: (h) => {
        handler = h;
      },
    };
    setSharedRealtime(realtime);
    const deliver = (event: Record<string, unknown>) => {
      const body = new TextEncoder().encode(JSON.stringify(event));
      const frame = new Uint8Array(body.length + 1);
      frame[0] = 0x03;
      frame.set(body, 1);
      handler?.(frame);
    };
    return { sent, deliver };
  }

  function serveFile(path: string, language: string, content: string) {
    vi.spyOn(api, "file").mockResolvedValue({ path, language, content, hash: `h:${content}` });
    vi.spyOn(api, "drafts").mockResolvedValue({ drafts: [] });
    vi.spyOn(api, "files").mockResolvedValue({ root: "repo", tree: [{ name: path, path, dir: false }] });
  }

  it("offers Debug and a breakpoint gutter for a language hick can debug", async () => {
    debugWire();
    serveFile("tools/app.py", "python", APP);
    render(<><PlainDebugHosts /><PlainFilePane path="tools/app.py" /></>);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Debug" })).toBeDefined();
      expect(document.querySelector(".cm-breakpoint-gutter")).not.toBeNull();
    });
  });

  it("offers neither for a file hick cannot debug", async () => {
    // A gutter that takes a dot it can never bind is a promise the pane
    // cannot keep, and a Debug button that only ever fails is worse.
    debugWire();
    serveFile("notes.md", "markdown", ON_DISK);
    render(<><PlainDebugHosts /><PlainFilePane path="notes.md" /></>);
    await waitFor(() => expect(document.querySelector(".cm-content")).not.toBeNull());
    expect(screen.queryByRole("button", { name: "Debug" })).toBeNull();
    expect(document.querySelector(".cm-breakpoint-gutter")).toBeNull();
  });

  it("draws the paused line when another pane's session is stopped in this file", async () => {
    // The pane for `app.py` owns the session and stepped into `helpers.py`.
    // This pane holds helpers.py; for reading, the paused line is its own.
    debugWire();
    serveFile("tools/helpers.py", "python", "def double(x):\n    return x * 2\n");
    render(<><PlainDebugHosts /><PlainFilePane path="tools/helpers.py" /></>);
    await waitFor(() => expect(document.querySelector(".cm-breakpoint-gutter")).not.toBeNull());
    publishPausedElsewhere("tools/app.py", { path: "tools/helpers.py", line: 1 });
    await waitFor(() => expect(document.querySelector(".cm-paused-arrow")).not.toBeNull());
    // Only the owner clears it, and it does when the program moves on.
    publishPausedElsewhere("tools/other.py", null);
    expect(document.querySelector(".cm-paused-arrow")).not.toBeNull();
    publishPausedElsewhere("tools/app.py", null);
    await waitFor(() => expect(document.querySelector(".cm-paused-arrow")).toBeNull());
  });

  it("starts at the file's own path and shows the session above the file", async () => {
    const { sent, deliver } = debugWire();
    serveFile("tools/app.py", "python", APP);
    render(<><PlainDebugHosts /><PlainFilePane path="tools/app.py" /></>);
    const button = await screen.findByRole("button", { name: "Debug" });
    button.click();
    await waitFor(() =>
      expect(sent[0]).toMatchObject({ op: "start", doc: "hick:///tools/app.py", breakpoints: [] }),
    );
    // An answer for ANOTHER pane's file is not this pane's business.
    deliver({
      event: "started",
      session: "dbg-7",
      doc: "hick:///other.py",
      capabilities: {},
      breakpoints: [],
    });
    deliver({
      event: "started",
      session: "dbg-0",
      doc: "hick:///tools/app.py",
      capabilities: {},
      breakpoints: [],
    });
    deliver({
      event: "stopped",
      session: "dbg-0",
      reason: "breakpoint",
      line: 3,
      frames: [{ id: 1, name: "main", line: 3, source: "tools/app.py", in_document: true }],
      variables: [],
    });
    await waitFor(() => {
      const strip = screen.getByRole("toolbar", { name: "Debugger" });
      expect(strip.textContent).toContain("paused");
    });
    // The way in is gone while a session runs; the strip's verbs take over.
    expect(screen.queryByRole("button", { name: "Debug" })).toBeNull();
    expect(screen.getByRole("button", { name: "Step over" })).toBeDefined();
  });
});

