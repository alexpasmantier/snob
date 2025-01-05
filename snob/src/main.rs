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
    //snob_debug!("Git root: {:?}", git_root);

    let config = Config::new(&git_root);
    //snob_debug!("Config: {:?}", config);

    // files that were modified by the patch
    let updated_files = make_files_relative_to(
        if is_readable_stdin() {
            read_from_stdin()
        } else {
            cli.updated_files
        }?,
        &current_dir,
    );
    //snob_debug!("Updated files: {:?}", updated_files);

    check_files_exist(&updated_files)?;

    let run_all_tests_on_change = build_glob_set(&config.files.run_all_tests_on_change)?;
    if run_all_tests(&updated_files, &run_all_tests_on_change, &git_root) {
        // exit early and run all tests
        //snob_info!("Running all tests");
        println!(".");
        return Ok(());
    }

    std::env::set_current_dir(&cli.target_directory)?;
    //snob_debug!("Current directory: {:?}", current_dir);
    let lookup_paths = get_python_local_lookup_paths(&current_dir, &git_root);
    //snob_debug!("Python lookup paths: {:?}", lookup_paths);

    let instant = std::time::Instant::now();

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
