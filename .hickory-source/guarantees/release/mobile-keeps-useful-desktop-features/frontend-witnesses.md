<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Check draft ordering and native transport ownership

For maintainers reviewing failure recovery before device checks. Deferred writes make save races observable; a native channel stand-in exercises closing during setup without requiring a running Tauri application.

## Draft save races

Two snapshots submitted while the first save is pending must reach storage in order with successive accepted revisions. A failed write does not advance the revision, so a later save preserves the latest text without claiming that the failed write succeeded.

<mobile:copy id="mobile-frontend-witnesses-draft-save-races">
import { afterEach, describe, expect, it, vi } from "vitest";
import { DraftWriter, NativeSocket, type Draft } from "./bridge";

const draft: Draft = { id: "one", title: "Notes", source: "first", revision: 0, remote: null, y_state: null };
afterEach(() => { delete (window as unknown as { __TAURI__?: unknown }).__TAURI__; });
describe("mobile recovery saves", () => {
  it("orders pending saves using only accepted revisions", async () => {
    let release!: () => void;
    const wait = new Promise<void>(resolve => { release = resolve; });
    const snapshots: Draft[] = [];
    const writer = new DraftWriter(draft, async value => { snapshots.push(value); if (value.revision === 0) await wait; return { ...value, revision: value.revision + 1 }; });
    const first = writer.save(draft);
    const second = writer.save({ ...draft, source: "latest 🦀" });
    await Promise.resolve(); await Promise.resolve();
    expect(snapshots).toHaveLength(1);
    release(); await first; await second;
    expect(snapshots.map(d => [d.source,d.revision])).toEqual([["first",0],["latest 🦀",1]]);
  });
  it("keeps a failed save visible and retries the latest text at the accepted revision", async () => {
    const write = vi.fn().mockRejectedValueOnce(Error("disk full")).mockImplementation(async (value: Draft) => ({ ...value, revision: value.revision + 1 }));
    const writer = new DraftWriter(draft, write);
    await expect(writer.save(draft)).rejects.toThrow("disk full");
    await expect(writer.flush()).rejects.toThrow("disk full");
    await expect(writer.save({ ...draft, source: "recover me" })).resolves.toMatchObject({ source: "recover me", revision: 1 });
    expect(write.mock.calls[1][0].revision).toBe(0);
  });
});
</mobile:copy>

## Late room setup

Closing before native room setup returns must retire that exact room rather than reopen it. The adapter must not send bytes after that close, even if setup eventually succeeds.

<mobile:copy id="mobile-frontend-witnesses-late-room-setup">
it("retires a room closed while native setup was pending", async () => {
  let release!: () => void;
  const setup = new Promise<void>(resolve => { release = resolve; });
  const invoke = vi.fn((command: string) => command === "mobile_room_open" ? setup : Promise.resolve());
  (window as unknown as { __TAURI__: unknown }).__TAURI__ = { core: { invoke, Channel: class { onmessage = () => {}; } } };
  const socket = new NativeSocket("ws://localhost/api/ws?doc=doc%3Aa", () => {});
  const opened = vi.fn(); socket.onopen = opened;
  socket.close(); socket.send(new Uint8Array([0,1]));
  release(); await Promise.resolve(); await Promise.resolve();
  expect(opened).not.toHaveBeenCalled();
  expect(invoke.mock.calls.filter(([command]) => command === "mobile_room_send")).toHaveLength(0);
  expect(invoke.mock.calls.filter(([command]) => command === "mobile_room_close").length).toBeGreaterThan(0);
});
</mobile:copy>

Assemble `apps/web/src/mobile/bridge.test.ts` in the order explained above.

<mobile:file path="apps/web/src/mobile/bridge.test.ts"><mobile:paste select="#mobile-frontend-witnesses-draft-save-races"/><mobile:paste select="#mobile-frontend-witnesses-late-room-setup"/></mobile:file>

## Shared protocol reconnect

The mobile transport seam still sends the existing Yjs handshake and reacts to server state. A queued retry must not reconnect after its owner closed the room; this protects a newer editor from an obsolete native room.

<mobile:copy id="mobile-frontend-witnesses-shared-protocol-reconnect">
import { afterEach, expect, it, vi } from "vitest";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { WsRealtime, type RealtimeSocket } from "../api/realtime";

class Socket implements RealtimeSocket {
  readyState = 0; binaryType = "";
  onopen: RealtimeSocket["onopen"] = null;
  onmessage: RealtimeSocket["onmessage"] = null;
  onclose: RealtimeSocket["onclose"] = null;
  send = vi.fn();
  close() { this.readyState = 3; this.onclose?.(new CloseEvent("close")); }
}
afterEach(() => vi.useRealTimers());
it("uses the supplied native transport and cancels retries after close", () => {
  vi.useFakeTimers();
  const sockets: Socket[] = [];
  const factory = vi.fn(() => { const socket = new Socket(); sockets.push(socket); return socket; });
  const realtime = new WsRealtime("doc:a", factory);
  const doc = new Y.Doc(); const awareness = new Awareness(doc);
  realtime.bindDoc(doc, awareness);
  const first = sockets[0]; first.readyState = 1; first.onopen?.(new Event("open"));
  expect(first.send).toHaveBeenCalled();
  expect((first.send.mock.calls[0] as unknown as [Uint8Array])[0][0]).toBe(0);
  first.close(); realtime.close(); vi.advanceTimersByTime(2000);
  expect(factory).toHaveBeenCalledTimes(1);
  awareness.destroy(); doc.destroy();
});
</mobile:copy>

Assemble `apps/web/src/mobile/transport.test.ts` in the order explained above.

<mobile:file path="apps/web/src/mobile/transport.test.ts"><mobile:paste select="#mobile-frontend-witnesses-shared-protocol-reconnect"/></mobile:file>

## Offline browser flow

This browser witness checks the narrow-screen local authoring flow, restart recovery, disabled desktop-only actions, and absence of horizontal page overflow. Native storage and peer sockets are exercised by separate host tests, not inferred from this preview.

<mobile:copy id="mobile-frontend-witnesses-offline-browser-flow">
import { test, expect } from "@playwright/test";

test("mobile local editing survives reload without a desktop", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/mobile.html");
  await expect(page.getByText(/Browser preview/)).toBeVisible();
  await page.getByRole("button", { name: "New document", exact: true }).click();
  await page.getByLabel("Document title").fill("Field notes");
  const editor = page.locator(".cm-content[contenteditable=true]");
  await editor.click(); await page.keyboard.insertText("Offline notes 🦀");
  await expect(page.getByRole("status")).toHaveText("Saved in this browser");
  await expect(page.getByRole("button", { name: "Run on desktop" })).toBeDisabled();
  await page.reload();
  await page.getByRole("button", { name: /Field notes/ }).click();
  await expect(editor).toContainText("Offline notes 🦀");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole("button", { name: "Documents", exact: true }).click();
  await page.getByText("Connect a desktop", { exact: true }).click();
  await expect(page.getByRole("button", { name: "Pair", exact: true })).toBeDisabled();
});
</mobile:copy>

Assemble `apps/web/e2e/mobile.spec.ts` in the order explained above.

<mobile:file path="apps/web/e2e/mobile.spec.ts"><mobile:paste select="#mobile-frontend-witnesses-offline-browser-flow"/></mobile:file>

</mobile:doc>
