<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Watched Cell Shows What The Terminal Showed

For: Authors running interactive tools alongside documents without losing session state.

Given a `hick:exec` cell that has run — or is running now, with its
transcript arriving on the run channel — when its output appears under the
cell in the Document view, then it is drawn by a terminal emulator rather
than printed as text, so what the reader sees is what the command actually
drew:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-watched-cell-shows-what-the-terminal-showed.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-e0bd4a81d48d"></a>
## `apps/web/src/lib/watchStream.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Watched Cell Shows What The Terminal Showed](contract.md).

<a id="f-e0bd4a81d48d-ok"></a>
### `ok`

<storycode:copy id="f-e0bd4a81d48d-ok">
import { describe, expect, it } from "vitest";
import { showsCommands, watchBytes } from "./watchStream";
import type { TranscriptEvent } from "../api/types";

const ok = (data: string): TranscriptEvent[] => [
  { t: 0, kind: "cmd", data: "build" },
  { t: 1, kind: "out", data },
  { t: 2, kind: "exit", code: 0 },
];

</storycode:copy>
<a id="f-e0bd4a81d48d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e0bd4a81d48d-expression_statement">
describe("watchBytes", () => {
  it("passes output through byte-for-byte, escape sequences included", () => {
    // The whole reason the watching binding is an emulator: these bytes must
    // arrive unmangled, because only the parser downstream can tell that
    // `\x1b[32m` is a colour and `\r` is a cursor move.
    const raw = "\x1b[32mPASS\x1b[0m 12 tests\r\x1b[K\x1b[32mPASS\x1b[0m 40 tests\n";
    expect(watchBytes(ok(raw), { showCommands: false })).toBe(raw);
  });

  it("does not tell stdout and stderr apart", () => {
    // Two streams, one screen. The executor already interleaved them by
    // time; recolouring one of them here would say something the run did not.
    const events: TranscriptEvent[] = [
      { t: 0, kind: "out", data: "a" },
      { t: 1, kind: "err", data: "b" },
      { t: 2, kind: "out", data: "c" },
    ];
    expect(watchBytes(events, { showCommands: false })).toBe("abc");
  });

  it("suppresses the prompt for a single command and keeps it for several", () => {
    const one: TranscriptEvent[] = [{ t: 0, kind: "cmd", data: "cat f" }];
    const two: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "printf a > f" },
      { t: 1, kind: "cmd", data: "cat f" },
    ];
    expect(showsCommands(one)).toBe(false);
    expect(showsCommands(two)).toBe(true);
    expect(watchBytes(one, { showCommands: showsCommands(one) })).toBe("");
    const bytes = watchBytes(two, { showCommands: showsCommands(two) });
    expect(bytes).toContain("$ \x1b[22mcat f");
    expect(bytes).toContain("$ \x1b[22mprintf a > f");
  });

  it("reports a non-zero exit and stays silent about a zero one", () => {
    expect(watchBytes(ok("x"), { showCommands: false })).toBe("x");
    const failed: TranscriptEvent[] = [{ t: 0, kind: "exit", code: 2 }];
    expect(watchBytes(failed, { showCommands: false })).toContain("[exit 2]");
  });

  it("resets before every decoration it writes", () => {
    // A command whose output left the terminal bold-red must not bleed into
    // the prompt of the next command, or into our exit line.
    const events: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "one" },
      { t: 1, kind: "out", data: "\x1b[1;31mangry" },
      { t: 2, kind: "cmd", data: "two" },
      { t: 3, kind: "exit", code: 1 },
    ];
    const bytes = watchBytes(events, { showCommands: true });
    expect(bytes).toContain("\x1b[0m\x1b[2m$ \x1b[22mtwo");
    expect(bytes).toContain("\x1b[0m\x1b[31m[exit 1]");
  });

  it("emits only the tail from `from`, so a live run appends", () => {
    const events: TranscriptEvent[] = [
      { t: 0, kind: "out", data: "first" },
      { t: 1, kind: "out", data: "second" },
    ];
    expect(watchBytes(events, { showCommands: false }, 1)).toBe("second");
    expect(watchBytes(events, { showCommands: false }, 2)).toBe("");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/watchStream.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/watchStream.test.ts"><storycode:paste select="#f-e0bd4a81d48d-ok"/><storycode:paste select="#f-e0bd4a81d48d-expression_statement"/></storycode:file>

<a id="output-4174919b979f"></a>
## `apps/web/src/lib/watchStream.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Watched Cell Shows What The Terminal Showed](contract.md).

<a id="f-4174919b979f-showsCommands"></a>
### `showsCommands`

A cell's transcript, as the bytes a terminal was sent.
`docs/specs/freeform/a-terminal-that-writes-the-document.md`: a cell's
transcript is `Cmd`/`Out`/`Err`/`Exit` events in order, which is exactly a
REPL scrollback, so "the same bytes have two honest renderings — as a
script, which is what you edit, and as a session, which is what you read."
This module is the second rendering, and it stores nothing new to get it.
It is pure and returns a string because the interesting decisions are all
about which bytes, not about the emulator: what a command line looks like,
when a prompt is worth showing, and — the reason this exists at all —
which bytes are passed through untouched.

<storycode:copy id="f-4174919b979f-showsCommands">
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

</storycode:copy>
<a id="f-4174919b979f-watchBytes"></a>
### `watchBytes`

The bytes for `events[from..]`, to be written to a terminal.
Three rules, each of which is the whole point of using an emulator:
- **`out` and `err` are passed through byte-for-byte.** A colour code is
a colour, a `\r` moves the cursor to column zero, and a progress bar
rewrites its own line — which is what the run actually did. Recolouring
stderr would mean rewriting the stream to say something the stream did
not say, and would fight any styling the program set itself.
- **stdout and stderr are not told apart.** They were not told apart on
the screen the command ran on either: two streams, one terminal,
interleaved in time. The executor already orders them by timestamp, so
writing both here reproduces that screen rather than inventing a
two-column view of it.
- **The decorations are ours and say so.** A `$ ` prompt and a non-zero
`[exit n]` are this app talking, not the program, so each is wrapped in
its own SGR and each resets first — a command boundary is exactly where
a shell would have reset, and leftover styling from the last command's
output must not bleed into our line.
Newlines are left as `\n`: the executor normalizes captured `\r\n` to
`\n` on every platform (`normalize_captured_newlines`), and the emulator
is opened with `convertEol`, so the one translation happens in one place.

<storycode:copy id="f-4174919b979f-watchBytes">
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
</storycode:copy>

### Assemble `apps/web/src/lib/watchStream.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/watchStream.ts"><storycode:paste select="#f-4174919b979f-showsCommands"/><storycode:paste select="#f-4174919b979f-watchBytes"/></storycode:file>

<a id="output-408e9d7c624e"></a>
## `apps/web/src/terminal/WatchingTerminal.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Watched Cell Shows What The Terminal Showed](contract.md).

<a id="f-408e9d7c624e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-408e9d7c624e-expression_statement">
// @vitest-environment jsdom
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { WatchingTerminal } from "./WatchingTerminal";
import type { TranscriptEvent } from "../api/types";

afterEach(cleanup);

</storycode:copy>
<a id="f-408e9d7c624e-rows"></a>
### `rows`

The emulator is deliberately not reachable through props — the watching
binding has no handle to hand out — so these tests read what a person
reads: the rows xterm drew. Every escape sequence has already been applied
and is gone from the text by the time it gets here, which is the whole
claim being checked.

<storycode:copy id="f-408e9d7c624e-rows">
// The emulator is deliberately not reachable through props — the watching
// binding has no handle to hand out — so these tests read what a person
// reads: the rows xterm drew. Every escape sequence has already been applied
// and is gone from the text by the time it gets here, which is the whole
// claim being checked.
function rows(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll(".xterm-rows > div")) as HTMLElement[];
}

</storycode:copy>
<a id="f-408e9d7c624e-screenOf"></a>
### `screenOf`

<storycode:copy id="f-408e9d7c624e-screenOf">
function screenOf(root: HTMLElement): string[] {
  // Rows are padded to the terminal width and the cursor cell renders as a
  // space, neither of which is content; trailing blanks are trimmed so a
  // screen compares as what was written to it.
  return rows(root).map((row) => (row.textContent ?? "").replace(/\s+$/, ""));
}

</storycode:copy>
<a id="f-408e9d7c624e-settle"></a>
### `settle`

xterm writes asynchronously (its parser is fed from a queue) and this
component resizes in the write callback, so a render is not on screen the
instant React returns.

<storycode:copy id="f-408e9d7c624e-settle">
// xterm writes asynchronously (its parser is fed from a queue) and this
// component resizes in the write callback, so a render is not on screen the
// instant React returns.
async function settle(): Promise<void> {
  await new Promise((r) => setTimeout(r, 30));
}

</storycode:copy>
<a id="f-408e9d7c624e-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-408e9d7c624e-expression_statement-2">
describe("WatchingTerminal", () => {
  it("interprets ANSI instead of showing it — the reason it is not a text card", async () => {
    const events: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "cargo test" },
      { t: 1, kind: "out", data: "\x1b[32mok\x1b[0m 3 passed\n" },
      { t: 2, kind: "exit", code: 0 },
    ];
    const { container } = render(<WatchingTerminal events={events} />);
    await settle();
    // The escape is not in the text, because it became an attribute: ANSI
    // green is palette index 2, which the renderer spells `xterm-fg-2`.
    expect(screenOf(container)[0]).toBe("ok 3 passed");
    const green = rows(container)[0].querySelector(".xterm-fg-2");
    expect(green?.textContent).toBe("ok");
  });

  it("applies a carriage return as a cursor move, so a progress bar is one line", async () => {
    // A text card treats `\r` as a character and prints both states, one
    // under the other. A build's output is full of these.
    const events: TranscriptEvent[] = [
      { t: 0, kind: "out", data: "Compiling  12/40\rCompiling  40/40\n" },
    ];
    const { container } = render(<WatchingTerminal events={events} />);
    await settle();
    expect(screenOf(container)[0]).toBe("Compiling  40/40");
  });

  it("suppresses the prompt for one command and shows it for several", async () => {
    const one: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "cat f" },
      { t: 1, kind: "out", data: "a\n" },
    ];
    const { container, rerender } = render(<WatchingTerminal events={one} />);
    await settle();
    expect(screenOf(container).join("\n")).not.toContain("cat f");

    // The second command flipping the rule forces a redraw, because a
    // terminal cannot go back and add a prompt above output it already drew.
    rerender(
      <WatchingTerminal
        events={[
          ...one,
          { t: 2, kind: "cmd", data: "cat g" },
          { t: 3, kind: "out", data: "b\n" },
        ]}
      />,
    );
    await settle();
    expect(screenOf(container)).toEqual(["$ cat f", "a", "$ cat g", "b", ""]);
  });

  it("has no input path: stdin is disabled and there is no socket to reach", async () => {
    const { container } = render(<WatchingTerminal events={[]} />);
    await settle();
    // A person can click it and type; nothing is listening. The component
    // holds no WebSocket and registers no `onData` handler, so the keystroke
    // has nowhere to go — the absence is the guarantee, not the flag.
    expect(container.querySelector("[data-testid='watch-terminal']")).toBeTruthy();
    expect(container.querySelector(".xterm-cursor-blink")).toBeNull();
  });

  it("appends a live run rather than redrawing it", async () => {
    const first: TranscriptEvent[] = [{ t: 0, kind: "out", data: "one\n" }];
    const { container, rerender } = render(<WatchingTerminal events={first} live />);
    await settle();
    rerender(<WatchingTerminal events={[...first, { t: 1, kind: "out", data: "two\n" }]} live />);
    await settle();
    expect(screenOf(container)).toEqual(["one", "two", ""]);
  });

  it("starts over when a re-run shortens the event list", async () => {
    const many: TranscriptEvent[] = [
      { t: 0, kind: "out", data: "stale\n" },
      { t: 1, kind: "out", data: "output\n" },
    ];
    const { container, rerender } = render(<WatchingTerminal events={many} />);
    await settle();
    rerender(<WatchingTerminal events={[{ t: 0, kind: "out", data: "fresh\n" }]} />);
    await settle();
    expect(screenOf(container)).toEqual(["fresh", ""]);
  });

  it("grows to its content and stops, rather than taking the page over", async () => {
    const long = Array.from({ length: 60 }, (_, i) => ({
      t: i,
      kind: "out" as const,
      data: `line ${i}\n`,
    }));
    const { container } = render(<WatchingTerminal events={long} />);
    await settle();
    // Capped at MAX_ROWS; the earlier lines are still in scrollback.
    expect(rows(container).length).toBe(20);
    expect(screenOf(container).at(-2)).toBe("line 59");
  });
});
</storycode:copy>

