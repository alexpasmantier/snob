use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use rayon::prelude::*;

use crate::ast::{extract_file_dependencies, INIT_FILE};
use crate::cli::Cli;
use crate::fs::{check_files_exist, crawl_workspace};
use crate::graph::{discover_impacted_nodes, discover_impacted_nodes_with_graphviz};
use crate::stdin::{is_readable_stdin, read_from_stdin};
use crate::utils::merge_hashmaps;

use clap::Parser;

mod ast;
mod cli;
mod fs;
mod graph;
mod stdin;
mod utils;

fn main() -> Result<()> {
    better_panic::install();

    let cli = Cli::parse();

    stderrlog::new()
        .verbosity(cli.verbosity_level)
        .quiet(cli.quiet)
        .init()
        .unwrap();

    // files that were modified by the patch
    let updated_files = if is_readable_stdin() {
        read_from_stdin()
    } else {
        cli.updated_files
    };

    check_files_exist(&updated_files)?;

    std::env::set_current_dir(&cli.target_directory)?;
    let current_dir = std::env::current_dir()?;
    debug!("Current directory: {:?}", current_dir);
    let git_root = get_repo_root(&current_dir);
    debug!("Git root: {:?}", git_root);
    let lookup_paths = get_python_local_lookup_paths(&current_dir, &git_root);
    debug!("Python lookup paths: {:?}", lookup_paths);

    let instant = std::time::Instant::now();

    // crawl the target directory
    let workspace_files = crawl_workspace();

    // these need to retain some sort of order information
    let first_level_components: Vec<Vec<PathBuf>> = lookup_paths
        // ordered
        .local_paths
        .iter()
        .map(|p| {
            p.read_dir()
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|p| p.is_file() || (p.is_dir() && p.join(INIT_FILE).exists()))
                .collect::<Vec<_>>()
        })
        .collect();

    debug!("First level components: {:?}", first_level_components);

    debug!(
        "Crawled {:?} files and {:?} directories",
        workspace_files.len(),
        first_level_components.len()
    );

    // keep a copy of the tree
    let project_files = workspace_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<HashSet<String>>();

    //debug!("Project files: {:?}", project_files);

    // build dependency graph
    let mut all_file_imports: Vec<HashMap<String, Vec<String>>> = workspace_files
        .par_iter()
        .filter_map(|f| {
            if let Ok(graph) = extract_file_dependencies(f, &project_files, &first_level_components)
            {
                Some(graph)
            } else {
                error!("Failed to parse file {:?}", f);
                None
            }
        })
        .collect::<Vec<HashMap<String, Vec<String>>>>();

    debug!("All file imports: {:?}", all_file_imports);

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

    let impacted_nodes: HashSet<String> = if let Some(dot_graph) = &cli.dot_graph {
        discover_impacted_nodes_with_graphviz(&dependency_graph, &updated_files, dot_graph)
    } else {
        discover_impacted_nodes(&dependency_graph, &updated_files)
    };

    debug!(
        "Impacted nodes: {:?}",
        impacted_nodes.iter().filter(|n| {
            !PathBuf::from(n)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("test_")
                || n.ends_with("_test.py")
        })
    );

    // filter impacted nodes to get the tests
    // test_*.py   or   *_test.py
    let impacted_tests = impacted_nodes.into_iter().filter(|n| {
        PathBuf::from(n)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("test_")
            || n.ends_with("_test.py")
    });
    debug!("Impacted tests: {:?}", impacted_tests);

    info!(
        "Analyzed {:?} files in {:?}",
        workspace_files.len(),
        instant.elapsed()
    );

    // output resulting test files
    let stdout = std::io::stdout().lock();
    let mut writer = BufWriter::new(stdout);

    for test in impacted_tests {
        writeln!(writer, "{test}").unwrap();
    }

    writer.flush().unwrap();

    Ok(())
}

const PYTHONPATH_ENV: &str = "PYTHONPATH";

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
