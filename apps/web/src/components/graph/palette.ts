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

/** The same hues at ~15% alpha — a wash, not a wall. */
export const FILLS: PaletteColor[] = STROKES.map(({ name, value }) => ({
  name,
  value: `${value}26`,
}));

export const SHAPES = [
  "rect",
  "round",
  "pill",
  "circle",
  "diamond",
  "hexagon",
  "cylinder",
] as const;
