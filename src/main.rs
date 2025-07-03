use std::{fs::File, io::BufWriter};

use clap::Parser;
use miette::IntoDiagnostic;

use crate::{
    args::{Args, Cmd},
    context::Context,
    model::Library,
};

mod args;
mod codegen;
mod context;
mod model;
mod validate;

#[cfg(test)]
mod tests;

fn main() -> miette::Result<()> {
    let args = Args::parse();

    match &args.cmd {
        Cmd::Parse { path } => {
            let context = Context::from_path(path)?;
            context.validate()?;

            let library = &context.library;
            println!("{library:#?}");
        }
        Cmd::Generate { input, output } => {
            let context = Context::from_path(path)?;
            context.validate()?;

            let output = File::create(&output).into_diagnostic()?;
            let output = BufWriter::new(output);

            codegen::codegen(&mut output, &context.library)?;
        }
    }

    Ok(())
}
