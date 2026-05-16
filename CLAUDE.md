# Pre-commit

Before committing, always run the full test suite:

```bash
cargo test --workspace -- --test-threads=1
```

## XML Entity Escaping in Pipeline Writes

**hick's parser does NOT unescape XML entity references in text content.**
`&lt;` in a `.hick` source file will appear as `&lt;` in the pipeline output —
not `<`. This is different from standard XML parsers.

Consequence: code or text content containing `<`, `>`, or `&` (Rust generics,
C++ templates, HTML, boolean `&&`, shell redirects, etc.) will be corrupted if
passed through `pipeline_write`, `pipeline_new_file`, or `append_to_hick_source`
without special handling.

**Safe options:**
1. Wrap content in a CDATA section before inserting into the `.hick` source:
   `<![CDATA[pub fn foo<T>() -> Result<T>]]>` — CDATA content is never escaped
   and hick preserves it literally.
2. After any pipeline write, run `hick run` and grep the output files for XML
   entities (`&lt;`, `&gt;`, `&amp;`) as a smoke check.

**Anti-pattern to avoid:**
```python
# This will corrupt Rust/C++/HTML content:
hick.pipeline_write("src/lib.rs", rust_source_with_generics)
# The '<' in 'Result<()>' becomes '&lt;' in the output file.
```

<!-- BACKLOG.MD MCP GUIDELINES START -->

<CRITICAL_INSTRUCTION>

## BACKLOG WORKFLOW INSTRUCTIONS

This project uses Backlog.md MCP for all task and project management activities.

**CRITICAL GUIDANCE**

- If your client supports MCP resources, read `backlog://workflow/overview` to understand when and how to use Backlog for this project.
- If your client only supports tools or the above request fails, call `backlog.get_backlog_instructions()` to load the tool-oriented overview. Use the `instruction` selector when you need `task-creation`, `task-execution`, or `task-finalization`.

- **First time working here?** Read the overview resource IMMEDIATELY to learn the workflow
- **Already familiar?** You should have the overview cached ("## Backlog.md Overview (MCP)")
- **When to read it**: BEFORE creating tasks, or when you're unsure whether to track work

These guides cover:
- Decision framework for when to create tasks
- Search-first workflow to avoid duplicates
- Links to detailed guides for task creation, execution, and finalization
- MCP tools reference

You MUST read the overview resource to understand the complete workflow. The information is NOT summarized here.

</CRITICAL_INSTRUCTION>

<!-- BACKLOG.MD MCP GUIDELINES END -->
