// Which PROVENANCES the overlay draws. Three kinds live in this product and
// must never look alike (docs/specs/freeform/three-provenances.md):
//
// - lineage  — the weave's byte-exact derivation; nobody can forge it
// - context  — what was in front of the model when it wrote these lines,
//              derived from the session record; nobody can forge it either,
//              but it says "was present", never "was used"
// - declared — what the author (a model or a person) says it cites; anyone
//              can write it, so it is drawn as an assertion
//
// A person asking "why is this here?" wants all three at their fingertips,
// any subset, or none — so the choice is a live toggle, remembered per
// browser like the ribbon style, never a per-document setting.

// A FOURTH family joined them (docs/specs/freeform/provenance-across-versions.md):
//
// - continuity — this span and that one, at an earlier commit, are the same
//                thing. Not forgeable where recorded, and it says whether it
//                was recorded or guessed, and how precisely.
//
// It is different from the other three in a way that shows up here: they
// answer "why is this here" about the present, and it answers "what was this
// before". It is also OFF BY DEFAULT and its toggle is not a browser
// preference — the whole feature rides one project switch, because turning
// the drawing on without the bookkeeping would draw nothing, and turning the
// bookkeeping on without asking would tax every commit for a feature most
// people never use.
export type ProvenanceLayer = "lineage" | "context" | "declared" | "continuity";

export const PROVENANCE_LAYERS: readonly ProvenanceLayer[] = [
  "lineage",
  "context",
  "declared",
  "continuity",
];

export const PROVENANCE_LAYERS_KEY = "hickory.provenanceLayers";

/** What a reader who has expressed no preference gets. */
export const DEFAULT_LAYERS: readonly ProvenanceLayer[] = ["lineage", "context", "declared"];

const isLayer = (value: unknown): value is ProvenanceLayer =>
  value === "lineage" ||
  value === "context" ||
  value === "declared" ||
  value === "continuity";

/** The persisted set, defaulting to every layer on. */
export function loadProvenanceLayers(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): ReadonlySet<ProvenanceLayer> {
  const stored = storage?.getItem(PROVENANCE_LAYERS_KEY);
  // Continuity is deliberately NOT in the default set: a fourth stroke taxes
  // every reader, including the ones who never ask a history question, and
  // the drawing is only half of a feature whose other half is a project
  // switch that is also off.
  if (stored === null || stored === undefined) return new Set(DEFAULT_LAYERS);
  try {
    const parsed: unknown = JSON.parse(stored);
    if (!Array.isArray(parsed)) return new Set(DEFAULT_LAYERS);
    // An explicit empty list is a real choice: "draw nothing".
    return new Set(parsed.filter(isLayer));
  } catch {
    return new Set(DEFAULT_LAYERS);
  }
}

export function saveProvenanceLayers(
  layers: ReadonlySet<ProvenanceLayer>,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(
    PROVENANCE_LAYERS_KEY,
    JSON.stringify(PROVENANCE_LAYERS.filter((l) => layers.has(l))),
  );
}

/** The set with one layer flipped. */
export function toggleLayer(
  layers: ReadonlySet<ProvenanceLayer>,
  layer: ProvenanceLayer,
): Set<ProvenanceLayer> {
  const next = new Set(layers);
  if (next.has(layer)) next.delete(layer);
  else next.add(layer);
  return next;
}

/** What each layer is, in one line — the toggle's tooltip. */
export const LAYER_LABEL: Record<ProvenanceLayer, { short: string; tip: string }> = {
  lineage: {
    short: "Lineage",
    tip: "Lineage — where these bytes came from, by the weave. Derived, byte-exact; nobody can forge it.",
  },
  context: {
    short: "Context",
    tip: "Context — what was in front of the model when an agent wrote these lines, from the session record. Derived; says present, not used.",
  },
  declared: {
    short: "Declared",
    tip: "Declared — what the author says this cites (cites=). An assertion anyone can write; drawn as one.",
  },
  continuity: {
    short: "Continuity",
    tip: "Continuity — what this span was at an earlier commit. Off by default, and the whole feature rides one project switch: no continuity, no journal, no repair. Every link says whether it was recorded or guessed, and how precisely.",
  },
};
