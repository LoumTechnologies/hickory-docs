import { describe, expect, it } from "vitest";
import {
  DEFAULT_LAYERS,
  PROVENANCE_LAYERS,
  PROVENANCE_LAYERS_KEY,
  loadProvenanceLayers,
  saveProvenanceLayers,
  toggleLayer,
} from "./provenanceLayers";

function memory(initial: Record<string, string> = {}) {
  const store = new Map(Object.entries(initial));
  return {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, v),
    store,
  };
}

// Guarantee: docs/guarantees/lineage/three-provenances-are-drawn-apart.md
describe("provenance layer toggles", () => {
  it("defaults to the three DRAWN layers on, and survives garbage", () => {
    // Continuity is the fourth family and is deliberately NOT in the default
    // set: a fourth stroke taxes every reader including the ones who never
    // ask a history question, and the drawing is only half of a feature whose
    // other half is a project switch that is also off.
    // docs/specs/freeform/provenance-across-versions.md
    expect([...loadProvenanceLayers(memory())]).toEqual([...DEFAULT_LAYERS]);
    expect([...loadProvenanceLayers(memory({ [PROVENANCE_LAYERS_KEY]: "{nope" }))]).toEqual([
      ...DEFAULT_LAYERS,
    ]);
    expect(loadProvenanceLayers(memory()).has("continuity")).toBe(false);
    // It is still a layer somebody can turn on.
    expect(PROVENANCE_LAYERS).toContain("continuity");
  });

  it("an explicit empty set is a choice, not a default", () => {
    const s = memory();
    saveProvenanceLayers(new Set(), s);
    expect(loadProvenanceLayers(s).size).toBe(0);
  });

  it("round-trips a subset and ignores unknown names", () => {
    const s = memory();
    saveProvenanceLayers(new Set(["context"]), s);
    expect([...loadProvenanceLayers(s)]).toEqual(["context"]);
    s.setItem(PROVENANCE_LAYERS_KEY, JSON.stringify(["lineage", "vibes"]));
    expect([...loadProvenanceLayers(s)]).toEqual(["lineage"]);
  });

  it("toggles one layer without touching the others", () => {
    const all = loadProvenanceLayers(memory());
    const without = toggleLayer(all, "declared");
    expect([...without]).toEqual(["lineage", "context"]);
    expect([...toggleLayer(without, "declared")].sort()).toEqual([...DEFAULT_LAYERS].sort());
  });
});
