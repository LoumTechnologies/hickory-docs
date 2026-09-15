# A Review Belongs To The Worktree For Its Head

Given open GitHub pull requests or GitLab merge requests and local clones or
worktrees, when the workspace tree is shown, then a review appears beneath the
clone or worktree whose normalized repository and checked-out branch match its
current head; expanding it exposes its current state, reviews, comments, checks,
bounded logs, provider mergeability, and honestly labeled current-head conflict
analysis.

The conflict claim is deliberately current and narrow: candidates share the
base and an overlapping changed path, their current provider heads are fetched,
and `git merge-tree` compares those commits without touching the working tree.
The UI says “these current head commits conflict if merged together,” never
that future edits are predicted.

---

Last LLM verification:

- Date: 2026-09-14
- Reviewer: Codex (GPT-5)
- Result: verified for GitHub; GitLab remains unimplemented
- Evidence: `crates/hickory-cli/src/serve/github.rs` normalizes the remote,
  selects open PRs by checked-out head branch, lazily reads details, bounds
  Actions logs at 512 KiB, and compares fetched current heads;
  `apps/web/src/shell/GithubTreeNodes.tsx` renders and edits the tree.
- Test coverage: Rust unit fixtures cover remote normalization, Actions ids,
  notification identity, overlapping-path bounds, and author identity;
  `GithubTreeNodes.test.tsx` covers lazy details, reviews, checks, logs, and the
  honest current-head conflict sentence. A live GitHub response was also
  exercised against the authenticated repository on 2026-09-14 (no open PR
  exists on `master`). GitLab fixtures remain required with that adapter.
