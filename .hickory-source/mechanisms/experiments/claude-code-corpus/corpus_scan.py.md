<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain experiments/claude-code-corpus/corpus_scan.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5af8170a3224"></a>
## `experiments/claude-code-corpus/corpus_scan.py`

Related contracts: [A Report Is Not a Reproducible Artifact](../../../guarantees/verification/a-report-is-not-a-reproducible-artifact/contract.md).

<a id="f-5af8170a3224-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement">


"""Aggregate the Claude Code session logs.

Reads ~/.claude/projects/**/*.jsonl and emits counts only.
Assembled by `hick run` from docs/analysis/claude-code-corpus.md.
"""
</storycode:copy>
<a id="f-5af8170a3224-glob"></a>
### `glob`

<storycode:copy id="f-5af8170a3224-glob">
import glob
</storycode:copy>
<a id="f-5af8170a3224-json"></a>
### `json`

<storycode:copy id="f-5af8170a3224-json">
import json
</storycode:copy>
<a id="f-5af8170a3224-os"></a>
### `os`

<storycode:copy id="f-5af8170a3224-os">
import os
</storycode:copy>
<a id="f-5af8170a3224-Counter"></a>
### `Counter`

<storycode:copy id="f-5af8170a3224-Counter">
from collections import Counter

</storycode:copy>
<a id="f-5af8170a3224-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-2">
ROOT = os.path.expanduser("~/.claude/projects")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-3">
FILES = sorted(glob.glob(ROOT + "/**/*.jsonl", recursive=True))


</storycode:copy>
<a id="f-5af8170a3224-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-4">
tools = Counter()
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-5">
rounds = []          </storycode:copy>
<a id="f-5af8170a3224-expression_statement-6"></a>
### `expression_statement`

LLM turns that issued at least one tool call

<storycode:copy id="f-5af8170a3224-expression_statement-6">
# LLM turns that issued at least one tool call
git_sessions = 0
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-7">
files_per_session = []

</storycode:copy>
<a id="f-5af8170a3224-for_statement"></a>
### `for_statement`

<storycode:copy id="f-5af8170a3224-for_statement">
for path in FILES:
    n_rounds = 0
    touched = set()
    used_git = False
    with open(path, errors="replace") as fh:
        for line in fh:
            if not line.strip():
                continue
            try:
                rec = json.loads(line)
            except ValueError:
                continue
            if rec.get("type") != "assistant":
                continue
            blocks = rec.get("message", {}).get("content") or []
            calls = [b for b in blocks if isinstance(b, dict) and b.get("type") == "tool_use"]
            if calls:
                n_rounds += 1
            for b in calls:
                name = b.get("name", "?")
                tools[name] += 1
                arg = b.get("input", {}) or {}
                if name in ("Edit", "Write", "Read", "NotebookEdit") and arg.get("file_path"):
                    touched.add(arg["file_path"])
                if name == "Bash" and "git " in (arg.get("command") or ""):
                    used_git = True
    if n_rounds:
        rounds.append(n_rounds)
    if touched:
        files_per_session.append(len(touched))
    if used_git:
        git_sessions += 1


</storycode:copy>
<a id="f-5af8170a3224-pct"></a>
### `pct`

<storycode:copy id="f-5af8170a3224-pct">
def pct(values, q):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * q))]

</storycode:copy>
<a id="f-5af8170a3224-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-8">
print(f"sessions: {len(FILES)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-9">
print(f"sessions using tools: {len(rounds)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-10">
print(f"tool calls: {sum(tools.values())}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-11">
print(f"median rounds: {pct(rounds, 0.5)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-12">
print(f"p90 rounds: {pct(rounds, 0.9)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-13">
print(f"over budget: {sum(1 for r in rounds if r > 20)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-14">
print(f"median files: {pct(files_per_session, 0.5)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-15">
print(f"multi file sessions: {sum(1 for n in files_per_session if n > 1)}")
</storycode:copy>
<a id="f-5af8170a3224-expression_statement-16"></a>
### `expression_statement`

<storycode:copy id="f-5af8170a3224-expression_statement-16">
print(f"git sessions: {git_sessions}")
</storycode:copy>
<a id="f-5af8170a3224-for_statement-2"></a>
### `for_statement`

<storycode:copy id="f-5af8170a3224-for_statement-2">
for name, n in tools.most_common(4):
    print(f"tool {name}: {n}")

</storycode:copy>

### Assemble `experiments/claude-code-corpus/corpus_scan.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="experiments/claude-code-corpus/corpus_scan.py"><storycode:paste select="#f-5af8170a3224-expression_statement"/><storycode:paste select="#f-5af8170a3224-glob"/><storycode:paste select="#f-5af8170a3224-json"/><storycode:paste select="#f-5af8170a3224-os"/><storycode:paste select="#f-5af8170a3224-Counter"/><storycode:paste select="#f-5af8170a3224-expression_statement-2"/><storycode:paste select="#f-5af8170a3224-expression_statement-3"/><storycode:paste select="#f-5af8170a3224-expression_statement-4"/><storycode:paste select="#f-5af8170a3224-expression_statement-5"/><storycode:paste select="#f-5af8170a3224-expression_statement-6"/><storycode:paste select="#f-5af8170a3224-expression_statement-7"/><storycode:paste select="#f-5af8170a3224-for_statement"/><storycode:paste select="#f-5af8170a3224-pct"/><storycode:paste select="#f-5af8170a3224-expression_statement-8"/><storycode:paste select="#f-5af8170a3224-expression_statement-9"/><storycode:paste select="#f-5af8170a3224-expression_statement-10"/><storycode:paste select="#f-5af8170a3224-expression_statement-11"/><storycode:paste select="#f-5af8170a3224-expression_statement-12"/><storycode:paste select="#f-5af8170a3224-expression_statement-13"/><storycode:paste select="#f-5af8170a3224-expression_statement-14"/><storycode:paste select="#f-5af8170a3224-expression_statement-15"/><storycode:paste select="#f-5af8170a3224-expression_statement-16"/><storycode:paste select="#f-5af8170a3224-for_statement-2"/></storycode:file>

</storycode:doc>
