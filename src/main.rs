use clap::Parser;

use crate::{args::{Args, Cmd}, model::Library};

mod args;
mod codegen;
mod context;
mod model;

#[cfg(test)]
mod tests;

fn main() {
    let args = Args::parse();

    match &args.cmd {
        Cmd::Parse { path }  => {
            let config = Library::parse_file(path).unwrap();
            println!("{config:#?}");
        },
    }
}
