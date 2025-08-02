use anyhow::Result;
use config::Config;
use fs::crawl_workspace;
use graph::discover_impacted_nodes;
use logging::{init_logging, LoggingConfiguration};
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use utils::{get_python_local_lookup_paths, get_repo_root, merge_hashmaps};

use pyo3::prelude::*;

pub mod ast;
pub mod config;
pub mod fs;
pub mod graph;
pub mod logging;
pub mod results;
pub mod stdin;
pub mod utils;

#[pyfunction]
pub fn get_tests(changed_files: Vec<String>) -> PyResult<Vec<String>> {
    let current_dir = std::env::current_dir()?;
    let git_root = get_repo_root(&current_dir)?;

    let config = Config::new(&git_root);

    let logging_configuration =
        LoggingConfiguration::new(config.general.verbosity_level, config.general.quiet);
    init_logging(&logging_configuration);

    snob_debug!("Git root: {:?}", git_root);
    snob_debug!("Config: {:?}", config);

    let snob_output = get_impacted_tests_from_changed_files(
        &config,
        &current_dir,
        &git_root,
        &changed_files
            .into_iter()
            .map(|c| git_root.join(c).to_string_lossy().to_string())
            .collect::<FxHashSet<String>>(),
    );
    match snob_output {
        Ok(SnobOutput::All) => Ok(vec![]),
        Ok(SnobOutput::Partial(snob_results)) => Ok(snob_results.impacted.into_iter().collect()),
        Err(e) => {
            snob_error!("Error: {:?}", e);
            PyResult::Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                "{e:?}",
            )))
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
pub fn snob_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_tests, m)?)?;
    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum SnobOutput {
    All,
    Partial(results::SnobResult),
}

pub fn get_impacted_tests_from_changed_files(
    config: &Config,
    current_dir: &PathBuf,
    git_root: &PathBuf,
    // absolute paths that are guaranteed to exist
    changed: &FxHashSet<String>,
) -> Result<SnobOutput> {
    let run_all_tests_on_change = fs::build_glob_set(&config.files.run_all_tests_on_change)?;
    if utils::should_run_all_tests(changed, &run_all_tests_on_change, git_root) {
        // exit early and run all tests
        snob_info!("Running all tests");
        return Ok(SnobOutput::All);
    }

    let lookup_paths = get_python_local_lookup_paths(current_dir, git_root);
    snob_debug!("Python lookup paths: {:?}", lookup_paths);

    // crawl the target directory
    let workspace_files = crawl_workspace(current_dir);

    // these need to retain some sort of order information
    let first_level_components: Vec<PathBuf> = fs::get_first_level_components(&lookup_paths);
    snob_debug!("First level components: {:?}", first_level_components);

    snob_debug!(
        "Crawled {:?} files and {:?} directories",
        workspace_files.len(),
        first_level_components.len()
    );

    // keep a copy of the tree (contains all workspace files)
    let project_files = workspace_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<FxHashSet<String>>();

    // build dependency graph (remove ignored files)
    let file_ignores = fs::build_glob_set(&config.files.ignores)?;
    let mut all_file_imports: Vec<FxHashMap<String, Vec<String>>> = graph::build_dependency_graph(
        &workspace_files,
        &project_files,
        &file_ignores,
        &first_level_components,
        git_root,
    );

    // not deduplicated
    let dependency_graph = utils::deduplicate_dependencies(merge_hashmaps(&mut all_file_imports));
    snob_debug!("Dependency graph:");
    for (k, v) in &dependency_graph {
        snob_debug!("\t{k} is used by:");
        v.iter().for_each(|v| snob_debug!("\t\t{v}"));
    }

    let impacted_nodes: FxHashSet<String> = discover_impacted_nodes(&dependency_graph, changed);

    // filter impacted nodes to get the tests
    // test_*.py   or   *_test.py
    let ignored_tests = fs::build_glob_set(&config.tests.ignores)?;
    let tests_to_always_run = fs::build_glob_set(&config.tests.always_run)?;

    let snob_results = results::SnobResult::new(
        impacted_nodes,
        project_files.clone(),
        &ignored_tests,
        &tests_to_always_run,
        git_root,
    );
    snob_debug!(" impacted tests: {:?}", snob_results.impacted);
    snob_debug!(" ignored tests: {:?}", snob_results.ignored);
    snob_debug!(" always run tests: {:?}", snob_results.always_run);

    Ok(SnobOutput::Partial(snob_results))
}