### Assemble `apps/web/src/terminal/WatchingTerminal.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/WatchingTerminal.test.tsx"><storycode:paste select="#f-408e9d7c624e-expression_statement"/><storycode:paste select="#f-408e9d7c624e-rows"/><storycode:paste select="#f-408e9d7c624e-screenOf"/><storycode:paste select="#f-408e9d7c624e-settle"/><storycode:paste select="#f-408e9d7c624e-expression_statement-2"/></storycode:file>

<a id="output-8f16bf6c8702"></a>
## `apps/web/src/terminal/WatchingTerminal.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Watched Cell Shows What The Terminal Showed](contract.md).

<a id="f-8f16bf6c8702-MAX_ROWS"></a>
### `MAX_ROWS`

The watching binding: a terminal with no input path at all.
`docs/specs/freeform/a-terminal-that-writes-the-document.md` names three
bindings of one component, and this is the third row of its table —
**a cell that is running**: input *none*, writes *nothing*. It is fed by
the `Cmd`/`Out`/`Err`/`Exit` events the executor publishes on the run
channel, and it exists because a build "says why in its own output, and
ANSI colour and carriage-return rewriting are why an emulator rather than
the text card that exists" (`launching-what-a-document-builds.md`).
What makes it the *watching* binding is not a flag, it is an absence:
there is no socket, no `onData` handler, and nothing anywhere in this file
that could send a byte to a process. `disableStdin` is belt to that
braces — it stops the emulator taking focus and drawing a cursor that
invites typing. A terminal you can type into is a second input the
document does not have, and this one cannot be typed into by construction
rather than by policy.

