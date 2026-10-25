<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/todo-app/work/task-rename.task.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `c6c59fe938db0bffe77efdf5fff04a478ed43650e620968e85f143577548a20b`.

<book:entry path="docs/todo-app/work/task-rename.task.md">
---
slug: task-rename
title: Rename a task
status: todo
---


`todo rename <id> "<new title>"` changes a task's title, leaving its id and
state untouched. This is the requirement the opaque-id decision exists for:
anything referencing a task must survive a retitle. An unknown id exits
nonzero, like the other commands that take one.

</book:entry>

</book:doc>
