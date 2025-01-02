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

pub fn crawl_workspace(current_dir: &std::path::PathBuf) -> Vec<std::path::PathBuf> {
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
