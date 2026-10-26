<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Every Claim On The Home Page Is True Of The Downloaded Binary

For: Prospective users checking what the homepage demonstrates and promises.

Given a visitor reading hickorydocs.com, when they act on anything the page
tells them — a command, a flag, a file the tool writes, a hick fragment they
paste — then it behaves as the page said, because the page describes only what
the shipped binary does.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-d8866b6fc796"></a>
## `apps/web/src/views/StaticSite.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Claim On The Home Page Is True Of The Downloaded Binary](contract.md).

<a id="f-d8866b6fc796-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d8866b6fc796-expression_statement">
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { setDeliveryForTest } from "../analytics/sink";
import { LandingView, resetViewedForTest } from "./LandingView";

/**
 * The marketing site is a static file: no API, no accounts, no billing.
 *
 * These tests protect that property from the direction it will actually be
 * broken — someone adding a fetch to the landing path, or a subscribe button to
 * a page with nothing behind it. See docs/specs/freeform/local-first.md.
 */

beforeEach(() => {
  localStorage.clear();
  resetViewedForTest();
  setDeliveryForTest(() => {});
});

</storycode:copy>
<a id="f-d8866b6fc796-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d8866b6fc796-expression_statement-2">
afterEach(cleanup);

</storycode:copy>
<a id="f-d8866b6fc796-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d8866b6fc796-expression_statement-3">
describe("the marketing site with no server behind it", () => {
  it("renders the landing page without making a single request", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    render(<LandingView />);
    expect(screen.getByRole("heading", { level: 1 })).toBeTruthy();
    // Rendering mounts no API client. Asset loading and the real worker
    // debugger are exercised by the static Playwright acceptance suite.
    expect(fetchSpy).not.toHaveBeenCalled();
    fetchSpy.mockRestore();
  });

  it("offers installing the tool, not signing up for a workspace", () => {
    const { container } = render(<LandingView />);
    expect(screen.getByRole("link", { name: "Get the desktop app" })).toBeTruthy();
    expect(container.textContent).toContain("Downloadable software");
    // The hosted front door is hidden unless this build is the hosted app.
    expect(container.textContent).not.toContain("Start free");
    expect(container.textContent).not.toContain("hosted workspace");
  });

});
</storycode:copy>

### Assemble `apps/web/src/views/StaticSite.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/StaticSite.test.tsx"><storycode:paste select="#f-d8866b6fc796-expression_statement"/><storycode:paste select="#f-d8866b6fc796-expression_statement-2"/><storycode:paste select="#f-d8866b6fc796-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/AgentToolSurface.tsx](../../../mechanisms/apps/web/src/components/AgentToolSurface.tsx.md#output-9fa0a353dc49)
- [apps/web/src/landing/interests.ts](../../../mechanisms/apps/web/src/landing/interests.ts.md#output-016206bf5500)
- [apps/web/src/views/LandingView.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-e86ad48a0990)
- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [apps/web/src/landing/demos/demos.test.tsx](../../../mechanisms/apps/web/src/landing/demos/demos.test.tsx.md#output-e2fe96c33971)
- [apps/web/src/landing/demos/scripts.test.ts](../../../mechanisms/apps/web/src/landing/demos/scripts.test.ts.md#output-1555db6bf9bb)
- [apps/web/src/views/StaticSite.test.tsx](contract.md#output-d8866b6fc796)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
