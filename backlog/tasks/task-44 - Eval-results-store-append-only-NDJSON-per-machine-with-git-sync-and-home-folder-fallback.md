---
id: TASK-44
title: >-
  Eval results store: append-only NDJSON per machine with git-sync and
  home-folder fallback
status: To Do
assignee: []
created_date: '2026-05-20 12:16'
labels:
  - eval
  - persistence
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design and implement the eval persistence layer. Each machine writes exclusively to its own file named by a stable machine ID (hostname + UUID written once to `~/.hick/machine-id`). Storage location: `.hick/eval/<machine-id>.ndjson` when inside a git repo, otherwise `~/.hick/eval/<machine-id>.ndjson`. Since each machine owns exactly one file, git merges are always conflict-free. An aggregating reader merges all machine files at query time. Records are append-only — pruning decisions are written as metadata records on top of existing entries, never deletions. On startup, `hick` runs `git pull` (if in a repo and a remote exists). After each eval session, it commits and pushes the machine file. A storage trait abstracts git vs. future server backends so the transport can be swapped without changing the record format.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Machine ID is stable across restarts and written to ~/.hick/machine-id on first run
- [ ] #2 Each eval result (query, config, response, scores, token cost, timestamp, machine-id) is written as one NDJSON line
- [ ] #3 Pruning decisions are appended as metadata records; no lines are ever deleted
- [ ] #4 Aggregating reader merges all <machine-id>.ndjson files and presents a unified view
- [ ] #5 git pull on startup and git commit+push after session complete when inside a repo with a remote
- [ ] #6 Falls back to ~/.hick/eval/ when not in a git repo, with no error
- [ ] #7 A `StorageBackend` trait exists with a git implementation; server impl is stubbed for future use
- [ ] #8 Two machines writing concurrently then merging via git produces no conflicts
<!-- AC:END -->
