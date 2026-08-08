---
name: bootstrap-tauri-app
description: >
  Single entry point for creating a brand-new Tauri v2 app: scaffolds a shell composing
  plain Rust crates and React/TypeScript packages ("plugins" without a plugin
  framework), always wires up the $retrofit-tauri-loop feedback loop, and asks the user
  up front whether to add CLI twins for every command (scaffolding that via
  $retrofit-tauri-cli) and whether the app should ship to iOS/Android/desktop-everywhere
  (delegating to $ship-tauri-everywhere). Use when asked to create a new Tauri app,
  start a Rust-backend + React-frontend desktop tool from scratch, or bootstrap a
  portfolio app. For adding these capabilities to an app that already exists, use the
  retrofit skills directly instead: $retrofit-tauri-loop (feedback loop),
  $retrofit-tauri-cli (CLI twins), $ship-tauri-everywhere (mobile/cross-platform).
  Reference implementation ~/Documents/src/fleet.
---

# Bootstrap Tauri App

This is the one place to start a *new* Tauri v2 app. It always produces the shell +
plugin structure below and the `$retrofit-tauri-loop` feedback loop; CLI twins and
mobile/cross-platform support are asked about, not assumed, and delegated to their own
skills so this one stays focused on scaffolding.

If the app already exists, don't use this skill — go straight to the retrofit skill for
the one capability you're adding: `$retrofit-tauri-loop`, `$retrofit-tauri-cli`, or
`$ship-tauri-everywhere`. Each of those detects the app's real shape and adapts,
whether or not it was originally bootstrapped here.

## Setup flow (in order)

1. **Scaffold the structure** — the shell + plugin pattern below. Every new app gets
   this regardless of the answers to steps 3–4.
2. **Always scaffold the feedback loop** — run `$retrofit-tauri-loop` against the
   freshly scaffolded app before writing any real feature. It's cheap when the app is
   brand new (clean `lib.rs`, centralized `invoke`, no messy history to untangle) and
   every feature added afterward gets the fast headless-Chromium loop for free instead
   of retrofitting it later.
3. **Ask: "Should every feature also be callable as a CLI command (for scripts and
   coding agents), or just from the UI?"** If yes, run `$retrofit-tauri-cli` against
   the freshly scaffolded app (again, cheap now — no commands to catch up on yet) and
   keep Wiring Rule 3 below in force for every feature added from here on. If no, skip
   `crates/<name>-cli` entirely and drop Wiring Rule 3 — don't scaffold CLI crates
   nobody asked for.
4. **Ask: "Should this app ship to iPhone/Android too, or stay desktop-only (macOS/
   Windows/Linux)?"** If mobile, hand off to `$ship-tauri-everywhere` — it converts the
   shell to lib+bin, initializes iOS/Android, adds phone-viewport E2E projects to the
   loop from step 2, and sets up cross-platform CI. If desktop-only, stop here; nothing
   further to do for platform reach.
