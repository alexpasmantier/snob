use std::path::{Path, PathBuf};

use crate::ast::INIT_FILE;
use crate::utils::LookupPaths;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{types::TypesBuilder, DirEntry, WalkBuilder};
use rustc_hash::FxHashSet;

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
pub fn crawl_workspace(current_dir: &std::path::PathBuf) -> Vec<std::path::PathBuf> {
    let builder = create_walk_builder(current_dir);
    let (tx_file_handle, rx_file_handle) = std::sync::mpsc::channel();

    let parallel_walker = builder.build_parallel();
    parallel_walker.run(|| {
        Box::new(
            |entry: Result<DirEntry, ignore::Error>| -> ignore::WalkState {
                match entry {
                    Ok(entry) => {
                        if let Some(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                tx_file_handle.send(entry.path().to_path_buf()).unwrap();
                            }
                        }
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

    rx_file_handle.try_iter().collect()
}

pub fn check_files_exist<P>(files: &FxHashSet<P>) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    for file in files {
        if !file.as_ref().exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File {:?} does not exist", file.as_ref()),
            ));
        }
    }
    Ok(())
}

pub fn make_files_relative_to<P>(files: &[P], base: &Path) -> Vec<String>
where
    P: AsRef<Path>,
{
    files
        .iter()
        .map(|f| {
            let p = PathBuf::from(f.as_ref());
            if p.is_relative() {
                base.join(p).to_string_lossy().to_string()
            } else {
                p.to_string_lossy().to_string()
            }
        })
        .collect::<Vec<_>>()
}

// this function is used to build globs from the settings defined in the user configuration
// file
pub fn build_glob_set(globs: &FxHashSet<String>) -> anyhow::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob)?);
    }
    Ok(builder.build()?)
}

// first level components are either packages or modules defined at the top level
// of the lookup paths used by python's import system
// those first level components allow us to determine how to handle external packages
// down the line
pub fn get_first_level_components(lookup_paths: &LookupPaths) -> Vec<PathBuf> {
    lookup_paths
        .local_paths
        .iter()
        .flat_map(|p| {
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
