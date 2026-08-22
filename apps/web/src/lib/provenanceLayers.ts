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

export type ProvenanceLayer = "lineage" | "context" | "declared";

export const PROVENANCE_LAYERS: readonly ProvenanceLayer[] = ["lineage", "context", "declared"];

export const PROVENANCE_LAYERS_KEY = "hickory.provenanceLayers";

const isLayer = (value: unknown): value is ProvenanceLayer =>
  value === "lineage" || value === "context" || value === "declared";

/** The persisted set, defaulting to every layer on. */
export function loadProvenanceLayers(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): ReadonlySet<ProvenanceLayer> {
  const stored = storage?.getItem(PROVENANCE_LAYERS_KEY);
  if (stored === null || stored === undefined) return new Set(PROVENANCE_LAYERS);
  try {
    const parsed: unknown = JSON.parse(stored);
    if (!Array.isArray(parsed)) return new Set(PROVENANCE_LAYERS);
    // An explicit empty list is a real choice: "draw nothing".
    return new Set(parsed.filter(isLayer));
  } catch {
    return new Set(PROVENANCE_LAYERS);
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
};
