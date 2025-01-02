use std::collections::HashSet;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::fs::crawl_workspace;

use ast::extract_imports;
use clap::Parser;

mod ast;
mod cli;
mod fs;

fn main() {
    better_panic::install();

    let cli = Cli::parse();

    // files that were modified by the patch
    let updated_files = &cli.updated_files;

    // crawl the target directory
    let workspace_files = crawl_workspace(
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
    workspace_files.iter().for_each(|f| {
        if let Ok(file_imports) = extract_imports(f) {
            // File: python_code/package_1/module_1.py
            // depends on [python_code/package_1/module_2.py, python_code/package_2/__init__.py]
            let resolved_imports = file_imports.resolve_imports(&project_files);
            println!("{:?}", f);
            println!("{:?}", resolved_imports);
            println!();
        }
    });

    //let dependency_graph = build_dependency_graph(&workspace_files);

    // resolve transitive closure of dependencies

    // only keep sugraphs of interest (those that include `updated_files`)

    // find tests that depend on each subgraph of interest

    // output resulting test files
}

fn build_dependency_graph(workspace_files: &Vec<std::path::PathBuf>) {
    todo!()
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
