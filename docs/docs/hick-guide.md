# The Hick Language

Hick is a document format for describing secure, containerised workflows.
A `.hick` file is XML that uses a single namespace to define containers,
capabilities, execution steps, and file outputs — all validated as a
directed acyclic graph before anything runs.

Namespace URI: `http://www.hickorydocs.com/1.0`

---

## 1. Document Structure

Every hick file is an XML document whose root element binds a prefix to
the hick namespace. The conventional prefix is `hick:`, but any prefix
works as long as the xmlns points to the right URI.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  
</hick:doc>
```

Only tags with the bound prefix are parsed as structured elements.
Everything else — including `&amp;`, `&lt;`, `&gt;` — is raw text, no
escaping required.

---

## 2. Containers and Capabilities

Containers declare isolated execution environments with explicit
capability grants. The security model is deny-by-default when a
`deny` rule is present.

```xml
<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-write="/output/*" />
</hick:container>
```

### What a container needs installed

A cell runs against the programs on the machine that runs it. `image=` is
recorded but ignored by the local and sandbox executors, so a document that
uses `duckdb` works where it was written and fails everywhere else with
`sh: 1: duckdb: not found` — a message from a shell, about a program, that
says nothing about the document that wanted it.

`<hick:needs>` is how a document says so itself:

```xml
<hick:container name="reporter" image="python:3.12">
  <hick:needs bin="duckdb" for="the queries in section 3" />
  <hick:needs bin="gnuplot" for="the chart" />
</hick:container>
```

Every declared program is looked for **before any cell runs**, and a
document missing one stops immediately, naming all of them at once with the
line each was declared on. Nothing is downloaded and no network is touched:
this is a declaration and a check, not a package manager. Installing them
is yours to do, on purpose — resolving tools for people means owning the
difference between what we installed and what your own tooling installs.

The check asks the executor, not the machine, because those differ. A
sandboxed cell has an empty `$HOME` with only the usual toolchain
directories bound back, so a program in an unusual place exists on your
machine and not in the cell; under Docker it is the image's contents that
matter and the host's not at all. "Can the thing that will run this cell
see it?" is the only question worth asking.

The `for=` text is optional and worth writing. "duckdb is not installed"
tells a reader what to install; "duckdb is not installed — the queries in
section 3 run against it" tells them whether they want to.

### Network rules

| Rule | Effect |
|------|--------|
| `<hick:allow network="host:port" />` | Permit traffic to host:port |
| `<hick:allow network="*:443" />` | Wildcard port or host |
| `<hick:deny network="*" />` | Deny all not explicitly allowed |

**What is enforced.** A container gets a network only when it carries at
least one `allow network=` rule; a container that says nothing about the
network has none. The rule's host and port are not enforced by the
sandbox — a container granted `github.com:443` can reach whatever its
network reaches — so `deny network="*"` documents intent alongside an
allowlist, and on its own means "no network", which is already the
default.

### File rules

| Rule | Effect |
|------|--------|
| `<hick:allow file-read="/path/*" />` | Permit reading under path |
| `<hick:allow file-write="/path/*" />` | Permit writing (implies read) |

### Secrets

Secrets are injected as environment variables, sourced from an external
secrets provider (age-encrypted files by default).

```xml
<hick:container name="deploy" image="alpine">
  <hick:secret name="PULUMI_TOKEN" from="pulumi-token" />
</hick:container>
```

---

## 3. Execution

The `exec` tag runs commands inside a container. Multiple execs in the
same container are sequenced automatically (container-state dependency).

```xml
<hick:exec container="reporter" image="alpine">
apk add curl
</hick:exec>

<hick:exec container="reporter">
curl --version
</hick:exec>
```

If the container hasn't been declared with a `container` tag, providing
`image` on the first exec creates it implicitly.

### Volumes

Shared volumes let containers exchange data. The first exec to mount a
volume is treated as the writer; subsequent mounters become readers,
creating a data-flow dependency.

```xml
<hick:volume name="shared-output" />

