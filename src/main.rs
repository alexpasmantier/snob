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

    // build dependency graph
    workspace_files.iter().for_each(|f| {
        if let Ok(file_imports) = extract_imports(f) {
            println!("\n");
            println!("FILE: {:?}", file_imports.file);
            for import in file_imports.imports {
                println!(
                    "\tIMPORT: path: {:?}  level: {:?}",
                    import.segments, import.level
                );
            }
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
fn get_pythonpath() -> Vec<PathBuf> {
    return vec![std::env::current_dir().unwrap()];
}