<storycode:copy id="f-8f16bf6c8702-MAX_ROWS">
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

</storycode:copy>
<a id="f-8f16bf6c8702-FALLBACK_COLS"></a>
### `FALLBACK_COLS`

Columns to assume when the pane has no layout to measure — a hidden tab,
the first paint, or jsdom, which does no layout at all.

<storycode:copy id="f-8f16bf6c8702-FALLBACK_COLS">
/** Columns to assume when the pane has no layout to measure — a hidden tab,
 * the first paint, or jsdom, which does no layout at all. */
const FALLBACK_COLS = 80;

</storycode:copy>
<a id="f-8f16bf6c8702-WatchingTerminal"></a>
### `WatchingTerminal`

<storycode:copy id="f-8f16bf6c8702-WatchingTerminal">
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

</storycode:copy>
<a id="f-8f16bf6c8702-resize"></a>
### `resize`

Width from the pane, height from the content.
`baseY + cursorY + 1` is how many rows the output actually used; growing to
that pulls lines back out of scrollback, so a terminal that filled up while
the pane was hidden still shows its output when the pane appears.
The emulator is absolutely positioned (see `.watch-terminal` in
`styles.css` for why it has to be), which means the wrapper has no height
of its own — so this hands it one. Measuring the emulator and writing the
number back onto its own container would normally be a loop; it is not one
here, because an out-of-flow box's height is its content's and does not
depend on the wrapper it is told to fill.

<storycode:copy id="f-8f16bf6c8702-resize">
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
</storycode:copy>

### Assemble `apps/web/src/terminal/WatchingTerminal.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/WatchingTerminal.tsx"><storycode:paste select="#f-8f16bf6c8702-MAX_ROWS"/><storycode:paste select="#f-8f16bf6c8702-FALLBACK_COLS"/><storycode:paste select="#f-8f16bf6c8702-WatchingTerminal"/><storycode:paste select="#f-8f16bf6c8702-resize"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/CellPanel.tsx](../../../mechanisms/apps/web/src/components/CellPanel.tsx.md#output-c3c409b02e61)
- [apps/web/src/lib/watchStream.ts](contract.md#output-4174919b979f)
- [apps/web/src/terminal/WatchingTerminal.tsx](contract.md#output-8f16bf6c8702)
- [apps/web/src/test-setup.ts](../../../mechanisms/apps/web/src/test-setup.ts.md#output-0e833cff7a55)

### Tests and fixtures

- [apps/web/src/lib/watchStream.test.ts](contract.md#output-e0bd4a81d48d)
- [apps/web/src/terminal/WatchingTerminal.test.tsx](contract.md#output-408e9d7c624e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
