<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/examples/fskit_live.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6599354add33"></a>
## `crates/hickory-cli/examples/fskit_live.rs`

<a id="f-6599354add33-main"></a>
### `main`

Exercise a signed bundled app. No development extension or protocol mock.

<storycode:copy id="f-6599354add33-main">
//! Exercise a signed bundled app. No development extension or protocol mock.
use anyhow::{Context, Result, ensure};
use std::{path::PathBuf, process::Command};
fn main() -> Result<()> {
    let app = PathBuf::from(std::env::var_os("HICKORY_FSKIT_APP").context("set HICKORY_FSKIT_APP to a signed Hickory Docs.app containing the enabled workspace extension")?);
    let name = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleExecutable"])
        .arg(app.join("Contents/Info.plist"))
        .output()?;
    ensure!(
        name.status.success(),
        "cannot read the app's executable name"
    );
    let name = String::from_utf8(name.stdout)?.trim().to_string();
    let status = Command::new(app.join("Contents/MacOS").join(name))
        .arg("--hickory-workspace-smoke")
        .status()?;
    ensure!(
        status.success(),
        "native filesystem smoke failed; inspect the app's error above"
    );
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/examples/fskit_live.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/examples/fskit_live.rs"><storycode:paste select="#f-6599354add33-main"/></storycode:file>

</storycode:doc>
