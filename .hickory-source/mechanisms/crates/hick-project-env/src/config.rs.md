<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-project-env/src/config.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-16e0caa4d632"></a>
## `crates/hick-project-env/src/config.rs`

Discover workspace environments and publish evaluated project entries through the host filesystem boundary.

<a id="f-16e0caa4d632-Config"></a>
### `Config`

External package-manager configuration, read once into typed fields.

<storycode:copy id="f-16e0caa4d632-Config">
//! External package-manager configuration, read once into typed fields.
use std::{ffi::OsString, path::PathBuf, sync::OnceLock};

pub struct Config {
    pub executable_path: OsString,
    pub uv_project_environment: Option<PathBuf>,
}

</storycode:copy>
<a id="f-16e0caa4d632-impl-Config"></a>
### `impl-Config`

<storycode:copy id="f-16e0caa4d632-impl-Config">
impl Config {
    fn from_env() -> Self {
        Self {
            executable_path: std::env::var_os("PATH").unwrap_or_default(),
            uv_project_environment: std::env::var_os("UV_PROJECT_ENVIRONMENT")
                .filter(|path| !path.is_empty())
                .map(PathBuf::from),
        }
    }
}

</storycode:copy>
<a id="f-16e0caa4d632-config"></a>
### `config`

<storycode:copy id="f-16e0caa4d632-config">
pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(Config::from_env)
}
</storycode:copy>

### Assemble `crates/hick-project-env/src/config.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-project-env/src/config.rs"><storycode:paste select="#f-16e0caa4d632-Config"/><storycode:paste select="#f-16e0caa4d632-impl-Config"/><storycode:paste select="#f-16e0caa4d632-config"/></storycode:file>

</storycode:doc>
