use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// The target directory to analyze
    #[arg(short, long, value_name = "FILE", default_value = ".")]
    pub target_directory: PathBuf,

    /// Files that were modified by the patch
    #[arg(value_name = "FILE")]
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
