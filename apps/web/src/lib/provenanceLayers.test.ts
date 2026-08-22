import { describe, expect, it } from "vitest";
import {
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
  it("defaults to every layer on, and survives garbage", () => {
    expect([...loadProvenanceLayers(memory())]).toEqual([...PROVENANCE_LAYERS]);
    expect([...loadProvenanceLayers(memory({ [PROVENANCE_LAYERS_KEY]: "{nope" }))]).toEqual([
      ...PROVENANCE_LAYERS,
    ]);
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
    expect([...toggleLayer(without, "declared")].sort()).toEqual([...PROVENANCE_LAYERS].sort());
  });
});
