import * as Y from "yjs";
import { Awareness, applyAwarenessUpdate, encodeAwarenessUpdate, removeAwarenessStates } from "y-protocols/awareness";
import * as syncProtocol from "y-protocols/sync";
import * as encoding from "lib0/encoding";
import * as decoding from "lib0/decoding";
import { getToken } from "./client";
import type { RunWsMessage } from "./types";

// WS /api/ws — one socket, 1-byte channel prefix per api.md:
//   0x00 + Yjs sync/awareness bytes (y-websocket protocol, doc name `doc:<id>`)
//   0x01 + JSON run event
export const CHANNEL_YJS = 0x00;
export const CHANNEL_RUN = 0x01;

export interface Realtime {
  /** Subscribe to channel-0x01 run events. Returns an unsubscribe fn. */
  onRunEvent(cb: (msg: RunWsMessage) => void): () => void;
  /** Bind a Y.Doc + Awareness to the socket for collaborative editing. */
  bindDoc(doc: Y.Doc, awareness: Awareness): void;
  close(): void;
}

/**
 * Real WebSocket implementation. The doc name (`doc:<id>`) is conveyed as a
 * query parameter since the framed protocol itself carries no doc name.
 */
export class WsRealtime implements Realtime {
  private ws: WebSocket | null = null;
  private runListeners = new Set<(msg: RunWsMessage) => void>();
  private doc: Y.Doc | null = null;
  private awareness: Awareness | null = null;
  private closed = false;
  private queue: Uint8Array[] = [];

  constructor(private docName: string) {
    this.connect();
  }

  private connect() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const params = new URLSearchParams({ doc: this.docName });
    const token = getToken();
    if (token) params.set("token", token);
    const ws = new WebSocket(`${proto}//${location.host}/api/ws?${params}`);
    ws.binaryType = "arraybuffer";
    this.ws = ws;
    ws.onopen = () => {
      for (const frame of this.queue.splice(0)) ws.send(frame);
      if (this.doc && this.awareness) this.sendSyncStep1();
    };
    ws.onmessage = (ev) => this.handleFrame(new Uint8Array(ev.data as ArrayBuffer));
    ws.onclose = () => {
      if (!this.closed) setTimeout(() => this.connect(), 1500);
    };
  }

  private send(frame: Uint8Array) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) this.ws.send(frame);
    else this.queue.push(frame);
  }

  private sendYjs(payload: Uint8Array) {
    const frame = new Uint8Array(payload.length + 1);
    frame[0] = CHANNEL_YJS;
    frame.set(payload, 1);
    this.send(frame);
  }

  private handleFrame(frame: Uint8Array) {
    if (frame.length === 0) return;
    const channel = frame[0];
    const payload = frame.subarray(1);
    if (channel === CHANNEL_RUN) {
      const msg = JSON.parse(new TextDecoder().decode(payload)) as RunWsMessage;
      for (const cb of this.runListeners) cb(msg);
    } else if (channel === CHANNEL_YJS && this.doc && this.awareness) {
      const decoder = decoding.createDecoder(payload);
      const encoder = encoding.createEncoder();
      const messageType = decoding.readVarUint(decoder);
      if (messageType === 0) {
        // sync message
        encoding.writeVarUint(encoder, 0);
        syncProtocol.readSyncMessage(decoder, encoder, this.doc, this);
        if (encoding.length(encoder) > 1) this.sendYjs(encoding.toUint8Array(encoder));
      } else if (messageType === 1) {
        // awareness message
        applyAwarenessUpdate(this.awareness, decoding.readVarUint8Array(decoder), this);
      }
    }
  }

  private sendSyncStep1() {
    if (!this.doc) return;
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, 0);
    syncProtocol.writeSyncStep1(encoder, this.doc);
    this.sendYjs(encoding.toUint8Array(encoder));
  }

  bindDoc(doc: Y.Doc, awareness: Awareness) {
    this.doc = doc;
    this.awareness = awareness;
    doc.on("update", (update: Uint8Array, origin: unknown) => {
      if (origin === this) return;
      const encoder = encoding.createEncoder();
      encoding.writeVarUint(encoder, 0);
      syncProtocol.writeUpdate(encoder, update);
      this.sendYjs(encoding.toUint8Array(encoder));
    });
    awareness.on("update", ({ added, updated, removed }: { added: number[]; updated: number[]; removed: number[] }) => {
      const changed = added.concat(updated, removed);
      const encoder = encoding.createEncoder();
      encoding.writeVarUint(encoder, 1);
      encoding.writeVarUint8Array(encoder, encodeAwarenessUpdate(awareness, changed));
      this.sendYjs(encoding.toUint8Array(encoder));
    });
    this.sendSyncStep1();
  }

  onRunEvent(cb: (msg: RunWsMessage) => void): () => void {
    this.runListeners.add(cb);
    return () => this.runListeners.delete(cb);
  }

  close() {
    this.closed = true;
    if (this.awareness) {
      removeAwarenessStates(this.awareness, [this.awareness.clientID], "close");
    }
    this.ws?.close();
  }
}

// A process-wide realtime instance registered at startup in mock mode so views
// can reach the fake event bus without importing the mock layer.
let sharedRealtime: Realtime | null = null;
export function setSharedRealtime(r: Realtime) {
  sharedRealtime = r;
}
export function getSharedRealtime(): Realtime | null {
  return sharedRealtime;
}

/** In-browser realtime for VITE_MOCK=1: no network; run events are pushed locally. */
export class LocalRealtime implements Realtime {
  private runListeners = new Set<(msg: RunWsMessage) => void>();

  emit(msg: RunWsMessage) {
    for (const cb of this.runListeners) cb(msg);
  }

  bindDoc(_doc: Y.Doc, _awareness: Awareness) {
    // Local-only Y.Doc; nothing to sync.
  }

  onRunEvent(cb: (msg: RunWsMessage) => void): () => void {
    this.runListeners.add(cb);
    return () => this.runListeners.delete(cb);
  }

  close() {
    this.runListeners.clear();
  }
}
