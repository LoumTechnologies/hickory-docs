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
  continuity,
  onContinuity,
}: {
  layers: ReadonlySet<ProvenanceLayer>;
  onToggle: (layer: ProvenanceLayer) => void;
  /** Whether continuity is on FOR THIS PROJECT. Unlike the other three, this
   * is not a per-browser drawing preference: the whole feature — ribbon,
   * journal and pre-commit repair — rides one switch, so the toggle that
   * draws it is the switch that records it. */
  continuity?: boolean;
  onContinuity?: (enabled: boolean) => void;
}) {
  return (
    <span
      className="provenance-toggles"
      role="group"
      aria-label="Provenance layers"
    >
      {PROVENANCE_LAYERS.map((layer) => {
        // Continuity is not shown at all until the project has it, because a
        // toggle for a feature that records nothing is a toggle that does
        // nothing — and one that silently started writing records into
        // somebody's repository would be worse.
        const isContinuity = layer === "continuity";
        if (isContinuity && continuity === undefined) return null;
        const on = isContinuity ? Boolean(continuity) : layers.has(layer);
        return (
          <button
            key={layer}
            type="button"
            className={`status-bar__item provenance-toggle provenance-toggle--${layer}${
              on ? " provenance-toggle--on" : ""
            }`}
            aria-pressed={on}
            data-tip={LAYER_LABEL[layer].tip}
            onClick={() => (isContinuity ? onContinuity?.(!on) : onToggle(layer))}
          >
            <span className="provenance-toggle__swatch" aria-hidden />
            {LAYER_LABEL[layer].short}
          </button>
        );
      })}
    </span>
  );
}
