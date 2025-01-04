use std::{
    collections::{HashMap, HashSet},
    io::{BufWriter, Write},
    path::PathBuf,
};

pub fn discover_impacted_nodes(
    dependency_graph: &HashMap<String, HashSet<String>>,
    updated_files: &[String],
) -> HashSet<String> {
    let mut impacted_nodes = HashSet::new();
    let mut stack = updated_files.to_owned();
    while let Some(file) = stack.pop() {
        if impacted_nodes.contains(&file) {
            continue;
        }

        impacted_nodes.insert(file.clone());
        if let Some(consumers) = dependency_graph.get(&file) {
            log::debug!("{:?} has the following consumers: {:?}", file, consumers);
            stack.extend(consumers.iter().cloned());
        }
    }
    impacted_nodes
}

pub fn discover_impacted_nodes_with_graphviz(
    dependency_graph: &HashMap<String, HashSet<String>>,
    updated_files: &[String],
    dot_graph: &PathBuf,
) -> HashSet<String> {
    let file_handle = std::fs::File::create(dot_graph).unwrap();
    let mut writer = BufWriter::new(file_handle);
    writeln!(writer, "digraph G {{").unwrap();

    let mut impacted_nodes = HashSet::new();
    let mut stack = updated_files.to_owned();
    while let Some(file) = stack.pop() {
        if impacted_nodes.contains(&file) {
            continue;
        }

        impacted_nodes.insert(file.clone());
        if let Some(consumers) = dependency_graph.get(&file) {
            log::debug!("{:?} has the following consumers: {:?}", file, consumers);
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
