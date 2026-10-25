<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/analytics/attribution.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Landing Attribution Credits The First Touch, And Survives The Hash Router](../../../../../contract.md)

Captured SHA-256: `809d5ce463268b33c0e9e2b2ab76b3b87d36f04c0c0568d3f248d025b138a279`.

<book:entry path="apps/web/src/analytics/attribution.test.ts">
import { beforeEach, describe, expect, it } from "vitest";

import { attribution, readUtm, referringDomain, visitorId } from "./attribution";

beforeEach(() => localStorage.clear());

describe("attribution", () => {
  // Protects docs/guarantees/analytics/landing-attribution-is-first-touch.md
  it("reads UTM parameters from the query string, which the hash router never sees", () => {
    expect(readUtm("?utm_source=hn&utm_campaign=platform-team&unrelated=1")).toEqual({
      utm_source: "hn",
      utm_campaign: "platform-team",
    });
  });

  // Protects docs/guarantees/analytics/landing-attribution-is-first-touch.md
  it("truncates an over-long UTM value to the beacon's limit rather than being rejected", () => {
    const utm = readUtm(`?utm_content=${"x".repeat(500)}`);
    expect(utm.utm_content).toHaveLength(300);
  });

  // Protects docs/guarantees/analytics/landing-attribution-is-first-touch.md
  it("keeps the first touch when a later visit arrives from somewhere else", () => {
    const first = attribution(
      { search: "?utm_source=hn&utm_campaign=launch" } as Location,
      "https://news.ycombinator.com/item?id=1",
    );
    expect(first.firstTouch.utm_source).toBe("hn");
    expect(first.intendedSegment).toBe("launch");

    const second = attribution({ search: "?utm_source=reddit" } as Location, "");
    expect(second.utm.utm_source).toBe("reddit");
    // The click that actually acquired this visitor is the one worth crediting.
    expect(second.firstTouch.utm_source).toBe("hn");
    expect(second.intendedSegment).toBe("launch");
  });

  // Protects docs/guarantees/analytics/landing-attribution-is-first-touch.md
  it("does not stamp an untagged visit as the first touch", () => {
    attribution({ search: "" } as Location, "");
    const tagged = attribution({ search: "?utm_source=hn" } as Location, "");
    expect(tagged.firstTouch.utm_source).toBe("hn");
  });

  // Protects docs/guarantees/analytics/landing-attribution-is-first-touch.md
  it("reports no intended segment in the discovery posture", () => {
    const a = attribution({ search: "?utm_source=hn" } as Location, "");
    expect(a.intendedSegment).toBeNull();
  });

  it("labels a missing or unparseable referrer as direct", () => {
    expect(referringDomain("")).toBe("$direct");
    expect(referringDomain("not a url")).toBe("$direct");
    expect(referringDomain("https://news.ycombinator.com/x")).toBe("news.ycombinator.com");
  });

  it("keeps the anonymous visitor id stable across calls", () => {
    expect(visitorId()).toBe(visitorId());
  });
});
</book:entry>

</book:doc>
