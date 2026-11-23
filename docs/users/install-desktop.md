# Install Hickory Simple

Pick a package whose name starts with `hickory-simple-` from the
[releases page](https://github.com/LoumTechnologies/hickory-docs/releases).
For an Unstable build, follow the rolling release's link to the latest
successful CI build. The version identifies the build used for the CLI and
the editor packages. Each package includes a SHA-256 checksum file.

- **macOS** — Extract the ZIP for Apple Silicon (`macos-arm64`) or Intel
  (`macos-x64`), then move **Hickory Simple.app** to Applications. In its
  connection sheet, choose **Open Local Workspace…** and select a folder or file.
  From a checkout, `just local-install` builds and signs a copy on this Mac.
- **Linux** — Extract the KDE archive, install your distribution's Qt WebEngine
  and KDE Frameworks runtime packages, then run `bin/hickory-simple` from the
  extracted tree.
- **Windows** — Extract the WinForms archive and run `HickorySimple.exe` from
  that folder. Keep the workspace host and web files beside it. The app needs
  the WebView2 Runtime.

The CLI is a separate download. [Install the CLI](install.md) to use
`hick open PATH` or to run, test and weave documents from a terminal.
The [frontend guide](../developers/simple-frontends.md) covers development,
the browser, iOS and Android.
