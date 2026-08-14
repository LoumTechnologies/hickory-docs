# Local mode: `.hick` documents in your own git repo

*For engineers who want executable documents in a repository they already
own — plain files, plain git, CI-style verification — instead of (or before)
a hosted workspace.*

## The two storage modes

| | Cloud workspace | Local git repo (this doc) |
|---|---|---|
| Where documents live | Per-project server-side git, mirrored to Postgres | `.hick` files in your repo |
| Editing | Web/mobile notebook UI with live CRDT sync (multiple cursors, offline merge) | Your editor + `hick-lsp` ([editor setup](editor-setup.md)) — or the same notebook UI, hosted by your own machine (`hick serve`, below) |
| Execution | Firecracker microVMs on Cloud Canopy nodes | `hick` CLI, executor of your choice (below) |
| Drift gate | Server re-checks on save/run | `hick init` pre-commit hook + `hick test` in CI |
| Agent | Built-in Agent panel / `hick agent` | Either the built-in agent or your own coding agent ([ai-agents](ai-agents.md)) |

The document format is identical in both. A repo can graduate to a cloud
workspace later (the server's durable state is also git), and cloud projects
can be cloned down.

## Set up a local repo

```sh
curl -fsSL https://raw.githubusercontent.com/LoumTechnologies/hickory-docs/master/scripts/install.sh | sh
cd your-repo
hick init
```

`hick init` is idempotent — run it again any time. It:

1. installs a **pre-commit hook** (a sentinel-delimited `### HICKORY ###`
   block, appended to any hook you already have; `core.hooksPath` is
   respected). At commit time the hook discovers all tracked `*.hick` files
   and runs `hick test` on each; any drift blocks the commit. No `.hick`
   files, no-op.
2. adds `.hick-cache/` to `.gitignore` (cached transcripts).
3. writes a managed `<!-- HICKORY -->` section into `AGENTS.md` teaching
   coding agents the hick grammar and the golden rules (edit sources, run
   `hick run` then `hick test`), and points `CLAUDE.md` at it.
4. registers the `hick` MCP server in `.mcp.json`, so a harness that reads it
   (Claude Code does) gets the document tools without per-developer setup.
5. wires up the editor: it writes `.hick-lsp.json` from the language servers
   this repository's editor config already names, and registers `hick-lsp` for
   `*.hick` in `.helix/languages.toml` and `.vscode/settings.json` where those
   files can do the job. See [editor-setup](editor-setup.md).
6. prints a toolchain doctor: warnings (non-fatal) for missing child language
   servers like `rust-analyzer` or `pyright-langserver`.

Daily loop:

```sh
$EDITOR docs/quickstart.hick
hick run docs/quickstart.hick    # execute, weave quickstart.md, write outputs
hick test docs/quickstart.hick  # verify — same command the hook runs
git add -A && git commit            # hook re-checks every tracked .hick doc
```

## Editing with any editor: `hick up`

`hick run` is a one-shot. `hick up` is the same thing, kept going:

```sh
hick up docs/          # weave everything, then watch. Ctrl-C to stop.
```

It writes every document's output files and then watches the folder. Change a
document and its outputs are rewritten. **Change one of the generated files
and the change lands back in the document it came from** — so you can open
`analysis.py` in whatever editor you already use, edit it as an ordinary
Python file, and the `.hick` document it lives in is what actually changes.

Some of a generated file comes from the document and some of it does not.
Command output, transcripts, and interpolated values have no source to carry
an edit back to, so:

- A file with **nothing** editable in it — a woven report, a pure transcript —
  is marked **read-only** while `hick up` runs. Your editor will say so when
  you open it. The mark comes off when `hick up` exits.
- A file that mixes the two stays writable. If a save touches generated text,
  that save is refused, the file is put back, and the message names the line
  and points you at the document.

By default nothing is executed: cells are answered from the transcripts in
`.hick-cache/`, and a cell with no recording is marked never-run. Add `--run`
to execute a changed document in full on every save:

```sh
hick up docs/ --run    # re-execute on every change
```

Use `--run` when the cells are quick and you want the numbers live; leave it
off when they are slow, or when you would rather not run code every time you
hit save. One `hick up` at a time per folder — a second one refuses to start,
because two of them would each mistake the other's writes for your edits.

## Choosing an executor

`hick run`/`check` execute each `h:exec` block through an executor,
selected by the `HICKORY_EXECUTOR` environment variable:

- **`HICKORY_EXECUTOR=sandbox` (the default).** Each cell runs confined:
  it may write only its own working directory, its `$HOME` is empty apart
  from your toolchains (bound read-only, so `node`, `python`, `cargo` and
  friends still work while your dotfiles, keys and `.env` files are not
  there), and it has **no network unless the document declares one**:

  ```xml
  <hick:container name="lab" image="python:3.12">
    <hick:allow network="pypi.org:443" />
  </hick:container>
  ```

  Enforcement is bubblewrap on Linux, Seatbelt on macOS, AppContainer on
  Windows. Where none of them is available the run is **refused** rather than
  quietly falling back — see below. `image=` is recorded but ignored: the
  cell uses your host tools.
- **`HICKORY_EXECUTOR=local`.** The same thing unconfined: commands run as
  ordinary host processes with your permissions, like `make`. Fast, zero
  setup, works anywhere — and a document you did not write gets your files,
  your keys and your network. Reasonable for documents you wrote; a decision
  worth making deliberately for anything else.
- **`HICKORY_EXECUTOR=canopy`.** Each container becomes a Firecracker microVM
  on a [Cloud Canopy](https://github.com/LoumTechnologies/cloud-canopy) node —
  the same isolation the hosted platform uses, pointed at a node you run
  yourself. Configure the node endpoint and capability token per the
  `hickory-executor-canopy` crate docs.

## Platform reality for `canopy` locally

Firecracker requires **Linux with KVM**. Local sandboxed execution is
**Linux-first today**; everything else below is a workaround or future work.

- **Linux**: install cloud-canopy, verify `/dev/kvm` exists, run a node.
  Works today.
- **macOS**: no Firecracker. Options, in order of practicality:
  - Run a thin Linux VM with [Lima](https://lima-vm.io/) and host the canopy
    node inside it (nested virtualization needs Apple Silicon M3 or newer on
    macOS 15+ for KVM-in-VM to work).
  - Future: a libkrun-style backend for canopy using Hypervisor.framework
    directly — planned, not built.
  - Or just use `HICKORY_EXECUTOR=local` (unsandboxed) or a cloud workspace.

## When nothing can confine a cell

`hick` refuses to run rather than running unconfined, and says what to
install: bubblewrap on Linux (`sudo apt install bubblewrap`), nothing on
macOS (Seatbelt ships with it), Windows 8 or later for AppContainer. The
refusal names `HICKORY_EXECUTOR=local` as the way to proceed anyway. This is
deliberate: a fallback you did not notice is a safety claim that is false.
- **Windows**: run the canopy node inside **WSL2** with nested virtualization
  enabled (KVM inside WSL2 works on current Windows 11 builds). Point
  `hick` on either side at it.

If you only need verification — not isolation — `local` is fine on all three
platforms.

## Work on it together, from your machine

```sh
hick serve docs/tour.hick              # just you: opens on 127.0.0.1
hick serve docs/tour.hick --share      # + a link for people on your network
```

The second form prints two URLs: yours, and one to send to whoever you are
working with. They open it in a browser — no account, no install — and you are
editing the same document live, with cursors, in the same notebook UI
hickorydocs.com serves.

What makes this different from a hosted workspace is where the state lives:

- **Your file is the document.** Every edit anyone makes lands in
  `docs/tour.hick` on your disk within a second, so your editor, `git diff`,
  and `hick test` see your collaborator's work as ordinary changes.
- **Your machine runs the code.** Cells execute through your executor, on your
  hardware, under your rules.
- **The session is yours to end.** Ctrl-C and the link stops working. Nothing
  outlives the process.

You also get the **lineage ribbons** — the split view threading each generated
byte back to the document span that produced it — for a file on your own disk,
which is the one thing the CLI alone cannot show you.

### Who can do what

`--scope` decides what the *link* allows. You always have full control of your
own session; the scope governs the people you sent the link to.

| Scope | A guest may |
| --- | --- |
| `read` | read the document, its outputs, and the lineage |
| `edit` (default) | …and change the document |
| `run` | …and execute it on your machine |

`--scope run` is refused for a shared session on the local executor, and says
so: the local executor runs commands as you, with your files and your network,
so a forwarded link would be a shell on your machine. Sandbox it first:

```sh
HICKORY_EXECUTOR=docker hick serve docs/tour.hick --share --scope run
```

### Someone who isn't on your network

`--public` publishes an address that reaches further:

```sh
hick login --signup                          # once: email and password
hick serve docs/tour.hick --share --public
```

That prints a link anyone can open, forwarded to your machine by our relay.
The relay carries bytes and nothing else: it never stores your document, never
runs it, and keeps nothing after you press Ctrl-C.

**Why signing in is required for this and nothing else.** A relay is a byte
forwarder pointed at the internet. Anonymous, it becomes a free tunnel service
for whatever a stranger wants to expose — so a tunnel is tied to an account,
which is what makes a quota enforceable and misuse attributable. Three tunnels
at a time, eight hours each.

`hick login` asks the relay how it lets people in. Email and password always
works. If that relay has a GitHub app configured, `hick login --github` uses
device flow instead — you type a code in a browser, nothing is redirected
anywhere, and **no scopes are requested** (it can read your public profile and
nothing else). If the relay has no GitHub app, the option is not offered,
because an option that fails at the last step is worse than one you never saw.

Prefer your own tunnel? `--public` uses it if you have one:

```sh
# any tunnel you already run — Cloudflare, ngrok, Tailscale, an SSH reverse tunnel
HICKORY_PUBLIC_URL=https://your-tunnel.example hick serve doc.hick --share --public

# or PortZero, which its daemon picks up from the environment at launch
PZ_TUNNEL=hick hick serve doc.hick --share --public
```

With none of the three, `--public` refuses rather than printing a link that
cannot open.

### If the editor does not appear

`hick serve` needs the built web client. It looks at `--web-dist`, then
`HICKORY_WEB_DIST`, then `apps/web/dist` upwards from the working directory;
with none of them it serves the API only and says so. A binary installed from
a release does not carry the client yet.

## Don't assume

- **The pre-commit hook checks *tracked* `.hick` files, not just staged
  ones** — a doc drifted by someone else's change still blocks your commit,
  which is the point.
- **`hick test` re-executes documents.** A slow document makes commits
  slow; keep heavyweight docs out of the hook by not tracking them, or gate
  them in CI only.
- **`.hick-cache/` is disposable** — never commit it; `hick weave` uses it
  to render without executing.
- **Editing a file in your editor is still just a file.** `hick serve` adds
  live sync for the people connected to *that session*; two people editing the
  same `.hick` in their own editors still merge through git like any other
  file.
- **A share link is a credential.** Anyone it is forwarded to has whatever the
  link's scope allows, it does not expire, and the only way to revoke it is to
  end the session (Ctrl-C) and start a new one.
- **A public link is not end-to-end encrypted.** Traffic is encrypted to the
  relay and again from it, but forwarding means reading: the relay sees the
  bytes as they pass. It keeps none of them, and we would rather say this
  plainly than let "encrypted" imply more than it does.
- **`hick logout` forgets the token on your machine**, which is not the same
  as revoking it: the token the relay issued stays valid until it expires.
