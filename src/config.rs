use rustc_hash::FxHashSet;
use std::path::Path;

use serde::Deserialize;

use crate::snob_debug;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// general configuration
    #[serde(default)]
    pub general: GeneralConfig,
    /// files-related configuration
    #[serde(default)]
    pub files: FilesConfig,
    /// tests-related configuration
    #[serde(default)]
    pub tests: TestsConfig,
}

// FIXME: not sure we're actually using this
#[derive(Debug, Default, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_verbosity_level")]
    pub verbosity_level: usize,
    #[serde(default = "default_quiet")]
    pub quiet: bool,
}

fn default_verbosity_level() -> usize {
    2
}

fn default_quiet() -> bool {
    false
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilesConfig {
    /// the files listed here will be ignored by snob when crawling the workspace
    #[serde(default)]
    pub ignores: FxHashSet<String>,
    /// the files listed here will trigger all tests on change
    #[serde(default)]
    pub run_all_tests_on_change: FxHashSet<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestsConfig {
    /// the tests listed here will always run
    #[serde(default)]
    pub always_run: FxHashSet<String>,
    /// the tests listed here will never run
    #[serde(default)]
    pub ignores: FxHashSet<String>,
}

const CONFIG_FILE_SNOB: &str = "snob.toml";
const CONFIG_FILE_PYPROJECT: &str = "pyproject.toml";

impl Config {
    pub fn new(git_root: &Path) -> Self {
        // try to find the config file at `$GIT_ROOT/snob.toml`
        if let Ok(config) = std::fs::read_to_string(git_root.join(CONFIG_FILE_SNOB)) {
            snob_debug!("Found config file at {:?}", git_root.join(CONFIG_FILE_SNOB));
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
