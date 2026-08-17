import * as Y from "yjs";
import { Awareness, applyAwarenessUpdate, encodeAwarenessUpdate, removeAwarenessStates } from "y-protocols/awareness";
import * as syncProtocol from "y-protocols/sync";
import * as encoding from "lib0/encoding";
import * as decoding from "lib0/decoding";
import { createLspChannel, type LspChannel } from "../lsp/channel";
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
  /** Resolves once the server's initial sync state has been applied (or
   *  immediately when there is no server, e.g. mock mode). */
  whenSynced(): Promise<void>;
  /**
   * True when the server owns the document's initial content. The server
   * builds each room's Y.Doc from `docs.source`, so a client that also
   * seeds mints a rival copy that the CRDT merges by CONCATENATION —
   * the document doubles on every connect. Only a client-only realtime
   * (mock mode) may seed.
   */
  readonly serverAuthoritative: boolean;
  /**
   * The LSP bridge channel (0x02), or null when no server backs this realtime.
   * The server owns the `hick-lsp` lifecycle, so a client starts at `didOpen`.
   */
  lsp(): LspChannel | null;
  /**
   * The debug channel (0x03), or null when no server backs this realtime.
   * Raw frames: the debug client does its own encoding, because it speaks the
   * session API's verbs rather than JSON-RPC.
   */
  debug?(): { send(frame: Uint8Array): void } | null;
  /**
   * Register the debug client so inbound `0x03` frames reach it.
   *
   * Part of the interface rather than a method only `WsRealtime` happens to
   * have: without the registration the requests still go out and every answer
   * is dropped, which looks exactly like a debugger that will not start.
   */
  onDebugFrame?(handler: (frame: Uint8Array) => boolean): void;
  close(): void;
}

/**
 * Real WebSocket implementation. The doc name (`doc:<id>`) is conveyed as a
 * query parameter since the framed protocol itself carries no doc name.
 */
/** How many refused connections before a room is treated as unavailable. */
const REFUSALS_BEFORE_GIVING_UP = 3;

export class WsRealtime implements Realtime {
  /** The server seeds the room from `docs.source`; never seed from here. */
  readonly serverAuthoritative = true;
  private ws: WebSocket | null = null;
  private runListeners = new Set<(msg: RunWsMessage) => void>();
  private debugHandler: ((frame: Uint8Array) => boolean) | null = null;
  private doc: Y.Doc | null = null;
  private awareness: Awareness | null = null;
  private closed = false;
  /** Whether a connection has ever succeeded, which decides whether a close
      is a refusal or a drop. */
  private everOpened = false;
  private refusals = 0;
  private queue: Uint8Array[] = [];
  private lspChannel: LspChannel | null = null;

  constructor(private docName: string) {
    this.connect();
  }

  private connect() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const params = new URLSearchParams({ doc: this.docName });
    const ws = new WebSocket(`${proto}//${location.host}/api/ws?${params}`);
    ws.binaryType = "arraybuffer";
    this.ws = ws;
    ws.onopen = () => {
      this.everOpened = true;
      this.refusals = 0;
      for (const frame of this.queue.splice(0)) ws.send(frame);
      if (this.doc && this.awareness) this.sendSyncStep1();
    };
    ws.onmessage = (ev) => this.handleFrame(new Uint8Array(ev.data as ArrayBuffer));
    ws.onclose = () => {
      if (this.closed) return;
      // A room the server refuses is refused forever: retrying it every 1.5
      // seconds fills the console with hundreds of identical failures and
      // buries whatever went wrong for real. A room that was open and then
      // dropped is a different thing — that one is worth reconnecting.
      if (!this.everOpened && ++this.refusals >= REFUSALS_BEFORE_GIVING_UP) {
        this.closed = true;
        console.warn(
          `hickory: the server would not open the room "${this.docName}"; not retrying.`,
        );
        return;
      }
      setTimeout(() => this.connect(), 1500);
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
    if (this.lspChannel?.handleFrame(frame)) return;
    if (this.debugHandler?.(frame)) return;
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
        const syncType = syncProtocol.readSyncMessage(decoder, encoder, this.doc, this);
        if (encoding.length(encoder) > 1) this.sendYjs(encoding.toUint8Array(encoder));
        // Step 2 carries the server's state: only AFTER applying it can a
        // client decide the shared doc is genuinely empty. Seeding before
        // this point duplicated the document on every load.
        if (syncType === syncProtocol.messageYjsSyncStep2) this.markSynced();
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

  private synced = false;
  private syncWaiters: (() => void)[] = [];

  private markSynced() {
    if (this.synced) return;
    this.synced = true;
    for (const fn of this.syncWaiters) fn();
    this.syncWaiters = [];
  }

  whenSynced(): Promise<void> {
    if (this.synced) return Promise.resolve();
    return new Promise((resolve) => this.syncWaiters.push(resolve));
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

  lsp(): LspChannel {
    // Lazily created: the server spawns hick-lsp on the first 0x02 frame, so
    // no session exists until something actually asks a language question.
    this.lspChannel ??= createLspChannel({ send: (frame) => this.send(frame) });
    return this.lspChannel;
  }

  debug(): { send(frame: Uint8Array): void } {
    // No lazy object needed: the debug client encodes its own frames, so this
    // is only the way out. Nothing starts on the server until a frame
    // arrives, exactly as with the language channel.
    return { send: (frame: Uint8Array) => this.send(frame) };
  }

  /** Register the debug client so inbound 0x03 frames reach it. */
  onDebugFrame(handler: (frame: Uint8Array) => boolean) {
    this.debugHandler = handler;
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
  /** No server in mock mode — the client must seed its own content. */
  readonly serverAuthoritative = false;
  private runListeners = new Set<(msg: RunWsMessage) => void>();

  whenSynced(): Promise<void> {
    return Promise.resolve();
  }

  emit(msg: RunWsMessage) {
    for (const cb of this.runListeners) cb(msg);
  }

  bindDoc(_doc: Y.Doc, _awareness: Awareness) {
    // Local-only Y.Doc; nothing to sync.
  }

  lsp(): LspChannel | null {
    return null; // no server, no language session
  }

  onRunEvent(cb: (msg: RunWsMessage) => void): () => void {
    this.runListeners.add(cb);
    return () => this.runListeners.delete(cb);
  }

  close() {
    this.runListeners.clear();
  }
}
