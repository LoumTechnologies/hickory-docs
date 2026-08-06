import { describe, expect, it } from "vitest";

// Layout invariants with real failure modes behind them. jsdom does no layout,
// so nothing else in this suite can catch them — and each one has already
// broken the app once.
// Read the stylesheet off disk. `?raw` imports are stubbed to "" under
// vitest, and @types/node is not a dependency of this app, so the couple of
// Node bindings needed here are declared locally rather than pulled in.
declare const require: (id: string) => { readFileSync(p: string, e: string): string };
declare const process: { cwd(): string };
const css = require("node:fs").readFileSync(`${process.cwd()}/src/styles.css`, "utf8");

/** The declarations of the first rule with this exact selector. */
function rule(selector: string): string {
  // Anchor at the start of a line so `.cm-scroller` does not match
  // `.document-editor .cm-scroller`.
  const i = css.indexOf(`
${selector} {`);
  expect(i, `no rule for ${selector}`).toBeGreaterThan(-1);
  return css.slice(i, css.indexOf("}", i));
}

describe("workspace layout invariants", () => {
  it("reserves the scrollbar gutter in every editor", () => {
    // Wrapped lines inside a max-height box + a scrollbar that appears and
    // disappears = an oscillation with no stable state (the "metastable
    // bounce"). Reserving the gutter keeps the content width constant.
    expect(rule(".cm-scroller")).toContain("scrollbar-gutter: stable");
  });

  it("makes the workspace a fixed-height shell rather than a scrolling page", () => {
    // Otherwise the agent dock (bottom of a scrolling page) covers the
    // document instead of sitting beside it.
    const page = rule(".doc-page");
    expect(page).toContain("flex-direction: column");
    expect(page).toContain("overflow: hidden");
    expect(page).toMatch(/height: calc\(100vh/);
  });

  it("lets split panes shrink so their editors scroll internally", () => {
    // A flex child defaults to min-height:auto and refuses to shrink below its
    // content. As a block box, .split-pane let a 12,000px editor overflow with
    // no scrollbar: everything below the fold became unreachable.
    const pane = rule(".split-pane");
    expect(pane).toContain("min-height: 0");
    expect(pane).toContain("display: flex");
    expect(pane).toContain("flex-direction: column");
  });

  it("keeps the ribbon channels wide enough to read as a Sankey", () => {
    // The gaps between the three columns ARE the diagram; squeezed, the
    // ribbons collapse into vertical bars with nothing to follow.
    const gap = /gap: ([\d.]+)rem/.exec(rule(".split-view"));
    expect(gap).not.toBeNull();
    expect(Number(gap![1])).toBeGreaterThanOrEqual(2);
  });
});