<hick:exec container="writer" mount="shared-output:/output">
echo "data" &gt; /output/result.txt
</hick:exec>

<hick:exec container="reader" mount="shared-output:/input">
cat /input/result.txt
</hick:exec>
```

A volume also connects a cell to the **working tree**, and this is the
attribute most people need and do not find:

| attribute | what it does |
|---|---|
| `input="dir"` | the volume starts as a copy of `dir` from your project |
| `output="dir"` | what the volume holds at the end is written back to `dir` |
| both | read the tree in, write the tree out |
| neither | scratch space, thrown away when the run ends |

**Without `input=`, a cell sees nothing of your project.** A cell's workdir
is an empty scratch directory — not the folder the document is in, and not
the files the document itself writes. That is the right default (a document
you were sent should not read your repository just by running), but it is
also the first thing that surprises anyone writing a **code generator**, which
is a cell whose whole job is to read your source:

```xml
<hick:volume name="domain" input="src/Domain" />
<hick:volume name="generated" output="src/Api/Generated" />

<hick:exec container="sdk" mount="domain:Domain,generated:out">
<hick:copy id="generate">
dotnet run --project tools/ApiGen -- Domain out
</hick:copy>
</hick:exec>
```

Keep each volume as narrow as the job. `input="src" output="src"` round-trips
the entire tree, which for a compiled language means the build's `obj/` comes
back with it. Gitignored files are not written back, but a volume scoped to
what the cell actually reads and writes is clearer than relying on that.

---

## 4. File Outputs

The `file` tag declares an output file. Its content is a mix of literal
text, exec placeholders, and paste references.

```xml
<hick:file path="report.md">
# Report

Build output:
<hick:exec container="builder">make build</hick:exec>

Version: <hick:paste select="#version" />
</hick:file>
```

Exec tags inside a file render the full container transcript — every
command prefixed with `$ ` followed by its output. In dry-run mode only
the commands appear; in live mode the real output follows each command.
Paste tags are resolved from matching copy/cut blocks.

### Owning what a scaffolder wrote

`dotnet new webapi` writes forty files nobody typed, and the interesting
work is changing four lines across three of them. Pasting the scaffold
into the document makes it a silent snapshot; leaving it out makes the
scaffold a step in a README. `hick ingest --from` is the third answer:
it reads a cell's **output volume** and writes those bytes into the
document as ordinary `hick:file` blocks under a `hick:ingested` element
that records the run.

```xml
<hick:volume name="project" output="." />

<hick:exec container="sdk" mount="project:/out">
<hick:copy id="scaffold">
cd /out &amp;&amp; dotnet new webapi -o .
</hick:copy>
</hick:exec>
```

Then, once:

```
hick ingest --from '#scaffold' app.hick
```

The document gains, inside that cell:

```xml
<hick:ingested from="#scaffold" sha256="9f2c…" at="2026-08-23"
               files="38" skipped="2">
<hick:file path="Program.cs">…every byte the scaffolder wrote…</hick:file>
…
</hick:ingested>
```

Four things about it are worth knowing before you run it:

- **The command must be wrapped in a `hick:copy`.** Otherwise forty file
  bodies sit as siblings of ambient command text and there is nothing
  marking which bytes get run.
- **The nesting is `exec > ingested > file`**, never `exec > file` —
  `file > exec` already means the opposite ("run this, paste the output
  here"), and the same two tags meaning inverse things by order would be
  unreadable.
- **Your project's `.gitignore` is the filter.** `bin/`, `obj/`,
  `node_modules/` are skipped, counted in `skipped=`, and named on the
  way past. A file that is not UTF-8 text and is *not* ignored refuses
  the whole ingest by name: a document body is raw bytes, so there is no
  encoding to hide a binary in.
- **After this the document owns those bytes.** Your four edits are
  ordinary edits to ordinary `hick:file` content; `hick lineage` reports
  them as `ingested` — with the run's fingerprint, so blame says "this
  arrived from that run" rather than "you wrote this" — and the volume
  is no longer flushed over them on the next run.

Re-ingesting the same cell is refused for now: it is a three-way merge
against the recorded `sha256`, and that is not built yet.

---

## 5. Copy, Cut, and Paste

Content reuse across the document uses an id-based clipboard.

- **copy** — stores content and keeps it in output
- **cut** — stores content but removes it from output
- **paste** — inserts stored content by id selector

```xml
<hick:copy id="version">2.0.0</hick:copy>

