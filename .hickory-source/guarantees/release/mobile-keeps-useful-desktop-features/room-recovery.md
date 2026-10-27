<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Recover offline edits without duplicating desktop text

For maintainers checking why the mobile draft stores CRDT bytes. This witness uses the shared realtime client and real Yjs sync messages against a second document, then restarts the client from its saved state while the desktop has also changed.

## In memory desktop room

The socket stand-in implements the same sync envelope as the document room, including the server’s initial state-vector request. It changes transport timing while keeping the real Yjs merge implementation.

<mobile:copy id="mobile-room-recovery-in-memory-desktop-room">
import { expect, it } from "vitest";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import * as sync from "y-protocols/sync";
import * as encoding from "lib0/encoding";
import * as decoding from "lib0/decoding";
import { WsRealtime, type RealtimeSocket } from "../api/realtime";

class RoomSocket implements RealtimeSocket {
  readyState = 0; binaryType = "arraybuffer";
  onopen: RealtimeSocket["onopen"] = null;
  onmessage: RealtimeSocket["onmessage"] = null;
  onclose: RealtimeSocket["onclose"] = null;
  constructor(private server: Y.Doc) {
    queueMicrotask(() => {
      this.readyState = 1; this.onopen?.(new Event("open"));
      const encoder = encoding.createEncoder();
      encoding.writeVarUint(encoder, 0); sync.writeSyncStep1(encoder, server);
      this.receive(encoding.toUint8Array(encoder));
    });
  }
  private receive(payload: Uint8Array) {
    queueMicrotask(() => {
      if (this.readyState !== 1) return;
      const frame = new Uint8Array(payload.length + 1); frame.set(payload, 1);
      this.onmessage?.(new MessageEvent("message", { data: frame.buffer }));
    });
  }
  send(frame: Uint8Array) {
    if (this.readyState !== 1 || frame[0] !== 0) return;
    const decoder = decoding.createDecoder(frame.subarray(1));
    if (decoding.readVarUint(decoder) !== 0) return;
    const encoder = encoding.createEncoder(); encoding.writeVarUint(encoder, 0);
    sync.readSyncMessage(decoder, encoder, this.server, this);
    if (encoding.length(encoder) > 1) this.receive(encoding.toUint8Array(encoder));
  }
  close() { this.readyState = 3; this.onclose?.(new CloseEvent("close")); }
}
</mobile:copy>

## Restart and merge

The phone starts without seeding fetched text. After the first sync it goes offline, changes its document, and saves the CRDT bytes. A new phone document restores those bytes and syncs with a desktop that changed independently. Both must converge with one copy of the original text.

<mobile:copy id="mobile-room-recovery-restart-and-merge">
it("restores offline CRDT state and merges a newer desktop edit without duplicating text", async () => {
  const server = new Y.Doc(); server.getText("source").insert(0, "original");
  const first = new Y.Doc(); const presence = new Awareness(first);
  const room = new WsRealtime("doc:a", () => new RoomSocket(server));
  room.bindDoc(first, presence); await room.whenSynced();
  expect(first.getText("source").toString()).toBe("original");
  room.close();
  first.getText("source").insert(first.getText("source").length, " phone offline");
  const saved = Y.encodeStateAsUpdate(first);
  presence.destroy(); first.destroy();
  server.getText("source").insert(0, "desktop changed ");
  const restored = new Y.Doc(); Y.applyUpdate(restored, saved);
  const restoredPresence = new Awareness(restored);
  const resumed = new WsRealtime("doc:a", () => new RoomSocket(server));
  resumed.bindDoc(restored, restoredPresence); await resumed.whenSynced();
  await new Promise(resolve => setTimeout(resolve, 0));
  const text = restored.getText("source").toString();
  expect(text).toBe(server.getText("source").toString());
  expect(text).toContain("phone offline"); expect(text).toContain("desktop changed");
  expect(text.split("original")).toHaveLength(2);
  resumed.close(); restoredPresence.destroy(); restored.destroy(); server.destroy();
});
</mobile:copy>

Assemble `apps/web/src/mobile/recovery.test.ts` in the order explained above.

<mobile:file path="apps/web/src/mobile/recovery.test.ts"><mobile:paste select="#mobile-room-recovery-in-memory-desktop-room"/><mobile:paste select="#mobile-room-recovery-restart-and-merge"/></mobile:file>

</mobile:doc>
