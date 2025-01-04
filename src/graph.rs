use std::collections::{HashMap, HashSet};

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
