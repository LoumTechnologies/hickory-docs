<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/todo-app/work/task-transitions.task.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `fe1d575263ef0f157109419651e5ae3c9e67d25277afdc7b8f9598d0d7383f1a`.

<book:entry path="docs/todo-app/work/task-transitions.task.md">
---
slug: task-transitions
title: Complete and drop tasks
status: todo
---


`todo done <id>` moves a task to done. `todo drop <id>` moves it to dropped.
`todo block <id>` moves it to blocked. All are idempotent: re-running on a
task already in that state succeeds and changes nothing, so a retried
command is never an error.

</book:entry>

</book:doc>
