<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/analytics/attribution.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-433f931d57e0"></a>
## `apps/web/src/analytics/attribution.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Landing Attribution Credits The First Touch, And Survives The Hash Router](../../../../../guarantees/analytics/landing-attribution-is-first-touch/contract.md).

<a id="f-433f931d57e0-VISITOR_KEY"></a>
### `VISITOR_KEY`

Who arrived, and where from. Everything here is derived once per page load
and is deliberately boring: no fingerprinting, no third-party script, no
cross-site identifier. The anonymous id is a random UUID this browser
generated for itself and can clear at any time.

<storycode:copy id="f-433f931d57e0-VISITOR_KEY">
// Who arrived, and where from. Everything here is derived once per page load
// and is deliberately boring: no fingerprinting, no third-party script, no
// cross-site identifier. The anonymous id is a random UUID this browser
// generated for itself and can clear at any time.

const VISITOR_KEY = "hickory.visitor";
</storycode:copy>
<a id="f-433f931d57e0-FIRST_TOUCH_KEY"></a>
### `FIRST_TOUCH_KEY`

<storycode:copy id="f-433f931d57e0-FIRST_TOUCH_KEY">
const FIRST_TOUCH_KEY = "hickory.first_touch";
</storycode:copy>
<a id="f-433f931d57e0-DECLARED_KEY"></a>
### `DECLARED_KEY`

<storycode:copy id="f-433f931d57e0-DECLARED_KEY">
const DECLARED_KEY = "hickory.declared_segment";

</storycode:copy>
<a id="f-433f931d57e0-UTM_KEYS"></a>
### `UTM_KEYS`

The UTM parameters an ad or community link may carry.

<storycode:copy id="f-433f931d57e0-UTM_KEYS">
/** The UTM parameters an ad or community link may carry. */
export const UTM_KEYS = [
  "utm_source",
  "utm_medium",
  "utm_campaign",
  "utm_content",
  "utm_term",
] as const;

</storycode:copy>
<a id="f-433f931d57e0-UtmKey"></a>
### `UtmKey`

<storycode:copy id="f-433f931d57e0-UtmKey">
export type UtmKey = (typeof UTM_KEYS)[number];
</storycode:copy>
<a id="f-433f931d57e0-Utm"></a>
### `Utm`

<storycode:copy id="f-433f931d57e0-Utm">
export type Utm = Partial<Record<UtmKey, string>>;

</storycode:copy>
<a id="f-433f931d57e0-Attribution"></a>
### `Attribution`

<storycode:copy id="f-433f931d57e0-Attribution">
export type Attribution = {
  /** Anonymous, browser-local, stable across visits until storage is cleared. */
  visitorId: string;
  /** UTM values on *this* page load. */
  utm: Utm;
  /** UTM values from the visitor's very first landing — never overwritten. */
  firstTouch: Utm;
  /** Referrer host, or `"$direct"` when the browser sent none. */
  referringDomain: string;
  /**
   * The segment the *campaign* believed it was buying, read from
   * `utm_campaign` on first touch. Null in the discovery posture, where no
   * segment has been named yet — which is the point: discovery finds them.
   */
  intendedSegment: string | null;
};

</storycode:copy>
<a id="f-433f931d57e0-readStorage"></a>
### `readStorage`

<storycode:copy id="f-433f931d57e0-readStorage">
function readStorage(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    // Private mode / storage disabled. Everything below degrades to a
    // per-page-load identity rather than failing the page.
    return null;
  }
}

</storycode:copy>
<a id="f-433f931d57e0-writeStorage"></a>
### `writeStorage`

<storycode:copy id="f-433f931d57e0-writeStorage">
function writeStorage(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* see readStorage */
  }
}

</storycode:copy>
<a id="f-433f931d57e0-newId"></a>
### `newId`

<storycode:copy id="f-433f931d57e0-newId">
function newId(): string {
  if (typeof crypto?.randomUUID === "function") return crypto.randomUUID();
  // crypto.randomUUID needs a secure context; plain-http dev origins get a
  // weaker id rather than no id, because an unmeasurable dev page hides
  // instrumentation bugs until they reach staging.
  return `nc-${Math.random().toString(36).slice(2)}${Date.now().toString(36)}`;
}

</storycode:copy>
<a id="f-433f931d57e0-visitorId"></a>
### `visitorId`

<storycode:copy id="f-433f931d57e0-visitorId">
export function visitorId(): string {
  const existing = readStorage(VISITOR_KEY);
  if (existing) return existing;
  const id = newId();
  writeStorage(VISITOR_KEY, id);
  return id;
}

