<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/toolTabs.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Pair identity without silently granting execution](../../../../../contract.md)
- [A Local Install Is a Launchable macOS App](../../../../../../../release/a-local-install-is-a-launchable-macos-app/contract.md)

Captured SHA-256: `454d68c487c34e89423c9dfde784ebdb95b11acb1ab1afa189610443ac389c99`.

<book:entry path="apps/web/src/views/toolTabs.test.ts">
// Every tool pane is reachable from the shell.
//
// This exists because two of them were NOT: `FleetPane` and `MergedView` were
// written, tested, and rendered by nothing — their own tests passed because a
// test renders a component directly, which is exactly the thing a user cannot
// do. A component with no route is dead code that looks finished.
//
// docs/specs/freeform/shell-layouts.md
import { describe, expect, it } from "vitest";

import {
  FLEET_TAB,
  GIT_TAB,
  MERGED_TAB,
  WELCOME_TAB,
  openFleetTab,
  openGitTab,
  openMergedTab,
  openWelcomeTab,
  initialWorkspace,
} from "./workspaceState";
import { panes } from "../shell/layout";

const targets = (layout: ReturnType<typeof initialWorkspace>) =>
  [...panes(layout.root)].flatMap((p) => p.tabs.map((t) => t.target));

describe("the tool tabs a person can open", () => {
  it("opens the fleet, and fronts the existing one rather than duplicating", () => {
    let layout = openFleetTab(initialWorkspace());
    expect(targets(layout)).toContain(FLEET_TAB);
    layout = openFleetTab(layout);
    expect(targets(layout).filter((t) => t === FLEET_TAB)).toHaveLength(1);
  });

  it("opens a merged view per path, because the view IS of a path", () => {
    // Two files compared at once are two tabs, not one confused one.
    let layout = openMergedTab(initialWorkspace(), "src/a.rs");
    layout = openMergedTab(layout, "src/b.rs");
    expect(targets(layout)).toContain(`${MERGED_TAB}src/a.rs`);
    expect(targets(layout)).toContain(`${MERGED_TAB}src/b.rs`);
    // And the same path twice is still one tab.
    layout = openMergedTab(layout, "src/a.rs");
    expect(targets(layout).filter((t) => t === `${MERGED_TAB}src/a.rs`)).toHaveLength(1);
  });

  it("still opens the ones that already worked", () => {
    expect(targets(openGitTab(initialWorkspace()))).toContain(GIT_TAB);
    expect(targets(openWelcomeTab(initialWorkspace()))).toContain(WELCOME_TAB);
  });

  it("gives every tool tab a name a person can read on the strip", () => {
    for (const layout of [
      openFleetTab(initialWorkspace()),
      openGitTab(initialWorkspace()),
      openMergedTab(initialWorkspace(), "src/a.rs"),
    ]) {
      for (const pane of panes(layout.root)) {
        for (const tab of pane.tabs) {
          // A tool tab must name itself; a document tab is titled from its
          // path by the strip, so only tool tabs are asserted here.
          if (tab.kind === "tool") {
            expect((tab.title ?? "").trim().length).toBeGreaterThan(0);
          }
        }
      }
    }
  });
});
</book:entry>

</book:doc>
