use crate::utils::is_test_file;
use globset::GlobSet;
use rustc_hash::FxHashSet;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct SnobResult {
    pub impacted: HashSet<String>,
    pub always_run: HashSet<String>,
    pub ignored: HashSet<String>,
}

impl SnobResult {
    pub fn new(
        impacted: FxHashSet<String>,
        workspace_files: FxHashSet<String>,
        ignore_glob: &GlobSet,
        always_run_glob: &GlobSet,
        git_root: &Path,
    ) -> Self {
        let always_run_tests = workspace_files
            .into_iter()
            .filter(|f| {
                !always_run_glob
                    .matches(PathBuf::from(f).strip_prefix(git_root).unwrap())
                    .is_empty()
                    && is_test_file(f)
            })
            .collect::<HashSet<String>>();

        let impacted_tests = impacted
            .into_iter()
            .filter(|f| is_test_file(f))
            .collect::<HashSet<String>>();

        let ignored_tests = impacted_tests
            .iter()
            .map(std::string::ToString::to_string)
            .filter(|f| {
                !ignore_glob
                    .matches(PathBuf::from(f).strip_prefix(git_root).unwrap())
                    .is_empty()
            })
            .collect::<HashSet<String>>();
        Self {
            impacted: impacted_tests.difference(&ignored_tests).cloned().collect(),
            always_run: always_run_tests,
            ignored: ignored_tests,
        }
    }
}

