<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Unread Item Notifies At Its Nearest Visible Ancestor

For: Engineers connecting repository issues, reviews and locally installed tools.

Given an unread provider event on a ticket, issue, review, check, or comment,
when its own node is hidden by collapsed ancestors, then its count appears on
the nearest visible ancestor; when expansion makes its node visible, the badge
moves to that node; and expansion alone does not mark the event read.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-d824b0df0555"></a>
## `apps/web/src/lib/workspaceTree.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Unread Item Notifies At Its Nearest Visible Ancestor](contract.md).

<a id="f-d824b0df0555-node"></a>
### `node`

<storycode:copy id="f-d824b0df0555-node">
import { describe, expect, it } from "vitest";

import {
  projectUnread,
  workspaceNodeKey,
  type WorkspaceNode,
} from "./workspaceTree";

const node = (
  key: string,
  parent: string | null,
  unread = 0,
  provider = "filesystem",
): WorkspaceNode => ({
  key,
  parent,
  unread,
  kind: parent === null || provider === "filesystem" ? "folder" : "work-item",
  label: key,
  capabilities: [],
  source: { provider, id: key },
  freshness: { kind: "live" },
});

</storycode:copy>
<a id="f-d824b0df0555-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d824b0df0555-expression_statement">
// Guarantee: docs/guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor.md
describe("projecting unread events onto the visible tree", () => {
  const nodes = [
    node("root", null),
    node("folder", "root"),
    node("ticket", "folder", 2, "jira"),
    node("comment", "ticket", 1, "github"),
  ];

  it("puts hidden descendants on their nearest visible ancestor", () => {
    const badges = projectUnread(nodes, new Set());
    expect(badges.get("root")?.count).toBe(3);
    expect([...badges.get("root")!.sources]).toEqual([
      ["jira", 2],
      ["github", 1],
    ]);
  });

  it("moves a badge down as expansion makes nodes visible", () => {
    const folderVisible = projectUnread(nodes, new Set(["root"]));
    expect(folderVisible.has("root")).toBe(false);
    expect(folderVisible.get("folder")?.count).toBe(3);

    const ticketVisible = projectUnread(nodes, new Set(["root", "folder"]));
    expect(ticketVisible.get("ticket")?.count).toBe(3);

    const commentVisible = projectUnread(nodes, new Set(["root", "folder", "ticket"]));
    expect(commentVisible.get("ticket")?.count).toBe(2);
    expect(commentVisible.get("comment")?.count).toBe(1);
  });

  it("does not change the unread events when expansion changes", () => {
    const before = nodes.map((item) => item.unread);
    projectUnread(nodes, new Set(["root", "folder", "ticket"]));
    expect(nodes.map((item) => item.unread)).toEqual(before);
  });

  it("omits orphaned and cyclic nodes rather than misplacing their badges", () => {
    const broken = [
      node("root", null),
      node("orphan", "missing", 2, "jira"),
      node("a", "b", 1, "github"),
      node("b", "a"),
    ];
    expect(projectUnread(broken, new Set(["root", "a", "b"])).size).toBe(0);
  });
});

</storycode:copy>
<a id="f-d824b0df0555-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d824b0df0555-expression_statement-2">
describe("workspace node identity", () => {
  it("includes the provider and escapes delimiters", () => {
    expect(workspaceNodeKey({ provider: "github", id: "org/repo#12" })).toBe(
      "github:org%2Frepo%2312",
    );
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/workspaceTree.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/workspaceTree.test.ts"><storycode:paste select="#f-d824b0df0555-node"/><storycode:paste select="#f-d824b0df0555-expression_statement"/><storycode:paste select="#f-d824b0df0555-expression_statement-2"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/workspaceTree.ts](../../../mechanisms/apps/web/src/lib/workspaceTree.ts.md#output-59ed104225da)
- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/shell/GithubTreeNodes.tsx](../../../mechanisms/apps/web/src/shell/GithubTreeNodes.tsx.md#output-8c9096f3895b)
- [crates/hickory-cli/src/serve/github.rs](../../../mechanisms/crates/hickory-cli/src/serve/github.rs.md#output-1de81b976483)

### Tests and fixtures

- [apps/web/src/lib/workspaceTree.test.ts](contract.md#output-d824b0df0555)
- [apps/web/src/shell/GithubTreeNodes.test.tsx](../a-review-belongs-to-the-worktree-for-its-head/contract.md#output-7956bbacfb50)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
