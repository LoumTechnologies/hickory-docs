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
19. [Planned Features](#19-planned-features)

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

Inserts previously stored content by id selector. Self-closing tag.

```xml
<hick:paste select="#version" />
```

### Attributes

**Copy / Cut:**

| Attribute | Required | Description |
|-----------|----------|-------------|
| `id` | Yes | Unique identifier for the content block |

**Paste:**

| Attribute | Required | Description |
|-----------|----------|-------------|
| `select` | Yes | CSS-style `#id` selector referencing a copy or cut block |

### Resolution

Paste resolves in this order:

1. Cut blocks (checked first)
2. Copy blocks

If no matching block is found, a warning is logged and the paste produces
empty output.

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
hick [OPTIONS] <FILES...>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILES...>` | One or more `.hick` files to process (required) |

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--key-file <PATH>` | `~/.config/hick/key.txt` | Age identity file for decrypting secrets |
| `--secrets-dir <PATH>` | `~/.config/hick/secrets` | Directory containing age-encrypted secret files |
| `--images-dir <PATH>` | `crates/c2w/converted_images` | Directory containing pre-converted `.wasm` container images |
| `--dry-run` | `false` | Produce placeholder transcripts instead of running containers |
| `-v`, `--verbose` | `false` | Enable verbose (debug-level) logging |

### Examples

```bash
# Dry-run: parse, validate, and generate file outputs without containers
hick --dry-run docs/guide.hick

# Live execution with default settings
hick docs/guide.hick

# Multiple files processed in sequence (shared state)
hick docs/setup.hick docs/build.hick docs/deploy.hick

# Custom secrets and images directories
hick --key-file /etc/hick/key.txt \
     --secrets-dir /etc/hick/secrets \
     --images-dir /opt/wasm-images \
     docs/deploy.hick

# Verbose logging for debugging
hick -v docs/guide.hick
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
| `--dry-run` | Show combinations without generating |

---

## 19. Planned Features

These features are designed but not yet implemented.

### Variables and parameters

Document-level variables with CLI override:

```xml
<hick:var name="python_version">3.12</hick:var>

<hick:file path="setup.md">
Python version: <hick:val name="python_version" />
</hick:file>
```

CLI override: `hick --param python_version=3.13 docs/guide.hick`

### Conditional content

Skip content based on variable values:

```xml
<hick:exec container="deploy" when="env=prod">
./deploy-prod.sh
</hick:exec>

<hick:when test="env=prod">
Production-only content here.
</hick:when>
```

Condition syntax: `when="var"`, `when="var=value"`, `when="var!=value"`,
`when="!var"`.

### Exec output control

Control what is rendered in file outputs:

```xml
<hick:exec container="demo" show="command">make build</hick:exec>
<hick:exec container="demo" show="output">cat result.txt</hick:exec>
<hick:exec container="demo" show="none">rm -rf /tmp/cache</hick:exec>
```

Values: `all` (default), `command`, `output`, `none`.

### File includes

Include content from other `.hick` files:

```xml
<hick:include file="fragments/setup.hick" />
```

### Caching and freeze

Cache exec results based on a hash of the command, image, and capabilities:

```bash
hick --cache docs/guide.hick    # Use cache on hash match
hick --freeze docs/guide.hick   # Fail if cache is missing
hick --clear-cache docs/guide.hick  # Clear cache and run fresh
```

### Project configuration

A `_hick.yml` file at the project root for default settings:

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
| `<hick:paste>` | Yes | `file`, `exec` | Insert stored content by id |
| `<hick:fork>` | No | `doc` | Clone a container's state |
| `<hick:attenuate>` | No | `doc` | Add restrictions to a container |
| `<hick:confirm>` | Yes | `doc` | Pause for user approval (planned) |
| `<hick:feature>` | Yes | `doc` | Define a feature flag |
| `<hick:when>` | No | `doc`, `file` | Conditional content based on features |

## Attribute Reference Summary

| Attribute | Tags | Values |
|-----------|------|--------|
| `name` | `container`, `volume`, `secret`, `feature` | Identifier string |
| `image` | `container`, `exec` | OCI image name (e.g., `alpine:3.20`) |
| `container` | `exec`, `attenuate` | Container name reference |
| `path` | `file` | Output file path |
| `mount` | `exec` | `volume:mountpoint` (comma-separated) |
| `id` | `copy`, `cut` | Unique content identifier |
| `select` | `paste` | CSS `#id` selector |
| `from` | `fork`, `secret` | Source container name or secret name |
| `to` | `fork` | Target container name |
| `network` | `allow`, `deny` | `host:port`, `*:port`, `host:*`, or `*` |
| `file-read` | `allow` | File path pattern |
| `file-write` | `allow` | File path pattern |
| `message` | `confirm` | Display message string |
| `description` | `feature` | Human-readable feature description |
| `requires` | `feature` | Space-separated list of dependent features |
| `conflicts_with` | `feature` | Space-separated list of conflicting features |
| `exclusive_group` | `feature` | Group name (only one from group can be enabled) |
| `test` | `when` | Feature expression to evaluate |