<hick:cut id="internal-notes">
This text is available to paste but not in the output.
</hick:cut>

<hick:file path="manifest.txt">
Version: <hick:paste select="#version" />
Notes: <hick:paste select="#internal-notes" />
</hick:file>
```

### Selecting

Selectors are CSS-shaped. `#id` takes one fragment; `.class` collects every
fragment carrying that class, in document order. `.a.b` requires **both**
classes, and a comma is union — `.a,.b` is "either".

`separator` goes between collected fragments, `distinct` collapses ones whose
text is identical (keeping the first), and `min`/`max` fail the run when a
paste collects the wrong number of them. `min`/`max` count what is emitted, so
`distinct` applies before the count.

```xml
<hick:paste select=".ignore" distinct separator="&#10;" min="1" />
```

### Contributing across documents

**Every other `.hick` in the same folder contributes its fragments**, and
neither document names the other — they meet on the class. This is how several
documents fill one generated file:

```xml

<hick:copy class="ignore">bin/</hick:copy>


<hick:file path=".gitignore"><hick:paste select=".ignore" distinct /></hick:file>
```

Discovery is that one folder, not the whole repository, and a contributor
offers only the fragments it declares itself — not the ones it gets from its
own `hick:upstream`.

A document that should keep its fragments to itself says so:

```xml
<hick:private />
```

A document's own fragment always wins over a contributed one with the same id;
`.class` collects both. `hick:upstream` is still there for naming a specific
document, including one in another folder.

A cell can contribute too: a `.hick` file a cell writes into an output volume
is read back, and the fragments in it are available to the documents in the
run. Only `.hick` outputs are read this way — a generator's ordinary output is
bytes, and scanning it for markup would make `<hick:` unwritable by any
program.

---

## 6. Fork and Attenuate

### Fork

Fork clones a container's state into a new container, optionally adding
restrictions. The fork depends on the source container's last exec.

```xml
<hick:container name="base" image="ubuntu:22.04" />

<hick:exec container="base">
apt-get update &amp;&amp; apt-get install -y build-essential
</hick:exec>

<hick:fork from="base" to="analyzer">
  <hick:deny network="*" />
</hick:fork>

<hick:exec container="analyzer">
./run-analysis
</hick:exec>
```

### Attenuate

Attenuate adds restrictions to an existing container without cloning.
Capabilities can only be narrowed, never escalated.

```xml
<hick:attenuate container="sandbox">
  <hick:deny network="*" />
</hick:attenuate>
```

---

## 7. Confirmation Gates

The `confirm` tag pauses execution and requires user approval before
proceeding.

```xml
<hick:confirm message="Deploy to production?" />
```

---

## 8. Variables

Variables let you define reusable values and reference them in file outputs.

### Declaring variables

```xml
<hick:var name="version">2.0.0</hick:var>
<hick:var name="env">production</hick:var>
```

### Using variables in file output

Inside `<hick:file>`, the `<hick:val>` tag interpolates a variable:

```xml
<hick:file path="banner.txt">
Application v<hick:val name="version" />
Environment: <hick:val name="env" />
</hick:file>
```

### CLI parameter overrides

Variables can be overridden from the command line with `--param`:

```
hick --param env=staging file.hick
```

CLI params take priority over `<hick:var>` declarations, which in turn
take priority over `_hick.yml` defaults.

---

## 9. Conditional Content

Nodes can be included or excluded based on variable values.

### The `<hick:when>` tag

Wraps a block of content that is included only when the condition is true.
When true, the wrapper is removed and children are promoted into the
parent. When false, the entire block is dropped.

