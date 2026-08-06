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
