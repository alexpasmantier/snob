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

    /// Verbosity level (0-4)
    /// 0 -> Error
    /// 1 -> Warn
    /// 2 -> Info
    /// 3 -> Debug
    /// 4 or higher -> Trace
    #[arg(short, long, default_value = "2")]
    pub verbosity_level: usize,

    /// Quiet mode
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// Produce DOT graph at provided path
    #[arg(short, long, value_name = "FILE")]
    pub dot_graph: Option<PathBuf>,
}
