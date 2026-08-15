import { describe, expect, it } from "vitest";
import { panes, regionOf } from "./layout";
import { declaresLayout, defaultChoice, layoutsFor, regionsOf } from "./layouts";

const LAYERS = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="layers.md">
# Layers

<hick:copy id="ui" class="layout-region">
apps/web/**
</hick:copy>

<hick:copy id="application" class="layout-region">
crates/hickory-*/**
</hick:copy>

<hick:copy id="domain" class="layout-region">
# the language and its engines
crates/hick-*/**
*.hick
</hick:copy>
</hick:doc>
`;

const ORDINARY = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:copy id="c-main" class="app-py">
print("hello")
</hick:copy>
</hick:doc>
`;

describe("reading a layout out of a document", () => {
  it("takes each region and its globs, in order", () => {
    const regions = regionsOf(LAYERS);
    expect(regions.map((r) => r.name)).toEqual(["ui", "application", "domain"]);
    expect(regions[0].match).toEqual(["apps/web/**"]);
    // Comments are for the reader, not for the matcher.
    expect(regions[2].match).toEqual(["crates/hick-*/**", "*.hick"]);
  });

  it("ignores copies that are not layout regions", () => {
    // Every document is full of `hick:copy`; only the ones that say they are
    // regions are regions.
    expect(regionsOf(ORDINARY)).toEqual([]);
    expect(declaresLayout(ORDINARY)).toBe(false);
    expect(declaresLayout(LAYERS)).toBe(true);
  });

  it("skips a region with no globs, rather than making an empty pane", () => {
    const empty = LAYERS.replace("apps/web/**", "   ");
    expect(regionsOf(empty).map((r) => r.name)).toEqual(["application", "domain"]);
  });
});

describe("what a folder offers", () => {
  const folder = [
    { path: "layers.hick", source: LAYERS },
    { path: "notes.hick", source: ORDINARY },
  ];

  it("always offers freeform first", () => {
    const choices = layoutsFor(folder);
    expect(choices[0].id).toBe("freeform");
    expect(choices.map((c) => c.name)).toEqual(["Freeform", "layers"]);
  });

  it("names the document a layout came from", () => {
    const declared = layoutsFor(folder)[1];
    expect(declared.source).toBe("layers.hick");
    expect(declared.detail).toContain("3 regions");
  });

  it("builds a pane per region, in the declared order", () => {
    const layout = layoutsFor(folder)[1].build();
    expect(panes(layout.root).map((p) => p.region)).toEqual(["ui", "application", "domain"]);
  });

  it("routes by the globs it read", () => {
    const regions = layoutsFor(folder)[1].regions!;
    expect(regionOf(regions, "apps/web/src/main.tsx")).toBe("ui");
    expect(regionOf(regions, "crates/hick-dap/src/session.rs")).toBe("domain");
    expect(regionOf(regions, "README.md")).toBeNull();
  });

  it("offers only freeform for a folder that declares nothing", () => {
    expect(layoutsFor([{ path: "notes.hick", source: ORDINARY }])).toHaveLength(1);
  });

  it("opens into the one declared layout, and asks when there are two", () => {
    // One declaration means it; two means choosing would be guessing.
    expect(defaultChoice(layoutsFor(folder)).source).toBe("layers.hick");
    const two = layoutsFor([...folder, { path: "pipeline.hick", source: LAYERS }]);
    expect(defaultChoice(two).id).toBe("freeform");
    expect(defaultChoice(layoutsFor([])).id).toBe("freeform");
  });
});
