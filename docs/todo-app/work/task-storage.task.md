---
slug: task-storage
title: Persist tasks to disk
status: todo
---


Tasks persist to `tasks.json` in the working directory, created on first
write. A missing file reads as an empty list, not an error — the first run of
a fresh install must behave like every later run.

