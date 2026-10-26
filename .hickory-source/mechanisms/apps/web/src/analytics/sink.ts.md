<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/analytics/sink.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d259c2b0da38"></a>
## `apps/web/src/analytics/sink.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Driving The Home-Page Demo Is Reported As Revealed Interest](../../../../../guarantees/analytics/landing-demo-engagement/contract.md).

<a id="f-d259c2b0da38-ScalarProps"></a>
### `ScalarProps`

Where events go: direct to PostHog, and only when `VITE_POSTHOG_KEY` is
set. A project write key can only send events, so it is safe in a bundle —
and it is what lets the marketing site be a static file with no server
behind it at all (docs/specs/freeform/local-only.md). There is no
same-origin beacon any more: the server that proxied it is gone, and
without a key the event is simply dropped.

<storycode:copy id="f-d259c2b0da38-ScalarProps">
// Where events go: direct to PostHog, and only when `VITE_POSTHOG_KEY` is
// set. A project write key can only send events, so it is safe in a bundle —
// and it is what lets the marketing site be a static file with no server
// behind it at all (docs/specs/freeform/local-only.md). There is no
// same-origin beacon any more: the server that proxied it is gone, and
// without a key the event is simply dropped.

import { config } from "../config";

export type ScalarProps = Record<string, string | number | boolean>;

</storycode:copy>
<a id="f-d259c2b0da38-trackingRefused"></a>
### `trackingRefused`

Honour Do Not Track. This costs us data — see the declared unknowables in
docs/specs/freeform/landing-discovery.md — and we do it anyway, because a
product whose pitch is verifiable provenance cannot quietly ignore the one
signal a visitor has for saying no.

<storycode:copy id="f-d259c2b0da38-trackingRefused">
/**
 * Honour Do Not Track. This costs us data — see the declared unknowables in
 * docs/specs/freeform/landing-discovery.md — and we do it anyway, because a
 * product whose pitch is verifiable provenance cannot quietly ignore the one
 * signal a visitor has for saying no.
 */
function trackingRefused(): boolean {
  const nav = navigator as Navigator & { msDoNotTrack?: string };
  const dnt = nav.doNotTrack ?? nav.msDoNotTrack ?? (window as { doNotTrack?: string }).doNotTrack;
  return dnt === "1" || dnt === "yes";
}

</storycode:copy>
<a id="f-d259c2b0da38-deliver"></a>
### `deliver`

Overridable for tests; the default is the real beacon.

<storycode:copy id="f-d259c2b0da38-deliver">
/** Overridable for tests; the default is the real beacon. */
let deliver = beacon;

</storycode:copy>
<a id="f-d259c2b0da38-setDeliveryForTest"></a>
### `setDeliveryForTest`

<storycode:copy id="f-d259c2b0da38-setDeliveryForTest">
export function setDeliveryForTest(fn: (event: string, body: unknown) => void) {
  deliver = fn;
}

</storycode:copy>
<a id="f-d259c2b0da38-beacon"></a>
### `beacon`

<storycode:copy id="f-d259c2b0da38-beacon">
function beacon(event: string, body: unknown) {
  // No key, no delivery. There is no server-side beacon to fall back to.
  if (config.posthogKey === null) return;
  const url = `${config.posthogHost}/capture/`;
  // PostHog's capture endpoint wants the key in the payload.
  const payload = { api_key: config.posthogKey, ...(body as object) };

  // `keepalive` is what makes a click-then-navigate event survive: without it
  // the browser cancels in-flight fetches on unload, and every `cta_clicked`
  // — the events that matter most — would be lost exactly when they fire.
  void fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
    keepalive: true,
  }).catch(() => {
    // Analytics must never surface as a user-visible failure, and an offline
    // or ad-blocked page must still work. Dropping the event is correct.
  });
  void event;
}

</storycode:copy>
<a id="f-d259c2b0da38-send"></a>
### `send`

<storycode:copy id="f-d259c2b0da38-send">
export function send(distinctId: string, event: string, properties: ScalarProps) {
  if (trackingRefused()) return;
  if (config.mock) {
    // The mock profile has no server. Logging keeps instrumentation
    // debuggable in `npm run dev:mock` instead of silently doing nothing.
    console.info("[analytics]", event, properties);
    return;
  }
  deliver(event, { distinct_id: distinctId, event, properties });
}
</storycode:copy>

### Assemble `apps/web/src/analytics/sink.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/analytics/sink.ts"><storycode:paste select="#f-d259c2b0da38-ScalarProps"/><storycode:paste select="#f-d259c2b0da38-trackingRefused"/><storycode:paste select="#f-d259c2b0da38-deliver"/><storycode:paste select="#f-d259c2b0da38-setDeliveryForTest"/><storycode:paste select="#f-d259c2b0da38-beacon"/><storycode:paste select="#f-d259c2b0da38-send"/></storycode:file>

</storycode:doc>