```xml
<hick:var name="mode">prod</hick:var>

<hick:when test="mode=prod">
  <hick:exec container="deploy">./deploy.sh</hick:exec>
</hick:when>

<hick:when test="!debug">
  <hick:file path="release.txt">production build</hick:file>
</hick:when>
```

### The `when` attribute

Any hick tag can carry a `when` attribute for inline conditional logic:

```xml
<hick:exec container="linter" when="lint">
eslint src/
</hick:exec>

<hick:file path="debug.log" when="debug">
Debug output here
</hick:file>
```

### Condition syntax

| Condition | Meaning |
|-----------|---------|
| `var_name` | True if defined and non-empty |
| `!var_name` | True if undefined or empty |
| `var_name=value` | True if equals value |
| `var_name!=value` | True if not equal to value |

---

## 10. Exec Show Modes

When an `<hick:exec>` appears inside a `<hick:file>`, the `show`
attribute controls which parts of the transcript are rendered:

| Value | Renders |
|-------|---------|
| *(default)* | Commands (`$ ...`) and output |
| `command` | Commands only |
| `output` | Output only |
| `none` | Nothing (side-effect only) |

```xml
<hick:file path="report.md">
Full transcript:
<hick:exec container="builder">make build</hick:exec>

Output only:
<hick:exec container="builder" show="output">make test</hick:exec>

Side-effect (no output in file):
<hick:exec container="builder" show="none">make clean</hick:exec>
</hick:file>
```

---

## 11. Volume Access Rules

Volumes support explicit access rules that control which containers can
read or write. This replaces the default heuristic (first mounter writes,
others read) with fine-grained permissions.

```xml
<hick:volume name="project" output="src/MyApi/">
  <hick:allow container="scaffolder" write="**" />
  <hick:allow container="linter" read="**" />
  <hick:allow container="patcher" read="**" write="Controllers/**" />
</hick:volume>
```

**What is enforced.** A container the rules never name cannot mount the
volume at all — the run stops and says so. A container granted part of a
volume is handed that part and nothing else, and only its granted paths
are merged back in when it finishes; everything it wrote elsewhere stays
inside it. A volume with no `allow` children keeps the old heuristic and
stays unrestricted, so rules bind everyone only once anyone is named.

### Volume kinds

| Declaration | Behaviour |
|-------------|-----------|
| `<hick:volume name="x" />` | Ephemeral scratch (no host mapping) |
| `<hick:volume name="x" input="." />` | Snapshot of host directory (read source) |
| `<hick:volume name="x" output="dist/" />` | Contents become pipeline file outputs |
| `<hick:volume name="x" input="." output="." />` | Read existing state AND write updates |

When access rules are present, the DAG builder uses them to determine
writer/reader edges instead of the first-mounter heuristic. Multiple
writers on the same volume are sequenced automatically.

---

## 12. Project Configuration

A `_hick.yml` file in the project root provides defaults for the pipeline.
Hick searches for it by walking up directories (stopping at a `.git`
boundary).

```yaml
files:
  - docs/*.hick
  - reference.hick

vars:
  version: "2.0.0"
  env: staging

defaults:
  image: alpine:3.20
  images-dir: /opt/hick/images

output-dir: dist/

secrets:
  key-file: /etc/hick/key.txt
  secrets-dir: /etc/hick/secrets
```

With a config file, you can simply run `hick` with no arguments — files
and defaults are read from `_hick.yml`. CLI flags and `--param` values
override config file values.

---

## 13. Execution Caching

The `--cache` flag caches exec results to avoid re-running containers
when inputs haven't changed. Cache keys are SHA-256 hashes of the
container image, capabilities, command text, and secret names.

```
hick run --cache file.hick   # Record every cell, reuse on match
hick run --freeze file.hick  # Serve every cell from its recording
hick test --freeze file.hick # Verify against recordings, never execute
```

