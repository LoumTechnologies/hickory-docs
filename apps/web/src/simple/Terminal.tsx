import { useEffect, useRef, useSyncExternalStore } from "react";
import { Terminal as Emulator } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { api } from "../api/client";
export class TerminalSession {
  id: string | null = null;
  opening = false;
  error = "";
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
}
export function Terminal({ session }: { session: TerminalSession }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const id = session.id, mount = host.current;
    if (!id || !mount) return;
    const terminal = new Emulator({ convertEol: false, cursorBlink: true, fontSize: 13, scrollback: 5000,
      theme: { background: "#10151d", foreground: "#e6edf3" } });
    const fit = new FitAddon(); terminal.loadAddon(fit); terminal.open(mount);
    const socket = new WebSocket(`${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/api/terminals/ws?session=${encodeURIComponent(id)}`);
    socket.binaryType = "arraybuffer";
    let active = true;
    const fail = (message: string) => { if (active) { session.error = message; session.publish(); } };
    socket.onopen = () => { if (active) { session.error = ""; session.publish(); measure(); } };
    socket.onmessage = event => { if (active) terminal.write(typeof event.data === "string" ? event.data : new Uint8Array(event.data)); };
    socket.onclose = () => fail("Terminal disconnected. Choose another tab and reopen Terminal to reconnect. Input was not retried.");
    socket.onerror = () => fail("Cannot connect to Terminal. Check the workspace connection and reopen the tab.");
    const typed = terminal.onData(data => {
      if (socket.readyState === WebSocket.OPEN) socket.send(data);
      else fail("Terminal is disconnected. Input was not sent. Reopen the tab before typing again.");
    });
    let lastSize = "";
    function measure() {
      if (!active || !mount!.clientWidth || !mount!.clientHeight) return;
      fit.fit();
      const size = `${terminal.rows}:${terminal.cols}`;
      if (lastSize === size) return;
      lastSize = size;
      void api.terminalResize(id!, terminal.rows, terminal.cols).catch(reason => {
        lastSize = ""; fail(`${String(reason)} Resize the pane to retry its terminal size.`);
      });
    }
    const observer = new ResizeObserver(measure); observer.observe(mount);
    const frame = requestAnimationFrame(measure);
    // Attaching never creates or closes the backend session. Replay comes from
    // this socket before live bytes, including ANSI cursor and mode sequences.
    return () => { active = false; cancelAnimationFrame(frame); observer.disconnect(); typed.dispose(); socket.close(); terminal.dispose(); };
  }, [session, session.id]);
  return <div className="terminal-body">
    <div ref={host} className="terminal-screen" aria-label="Terminal screen" />
    {session.error && <div role="alert" className="notice">{session.error}</div>}
  </div>;
}
