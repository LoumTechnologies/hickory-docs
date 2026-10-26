<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Review Belongs To The Worktree For Its Head

For: Engineers connecting repository issues, reviews and locally installed tools.

Given open GitHub pull requests or GitLab merge requests and local clones or
worktrees, when the workspace tree is shown, then a review appears beneath the
clone or worktree whose normalized repository and checked-out branch match its
current head; expanding it exposes its current state, reviews, comments, checks,
bounded logs, provider mergeability, and honestly labeled current-head conflict
analysis.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/a-review-belongs-to-the-worktree-for-its-head.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-7956bbacfb50"></a>
## `apps/web/src/shell/GithubTreeNodes.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is An Editor Buffer](../../authoring/the-workspace-tree-edits-like-an-editor/contract.md), [A GitHub Issue Belongs To Its Associated Folder](../a-github-issue-belongs-to-its-associated-folder/contract.md), [A Review Belongs To The Worktree For Its Head](contract.md), [An Unread Item Notifies At Its Nearest Visible Ancestor](../an-unread-item-notifies-at-its-nearest-visible-ancestor/contract.md).

<a id="f-7956bbacfb50-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7956bbacfb50-expression_statement">
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";

import { installMockHandler } from "../api/client";
import type { GithubPullRequest, GithubWorkspace } from "../api/github";
import { GithubIssueRows, GithubReviewRows, githubUnreadUnder, parseGithubIssueReference } from "./GithubTreeNodes";

afterEach(cleanup);

</storycode:copy>
<a id="f-7956bbacfb50-PR"></a>
### `PR`

<storycode:copy id="f-7956bbacfb50-PR">
const PR: GithubPullRequest = {
  repository: "acme/widget", number: 12, title: "Retry requests", state: "OPEN",
  draft: false, url: "https://github.com/acme/widget/pull/12", author: "ada",
  review_decision: "REVIEW_REQUIRED", mergeable: "MERGEABLE", merge_state: "CLEAN",
  head: "retry", head_sha: "abc", base: "main", updated_at: "2026-09-14T00:00:00Z",
  checks: [], unread: 1, notification_thread: "991",
};

</storycode:copy>
<a id="f-7956bbacfb50-WORKSPACE"></a>
### `WORKSPACE`

<storycode:copy id="f-7956bbacfb50-WORKSPACE">
const WORKSPACE: GithubWorkspace = {
  status: "available", repository: "acme/widget", branch: "retry",
  reviews: [PR],
  issues: [{
    repository: "acme/widget", number: 7, folder: "docs", title: "Clarify retries",
    state: "OPEN", freshness: "live", labels: ["docs"],
  }],
};