There is one setting here, not two switches, and it says **what a missing
recording means**: nothing (the default — run the cell, remember nothing),
"remember this" (`--cache`), or "this cell has no baseline yet"
(`--freeze`, and the per-cell `freeze="true"` below).

`hick run` is what establishes a baseline: a cell with no recording
that is not supposed to execute twice runs exactly once, on the run that
records it. `hick test` never writes a recording under any flag — a
check that can write the baseline it then compares against is not a check,
so it reports a cell with no recording as **unverifiable** (exit `2`)
instead. That is the command to use in CI when the question is *"is
everything already recorded?"*.

Cached transcripts are stored in `.hick-cache/transcripts/`.

### Freezing a single cell

`--freeze` sets the default for the whole run, but freeze is really a
property of one cell. Declare it with a `freeze` attribute on the exec:

```xml
<hick:exec container="deps" freeze="true">
cargo generate-lockfile
</hick:exec>

<hick:exec container="tests" freeze="false">
cargo test --test integration
</hick:exec>
```

The first cell runs once — on the run that records it — and is answered
from that recording forever after; the second runs every time, even under
`hick run --freeze`.

The rules:

- `freeze="true"` — this cell executes at most once, ever. The first
  `hick run` has no recording to answer it with, so it runs the cell
  and records it; every run after that replays the recording. No flag,
  and no edit to the document, is involved: declare a cell frozen the
  moment you write it. `hick test` never records, so it reports the
  cell as **unverifiable** (exit `2`) until some `hick run` has
  established the baseline.
  The recording is keyed by the container image, capabilities, command
  text, and secret names — not by the freeze attribute — so editing the
  command retires the recording and the next `hick run` records the
  new one.
- `freeze="false"` — this cell always runs, and is never answered from a
  recording, even when the run was started with `--freeze`.
- attribute omitted — the cell inherits the run-wide default.

Only `true` and `false` are accepted. A value like `freeze="yes"` is an
error rather than a silent `false`, because a typo in a verification
switch must never quietly turn verification off.

#### What freeze does *not* verify

A frozen cell is not executed, so its `<hick:expect>` assertions are
checked against the **recorded** output and therefore always pass. Freeze
verifies *"the document still produces what we recorded"* — not *"the
world still agrees."*

A document in which every cell is frozen is not a passing suite; it is a
suite that did not run. Freeze the cells whose output legitimately moves
over time — lockfiles, network fetches, timestamps — and leave your
integration tests unfrozen, which is exactly what per-cell freeze is for.

### Cell timeouts

Every executed cell runs under a wall-clock limit, so a cell that blocks —
reading stdin nobody will feed, listening on a socket, looping forever —
fails with a clear error instead of hanging `hick run`, `hick test`, and CI.
The limit resolves most-specific-first:

- `timeout="<seconds>"` on the cell — whole seconds, e.g. `timeout="600"`.
- `timeout="0"` on the cell — this one cell runs unbounded, explicitly.
- `HICKORY_CELL_TIMEOUT=<seconds>` — the default for every cell on this
  machine (`0` removes the default limit entirely).
- Otherwise: **120 seconds**.

```xml
<hick:exec container="bench" timeout="600">
cargo bench
</hick:exec>
```

A timed-out cell is killed for real — on Unix its whole process group, so
background children die with it — and the run fails naming the cell, the
limit that was hit, and both ways to raise it. A malformed value
(`timeout="2m"`, `timeout="1.5"`) is an error rather than a silently
removed limit. The limit is enforced where the process is spawned: the
local and sandbox executors; the docker and canopy executors currently run
cells unbounded.

---

## 14. Multi-Stage Pipeline

When a pipeline produces `.hick` files as output, they are automatically
fed as sources to a subsequent pipeline stage. This enables containers
to generate hick documents that drive further execution.

```xml

<hick:container name="scaffolder" image="alpine" />
<hick:exec container="scaffolder">
cat &gt; /output/build.hick &lt;&lt;'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="generated.txt">built by scaffolder</hick:file>
</hick:doc>
EOF
</hick:exec>
```

