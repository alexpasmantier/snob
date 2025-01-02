use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub target_directory: Option<PathBuf>,

    #[arg(value_name = "FILE")]
    pub updated_files: Vec<PathBuf>,
}
