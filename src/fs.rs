use std::{collections::HashSet, path::PathBuf};

use ignore::{DirEntry, WalkBuilder, types::TypesBuilder};

fn create_walk_builder(current_dir: &std::path::PathBuf) -> WalkBuilder {
    let mut builder = WalkBuilder::new(current_dir);

    // only python files
    let mut types_builder = TypesBuilder::new();
    types_builder.add_defaults();
    types_builder.select("py");
    builder.types(types_builder.build().unwrap());

    builder
}

/// Crawl the workspace and return a list of files and directories
/// # Arguments
/// * `current_dir` - The directory to start the crawl from
/// # Returns
/// * A tuple containing a list of files and a list of directories
pub fn crawl_workspace(
    current_dir: &std::path::PathBuf,
) -> (Vec<std::path::PathBuf>, HashSet<PathBuf>) {
    let builder = create_walk_builder(current_dir);
    let (tx_file_handle, rx_file_handle) = std::sync::mpsc::channel();
    // for first level dirs
    let (tx_dir_handle, rx_dir_handle) = std::sync::mpsc::channel();

    let parallel_walker = builder.build_parallel();
    parallel_walker.run(|| {
        Box::new(
            |entry: Result<DirEntry, ignore::Error>| -> ignore::WalkState {
                match entry {
                    Ok(entry) => {
                        let p = entry.path().strip_prefix(current_dir).unwrap();
                        if let Some(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                if p.components().count() == 1 {
                                    tx_dir_handle.send(p.to_path_buf()).unwrap();
                                }
                                return ignore::WalkState::Continue;
                            }
                        }
                        tx_file_handle.send(p.to_path_buf()).unwrap();
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

    (
        rx_file_handle.try_iter().collect(),
        rx_dir_handle.try_iter().collect(),
    )
}

//import a.b.c as c
//
//from a.b import c
//
//a.b.c -> a/b/c/__init__.py existe pas
//
//a/
//  b.py
//  b/
//    __init__.py
