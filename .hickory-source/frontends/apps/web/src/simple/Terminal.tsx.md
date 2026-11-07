<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep a real PTY session independent of its tab

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The existing backend already emulates the terminal screen. All three first frontends show that same plain-text screen and submit line input, interrupt and end-of-input. Session creation is an explicit menu action, never a mount effect. Hiding or moving the tab does not kill its process or lose pending input. Full cursor-key/ANSI client emulation is outside this first matched terminal slice.

<ui:copy id="simple-5d9f52220f-0">
import { useEffect, useRef, useSyncExternalStore } from "react";
import { api } from "../api/client";
export class TerminalSession {
  id: string | null = null;
  opening = false;
  input = "";
  screen = "";
  error = "";
  sending = false;
  private version = 0;
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  async open() {
    if (this.id || this.opening) return;
    this.opening = true; this.error = ""; this.publish();
    try { this.id = (await api.openTerminal({ title: "Terminal" })).id; }
    catch (reason) { this.error = `${String(reason)} Check the terminal backend, then choose Terminal again.`; }
    finally { this.opening = false; this.publish(); }
  }
  async send(data = this.input + "\r", clear = true) {
    if (!this.id || this.sending) return;
    const submitted = this.input, id = this.id;
    this.sending = true; this.error = ""; this.publish();
    try { await api.terminalInput(id, data); if (clear && this.input === submitted) this.input = ""; }
    catch (reason) { this.error = `${String(reason)} Input was not retried. Check the screen before sending again.`; }
    finally { this.sending = false; this.publish(); }
  }
}
export function Terminal({ session }: { session: TerminalSession }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const id = session.id;
    if (!id) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const refresh = async () => {
      try {
        let offset = 0, revision = "";
        const bytes: number[] = [];
        for (;;) {
          const page = await api.terminalScreen(id, offset, revision);
          bytes.push(...page.bytes); revision = page.revision;
          if (page.next === null) break;
          offset = page.next;
        }
        if (!cancelled && session.id === id) { session.screen = new TextDecoder().decode(new Uint8Array(bytes)); session.publish(); }
      } catch (reason) { if (!cancelled) { session.error = `${String(reason)} Reopen Terminal to reconnect to this session.`; session.publish(); } }
      if (!cancelled) timer = setTimeout(refresh, 250);
    };
    const observer = new ResizeObserver(entries => {
      const rect = entries[0]?.contentRect;
      if (rect) void api.terminalResize(id, Math.max(1, Math.min(60, Math.floor(rect.height / 17))), Math.max(1, Math.min(160, Math.floor(rect.width / 8))))
        .catch(reason => { session.error = `${String(reason)} Resize the pane to retry its terminal size.`; session.publish(); });
    });
    if (host.current) observer.observe(host.current);
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); observer.disconnect(); };
  }, [session, session.id]);
  return <div className="terminal-body">
    <div ref={host} className="terminal-screen"><pre aria-label="Terminal screen">{session.screen}</pre></div>
    {session.error && <div role="alert" className="notice">{session.error}</div>}
    <form onSubmit={event => { event.preventDefault(); void session.send(); }}>
      <input aria-label="Terminal input" value={session.input} onChange={event => { session.input = event.target.value; session.publish(); }} autoComplete="off" spellCheck={false} />
      <button disabled={!session.id || session.sending}>Send</button>
      <button type="button" disabled={!session.id || session.sending} onClick={() => { void session.send("\u0003", false); }}>Interrupt</button>
      <button type="button" disabled={!session.id || session.sending} onClick={() => { void session.send("\u0004", false); }}>EOF</button>
    </form>
  </div>;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Terminal.tsx"><ui:paste select="#simple-5d9f52220f-0"/></ui:file>

</ui:doc>
