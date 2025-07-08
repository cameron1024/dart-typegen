use clap::Parser;

use crate::args::{Args, run};

mod args;
mod codegen;
mod context;
mod model;
mod validate;

#[cfg(test)]
mod tests;

fn main() -> miette::Result<()> {
    run(&Args::parse())
}
