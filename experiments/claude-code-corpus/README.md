# Claude Code corpus analysis

Measures the Claude Code session logs on **this machine** (`~/.claude/projects`)
to size the agent harness against real work: how many LLM rounds a session
actually needs, how many files it touches, how often it uses git.

It lives in `experiments/` and not in `docs/` because its input is one
person's machine. CI has no such logs, so the assertions would be measuring an
empty set — and a document that asserts nothing is worse than no document.

```sh
hickory run experiments/claude-code-corpus/claude-code-corpus.hick
```

The woven report is marked `volatile="true"`: the corpus grows every time the
CLI runs, so "do these bytes reproduce" has no useful answer. The pinned
`hick:expect` assertions do — they check the shape of the conclusion, which is
what the harness design actually rests on.
