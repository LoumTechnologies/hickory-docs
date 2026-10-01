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
