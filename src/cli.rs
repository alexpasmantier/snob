use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = ".")]
    pub target_directory: PathBuf,

    #[arg(value_name = "FILE")]
    pub updated_files: Vec<String>,
}
