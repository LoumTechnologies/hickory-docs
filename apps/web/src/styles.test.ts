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
    // The native menu needs no row in the webview. Reserving the old web
    // header's 3rem leaves an empty strip beneath the status bar.
    expect(page).toMatch(/\bheight: 100vh;/);
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

describe("table grid invariants", () => {
  it("makes a cell fill its row, however tall the row was dragged", () => {
    // The span used to be as tall as its one line of text, so the space added
    // by dragging a row taller belonged to the `td` and to no cell at all.
    // Clicking there hit nothing: the selection did not move, focus left the
    // grid, and the ruler dropped back from naming the table's columns to
    // measuring prose.
    const cell = rule(".table-panel__cell");
    expect(cell).toContain("height: 100%");
    expect(cell).toContain("box-sizing: border-box");
  });

  it("gives every row a declared height, which is what that percentage needs", () => {
    // A percentage height against an `auto` cell resolves to `auto`, so the
    // rule above would silently do nothing.
    const body = rule(".table-panel__grid tbody th,\n.table-panel__grid tbody td");
    expect(body).toContain("height: var(--table-row-height");
  });
});

describe("theming invariants", () => {
  /** The three token blocks: `:root { … }` and the two data-theme overrides. */
  const tokenBlocks = [...css.matchAll(/^:root(?:\[data-theme="[a-z-]+"\])? \{[^}]*\}/gms)].map(
    (m) => m[0],
  );

  it("declares exactly the three themes", () => {
    expect(tokenBlocks).toHaveLength(3);
    expect(tokenBlocks[0].startsWith(":root {")).toBe(true);
    expect(css).toContain(':root[data-theme="light"]');
    expect(css).toContain(':root[data-theme="warm-dark"]');
    // The old mechanism must be gone: dark-by-default comes from :root, not
    // from the OS preference.
    expect(css).not.toContain("prefers-color-scheme: dark");
  });

  it("gives every theme the complete token set — a partial override would leak the default theme through", () => {
    const names = (block: string) =>
      [...block.matchAll(/--[\w-]+(?=:)/g)].map((m) => m[0]).sort();
    const [root, light, warmDark] = tokenBlocks.map(names);
    expect(root.length).toBeGreaterThan(30);
    expect(light).toEqual(root);
    expect(warmDark).toEqual(root);
  });

  it("keeps color-scheme on each theme's side of light/dark", () => {
    expect(tokenBlocks[0]).toContain("color-scheme: dark");
    const light = tokenBlocks.find((b) => b.includes('"light"'))!;
    expect(light).toContain("color-scheme: light");
    const warm = tokenBlocks.find((b) => b.includes('"warm-dark"'))!;
    expect(warm).toContain("color-scheme: dark");
  });

  it("allows raw hex colors only inside the token blocks", () => {
    // Everything below the token blocks styles through var(--…): a raw hex
    // in a rule is invisible to two of the three themes.
    let rest = css;
    for (const block of tokenBlocks) rest = rest.replace(block, "");
    expect(rest).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
  });
});

describe("block widget height invariants", () => {
  // CodeMirror takes a block widget's height from `getBoundingClientRect()`
  // (`measureVisibleLineHeights`), which EXCLUDES margins. A vertical margin
  // on a block widget is therefore height the height map never learns about,
  // and every line below it drifts by exactly that much: measured in the
  // running app on a document with one exec card, `.cm-rendered`'s
  // `margin: 0.35rem 0` put each line number 11.2px above the line it named,
  // and broke Home/End inside the blocks below (`moveToLineBoundary` resolves
  // a boundary by asking `posAtCoords` about a real y, which the height map
  // then mapped to the next line down). The gap has to be padding on
  // something CodeMirror measures.
  const noVerticalMargin = (selector: string) => {
    const declarations = rule(selector);
    const margins = declarations.matchAll(/(?:^|[;{])\s*margin(?:-top|-bottom|-block[a-z-]*)?:([^;]+)/g);
    for (const [, value] of margins) {
      // The first component of a shorthand is the top one; a `-top`/`-bottom`
      // longhand has only the one.
      expect(value.trim().split(/\s+/)[0], `${selector} carries a vertical margin`).toBe("0");
    }
  };

  it("gives the rendered card its gap as padding on a measured wrapper", () => {
    expect(rule(".cm-rendered-frame")).toMatch(/padding: [\d.]+rem 0/);
    noVerticalMargin(".cm-rendered");
  });

  it("gives an inline picture its gap as padding", () => {
    noVerticalMargin(".cm-md-image");
    expect(rule(".cm-md-image")).toMatch(/padding: /);
  });
});
