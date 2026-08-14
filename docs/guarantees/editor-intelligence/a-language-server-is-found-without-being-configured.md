# A Language Server Is Found Without Being Configured

Given a `hick:file` block whose `path` names a language — `app.py`,
`src/main.rs`, `index.ts` — when `hick-lsp` needs a server for that block,
then it finds one already installed on the machine and uses it, with no
configuration written by anyone. Nothing in the product asks a user which
language server to use, and a repository with no `.hick-lsp.json` gets the
same intelligence as one with a hand-tuned file.

The routing key is the file extension, not a declaration in the document.
A block's path is the same string that names the file it generates, so the
block and the generated file are analysed by the same server for the same
reason `hick init` adopts the project's own choice — see
[hick-init-adopts-the-projects-language-servers](hick-init-adopts-the-projects-language-servers.md).

## Where it looks, in order

1. **An explicit override** in `.hick-lsp.json`. A human decision outranks
   anything found, always.
2. **The project.** `node_modules/.bin`, a virtualenv's `bin` (identified by
   `pyvenv.cfg`, not by being called `.venv`), `vendor/bin`. A repository that
   pins a server version means that version, not whatever is on `PATH`.
3. **The machine.** Rustup's toolchain directories, then `PATH`, then the
   places installers use without putting anything on `PATH`: `~/.local/bin`,
   `~/.cargo/bin`, `~/go/bin`, Mason's registry
   (`~/.local/share/nvim/mason/bin`).

   The toolchains come before `PATH`, and they are the only thing that does.
   `~/.cargo/bin/rust-analyzer` is on everyone's `PATH` and is not a language
   server: it is a rustup shim that re-dispatches to whichever toolchain the
   current directory pins, and it exits with `Unknown binary 'rust-analyzer'`
   whenever that toolchain lacks the component — which is every repository
   with a `rust-toolchain.toml`, including this one. Searching the toolchain
   first finds the binary that actually runs. Nothing but rustc's own tools
   lives there, so it cannot shadow another language's server.

Each language has several candidates in preference order (Python: `pyright`,
then `pylsp`, then `pyls`; Rust: `rust-analyzer` including the rustup shim),
so "the one you have" is usually the answer without a preference being
expressed.

## What it never does

- **Never installs anything.** Discovery is a search of what is present. A
  language with no server installed produces no server, and the document's
  other blocks are unaffected — `hick-lsp` degrades rather than failing, per
  [lsp-channel-degrades-never-errors](lsp-channel-degrades-never-errors.md).
- **Never reaches the network.** The search is filesystem-only, so it works
  on a machine that has never had internet.
- **Never overrides a person.** `.hick-lsp.json` wins, unconditionally.

`hick init` reports what it found rather than what is missing —
`Rust: /home/you/.cargo/bin/rust-analyzer (machine)` — so a user can see the
exact binary that will analyse their blocks, including which of the three
sources it came from.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `crates/hick-lsp/src/discovery.rs` — `discover(language, root)` searches
    `project_dirs(root)` before `machine_dirs()` and returns `Discovered`
    with a `command` and an `origin` of `"project"` or `"machine"`. Candidate
    tables are per-language `const` slices; the `pyvenv.cfg` check is in
    `project_dirs`, the Mason path in `machine_dirs`, and `toolchain_dirs`
    is spliced in ahead of `PATH`.
  - `crates/hick-lsp/src/child_lsp.rs` — `lsp_command` consults the
    `.hick-lsp.json` override first, then `discovery::discover`, then falls
    back to a bare binary name so an unusual install still spawns if it
    happens to be on `PATH`. It logs the chosen command and origin at INFO.
  - `crates/hickory-cli/src/init.rs` — `discovered_language_servers(root)`
    drives the report; it names the resolved path and origin per language.
  - Unit tests in `discovery.rs`: a project-local binary beats one on the
    machine, a venv is recognised by `pyvenv.cfg`, an unknown language yields
    nothing, the candidate order is preference order, and
    `a_rustup_toolchain_outranks_the_shim_on_path` pins both the ordering and
    its determinism across runs.
  - Observed end to end on this machine: opening a `.hick` document with a
    `src/main.rs` block, in a directory pinning toolchain 1.96.1 (which has
    no `rust-analyzer` component), spawned
    `~/.rustup/toolchains/stable-…/bin/rust-analyzer` rather than the shim,
    and answered `semanticTokens/full` and `foldingRange` from it. A document
    with an `app.py` block spawned `pyright-langserver` from `PATH`. Neither
    directory had a `.hick-lsp.json`.
- Caveats — what LLM review could NOT establish:
  - Only Linux paths were exercised. The macOS and Windows equivalents
    (Homebrew prefixes, `%LOCALAPPDATA%`) are in the table but unobserved,
    and a Windows `.exe` suffix is handled by name only.
  - Discovery runs once per language per process; a server installed while
    `hick-lsp` is running is not picked up until it restarts.
- Test coverage: the `discovery.rs` unit tests above. The end-to-end spawn is
  observed, not automated — see
  [the-meta-lsp-forwards-what-the-child-supports](the-meta-lsp-forwards-what-the-child-supports.md)
  for the same gap on the feature side.
