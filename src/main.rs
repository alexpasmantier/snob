use crate::cli::Cli;
use clap::Parser;
use ignore::{DirEntry, WalkBuilder, types::TypesBuilder};

mod cli;

fn main() {
    let cli = Cli::parse();

    // files that were modified by the patch
    let updated_files = &cli.updated_files;

    // crawl the target directory
    let workspace_files = crawl_workspace(
        &cli.target_directory
            .unwrap_or(std::env::current_dir().unwrap()),
    );
    println!("{:?}", workspace_files);

    // build dependency graph
    //let dependency_graph = build_dependency_graph(&workspace_files);

    // resolve transitive closure of dependencies

    // only keep sugraphs of interest (those that include `updated_files`)

    // find tests that depend on each subgraph of interest

    // output resulting test files
}

fn create_walk_builder(current_dir: &std::path::PathBuf) -> WalkBuilder {
    let mut builder = WalkBuilder::new(current_dir);

    // only python files
    let mut types_builder = TypesBuilder::new();
    types_builder.add_defaults();
    types_builder.select("py");
    builder.types(types_builder.build().unwrap());

    builder
}

fn crawl_workspace(current_dir: &std::path::PathBuf) -> Vec<std::path::PathBuf> {
    let builder = create_walk_builder(current_dir);
    let (tx_handle, rx_handle) = std::sync::mpsc::channel();

    let parallel_walker = builder.build_parallel();
    parallel_walker.run(|| {
        Box::new(
            |entry: Result<DirEntry, ignore::Error>| -> ignore::WalkState {
                match entry {
                    Ok(entry) => {
                        if let Some(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                return ignore::WalkState::Continue;
                            }
                        }
                        tx_handle.send(entry.path().to_path_buf()).unwrap();
                        ignore::WalkState::Continue
                    }
                    Err(err) => {
                        eprintln!("Error: {err}");
                        ignore::WalkState::Continue
                    }
                }
            },
        )
    });

    rx_handle.try_iter().collect()
}

fn build_dependency_graph(workspace_files: &Vec<std::path::PathBuf>) {
    todo!()
}
