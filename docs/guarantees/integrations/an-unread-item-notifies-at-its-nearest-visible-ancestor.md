# An Unread Item Notifies At Its Nearest Visible Ancestor

Given an unread provider event on a ticket, issue, review, check, or comment,
when its own node is hidden by collapsed ancestors, then its count appears on
the nearest visible ancestor; when expansion makes its node visible, the badge
moves to that node; and expansion alone does not mark the event read.

---

Last LLM verification:

- Date: 2026-09-14
- Reviewer: Codex (GPT-5)
- Result: verified for GitHub issue and PR notifications; other providers remain pending
- Evidence: `apps/web/src/lib/workspaceTree.ts` defines the provider-neutral
  node/capability/freshness contract and `projectUnread`, which places direct
  unread events on the nearest visible ancestor without mutating them.
  `serve/github.rs` reads GitHub's notification thread as the unread record;
  `GithubTreeNodes.tsx` and `FolderTreePane.tsx` draw its badge on a visible
  issue/PR or aggregate it on a collapsed associated folder. Expansion does
  not acknowledge it; the explicit Mark notification read action does.
- Test coverage: `apps/web/src/lib/workspaceTree.test.ts` covers hidden items,
  several providers aggregated on an ancestor, successive expansion levels,
  expansion independent of unread state, and orphan/cycle refusal;
  `GithubTreeNodes.test.tsx` covers folder aggregation and Rust fixtures retain
  notification thread identity.
