---
id: TASK-7
title: >-
  Bug: hick pipeline outputs XML entities (&lt; &gt; &amp;) into generated code
  files
status: Done
assignee: []
created_date: '2026-05-12 21:15'
updated_date: '2026-05-18 12:53'
labels: []
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When hick generates files containing Rust (or other code), XML entities are not unescaped before writing. Affects backend/src/lib.rs which contains literal &lt;()&gt; and &amp;'static str instead of <()> and &'static str, causing compilation failure. Root cause is in the file-output rendering path in crates/hick/src/lib.rs or crates/hick-handlers.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Root cause was misidentified in the task description. Hickory XML is not a standard XML parser — it only processes `<hick:...>` namespaced elements and passes all other content through literally, including bare `<`, `>`, and `&`. The entity refs in the generated files (`&lt;`, `&gt;`, `&amp;`) came from pre-escaped content already present in `blue-iron-cube/main.hick`, not from hick's file-output rendering path.

Fix: replaced all `&lt;`, `&gt;`, `&amp;` in `blue-iron-cube/main.hick` with bare characters. Regenerated backend via `hick run`, verified no entity refs in output, and confirmed `blue-iron-cube` compiles clean. CLAUDE.md updated to document hickory XML's passthrough behavior and the generated-file policy.
<!-- SECTION:FINAL_SUMMARY:END -->
