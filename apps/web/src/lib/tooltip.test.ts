import { describe, expect, it } from "vitest";
import { TIP_ATTR, placeTip, tipTargetOf } from "./tooltip";

const viewport = { width: 1000, height: 800 };
const tip = { width: 200, height: 40 };

describe("placeTip", () => {
  it("prefers above the anchor, centred on it", () => {
    const p = placeTip({ left: 400, top: 300, width: 100, height: 20 }, tip, viewport);
    expect(p.side).toBe("above");
    // 300 - 8 gap - 40 tall.
    expect(p.top).toBe(252);
    // 400 + 50 (centre) - 100 (half the tip).
    expect(p.left).toBe(350);
  });

  it("flips below when there is no room above", () => {
    const p = placeTip({ left: 400, top: 10, width: 100, height: 20 }, tip, viewport);
    expect(p.side).toBe("below");
    expect(p.top).toBe(38);
  });

  it("clamps to the right edge rather than leaving the window", () => {
    // A tab strip's rightmost tab: centring would put half the card offscreen,
    // which is the failure the OS tooltip never had.
    const p = placeTip({ left: 960, top: 300, width: 40, height: 20 }, tip, viewport);
    expect(p.left).toBe(viewport.width - tip.width - 6);
  });

  it("clamps to the left edge too", () => {
    const p = placeTip({ left: 0, top: 300, width: 20, height: 20 }, tip, viewport);
    expect(p.left).toBe(6);
  });

  it("keeps a tooltip with no room either way inside the viewport", () => {
    const tall = { width: 200, height: 700 };
    const p = placeTip({ left: 400, top: 400, width: 100, height: 20 }, tall, viewport);
    expect(p.top).toBeGreaterThanOrEqual(6);
    expect(p.top + tall.height).toBeLessThanOrEqual(viewport.height);
  });
});

// Read the sources off disk, the way styles.test.ts reads the stylesheet:
// this invariant is about code that does not exist yet, so no amount of
// rendering the current components can protect it.
declare const require: (id: string) => {
  readFileSync(p: string, e: string): string;
  readdirSync(p: string, o: { withFileTypes: true }): { name: string; isDirectory(): boolean }[];
};
declare const process: { cwd(): string };

function sources(dir: string, found: string[] = []): string[] {
  for (const entry of require("node:fs").readdirSync(dir, { withFileTypes: true })) {
    const path = `${dir}/${entry.name}`;
    if (entry.isDirectory()) sources(path, found);
    else if (/\.tsx?$/.test(entry.name) && !/\.test\.tsx?$/.test(entry.name)) found.push(path);
  }
  return found;
}

describe("no native tooltips", () => {
  it("leaves every hover hint to the themed layer", () => {
    // The OS draws `title=` in its own palette, which is never one of the
    // three in styles.css. See
    // docs/guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme.md
    const offenders = sources(`${process.cwd()}/src`)
      .filter((path) => !path.endsWith("components/TooltipLayer.tsx"))
      .filter((path) => {
        const src = require("node:fs").readFileSync(path, "utf8");
        // `document.title` is the window title, a different thing entirely.
        return /\btitle=["'{]/.test(src) || /(?<!document)\.title\s*=[^=]/.test(src);
      });
    expect(offenders, "use data-tip / .dataset.tip instead of title").toEqual([]);
  });
});

describe("tipTargetOf", () => {
  it("finds the tooltip owner from a child the pointer is actually over", () => {
    const button = document.createElement("button");
    button.setAttribute(TIP_ATTR, "Split right");
    const glyph = document.createElement("span");
    button.appendChild(glyph);
    expect(tipTargetOf(glyph)?.text).toBe("Split right");
    expect(tipTargetOf(glyph)?.el).toBe(button);
  });

  it("ignores an empty tooltip, so an absent message shows no empty card", () => {
    const el = document.createElement("span");
    el.setAttribute(TIP_ATTR, "  ");
    expect(tipTargetOf(el)).toBeNull();
    expect(tipTargetOf(document.createElement("div"))).toBeNull();
    expect(tipTargetOf(null)).toBeNull();
  });
});
