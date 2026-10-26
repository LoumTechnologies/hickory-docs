<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/graph/palette.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-286c52e1bf6c"></a>
## `apps/web/src/components/graph/palette.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-286c52e1bf6c-PaletteColor"></a>
### `PaletteColor`

The palette: eight hues that hold up on the app's dark ground, stored in
the document as plain hex so the file means the same thing everywhere.
A palette rather than a colour picker on purpose: eight named choices keep
two diagrams in one project looking like siblings, and keep the JSON diff
of "made the store red" one readable word of hex, not a bikeshed. Fills
are the same hues at low alpha, so any fill stays legible under any text.

<storycode:copy id="f-286c52e1bf6c-PaletteColor">
// The palette: eight hues that hold up on the app's dark ground, stored in
// the document as plain hex so the file means the same thing everywhere.
//
// A palette rather than a colour picker on purpose: eight named choices keep
// two diagrams in one project looking like siblings, and keep the JSON diff
// of "made the store red" one readable word of hex, not a bikeshed. Fills
// are the same hues at low alpha, so any fill stays legible under any text.

export interface PaletteColor {
  name: string;
  value: string;
}

</storycode:copy>
<a id="f-286c52e1bf6c-STROKES"></a>
### `STROKES`

ggplot2's default discrete palette (eight evenly spaced HCL hues, the ones
every scale_hue chart ships with), plus pure black and pure white — so a
diagram and a ggplot figure in the same document speak one colour language.

<storycode:copy id="f-286c52e1bf6c-STROKES">
/* ggplot2's default discrete palette (eight evenly spaced HCL hues, the ones
 * every scale_hue chart ships with), plus pure black and pure white — so a
 * diagram and a ggplot figure in the same document speak one colour language. */
export const STROKES: PaletteColor[] = [
  { name: "red", value: "#F8766D" },
  { name: "gold", value: "#CD9600" },
  { name: "green", value: "#7CAE00" },
  { name: "mint", value: "#00BE67" },
  { name: "cyan", value: "#00BFC4" },
  { name: "blue", value: "#00A9FF" },
  { name: "purple", value: "#C77CFF" },
  { name: "pink", value: "#FF61CC" },
  { name: "black", value: "#000000" },
  { name: "white", value: "#FFFFFF" },
];

</storycode:copy>
<a id="f-286c52e1bf6c-FILLS"></a>
### `FILLS`

<storycode:copy id="f-286c52e1bf6c-FILLS">
/** The same hues at ~15% alpha — a wash, not a wall. */
export const FILLS: PaletteColor[] = STROKES.map(({ name, value }) => ({
  name,
  value: `${value}26`,
}));

</storycode:copy>
<a id="f-286c52e1bf6c-SHAPES"></a>
### `SHAPES`

<storycode:copy id="f-286c52e1bf6c-SHAPES">
export const SHAPES = [
  "rect",
  "round",
  "pill",
  "circle",
  "diamond",
  "hexagon",
  "cylinder",
] as const;
</storycode:copy>

### Assemble `apps/web/src/components/graph/palette.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/palette.ts"><storycode:paste select="#f-286c52e1bf6c-PaletteColor"/><storycode:paste select="#f-286c52e1bf6c-STROKES"/><storycode:paste select="#f-286c52e1bf6c-FILLS"/><storycode:paste select="#f-286c52e1bf6c-SHAPES"/></storycode:file>

</storycode:doc>
