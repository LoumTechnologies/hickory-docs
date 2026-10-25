<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/ProvenanceToggles.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Three Provenances Are Drawn Apart, And Any Subset Can Be Shown](../../../../../contract.md)

Captured SHA-256: `d5f5f902e5c8d22397811a511de32165dbf35b0527101052884bc9d8756a3e07`.

<book:entry path="apps/web/src/shell/ProvenanceToggles.tsx">
import { useId, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
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
  const parent = `provenance-toggles:${useId()}`;
  const revision = useUiRevision(() => [[...layers].sort(), continuity]);
  const [error, setError] = useState("");
  const visible = PROVENANCE_LAYERS.filter(layer => layer !== "continuity" || continuity !== undefined);
  useUiReading("visible provenance layers", { layers: visible.map(layer => ({ layer, enabled: layer === "continuity" ? !!continuity : layers.has(layer), meaning: LAYER_LABEL[layer].tip })), error });
  const invoke = useUiCommands(visible.map(layer => ({ key: `${parent}:${layer}`, command: {
    label: `Toggle ${LAYER_LABEL[layer].short} provenance`, parent, subject: parent, parameters: {}, prerequisites: [layer === "continuity" ? "Changes the project continuity setting through its existing host handler" : "Changes only the existing overlay drawing preference"],
    read: () => ({ revision: revision(), enabled: layer !== "continuity" || !!onContinuity, value: layer === "continuity" ? !!continuity : layers.has(layer) }),
    execute: async () => { if (layer === "continuity") { if (!onContinuity) return { status: "unavailable", reason: "No continuity handler is attached" }; await onContinuity(!continuity); } else await onToggle(layer); return { status: "completed", value: { change_requested: true, layer, persistence: "Inspect the resulting project/UI state; no write completion is certified here" } }; },
  } })));
  const act = (layer: ProvenanceLayer) => { void invoke(`${parent}:${layer}`).catch(reason => setError(String(reason))); };
  return (
    <span
      className="provenance-toggles"
      role="group"
      aria-label="Provenance layers"
    >
      {error && <span role="alert">{error}</span>}
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
            disabled={isContinuity && !onContinuity}
            onClick={() => act(layer)}
          >
            <span className="provenance-toggle__swatch" aria-hidden />
            {LAYER_LABEL[layer].short}
          </button>
        );
      })}
    </span>
  );
}
</book:entry>

</book:doc>
