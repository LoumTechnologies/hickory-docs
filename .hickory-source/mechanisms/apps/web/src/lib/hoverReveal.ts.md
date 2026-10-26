<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/hoverReveal.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1b3631b1a1f5"></a>
## `apps/web/src/lib/hoverReveal.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1b3631b1a1f5-HoverReveal"></a>
### `HoverReveal`

Hover-revealed connections: the tiny state machine behind "lines to a tab
or port draw only while you hover the text involved in them".
Enter shows a key immediately; leave hides it after a short grace period,
so travelling from the invisible hover strip along the revealed line to
its terminal (to click it) survives the micro-gaps between SVG shapes.
Re-entering during the grace cancels the hide. Purely mechanical — the
caller decides what a key means and what "shown" looks like.

<storycode:copy id="f-1b3631b1a1f5-HoverReveal">
// Hover-revealed connections: the tiny state machine behind "lines to a tab
// or port draw only while you hover the text involved in them".
//
// Enter shows a key immediately; leave hides it after a short grace period,
// so travelling from the invisible hover strip along the revealed line to
// its terminal (to click it) survives the micro-gaps between SVG shapes.
// Re-entering during the grace cancels the hide. Purely mechanical — the
// caller decides what a key means and what "shown" looks like.

export class HoverReveal {
  private shown = new Set<string>();
  private timers = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(
    private onChange: (shown: ReadonlySet<string>) => void,
    /** Grace period (ms) between leaving a connection and hiding it. */
    private grace = 200,
  ) {}

  /** The pointer entered `key`'s hover region (or its revealed shapes). */
  enter(key: string): void {
    const pending = this.timers.get(key);
    if (pending !== undefined) {
      clearTimeout(pending);
      this.timers.delete(key);
    }
    if (!this.shown.has(key)) {
      this.shown.add(key);
      this.emit();
    }
  }

  /** The pointer left: hide after the grace period, unless it comes back. */
  leave(key: string): void {
    if (!this.shown.has(key)) return;
    const pending = this.timers.get(key);
    if (pending !== undefined) clearTimeout(pending);
    this.timers.set(
      key,
      setTimeout(() => {
        this.timers.delete(key);
        this.shown.delete(key);
        this.emit();
      }, this.grace),
    );
  }

  /** Cancel everything — the overlay unmounted; no timer may fire after. */
  dispose(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
    this.shown.clear();
  }

  private emit(): void {
    this.onChange(new Set(this.shown));
  }
}
</storycode:copy>

### Assemble `apps/web/src/lib/hoverReveal.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/hoverReveal.ts"><storycode:paste select="#f-1b3631b1a1f5-HoverReveal"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-3394847f46f0"></a>
## `apps/web/src/lib/hoverReveal.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3394847f46f0-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3394847f46f0-expression_statement">
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { HoverReveal } from "./hoverReveal";

describe("HoverReveal", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  const last = (spy: ReturnType<typeof vi.fn>): ReadonlySet<string> =>
    spy.mock.calls[spy.mock.calls.length - 1][0] as ReadonlySet<string>;

  it("shows a connection the moment its hover region is entered", () => {
    const onChange = vi.fn();
    const reveal = new HoverReveal(onChange, 200);
    reveal.enter("a");
    expect(last(onChange).has("a")).toBe(true);
  });

  it("keeps it shown through the grace period after leaving, then hides", () => {
    const onChange = vi.fn();
    const reveal = new HoverReveal(onChange, 200);
    reveal.enter("a");
    reveal.leave("a");
    // Still shown mid-grace: the pointer is travelling the line.
    vi.advanceTimersByTime(150);
    expect(last(onChange).has("a")).toBe(true);
    vi.advanceTimersByTime(100);
    expect(last(onChange).has("a")).toBe(false);
  });

  it("re-entering during the grace cancels the hide", () => {
    const onChange = vi.fn();
    const reveal = new HoverReveal(onChange, 200);
    reveal.enter("a");
    reveal.leave("a");
    vi.advanceTimersByTime(150);
    reveal.enter("a"); // back on the line, or on the terminal
    vi.advanceTimersByTime(10_000);
    expect(last(onChange).has("a")).toBe(true);
  });

  it("tracks connections independently", () => {
    const onChange = vi.fn();
    const reveal = new HoverReveal(onChange, 200);
    reveal.enter("a");
    reveal.enter("b");
    reveal.leave("a");
    vi.advanceTimersByTime(300);
    const shown = last(onChange);
    expect(shown.has("a")).toBe(false);
    expect(shown.has("b")).toBe(true);
  });

  it("leaving something never shown is a no-op", () => {
    const onChange = vi.fn();
    const reveal = new HoverReveal(onChange, 200);
    reveal.leave("ghost");
    vi.advanceTimersByTime(1000);
    expect(onChange).not.toHaveBeenCalled();
  });

  it("dispose cancels every pending hide so nothing fires after unmount", () => {
    const onChange = vi.fn();
    const reveal = new HoverReveal(onChange, 200);
    reveal.enter("a");
    reveal.leave("a");
    onChange.mockClear();
    reveal.dispose();
    vi.advanceTimersByTime(1000);
    expect(onChange).not.toHaveBeenCalled();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/hoverReveal.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/hoverReveal.test.ts"><storycode:paste select="#f-3394847f46f0-expression_statement"/></storycode:file>


</storycode:doc>
