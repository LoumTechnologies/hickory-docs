
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

