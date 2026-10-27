<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Adapt native persistence and room frames

For maintainers tracing saves and network failures. The native adapter uses typed Tauri commands. The browser preview stores local drafts only and cannot pretend to connect to a desktop.

## Native contract

Only the phone draft metadata is new here. Desktop HTTP response types continue to come from the existing API models. The app never falls back to browser storage after a native save fails.

<mobile:copy id="mobile-bridge-native-contract">
import type { RenderResponse } from "../api/types";
import type { RealtimeSocket } from "../api/realtime";

export interface Remote { machine: string; doc_id: string }
export interface Draft {
  id: string; title: string; source: string; revision: number;
  remote: Remote | null; y_state: number[] | null;
  outputs?: { captured_at: string; render: RenderResponse } | null;
}
export interface Machine { name: string; public_key: string }
export interface Initial { drafts: Draft[]; invitation: string; machines: Machine[] }
interface NativeChannel<T> { onmessage: (message: T) => void }
interface NativeCore {
  invoke<T>(command: string, args: Record<string, unknown>): Promise<T>;
  Channel: new <T>() => NativeChannel<T>;
}
function core(): NativeCore | undefined {
  return (window as unknown as { __TAURI__?: { core: NativeCore } }).__TAURI__?.core;
}
export const native = () => !!core();

export async function action<T>(op: string, fields: Record<string, unknown> = {}): Promise<T> {
  const host = core();
  if (host) return host.invoke<T>("mobile", { action: { op, ...fields } });
  const key = "hickory.mobile.preview.drafts";
  const drafts = JSON.parse(localStorage.getItem(key) ?? "[]") as Draft[];
  if (op === "init") return { drafts, invitation: "", machines: [] } as T;
  if (op === "save") {
    const draft = fields.draft as Draft;
    const old = drafts.find(d => d.id === draft.id);
    if ((old?.revision ?? 0) !== draft.revision) throw Error("A newer browser draft exists; reopen it before saving");
    const saved = { ...draft, revision: draft.revision + 1 };
    localStorage.setItem(key, JSON.stringify([...drafts.filter(d => d.id !== draft.id), saved]));
    return saved as T;
  }
  throw Error("Desktop connections require the native mobile app; this page previews local editing");
}
export const desktop = <T,>(path: string, method = "GET", body: unknown = null) => action<T>("request", { path, method, body });
</mobile:copy>

## Ordered saves

Every edit is queued in order. Each write uses the last accepted native revision, including when a previous write was still pending when the next edit happened. A failed save remains a failure; a later explicit save can retry the latest in-memory draft.

<mobile:copy id="mobile-bridge-ordered-saves">
export class DraftWriter {
  private tail: Promise<unknown> = Promise.resolve();
  private revision: number;
  constructor(draft: Draft, private write = (value: Draft) => action<Draft>("save", { draft: value })) {
    this.revision = draft.revision;
  }
  save(snapshot: Draft): Promise<Draft> {
    const task = this.tail.catch(() => {}).then(async () => {
      const saved = await this.write({ ...snapshot, revision: this.revision });
      this.revision = saved.revision;
      return saved;
    });
    this.tail = task;
    return task;
  }
  async flush() { await this.tail; }
}
</mobile:copy>

## Native socket

The adapter sends only existing realtime frames, in order. Once closed, it never submits a pending frame into a replacement room. Setup failures become close events so the shared realtime client retains its normal refusal behavior. Tauri Channel buffers callbacks during command setup.

<mobile:copy id="mobile-bridge-native-socket">
type RoomEvent = { kind: "frame"; bytes: number[] } | { kind: "closed"; reason: string };
export class NativeSocket implements RealtimeSocket {
  readyState = 0;
  binaryType = "arraybuffer";
  onopen: RealtimeSocket["onopen"] = null;
  onmessage: RealtimeSocket["onmessage"] = null;
  onclose: RealtimeSocket["onclose"] = null;
  private id = crypto.randomUUID();
  private sending = Promise.resolve();
  private ended = false;
  private host = core();
  constructor(url: string, private status: (text: string, online: boolean) => void) {
    const host = this.host;
    if (!host) { queueMicrotask(() => this.fail("Open the native mobile app to connect")); return; }
    const name = new URL(url).searchParams.get("doc") ?? "";
    if (!name.startsWith("doc:")) { queueMicrotask(() => this.fail("Choose a desktop document")); return; }
    const events = new host.Channel<RoomEvent>();
    events.onmessage = (event) => {
      if (this.ended) return;
      if (event.kind === "closed") { this.fail(event.reason); return; }
      this.onmessage?.(new MessageEvent("message", { data: new Uint8Array(event.bytes).buffer }));
    };
    void host.invoke<void>("mobile_room_open", { id: this.id, doc: name.slice(4), events }).then(() => {
      if (this.ended) { void host.invoke("mobile_room_close", { id: this.id }).catch(() => {}); return; }
      this.readyState = 1;
      this.status("Live desktop room connected. Recovery copy saved separately on this phone.", true);
      this.onopen?.(new Event("open"));
    }).catch(error => this.fail(String(error)));
  }
  send(frame: Uint8Array) {
    const bytes = Array.from(frame);
    this.sending = this.sending.then(async () => {
      if (this.ended || !this.host) return;
      await this.host.invoke("mobile_room_send", { id: this.id, bytes });
    }).catch(error => this.fail(String(error)));
  }
  private fail(reason: string) {
    if (this.ended) return;
    this.status(reason, false);
    this.close();
  }
  close() {
    if (this.ended) return;
    this.ended = true;
    this.readyState = 3;
    void this.host?.invoke("mobile_room_close", { id: this.id }).catch(() => {});
    this.onclose?.(new CloseEvent("close"));
  }
}
</mobile:copy>

Assemble `apps/web/src/mobile/bridge.ts` in the order explained above.

<mobile:file path="apps/web/src/mobile/bridge.ts"><mobile:paste select="#mobile-bridge-native-contract"/><mobile:paste select="#mobile-bridge-ordered-saves"/><mobile:paste select="#mobile-bridge-native-socket"/></mobile:file>

</mobile:doc>