Rules:
- `.hick` files are removed from the final output (they are intermediate)
- The pipeline repeats up to `max_stages` times (default 3)
- If `.hick` files are still being produced at the limit, an error is raised
- Stage 1 files get `HickFile` provenance, stage 2+ get `GeneratedHick`

---

## 15. Watch Mode (`hick up`)

The `hick up` command runs the pipeline and watches source files for
changes, re-running automatically with merge-aware snapshots.

```
hick up docs/guide.hick

Options:
  --store auto|git|builtin   Version store backend (default: auto)
  --branch <name>            Snapshot branch (default: main)
  --merge-api <url>          LLM merge API for conflict resolution
  --max-stages <n>           Multi-stage limit (default: 3)
```

### How it works

1. Initial pipeline run produces output files
2. Output is merged with any user edits via three-way merge
3. A snapshot is committed to the version store
4. File watcher monitors `.hick` sources (debounced 100ms)
5. On change: re-run pipeline, merge, snapshot, write outputs

### Three-way merge

The merge system tracks three versions of each file:
- **Base** — the last snapshot's version
- **Generated** — the new pipeline output
- **Edited** — what's currently on disk (may have user edits)

| Base vs Edited | Base vs Generated | Result |
|---------------|-------------------|--------|
| Same | Different | Use generated (no user edits) |
| Different | Same | Keep edited (no pipeline changes) |
| Both different | — | Delegate to merge strategy |

Files with `HickFile` provenance always use the generated version
(users edit the `.hick` source, not the output).

### Store backends

| Backend | Storage | When to use |
|---------|---------|-------------|
| `auto` | Detects `.git` → git, else builtin | Default |
| `git` | Git plumbing at `.hick/git/` | When you want git-compatible history |
| `builtin` | Content-addressed files at `.hick/store/` | Standalone projects |

---

## 16. Information-Flow DAG

Hick validates the entire document as a directed acyclic graph before
execution. Dependencies are inferred automatically:

| Dependency type | Trigger |
|-----------------|---------|
| Container state | Multiple execs in same container |
| Volume flow | Writer then reader on same volume |
| Volume access rules | Explicit read/write permissions on volumes |
| Copy/paste | Producer then consumer of an id |
| Fork | Source container's last exec before fork target |
| Attenuate | Barrier between execs in same container |

Cycles are rejected at validation time with clear error messages.

---

## 17. Capability Tokens

Each container receives a macaroon-based capability token at pipeline
start. Tokens encode the container's allowed operations and can be
attenuated (restricted) but never escalated. This enables secure
delegation — a container can pass a restricted version of its own
token to a subprocess.

---

## 18. Custom Namespace Prefix

The parser detects the namespace prefix from the `xmlns:` declaration.
This means you can use any prefix, not just `hick:`:

```xml
<x:doc xmlns:x="http://www.hickorydocs.com/1.0">
  <x:file path="out.txt">hello</x:file>
</x:doc>
```

This is how this guide itself is written — using the `h:` prefix so
that `hick:` examples in the text body are treated as literal content
rather than parsed tags.

---

## 19. Running Hick

