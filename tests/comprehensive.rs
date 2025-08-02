use anyhow::Result;
use rustc_hash::FxHashSet;

use snob_lib::config::Config;
use snob_lib::{get_impacted_tests_from_changed_files, SnobOutput};

mod test_utils;
use test_utils::setup_pythonpath_for_project;

mod fixtures;
use fixtures::*;

#[test]
fn test_with_clean_imports_simple_project() -> Result<()> {
    let temp_dir = create_simple_project()?;
    let project_path = temp_dir.path().to_path_buf();

    // Set up PYTHONPATH so our clean import statements work
    setup_pythonpath_for_project(&project_path);

    // Test dependency analysis
    let changed_files = vec![project_path
        .join("src/module.py")
        .to_string_lossy()
        .to_string()]
    .into_iter()
    .collect::<FxHashSet<String>>();

    let config = Config::new(&project_path);
    let result = get_impacted_tests_from_changed_files(
        &config,
        &project_path,
        &project_path,
        &changed_files,
    )?;

    match result {
        SnobOutput::Partial(results) => {
            println!(
                "Simple project - Found {} impacted tests",
                results.impacted.len()
            );
            // The test files should be found (or at least the analysis should complete)
            // Basic sanity check - just verify the analysis completes
        }
        SnobOutput::All => {
            println!("Simple project - All tests should be run");
        }
    }

    Ok(())
}

#[test]
fn test_with_clean_imports_complex_project() -> Result<()> {
    let temp_dir = create_complex_project()?;
    let project_path = temp_dir.path().to_path_buf();

    // Set up PYTHONPATH
    setup_pythonpath_for_project(&project_path);

    // Test changing a utility file that affects multiple modules
    let changed_files = vec![project_path
        .join("src/utils.py")
        .to_string_lossy()
        .to_string()]
    .into_iter()
    .collect::<FxHashSet<String>>();

    let config = Config::new(&project_path);
    let result = get_impacted_tests_from_changed_files(
        &config,
        &project_path,
        &project_path,
        &changed_files,
    )?;

    match result {
        SnobOutput::Partial(results) => {
            println!(
                "Complex project - Found {} impacted tests",
                results.impacted.len()
            );
            // Should potentially find multiple test files due to transitive dependencies
        }
        SnobOutput::All => {
            println!("Complex project - All tests should be run");
        }
    }

    Ok(())
}

#[test]
fn test_with_clean_imports_isolated_project() -> Result<()> {
    let temp_dir = create_isolated_project()?;
    let project_path = temp_dir.path().to_path_buf();

    // Set up PYTHONPATH
    setup_pythonpath_for_project(&project_path);

    // Test changing an isolated file
    let changed_files = vec![project_path
        .join("src/isolated.py")
        .to_string_lossy()
        .to_string()]
    .into_iter()
    .collect::<FxHashSet<String>>();

    let config = Config::new(&project_path);
    let result = get_impacted_tests_from_changed_files(
        &config,
        &project_path,
        &project_path,
        &changed_files,
    )?;

    match result {
        SnobOutput::Partial(results) => {
            println!(
                "Isolated project - Found {} impacted tests",
                results.impacted.len()
            );
            // Should find minimal tests since files are isolated
        }
        SnobOutput::All => {
            println!("Isolated project - All tests should be run");
        }
    }

    Ok(())
}

#[test]
fn test_with_config_file() -> Result<()> {
    let temp_dir = create_project_with_config()?;
    let project_path = temp_dir.path().to_path_buf();

    // Set up PYTHONPATH
    setup_pythonpath_for_project(&project_path);

    // Test changing main.py with configuration rules applied
    let changed_files = vec![project_path
        .join("src/main.py")
        .to_string_lossy()
        .to_string()]
    .into_iter()
    .collect::<FxHashSet<String>>();

    let config = Config::new(&project_path);
    let result = get_impacted_tests_from_changed_files(
        &config,
        &project_path,
        &project_path,
        &changed_files,
    )?;

    match result {
        SnobOutput::Partial(results) => {
            println!(
                "Config project - Found {} impacted tests",
                results.impacted.len()
            );
            println!("  - Always run: {}", results.always_run.len());
            println!("  - Ignored: {}", results.ignored.len());

            // Should include always-run tests even if not directly impacted
            assert!(results.always_run.len() > 0, "Should have always-run tests");
        }
        SnobOutput::All => {
            println!("Config project - All tests should be run");
        }
    }

    Ok(())
}
