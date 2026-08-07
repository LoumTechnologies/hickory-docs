
# Todo app — implementation



The code lives here, in prose order, and is tangled into `todo.py`. The cells
at the bottom run the real CLI and pin its output, so this document is both
the source and the acceptance test: `hickory check` fails if the code stops
satisfying the requirements it was written from.

## Requirements being implemented





## Storage



## Commands



## Entry point




### `todo.py`

```python


"""A todo list. Tangled from docs/todo-app/implementation.hick — edit the
document, not this file."""
import json
import os
import secrets
import sys

PATH = os.environ.get("TODO_FILE", "tasks.json")
STATES = ("open", "blocked", "done", "dropped")


def load():
    """A missing file reads as an empty list — first run behaves like any."""
    if not os.path.exists(PATH):
        return []
    with open(PATH) as fh:
        return json.load(fh)


def save(tasks):
    with open(PATH, "w") as fh:
        json.dump(tasks, fh, indent=2)



def add(title):
    """Ids are opaque and assigned at creation, never derived from the title."""
    tasks = load()
    task = {"id": secrets.token_hex(3), "title": title, "state": "open"}
    tasks.append(task)
    save(tasks)
    return task["id"]


def transition(task_id, state):
    """Idempotent: re-running on a task already in that state changes nothing,
    so a retried command is never an error."""
    tasks = load()
    for task in tasks:
        if task["id"] == task_id:
            task["state"] = state
            save(tasks)
            return True
    return False


def render(tasks):
    return "".join(f"{t['id']} {t['state']} {t['title']}\n" for t in tasks)


def rename(task_id, new_title):
    """Renaming leaves id and state untouched, so anything referencing this
    task survives a retitle."""
    tasks = load()
    for task in tasks:
        if task["id"] == task_id:
            task["title"] = new_title
            save(tasks)
            return True
    return False



def main(argv):
    if not argv:
        print("usage: todo add|list|done|drop|block", file=sys.stderr)
        return 2
    cmd, rest = argv[0], argv[1:]
    if cmd == "add":
        print(add(rest[0]))
        return 0
    if cmd == "list":
        tasks = load()
        if len(rest) == 2 and rest[0] == "--state":
            tasks = [t for t in tasks if t["state"] == rest[1]]
        sys.stdout.write(render(tasks))
        return 0
    if cmd in ("done", "drop", "block"):
        state = {"done": "done", "drop": "dropped", "block": "blocked"}[cmd]
        if not transition(rest[0], state):
            print(f"no such task: {rest[0]}", file=sys.stderr)
            return 1
        return 0
    if cmd == "rename":
        if not rename(rest[0], rest[1]):
            print(f"no such task: {rest[0]}", file=sys.stderr)
            return 1
        return 0
    print(f"unknown command: {cmd}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

```


## Acceptance

These cells run the real CLI. Ids are opaque and random, so the tests set
titles and filter on state rather than pinning ids — pinning a random id
would make the document fail on every run for no reason.



$ cd project && rm -f /tmp/todo-acceptance.json && export TODO_FILE=/tmp/todo-acceptance.json
  python3 todo.py list
  echo "--- empty file reads as empty list, no error: $?"
--- empty file reads as empty list, no error: 0



$ cd project && export TODO_FILE=/tmp/todo-acceptance.json
  python3 todo.py add "write the spec" > /dev/null
  python3 todo.py add "ship it" > /dev/null
  python3 todo.py add "abandon this one" > /dev/null
  python3 todo.py list | awk '{$1=""; print substr($0,2)}'
open write the spec
open ship it
open abandon this one



$ cd project && export TODO_FILE=/tmp/todo-acceptance.json
  first=$(python3 todo.py list | head -1 | cut -d' ' -f1)
  last=$(python3 todo.py list | tail -1 | cut -d' ' -f1)
  python3 todo.py done "$first"
  python3 todo.py done "$first"   # idempotent: a retry is never an error
  python3 todo.py drop "$last"
  python3 todo.py list | awk '{$1=""; print substr($0,2)}'
done write the spec
open ship it
dropped abandon this one



$ cd project && export TODO_FILE=/tmp/todo-acceptance.json
  python3 todo.py list --state open | awk '{$1=""; print substr($0,2)}'
  echo "--- unknown id exits nonzero:"
  python3 todo.py done nosuchid 2>/dev/null || echo "yes"
open ship it
--- unknown id exits nonzero:
yes



$ cd project && export TODO_FILE=/tmp/todo-acceptance.json
  id=$(python3 todo.py list --state open | head -1 | cut -d' ' -f1)
  python3 todo.py rename "$id" "ship it today"
  python3 todo.py list --state open | awk '{$1=""; print substr($0,2)}'
  echo "--- id survived the retitle:"
  python3 todo.py list --state open | head -1 | cut -d' ' -f1 | grep -qx "$id" && echo "yes"
  echo "--- unknown id exits nonzero:"
  python3 todo.py rename nosuchid "x" 2>/dev/null || echo "yes"
open ship it today
--- id survived the retitle:
yes
--- unknown id exits nonzero:
yes


$ cd project && rm -f /tmp/todo-acceptance.json && export TODO_FILE=/tmp/todo-acceptance.json
  python3 todo.py list
  echo "--- empty file reads as empty list, no error: $?"
--- empty file reads as empty list, no error: 0


