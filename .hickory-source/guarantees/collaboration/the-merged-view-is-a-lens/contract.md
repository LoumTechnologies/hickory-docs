<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Merged View Is A Lens Over Several Worktrees, Never A Document

For: Engineers using local sessions or pairing their own machines.

Given a file that exists in several worktrees of one repository, when a merged
view is opened over it, then regions **every** source agrees on appear once,
regions that differ appear as variants naming each source, and the view says
plainly that it is read-only by default and exists on disk nowhere. Explicit
region edits name the targets opened for writing and use the captured source
preconditions; they do not give the lens a save path.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-merged-view-is-a-lens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4cbd2ec09430"></a>
## `apps/web/src/views/MergedView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Merged View Is A Lens Over Several Worktrees, Never A Document](contract.md).

<a id="f-4cbd2ec09430-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4cbd2ec09430-expression_statement">
// docs/guarantees/collaboration/the-merged-view-is-a-lens.md
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { api, installMockHandler } from "../api/client";
import { MergedView } from "./MergedView";

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

</storycode:copy>
<a id="f-4cbd2ec09430-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4cbd2ec09430-expression_statement-2">
afterEach(() => installMockHandler(null as never));

</storycode:copy>
<a id="f-4cbd2ec09430-worktrees"></a>
### `worktrees`

<storycode:copy id="f-4cbd2ec09430-worktrees">
const worktrees = (n = 2) =>
  vi.spyOn(api, "worktrees").mockResolvedValue({
    repository: n > 0,
    worktrees: [
      { path: "/repo", name: "repo", branch: "master", current: true },
      { path: "/side", name: "side-tree", branch: "side", current: false },
    ].slice(0, n),
  });

</storycode:copy>
<a id="f-4cbd2ec09430-merged"></a>
### `merged`

<storycode:copy id="f-4cbd2ec09430-merged">
const merged = (over: Partial<Parameters<typeof api.merged> extends never ? never : any> = {}) =>
  vi.spyOn(api, "merged").mockResolvedValue({
    repository: true,
    path: "shared.txt",
    sources: [
      { path: "/repo", name: "repo", branch: "master", current: true },
      { path: "/side", name: "side-tree", branch: "side", current: false },
    ],
    regions: [
      { kind: "shared", text: "head\n" },
      {
        kind: "variant",
        by_source: { repo: "master only\n", "side-tree": "side only\n" },
      },
      { kind: "shared", text: "tail\n" },
    ],
    missing: [],
    shared_lines: 2,
    variants: 1,
    read_only: true,
    ...over,
  });

</storycode:copy>
<a id="f-4cbd2ec09430-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4cbd2ec09430-expression_statement-3">
describe("the merged view", () => {
  it("shows agreed regions once and differences per source", async () => {
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() => expect(screen.getByText(/head/)).toBeTruthy());
    // The agreed lines appear ONCE, not once per source.
    expect(screen.getAllByText(/^head$/m)).toHaveLength(1);
    expect(screen.getByText(/master only/)).toBeTruthy();
    expect(screen.getByText(/side only/)).toBeTruthy();
  });

  it("says it is a lens rather than a document you could save", async () => {
    // Getting this wrong reverses the product's central claim.
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() => expect(screen.getByText(/lens · read-only/)).toBeTruthy());
  });

  it("scopes agreement to the capture and says later writes recheck target bytes", async () => {
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/agreed by construction for this capture/)).toBeTruthy(),
    );
    expect(screen.getByText(/recheck each target’s captured bytes/)).toBeTruthy();
  });

  it("names a worktree that does not have the file rather than dropping it", async () => {
    worktrees();
    merged({ missing: ["side-tree"] });
    render(<MergedView path="only-here.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/does not\s+have this file at all/)).toBeTruthy(),
    );
    expect(screen.getByText(/an answer, not an omission/)).toBeTruthy();
  });

  it("says what a one-worktree repository would show, rather than an empty pane", async () => {
    worktrees(1);
    merged({ sources: [], regions: [], shared_lines: 0, variants: 0 });
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/would just\s+be the file/)).toBeTruthy(),
    );
    expect(screen.getByText(/git worktree add/)).toBeTruthy();
  });

  it("says plainly when a folder is not a repository", async () => {
    vi.spyOn(api, "worktrees").mockResolvedValue({ repository: false, worktrees: [] });
    vi.spyOn(api, "merged").mockResolvedValue({
      repository: false,
      sources: [],
      regions: [],
      missing: [],
    });
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/not a git repository with worktrees/)).toBeTruthy(),
    );
  });
});