</storycode:copy>
<a id="f-7956bbacfb50-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7956bbacfb50-expression_statement-2">
// docs/guarantees/integrations/a-review-belongs-to-the-worktree-for-its-head.md
describe("GitHub workspace-tree nodes", () => {
  it("accepts a repository reference, URL, or local issue number", () => {
    expect(parseGithubIssueReference("acme/widget#7")).toEqual({ repository: "acme/widget", number: 7 });
    expect(parseGithubIssueReference("https://github.com/acme/widget/issues/8/")).toEqual({ repository: "acme/widget", number: 8 });
    expect(parseGithubIssueReference("#9", "acme/widget")).toEqual({ repository: "acme/widget", number: 9 });
    expect(parseGithubIssueReference("#9")).toBeNull();
  });

  it("aggregates unread issues only beneath the collapsed folder", () => {
    const issues = [
      { ...WORKSPACE.issues[0], folder: "docs", unread: 1 },
      { ...WORKSPACE.issues[0], number: 8, folder: "docs/api", unread: 2 },
      { ...WORKSPACE.issues[0], number: 9, folder: "src", unread: 4 },
    ];
    expect(githubUnreadUnder(issues, "docs")).toBe(3);
    expect(githubUnreadUnder(issues, "src")).toBe(4);
  });
  it("lazily opens a PR through comments, reviews, checks, and a bounded log", async () => {
    const marked = vi.fn();
    installMockHandler(async (method, path) => {
      if (method === "GET" && path === "/api/workspace/github/pr/12") return {
        ...PR, body: "The PR body", comments: [{ id: "c1", author: { login: "lin" }, body: "Looks close" }],
        reviews: [{ id: "r1", author: { login: "sam" }, body: "Ship it", state: "APPROVED" }],
        checks: [{ name: "unit", workflow: "CI", status: "COMPLETED", conclusion: "FAILURE", run: 2, job: 3 }],
        current_head_conflicts: [{ number: 13, title: "Replace transport", url: "https://github.com/acme/widget/pull/13", author: "lin", commit_authors: ["lin", "sam"], head: "transport", head_sha: "def", status: "conflicting", claim: "these current head commits conflict if merged together" }],
      };
      if (method === "GET" && path === "/api/workspace/github/check-log?run=2&job=3") return { text: "assertion failed", truncated: false };
      if (method === "POST" && path === "/api/workspace/github/notification/991") { marked(); return { ok: true }; }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<ul role="tree"><GithubReviewRows workspace={WORKSPACE} onChanged={() => {}} /></ul>);
    fireEvent.click(screen.getByRole("button", { name: /PR.*#12/ }));
    expect(await screen.findByDisplayValue("The PR body")).toBeTruthy();
    expect(screen.getByText(/sam · approved/)).toBeTruthy();
    expect(screen.getByText(/these current head commits conflict/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Mark notification read" }));
    await vi.waitFor(() => expect(marked).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: /CI: unit/ }));
    expect(await screen.findByText("assertion failed")).toBeTruthy();
  });

  it("edits issue titles with the shared rename editor and writes body/comments semantically", async () => {
    const calls: Array<[string, string, unknown]> = [];
    installMockHandler(async (method, path, body) => {
      calls.push([method, path, body]);
      if (method === "GET" && path.startsWith("/api/workspace/github/issue/7")) return {
        ...WORKSPACE.issues[0], body: "Old body", author: "ada", comments: [],
      };
      if (method === "POST") return { ok: true };
      throw new Error(`unexpected ${method} ${path}`);
    });
    const changed = vi.fn();
    render(<ul role="tree"><GithubIssueRows workspace={WORKSPACE} folder="docs" parentKey="filesystem:docs" onChanged={changed} /></ul>);
    const row = screen.getByRole("button", { name: /Issue.*#7/ });
    fireEvent.keyDown(row, { key: "F2" });
    const editor = EditorView.findFromDOM(document.querySelector(".tree-rename .cm-editor")!);
    if (!editor) throw new Error("rename editor did not mount");
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "Explain retry policy" } });
    fireEvent.click(screen.getByRole("button", { name: "Rename" }));
    await screen.findByText(/renamed/);
    expect(calls).toContainEqual(["POST", "/api/workspace/github/edit", {
      kind: "issue", repository: "acme/widget", number: 7, field: "title", value: "Explain retry policy",
    }]);

    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    await screen.findByDisplayValue("Old body");
    fireEvent.change(screen.getByLabelText("Body"), { target: { value: "New body" } });
    fireEvent.click(screen.getByRole("button", { name: "Save body" }));
    await screen.findByText("Body saved.");
    fireEvent.change(screen.getByLabelText("New comment"), { target: { value: "I checked this." } });
    fireEvent.click(screen.getByRole("button", { name: "Add comment" }));
    await screen.findByText("Comment added.");
    expect(calls.some(([method, path, body]) => method === "POST" && path.endsWith("/comment") && (body as { body: string }).body === "I checked this.")).toBe(true);
  });
});

</storycode:copy>
<a id="f-7956bbacfb50-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7956bbacfb50-expression_statement-3">
it("shares lazy PR/check disclosure, notification and link controls and retires closed children", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const calls: string[] = [];
  const opened = vi.spyOn(window, "open").mockReturnValue(null);
  installMockHandler(async (method, path) => {
    calls.push(`${method} ${path}`);
    if (path === "/api/workspace/github/pr/12") return { ...PR, body: "Explicit PR body", comments: [],
      checks: [{ name: "unit", workflow: "CI", status: "COMPLETED", run: 2, job: 3 }] };
    if (path.startsWith("/api/workspace/github/check-log")) return { text: "Explicit log", truncated: false };
    if (path === "/api/workspace/github/notification/991") return { ok: true };
    throw Error(path);
  });
  const view = render(<UiActionContext.Provider value={registry}><ul><GithubReviewRows workspace={WORKSPACE} onChanged={() => {}} /></ul></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  expect(actions().some(a => a.label.includes("GitHub body draft"))).toBe(false);
  expect(calls).toEqual([]);
  await run("Set GitHub PR acme/widget#12 visibility", { open: true });
  await screen.findByDisplayValue("Explicit PR body");
  expect(calls).toEqual(["GET /api/workspace/github/pr/12"]);
  const bodyHandle = actions().find(a => a.label === "Set GitHub body draft pr:acme/widget#12")!.handle;
  await run("Set GitHub check acme/widget#12/CI/unit visibility", { open: true });
  await screen.findByText("Explicit log");
  expect(await run("Mark GitHub notification 991 read")).toMatchObject({ status: "result", outcome: { value: { response: { ok: true } } } });
  expect(await run("Open GitHub link: Open on GitHub")).toMatchObject({ status: "result", outcome: { value: { navigation_requested: true, tab_handle_returned: false } } });
  fireEvent.click(screen.getByRole("link", { name: "Open on GitHub" }));
  expect(opened).toHaveBeenCalledTimes(2);
  await run("Set GitHub PR acme/widget#12 visibility", { open: false });
  expect(actions().some(a => a.label.includes("GitHub body draft") || a.label.includes("GitHub check"))).toBe(false);
  expect(await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, { handle: bodyHandle, arguments: { text: "stale" }, request_id: "closed" })).toMatchObject({ status: "unavailable" });
  view.unmount(); opened.mockRestore();
});

</storycode:copy>
<a id="f-7956bbacfb50-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-7956bbacfb50-expression_statement-4">
it("refuses late issue reads and preserves a dirty body while provider identity refreshes", async () => {
  const { act } = await import("@testing-library/react");
  const responses: ((value: unknown) => void)[] = [];
  installMockHandler(async () => new Promise(done => responses.push(done)));
  const mount = (updated_at: string) => <ul><GithubIssueRows workspace={{ ...WORKSPACE,
    issues: [{ ...WORKSPACE.issues[0], updated_at }] }} folder="docs" onChanged={() => {}} /></ul>;
  const view = render(mount("one"));
  fireEvent.click(screen.getByRole("button", { name: /Issue.*#7/ }));
  await vi.waitFor(() => expect(responses).toHaveLength(1));
  view.rerender(mount("two"));
  await vi.waitFor(() => expect(responses).toHaveLength(2));
  await act(async () => { responses[0]({ ...WORKSPACE.issues[0], body: "Late old body", comments: [] }); });
  expect(screen.queryByDisplayValue("Late old body")).toBeNull();
  await act(async () => { responses[1]({ ...WORKSPACE.issues[0], body: "Current body", comments: [] }); });
  fireEvent.change(screen.getByLabelText("Body"), { target: { value: "Retained local draft" } });
  view.rerender(mount("three"));
  expect((screen.getByLabelText("Body") as HTMLTextAreaElement).value).toBe("Retained local draft");
  expect((screen.getByRole("button", { name: "Save body" }) as HTMLButtonElement).disabled).toBe(true);
  await vi.waitFor(() => expect(responses).toHaveLength(3));
  await act(async () => { responses[2]({ ...WORKSPACE.issues[0], body: "New provider body", comments: [] }); });
  expect((screen.getByLabelText("Body") as HTMLTextAreaElement).value).toBe("Retained local draft");
  expect(screen.getByRole("button", { name: "Reset body draft to current" })).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/shell/GithubTreeNodes.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GithubTreeNodes.test.tsx"><storycode:paste select="#f-7956bbacfb50-expression_statement"/><storycode:paste select="#f-7956bbacfb50-PR"/><storycode:paste select="#f-7956bbacfb50-WORKSPACE"/><storycode:paste select="#f-7956bbacfb50-expression_statement-2"/><storycode:paste select="#f-7956bbacfb50-expression_statement-3"/><storycode:paste select="#f-7956bbacfb50-expression_statement-4"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/shell/GithubTreeNodes.tsx](../../../mechanisms/apps/web/src/shell/GithubTreeNodes.tsx.md#output-8c9096f3895b)
- [crates/hickory-cli/src/serve/github.rs](../../../mechanisms/crates/hickory-cli/src/serve/github.rs.md#output-1de81b976483)

### Tests and fixtures

- [apps/web/src/shell/GithubTreeNodes.test.tsx](contract.md#output-7956bbacfb50)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
