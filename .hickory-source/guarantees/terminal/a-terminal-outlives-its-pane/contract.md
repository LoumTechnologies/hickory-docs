<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Terminal Outlives Its Pane

For: Authors running interactive tools alongside documents without losing session state.

Given a terminal session started through `POST /api/terminals`, when its tab
is closed — or the window is rearranged, or nobody is attached at all — then
the process keeps running, its output keeps accumulating server-side, and a
client that attaches later is sent everything the session has already said
before it is sent anything new.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-terminal-outlives-its-pane.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-db9058d1877e"></a>
## `apps/web/src/terminal/TerminalPane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Terminal Outlives Its Pane](contract.md).

<a id="f-db9058d1877e-socketUrl"></a>
### `socketUrl`

A terminal, in a pane.
The pane owns the emulator and the socket, and nothing else: the session
itself lives on the server, so mounting attaches and unmounting detaches.
Closing this tab must not kill a build — which is why there is no close
call anywhere in here.

<storycode:copy id="f-db9058d1877e-socketUrl">
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

</storycode:copy>
<a id="f-db9058d1877e-TERMINAL_BASE_FONT_PX"></a>
### `TERMINAL_BASE_FONT_PX`

The size a terminal is at 100%. Everything else is this times the zoom.

<storycode:copy id="f-db9058d1877e-TERMINAL_BASE_FONT_PX">
/** The size a terminal is at 100%. Everything else is this times the zoom. */
export const TERMINAL_BASE_FONT_PX = 13;

</storycode:copy>
<a id="f-db9058d1877e-TerminalPane"></a>
### `TerminalPane`

<storycode:copy id="f-db9058d1877e-TerminalPane">
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
</storycode:copy>

### Assemble `apps/web/src/terminal/TerminalPane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/TerminalPane.tsx"><storycode:paste select="#f-db9058d1877e-socketUrl"/><storycode:paste select="#f-db9058d1877e-TERMINAL_BASE_FONT_PX"/><storycode:paste select="#f-db9058d1877e-TerminalPane"/></storycode:file>

## Test the behavior: `installed`

This is the maintained test fragment from `crates/hick-term/tests/full_screen_apps.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-31c9ccf64945-installed"></a>
### `installed`

Real full-screen programs, in a real PTY.
The tests above this one prove that bytes move. These prove the harder
thing: that a program which takes over the whole screen — an editor, a
process monitor, a coding agent's UI — gets a terminal it believes in.
Those are the programs a half-built PTY breaks: they ask the terminal its
size, switch to the alternate screen, position the cursor absolutely, and
stop drawing entirely if any of it is missing.
What is asserted is the server's own screen model (`vt100`) after feeding
it exactly the bytes the client would receive. If the model shows the
program's interface, the client's emulator has everything it needs to draw
it — the two are parsing the same stream.
Every test skips when its program is not installed, because these run on
whatever machine happens to have the checkout.
Protects docs/guarantees/terminal/a-terminal-outlives-its-pane.md

<storycode:copy id="f-31c9ccf64945-installed">
//! Real full-screen programs, in a real PTY.
//!
//! The tests above this one prove that bytes move. These prove the harder
//! thing: that a program which takes over the whole screen — an editor, a
//! process monitor, a coding agent's UI — gets a terminal it believes in.
//! Those are the programs a half-built PTY breaks: they ask the terminal its
//! size, switch to the alternate screen, position the cursor absolutely, and
//! stop drawing entirely if any of it is missing.
//!
//! What is asserted is the server's own screen model (`vt100`) after feeding
//! it exactly the bytes the client would receive. If the model shows the
//! program's interface, the client's emulator has everything it needs to draw
//! it — the two are parsing the same stream.
//!
//! Every test skips when its program is not installed, because these run on
//! whatever machine happens to have the checkout.
//!
//! Protects docs/guarantees/terminal/a-terminal-outlives-its-pane.md

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_term::session::Session;
use hick_term::{SessionSpec, TermConfig, Terminals};

/// Is `program` on this machine's PATH?
///
/// Walked directly rather than asked of a shell. `sh -c "command -v x"` was
/// the old spelling and it fails to SPAWN on a machine with no `sh` — which
/// `unwrap_or(false)` then read as "not installed", so every test in this file
/// skipped on Windows without printing a word. A silent skip is the one
/// outcome a suite must never have: it is indistinguishable from a pass, and
/// this file would have stayed green with the PTY layer entirely broken.
fn installed(program: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    // `PATHEXT` is how Windows decides what "executable" means; a bare name on
    // PATH there is `vim.exe` or `less.bat`, never `vim`.
    let suffixes: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(str::to_string)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&paths).any(|dir| {
        suffixes
            .iter()
            .any(|suffix| dir.join(format!("{program}{suffix}")).is_file())
    })
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/terminal/TerminalPane.tsx](contract.md#output-db9058d1877e)
- [crates/hick-term/src/registry.rs](../../../mechanisms/crates/hick-term/src/registry.rs.md#output-8678d496098a)
- [crates/hickory-cli/src/serve/terminal.rs](../../../mechanisms/crates/hickory-cli/src/serve/terminal.rs.md#output-e67fa7430800)

### Tests and fixtures

- [crates/hick-term/tests/full_screen_apps.rs](../../../mechanisms/crates/hick-term/tests/full_screen_apps.rs.md#output-31c9ccf64945)
- [crates/hickory-cli/tests/serve_terminals.rs](../../../mechanisms/crates/hickory-cli/tests/serve_terminals.rs.md#output-5d8946e53299)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
