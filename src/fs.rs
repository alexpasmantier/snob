use std::path::Path;

use ignore::{types::TypesBuilder, DirEntry, WalkBuilder};

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
pub fn crawl_workspace() -> Vec<std::path::PathBuf> {
    let current_dir = std::env::current_dir().unwrap();
    let builder = create_walk_builder(&current_dir);
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

pub fn check_files_exist<P>(files: &[P]) -> Result<(), std::io::Error>
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
