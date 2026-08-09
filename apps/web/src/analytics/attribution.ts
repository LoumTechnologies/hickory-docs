// Who arrived, and where from. Everything here is derived once per page load
// and is deliberately boring: no fingerprinting, no third-party script, no
// cross-site identifier. The anonymous id is a random UUID this browser
// generated for itself and can clear at any time.

const VISITOR_KEY = "hickory.visitor";
const FIRST_TOUCH_KEY = "hickory.first_touch";
const DECLARED_KEY = "hickory.declared_segment";

/** The UTM parameters an ad or community link may carry. */
export const UTM_KEYS = [
  "utm_source",
  "utm_medium",
  "utm_campaign",
  "utm_content",
  "utm_term",
] as const;

export type UtmKey = (typeof UTM_KEYS)[number];
export type Utm = Partial<Record<UtmKey, string>>;

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

function readStorage(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    // Private mode / storage disabled. Everything below degrades to a
    // per-page-load identity rather than failing the page.
    return null;
  }
}

function writeStorage(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* see readStorage */
  }
}

function newId(): string {
  if (typeof crypto?.randomUUID === "function") return crypto.randomUUID();
  // crypto.randomUUID needs a secure context; plain-http dev origins get a
  // weaker id rather than no id, because an unmeasurable dev page hides
  // instrumentation bugs until they reach staging.
  return `nc-${Math.random().toString(36).slice(2)}${Date.now().toString(36)}`;
}

export function visitorId(): string {
  const existing = readStorage(VISITOR_KEY);
  if (existing) return existing;
  const id = newId();
  writeStorage(VISITOR_KEY, id);
  return id;
}

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

export function referringDomain(referrer: string): string {
  if (!referrer) return "$direct";
  try {
    return new URL(referrer).hostname;
  } catch {
    return "$direct";
  }
}

/** The last identity the visitor claimed for themselves, if they claimed one. */
export function declaredSegment(): string | null {
  return readStorage(DECLARED_KEY);
}

export function setDeclaredSegment(segment: string) {
  writeStorage(DECLARED_KEY, segment);
}

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
