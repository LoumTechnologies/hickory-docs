<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-handlers/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `fe0f9f41ddb29ac73fa730c7c0240fe4c99a0ca68a997829c560ec7dbb3c340c`.

<book:entry path="crates/hick-handlers/README.md">
# hick-handlers

Plugin-style tag dispatch system for the hick pipeline.

## Core abstractions

- `TagHandler` trait — `tag_name()`, `phase()`, and `process()` methods
- `TagRegistry` — name-keyed lookup table of `Box<dyn TagHandler>` instances
- `ProcessingContext` — shared mutable state: transcripts, indentation, recursive child access

## Processing phases

| Phase | Tags | Effect |
|---|---|---|
| `Declaration` | copy, cut, substitute, exclude | Side-effects only, no output |
| `Content` | exec, paste, val | Produce node output |

All built-in tag handlers live in the `handlers` submodule. `ExecShow` and
`render_transcript` control how container execution output is formatted in the
final generated files.
</book:entry>

</book:doc>
