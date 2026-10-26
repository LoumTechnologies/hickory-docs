<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/analytics/events.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-776c02024e1f"></a>
## `apps/web/src/analytics/events.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Landing Attribution Credits The First Touch, And Survives The Hash Router](../../../../../guarantees/analytics/landing-attribution-is-first-touch/contract.md), [Driving The Home-Page Demo Is Reported As Revealed Interest](../../../../../guarantees/analytics/landing-demo-engagement/contract.md).

<a id="f-776c02024e1f-LandingEvent"></a>
### `LandingEvent`

The landing page's event vocabulary. This union is one half of a contract:
the other half is ALLOWED_EVENTS in apps/server/src/routes/analytics.rs.
Adding an event means adding it in both places — the server rejects, with a
message naming this file, anything it does not know.

<storycode:copy id="f-776c02024e1f-LandingEvent">
// The landing page's event vocabulary. This union is one half of a contract:
// the other half is ALLOWED_EVENTS in apps/server/src/routes/analytics.rs.
// Adding an event means adding it in both places — the server rejects, with a
// message naming this file, anything it does not know.

import { attribution, declaredSegment, setDeclaredSegment } from "./attribution";
import { send, type ScalarProps } from "./sink";

export type LandingEvent =
  /** A visit began. Carries everything about *how* they arrived. */
  | { name: "landing_viewed"; path: string }
  /**
   * An interest section was opened or closed. `phase: "open"` builds the
   * interest vector; `phase: "close"` carries the dwell that says whether the
   * open was a real read or a bounce off a bad heading.
   */
  | {
      name: "interest_expanded";
      interest_id: string;
      phase: "open" | "close";
      dwell_ms?: number;
      open_index?: number;
    }
  /** A link inside an interest section was followed. */
  | { name: "interest_clicked"; interest_id: string; link_id: string }
  /** The visitor answered the light identity anchor. */
  | { name: "segment_declared"; declared_segment: string }
  /** A call to action was taken. */
  | { name: "cta_clicked"; cta_id: string }
  /**
   * The visitor drove one of the home page's demos. `step` is which part they
   * reached — the walkthrough step id, or the interaction ("round-trip",
   * "pull") for the demos that are not stepped. This is revealed interest at
   * its strongest: it costs effort, so it separates people who read the page
   * from people who tried the product.
   */
  // `autoplay` is retained on the wire but no longer sent: the page's one demo
  // does not play itself. Keeping the field means a historical query that
  // splits on it still parses rather than erroring on the old rows.
  | { name: "demo_engaged"; demo_id: string; step: string; autoplay?: boolean };

</storycode:copy>
<a id="f-776c02024e1f-baseProperties"></a>
### `baseProperties`

Properties every event carries, so declared-vs-intended is answerable from
any single event rather than only by joining sessions together.

<storycode:copy id="f-776c02024e1f-baseProperties">
/**
 * Properties every event carries, so declared-vs-intended is answerable from
 * any single event rather than only by joining sessions together.
 */
function baseProperties(): ScalarProps {
  const a = attribution();
  const props: ScalarProps = {
    referring_domain: a.referringDomain,
    intended_segment: a.intendedSegment ?? "$none",
    declared_segment: declaredSegment() ?? "$none",
  };
  for (const [key, value] of Object.entries(a.utm)) props[key] = value;
  return props;
}

</storycode:copy>
<a id="f-776c02024e1f-emit"></a>
### `emit`

<storycode:copy id="f-776c02024e1f-emit">
export function emit(event: LandingEvent) {
  const { name, ...rest } = event;
  const properties: ScalarProps = { ...baseProperties() };
  for (const [key, value] of Object.entries(rest)) {
    if (value !== undefined) properties[key] = value as string | number | boolean;
  }
  // segment_declared must record the *new* claim, not the stale one that
  // baseProperties read a moment before the visitor changed it.
  if (name === "segment_declared") {
    properties.declared_segment = event.declared_segment;
    setDeclaredSegment(event.declared_segment);
  }
  send(attribution().visitorId, name, properties);
}
</storycode:copy>

### Assemble `apps/web/src/analytics/events.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/analytics/events.ts"><storycode:paste select="#f-776c02024e1f-LandingEvent"/><storycode:paste select="#f-776c02024e1f-baseProperties"/><storycode:paste select="#f-776c02024e1f-emit"/></storycode:file>

</storycode:doc>
