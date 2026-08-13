# Continuous Delivery — downloadable product

This repository ships a **program people download**, not a service we operate.
There is no staging environment, no production environment, no promote gate,
and no infrastructure to provision — because there is no server. See
`docs/specs/freeform/local-only.md`.

This module **replaces** `continuous-delivery-shared`,
`continuous-delivery-paas`, and `start-with-production`, all of which describe
delivering a cloud service. Do not re-enable them. If a change seems to need
one of them, the change is reintroducing a backend, which is out of scope by
decision rather than by sequencing.

## Branch policy

- There is **always** a long-lived **`master`** branch.
- There is **never** a `production`, `main`, or `staging` branch.
- Never enable GitHub Actions jobs on arbitrary branches; they exist on
  `master` or on PRs into `master`.

## Channels

| Workflow `name:` (Actions UI) | Trigger | What it does |
|-------------------------------|---------|--------------|
| **Unstable Release** | Push to `master` | Builds every target and publishes a prerelease. Automatic — this is the channel that proves the build works. |
| **Trigger Stable Release** | `workflow_dispatch` | Human-chosen version bump (`patch`/`minor`/`major`), starts Stable Release. |
| **Stable Release** | Started by the trigger workflow | Builds, verifies the artifacts, publishes the release, stamps an immutable `vX.Y.Z` tag. |

Use the word **unstable** — not "edge", not "prerelease" — for the non-stable
channel in anything a user reads. GitHub's `prerelease: true` is a mechanic,
not the name.

**Release language, not promote language.** Promote/deploy naming belongs to
cloud products; this repo has neither.

## Rules

- **The artifact is the deliverable, so the artifact is what CI verifies.**
  A release build must be unpacked and exercised on each target before it is
  published — at minimum `hick --version`, `hick --help`, and executing a real
  document from the bundled `examples/`. A binary that builds but cannot run a
  document is a failed release, and the only place to catch that is here.
  Guarantee: `docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md`.
- **A download must work with no toolchain.** Never assume the user has Rust,
  Node, or a package manager. The archive carries the binary, the licence, and
  `examples/`.
- **Every platform we advertise is built and smoke-tested in CI.** If a target
  is not verified by a workflow, it is not a supported platform and must not
  appear in the installer or the docs.
- **The installer and the release assets are one contract.** `scripts/install.sh`
  resolves an asset name that `scripts/dist.sh` produces. Changing either name
  breaks installation for everyone; change them together, and keep the version
  the binary reports equal to the version in the asset name.
- **Nothing in the product phones home.** No telemetry, no update check, no
  licence check, no crash reporting. A tool that runs on private repositories
  earns that by not talking to anyone.
- **The site is static.** hickorydocs.com is files and a link to GitHub
  releases. It has no backend, no signup, and no pricing. If a change to the
  site needs a server, it is out of scope.
- **No monetization surface.** No `plans.json`, no payment provider, no
  entitlements, no feature gates, no upgrade prompts. This is a product
  decision recorded in `local-only.md`, not an unimplemented feature.

## Configuration

Because there is no deployment, there are no GitHub Environments, no
staging/production secret parity, and no `env-parity` check. The only secrets
are what publishing a release needs (the default `GITHUB_TOKEN`, plus signing
credentials if code signing is added).

Runtime configuration is the **user's**, read from their environment at
startup, and must still be strongly typed and validated at boot per
`config-and-environments` — that rule survives the removal of the deployment,
because a badly-typed `HICKORY_EXECUTOR` fails on a user's machine where nobody
can debug it for them.

## Versioning

- Semantic versioning, starting from `0.x` — major and minor may be zero.
- A stable release stamps an immutable tag. Never move a published tag.
- **Rolling back means publishing a newer version**, not deleting an old one.
  Someone has already downloaded it.
