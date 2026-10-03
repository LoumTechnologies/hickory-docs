# Installing the desktop app

For someone who wants to open a folder of `.md` documents in a window,
rather than run `hick` in a terminal. No Rust toolchain, no Node, no clone.

The desktop app and the CLI are **separate downloads**. They are the same
engine — the app runs the local document server in its own process, so a
document weaves, runs, and carries an output edit back exactly as it does
under `hick up` — but a GUI application is a `.dmg`, an `.msi`, or a
`.deb`/`.AppImage`, and a CLI is a binary on your `PATH`. Install either,
neither, or both.

## Download

From the [releases page](https://github.com/LoumTechnologies/hickory-docs/releases):

| Platform | File |
|---|---|
| macOS, Apple Silicon | `hickory-docs-<version>-aarch64-apple-darwin.dmg` |
| macOS, Intel | `hickory-docs-<version>-x86_64-apple-darwin.dmg` |
| Linux, x86_64 | `hickory-docs-<version>-x86_64-unknown-linux-gnu.deb` or `.AppImage` |
| Windows, x86_64 | `hickory-docs-<version>-x86_64-pc-windows-msvc.msi` |

Each has a `.sha256` beside it:

```sh
sha256sum -c hickory-docs-<version>-<target>.deb.sha256
```

## Unsigned, and what that means for you

The bundles are **not code-signed**. Nobody has paid Apple or a Windows
certificate authority for the privilege, and this is a program you download
from a public releases page rather than an app store.

So the first launch takes an extra step:

- **macOS** — Gatekeeper refuses it: *"Hickory Docs" cannot be opened because
  the developer cannot be verified*. Right-click the app → **Open** → **Open**,
  once. Or `xattr -d com.apple.quarantine "/Applications/Hickory Docs.app"`.
- **Windows** — SmartScreen shows *Windows protected your PC*. **More info** →
  **Run anyway**.
- **Linux** — nothing to clear. `sudo dpkg -i hickory-docs-*.deb`, or
  `chmod +x` the `.AppImage` and run it.

Verifying the `.sha256` is the check that actually means something here: it
proves you have the bytes the release published. A signature would prove the
same bytes came from us, which is the part that is missing until there is a
certificate.

## Markdown file associations

The desktop bundles register Hickory Docs as an editor for `.md` files.
On macOS, `just local-install` also makes it your default Markdown editor,
so double-clicking a `.md` file in Finder opens it in Hickory Docs. Confirm
the switch if macOS asks; reinstalling skips the request if it is already
your default.

For a drag-and-drop `.dmg` installation, choose Hickory Docs in Finder’s
**Get Info → Open with → Change All…** for a Markdown file. On Windows or
Linux, choose it in your system’s default-app settings.

## What it opens

The app opens a folder of `.md` documents, including an empty folder. It picks one
in this order, and says so rather than guessing:

1. `HICKORY_PROJECT_DIR`, or the first command-line argument.
2. The folder you opened last time, if it still exists.
3. Otherwise it asks, with a native folder picker.

It takes the same directory lock `hick up` takes, so opening a folder that a
`hick up` is already watching is refused rather than quietly producing two
processes writing the same files. Stop the other one, or open a different
folder.

## Editor intelligence

The window's editor has the same language support your own editor gets from
`hick-lsp` — hover, definitions, references, completion, and diagnostics from
the real language servers, mapped onto the document. That runs **inside** the
app, so the desktop download needs no `hick-lsp` on your `PATH`.

The child language servers are still yours: install `rust-analyzer`,
`pyright`, or whatever the languages in your documents need, and put them in
`.hick-lsp.json` if this repository has already chosen particular ones. See
[editor setup](editor-setup.md) — the same file serves both.

## Uninstalling

- **macOS** — drag `Hickory Docs.app` to the Trash.
- **Linux** — `sudo apt remove hickory-docs`, or delete the `.AppImage`.
- **Windows** — Settings → Apps → Hickory Docs → Uninstall.

Your documents are your own files in your own repository; removing the app
touches none of them.
