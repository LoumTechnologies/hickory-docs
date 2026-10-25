<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-project-env/src/config.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ff758437480cec19d11ec7723179926af36c1a8d08af613b2df2557125fdc28e`.

<book:entry path="crates/hick-project-env/src/config.rs">
//! External package-manager configuration, read once into typed fields.
use std::{ffi::OsString, path::PathBuf, sync::OnceLock};

pub struct Config {
    pub executable_path: OsString,
    pub uv_project_environment: Option<PathBuf>,
}

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

pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(Config::from_env)
}
</book:entry>

</book:doc>
