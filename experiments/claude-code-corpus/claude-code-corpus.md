
# What the agent harness is actually up against

Every number below is measured from the Claude Code session logs on this
machine by the cells in this document. Nothing is pasted in by hand, and
`hick check` fails if re-running produces different numbers than the ones
committed — which is the point: this is a claim about a corpus that grows every
day, and a stale claim about it should be loud rather than quiet.

Only aggregates are read. No prompt text, file content, or transcript leaves
the analysis.





## The sample

The corpus is every session log the CLI has written on this machine, across
every project directory.



## The three findings that bound the harness

**Turn budget.** A hick turn is one LLM call, so the comparable measure is
LLM rounds that issued tools — not raw tool calls. The median session needs
several times the default budget of 20.

**Multi-file reach.** The agent edits every file its primary document tangles.
The question is how often real work stays inside one document's outputs, and
the file counts below are the answer.

**Git.** The agent's workspace is a copy with `.git` skipped, so anything the
git count covers is out of reach today.




### `corpus_scan.py`

```python


"""Aggregate the Claude Code session logs.

Reads ~/.claude/projects/**/*.jsonl and emits counts only.
Assembled by `hick run` from docs/analysis/claude-code-corpus.hick.
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

```


## Measured



$ python3 project/corpus_scan.py
sessions: 510
sessions using tools: 411
tool calls: 38534
median rounds: 53
p90 rounds: 180
over budget: 296
median files: 11
multi file sessions: 325
git sessions: 322
tool Bash: 20751
tool Edit: 6895
tool Read: 4675
tool Write: 1850



The numbers above are the live corpus, and they move every time the CLI runs —
including the run that produced this page. The document root is therefore
marked `volatile="true"`: the woven report is a *report*, not a reproducible
artifact, so `hick check` does not diff it. Asking "do these bytes
reproduce" of a corpus that grows by the hour has one answer, and a check that
always fails is a check people learn to ignore.

`corpus_scan.py` is NOT volatile. It is assembled from the fragments above and
must reproduce exactly, so editing a fragment without re-running the document
still fails the check.

What remains verified is the shape of the conclusion. The assertions below are
the claims the harness design rests on, and each fails loudly if it stops being
true — which is the check that actually matters here.

$ python3 - <<'PY'
  import subprocess
  out = subprocess.run(
      ["python3", "project/corpus_scan.py"], capture_output=True, text=True
  ).stdout
  m = dict(
      line.rsplit(": ", 1) for line in out.strip().splitlines() if ": " in line
  )
  median = int(m["median rounds"])
  over = int(m["over budget"])
  sessions = int(m["sessions using tools"])
  multi = int(m["multi file sessions"])
  print("median rounds exceed the default budget:", median > 20)
  print("majority of sessions exceed it:", over > sessions / 2)
  print("most sessions touch several files:", multi > sessions / 2)
  PY
median rounds exceed the default budget: True
majority of sessions exceed it: True
most sessions touch several files: True


