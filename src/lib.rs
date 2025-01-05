use anyhow::Result;
use ast::{extract_file_dependencies, INIT_FILE};
use config::Config;
use fs::crawl_workspace;
use graph::discover_impacted_nodes;
use logging::{init_logging, LoggingConfiguration};
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use utils::{
    get_python_local_lookup_paths, get_repo_root, is_test_file, merge_hashmaps, LookupPaths,
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use pyo3::prelude::*;

mod ast;
mod config;
mod fs;
mod graph;
mod logging;
mod stdin;
mod utils;

// pytest --snob='git diff HEAD~1 --name-only'

#[pyfunction]
pub fn get_tests(changed_files: Vec<String>) -> PyResult<Vec<String>> {
    init_logging(&LoggingConfiguration::default());

    let current_dir = std::env::current_dir()?;
    let git_root = get_repo_root(&current_dir);
    let config = Config::new(&git_root);
    let snob_output = get_impacted_tests_from_changed_files(
        &config,
        &current_dir,
        &git_root,
        &changed_files
            .into_iter()
            .map(|c| git_root.join(c).to_string_lossy().to_string())
            .collect::<HashSet<String>>(),
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
pub fn snob(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_tests, m)?)?;
    Ok(())
}

fn build_glob_set(globs: &HashSet<String>) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob)?);
    }
    Ok(builder.build()?)
}

pub struct SnobResult {
    pub impacted: HashSet<String>,
    pub always_run: HashSet<String>,
    pub ignored: HashSet<String>,
}

impl SnobResult {
    pub fn new(
        impacted: HashSet<String>,
        workspace_files: HashSet<String>,
        ignore_glob: &GlobSet,
        always_run_glob: &GlobSet,
        git_root: &Path,
    ) -> Self {
        let always_run_tests = workspace_files
            .into_iter()
            .filter(|f| {
                !always_run_glob
                    .matches(PathBuf::from(f).strip_prefix(git_root).unwrap())
                    .is_empty()
                    && is_test_file(f)
            })
            .collect::<HashSet<String>>();

        let impacted_tests = impacted
            .into_iter()
            .filter(|f| is_test_file(f))
            .collect::<HashSet<String>>();

        let ignored_tests = impacted_tests
            .iter()
            .map(std::string::ToString::to_string)
            .filter(|f| {
                !ignore_glob
                    .matches(PathBuf::from(f).strip_prefix(git_root).unwrap())
                    .is_empty()
            })
            .collect::<HashSet<String>>();
        Self {
            impacted: impacted_tests.difference(&ignored_tests).cloned().collect(),
            always_run: always_run_tests,
            ignored: ignored_tests,
        }
    }
}

pub enum SnobOutput {
    All,
    Partial(SnobResult),
}

pub fn get_impacted_tests_from_changed_files(
    config: &Config,
    current_dir: &PathBuf,
    git_root: &PathBuf,
    // absolute paths that are guaranteed to exist
    changed: &HashSet<String>,
) -> Result<SnobOutput> {
    let run_all_tests_on_change = build_glob_set(&config.files.run_all_tests_on_change)?;
    if run_all_tests(changed, &run_all_tests_on_change, &git_root) {
        // exit early and run all tests
        snob_info!("Running all tests");
        return Ok(SnobOutput::All);
    }

    let lookup_paths = get_python_local_lookup_paths(current_dir, git_root);
    //snob_debug!("Python lookup paths: {:?}", lookup_paths);

    // crawl the target directory
    let workspace_files = crawl_workspace(&current_dir);

    // these need to retain some sort of order information
    let first_level_components: Vec<Vec<PathBuf>> = get_first_level_components(&lookup_paths);
    //snob_debug!("First level components: {:?}", first_level_components);

    //snob_debug!(
    //    "Crawled {:?} files and {:?} directories",
    //    workspace_files.len(),
    //    first_level_components.len()
    //);

    // keep a copy of the tree (contains all workspace files)
    let project_files = workspace_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<HashSet<String>>();

    // build dependency graph (remove ignored files)
    let file_ignores = build_glob_set(&config.files.ignores)?;
    let mut all_file_imports: Vec<HashMap<String, Vec<String>>> = build_dependency_graph(
        &workspace_files,
        &project_files,
        &file_ignores,
        &first_level_components,
        &git_root,
    );

    // not deduplicated
    let dependency_graph = deduplicate_dependencies(merge_hashmaps(&mut all_file_imports));
    snob_debug!("Dependency graph:");
    for (k, v) in &dependency_graph {
        snob_debug!("\t{k} is used by:");
        v.iter().for_each(|v| snob_debug!("\t\t{v}"));
    }

    let impacted_nodes: HashSet<String> = discover_impacted_nodes(&dependency_graph, changed);

    // filter impacted nodes to get the tests
    // test_*.py   or   *_test.py
    let ignored_tests = build_glob_set(&config.tests.ignores)?;
    let tests_to_always_run = build_glob_set(&config.tests.always_run)?;

    let snob_results = SnobResult::new(
        impacted_nodes,
        project_files.clone(),
        &ignored_tests,
        &tests_to_always_run,
        &git_root,
    );
    snob_debug!(" impacted tests: {:?}", snob_results.impacted);
    snob_debug!(" ignored tests: {:?}", snob_results.ignored);
    snob_debug!(" always run tests: {:?}", snob_results.always_run);

    Ok(SnobOutput::Partial(snob_results))
}

pub fn run_all_tests(
    updated_files: &HashSet<String>,
    run_all_tests_on_change: &GlobSet,
    git_root: &Path,
) -> bool {
    updated_files.iter().any(|f| {
        !run_all_tests_on_change
            .matches(PathBuf::from(f).strip_prefix(&git_root).unwrap())
            .is_empty()
    })
}

pub fn get_first_level_components(lookup_paths: &LookupPaths) -> Vec<Vec<PathBuf>> {
    lookup_paths
        .local_paths
        .iter()
        .map(|p| {
            p.read_dir()
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|p| {
                    (p.is_file() && p.extension().is_some_and(|ext| ext == "py"))
                        || (p.is_dir() && p.join(INIT_FILE).exists())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn build_dependency_graph(
    workspace_files: &[PathBuf],
    project_files: &HashSet<String>,
    file_ignores: &GlobSet,
    first_level_components: &Vec<Vec<PathBuf>>,
    git_root: &Path,
) -> Vec<HashMap<String, Vec<String>>> {
    workspace_files
        .par_iter()
        .filter(|f| {
            file_ignores
                .matches(PathBuf::from(f).strip_prefix(&git_root).unwrap())
                .is_empty()
        })
        .filter_map(|f| {
            if let Ok(graph) = extract_file_dependencies(f, &project_files, &first_level_components)
            {
                Some(graph)
            } else {
                snob_error!("Failed to parse file {:?}", f);
                None
            }
        })
        .collect::<Vec<HashMap<String, Vec<String>>>>()
}

pub fn deduplicate_dependencies(
    dependencies: HashMap<String, Vec<String>>,
) -> HashMap<String, HashSet<String>> {
    dependencies
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                v.iter()
                    .map(std::string::ToString::to_string)
                    .collect::<HashSet<_>>(),
            )
        })
        .collect::<HashMap<String, HashSet<String>>>()
}
