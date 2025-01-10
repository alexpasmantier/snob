use globset::GlobSet;
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

pub fn merge_hashmaps<K, V, A>(hashmaps: &mut [FxHashMap<K, V>]) -> FxHashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone + Extend<A> + IntoIterator<Item = A>,
{
    if hashmaps.is_empty() {
        return FxHashMap::default();
    }
    hashmaps
        .iter_mut()
        .reduce(|acc, hashmap| {
            for (k, v) in hashmap.iter() {
                acc.entry(k.clone())
                    .and_modify(|acc_v| {
                        acc_v.extend(v.clone());
                    })
                    .or_insert_with(|| v.clone());
            }
            acc
        })
        .unwrap()
        .clone()
}

const PYTHONPATH_ENV: &str = "PYTHONPATH";

pub fn is_test_file<P>(file: P) -> bool
where
    P: AsRef<Path>,
{
    let file = file.as_ref();
    file.file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("test_")
        || file.to_string_lossy().ends_with("_test.py")
}

#[derive(Debug, PartialEq)]
pub struct LookupPaths {
    // to retain discovery order
    pub local_paths: Vec<PathBuf>,
    path_set: FxHashSet<PathBuf>,
}

impl LookupPaths {
    pub fn new() -> Self {
        Self {
            local_paths: Vec::new(),
            path_set: FxHashSet::default(),
        }
    }

    pub fn insert(&mut self, path: PathBuf) {
        if self.path_set.insert(path.clone()) {
            self.local_paths.push(path);
        }
    }
}

// PYTHONPATH
// python's import paths: [cwd, PYTHONPATH, others]
pub fn get_python_local_lookup_paths(current_dir: &PathBuf, git_root: &PathBuf) -> LookupPaths {
    // ordered
    let pythonpath = get_pythonpath();
    let mut local_paths = vec![current_dir.clone()];
    local_paths.extend(pythonpath);
    let mut lookup_paths = LookupPaths::new();
    local_paths
        .into_iter()
        .filter(|p| p.starts_with(git_root))
        .for_each(|p| lookup_paths.insert(p));
    lookup_paths
}

#[cfg(target_os = "windows")]
pub const PYTHONPATH_SEPARATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const PYTHONPATH_SEPARATOR: &str = ":";

pub fn get_pythonpath() -> Vec<PathBuf> {
    let p = std::env::var(PYTHONPATH_ENV).unwrap_or_default();
    if p.is_empty() {
        return vec![];
    }
    p.split(PYTHONPATH_SEPARATOR).map(PathBuf::from).collect()
}

pub fn get_repo_root(current_dir: &PathBuf) -> PathBuf {
    let mut path = current_dir.clone();
    // let's cross our fingers here
    while !path.join(".git").exists() {
        path = path.parent().unwrap().to_path_buf();
    }
    path
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;

    use super::*;

    #[test]
    fn test_merge_hashmaps() {
        let mut hm1 = FxHashMap::default();
        hm1.insert("a", vec![1, 2]);
        hm1.insert("b", vec![2, 3]);

        let mut hm2 = FxHashMap::default();
        hm2.insert("a", vec![1, 2, 4]);
        hm2.insert("b", vec![]);

        let mut hm3 = FxHashMap::default();
        hm3.insert("a", vec![1, 2]);
        hm3.insert("c", vec![7, 8, 9]);

        let mut hashmaps = vec![hm1, hm2, hm3];
        let merged = merge_hashmaps(&mut hashmaps);

        let mut expected = FxHashMap::default();
        expected.insert("a", vec![1, 2, 1, 2, 4, 1, 2]);
        expected.insert("b", vec![2, 3]);
        expected.insert("c", vec![7, 8, 9]);

        assert_eq!(merged, expected);
    }

    #[test]
    fn test_get_pythonpath_empty() {
        std::env::remove_var(PYTHONPATH_ENV);
        let pythonpath = get_pythonpath();
        assert!(pythonpath.is_empty());
    }

    #[test]
    fn test_get_pythonpath() {
        std::env::set_var(PYTHONPATH_ENV, "/some/path:/another/path");
        let pythonpath = get_pythonpath();
        assert_eq!(
            pythonpath,
            vec![PathBuf::from("/some/path"), PathBuf::from("/another/path")]
        );
    }

    #[test]
    fn test_get_python_local_lookup_paths() {
        let current_dir = PathBuf::from("/home/user/project/src");
        let git_root = PathBuf::from("/home/user/project");
        let pythonpath = vec![
            PathBuf::from("/home/user/project/src"),
            PathBuf::from("/home/user/project/lib"),
            PathBuf::from("/home/user/other-project/src/lib"),
        ];
        std::env::set_var(
            PYTHONPATH_ENV,
            pythonpath
                .iter()
                .map(|p| p.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(":"),
        );
        let local_paths = get_python_local_lookup_paths(&current_dir, &git_root);
        assert_eq!(
            local_paths,
            LookupPaths {
                local_paths: vec![
                    PathBuf::from("/home/user/project/src"),
                    PathBuf::from("/home/user/project/lib"),
                ],
                path_set: vec![
                    PathBuf::from("/home/user/project/src"),
                    PathBuf::from("/home/user/project/lib"),
                ]
                .into_iter()
                .collect()
            }
        );
    }
}

pub fn should_run_all_tests(
    updated_files: &FxHashSet<String>,
    run_all_tests_on_change: &GlobSet,
    git_root: &Path,
) -> bool {
    updated_files.iter().any(|f| {
        !run_all_tests_on_change
            .matches(PathBuf::from(f).strip_prefix(&git_root).unwrap())
            .is_empty()
    })
}

pub fn deduplicate_dependencies(
    dependencies: FxHashMap<String, Vec<String>>,
) -> FxHashMap<String, FxHashSet<String>> {
    dependencies
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                v.iter()
                    .map(std::string::ToString::to_string)
                    .collect::<FxHashSet<_>>(),
            )
        })
        .collect::<FxHashMap<String, FxHashSet<String>>>()
}

