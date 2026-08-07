
---
slug: task-rename
title: Rename a task
status: todo
---


`todo rename <id> "<new title>"` changes a task's title, leaving its id and
state untouched. This is the requirement the opaque-id decision exists for:
anything referencing a task must survive a retitle. An unknown id exits
nonzero, like the other commands that take one.

