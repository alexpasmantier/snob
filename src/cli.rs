use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Find tests impacted by code changes",
    long_about = "Snob analyzes your Python code changes and identifies which tests need to run.\n\nUsage examples:\n  git diff --name-only | snob | xargs pytest\n  snob src/auth.py src/utils.py\n  git diff --name-only HEAD~1..HEAD | snob | xargs pytest"
)]
pub struct Cli {
    /// The target directory to analyze for dependencies
    #[arg(short, long, value_name = "DIR", default_value = ".")]
    pub target_directory: PathBuf,

    /// Python files that have been changed/modified (e.g., from git diff, your editor, etc.)
    /// These are the files you want to find tests for
    #[arg(
        value_name = "CHANGED_FILES",
        help = "Python files that were modified and need testing.
These can easily be obtained using your version control system (e.g., `git diff --name-only`)"
    )]
    pub updated_files: Vec<String>,

    /// Verbosity level (0-4+)
    /// 0 -> ERROR
    /// 1 -> WARN
    /// 2 -> INFO
    /// 3 -> DEBUG
    /// 4 or higher -> TRACE
    #[arg(short, long, default_value = "2")]
    pub verbosity_level: usize,

    /// Quiet mode
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// Produce DOT graph at provided path
    /// see https://graphviz.org/doc/info/lang.html
    #[arg(short, long, value_name = "FILE")]
    pub dot_graph: Option<PathBuf>,
}
