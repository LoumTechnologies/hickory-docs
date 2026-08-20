// A terminal, in a pane.
//
// The pane owns the emulator and the socket, and nothing else: the session
// itself lives on the server, so mounting attaches and unmounting detaches.
// Closing this tab must not kill a build — which is why there is no close
// call anywhere in here.

import { useEffect, useRef } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

import { api } from "../api/client";

/** The socket carrying one session's bytes. Same origin as the API. */
function socketUrl(id: string): string {
  const scheme = window.location.protocol === "https:" ? "wss" : "ws";
  return `${scheme}://${window.location.host}/api/terminals/ws?session=${encodeURIComponent(id)}`;
}

/** The size a terminal is at 100%. Everything else is this times the zoom. */
export const TERMINAL_BASE_FONT_PX = 13;

export function TerminalPane({
  sessionId,
  zoom = 1,
}: {
  sessionId: string;
  /** This tab's zoom level. xterm renders to a canvas and ignores the CSS
   * font size the rest of the app is zoomed by, so its own option has to be
   * moved — and the emulator re-fitted, because changing the cell size
   * changes how many rows and columns fit and the child process has to be
   * told the new winsize. */
  zoom?: number;
}) {
  const host = useRef<HTMLDivElement | null>(null);
  // The live emulator and its re-fit, held so the zoom effect below can reach
  // them without rebuilding the terminal — a rebuild would drop the socket
  // and replay the whole scrollback for a font-size change.
  const emulatorRef = useRef<Terminal | null>(null);
  const refitRef = useRef<() => void>(() => {});
  // Read through a ref inside the mount effect, so changing the zoom does not
  // re-run it.
  const zoomRef = useRef(zoom);
  zoomRef.current = zoom;

  useEffect(() => {
    const mount = host.current;
    if (!mount) return;

    const emulator = new Terminal({
      convertEol: false,
      cursorBlink: true,
      fontSize: TERMINAL_BASE_FONT_PX * zoomRef.current,
      // Inherit the app's monospace stack and theme colours rather than
      // xterm's defaults, so a terminal looks like part of this window and
      // not like a screenshot of another program.
      fontFamily:
        getComputedStyle(document.body).getPropertyValue("--mono").trim() ||
        "ui-monospace, SFMono-Regular, Menlo, monospace",
      theme: { background: "rgba(0,0,0,0)" },
      allowTransparency: true,
    });
    const fit = new FitAddon();
    emulator.loadAddon(fit);
    emulator.open(mount);

    let socket: WebSocket | null = new WebSocket(socketUrl(sessionId));
    socket.binaryType = "arraybuffer";

    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        emulator.write(new Uint8Array(event.data));
      } else if (typeof event.data === "string") {
        emulator.write(event.data);
      }
    };
    socket.onclose = () => {
      // The session may simply have been closed from elsewhere; the pane
      // says so rather than sitting there looking live.
      emulator.write("\r\n\x1b[2m[detached]\x1b[0m\r\n");
    };

    const typed = emulator.onData((data) => {
      if (socket?.readyState === WebSocket.OPEN) socket.send(data);
    });

    // Size follows the pane. The child needs the winsize to lay out, so the
    // server is told too — over REST, so every frame on the socket means the
    // same thing.
    let lastSize = { rows: 0, cols: 0 };
    const measure = () => {
      try {
        fit.fit();
      } catch {
        // A pane with no layout yet (hidden tab, first paint) has nothing to
        // fit to; the observer fires again when it does.
        return;
      }
      const { rows, cols } = emulator;
      if (rows === lastSize.rows && cols === lastSize.cols) return;
      if (rows < 1 || cols < 1) return;
      lastSize = { rows, cols };
      api.terminalResize(sessionId, rows, cols).catch(() => {});
    };
    // Once now, and once after the browser has laid the pane out: a tab that
    // mounts hidden (or in the same frame as its split) has no box to fit to
    // yet, and the observer alone would leave it at the PTY's opening 24×80
    // until something else happened to resize it.
    emulatorRef.current = emulator;
    refitRef.current = measure;
    measure();
    const settled = requestAnimationFrame(measure);
    const observer = new ResizeObserver(measure);
    observer.observe(mount);

    return () => {
      cancelAnimationFrame(settled);
      observer.disconnect();
      typed.dispose();
      const closing = socket;
      socket = null;
      closing?.close();
      emulatorRef.current = null;
      refitRef.current = () => {};
      emulator.dispose();
    };
  }, [sessionId]);

  // Zoom, applied without rebuilding anything. The emulator's own font size
  // moves, and then it is re-fitted: a different cell size means a different
  // number of rows and columns, and a child process that is not told about it
  // keeps drawing to the old winsize.
  useEffect(() => {
    const emulator = emulatorRef.current;
    if (!emulator) return;
    const next = TERMINAL_BASE_FONT_PX * zoom;
    if (emulator.options.fontSize === next) return;
    emulator.options.fontSize = next;
    refitRef.current();
  }, [zoom]);

  return <div className="terminal-pane" ref={host} data-testid="terminal-pane" />;
}
