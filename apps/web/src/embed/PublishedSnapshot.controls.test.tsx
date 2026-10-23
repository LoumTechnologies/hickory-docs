// Guarantee: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { PublishedSnapshot, type ViewerSnapshot } from "./PublishedSnapshot";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { sha256 } from "../lib/reviews";
const { hosts } = vi.hoisted(() => ({ hosts: [] as { resolveAsset: (path: string, signal: AbortSignal) => Promise<string> }[] }));
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ host }: { host: typeof hosts[number] }) => { hosts.push(host); return <div>Readonly document fixture</div>; } }));
afterEach(() => { cleanup(); hosts.length = 0; vi.restoreAllMocks(); vi.unstubAllGlobals(); });
const snapshot = (status: "passed" | "failed" = "passed"): ViewerSnapshot => ({ format: "hickory-viewer-snapshot", version: 1, path: "spec.md", source: "# Captured", source_hash: "hash", captured_at_ms: 0, execution: "recorded", cells: [],
  verification: { status, revision: "a".repeat(40), source_hash: "hash", dependencies: {}, assets: {}, environment: { hidden: "private environment" },
    commands: [{ command: ["run", "check"], exit_code: status === "passed" ? 0 : 1, stdout: "recorded stdout" }], requirements: [], origin: "supplied fixture", scope: "historical only" } });
it("shares publication disclosure and transcript state and resets a replacement capture", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const draw = (value: ViewerSnapshot) => <UiActionContext.Provider value={registry}><PublishedSnapshot snapshot={value} /></UiActionContext.Provider>;
  const view = render(draw(snapshot()));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = (label: string) => actions().find(action => action.label === label)!.handle;
  const run = async (target: string, args: Record<string, boolean | number>) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: target, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(actions().some(action => action.label === "Read disclosed publication evidence bytes")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("private environment");
  await run(handle("Set publication evidence visibility: Environment and dependency identities"), { open: true });
  const reading = handle("Read disclosed publication evidence bytes");
  const bytes = await run(reading, { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(bytes.outcome.value.bytes))).toContain("private environment");
  const fold = screen.getByText("Environment and dependency identities").closest("details")!;
  fold.open = false; fireEvent(fold, new Event("toggle"));
  expect(await run(reading, { offset: 0, limit: 8192 })).not.toMatchObject({ status: "result" });
  await run(handle("Set published execution transcript visibility"), { shown: false });
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
  const stale = handle("Set published execution transcript visibility");
  view.rerender(draw(snapshot("failed")));
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(true);
  expect(screen.getByText("recorded stdout").closest("details")!.open).toBe(true);
  expect(await run(stale, { shown: false })).not.toMatchObject({ status: "result" });
});
it("revokes a verified asset completing after its publication capture retires", async () => {
  const create = vi.fn(() => "blob:retired-asset"), revoke = vi.fn();
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL: create, revokeObjectURL: revoke }));
  const value = snapshot(); value.verification!.assets["asset.png"] = await sha256("asset bytes");
  let resolve!: (response: Response) => void;
  vi.spyOn(globalThis, "fetch").mockImplementation(() => new Promise(done => { resolve = done; }));
  const view = render(<PublishedSnapshot snapshot={value} artifactUrl={`${location.origin}/publication/`} />);
  const pending = hosts.at(-1)!.resolveAsset("asset.png", new AbortController().signal);
  view.unmount();
  resolve(new Response("asset bytes", { headers: { "Content-Type": "image/png" } }));
  await expect(pending).rejects.toThrow("capture was retired");
  expect(create).toHaveBeenCalledTimes(1); expect(revoke).toHaveBeenCalledExactlyOnceWith("blob:retired-asset");
});
it("renders saved text and SVG safely without fetching or running a cell", () => {
  const fetch = vi.spyOn(globalThis, "fetch");
  const value = snapshot();
  const svg = btoa('<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>');
  value.cells = [{ kind: "exec", id: "rich", command: "print('do not execute')", container: "python", span: [0, 1], status: "unrecorded", saved_output: `recorded text\n\x1eHICKORY_DISPLAY ${JSON.stringify({ kind: "image", mime: "image/svg+xml", data: svg })}\n` }];
  render(<PublishedSnapshot snapshot={value} />);
  expect(screen.getByText("recorded text")).toBeTruthy();
  const image = screen.getByRole("img", { name: "Python cell output" });
  expect(image.getAttribute("src")).toBe(`data:image/svg+xml;base64,${svg}`);
  expect(document.querySelector("script")).toBeNull();
  expect(fetch).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("checkbox"));
  expect(screen.queryByText("recorded text")).toBeNull();
  expect(screen.queryByRole("img")).toBeNull();
});
