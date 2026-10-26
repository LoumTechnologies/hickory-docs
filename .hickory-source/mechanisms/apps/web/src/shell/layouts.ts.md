<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/layouts.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-69d30de169db"></a>
## `apps/web/src/shell/layouts.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-69d30de169db-LayoutChoice"></a>
### `LayoutChoice`

What layouts a folder offers, and how a document declares one.
A layout is either built in — freeform, and the two arrangements the mode
switcher used to hard-code — or declared by a document in the folder. The
declaration is structure only: regions and what belongs to them. Geometry is
derived, and what you have open is session state.
See docs/specs/freeform/shell-layouts.md.

<storycode:copy id="f-69d30de169db-LayoutChoice">
// What layouts a folder offers, and how a document declares one.
//
// A layout is either built in — freeform, and the two arrangements the mode
// switcher used to hard-code — or declared by a document in the folder. The
// declaration is structure only: regions and what belongs to them. Geometry is
// derived, and what you have open is session state.
//
// See docs/specs/freeform/shell-layouts.md.

import { freeform, fromRegions, type Layout, type Region } from "./layout";

export interface LayoutChoice {
  id: string;
  name: string;
  /** One line, shown under the name in the picker. */
  detail: string;
  /** The document that declared it, when one did. */
  source?: string;
  regions?: Region[];
  build: () => Layout;
}

</storycode:copy>
<a id="f-69d30de169db-regionsOf"></a>
### `regionsOf`

Read region declarations out of a document.
The syntax is deliberately the language as it stands rather than new tags:
a `hick:copy` whose class is `layout-region` names a region, and its text is
one glob per line. Dedicated tags would read better, and adding them to the
grammar before anyone has lived with the feature is how a language collects
things it cannot remove.
<hick:copy id="ui" class="layout-region">
apps/web/**
</hick:copy>

<storycode:copy id="f-69d30de169db-regionsOf">
/**
 * Read region declarations out of a document.
 *
 * The syntax is deliberately the language as it stands rather than new tags:
 * a `hick:copy` whose class is `layout-region` names a region, and its text is
 * one glob per line. Dedicated tags would read better, and adding them to the
 * grammar before anyone has lived with the feature is how a language collects
 * things it cannot remove.
 *
 *     <hick:copy id="ui" class="layout-region">
 *     apps/web/**
 *     </hick:copy>
 */
export function regionsOf(source: string): Region[] {
  const regions: Region[] = [];
  const block = /<hick:copy\b([^>]*)>([\s\S]*?)<\/hick:copy>/g;
  for (const match of source.matchAll(block)) {
    const attrs = match[1];
    if (!/class\s*=\s*"[^"]*\blayout-region\b[^"]*"/.test(attrs)) continue;
    const name = /id\s*=\s*"([^"]+)"/.exec(attrs)?.[1];
    if (!name) continue;
    const globs = match[2]
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0 && !line.startsWith("#"));
    if (globs.length > 0) regions.push({ name, match: globs });
  }
  return regions;
}

</storycode:copy>
<a id="f-69d30de169db-declaresLayout"></a>
### `declaresLayout`

Whether a document declares a layout at all.

<storycode:copy id="f-69d30de169db-declaresLayout">
/** Whether a document declares a layout at all. */
export function declaresLayout(source: string): boolean {
  return regionsOf(source).length > 0;
}

</storycode:copy>
<a id="f-69d30de169db-FolderDocument"></a>
### `FolderDocument`

<storycode:copy id="f-69d30de169db-FolderDocument">
export interface FolderDocument {
  path: string;
  source: string;
}

</storycode:copy>
<a id="f-69d30de169db-layoutsFor"></a>
### `layoutsFor`

Everything this folder can be arranged as.
Freeform comes first and is always present: a folder that declares nothing
still has to open, and a person who wants to arrange things themselves is
not doing anything unusual.

<storycode:copy id="f-69d30de169db-layoutsFor">
/**
 * Everything this folder can be arranged as.
 *
 * Freeform comes first and is always present: a folder that declares nothing
 * still has to open, and a person who wants to arrange things themselves is
 * not doing anything unusual.
 */
