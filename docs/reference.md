# Hick Reference

Hick is a document format and CLI tool for describing secure, containerised
workflows. A `.hick` file is XML that uses a single namespace to define
containers, capabilities, execution steps, and file outputs -- all validated
as a directed acyclic graph (DAG) before anything runs.

Containers execute in WebAssembly, network and file access is gated by
capability tokens, and the entire pipeline can run in dry-run mode to
produce documentation without starting any containers.

---

## Table of Contents

1. [Document Structure](#1-document-structure)
2. [Containers](#2-containers)
3. [Capability Rules](#3-capability-rules)
4. [Secrets](#4-secrets)
5. [Execution](#5-execution)
6. [Volumes](#6-volumes)
7. [File Outputs](#7-file-outputs)
8. [Copy, Cut, and Paste](#8-copy-cut-and-paste)
9. [Fork and Attenuate](#9-fork-and-attenuate)
10. [Confirmation Gates](#10-confirmation-gates)
11. [Information-Flow DAG](#11-information-flow-dag)
12. [Capability Tokens](#12-capability-tokens)
13. [CLI Reference](#13-cli-reference)
14. [Secrets Management](#14-secrets-management)
15. [Execution Modes](#15-execution-modes)
16. [Namespace Prefix Trick](#16-namespace-prefix-trick)
17. [Complete Example](#17-complete-example)
18. [Features](#18-features)
19. [Variables and Parameters](#19-variables-and-parameters)
20. [Conditional Content](#20-conditional-content)
21. [Text Substitution](#21-text-substitution)
22. [Exclude Patterns](#22-exclude-patterns)
23. [File Includes](#23-file-includes)
24. [Literate Programming (Weave)](#24-literate-programming-weave)
25. [Execution Caching](#25-execution-caching)
26. [Project Configuration](#26-project-configuration)
27. [Watch Mode](#27-watch-mode)
28. [Provenance Tracking](#28-provenance-tracking)

---

## 1. Document Structure

Every `.hick` file is an XML document whose root element binds a namespace
prefix to the hick namespace URI.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  <!-- content here -->
</hick:doc>
```

**Namespace URI:** `http://www.hickorydocs.com/1.0`

**Prefix:** The conventional prefix is `hick:`, but any prefix works as long
as the `xmlns` declaration points to the correct URI. The parser detects
which prefix to use from the declaration. See
[Namespace Prefix Trick](#16-namespace-prefix-trick) for advanced usage.

**Text content rules:** Only tags with the bound prefix are parsed as
structured elements. Everything else is raw text:

- `&`, `<`, `>` can appear literally without XML entity escaping
- This means you can embed markdown, code snippets, or other XML in the
  text body without escaping

**XML declaration:** The `<?xml ...?>` declaration is optional. If present
it is skipped during parsing.

**Comments:** Standard XML comments (`<!-- ... -->`) are treated as raw text
content, not parsed as hick elements.

---

## 2. Containers

Containers declare isolated WASM execution environments.

```xml
<hick:container name="builder" image="python:3.12">
  <!-- capability rules go here -->
</hick:container>
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Unique identifier used to reference this container in `exec` tags |
| `image` | No | OCI container image name (e.g., `alpine:3.20`, `ubuntu:22.04`) |

### Children

Container tags can contain any combination of:

- `<hick:allow .../>` -- Grant a specific capability
- `<hick:deny .../>` -- Deny a category of access
- `<hick:secret .../>` -- Inject a secret as an environment variable

### Implicit containers

If an `exec` tag references a container that hasn't been declared with a
`container` tag, providing the `image` attribute on that first `exec`
implicitly creates the container:

```xml
<!-- No <hick:container> needed -- this creates the container implicitly -->
<hick:exec container="demo" image="alpine">
echo hello
</hick:exec>
```

Implicit containers have no capability restrictions (all network and file
access is allowed).

---

## 3. Capability Rules

Capability rules control what a container can access. They appear as
children of `<hick:container>`, `<hick:fork>`, or `<hick:attenuate>` tags.

### Network rules

| Tag | Description |
|-----|-------------|
| `<hick:allow network="host:port" />` | Allow traffic to a specific host and port |
| `<hick:deny network="*" />` | Deny all network traffic not explicitly allowed |

**Host** can be a domain name, IP address, or `*` (wildcard).
**Port** can be a number or `*` (wildcard).

**Security model:**

- Without any `deny` rule: all network traffic is permitted (allow-all
  default)
- Once `<hick:deny network="*" />` is present: only connections matching
  explicit `allow` rules are permitted (deny-by-default)

**Examples:**

```xml
<!-- Allow HTTPS to two hosts, deny everything else -->
<hick:allow network="github.com:443" />
<hick:allow network="pypi.org:443" />
<hick:deny network="*" />

<!-- Allow all traffic to a host on any port -->
<hick:allow network="example.com:*" />

<!-- Allow all HTTPS traffic -->
<hick:allow network="*:443" />
```

### File rules

| Tag | Description |
|-----|-------------|
| `<hick:allow file-read="/path/*" />` | Allow reading files under path |
| `<hick:allow file-write="/path/*" />` | Allow writing files under path (implies read) |

**Path patterns** support a trailing `*` for prefix matching:

- `/output/*` -- matches any file under `/output/`
- `/input/data.txt` -- matches only that exact file
- `/etc/config/*` -- matches anything under `/etc/config/`

**Write implies read:** A container with `file-write` access to a path can
also read from that path.

**No default access:** If any file rules are defined, only paths matching
those rules are accessible. If no file rules are defined, all file access is
allowed.

---

## 4. Secrets

Secrets are injected into containers as environment variables. They are
declared as children of `<hick:container>` tags.

```xml
<hick:container name="deploy" image="alpine">
  <hick:secret name="API_TOKEN" from="github-token" />
  <hick:secret name="AWS_SECRET" from="aws-credentials" />
</hick:container>
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Environment variable name inside the container |
| `from` | Yes | Secret identifier (resolved by the secrets provider) |

### How secrets are stored

By default, secrets are stored as age-encrypted files in a secrets
directory. See [Secrets Management](#14-secrets-management) for details.

---

## 5. Execution

The `exec` tag runs commands inside a container.

```xml
<hick:exec container="builder" image="alpine">
apk add curl
curl --version
</hick:exec>
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `container` | Yes | Name of the container to execute in |
| `image` | No | Container image. Creates the container implicitly if not yet declared |
| `mount` | No | Volume mounts in `volume-name:/mount/path` format. Comma-separated for multiple |
| `show` | No | Control what is rendered in file outputs: `all` (default), `command`, `output`, `none` |
| `when` | No | Condition expression; exec is skipped if the condition evaluates to false |

### Content

The text content of an `exec` tag is the command to run. Multiple lines are
supported -- each non-empty line is sent to the container's shell as a
separate command.

### Sequencing

Multiple `exec` tags targeting the same container are automatically
sequenced in document order. This is called a **container-state dependency**
and is enforced by the DAG.

```xml
<!-- These run in order, guaranteed by the DAG -->
<hick:exec container="builder" image="alpine">
apk add curl
</hick:exec>

<hick:exec container="builder">
curl --version
</hick:exec>
```

### Nested execs

`exec` tags can appear inside `<hick:file>` tags. When they do, their
output is rendered into the file content. See [File Outputs](#7-file-outputs).

---

## 6. Volumes

Volumes provide shared filesystem storage between containers.

```xml
<hick:volume name="artifacts" />
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Unique identifier for the volume |

### Mounting volumes

Volumes are mounted via the `mount` attribute on `exec` tags:

```xml
<hick:volume name="shared" />

<hick:exec container="producer" mount="shared:/output">
echo "data" > /output/result.txt
</hick:exec>

<hick:exec container="consumer" mount="shared:/input">
cat /input/result.txt
</hick:exec>
```

### Writer / reader semantics

- The **first** `exec` to mount a volume is treated as the **writer**
- **Subsequent** `exec` tags mounting the same volume are **readers**
- A volume-flow dependency edge is automatically created: all readers
  depend on the writer

### Multiple mounts

Comma-separate multiple volume mounts:

```xml
<hick:exec container="builder" mount="src:/source,out:/output">
cp /source/main.c /output/
</hick:exec>
```

---

## 7. File Outputs

The `file` tag declares a file to be written to disk. Its content is a mix
of literal text, exec transcripts, and paste references.

```xml
<hick:file path="report.md">
# Build Report

## Commands Run
<hick:exec container="builder">make build</hick:exec>

## Version
<hick:paste select="#version" />
</hick:file>
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `path` | Yes | Output file path, relative to the current working directory |

### Content

Files can contain:

- **Literal text** -- passed through as-is
- **`<hick:exec>` tags** -- replaced with the container's transcript
- **`<hick:paste>` tags** -- replaced with the referenced copy/cut content

### Exec rendering in files

How `exec` tags inside files are rendered depends on the execution mode:

**Dry-run mode** (default):

```
$ command1
$ command2
```

Only commands are shown, prefixed with `$ `.

**Live mode:**

```
$ command1
[actual command1 output]
$ command2
[actual command2 output]
```

Commands are shown with their real output.

### Parent directories

Parent directories are created automatically if they don't exist.

---

## 8. Copy, Cut, and Paste

Content reuse across the document uses an id-based clipboard system.

### Copy

Stores content **and** keeps it visible in the output.

```xml
<hick:copy id="version">2.0.0</hick:copy>
```

### Cut

Stores content but **removes** it from the output.

```xml
<hick:cut id="internal-notes">
This text is available to paste but not in the output.
</hick:cut>
```

### Paste

Inserts previously stored content by id or class selector. Self-closing tag.

```xml
<hick:paste select="#version" />
<hick:paste select=".dependencies" separator="\n" />
<hick:paste select=".routes" min="1" />
```

### Attributes

**Copy / Cut:**

| Attribute | Required | Description |
|-----------|----------|-------------|
| `id` | Yes | Unique identifier for the content block |

**Paste:**

| Attribute | Required | Description |
|-----------|----------|-------------|
| `select` | Yes | CSS-style `#id` or `.class` selector referencing copy or cut blocks |
| `separator` | No | String inserted between blocks for `.class` selectors |
| `min` | No | Minimum number of matching blocks required (error if fewer) |
| `max` | No | Maximum number of matching blocks allowed (error if more) |

The `min` and `max` attributes enable contract enforcement:
- `min="1"` makes the paste required (build fails if no matching block exists)
- `min="1" max="1"` requires exactly one match
- Applied before content resolution; errors abort the build

### Resolution

Paste resolves in this order:

1. Cut blocks (checked first)
2. Copy blocks

If no matching block is found and no `min` constraint is set, a warning is
logged and the paste produces empty output. If `min` is set and the count is
below the minimum, the build fails with an error.

### Cross-document scope

Copy, cut, and paste work across documents when multiple `.hick` files are
processed in the same pipeline run.

---

## 9. Fork and Attenuate

### Fork

Fork clones a container's state into a new container, optionally adding
capability restrictions. The new container inherits the source's filesystem
state by replaying its command history.

```xml
<hick:fork from="base" to="analyzer">
  <hick:deny network="*" />
</hick:fork>
```

#### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `from` | Yes | Name of the source container to clone |
| `to` | Yes | Name of the new container |

#### Children

Optional capability rules (`<hick:allow>`, `<hick:deny>`) that are applied
as additional restrictions on the forked container.

#### Behaviour

1. A fresh container is started from the same image as the source
2. Every command from the source container's history is replayed in order
3. The forked container inherits the source's capabilities plus any
   additional restrictions from the fork tag
4. DAG dependency: the fork depends on the source container's last `exec`

#### Use cases

- Build a base environment once, then create restricted variants for
  analysis
- Run untrusted code in a fork with network access denied
- Avoid rebuilding tools in multiple containers

### Attenuate

Attenuate adds capability restrictions to an existing container without
cloning it. It acts as a barrier in the DAG.

```xml
<hick:attenuate container="sandbox">
  <hick:deny network="*" />
</hick:attenuate>
```

#### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `container` | Yes | Name of the container to restrict |

#### Children

Capability rules (`<hick:allow>`, `<hick:deny>`) to add.

#### Key constraint

Capabilities can only be **narrowed** (add restrictions), never
**escalated** (remove restrictions). This is a fundamental property of the
macaroon-based token system.

---

## 10. Confirmation Gates

The `confirm` tag pauses execution and requires user approval before
continuing. Self-closing tag.

```xml
<hick:confirm message="Deploy to production?" />
```

| Attribute | Required | Description |
|-----------|----------|-------------|
| `message` | Yes | Message displayed to the user |

> **Note:** Confirmation gates are parsed but not yet implemented in the
> runtime. They are a planned feature.

---

## 11. Information-Flow DAG

Before any container starts, hick validates the entire document as a
directed acyclic graph. Dependencies are inferred automatically from the
document structure.

### Dependency types

| Type | Trigger | Effect |
|------|---------|--------|
| Container state | Multiple `exec` tags in the same container | Sequential execution in document order |
| Volume flow | First mounter writes, subsequent mount reads | Readers depend on the writer |
| Copy / paste | `copy` produces, `paste` consumes | Consumer depends on producer |
| Fork | `fork` references a source container | Fork target depends on source's last exec |
| Attenuate | `attenuate` references a container | Creates a sequencing barrier |

### Topological ordering

Exec nodes are sorted using Kahn's algorithm. Nodes with no incoming edges
are **roots** and can execute immediately. Downstream nodes execute only
after their dependencies complete.

### Independent execs

Execs in **different containers** with no volume or copy/paste relationship
have no dependency edges and are eligible for parallel execution (when
parallel execution is implemented).

### Cycle detection

If the dependency graph contains a cycle, hick rejects the document with an
error identifying the problematic node.

---

## 12. Capability Tokens

Each container receives a macaroon-based capability token at pipeline start.

### Properties

- **Unforgeable:** Tokens are signed with a random root key generated per
  pipeline run
- **Attenuable:** Restrictions can be added (via fork or attenuate) but
  never removed
- **Verifiable:** The token authority can verify any token against the root
  key
- **Serialisable:** Tokens can be serialised (V2 format) for delegation

### Token structure

Each token encodes:

- **Identifier:** `container-{name}`
- **Location:** `hick://gateway:8080`
- **Caveats:** One per capability rule, in these formats:
  - `network = allow host:port`
  - `network = deny *`
  - `file-read = /path`
  - `file-write = /path`
  - `secret = ENV_VAR:secret-name`

### Enforcement

- **Network filter:** Connection attempts are validated against the
  container's token rules
- **File filter:** File read/write operations are checked against token
  rules

---

## 13. CLI Reference

```
hick [SUBCOMMAND] [OPTIONS] [FILES...]
```

When invoked with no subcommand, bare `.hick` files default to `run`. When
invoked with no arguments at all, the REPL starts.

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `run` | Run the pipeline once (default when files are given) |
| `up` | Watch mode: re-run pipeline on file changes with merge-aware snapshots |
| `repl` | Interactive REPL: explore containers and record a session transcript |
| `generate-matrix` | Generate all valid feature combinations to separate output directories |
| `init` | Initialize AI agent instructions for the project |
| `validate` | Validate AI agent configuration |

### `run` options

| Option | Default | Description |
|--------|---------|-------------|
| `--config <PATH>` | auto-discover `_hick.yml` | Path to config file |
| `--key-file <PATH>` | `~/.config/hick/key.txt` | Age identity file for decrypting secrets |
| `--secrets-dir <PATH>` | `~/.config/hick/secrets` | Directory containing age-encrypted secret files |
| `--images-dir <PATH>` | | Directory containing pre-converted `.wasm` container images |
| `--param <KEY=VALUE>` | | Set a variable (repeatable) |
| `--features <LIST>` | | Enable features (comma-separated) |
| `--output-dir <PATH>` | | Output directory for generated files (overrides config) |
| `--dry-run` | `false` | Produce placeholder transcripts instead of running containers |
| `--cache` | `false` | Cache execution results; reuse on subsequent runs when inputs match |
| `--freeze` | `false` | Require all exec results to be cached, never re-execute |
| `--clear-cache` | `false` | Clear the cache directory before running |
| `-v`, `--verbose` | `false` | Enable verbose (debug-level) logging |

### `up` options

| Option | Default | Description |
|--------|---------|-------------|
| `--key-file <PATH>` | `~/.config/hick/key.txt` | Age identity file for decrypting secrets |
| `--secrets-dir <PATH>` | `~/.config/hick/secrets` | Directory containing age-encrypted secret files |
| `--images-dir <PATH>` | | Directory containing pre-converted `.wasm` container images |
| `--param <KEY=VALUE>` | | Set a variable (repeatable) |
| `--features <LIST>` | | Enable features (comma-separated) |
| `--dry-run` | `false` | Produce placeholder transcripts |
| `--store <BACKEND>` | `auto` | Store backend: `auto`, `git`, or `builtin` |
| `--merge-api <URL>` | | LLM merge API URL for intelligent conflict resolution |
| `--branch <NAME>` | `main` | Branch name for snapshot tracking |
| `--max-stages <N>` | `3` | Maximum number of multi-stage pipeline passes |
| `-v`, `--verbose` | `false` | Enable verbose logging |

### `generate-matrix` options

| Option | Description |
|--------|-------------|
| `--jobs N` / `-j N` | Run N combinations in parallel (0 = auto-detect CPU count) |
| `--singles-only` | Only generate single-feature combinations |
| `--max-combination-size N` | Limit combinations to at most N features |
| `--include-features X,Y` | Only use these features for combinations |
| `--exclude-features X,Y` | Exclude these features from combinations |
| `--verify "cmd"` | Run verification command for each combination |
| `--continue-on-error` | Continue if a combination fails |
| `-y`, `--yes` | Skip confirmation prompt for embedded verify commands |
| `--dry-run` | Show combinations without generating |

### `repl` options

| Option | Default | Description |
|--------|---------|-------------|
| `--images-dir <PATH>` | | Directory containing pre-converted `.wasm` container images |
| `--output <PATH>` | `session.hick` | Output transcript file |
| `-v`, `--verbose` | `false` | Enable verbose logging |

### Examples

```bash
# Dry-run: parse, validate, and generate file outputs without containers
hick --dry-run docs/guide.hick

# Live execution with default settings
hick docs/guide.hick

# Set variables and features
hick --param env=prod --features auth,oauth docs/project.hick

# Watch mode: re-run on file changes
hick up docs/guide.hick

# Generate all feature combinations and verify each
hick generate-matrix --verify "cargo check" ./scaffolding/

# REPL mode (no arguments)
hick

# Custom secrets and images directories
hick --key-file /etc/hick/key.txt \
     --secrets-dir /etc/hick/secrets \
     --images-dir /opt/wasm-images \
     docs/deploy.hick

# Cache execution results for faster re-runs
hick --cache docs/guide.hick

# Freeze mode (fail if any exec is not cached)
hick --freeze docs/guide.hick
```

### Environment variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Control log level (e.g., `RUST_LOG=debug` or `RUST_LOG=hick=debug`) |
| `HOME` | Used to expand `~` in default paths |

### Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (parse failure, DAG validation, execution error, or file write failure) |

---

## 14. Secrets Management

Hick uses [age](https://age-encryption.org/) encryption for secret storage.

### Directory layout

```
~/.config/hick/
  key.txt               # age identity (private key) file
  secrets/
    github-token.age    # encrypted secret file
    aws-credentials.age
    pulumi-token.age
```

### Creating a key

```bash
age-keygen -o ~/.config/hick/key.txt
```

### Encrypting a secret

```bash
# Get the public key from the identity file
age-keygen -y ~/.config/hick/key.txt
# Output: age1...

# Encrypt a secret
echo -n "ghp_abc123" | age -r age1... -o ~/.config/hick/secrets/github-token.age
```

### Referencing secrets in hick

```xml
<hick:container name="deploy" image="alpine">
  <hick:secret name="GITHUB_TOKEN" from="github-token" />
</hick:container>
```

The `from` attribute (`github-token`) maps to the file
`~/.config/hick/secrets/github-token.age`. The decrypted value is injected
as the environment variable `GITHUB_TOKEN` inside the container.

### Custom paths

Override the key file and secrets directory via CLI flags:

```bash
hick --key-file /path/to/key.txt --secrets-dir /path/to/secrets docs/deploy.hick
```

---

## 15. Execution Modes

### Dry-run mode

**When:** `--dry-run` flag is passed (or no `--images-dir` available).

**What happens:**

1. All `.hick` files are parsed
2. DAGs are built and validated
3. Capability tokens are minted
4. Copy/paste references are resolved
5. File outputs are generated with command-only transcripts
6. Files are written to disk

**No containers are started.** Exec tags in file outputs show only the
commands:

```
$ apk add curl
$ curl --version
```

**Use cases:** Generate documentation with example commands, test document
structure, validate DAGs without resource cost.

### Live mode

**When:** No `--dry-run` flag (default).

**What happens:**

1. All `.hick` files are parsed
2. DAGs are built and validated
3. Capability tokens are minted
4. WASM containers are started for each referenced image
5. Commands are executed in topological order
6. Full output is captured (stdout + stderr)
7. Containers are shut down
8. File outputs are generated with full transcripts
9. Files are written to disk

Exec tags in file outputs show commands **and** their real output:

```
$ apk add curl
fetch https://dl-cdn.alpinelinux.org/alpine/v3.20/main/...
OK: 8 MiB in 17 packages
$ curl --version
curl 8.9.1 (x86_64-alpine-linux-musl) ...
```

### Timeouts

| Timeout | Duration | Description |
|---------|----------|-------------|
| Boot timeout | 120 seconds | Waiting for the container kernel to show a shell prompt |
| Command timeout | 300 seconds | Waiting for a single command to complete |

### Output cleaning

Container output is automatically cleaned of:

- ANSI colour and escape sequences
- Terminal Device Status Report queries (ESC[6n)
- Carriage returns
- Shell prompt echoes (e.g., `/ # command`)
- Internal marker lines used for output capture

### Image resolution

When starting a container, hick resolves the image name to a `.wasm` file:

1. `{images_dir}/{base_name}.wasm` -- exact file match
2. `{images_dir}/{base_name}/` -- directory; picks the latest `.wasm` by
   filename sort
3. Falls back to the raw name (produces a clear error if not found)

Image tags (e.g., `:3.20` in `alpine:3.20`) are stripped for file
resolution. The base name `alpine` is used to find `alpine.wasm` or the
`alpine/` directory.

---

## 16. Namespace Prefix Trick

The parser only recognises tags with the prefix bound to the hick namespace.
This means you can use a short prefix (like `h:`) and embed `hick:` example
code as literal text:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="tutorial.md">
# Tutorial

Here is an example hick file:

<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  <hick:exec container="demo" image="alpine">
  echo hello
  </hick:exec>
</hick:doc>

The above produces output in the demo container.
</h:file>
</h:doc>
```

Because the parser uses `h:` as the active prefix, all `<hick:...>` tags in
the text body are treated as raw text and included verbatim in the output.

This is how the hick language guide (`docs/hick-guide.hick`) is written --
it generates its own documentation.

---

## 17. Complete Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<!-- Container definitions with capability restrictions -->
<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-write="/output/*" />
</hick:container>

<hick:container name="mailer" image="alpine">
  <hick:allow network="smtp.gmail.com:587" />
  <hick:deny network="*" />
  <hick:allow file-read="/output/*" />
</hick:container>

<!-- Shared volume for inter-container data -->
<hick:volume name="shared-output" />

<!-- Reusable values -->
<hick:copy id="version">1.0.0</hick:copy>
<hick:cut id="build-id">build-20240115-abc</hick:cut>

<!-- Reporter installs a package and generates a report -->
<hick:exec container="reporter" mount="shared-output:/output">
pip install untrusted-package
python -c "
import untrusted_package
with open('/output/report.html', 'w') as f:
    f.write(untrusted_package.generate())
"
</hick:exec>

<!-- Mailer sends the report (depends on reporter via volume) -->
<hick:exec container="mailer" mount="shared-output:/output">
apk add msmtp
cat /output/report.html | msmtp bob@example.com
</hick:exec>

<!-- Fork the reporter for offline analysis -->
<hick:fork from="reporter" to="analyzer">
  <hick:deny network="*" />
</hick:fork>

<hick:exec container="analyzer" mount="shared-output:/output">
wc -l /output/report.html
</hick:exec>

<!-- File output -->
<hick:file path="build-report.md">
# Build Report v<hick:paste select="#version" />

## Reporter Transcript
<hick:exec container="reporter">pip install untrusted-package</hick:exec>

## Mailer Transcript
<hick:exec container="mailer">cat /output/report.html</hick:exec>
</hick:file>

</hick:doc>
```

### What this document does

1. **Defines two containers** with restricted network access and file
   permissions
2. **Creates a shared volume** for passing data between containers
3. **Stores values** with copy (visible) and cut (hidden) for later reuse
4. **Runs the reporter** container, which installs a package and generates a
   report -- it can only reach GitHub and PyPI over HTTPS
5. **Runs the mailer** container, which reads the report and emails it --
   it can only reach the SMTP server
6. **Forks the reporter** into an analyzer with all network access denied
7. **Generates a markdown report** with embedded container transcripts

### DAG for this document

```
reporter (exec 0) ──volume──> mailer (exec 1)
reporter (exec 0) ──fork────> analyzer (exec 2)
```

The mailer and analyzer are both downstream of the reporter but independent
of each other.

---

## 18. Features

Features are named, optional capabilities that control conditional content and
code generation. They support dependencies, mutual exclusivity, and can be
combined in matrices for multi-variant builds.

### Defining features

```xml
<hick:feature name="auth" description="Authentication support" />
<hick:feature name="oauth" description="OAuth provider" requires="auth" />
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Unique feature identifier |
| `description` | Yes | Human-readable description |
| `requires` | No | Space-separated list of features this depends on |
| `conflicts_with` | No | Space-separated list of features this conflicts with |
| `exclusive_group` | No | Group name; only one feature from each group can be enabled |

### Dependencies with `requires`

When a feature is enabled, all features in its `requires` list are
automatically enabled too:

```xml
<hick:feature name="auth" description="Authentication" />
<hick:feature name="oauth" description="OAuth" requires="auth" />
<hick:feature name="premium" description="Premium" requires="auth billing" />
```

Enabling `premium` automatically enables `auth` and `billing`.

### Mutual exclusivity with `conflicts_with`

Use `conflicts_with` for arbitrary pairwise conflicts:

```xml
<hick:feature name="oidc" description="OIDC auth" conflicts_with="p2p-auth" />
<hick:feature name="p2p-auth" description="P2P auth" conflicts_with="oidc" />
```

Specifying the conflict on either side is sufficient, though declaring it on
both is recommended for clarity.

### Mutual exclusivity with `exclusive_group`

Use `exclusive_group` for "pick one of N" scenarios:

```xml
<hick:feature name="postgres" description="PostgreSQL" exclusive_group="database" />
<hick:feature name="sqlite" description="SQLite" exclusive_group="database" />
<hick:feature name="mysql" description="MySQL" exclusive_group="database" />
```

Only one feature from each exclusive group can be enabled at a time.

### Using features

Enable features via CLI:

```bash
hick --features auth,oauth docs/project.hick
```

Use features in conditional content:

```xml
<hick:when test="auth">
    Auth-specific content
</hick:when>

<hick:file path="config.json" when="oauth">
    OAuth-only file
</hick:file>
```

### Matrix generation

The `generate-matrix` command creates all valid feature combinations:

```bash
hick generate-matrix --verify "cargo check" ./scaffolding/
```

This command:

1. Extracts all features from the hick files
2. Generates the power set of feature combinations
3. Filters out combinations with conflicting features (via `conflicts_with` or
   `exclusive_group`)
4. Generates each valid combination to a separate directory
5. Optionally runs verification commands

Useful flags:

| Flag | Description |
|------|-------------|
| `--jobs N` / `-j N` | Run N combinations in parallel (0 = auto-detect CPU count) |
| `--singles-only` | Only generate single-feature combinations |
| `--max-combination-size N` | Limit combinations to at most N features |
| `--include-features X,Y` | Only use these features for combinations |
| `--exclude-features X,Y` | Exclude these features from combinations |
| `--verify "cmd"` | Run verification command for each combination |
| `--continue-on-error` | Continue if a combination fails |
| `-y`, `--yes` | Skip confirmation prompt for embedded verify commands |
| `--dry-run` | Show combinations without generating |

### Embedded verify commands

Instead of passing `--verify` on every invocation, embed verify commands
directly in hick files:

```xml
<hick:verify command="cargo check" description="Verify project compiles" />
<hick:verify command="[backend] cargo test" description="Run backend tests" />
<hick:verify command="[spa+backend] npm run e2e" description="E2E tests" />
```

| Attribute | Required | Description |
|-----------|----------|-------------|
| `command` | Yes | Command string in `[condition] command` format (same as `--verify`) |
| `description` | No | Human-readable label shown in confirmation prompt |

**Security:** Because embedded commands come from files that may be
untrusted, the CLI prompts for confirmation before running them:

| Scenario | Behaviour |
|----------|-----------|
| Embedded cmds, TTY, no `--yes` | Show commands, prompt `Run these commands? [y/N]` |
| Embedded cmds, TTY, `--yes` | Show commands, proceed without prompt |
| Embedded cmds, no TTY, no `--yes` | Error: "requires `--yes` in non-interactive mode" |
| Embedded cmds, no TTY, `--yes` | Show commands, proceed |
| No embedded cmds | No prompt regardless |
| Only CLI `--verify` | No prompt (user explicitly typed them) |
| `--dry-run` | Show commands in dry-run output, no prompt |

Embedded verify commands run first, followed by any CLI `--verify` commands.

---

## 19. Variables and Parameters

Document-level variables allow parameterising `.hick` files. Variables can be
set in the document itself and overridden from the CLI.

### Declaring variables

```xml
<hick:var name="python_version">3.12</hick:var>
<hick:var name="project_name">MyApp</hick:var>
```

Variable text content becomes the variable's value. Variables are scanned
and registered **before** conditional filtering, so declarations inside
`<hick:when>` blocks are available for condition evaluation.

### Referencing variables

Use `<hick:val>` to output a variable's value:

```xml
<hick:file path="setup.md">
Python version: <hick:val name="python_version" />
Project: <hick:val name="project_name" />
</hick:file>
```

Variables can also be referenced in file paths and substitution values using
double-brace syntax with optional case variants:

```
{{python_version}}
{{project_name:snake_case}}
{{project_name:PascalCase}}
```

### CLI override

```bash
hick --param python_version=3.13 --param project_name=OtherApp docs/guide.hick
```

CLI `--param` values take precedence over in-document `<hick:var>` defaults.

### `<hick:var>` attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Variable identifier |

### `<hick:val>` attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Variable name to output |

If the variable is not defined, a warning is logged and nothing is rendered.

---

## 20. Conditional Content

Content can be conditionally included or excluded based on variable values
and feature flags.

### The `<hick:when>` tag

```xml
<hick:when test="auth">
  Authentication setup here — only included if "auth" is truthy.
</hick:when>

<hick:when test="env=prod">
  Production-only content.
</hick:when>

<hick:when test="!debug">
  Included when "debug" is NOT set.
</hick:when>
```

When the condition is true, the children are promoted to the parent level
(the `<hick:when>` wrapper is removed). When false, the entire tag and its
children are removed.

### The `when` attribute

Any tag can carry a `when` attribute. If the condition evaluates to false,
the entire tag is removed:

```xml
<hick:exec container="deploy" when="env=prod">
./deploy-prod.sh
</hick:exec>

<hick:file path="config.json" when="oauth">
  OAuth-only file
</hick:file>
```

### Condition syntax

| Format | Meaning |
|--------|---------|
| `when="var"` | True if `var` is defined and non-empty |
| `when="var=value"` | True if `var` equals `value` |
| `when="var!=value"` | True if `var` does not equal `value` |
| `when="!var"` | True if `var` is not defined or empty |

Features are automatically available as variables — `when="auth"` is true
when the `auth` feature is enabled.

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `test` | Yes (on `<hick:when>`) | Condition expression |
| `when` | No (on any tag) | Condition expression |

---

## 21. Text Substitution

The `<hick:substitute>` tag registers a text replacement that applies to
all file and weave output. Substitutions are applied **after** file content
is assembled.

### Basic substitution

```xml
<hick:substitute name="project" pattern="MyTemplate" value="MyApp" />
```

Every occurrence of `MyTemplate` in any output file is replaced with `MyApp`.

### Case variants

When `variants="true"`, both the pattern and value are expanded into all
case forms (PascalCase, camelCase, snake_case, SCREAMING_SNAKE_CASE, etc.):

```xml
<hick:substitute name="codename" pattern="BlueIronCube" value="ProjectX" variants="true" />
```

This generates replacements for `BlueIronCube` → `ProjectX`,
`blueIronCube` → `projectX`, `blue_iron_cube` → `project_x`, etc.

### Variable fallback

If `value` is omitted, the substitution resolves `name` as a variable:

```xml
<hick:var name="app_name">MyApplication</hick:var>
<hick:substitute name="app_name" pattern="AppTemplate" />
<!-- Uses the value of app_name variable -->
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Substitution identifier (also fallback variable name) |
| `pattern` | Yes | Literal text to search for |
| `value` | No | Replacement text; falls back to resolving `name` as a variable |
| `variants` | No | When `"true"`, generates all case variants of pattern and value |

Patterns are applied longest-first to avoid partial replacements. Empty
patterns are silently ignored.

---

## 22. Exclude Patterns

The `<hick:exclude>` tag registers a glob pattern for files to exclude from
pipeline output. Exclusions are applied at the very end, after all file
outputs are generated.

```xml
<hick:exclude pattern="*.template-only" />
<hick:exclude pattern="docs/internal/**" />
<hick:exclude pattern="TEMPLATE_README.md" />
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `pattern` | Yes | Glob pattern (`*`, `**`, `?` wildcards) |

Multiple `<hick:exclude>` tags accumulate. Empty patterns are silently
ignored. Invalid patterns are logged as warnings but do not fail the
pipeline.

---

## 23. File Includes

The `<hick:include>` tag splices the contents of another `.hick` file into
the current document at parse time.

```xml
<hick:include file="fragments/setup.hick" />
<hick:include file="../common/vars.hick" />
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `file` | Yes | Relative path to the `.hick` file to include |

### Behaviour

- Resolved **during parsing**, before conditional filtering or processing.
- The path is resolved relative to the including file's directory.
- The included file's top-level nodes are spliced into the parent at the
  tag's position (the `<hick:include>` tag itself is removed).
- Recursive includes are supported.
- Circular includes are detected and produce an error.

---

## 24. Literate Programming (Weave)

The `weave` attribute on `<hick:doc>` enables literate programming output.
When set, hick generates a markdown documentation file alongside the regular
output files.

```xml
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="GUIDE.md">
  This prose goes directly into GUIDE.md.

  <hick:file path="setup.sh">
  #!/bin/bash
  echo "setup"
  </hick:file>
</hick:doc>
```

### Weave output rules

- **Text nodes** → prose (with substitutions and dedenting applied)
- **`<hick:file>` tags** → markdown heading + fenced code block with
  language detection based on file extension
- **`<hick:file doc-hidden="true">`** → excluded from weave output
  (processed normally for file output)
- **`<hick:val>` tags** → variable value resolved inline
- **`<hick:when>` tags** → children processed recursively (already filtered)

### Language detection for code fences

File extensions are mapped to markdown fence identifiers: `.rs` → `rust`,
`.ts`/`.tsx` → `typescript`, `.py` → `python`, `.swift` → `swift`,
`.kt` → `kotlin`, `.cs` → `csharp`, `.go` → `go`, `.json` → `json`,
`.yaml` → `yaml`, `.toml` → `toml`, `.sh` → `bash`, `.sql` → `sql`,
and many more.

---

## 25. Execution Caching

Exec results can be cached to avoid re-running containers when inputs have
not changed.

```bash
hick --cache docs/guide.hick          # Use cache on hash match
hick --freeze docs/guide.hick         # Fail if any exec is not cached
hick --clear-cache docs/guide.hick    # Clear cache and run fresh
```

The cache key is computed from the command text, container image, and
capability rules. When `--cache` is enabled and a matching cached result
exists, the cached transcript is used instead of running the container.

`--freeze` mode is stricter: it requires every exec result to already be
cached and will fail with an error if any is missing. This is useful for
CI/CD pipelines where you want deterministic builds without network access.

---

## 26. Project Configuration

A `_hick.yml` file at the project root provides default settings so they
do not need to be repeated on every CLI invocation.

```yaml
files:
  - docs/setup.hick
  - docs/build.hick
  - docs/deploy.hick
vars:
  env: dev
  region: us-east-1
defaults:
  image: alpine:3.20
  images-dir: ./images
output-dir: ./output
secrets:
  key-file: ~/.config/hick/key.txt
  secrets-dir: ~/.config/hick/secrets
```

The config file is auto-discovered by walking up from the current directory.
Use `--config <PATH>` to specify an explicit path. CLI options override
config file values.

---

## 27. Watch Mode

The `up` subcommand watches `.hick` files for changes and re-runs the
pipeline automatically, with merge-aware snapshots to handle concurrent
edits.

```bash
hick up docs/guide.hick
```

Watch mode supports the same `--param`, `--features`, and `--dry-run`
options as `run`. Additional options:

| Option | Default | Description |
|--------|---------|-------------|
| `--store <BACKEND>` | `auto` | Store backend: `auto`, `git`, or `builtin` |
| `--merge-api <URL>` | | LLM merge API URL for intelligent conflict resolution |
| `--branch <NAME>` | `main` | Branch name for snapshot tracking |
| `--max-stages <N>` | `3` | Maximum number of multi-stage pipeline passes |

---

## 28. Provenance Tracking

Hick tracks the origin of every character in generated output, mapping it
back to the `.hick` source position that produced it. This is used by
[hick-lens](lens-reference.md) and [hick-mcp](mcp-reference.md) to enable
AI agents to navigate and edit `.hick` files structurally.

### Source spans

Every text node and tag in the AST carries a `SourceSpan` recording its
byte offset, line, and column in the source file. These spans flow through
the reactive pipeline via `SpanNode` wrappers.

### Origin types

| Origin | Description |
|--------|-------------|
| `Literal` | Direct text from a `.hick` source file, with exact byte range |
| `Exec` | Output from a container execution |
| `Paste` | Content inserted via `<hick:paste>` |
| `Variable` | Value of a `<hick:val>` reference |
| `Substitution` | Text modified by `<hick:substitute>` |
| `Synthetic` | Programmatically generated (no source mapping) |

### Provenance map

After convergence, a `ProvenanceMap` records the mapping from output byte
ranges to source origins. This enables:

- **hick-lens**: tree-sitter AST analysis of output files, with structural
  selectors (`fn main`, `struct Config`) mapped back to `.hick` source
  positions for targeted edits
- **hick-mcp**: MCP server exposing provenance-aware query, structure,
  and replace tools for AI coding agents

See [Lens Reference](lens-reference.md) and [MCP Reference](mcp-reference.md)
for details.

---

## Tag Reference Summary

| Tag | Self-closing | Parent | Purpose |
|-----|-------------|--------|---------|
| `<hick:doc>` | No | Root | Document root |
| `<hick:container>` | No | `doc` | Declare a container with capabilities |
| `<hick:allow>` | Yes | `container`, `fork`, `attenuate` | Grant access (network or file) |
| `<hick:deny>` | Yes | `container`, `fork`, `attenuate` | Deny access (network) |
| `<hick:secret>` | Yes | `container` | Inject a secret as env var |
| `<hick:exec>` | No | `doc`, `file` | Execute commands in a container |
| `<hick:volume>` | Yes | `doc` | Declare a shared volume |
| `<hick:file>` | No | `doc` | Declare an output file |
| `<hick:copy>` | No | `doc`, `exec` | Store content (visible in output) |
| `<hick:cut>` | No | `doc` | Store content (hidden from output) |
| `<hick:paste>` | Yes | `file`, `exec` | Insert stored content by id or class |
| `<hick:fork>` | No | `doc` | Clone a container's state |
| `<hick:attenuate>` | No | `doc` | Add restrictions to a container |
| `<hick:confirm>` | Yes | `doc` | Pause for user approval (planned) |
| `<hick:feature>` | Yes | `doc` | Define a feature flag |
| `<hick:verify>` | Yes | `doc` | Embedded verify command for generate-matrix |
| `<hick:when>` | No | `doc`, `file` | Conditional content based on variables/features |
| `<hick:var>` | No | `doc` | Declare a variable with a default value |
| `<hick:val>` | Yes | `file` | Output a variable's value |
| `<hick:substitute>` | Yes | `doc` | Register a text substitution |
| `<hick:exclude>` | Yes | `doc` | Register a glob pattern for output exclusion |
| `<hick:include>` | Yes | any | Splice another `.hick` file's contents (resolved at parse time) |

## Attribute Reference Summary

| Attribute | Tags | Values |
|-----------|------|--------|
| `name` | `container`, `volume`, `secret`, `feature`, `var`, `val`, `substitute` | Identifier string |
| `image` | `container`, `exec` | OCI image name (e.g., `alpine:3.20`) |
| `container` | `exec`, `attenuate` | Container name reference |
| `path` | `file` | Output file path |
| `mount` | `exec` | `volume:mountpoint` (comma-separated) |
| `id` | `copy`, `cut` | Unique content identifier |
| `class` | `copy`, `cut` | Space-separated CSS class names |
| `select` | `paste` | CSS `#id` or `.class` selector |
| `separator` | `paste` | String inserted between blocks for `.class` selectors |
| `min` | `paste` | Minimum number of matching blocks required |
| `max` | `paste` | Maximum number of matching blocks allowed |
| `from` | `fork`, `secret` | Source container name or secret name |
| `to` | `fork` | Target container name |
| `network` | `allow`, `deny` | `host:port`, `*:port`, `host:*`, or `*` |
| `file-read` | `allow` | File path pattern |
| `file-write` | `allow` | File path pattern |
| `message` | `confirm` | Display message string |
| `description` | `feature`, `verify` | Human-readable description |
| `requires` | `feature` | Space-separated list of dependent features |
| `conflicts_with` | `feature` | Space-separated list of conflicting features |
| `exclusive_group` | `feature` | Group name (only one from group can be enabled) |
| `command` | `verify` | Command string in `[condition] command` format |
| `test` | `when` | Condition expression to evaluate |
| `when` | any tag | Condition expression (tag removed if false) |
| `show` | `exec` | Output control: `all`, `command`, `output`, `none` |
| `pattern` | `substitute`, `exclude` | Text pattern or glob pattern |
| `value` | `substitute` | Replacement text |
| `variants` | `substitute` | `"true"` to generate case variants |
| `file` | `include` | Relative path to `.hick` file to include |
| `weave` | `doc` | Output path for literate programming markdown |
| `doc-hidden` | `file` | `"true"` to exclude from weave output |
