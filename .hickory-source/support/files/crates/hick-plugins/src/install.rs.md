<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/install.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3d6e510529334e316252ed5ed305a02a1234a46b874ab68cf910e4d26e4fb17f`.

<book:entry path="crates/hick-plugins/src/install.rs">
/// One way to install one language's tool.
#[derive(Debug, Clone, Copy)]
pub struct Installer {
    pub language: &'static str,
    /// The tool that must exist on the machine for this to be possible.
    ///
    /// For an [`Installer::assets`] install this is the fetcher (`curl`); the
    /// unpacker is named per-asset, because a `.tar.gz` and a `.zip` do not
    /// need the same program.
    pub tool: &'static str,
    /// The package installed, named for the report.
    pub package: &'static str,
    /// The shell command, with `{prefix}` replaced by the install directory.
    ///
    /// Empty when this installs from [`Installer::assets`] instead.
    pub command: &'static str,
    /// Prebuilt archives, one per platform, when the tool is not published
    /// through any package manager.
    ///
    /// This is the shape `netcoredbg` needs and `csharp-ls` does not: a
    /// `dotnet tool` is one command on every platform, while a release
    /// archive is a different URL per target and a checksum that has to be
    /// pinned, because "download and run whatever is at this URL" is not
    /// something to do on somebody's machine.
    pub assets: &'static [Asset],
    /// Why this package and not another, for the person reading the report.
    pub reason: &'static str,
}

/// One platform's prebuilt archive, pinned.
#[derive(Debug, Clone, Copy)]
pub struct Asset {
    /// `std::env::consts::OS`.
    pub os: &'static str,
    /// `std::env::consts::ARCH`.
    pub arch: &'static str,
    pub url: &'static str,
    /// SHA-256 of the archive, checked before anything is unpacked.
    ///
    /// A pin, not a signature: it proves the bytes are the ones somebody
    /// looked at when this line was written, and proves nothing about
    /// whether those bytes are trustworthy. Upstream publishes no
    /// signatures, so this is the strongest available statement and it is
    /// worth being precise about which one it is.
    pub sha256: &'static str,
    /// The directory under the prefix to unpack into, or empty for the
    /// prefix itself. An archive that carries a top-level directory of its
    /// own (netcoredbg's `netcoredbg/`) needs nothing; one that does not
    /// (a `.vsix` is a zip of `extension/…`) would otherwise scatter its
    /// contents across every other adapter's home.
    pub into: &'static str,
    /// The program that unpacks it — `tar` or `unzip`.
    pub unpack: &'static str,
}
</book:entry>

</book:doc>
