use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Parse { path: PathBuf },
}
