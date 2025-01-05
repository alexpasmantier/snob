use std::{collections::HashSet, path::Path};

use serde::Deserialize;

use crate::snob_debug;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// files-related configuration
    #[serde(default)]
    pub files: FilesConfig,
    /// tests-related configuration
    #[serde(default)]
    pub tests: TestsConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilesConfig {
    /// the files listed here will be ignored by snob when crawling the workspace
    #[serde(default)]
    pub ignores: HashSet<String>,
    /// the files listed here will trigger all tests on change
    #[serde(default)]
    pub run_all_tests_on_change: HashSet<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestsConfig {
    /// the tests listed here will always run
    #[serde(default)]
    pub always_run: HashSet<String>,
    /// the tests listed here will never run
    #[serde(default)]
    pub ignores: HashSet<String>,
}

const CONFIG_FILE: &str = "snob.toml";
const CONFIG_FILE_PYPROJECT: &str = "pyproject.toml";

impl Config {
    pub fn new(git_root: &Path) -> Self {
        // try to find the config file at `$GIT_ROOT/snob.toml`
        if let Ok(config) = std::fs::read_to_string(git_root.join(CONFIG_FILE)) {
            snob_debug!("Found config file at {:?}", git_root.join(CONFIG_FILE));
            return toml::from_str(&config).unwrap();
        // if it doesn't exist, check `$GIT_ROOT/pyproject.toml`
        } else if let Ok(config) = std::fs::read_to_string(git_root.join(CONFIG_FILE_PYPROJECT)) {
            snob_debug!(
                "Found config file at {:?}",
                git_root.join(CONFIG_FILE_PYPROJECT)
            );
            let pyproject: toml::Value = toml::from_str(&config).unwrap();
            if let Some(tool) = pyproject.get("tool") {
                if let Some(snob) = tool.get("snob") {
                    return snob.clone().try_into().unwrap();
                }
            }
        }
        // if it doesn't exist, use the default config
        Self::default()
    }
}
