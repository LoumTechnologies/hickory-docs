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

export const STROKES: PaletteColor[] = [
  { name: "slate", value: "#94a3b8" },
  { name: "blue", value: "#60a5fa" },
  { name: "green", value: "#4ade80" },
  { name: "amber", value: "#fbbf24" },
  { name: "red", value: "#f87171" },
  { name: "purple", value: "#c084fc" },
  { name: "teal", value: "#2dd4bf" },
  { name: "pink", value: "#f472b6" },
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
