# A GitHub Issue Belongs To Its Associated Folder

Given a person associates `owner/repository#number` (or its GitHub URL) from a
folder's context menu, when the workspace tree is read here or from another
clone, then the issue is a child of that folder. The committed-capable
`.hick-workspace.json` record contains only provider identity, issue number,
and root-relative folder—never a token or copied provider response.

Expanding the issue asks GitHub for its body, labels, assignees, and comments.
Its title and body can be edited and a comment can be appended. Each mutation
is one semantic GitHub operation and a refusal keeps GitHub's useful words.
Authentication remains in the person's `gh` credential store. A missing or
signed-out CLI leaves files usable and marks GitHub or the issue unavailable.

---

Last LLM verification:

- Date: 2026-09-14
- Reviewer: Codex
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/github.rs`,
  `apps/web/src/shell/GithubTreeNodes.tsx`, `FolderTreePane.tsx`, and
  `treeMenu.ts`
- Test coverage: Rust fixtures cover credentialless persistence and safe folder
  placement; `GithubTreeNodes.test.tsx` covers lazy read/title/body/comment
  mutations; `FolderTreePane.test.tsx` covers association and placement.
