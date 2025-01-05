use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;

use crate::ast::{extract_file_dependencies, INIT_FILE};
use crate::cli::Cli;
use crate::config::Config;
use crate::fs::{check_files_exist, crawl_workspace};
use crate::graph::{discover_impacted_nodes, discover_impacted_nodes_with_graphviz};
use crate::logging::init_logging;
use crate::stdin::{is_readable_stdin, read_from_stdin};
use crate::utils::merge_hashmaps;

use clap::Parser;

mod ast;
mod cli;
mod config;
mod fs;
mod graph;
mod logging;
mod stdin;
mod utils;

fn main() -> Result<()> {
    better_panic::install();

    let cli = Cli::parse();

    init_logging(&cli);

    let current_dir = std::env::current_dir()?;
    let git_root = get_repo_root(&current_dir);
    snob_debug!("Git root: {:?}", git_root);

    let config = Config::new(&git_root);
    snob_debug!("Config: {:?}", config);

    // files that were modified by the patch
    let updated_files = if is_readable_stdin() {
        read_from_stdin()
    } else {
        cli.updated_files
    }
    .iter()
    .map(|f| {
        let p = PathBuf::from(f);
        if p.is_relative() {
            current_dir.join(p).to_string_lossy().to_string()
        } else {
            p.to_string_lossy().to_string()
        }
    })
    .collect::<Vec<_>>();
    snob_debug!("Updated files: {:?}", updated_files);

    check_files_exist(&updated_files)?;

    let run_all_tests_on_change = build_glob_set(&config.files.run_all_tests_on_change)?;

    if updated_files.iter().any(|f| {
        !run_all_tests_on_change
            .matches(PathBuf::from(f).strip_prefix(&git_root).unwrap())
            .is_empty()
    }) {
        // exit early and run all tests
        snob_info!("Running all tests");
        println!(".");
        return Ok(());
    }

    std::env::set_current_dir(&cli.target_directory)?;
    snob_debug!("Current directory: {:?}", current_dir);
    let lookup_paths = get_python_local_lookup_paths(&current_dir, &git_root);
    snob_debug!("Python lookup paths: {:?}", lookup_paths);

    let instant = std::time::Instant::now();

    // crawl the target directory
    let workspace_files = crawl_workspace(&current_dir);

    // these need to retain some sort of order information
    let first_level_components: Vec<Vec<PathBuf>> = lookup_paths
        // ordered
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
        .collect();

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
        .collect::<HashSet<String>>();

    // build dependency graph (remove ignored files)
    let file_ignores = build_glob_set(&config.files.ignores)?;
    let mut all_file_imports: Vec<HashMap<String, Vec<String>>> = workspace_files
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
        .collect::<Vec<HashMap<String, Vec<String>>>>();

    // not deduplicated
    let dependency_graph = merge_hashmaps(&mut all_file_imports)
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                v.iter()
                    .map(std::string::ToString::to_string)
                    .collect::<HashSet<_>>(),
            )
        })
        .collect::<HashMap<String, HashSet<String>>>();

    snob_debug!("Dependency graph:");
    for (k, v) in &dependency_graph {
        snob_debug!("\t{k} is used by:");
        v.iter().for_each(|v| snob_debug!("\t\t{v}"));
    }

    let impacted_nodes: HashSet<String> = if let Some(dot_graph) = &cli.dot_graph {
        discover_impacted_nodes_with_graphviz(&dependency_graph, &updated_files, dot_graph)
    } else {
        discover_impacted_nodes(&dependency_graph, &updated_files)
    };

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

    snob_info!(
        "Analyzed {:?} files in {:?}",
        workspace_files.len(),
        instant.elapsed()
    );
    snob_info!(
        "Found {}/{} impacted tests",
        snob_results.impacted.len(),
        workspace_files
            .iter()
            .filter(|f| is_test_file(f))
            .collect::<Vec<_>>()
            .len()
    );

    // output resulting test files
    let stdout = std::io::stdout().lock();
    let mut writer = BufWriter::new(stdout);

    for test in snob_results.impacted {
        writeln!(writer, "{test}").unwrap();
    }

    writer.flush().unwrap();

    Ok(())
}

