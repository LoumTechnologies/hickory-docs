import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditorView } from "@codemirror/view";
// Unsaved work coming back, and what happens when the file moved on while the
// app was closed.
//
// Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
//
// Driven through the pane against a mocked API client, because the decision
// being tested — restore quietly, drop a stale draft, or open a merge — is
// made from what those two requests answer.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { PlainFilePane } from "./PlainFilePane";
import { undo } from "@codemirror/commands";
import { api } from "../api/client";
import * as apiClient from "../api/client";
import type { RecordedPatchSource } from "../simple/AgentProposalAdapter";
import { authoredSource } from "../simple/SourceText";
import { historySourceHash } from "../simple/SourceHistory";
import type { WorkspaceDraft } from "../api/types";
import type { StartupPlainFile } from "../views/folderPane";
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
  vi.unstubAllGlobals();
});

it("promotes the same startup editor and includes its physical guard in later saves", async () => {
  const path = "startup.py", namespace = "/workspace", workspace = "a".repeat(32), content = "value = 1\n";
  let confirm!: (value: Awaited<StartupPlainFile["authority"]>) => void;
  const authority = new Promise<Awaited<StartupPlainFile["authority"]>>(resolve => { confirm = resolve; });
  const save = vi.spyOn(api, "saveFile").mockRejectedValue(Object.assign(Error("Source file was replaced"), {status: 409}));
  const startup: StartupPlainFile = {source: {version: 1, path, content, root_path: namespace, workspace_id: workspace},
    authority, current: () => true};
  const mounted = render(<PlainFilePane path={path} startup={startup} />);
  const editor = EditorView.findFromDOM(mounted.container.querySelector<HTMLElement>(".cm-editor")!)!;
  expect(editor.state.doc.toString()).toBe(content);
  expect(editor.state.readOnly).toBe(true);
  act(() => editor.dispatch({changes: {from: 0, insert: "blocked"}, userEvent: "input.type", selection: {anchor: 2}}));
  expect(editor.state.doc.toString()).toBe(content);
  expect(save).not.toHaveBeenCalled();
  act(() => editor.dispatch({selection: {anchor: 2, head: 7}}));
  const selection = editor.state.selection;
  expect(selection.main.empty).toBe(false);

  await act(async () => confirm({file: {path, content, language: "python", hash: "base", save_guard: "original-source"},
    namespace, workspace, generatedBy: null}));
  expect(EditorView.findFromDOM(mounted.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
  expect(editor.state.readOnly).toBe(false);
  expect(editor.state.selection.eq(selection)).toBe(true);
  act(() => editor.dispatch({changes: {from: 8, to: 9, insert: "2"}, userEvent: "input.type"}));
  await waitFor(() => expect(save).toHaveBeenCalledWith(path, "value = 2\n", "base", false, "original-source"));
  expect(editor.state.doc.toString()).toBe("value = 2\n");
  expect(save).toHaveBeenCalledTimes(1);
});

describe("reopening a file that had unsaved changes", () => {
  it("keeps a private marked recovery draft off the ordinary autosave path", async () => {
    const recovered = draft({ detached: true });
    serve({ content: ON_DISK, drafts: [recovered] });
    const view = render(<PlainFilePane path="notes.md" retainUnsaved />);
    await screen.findByText("Unsaved draft");
    const editor = EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)!;
    expect(authoredSource(editor.state)).toBe(recovered.contents);
    await act(async () => { await new Promise(resolve => setTimeout(resolve, 650)); });
    expect(api.saveFile).not.toHaveBeenCalled();
    await act(async () => view.unmount());
    expect(api.saveDraft).toHaveBeenCalledWith(expect.objectContaining({ contents: recovered.contents, base: ON_DISK, detached: true }));
    expect(api.saveFile).not.toHaveBeenCalled();
  });

  it("prepares a byte-exact detached transaction and publishes only the explicit guarded Save", async () => {
    const base = "one\r\ntwo 😀\r\nthree\n", after = "one\r\nchanged 😀\r\nthree\n";
    const path = "notes.md", workspace = "a".repeat(32);
    serve({ content: base, drafts: [] });
    vi.mocked(api.file).mockResolvedValue({path, content: base, language: "markdown", hash: "original-hash", save_guard: "original-guard"});
    vi.mocked(api.files).mockResolvedValue({root: "notebook", root_path: "/workspace", workspace_id: workspace, tree: [{name: path, path, dir: false}]});
    const originalRequest = apiClient.request;
    const publish = vi.fn(async (_method: string, _route: string, raw?: unknown) => {
      const body = raw as {request_id: string; content: string};
      return {version: 1, history_protocol: 1, path, hash: "saved-hash", save_guard: "next-guard", request_id: body.request_id,
        source_hash: await historySourceHash(body.content), history: {available: true, outcome: "published", id: body.request_id}};
    });
    vi.spyOn(apiClient, "request").mockImplementation(async <T,>(method: string, route: string, raw?: unknown) =>
      route === "/api/source-history/file" ? await publish(method, route, raw) as T : originalRequest<T>(method, route, raw));
    let read = (): RecordedPatchSource | null => null;
    let save = async () => false;
    const view = render(<PlainFilePane path={path} proposalTabId="original-tab"
      proposalOwner={(_tab, owner) => { read = owner; return () => {}; }}
      onUnsaved={(_dirty, actions) => { save = actions.save; }} />);
    await waitFor(() => expect(read()?.prepareDraft?.(after)?.current()).toBe(true));
    const editor = EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)!;
    act(() => editor.dispatch({selection: {anchor: 1, head: 3}}));
    const originalSelection = editor.state.selection;
    const aborted = read()!.prepareDraft!(after)!;
    act(() => { aborted.commit(); aborted.rollback(); });
    expect(EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
    expect(editor.state.selection.eq(originalSelection)).toBe(true);
    expect(authoredSource(editor.state)).toBe(base);
    act(() => { expect(undo(editor)).toBe(false); });
    act(() => read()!.prepareDraft!(after)!.commit());
    expect(authoredSource(editor.state)).toBe(after);
    expect(EditorView.findFromDOM(view.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
    act(() => { expect(undo(editor)).toBe(true); });
    expect(authoredSource(editor.state)).toBe(base);
    act(() => read()!.prepareDraft!(after)!.commit());
    await act(async () => { await new Promise(resolve => setTimeout(resolve, 650)); });
    expect(api.saveFile).not.toHaveBeenCalled();
    expect(publish).not.toHaveBeenCalled();
    await act(async () => { expect(await save()).toBe(true); });
    expect(publish).toHaveBeenCalledOnce();
    expect(publish).toHaveBeenCalledWith("PUT", "/api/source-history/file", expect.objectContaining({
      version: 1, path, content: after, base_hash: "original-hash", save_guard: "original-guard",
    }));
    expect(read()?.base).toBe(after);
    expect(read()?.guard).toBe("next-guard");
    expect(api.saveFile).not.toHaveBeenCalled();
  });

  it("adopts a confirmed saved resource baseline and retains its separate draft in place", async () => {
    const path = "notes.md", base = "first\r\nsecond\r\n";
    const saved = "first\r\nsaved\r\n", draftText = "first\r\nprivate 😀\r\n";
    serve({content: base, drafts: []});
    vi.mocked(api.file).mockResolvedValue({path, content: base, language: "markdown", hash: "before-hash", save_guard: "before-guard"});
    vi.mocked(api.files).mockResolvedValue({root: "notebook", root_path: "/workspace", workspace_id: "a".repeat(32),
      tree: [{path, name: path, dir: false}]});
    let read = (): RecordedPatchSource | null => null;
    let dirty = false;
    const mounted = render(<PlainFilePane path={path} proposalTabId="file"
      proposalOwner={(_tab, owner) => { read = owner; return () => {}; }}
      onUnsaved={value => { dirty = value; }} />);
    await waitFor(() => expect(read()?.prepareResource?.(draftText, saved)?.current()).toBe(true));
    const editor = EditorView.findFromDOM(mounted.container.querySelector<HTMLElement>(".cm-editor")!)!;
    act(() => editor.dispatch({selection: {anchor: 1}}));
    const staged = read()!.prepareResource!(draftText, saved)!;
    act(() => staged.adopt({hash: "confirmed-hash", guard: "confirmed-guard"}));
    expect(staged.current()).toBe(false);
    expect(authoredSource(editor.state)).toBe(draftText);
    expect(editor.state.selection.main.anchor).toBe(1);
    expect(EditorView.findFromDOM(mounted.container.querySelector<HTMLElement>(".cm-editor")!)).toBe(editor);
    expect(read()).toMatchObject({base: saved, guard: "confirmed-guard", text: draftText});
    expect(dirty).toBe(true);
    await act(async () => { await new Promise(resolve => setTimeout(resolve, 650)); });
    expect(api.saveFile).not.toHaveBeenCalled();
  });

  it("puts the unsaved text back, without asking", () => {
    // The common case by a wide margin. A dialog here would train people to
    // dismiss dialogs, and the work is theirs — nothing was lost or decided.
    serve({ content: ON_DISK, drafts: [draft()] });
    render(<PlainFilePane path="notes.md" retainUnsaved />);
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
    render(<PlainFilePane path="notes.md" retainUnsaved />);
    return waitFor(() => expect(discard).toHaveBeenCalledWith("notes.md"));
  });

  it("opens a merge when the file moved on and the buffer did too", () => {
    serve({
      content: "one\nTHEIRS\nthree\n",
      drafts: [draft({ contents: "one\nMINE\nthree\n", base: ON_DISK })],
    });
    render(<PlainFilePane path="notes.md" retainUnsaved />);
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
      expect(screen.getByRole("button", { name: "Debug tools/app.py" })).toBeDefined();
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
    expect(screen.queryByRole("button", { name: "Debug tools/app.py" })).toBeNull();
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

  it("starts from the gutter at the file's own path and exposes stepping", async () => {
    const { sent, deliver } = debugWire();
    // The debugger hashes the current saved buffer and checks its workspace
    // before sending Start. Use the same namespace in the tree and catalog.
    const cryptoModule = "node:" + "crypto";
    const { webcrypto } = await import(cryptoModule);
    vi.stubGlobal("crypto", webcrypto);
    const workspace = "/repo";
    vi.spyOn(apiClient, "request").mockResolvedValue({
      debug_protocol: 3,
      fingerprint: "plain-project", viewed: "tools/app.py", workspace,
      configuration: ".hick-launch.json",
      profiles: [{
        id: "app:tools/app.py", name: "Application", origin: "configured",
        program: "tools/app.py", command: [], args: [], env: {}, cwd: "repo",
        build: [], default: true, can_debug: true,
      }],
    });
    serveFile("tools/app.py", "python", APP);
    vi.mocked(api.files).mockResolvedValue({root: "repo", root_path: workspace,
      tree: [{name: "app.py", path: "tools/app.py", dir: false}]});
    render(<><PlainDebugHosts /><PlainFilePane path="tools/app.py" /></>);
    const button = await screen.findByRole("button", { name: "Debug tools/app.py" });
    button.click();
    await waitFor(() =>
      expect(sent[0]).toMatchObject({ op: "start", doc: "hick:///tools/app.py", breakpoints: [], profile: {id: "app:tools/app.py", fingerprint: "plain-project"} }),
    );
    // The current engine identifies both the caller of Start and its saved bytes.
    const { request_id, source_hash } = sent[0];
    // An answer for ANOTHER pane's file is not this pane's business.
    deliver({
      event: "started",
      session: "dbg-7",
      doc: "hick:///other.py",
      request_id,
      source_hash,
      capabilities: {},
      breakpoints: [],
    });
    deliver({
      event: "started",
      session: "dbg-0",
      doc: "hick:///tools/app.py",
      request_id,
      source_hash,
      capabilities: {},
      breakpoints: [],
    });
    deliver({
      event: "stopped",
      session: "dbg-0",
      stop: 1,
      thread: 1,
      reason: "breakpoint",
      line: 3,
      frames: [{ id: 1, name: "main", line: 3, source: "tools/app.py", in_document: true }],
      variables: [],
    });
    await waitFor(() => {
      const strip = screen.getByRole("toolbar", { name: "Debugger" });
      expect(strip.textContent).toContain("paused");
    });
    // The launch action stays in the gutter; stepping uses the session controls.
    expect(screen.getByRole("button", { name: "Debug tools/app.py" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Step over" })).toBeDefined();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reopens the merge against newer disk text instead of applying an old decision", async () => {
  serve({ content: "one\nTHEIRS\nthree\n", drafts: [draft()] });
  render(<PlainFilePane path="notes.md" retainUnsaved />);
  await screen.findByTestId("merge-view");
  await waitFor(() => expect((screen.getByRole("button", { name: /accept, keeping mine/i }) as HTMLButtonElement).disabled).toBe(false));
  vi.mocked(api.file).mockResolvedValue({ path: "notes.md", language: "markdown", content: "one\nNEW DISK\nthree\n", hash: "new-disk" });
  fireEvent.click(screen.getByRole("button", { name: /accept, keeping mine/i }));
  await waitFor(() => expect(screen.getByTestId("merge-view").textContent).toContain("NEW DISK"));
  expect(api.saveFile).not.toHaveBeenCalled();
  expect(api.discardDraft).not.toHaveBeenCalled();
});

it("applies a reviewed merge draft only after the observed disk side still matches", async () => {
  serve({ content: "one\nTHEIRS\nthree\n", drafts: [draft()] });
  render(<PlainFilePane path="notes.md" retainUnsaved />);
  await screen.findByTestId("merge-view");
  await waitFor(() => expect((screen.getByRole("button", { name: /accept, keeping mine/i }) as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(screen.getByRole("button", { name: /accept, keeping mine/i }));
  await waitFor(() => expect(screen.queryByTestId("merge-view")).toBeNull());
  expect(document.querySelector(".cm-content")?.textContent).toContain("MINE");
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps newer buffer edits when an explicit disk reload returns late", async () => {
  serve({ content: ON_DISK, drafts: [] });
  vi.mocked(api.saveFile).mockRejectedValue(Object.assign(new Error("disk changed"), { status: 409 }));
  render(<PlainFilePane path="notes.md" />);
  await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
  const editor = EditorView.findFromDOM(document.querySelector<HTMLElement>(".cm-editor")!)!;
  act(() => { editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "FIRST EDIT" }, userEvent: "input" }); });
  vi.mocked(api.file).mockResolvedValue({ path: "notes.md", language: "markdown", content: "new disk", hash: "new-disk-hash" });
  fireEvent(window, new Event("hickory:files-changed"));
  await screen.findByRole("button", { name: "Reload and discard unsaved changes" });
  let finish!: (value: Awaited<ReturnType<typeof api.file>>) => void;
  vi.mocked(api.file).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  fireEvent.click(screen.getByRole("button", { name: "Reload and discard unsaved changes" }));
  act(() => { editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "NEWER EDIT" }, userEvent: "input" }); });
  await act(async () => { finish({ path: "notes.md", language: "markdown", content: "new disk", hash: "new-disk-hash" }); });
  await screen.findByText(/newer edits were kept/);
  expect(editor.state.doc.toString()).toBe("NEWER EDIT");
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains actual adoption success if opening its created document fails", async () => {
  serve({ content: ON_DISK, drafts: [] });
  const adopted = { doc_id: "created-doc", doc_path: "notes.literate.md", file_path: "notes.md", output_path: "notes.md", created: true };
  const adopt = vi.spyOn(api, "adopt").mockResolvedValue(adopted);
  const onAdopted = vi.fn(() => { throw new Error("navigation refused"); });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PlainFilePane path="notes.md" onAdopted={onAdopted} /></UiActionContext.Provider>);
  await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
  await act(async () => { fireEvent.click(screen.getByLabelText("More actions for notes.md")); });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  const handle = actions().find(a => a.label === "Make current file literate")!.handle;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle, arguments: {}, request_id: "adopt" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { owner_reply: JSON.stringify({ ...adopted, navigation_error: "navigation refused" }) } } });
  expect(adopt).toHaveBeenCalledWith("notes.md");
  expect(onAdopted).toHaveBeenCalledWith(adopted);
  expect(screen.getByText(/Created notes.literate.md, but could not open it/)).toBeTruthy();
  expect(screen.getByText(/already literate/)).toBeTruthy();
  expect(actions().some(a => a.label === "Make current file literate")).toBe(false);
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("agent editor input uses the existing guarded plain-file autosave pipeline", async () => {
  serve({ content: ON_DISK, drafts: [] });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PlainFilePane path="notes.md" /></UiActionContext.Provider>);
  await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("two"));
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  await waitFor(() => expect(actions().some(a => a.label === "Replace file editor range")).toBe(true));
  const replace = actions().find(a => a.label === "Replace file editor range")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: replace.handle, arguments: { from: 4, to: 7, text: "AGENT" }, request_id: "input" }); });
  expect(result).toMatchObject({ status: "result", outcome: { value: { source_changed: true, approved: false, verified: false } } });
  await waitFor(() => expect(api.saveFile).toHaveBeenCalledWith("notes.md", "one\nAGENT\nthree\n", `h:${ON_DISK}`, false));
  expect(screen.getByText("Saved")).toBeTruthy();
});
