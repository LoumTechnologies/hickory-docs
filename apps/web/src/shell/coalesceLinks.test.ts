import { expect, it } from "vitest";
import { coalesceLinks } from "./coalesceLinks";
import type { RibbonLink } from "./Ribbons";
const link = (line: number): RibbonLink => ({ key: String(line), family: "declared", title: "citation",
  from: { path: "d.md", lines: [line, line] }, to: { path: "target.md", lines: [2, 3] } });
const sources = [{ docPath: "d.md", docSource: "first\n \t\nsecond\n\nthird" }];
it("groups whitespace-separated citations to the same passage", () => {
  const links = [link(1), link(3), link(5)];
  expect(coalesceLinks(links, sources).map(l => l.from.lines)).toEqual([[1, 5]]);
  expect(links[0].from.lines).toEqual([1, 1]);
});
it("preserves prose, destinations, families, and anchored cards", () => {
  expect(coalesceLinks([link(1), link(5)], sources)).toHaveLength(2);
  for (const changed of [{ to: { path: "other.md" } }, { to: { path: "target.md", lines: [4, 5] as [number, number] } },
    { family: "context" as const }, { anchor: {} as HTMLElement }]) {
    expect(coalesceLinks([link(1), { ...link(3), ...changed }], sources)).toHaveLength(2);
  }
});
