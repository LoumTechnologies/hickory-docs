// Where events go. One same-origin POST per event to the server's beacon
// (`apps/server/src/routes/analytics.rs`), which forwards to PostHog with the
// credential it already has. The browser holds no analytics key.

import { MOCK } from "../api/client";

export type ScalarProps = Record<string, string | number | boolean>;

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

/** Overridable for tests; the default is the real beacon. */
let deliver = beacon;

export function setDeliveryForTest(fn: (event: string, body: unknown) => void) {
  deliver = fn;
}

function beacon(event: string, body: unknown) {
  // `keepalive` is what makes a click-then-navigate event survive: without it
  // the browser cancels in-flight fetches on unload, and every `cta_clicked`
  // — the events that matter most — would be lost exactly when they fire.
  void fetch("/api/analytics/capture", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
    keepalive: true,
  }).catch(() => {
    // Analytics must never surface as a user-visible failure, and an offline
    // or ad-blocked page must still work. Dropping the event is correct.
  });
  void event;
}

export function send(distinctId: string, event: string, properties: ScalarProps) {
  if (trackingRefused()) return;
  if (MOCK) {
    // The mock profile has no server. Logging keeps instrumentation
    // debuggable in `npm run dev:mock` instead of silently doing nothing.
    console.info("[analytics]", event, properties);
    return;
  }
  deliver(event, { distinct_id: distinctId, event, properties });
}
