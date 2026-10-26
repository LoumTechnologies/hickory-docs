<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Three Provenances Are Drawn Apart, And Any Subset Can Be Shown

For: Authors editing generated text and reviewers tracing a result to its source.

Given a document on screen, when the overlay draws where its text came from,
then it draws up to three families — **lineage** (the weave's byte-exact
derivation), **context** (what was in front of the model when an agent wrote
these lines, from the session record), and **declared** (what the author says
it cites, `cites="…"`) — each in its own hue AND its own stroke (solid
palette / dotted amber / dashed grey), each with a title that says what it is
and what it cannot claim; and three toggles in the status bar show any
subset, remembered per browser.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/three-provenances-are-drawn-apart.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-bb5abd922b4d"></a>
## `apps/web/src/lib/provenanceLayers.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Three Provenances Are Drawn Apart, And Any Subset Can Be Shown](contract.md).

<a id="f-bb5abd922b4d-memory"></a>
### `memory`

<storycode:copy id="f-bb5abd922b4d-memory">
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

</storycode:copy>
<a id="f-bb5abd922b4d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-bb5abd922b4d-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/lib/provenanceLayers.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/provenanceLayers.test.ts"><storycode:paste select="#f-bb5abd922b4d-memory"/><storycode:paste select="#f-bb5abd922b4d-expression_statement"/></storycode:file>

<a id="output-031f3259ebc6"></a>
## `apps/web/src/lib/provenanceLayers.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Three Provenances Are Drawn Apart, And Any Subset Can Be Shown](contract.md).

<a id="f-031f3259ebc6-ProvenanceLayer"></a>
### `ProvenanceLayer`

Which PROVENANCES the overlay draws. Three kinds live in this product and
must never look alike (docs/specs/freeform/three-provenances.md):
- lineage  — the weave's byte-exact derivation; recomputable from source bytes
- context  — what was in front of the model when it wrote these lines,
derived from the session record; records the supplied context,
but it says "was present", never "was used"
- declared — what the author (a model or a person) says it cites; anyone
can write it, so it is drawn as an assertion
A person asking "why is this here?" wants all three at their fingertips,
any subset, or none — so the choice is a live toggle, remembered per
browser like the ribbon style, never a per-document setting.
A FOURTH family joined them (docs/specs/freeform/provenance-across-versions.md):
- continuity — this span and that one, at an earlier commit, are the same
thing. Not forgeable where recorded, and it says whether it
was recorded or guessed, and how precisely.
It is different from the other three in a way that shows up here: they
answer "why is this here" about the present, and it answers "what was this
before". It is also OFF BY DEFAULT and its toggle is not a browser
preference — the whole feature rides one project switch, because turning
the drawing on without the bookkeeping would draw nothing, and turning the
bookkeeping on without asking would tax every commit for a feature most
people never use.

<storycode:copy id="f-031f3259ebc6-ProvenanceLayer">
// Which PROVENANCES the overlay draws. Three kinds live in this product and
// must never look alike (docs/specs/freeform/three-provenances.md):
//
// - lineage  — the weave's byte-exact derivation; recomputable from source bytes
// - context  — what was in front of the model when it wrote these lines,
//              derived from the session record; records the supplied context,
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
export type ProvenanceLayer = "lineage" | "context" | "declared" | "continuity" | "execution";

</storycode:copy>
<a id="f-031f3259ebc6-PROVENANCE_LAYERS"></a>
### `PROVENANCE_LAYERS`

<storycode:copy id="f-031f3259ebc6-PROVENANCE_LAYERS">
export const PROVENANCE_LAYERS: readonly ProvenanceLayer[] = [
  "lineage",
  "context",
  "declared",
  "continuity",
  "execution",
];

</storycode:copy>
<a id="f-031f3259ebc6-PROVENANCE_LAYERS_KEY"></a>
### `PROVENANCE_LAYERS_KEY`

<storycode:copy id="f-031f3259ebc6-PROVENANCE_LAYERS_KEY">
export const PROVENANCE_LAYERS_KEY = "hickory.provenanceLayers";

