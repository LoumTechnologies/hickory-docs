<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/claude-code-corpus/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `395843565f975a9ddf4a78f89f8c33b361cbce7b6eca77ebbee17596268859d4`.

<book:entry path="experiments/claude-code-corpus/README.md">
# Claude Code corpus analysis

Measures the Claude Code session logs on **this machine** (`~/.claude/projects`)
to size the agent harness against real work: how many LLM rounds a session
actually needs, how many files it touches, how often it uses git.

It lives in `experiments/` and not in `docs/` because its input is one
person's machine. CI has no such logs, so the assertions would be measuring an
empty set — and a document that asserts nothing is worse than no document.

```sh
hick run experiments/claude-code-corpus/claude-code-corpus.md
```

The woven report is marked `volatile="true"`: the corpus grows every time the
CLI runs, so "do these bytes reproduce" has no useful answer. The pinned
`hick:expect` assertions do — they check the shape of the conclusion, which is
what the harness design actually rests on.
</book:entry>

</book:doc>
