
---
slug: task-transitions
title: Complete and drop tasks
status: todo
---


`todo done <id>` moves a task to done. `todo drop <id>` moves it to dropped.
Both are idempotent: re-running on a task already in that state succeeds and
changes nothing, so a retried command is never an error.

