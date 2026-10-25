<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# experiments/claude-code-corpus/corpus_scan.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Report Is Not a Reproducible Artifact](../../../../guarantees/verification/a-report-is-not-a-reproducible-artifact/contract.md)

Captured SHA-256: `b9ef67060a67940cc87332becc4b309be2bae8112502c1f9e31354667cb1468c`.

<book:entry path="experiments/claude-code-corpus/corpus_scan.py">


"""Aggregate the Claude Code session logs.

Reads ~/.claude/projects/**/*.jsonl and emits counts only.
Assembled by `hick run` from docs/analysis/claude-code-corpus.md.
"""
import glob
import json
import os
from collections import Counter

ROOT = os.path.expanduser("~/.claude/projects")
FILES = sorted(glob.glob(ROOT + "/**/*.jsonl", recursive=True))


tools = Counter()
rounds = []          # LLM turns that issued at least one tool call
git_sessions = 0
files_per_session = []

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


def pct(values, q):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * q))]

print(f"sessions: {len(FILES)}")
print(f"sessions using tools: {len(rounds)}")
print(f"tool calls: {sum(tools.values())}")
print(f"median rounds: {pct(rounds, 0.5)}")
print(f"p90 rounds: {pct(rounds, 0.9)}")
print(f"over budget: {sum(1 for r in rounds if r > 20)}")
print(f"median files: {pct(files_per_session, 0.5)}")
print(f"multi file sessions: {sum(1 for n in files_per_session if n > 1)}")
print(f"git sessions: {git_sessions}")
for name, n in tools.most_common(4):
    print(f"tool {name}: {n}")

</book:entry>

</book:doc>
