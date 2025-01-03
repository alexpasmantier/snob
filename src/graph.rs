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
        stack.extend(
            dependency_graph
                .get(&file)
                .unwrap_or(&HashSet::new())
                .iter()
                .cloned(),
        );
    }
    impacted_nodes
}
