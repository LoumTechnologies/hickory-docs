---
id: TASK-7
title: >-
  Bug: hick pipeline outputs XML entities (&lt; &gt; &amp;) into generated code
  files
status: To Do
assignee: []
created_date: '2026-05-12 21:15'
labels: []
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When hick generates files containing Rust (or other code), XML entities are not unescaped before writing. Affects backend/src/lib.rs which contains literal &lt;()&gt; and &amp;'static str instead of <()> and &'static str, causing compilation failure. Root cause is in the file-output rendering path in crates/hick/src/lib.rs or crates/hick-handlers.
<!-- SECTION:DESCRIPTION:END -->