</storycode:copy>
<a id="f-031f3259ebc6-DEFAULT_LAYERS"></a>
### `DEFAULT_LAYERS`

What a reader who has expressed no preference gets.

<storycode:copy id="f-031f3259ebc6-DEFAULT_LAYERS">
/** What a reader who has expressed no preference gets. */
export const DEFAULT_LAYERS: readonly ProvenanceLayer[] = ["lineage", "context", "declared"];

</storycode:copy>
<a id="f-031f3259ebc6-isLayer"></a>
### `isLayer`

<storycode:copy id="f-031f3259ebc6-isLayer">
const isLayer = (value: unknown): value is ProvenanceLayer =>
  value === "lineage" ||
  value === "context" ||
  value === "declared" ||
  value === "continuity" ||
  value === "execution";

</storycode:copy>
<a id="f-031f3259ebc6-loadProvenanceLayers"></a>
### `loadProvenanceLayers`

The persisted set, defaulting to every layer on.

<storycode:copy id="f-031f3259ebc6-loadProvenanceLayers">
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

</storycode:copy>
<a id="f-031f3259ebc6-saveProvenanceLayers"></a>
### `saveProvenanceLayers`

<storycode:copy id="f-031f3259ebc6-saveProvenanceLayers">
export function saveProvenanceLayers(
  layers: ReadonlySet<ProvenanceLayer>,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(
    PROVENANCE_LAYERS_KEY,
    JSON.stringify(PROVENANCE_LAYERS.filter((l) => layers.has(l))),
  );
}

</storycode:copy>
<a id="f-031f3259ebc6-toggleLayer"></a>
### `toggleLayer`

<storycode:copy id="f-031f3259ebc6-toggleLayer">
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

</storycode:copy>
<a id="f-031f3259ebc6-LAYER_LABEL"></a>
### `LAYER_LABEL`

What each layer is, in one line — the toggle's tooltip.

<storycode:copy id="f-031f3259ebc6-LAYER_LABEL">
/** What each layer is, in one line — the toggle's tooltip. */
export const LAYER_LABEL: Record<ProvenanceLayer, { short: string; tip: string }> = {
  lineage: {
    short: "Lineage",
    tip: "Lineage — where these bytes came from, by the weave. Derived byte correspondence, not evidence that a claim is true.",
  },
  context: {
    short: "Context",
    tip: "Context — what was in front of the model when an agent wrote these lines, from the session record. Derived; says present, not used.",
  },
  declared: {
    short: "Declared",
    tip: "Declared — what the author says this cites (cites=). An assertion anyone can write; drawn as one.",
  },
  execution: { short: "Execution", tip: "Execution — observed runtime emission/calling evidence, not byte lineage, supplied context or semantic support." },
  continuity: {
    short: "Continuity",
    tip: "Continuity — what this span was at an earlier commit. Off by default, and the whole feature rides one project switch: no continuity, no journal, no repair. Every link says whether it was recorded or guessed, and how precisely.",
  },
};
</storycode:copy>

### Assemble `apps/web/src/lib/provenanceLayers.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/provenanceLayers.ts"><storycode:paste select="#f-031f3259ebc6-ProvenanceLayer"/><storycode:paste select="#f-031f3259ebc6-PROVENANCE_LAYERS"/><storycode:paste select="#f-031f3259ebc6-PROVENANCE_LAYERS_KEY"/><storycode:paste select="#f-031f3259ebc6-DEFAULT_LAYERS"/><storycode:paste select="#f-031f3259ebc6-isLayer"/><storycode:paste select="#f-031f3259ebc6-loadProvenanceLayers"/><storycode:paste select="#f-031f3259ebc6-saveProvenanceLayers"/><storycode:paste select="#f-031f3259ebc6-toggleLayer"/><storycode:paste select="#f-031f3259ebc6-LAYER_LABEL"/></storycode:file>

<a id="output-2a3d9afad8f1"></a>
## `apps/web/src/shell/ProvenanceToggles.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Three Provenances Are Drawn Apart, And Any Subset Can Be Shown](contract.md).

<a id="f-2a3d9afad8f1-ProvenanceToggles"></a>
### `ProvenanceToggles`

