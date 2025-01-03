use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use rayon::prelude::*;
use stdin::{is_readable_stdin, read_from_stdin};

use crate::cli::Cli;
use crate::fs::crawl_workspace;

use ast::extract_file_dependencies;
use clap::Parser;
use graph::discover_impacted_nodes;
use utils::merge_hashmaps;

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
    // TODO: check if files exist
    let updated_files = if is_readable_stdin() {
        read_from_stdin()
    } else {
        cli.updated_files
    };

    std::env::set_current_dir(&cli.target_directory)?;

    // crawl the target directory
    let (workspace_files, first_level_dirs) = crawl_workspace();

    info!(
        "Crawled {:?} files and {:?} directories",
        workspace_files.len(),
        first_level_dirs.len()
    );

    // keep a copy of the tree
    let project_files = workspace_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<HashSet<String>>();

    // build dependency graph
    let mut all_file_imports: Vec<HashMap<String, Vec<String>>> = workspace_files
        .par_iter()
        .filter_map(
            |f| match extract_file_dependencies(f, &project_files, &first_level_dirs) {
                Ok(graph) => Some(graph),
                Err(e) => {
                    log::error!("Error parsing file {:?}", f);
                    log::debug!("Error parsing file {:?}: {:?}", f, e);
                    None
                }
            },
        )
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

    let impacted_nodes: HashSet<String> =
        discover_impacted_nodes(&dependency_graph, &updated_files);

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

    // output resulting test files
    let stdout = std::io::stdout().lock();
    let mut writer = BufWriter::new(stdout);

    for test in impacted_tests {
        writeln!(writer, "{test}").unwrap();
    }

    writer.flush().unwrap();

    Ok(())
}

// PYTHONPATH
// python's import paths: [cwd, PYTHONPATH, others]
#[allow(dead_code)]
fn get_importlib_paths() -> Vec<PathBuf> {
    vec![get_repo_root()]
}

#[allow(dead_code)]
fn get_repo_root() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    // let's cross our fingers here
    while !path.join(".git").exists() {
        path = path.parent().unwrap().to_path_buf();
    }
    path
}