</storycode:copy>
<a id="f-433f931d57e0-readUtm"></a>
### `readUtm`

UTM parameters live in the query string, which sits *before* the hash —
`https://…/?utm_source=hn#/` — so the hash router never sees or eats them.

<storycode:copy id="f-433f931d57e0-readUtm">
/**
 * UTM parameters live in the query string, which sits *before* the hash —
 * `https://…/?utm_source=hn#/` — so the hash router never sees or eats them.
 */
export function readUtm(search: string): Utm {
  const params = new URLSearchParams(search);
  const utm: Utm = {};
  for (const key of UTM_KEYS) {
    const value = params.get(key);
    // Cap at the server's per-property limit so a padded campaign value is
    // truncated here rather than rejected as a 400 by the beacon.
    if (value) utm[key] = value.slice(0, 300);
  }
  return utm;
}

</storycode:copy>
<a id="f-433f931d57e0-firstTouch"></a>
### `firstTouch`

<storycode:copy id="f-433f931d57e0-firstTouch">
function firstTouch(current: Utm): Utm {
  const stored = readStorage(FIRST_TOUCH_KEY);
  if (stored) {
    try {
      return JSON.parse(stored) as Utm;
    } catch {
      /* corrupt value: fall through and re-stamp from this load */
    }
  }
  // Only stamp when this load actually carries attribution. Stamping an empty
  // object would permanently record a visitor's first touch as "$direct" if
  // they happened to open the bare domain before clicking an ad.
  if (Object.keys(current).length === 0) return {};
  writeStorage(FIRST_TOUCH_KEY, JSON.stringify(current));
  return current;
}

</storycode:copy>
<a id="f-433f931d57e0-referringDomain"></a>
### `referringDomain`

<storycode:copy id="f-433f931d57e0-referringDomain">
export function referringDomain(referrer: string): string {
  if (!referrer) return "$direct";
  try {
    return new URL(referrer).hostname;
  } catch {
    return "$direct";
  }
}

</storycode:copy>
<a id="f-433f931d57e0-declaredSegment"></a>
### `declaredSegment`

The last identity the visitor claimed for themselves, if they claimed one.

<storycode:copy id="f-433f931d57e0-declaredSegment">
/** The last identity the visitor claimed for themselves, if they claimed one. */
export function declaredSegment(): string | null {
  return readStorage(DECLARED_KEY);
}

</storycode:copy>
<a id="f-433f931d57e0-setDeclaredSegment"></a>
### `setDeclaredSegment`

<storycode:copy id="f-433f931d57e0-setDeclaredSegment">
export function setDeclaredSegment(segment: string) {
  writeStorage(DECLARED_KEY, segment);
}

</storycode:copy>
<a id="f-433f931d57e0-attribution"></a>
### `attribution`

<storycode:copy id="f-433f931d57e0-attribution">
export function attribution(loc: Location = location, referrer = document.referrer): Attribution {
  const utm = readUtm(loc.search);
  const first = firstTouch(utm);
  return {
    visitorId: visitorId(),
    utm,
    firstTouch: first,
    referringDomain: referringDomain(referrer),
    intendedSegment: first.utm_campaign ?? utm.utm_campaign ?? null,
  };
}
</storycode:copy>

### Assemble `apps/web/src/analytics/attribution.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/analytics/attribution.ts"><storycode:paste select="#f-433f931d57e0-VISITOR_KEY"/><storycode:paste select="#f-433f931d57e0-FIRST_TOUCH_KEY"/><storycode:paste select="#f-433f931d57e0-DECLARED_KEY"/><storycode:paste select="#f-433f931d57e0-UTM_KEYS"/><storycode:paste select="#f-433f931d57e0-UtmKey"/><storycode:paste select="#f-433f931d57e0-Utm"/><storycode:paste select="#f-433f931d57e0-Attribution"/><storycode:paste select="#f-433f931d57e0-readStorage"/><storycode:paste select="#f-433f931d57e0-writeStorage"/><storycode:paste select="#f-433f931d57e0-newId"/><storycode:paste select="#f-433f931d57e0-visitorId"/><storycode:paste select="#f-433f931d57e0-readUtm"/><storycode:paste select="#f-433f931d57e0-firstTouch"/><storycode:paste select="#f-433f931d57e0-referringDomain"/><storycode:paste select="#f-433f931d57e0-declaredSegment"/><storycode:paste select="#f-433f931d57e0-setDeclaredSegment"/><storycode:paste select="#f-433f931d57e0-attribution"/></storycode:file>

</storycode:doc>
