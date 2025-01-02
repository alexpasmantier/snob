use std::collections::HashSet;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::fs::crawl_workspace;

use ast::{ResolvedFileImports, extract_imports};
use clap::Parser;
use graph::build_dependency_graph;

mod ast;
mod cli;
mod fs;
mod graph;

fn main() {
    better_panic::install();

    let cli = Cli::parse();

    // files that were modified by the patch
    let updated_files = &cli.updated_files;

    // crawl the target directory
    let (workspace_files, first_level_dirs) = crawl_workspace(
        &cli.target_directory
            .unwrap_or(std::env::current_dir().unwrap()),
    );

    // keep a copy of the tree
    let project_files = HashSet::<String>::from_iter(
        workspace_files
            .iter()
            .map(|p| p.to_string_lossy().to_string()),
    );

    // build dependency graph
    let all_file_imports: Vec<ResolvedFileImports> = workspace_files
        .iter()
        .flat_map(|f| {
            extract_imports(f)
                .map(|file_imports| file_imports.resolve_imports(&project_files, &first_level_dirs))
            // File: python_code/package_1/module_1.py
            // depends on [python_code/package_1/module_2.py, python_code/package_2/__init__.py]
        })
        .collect();

    let dependency_graph = build_dependency_graph(&all_file_imports);

    std::fs::write("dependency_graph.txt", format!("{:?}", dependency_graph)).unwrap();

    // resolve transitive closure of dependencies

    // only keep sugraphs of interest (those that include `updated_files`)

    // find tests that depend on each subgraph of interest

    // output resulting test files
}

// PYTHONPATH
// python's import paths: [cwd, PYTHONPATH, others]
fn get_importlib_paths() -> Vec<PathBuf> {
    return vec![get_repo_root()];
}

fn get_repo_root() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    // let's cross our fingers here
    while !path.join(".git").exists() {
        path = path.parent().unwrap().to_path_buf();
    }
    path
}
