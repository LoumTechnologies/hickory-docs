# Installing `hickory`

For someone who wants to run executable documents and does not care how the
project is built. No Rust toolchain is needed at any point.

## One line

```sh
curl -fsSL https://raw.githubusercontent.com/LoumTechnologies/hickory-docs/master/scripts/install.sh | sh
```

It works out which archive matches your machine, checks its SHA-256 against
the checksum published beside it, and installs `hickory` to `~/.local/bin`. If
that directory is not on your `PATH`, the script says so and prints the line
to add.

Knobs, all environment variables so the piped form still works:

| Variable | Default | What it does |
|---|---|---|
| `HICKORY_CHANNEL` | `stable` | `unstable` installs the build cut from the last green `master` instead |
| `HICKORY_VERSION` | — | An exact tag, e.g. `v0.1.0`. Overrides the channel |
| `HICKORY_INSTALL_DIR` | `~/.local/bin` | Where the binary goes |
| `HICKORY_GITHUB_TOKEN` | — | A GitHub token, needed only while this repository is private |

```sh
curl -fsSL …/install.sh | HICKORY_CHANNEL=unstable sh
```

## By hand

Every archive is on the [releases
page](https://github.com/LoumTechnologies/hickory-docs/releases).

| Platform | Archive |
|---|---|
| macOS, Apple Silicon | `hickory-<version>-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `hickory-<version>-x86_64-apple-darwin.tar.gz` |
| Linux, x86_64 | `hickory-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux, aarch64 | `hickory-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| Windows, x86_64 | `hickory-<version>-x86_64-pc-windows-msvc.zip` |

Each carries a `.sha256` beside it. Verify before you run it:

```sh
sha256sum -c hickory-<version>-<target>.tar.gz.sha256
```

Unpack it and you get the binary, `LICENSE`, `README.md`, and `examples/`:

```sh
tar -xzf hickory-<version>-<target>.tar.gz
cd hickory-<version>-<target>
./hickory test examples/text-tools-tour.hick
```

## Two channels

- **Stable** — an immutable `vX.Y.Z` tag, cut deliberately. This is what the
  installer picks by default and what `/releases/latest` answers.
- **Unstable** — one rolling `unstable` tag, replaced by every green build of
  `master`. Versions look like `0.1.0-unstable.a1b2c3d4`, so a bug report off
  this channel names the commit it came from. No compatibility promise.

## What each platform needs

The binary itself needs almost nothing. The Linux builds are made on Ubuntu
22.04 and link against glibc 2.35, so they run on Ubuntu 22.04, Debian 12,
RHEL 9, and anything newer. On an older distribution — or a musl one such as
Alpine — build from source (below); there is no musl artifact. The macOS
builds target macOS 11 and later.

Running a document is a different question, because local execution runs each
cell against **your** host toolchain — `image=` is recorded and ignored (see
the README).

- **`sh`.** Every cell is executed through a POSIX shell. macOS and Linux have
  one. On Windows, use Git Bash or WSL; `hickory --version`, `--help`, and the
  parser work without one, but executing a document does not.
- **Whatever the document invokes.** `examples/text-tools-tour.hick` needs
  `sort`, `awk`, `wc`, and `tr` — all POSIX, all present on macOS and Linux.
  `examples/bootstrap-ci.hick` needs `python3`.
  `examples/grand-tour.hick` needs `python3` with `polars` and the `duckdb`
  CLI, and its R chapter — off unless you pass `--features with-r` — needs
  `Rscript` with `ggplot2`. See [examples/README.md](../../examples/README.md).

A missing tool shows up as the cell failing with the shell's own "command not
found", naming the line in the document that invoked it.

## From source

```sh
git clone https://github.com/LoumTechnologies/hickory-docs
cd hickory-docs
cargo build --release -p hickory-cli   # binary at target/release/hickory
```

This is also the answer for any platform not in the table above: musl distros,
32-bit machines, BSDs, or a glibc older than 2.35. The build needs a C
compiler as well as Rust — `ring`, `libsodium`, `zstd`, `bzip2`, and `lzma`
are compiled from source as part of it — which on a Debian-family system means
`build-essential`.

## Uninstalling

```sh
rm ~/.local/bin/hickory
rm -rf ~/.local/share/hickory
```
