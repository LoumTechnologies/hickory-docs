<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/todo-app/work/task-storage.task.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `5e1f4b930d7356d132b649c90bdc1bf0c2fbf1c500b772bb42edf1728054a479`.

<book:entry path="docs/todo-app/work/task-storage.task.md">
---
slug: task-storage
title: Persist tasks to disk
status: todo
---


Tasks persist to `tasks.json` in the working directory, created on first
write. A missing file reads as an empty list, not an error — the first run of
a fresh install must behave like every later run.

</book:entry>

</book:doc>
