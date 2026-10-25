<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/hoverReveal.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7f38ccfa185465f03a766fdf1bb9f57912aa345b0579b3f24b7172abb0b90b6d`.

<book:entry path="apps/web/src/lib/hoverReveal.ts">
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
</book:entry>

</book:doc>
