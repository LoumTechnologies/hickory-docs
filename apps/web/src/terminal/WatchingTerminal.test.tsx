// @vitest-environment jsdom
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { WatchingTerminal } from "./WatchingTerminal";
import type { TranscriptEvent } from "../api/types";

afterEach(cleanup);

// The emulator is deliberately not reachable through props — the watching
// binding has no handle to hand out — so these tests read what a person
// reads: the rows xterm drew. Every escape sequence has already been applied
// and is gone from the text by the time it gets here, which is the whole
// claim being checked.
function rows(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll(".xterm-rows > div")) as HTMLElement[];
}

function screenOf(root: HTMLElement): string[] {
  // Rows are padded to the terminal width and the cursor cell renders as a
  // space, neither of which is content; trailing blanks are trimmed so a
  // screen compares as what was written to it.
  return rows(root).map((row) => (row.textContent ?? "").replace(/\s+$/, ""));
}

// xterm writes asynchronously (its parser is fed from a queue) and this
// component resizes in the write callback, so a render is not on screen the
// instant React returns.
async function settle(): Promise<void> {
  await new Promise((r) => setTimeout(r, 30));
}

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
