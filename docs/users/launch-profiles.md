# Run an application while reading its library

For developers debugging generated source or ordinary workspace files.

Press Debug beside the source you are reading. Hickory discovers application
entry points in the compatible project. If there is one debuggable application,
it starts automatically. If several targets are available, choose one in the
floating Launch profiles picker. The ellipsis beside Debug always opens that
picker. Its Run buttons also offer package scripts and discovered Python tests.
Nothing adds rows to the editor.

For example, while reading `library.py`, Debug can start `app.py`, which imports
that library. Breakpoints stay attached to the exact library bytes you see.
Hickory chooses the latest whole project containing those bytes, including
completed atomic scopes. An older library can therefore select an earlier
compatible project. The profile chooses the entry point within that project.

The picker remembers your chosen target for that source during this app session.
If that target disappears, it asks you to choose again. Discovery is conservative:
it recognizes common application syntax, npm scripts, and top-level Python
`test_` functions. It does not prove that an application will call your function.
A module containing only function definitions is not automatically an application.

## Configure a target

Put `.hick-launch.json` at the project root:

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "greeting",
      "name": "Greet Ada",
      "program": "app.py",
      "args": ["Ada Lovelace"],
      "env": {"GREETING": "Welcome"},
      "cwd": ".",
      "default": true
    }
  ]
}
```

For a declared literate project, construct this file with an ordinary `hick:file`
in the document. It must belong to the selected project tree; an unrelated
workspace configuration is not borrowed into historical or captured projects.
For ordinary workspace source, edit the file directly at the workspace root.

Press the ellipsis to see the named target, then Debug or Run. A default profile
lets the ordinary Debug button select that target immediately. Only one profile
may be the default. Reusing a discovered profile's ID overrides its definition.
Malformed configuration is shown as an error before launching.

`args` and `build` use argument arrays: each string is one argument. Shell
expansion does not happen inside a string. To run a command, use `command`
instead of `program`:

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "tests",
      "name": "Python tests",
      "command": ["python3", "-m", "pytest"],
      "args": ["-q"]
    }
  ]
}
```

Command profiles, discovered npm scripts and Python tests currently support Run.
Debug requires a supported source `program` and its debug adapter.

Optional `build` runs first and stops the launch if it fails. Optional `artifact`
names its executable output, relative to the project root; for a C# program it
names the built DLL. `cwd` is relative to that same root. `env` applies to the
explicit build command and launched program. Existing automatic native build
recipes keep their own build environment; use an explicit build and artifact
when the build needs profile-specific variables.

Captured projects run in scratch directories. Ordinary workspace files run in
place. A changed source or project between choosing and launching is refused;
open the picker again after saving. Function invocation and a persistent REPL
remain separate planned features.