</storycode:copy>
<a id="f-4cbd2ec09430-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-4cbd2ec09430-expression_statement-4">
it("routes a native-opened region through real agent draft edits, host callback replies and native guarded undo", async () => {
  worktrees();
  const original = { repository: true, path: "shared.txt", sources: [
    { path: "/repo", name: "repo", current: true }, { path: "/side", name: "side-tree", current: false },
  ], regions: [{ kind: "shared" as const, text: "old" }], missing: [], snapshot: { repo: "old", "side-tree": "old" } };
  const captured = vi.spyOn(api, "merged").mockResolvedValue(original);
  const posts: { path: string; body: unknown }[] = [];
  const undo = [{ source: "repo", text: "old", expected: "edited" }];
  let refuseWrite = false;
  installMockHandler(async (_method, path, body) => {
    posts.push({ path, body });
    if (path === "/api/merged/write") {
      if (refuseWrite) throw Error("fixture transport uncertainty");
      captured.mockResolvedValueOnce({ ...original, snapshot: { repo: "edited", "side-tree": "old" }, regions: [{ kind: "variant", by_source: { repo: "edited", "side-tree": "old" } }] });
      return { targets: [{ source: "repo", ok: true }], undo };
    }
    if (path === "/api/merged/undo") return { targets: [{ source: "repo", ok: true }] };
    throw Error(`Unexpected fixture request ${path}`);
  });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><MergedView path="shared.txt" /></UiActionContext.Provider>);
  await screen.findByLabelText("Open repo for writing");
  fireEvent.click(screen.getByLabelText("Open repo for writing"));
  await waitFor(() => expect((screen.getByLabelText("Open repo for writing") as HTMLInputElement).checked).toBe(true));
  fireEvent.click(screen.getByRole("button", { name: "Edit shared region" }));
  await screen.findByRole("region", { name: "Worktree region edit" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label && action.enabled)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  await run("Replace comparison draft range", { from: 0, to: 3, text: "edited" });
  expect(await run("Apply worktree region draft")).toMatchObject({ status: "result", outcome: { value: { path: "shared.txt", target: "repo", route: "shared", response: { targets: [{ source: "repo", ok: true }] } } } });
  expect(posts[0]).toEqual({ path: "/api/merged/write", body: { path: "shared.txt", targets: ["repo", "side-tree"], writable: ["repo"], expected: original.snapshot, region: 0, text: "edited", route: "shared", source: "repo" } });
  expect(screen.queryByRole("region", { name: "Worktree region edit" })).toBeNull();
  fireEvent.click(await screen.findByRole("button", { name: "Undo repo" }));
  await waitFor(() => expect(posts[1]).toEqual({ path: "/api/merged/undo", body: { path: "shared.txt", undo } }));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Undo repo" })).toBeNull());
  fireEvent.click(screen.getByRole("button", { name: "Edit shared region" }));
  await screen.findByRole("region", { name: "Worktree region edit" });
  await run("Replace comparison draft range", { from: 0, to: 3, text: "retry draft" });
  refuseWrite = true;
  expect(await run("Apply worktree region draft")).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect((screen.getByRole("button", { name: "Apply region to selected targets" }) as HTMLButtonElement).disabled).toBe(true);
  expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(false);
  expect(screen.getByText(/from an earlier request/)).toBeTruthy();
  expect(screen.getByText(/Latest write request failed/)).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/views/MergedView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/MergedView.test.tsx"><storycode:paste select="#f-4cbd2ec09430-expression_statement"/><storycode:paste select="#f-4cbd2ec09430-expression_statement-2"/><storycode:paste select="#f-4cbd2ec09430-worktrees"/><storycode:paste select="#f-4cbd2ec09430-merged"/><storycode:paste select="#f-4cbd2ec09430-expression_statement-3"/><storycode:paste select="#f-4cbd2ec09430-expression_statement-4"/></storycode:file>

## Test the behavior: `git`

This is the maintained test fragment from `crates/hickory-cli/tests/merged_view.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-87756f3e699e-git"></a>
### `git`

The merged view over two real local worktrees, read-only.
Protects `docs/guarantees/collaboration/the-merged-view-is-a-lens.md`.
This is where the alignment risk lives: a bad alignment routes an edit
silently into the wrong file, so the first step is deliberately read-only
and deliberately conservative — fewer regions declared shared, more shown
as variants, because the failure of over-sharing is an edit in the wrong
place and the failure of under-sharing is a little redundant typing.

<storycode:copy id="f-87756f3e699e-git">
//! The merged view over two real local worktrees, read-only.
//!
//! Protects `docs/guarantees/collaboration/the-merged-view-is-a-lens.md`.
//!
//! This is where the alignment risk lives: a bad alignment routes an edit
//! silently into the wrong file, so the first step is deliberately read-only
//! and deliberately conservative — fewer regions declared shared, more shown
//! as variants, because the failure of over-sharing is an edit in the wrong
//! place and the failure of under-sharing is a little redundant typing.

use std::path::{Path, PathBuf};

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/views/MergedView.tsx](../../../mechanisms/apps/web/src/views/MergedView.tsx.md#output-0e4f75bd1f77)
- [crates/hick-merge/src/nway.rs](../../../mechanisms/crates/hick-merge/src/nway.rs.md#output-fe8bfa541f0f)
- [crates/hickory-cli/src/serve/merged.rs](../../../mechanisms/crates/hickory-cli/src/serve/merged.rs.md#output-772c9c304ee4)

### Tests and fixtures

- [apps/web/src/views/MergedView.test.tsx](contract.md#output-4cbd2ec09430)
- [crates/hickory-cli/tests/merged_view.rs](../../../mechanisms/crates/hickory-cli/tests/merged_view.rs.md#output-87756f3e699e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