<storycode:copy id="f-2a3d9afad8f1-ProvenanceToggles">
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
</storycode:copy>

### Assemble `apps/web/src/shell/ProvenanceToggles.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/ProvenanceToggles.tsx"><storycode:paste select="#f-2a3d9afad8f1-ProvenanceToggles"/></storycode:file>

## Test the behavior: `cites_resolve_across_the_chain_and_report_dangling_ones`

This is the maintained test fragment from `crates/hickory-cli/tests/cites.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a19643f3dc7c-cites_resolve_across_the_chain_and_report_dangling_ones"></a>
### `cites_resolve_across_the_chain_and_report_dangling_ones`

Declared provenance: `cites="…"` is what an author SAYS an element rests
on. It resolves to places so it can be drawn, and it weaves as an
assertion in words — never as lineage, never as a check.

<storycode:copy id="f-a19643f3dc7c-cites_resolve_across_the_chain_and_report_dangling_ones">
//! Declared provenance: `cites="…"` is what an author SAYS an element rests
//! on. It resolves to places so it can be drawn, and it weaves as an
//! assertion in words — never as lineage, never as a check.
//! Guarantee: docs/guarantees/lineage/three-provenances-are-drawn-apart.md

use hickory_cli::{CheckFailure, CheckOutcome, check_outcome, dangling_citations, declared_cites};

#[test]
fn cites_resolve_across_the_chain_and_report_dangling_ones() {
    let dir = tempfile::tempdir().unwrap();
    let meeting = dir.path().join("meeting.md");
    std::fs::write(
        &meeting,
        "<hick:transcript id=\"t\" format=\"vtt\">\nWEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Forty is safe.\n</hick:transcript>\n",
    )
    .unwrap();
    let note = dir.path().join("note.md");
    std::fs::write(
        &note,
        "<hick:upstream file=\"meeting.md\" />\n\n<hick:copy id=\"finding\" class=\"f\">Pool is 40.</hick:copy>\n\n<hick:claim by=\"nate\" standing=\"judgment\" cites=\"#t-u1,#finding,#missing\">We are fine.</hick:claim>\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&note).unwrap();
    let cites = declared_cites(&note, &source).unwrap();
    assert_eq!(cites.len(), 1, "{cites:?}");
    let c = &cites[0];
    assert_eq!(c.from.element, "claim");
    assert_eq!((c.from.first_line, c.from.last_line), (5, 5));
    assert_eq!(
        c.to.len(),
        2,
        "the dangling #missing resolves to nothing: {:?}",
        c.to
    );
    // NAMED, not merely absent. This assertion is the point of the test's
    // own title and was missing: `to` having two entries says nothing about
    // WHICH of the three selectors contributed them, so `#missing` was
    // dropped in silence and wove beside the two that resolved.
    assert_eq!(c.dangling, vec!["#missing".to_string()], "{c:?}");
    let turn =
        c.to.iter()
            .find(|p| p.element == "said")
            .expect("the meeting turn");
    assert!(turn.path.ends_with("meeting.md"), "{}", turn.path);
    assert_eq!(turn.first_line, 5);
    let finding = c.to.iter().find(|p| p.element == "copy").unwrap();
    assert!(finding.path.ends_with("note.md"));
    assert_eq!(finding.id.as_deref(), Some("finding"));
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/lib/provenanceLayers.ts](contract.md#output-031f3259ebc6)
- [apps/web/src/shell/ProvenanceToggles.tsx](contract.md#output-2a3d9afad8f1)
- [apps/web/src/shell/Ribbons.tsx](../../../mechanisms/apps/web/src/shell/Ribbons.tsx.md#output-b916e85d3b0b)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hick-handlers/src/handlers/transform.rs](../../../mechanisms/crates/hick-handlers/src/handlers/transform.rs.md#output-399538a999c4)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [apps/web/src/lib/provenanceLayers.test.ts](contract.md#output-bb5abd922b4d)
- [crates/hickory-cli/tests/cites.rs](../../../mechanisms/crates/hickory-cli/tests/cites.rs.md#output-a19643f3dc7c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
