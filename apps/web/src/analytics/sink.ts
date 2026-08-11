// Where events go.
//
// Two deliveries, chosen by configuration rather than by branch:
//
// - **Direct to PostHog** when `VITE_POSTHOG_KEY` is set. A project write key
//   can only send events, so it is safe in a bundle — and it is what lets the
//   marketing site be a static file with no server behind it at all
//   (docs/specs/freeform/local-first.md).
// - **The server's same-origin beacon** otherwise, which is how the hosted app
//   has always worked (`apps/server/src/routes/analytics.rs` forwards with the
//   credential it holds).
//
// The original reason for routing through the server was that a build-time key
// would bake one environment's project into an image promoted between
// environments. With a single production environment there is nothing to
// promote, so that objection no longer applies — but the beacon stays, because
// a deployment that prefers not to put a key in the bundle should not have to.

import { config } from "../config";

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
  const direct = config.posthogKey !== null;
  const url = direct
    ? `${config.posthogHost}/capture/`
    : "/api/analytics/capture";
  // PostHog's capture endpoint wants the key in the payload; the server's
  // beacon supplies its own. Same event, same properties, either way — the
  // shape the server forwards is deliberately PostHog's own.
  const payload = direct ? { api_key: config.posthogKey, ...(body as object) } : body;

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
