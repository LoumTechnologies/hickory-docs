<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/examples/fskit_live.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c2f8e47d247fa24e5e0465e5c155be3622ffab04c651dc47892f3a3ee28420ad`.

<book:entry path="crates/hickory-cli/examples/fskit_live.rs">
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
</book:entry>

</book:doc>
