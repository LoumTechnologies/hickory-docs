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

/** Whether a document declares a layout at all. */
export function declaresLayout(source: string): boolean {
  return regionsOf(source).length > 0;
}

export interface FolderDocument {
  path: string;
  source: string;
}

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
