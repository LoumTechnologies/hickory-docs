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

impl Installer {
    /// The archive published for the host running Hickory.
    pub fn host_asset(&self) -> Option<&'static Asset> {
        self.assets
            .iter()
            .find(|asset| asset.os == std::env::consts::OS && asset.arch == std::env::consts::ARCH)
    }
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
    /// This check binds the downloaded bytes to the selected release pin.
    /// It does not verify an upstream signature.
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