```
hick [file.hick ...]               Single-shot pipeline (default)
hick run [file.hick ...] [options]  Explicit single-shot mode
hick up <file.hick> [options]       Watch mode with merge snapshots
hick open [path]                    Open a folder or document in the desktop
                                    app, the way `code .` does. The app is a
                                    separate download; HICKORY_DESKTOP points
                                    at a copy directly
hick ingest --from '#id' <file.hick>  Ingest a cell's output volume (§4);
                                    running it again is a three-way merge
                                    against the base, recovered from the commit
                                    that introduced the recorded fingerprint
hick ingest --from carry &lt;session.hick&gt;           Distil a session into what carries to the
                                    next attempt: a prompt, the tests you kept,
                                    the approaches you ruled out
hick emit [dir]                     Show the commits a re-emission WOULD
                                    produce — one stage, one commit. Emits
                                    nothing, and refuses to rewrite anything
                                    below the publication floor
hick fleet invite|accept|list|grant This machine's keypair, and the machines
                                    paired with it
hick fleet serve|attach             Reach another of your machines' sessions,
                                    over QUIC dialled by public key. Only keys
                                    you hold are admitted, and every request is
                                    gated on that machine's grants
hick broker serve|allow|deny|log    One road out for a sealed machine, with a
                                    toll booth on it. A CONNECT proxy with a
                                    per-host policy; an unlisted host is denied
hick sealed --check                 Whether this machine holds no credential
                                    worth stealing. NOT an airgap: a machine
                                    that talks to a model is on a network
hick lineage <file.hick> --output <f> [--at <commit>] [--history]
                                    Byte-precise lineage; --at replays it
                                    as it stood at a commit, weave-only
                                    and without checking anything out

Common options:
  --config <path>          Path to _hick.yml (default: auto-discover)
  --param key=value        Set/override a variable
  --key-file <path>        Age identity file
  --secrets-dir <path>     Secrets directory
  --images-dir <path>      Pre-converted .wasm images
  --dry-run                Produce placeholders instead of running containers
  --cache                  Cache exec results, reuse on match
  --freeze                 Default every cell to frozen: serve it from its
                           recording, and record it on the one run that
                           has none. Per-cell freeze="…" overrides this
                           either way (see §13)
  --clear-cache            Clear cache before running
  -v, --verbose            Enable verbose logging

Watch mode options (hick up):
  --store auto|git|builtin Version store backend
  --branch <name>          Snapshot branch name
  --merge-api <url>        LLM merge API endpoint
  --max-stages <n>         Multi-stage pipeline limit
```

If no files are specified and a `_hick.yml` exists, files are read from
the config. Output files declared with `<hick:file>` are written to disk
relative to the current directory.

---

## 20. Live Demo

This guide is itself a `.hick` file (`docs/hick-guide.hick`). It uses
the `h:` prefix so that `hick:` code examples appear as literal text.

The file defines two containers with restricted capabilities, a shared
volume, a fork, and exec steps that exercise the full pipeline — DAG
validation, capability minting, volume flow, forking, and copy/paste —
every time the guide is built.

### Containers used to build this guide

```
guide-builder   (alpine:3.20, network denied, file-write /out/*)
guide-verifier  (alpine:3.20, network denied, file-read  /out/*)
guide-forked    (forked from guide-builder, network denied)
```

### Container transcript

The following block is an exec tag rendered by the pipeline — it shows
the full transcript for the referenced container (commands and output):

> $ cat out/stamp.txt
Built by hick



### Forked container transcript

The `guide-forked` container is a fork of `guide-builder`. It inherits
the builder's filesystem state (including `/out/stamp.txt`) without
re-running the original commands on the original container. The fork
also adds a network deny rule, further restricting the builder's
capabilities:

> $ cat out/stamp.txt && echo "Fork inherited builder state"
Built by hick
Fork inherited builder state



### Pasted build command

The builder's command is captured with `<hick:copy>` and pasted here:

> `echo "Built by hick" > out/stamp.txt`

### Variables and conditionals

This guide defines variables and uses conditional rendering:

> Guide version: 0.2.0

The following line only appears because `show-features` is defined:

> Feature list is enabled via conditional rendering.

### Pipeline features exercised

- **Namespace prefix detection** — `h:` prefix bound via xmlns
- **Container capabilities** — network deny, file allow
- **Volume flow** — `guide-builder` writes, `guide-verifier` reads
- **DAG validation** — volume dependency enforces ordering
- **Fork** — `guide-forked` clones builder state with added restrictions
- **Copy/paste** — content extracted from exec, pasted into output
- **Cut/paste** — namespace URI pasted from a cut block
- **Variables** — `guide-version` declared with `<hick:var>`, rendered with `<hick:val>`
- **Conditional content** — `<hick:when>` block controlled by `show-features` variable
- **File output** — this entire document