5. Record both answers somewhere durable for future agents touching this app — a line
   in the README or `AGENTS.md` ("CLI twins: yes/no", "Targets: desktop-only /
   everywhere") — so "add a feature" work later doesn't have to re-derive the decision
   from what happens to exist yet.

## The Pattern

A desktop app is a **shell** plus **plugins**, where a plugin is nothing more than:

- `crates/<name>-core` — an ordinary Rust library with the real logic and
  serde-serializable DTOs;
- `crates/<name>-cli` — **only if step 3 above said yes** — a thin binary exposing the
  same core functions on the command line, so AI coding agents and scripts get every
  feature the UI has;
- `ui/<name>` — an ordinary pnpm-workspace React/TypeScript package exporting
  components and TS types that mirror the core crate's DTOs.

The shell (`apps/shell`) is normal Tauri v2 source code that depends on these
crates and packages directly and integrates them by hand: `#[tauri::command]`
functions in `src-tauri/src/main.rs` call core-crate functions; `App.tsx`
imports the UI components and wires them to `invoke`. **The shell source IS
the integration layer.**

Explicitly rejected — do not build these even when they feel "cleaner":
plugin registries, dynamic loading, plugin traits/interfaces, sandboxes,
message buses, or any abstraction whose only purpose is to treat plugins as
objects. In the age of coding agents, editing the shell to add a plugin is
cheaper than maintaining a framework.

## Repo Layout

```
Cargo.toml            # workspace: crates/* + apps/shell/src-tauri
                      # default-members = crates only (fast cargo check;
                      # the tauri dep tree builds only via -p fleet-shell/just dev)
package.json          # pnpm workspaces (pnpm-workspace.yaml): ui/*, apps/shell
pnpm-workspace.yaml   # @tauri-apps/cli devDep
justfile              # setup / check / test / dev / build + CLI shortcuts + e2e (from
                      # $retrofit-tauri-loop, always present)
crates/<name>-core/   # logic + serde DTOs + unit tests
crates/<name>-cli/    # only if CLI twins were requested (step 3) — [[bin]] with a
                      # USAGE string, --json output for agents
ui/<name>/            # main/types = src/index.ts (source package, no build step;
                      # peerDependency react — Vite compiles linked workspace TS directly)
apps/shell/           # Vite + React app: package.json, vite.config.ts, tsconfig,
                      # index.html, src/App.tsx, src/styles.css
apps/shell/src-tauri/ # tauri = "2", tauri-build = "2", build.rs → tauri_build::build(),
                      # tauri.conf.json, capabilities/default.json, src/main.rs
e2e/                  # Playwright specs from $retrofit-tauri-loop, always present
```

## Use pnpm, Not npm

**pnpm is the package manager for every app in this portfolio.** Not a style
preference — npm cannot express a dependency these apps need.

Portfolio apps depend on shared private packages that live in a **subdirectory
of another private repo** (`@pilockdb/react` at `packages/pilockdb-react`).
npm has never supported installing from a subdirectory of a git repo, so the
only npm-compatible options are publishing to a registry or checking the other
repo out beside this one and using a `file:` link. The `file:` approach is what
makes a repo unbuildable on any machine where the sibling isn't laid out on
disk — including CI and every fresh clone.

pnpm supports it directly, over HTTPS, authenticating with whatever GitHub
credential the machine already has:

```json
"@pilockdb/react": "github:LoumTechnologies/pilockdb#path:/packages/pilockdb-react"
```

The lockfile pins the resolved **commit sha**, so this is reproducible even
without release tags:

```yaml
resolution: {commit: 7357e9c5…, path: /packages/pilockdb-react, type: git}
```

No registry, no publish step, no token juggling for local development. The Rust
side uses the matching mechanism — a Cargo `git` dependency over HTTPS with
`git-fetch-with-cli = true` in `.cargo/config.toml` — so one GitHub credential
covers both ecosystems.

Two consequences worth stating up front:

- **pnpm does not flatten `node_modules`.** Any import of a package that isn't
  declared in that package's own `package.json` — working by accident under
  npm's hoisting — fails immediately. This is the migration's only real cost,
  and finding those is a benefit, not a regression.
- **CI still needs a token.** `GITHUB_TOKEN` can't read a *different* private
  repo, so configure a PAT via `url.insteadOf` for git fetches. What goes away
  is needing the sibling repo checked out beside this one.

## Wiring Rules

1. DTOs are defined once, in the core crate, with `#[derive(Serialize)]`;
   each UI package hand-mirrors them as TS interfaces (snake_case field names
   preserved; `Option<T>` → `T | null`). A comment in the TS file names the
   Rust source file it mirrors.
2. Commands use `#[tauri::command(rename_all = "snake_case")]` and the
   frontend invokes with snake_case argument keys — no case-conversion
   guessing.
3. **Only if CLI twins were requested (setup step 3):** every user-facing
   capability gets a CLI twin. The CLI calls the identical core functions the
   command does, supports `--json`, and prints a USAGE string on bad input. No
   logic lives only in the shell.
4. UI packages are presentational: props in, callbacks out, plain classNames.
   `invoke` calls, state, cross-plugin composition, and all CSS live in the
   shell. Composition between plugins (plugin A's badge on plugin B's card)
   is shell code, not a plugin API.
5. Errors from commands return `Result<_, String>` and are rendered visibly
   by the shell, never swallowed.

## Tauri v2 Specifics That Bite

- `tauri.conf.json` schema v2: top-level `identifier`, `build.devUrl` +
  `frontendDist`, `app.windows`. Set `bundle.active = false` until icons
  exist, or builds fail on the icon requirement.
- `capabilities/default.json` needs `"windows": ["main"]` and at least
  `"permissions": ["core:default"]`.
- Linux needs webkit2gtk-4.1 dev packages; add a `just doctor` that checks
  cargo, node, and `pkg-config --exists webkit2gtk-4.1`.
- `beforeDevCommand`/`beforeBuildCommand` should call the pnpm workspace
  scripts so `pnpm tauri dev` is the single entry point.
- Keep tokio/async out of core crates and CLIs; only the shell carries the
  tauri runtime.

## Workflow For "Add A Feature"

1. Decide whether it extends an existing plugin (same domain, same DTO family)
   or is a new plugin pair. Prefer extending; new plugin only for a new domain.
2. Implement and unit-test in the core crate first (`cargo test` stays fast —
   default-members excludes the shell).
3. If the app has CLI twins (setup step 3 said yes), add/extend the CLI twin;
   verify against fixture data on disk.
4. Add the command to `src-tauri/src/main.rs` and the handler list.
5. Mirror new DTO fields in the UI package types; build the component.
6. Compose in `App.tsx`; style in the shell CSS (light and dark via
   `prefers-color-scheme`).
7. Verify with `just e2e` (the `$retrofit-tauri-loop` suite) and `just dev`
   against fixture data, not just unit tests.
