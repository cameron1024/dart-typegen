use std::{
    fs::File,
    io::{BufWriter, stdout},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;

use crate::{codegen, context::Context};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,
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
            context.validate()?;
        }
        Cmd::Generate { input, output } => {
            let context = Context::from_path(&input)?;
            context.validate()?;

            match &output {
                Some(output) => {
                    let output = File::create(&output).into_diagnostic()?;
                    let mut output = BufWriter::new(output);

                    codegen::codegen(context, &mut output)?;
                }
                None => {
                    let mut output = BufWriter::new(stdout().lock());
                    codegen::codegen(context, &mut output)?;
                }
            }
        }
    }

    Ok(())
}
