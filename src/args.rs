use std::{
    fs::File,
    io::{BufWriter, stdout},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;

use crate::{context::Context};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,

    #[clap(long, short, default_value_t = false)]
    pub deny_warnings: bool,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Parse the config file at the given path and check for errors
    Validate { path: PathBuf },

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

pub fn run(args: &Args) -> miette::Result<()> {
    match &args.cmd {
        Cmd::Validate { path } => {
            let context = Context::from_path(path)?;
            context.validate(args.deny_warnings)?;
        }
        Cmd::Generate { input, output } => {
            let context = Context::from_path(input)?;
            context.validate(args.deny_warnings)?;

            match &output {
                Some(output) => {
                    let output = File::create(output).into_diagnostic()?;
                    let mut output = BufWriter::new(output);

                    context.codegen(&mut output)?;
                }
                None => {
                    let mut output = BufWriter::new(stdout().lock());
                    context.codegen(&mut output)?;
                }
            }
        }
    }

    Ok(())
}
