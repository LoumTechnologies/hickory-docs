// Three small toggles — Lineage · Context · Declared — for which provenances
// the overlay draws. They live in the status bar so they are at hand while
// reading, not in a settings page: "why is this here?" is asked mid-read.
// Each toggle's tooltip says what its family IS and what it can and cannot
// claim, because the three must never be mistaken for one another
// (docs/specs/freeform/three-provenances.md).

import {
  LAYER_LABEL,
  PROVENANCE_LAYERS,
  type ProvenanceLayer,
} from "../lib/provenanceLayers";

export function ProvenanceToggles({
  layers,
  onToggle,
}: {
  layers: ReadonlySet<ProvenanceLayer>;
  onToggle: (layer: ProvenanceLayer) => void;
}) {
  return (
    <span
      className="provenance-toggles"
      role="group"
      aria-label="Provenance layers"
    >
      {PROVENANCE_LAYERS.map((layer) => {
        const on = layers.has(layer);
        return (
          <button
            key={layer}
            type="button"
            className={`status-bar__item provenance-toggle provenance-toggle--${layer}${
              on ? " provenance-toggle--on" : ""
            }`}
            aria-pressed={on}
            data-tip={LAYER_LABEL[layer].tip}
            onClick={() => onToggle(layer)}
          >
            <span className="provenance-toggle__swatch" aria-hidden />
            {LAYER_LABEL[layer].short}
          </button>
        );
      })}
    </span>
  );
}
