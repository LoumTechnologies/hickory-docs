// A cell's transcript, as the bytes a terminal was sent.
//
// `docs/specs/freeform/a-terminal-that-writes-the-document.md`: a cell's
// transcript is `Cmd`/`Out`/`Err`/`Exit` events in order, which is exactly a
// REPL scrollback, so "the same bytes have two honest renderings — as a
// script, which is what you edit, and as a session, which is what you read."
// This module is the second rendering, and it stores nothing new to get it.
//
// It is pure and returns a string because the interesting decisions are all
// about which bytes, not about the emulator: what a command line looks like,
// when a prompt is worth showing, and — the reason this exists at all —
// which bytes are passed through untouched.

import type { TranscriptEvent } from "../api/types";

/**
 * Whether the `$ ` prompt lines are worth showing.
 *
 * With one command the panel sits directly under the source that holds it,
 * so echoing it is pure repetition. With several, the prompts are the only
 * thing saying which output belongs to which command — the same rule the
 * text card followed, kept here so one place decides it.
 */
export function showsCommands(events: TranscriptEvent[]): boolean {
  let n = 0;
  for (const e of events) if (e.kind === "cmd") n++;
  return n > 1;
}

/**
 * The bytes for `events[from..]`, to be written to a terminal.
 *
 * Three rules, each of which is the whole point of using an emulator:
 *
 *  - **`out` and `err` are passed through byte-for-byte.** A colour code is
 *    a colour, a `\r` moves the cursor to column zero, and a progress bar
 *    rewrites its own line — which is what the run actually did. Recolouring
 *    stderr would mean rewriting the stream to say something the stream did
 *    not say, and would fight any styling the program set itself.
 *  - **stdout and stderr are not told apart.** They were not told apart on
 *    the screen the command ran on either: two streams, one terminal,
 *    interleaved in time. The executor already orders them by timestamp, so
 *    writing both here reproduces that screen rather than inventing a
 *    two-column view of it.
 *  - **The decorations are ours and say so.** A `$ ` prompt and a non-zero
 *    `[exit n]` are this app talking, not the program, so each is wrapped in
 *    its own SGR and each resets first — a command boundary is exactly where
 *    a shell would have reset, and leftover styling from the last command's
 *    output must not bleed into our line.
 *
 * Newlines are left as `\n`: the executor normalizes captured `\r\n` to
 * `\n` on every platform (`normalize_captured_newlines`), and the emulator
 * is opened with `convertEol`, so the one translation happens in one place.
 */
export function watchBytes(
  events: TranscriptEvent[],
  options: { showCommands: boolean },
  from = 0,
): string {
  let out = "";
  for (let i = Math.max(0, from); i < events.length; i++) {
    const e = events[i];
    if (e.kind === "cmd") {
      if (options.showCommands) out += `\x1b[0m\x1b[2m$ \x1b[22m${e.data}\x1b[0m\n`;
    } else if (e.kind === "exit") {
      // A zero exit is the absence of news. Saying it on every cell would
      // train people to stop reading the line that matters.
      if (e.code !== 0) out += `\x1b[0m\x1b[31m[exit ${e.code}]\x1b[0m\n`;
    } else {
      out += e.data;
    }
  }
  return out;
}
