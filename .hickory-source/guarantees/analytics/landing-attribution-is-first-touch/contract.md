<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Landing Attribution Credits The First Touch, And Survives The Hash Router

For: Maintainers evaluating landing-page behavior and visitor privacy.

Given a visitor arriving with UTM parameters in the query string
(`https://…/?utm_source=hn&utm_campaign=launch#/`), when the landing page
reports any event, then those parameters are read from the query string —
which sits before the fragment, so the hash router never consumes them — and
the *first* tagged arrival is recorded permanently as that visitor's first
touch. A later visit from a different source reports its own `utm_*` values
but does not overwrite the first touch or the derived `intended_segment`. An
untagged arrival never stamps a first touch, so opening the bare domain
before clicking an ad does not permanently credit the visitor as direct.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/analytics/landing-attribution-is-first-touch.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-b355227ccd07"></a>
## `apps/web/src/analytics/attribution.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Landing Attribution Credits The First Touch, And Survives The Hash Router](contract.md).

<a id="f-b355227ccd07-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b355227ccd07-expression_statement">
import { beforeEach, describe, expect, it } from "vitest";

import { attribution, readUtm, referringDomain, visitorId } from "./attribution";

beforeEach(() => localStorage.clear());

</storycode:copy>
<a id="f-b355227ccd07-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b355227ccd07-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/analytics/attribution.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/analytics/attribution.test.ts"><storycode:paste select="#f-b355227ccd07-expression_statement"/><storycode:paste select="#f-b355227ccd07-expression_statement-2"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/analytics/attribution.ts](../../../mechanisms/apps/web/src/analytics/attribution.ts.md#output-433f931d57e0)
- [apps/web/src/analytics/events.ts](../../../mechanisms/apps/web/src/analytics/events.ts.md#output-776c02024e1f)

### Tests and fixtures

- [apps/web/src/analytics/attribution.test.ts](contract.md#output-b355227ccd07)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
