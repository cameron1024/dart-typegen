use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Parse the config file at the given path and print a pretty representation of the AST
    Parse { path: PathBuf },

    /// Generate the Dart for a given library definition
    Generate {
        /// Path to the KDL config file
        #[clap(long, short)]
        input: PathBuf,

        /// The path to write the output to. If not provided, it will be printed to stdout
        #[clap(long, short)]
        output: Option<PathBuf>,
    },
}
