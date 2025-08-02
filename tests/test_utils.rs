use std::env;
use std::path::Path;

// Use the same separator constants as the main codebase
#[cfg(target_os = "windows")]
const PYTHONPATH_SEPARATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
const PYTHONPATH_SEPARATOR: &str = ":";

/// Set up PYTHONPATH environment variable to include the src directory
/// of a test project, allowing Python imports to work without sys.path manipulation
pub fn setup_pythonpath_for_project(project_path: &Path) {
    let src_path = project_path.join("src");
    let current_pythonpath = env::var("PYTHONPATH").unwrap_or_default();

    let new_pythonpath = if current_pythonpath.is_empty() {
        src_path.to_string_lossy().to_string()
    } else {
        // Prepend our src path to existing PYTHONPATH using platform-appropriate separator
        format!(
            "{}{}{}",
            src_path.to_string_lossy(),
            PYTHONPATH_SEPARATOR,
            current_pythonpath
        )
    };

    env::set_var("PYTHONPATH", new_pythonpath);
}

/// Clean up PYTHONPATH after tests (optional, but good practice)
#[allow(dead_code)]
pub fn cleanup_pythonpath() {
    env::remove_var("PYTHONPATH");
}

/// Set up PYTHONPATH for multiple source directories (for more complex project structures)
#[allow(dead_code)]
pub fn setup_pythonpath_for_paths(paths: &[&Path]) {
    let current_pythonpath = env::var("PYTHONPATH").unwrap_or_default();

    let paths_str: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let new_pythonpath = if current_pythonpath.is_empty() {
        paths_str.join(PYTHONPATH_SEPARATOR)
    } else {
        format!(
            "{}{}{}",
            paths_str.join(PYTHONPATH_SEPARATOR),
            PYTHONPATH_SEPARATOR,
            current_pythonpath
        )
    };

    env::set_var("PYTHONPATH", new_pythonpath);
}