export function layoutsFor(documents: readonly FolderDocument[]): LayoutChoice[] {
  const choices: LayoutChoice[] = [
    {
      id: "freeform",
      name: "Freeform",
      detail: "One pane. Split it, fill it with tabs, arrange it yourself.",
      build: freeform,
    },
  ];

  for (const document of documents) {
    const regions = regionsOf(document.source);
    if (regions.length === 0) continue;
    const name = document.path.split("/").pop()?.replace(/\.md$/, "") ?? document.path;
    choices.push({
      id: `doc:${document.path}`,
      name,
      detail: `${regions.length} region${regions.length === 1 ? "" : "s"} declared by ${document.path}`,
      source: document.path,
      regions,
      build: () => fromRegions(regions),
    });
  }

  return choices;
}

</storycode:copy>
<a id="f-69d30de169db-defaultChoice"></a>
### `defaultChoice`

The layout to start in.
A folder that declares exactly one layout means it: opening into freeform
and making the person go and find it would be pretending we did not know.
Two or more, and choosing for them would be guessing.

<storycode:copy id="f-69d30de169db-defaultChoice">
/**
 * The layout to start in.
 *
 * A folder that declares exactly one layout means it: opening into freeform
 * and making the person go and find it would be pretending we did not know.
 * Two or more, and choosing for them would be guessing.
 */
export function defaultChoice(choices: readonly LayoutChoice[]): LayoutChoice {
  const declared = choices.filter((choice) => choice.source);
  return declared.length === 1 ? declared[0] : choices[0];
}
</storycode:copy>

### Assemble `apps/web/src/shell/layouts.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/layouts.ts"><storycode:paste select="#f-69d30de169db-LayoutChoice"/><storycode:paste select="#f-69d30de169db-regionsOf"/><storycode:paste select="#f-69d30de169db-declaresLayout"/><storycode:paste select="#f-69d30de169db-FolderDocument"/><storycode:paste select="#f-69d30de169db-layoutsFor"/><storycode:paste select="#f-69d30de169db-defaultChoice"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c735d0c864a7"></a>
## `apps/web/src/shell/layouts.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c735d0c864a7-LAYERS"></a>
### `LAYERS`

<storycode:copy id="f-c735d0c864a7-LAYERS">
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
*.md
</hick:copy>
</hick:doc>
`;

</storycode:copy>
<a id="f-c735d0c864a7-ORDINARY"></a>
### `ORDINARY`

<storycode:copy id="f-c735d0c864a7-ORDINARY">
const ORDINARY = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:copy id="c-main" class="app-py">
print("hello")
</hick:copy>
</hick:doc>
`;

</storycode:copy>
<a id="f-c735d0c864a7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c735d0c864a7-expression_statement">
describe("reading a layout out of a document", () => {
  it("takes each region and its globs, in order", () => {
    const regions = regionsOf(LAYERS);
    expect(regions.map((r) => r.name)).toEqual(["ui", "application", "domain"]);
    expect(regions[0].match).toEqual(["apps/web/**"]);
    // Comments are for the reader, not for the matcher.
    expect(regions[2].match).toEqual(["crates/hick-*/**", "*.md"]);
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

</storycode:copy>
<a id="f-c735d0c864a7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c735d0c864a7-expression_statement-2">
describe("what a folder offers", () => {
  const folder = [
    { path: "layers.md", source: LAYERS },
    { path: "notes.md", source: ORDINARY },
  ];

  it("always offers freeform first", () => {
    const choices = layoutsFor(folder);
    expect(choices[0].id).toBe("freeform");
    expect(choices.map((c) => c.name)).toEqual(["Freeform", "layers"]);
  });

  it("names the document a layout came from", () => {
    const declared = layoutsFor(folder)[1];
    expect(declared.source).toBe("layers.md");
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
    expect(regionOf(regions, "LICENSE.txt")).toBeNull();
  });

  it("offers only freeform for a folder that declares nothing", () => {
    expect(layoutsFor([{ path: "notes.md", source: ORDINARY }])).toHaveLength(1);
  });

  it("opens into the one declared layout, and asks when there are two", () => {
    // One declaration means it; two means choosing would be guessing.
    expect(defaultChoice(layoutsFor(folder)).source).toBe("layers.md");
    const two = layoutsFor([...folder, { path: "pipeline.md", source: LAYERS }]);
    expect(defaultChoice(two).id).toBe("freeform");
    expect(defaultChoice(layoutsFor([])).id).toBe("freeform");
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/layouts.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/layouts.test.ts"><storycode:paste select="#f-c735d0c864a7-LAYERS"/><storycode:paste select="#f-c735d0c864a7-ORDINARY"/><storycode:paste select="#f-c735d0c864a7-expression_statement"/><storycode:paste select="#f-c735d0c864a7-expression_statement-2"/></storycode:file>


</storycode:doc>
