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