fn build_glob_set(globs: &HashSet<String>) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob)?);
    }
    Ok(builder.build()?)
}

struct SnobResult {
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

const PYTHONPATH_ENV: &str = "PYTHONPATH";

fn is_test_file<P>(file: P) -> bool
where
    P: AsRef<Path>,
{
    let file = file.as_ref();
    file.file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("test_")
        || file.to_string_lossy().ends_with("_test.py")
}

#[derive(Debug, PartialEq)]
pub struct LookupPaths {
    // to retain discovery order
    pub local_paths: Vec<PathBuf>,
    path_set: HashSet<PathBuf>,
}

impl LookupPaths {
    pub fn new() -> Self {
        Self {
            local_paths: Vec::new(),
            path_set: HashSet::new(),
        }
    }

    pub fn insert(&mut self, path: PathBuf) {
        if self.path_set.insert(path.clone()) {
            self.local_paths.push(path);
        }
    }
}

// PYTHONPATH
// python's import paths: [cwd, PYTHONPATH, others]
#[allow(dead_code)]
fn get_python_local_lookup_paths(current_dir: &PathBuf, git_root: &PathBuf) -> LookupPaths {
    // ordered
    let pythonpath = get_pythonpath();
    let mut local_paths = vec![current_dir.clone()];
    local_paths.extend(pythonpath);
    let mut lookup_paths = LookupPaths::new();
    local_paths
        .into_iter()
        .filter(|p| p.starts_with(git_root))
        .for_each(|p| lookup_paths.insert(p));
    lookup_paths
}

#[cfg(target_os = "windows")]
const PYTHONPATH_SEPARATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
const PYTHONPATH_SEPARATOR: &str = ":";

fn get_pythonpath() -> Vec<PathBuf> {
    let p = std::env::var(PYTHONPATH_ENV).unwrap_or_default();
    if p.is_empty() {
        return vec![];
    }
    p.split(PYTHONPATH_SEPARATOR).map(PathBuf::from).collect()
}

#[allow(dead_code)]
fn get_repo_root(current_dir: &PathBuf) -> PathBuf {
    let mut path = current_dir.clone();
    // let's cross our fingers here
    while !path.join(".git").exists() {
        path = path.parent().unwrap().to_path_buf();
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pythonpath_empty() {
        std::env::remove_var(PYTHONPATH_ENV);
        let pythonpath = get_pythonpath();
        assert!(pythonpath.is_empty());
    }

    #[test]
    fn test_get_pythonpath() {
        std::env::set_var(PYTHONPATH_ENV, "/some/path:/another/path");
        let pythonpath = get_pythonpath();
        assert_eq!(
            pythonpath,
            vec![PathBuf::from("/some/path"), PathBuf::from("/another/path")]
        );
    }

    #[test]
    fn test_get_python_local_lookup_paths() {
        let current_dir = PathBuf::from("/home/user/project/src");
        let git_root = PathBuf::from("/home/user/project");
        let pythonpath = vec![
            PathBuf::from("/home/user/project/src"),
            PathBuf::from("/home/user/project/lib"),
            PathBuf::from("/home/user/other-project/src/lib"),
        ];
        std::env::set_var(
            PYTHONPATH_ENV,
            pythonpath
                .iter()
                .map(|p| p.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(":"),
        );
        let local_paths = get_python_local_lookup_paths(&current_dir, &git_root);
        assert_eq!(
            local_paths,
            LookupPaths {
                local_paths: vec![
                    PathBuf::from("/home/user/project/src"),
                    PathBuf::from("/home/user/project/lib"),
                ],
                path_set: vec![
                    PathBuf::from("/home/user/project/src"),
                    PathBuf::from("/home/user/project/lib"),
                ]
                .into_iter()
                .collect()
            }
        );
    }
}
