use crate::ast::extract_file_dependencies;
use crate::snob_error;
use globset::GlobSet;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::Path;
use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

pub fn discover_impacted_nodes(
    dependency_graph: &FxHashMap<String, FxHashSet<String>>,
    updated_files: &FxHashSet<String>,
) -> FxHashSet<String> {
    let mut impacted_nodes = FxHashSet::default();
    let mut stack = updated_files.iter().cloned().collect::<Vec<_>>();
    while let Some(file) = stack.pop() {
        if impacted_nodes.contains(&file) {
            continue;
        }

        impacted_nodes.insert(file.clone());
        if let Some(consumers) = dependency_graph.get(&file) {
            stack.extend(consumers.iter().cloned());
        }
    }
    impacted_nodes
}

pub fn discover_impacted_nodes_with_graphviz(
    dependency_graph: &FxHashMap<String, FxHashSet<String>>,
    updated_files: &FxHashSet<String>,
    dot_graph: &PathBuf,
) -> FxHashSet<String> {
    let file_handle = std::fs::File::create(dot_graph).unwrap();
    let mut writer = BufWriter::new(file_handle);
    writeln!(writer, "digraph G {{").unwrap();

    let mut impacted_nodes = FxHashSet::default();
    let mut stack = updated_files.iter().cloned().collect::<Vec<_>>();
    while let Some(file) = stack.pop() {
        if impacted_nodes.contains(&file) {
            continue;
        }

        impacted_nodes.insert(file.clone());
        if let Some(consumers) = dependency_graph.get(&file) {
            stack.extend(consumers.iter().cloned());
            for consumer in consumers {
                writeln!(writer, "    \"{consumer}\" -> \"{file}\";").unwrap();
            }
        }
    }

    writeln!(writer, "}}").unwrap();
    writer.flush().unwrap();
    impacted_nodes
}

pub fn build_dependency_graph(
    workspace_files: &[PathBuf],
    project_files: &FxHashSet<String>,
    file_ignores: &GlobSet,
    first_level_components: &[Vec<PathBuf>],
    git_root: &Path,
) -> Vec<FxHashMap<String, Vec<String>>> {
    workspace_files
        .par_iter()
        .filter(|f| {
            file_ignores
                .matches(PathBuf::from(f).strip_prefix(git_root).unwrap())
                .is_empty()
        })
        .filter_map(|f| {
            if let Ok(graph) = extract_file_dependencies(f, project_files, first_level_components) {
                Some(graph)
            } else {
                snob_error!("Failed to parse file {:?}", f);
                None
            }
        })
        .collect::<Vec<FxHashMap<String, Vec<String>>>>()
}
