// The watching binding: a terminal with no input path at all.
//
// `docs/specs/freeform/a-terminal-that-writes-the-document.md` names three
// bindings of one component, and this is the third row of its table —
// **a cell that is running**: input *none*, writes *nothing*. It is fed by
// the `Cmd`/`Out`/`Err`/`Exit` events the executor publishes on the run
// channel, and it exists because a build "says why in its own output, and
// ANSI colour and carriage-return rewriting are why an emulator rather than
// the text card that exists" (`launching-what-a-document-builds.md`).
//
// What makes it the *watching* binding is not a flag, it is an absence:
// there is no socket, no `onData` handler, and nothing anywhere in this file
// that could send a byte to a process. `disableStdin` is belt to that
// braces — it stops the emulator taking focus and drawing a cursor that
// invites typing. A terminal you can type into is a second input the
// document does not have, and this one cannot be typed into by construction
// rather than by policy.

import { useEffect, useRef } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

import type { TranscriptEvent } from "../api/types";
import { showsCommands, watchBytes } from "../lib/watchStream";

/** Rows a watching terminal grows to before it starts scrolling instead.
 * A cell sits inside a document someone is reading, so it may not take the
 * page over; past this the earlier output is still there, in scrollback. */
const MAX_ROWS = 20;

/** Columns to assume when the pane has no layout to measure — a hidden tab,
 * the first paint, or jsdom, which does no layout at all. */
const FALLBACK_COLS = 80;

export function WatchingTerminal({
  events,
  live = false,
}: {
  events: TranscriptEvent[];
  /** Whether the run is still going. Only decides whether a cursor blinks;
   * it is never an input affordance, because there is no input. */
  live?: boolean;
}) {
  const host = useRef<HTMLDivElement | null>(null);
  const emulatorRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  // How many events have already been written, so a live run appends instead
  // of redrawing — and the `$ ` rule in force when they were written, since
  // changing it retroactively means the screen no longer matches the bytes.
  const writtenRef = useRef(0);
  const promptsRef = useRef(false);

  useEffect(() => {
    const mount = host.current;
    if (!mount) return;

    const emulator = new Terminal({
      // The executor normalizes captured `\r\n` to `\n` on every platform,
      // so the emulator is the one place that turns a line feed back into a
      // new line. (The PTY pane sets this the other way round, because a PTY
      // sends real CRLF and translating again would double it.)
      convertEol: true,
      disableStdin: true,
      cursorBlink: false,
      cursorStyle: "bar",
      rows: 1,
      cols: FALLBACK_COLS,
      scrollback: 5000,
      fontSize: 12,
      fontFamily:
        getComputedStyle(document.body).getPropertyValue("--mono").trim() ||
        "ui-monospace, SFMono-Regular, Menlo, monospace",
      theme: { background: "rgba(0,0,0,0)" },
      allowTransparency: true,
    });
    const fit = new FitAddon();
    emulator.loadAddon(fit);
    emulator.open(mount);
    emulatorRef.current = emulator;
    fitRef.current = fit;
    writtenRef.current = 0;
    promptsRef.current = false;

    // Width follows the pane; height follows the content (see `resize`
    // below), so the fit addon is asked for its proposal rather than allowed
    // to apply it — `fit()` would set the row count too, and a build's output
    // would be padded out to whatever the pane happened to be tall.
    const observer = new ResizeObserver(() => resize(emulator, fit, mount));
    observer.observe(mount);

    return () => {
      observer.disconnect();
      emulatorRef.current = null;
      fitRef.current = null;
      emulator.dispose();
    };
  }, []);

  useEffect(() => {
    const emulator = emulatorRef.current;
    if (!emulator) return;

    const prompts = showsCommands(events);
    // Two things force a full redraw, and both are rare: a shorter event list
    // (a re-run starting over) and the `$ ` rule flipping when a cell's
    // second command arrives. A terminal cannot edit what it already printed,
    // so the honest answer is to print it again.
    if (events.length < writtenRef.current || prompts !== promptsRef.current) {
      emulator.reset();
      writtenRef.current = 0;
      promptsRef.current = prompts;
    }
    if (events.length === writtenRef.current) return;

    const bytes = watchBytes(events, { showCommands: prompts }, writtenRef.current);
    writtenRef.current = events.length;
    // Resize after the parser has caught up, not before: the row count comes
    // from where the cursor ended up, and `write` is asynchronous.
    emulator.write(bytes, () => {
      const fit = fitRef.current;
      const mount = host.current;
      if (fit && mount) resize(emulator, fit, mount);
      emulator.scrollToBottom();
    });
  }, [events]);

  return (
    <div
      className={`watch-terminal${live ? " watch-terminal-live" : ""}`}
      ref={host}
      data-testid="watch-terminal"
    />
  );
}

/**
 * Width from the pane, height from the content.
 *
 * `baseY + cursorY + 1` is how many rows the output actually used; growing to
 * that pulls lines back out of scrollback, so a terminal that filled up while
 * the pane was hidden still shows its output when the pane appears.
 *
 * The emulator is absolutely positioned (see `.watch-terminal` in
 * `styles.css` for why it has to be), which means the wrapper has no height
 * of its own — so this hands it one. Measuring the emulator and writing the
 * number back onto its own container would normally be a loop; it is not one
 * here, because an out-of-flow box's height is its content's and does not
 * depend on the wrapper it is told to fill.
 */
function resize(emulator: Terminal, fit: FitAddon, mount: HTMLElement): void {
  let cols = FALLBACK_COLS;
  try {
    cols = fit.proposeDimensions()?.cols ?? FALLBACK_COLS;
  } catch {
    // No layout to measure yet; the observer fires again when there is.
  }
  if (!Number.isFinite(cols) || cols < 1) cols = FALLBACK_COLS;
  const buffer = emulator.buffer.active;
  const used = buffer.baseY + buffer.cursorY + 1;
  const rows = Math.min(Math.max(used, 1), MAX_ROWS);
  if (emulator.cols !== cols || emulator.rows !== rows) emulator.resize(cols, rows);
  const height = emulator.element?.offsetHeight ?? 0;
  if (height > 0) mount.style.height = `${height}px`;
}
