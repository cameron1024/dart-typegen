use clap::Parser;

use crate::{args::{Args, Cmd}, model::Config};

mod args;
mod context;
// mod codegen;
mod model;

#[cfg(test)]
mod tests;

fn main() {
    let args = Args::parse();

    match &args.cmd {
        Cmd::Parse { path }  => {
            let config = Config::parse_file(path).unwrap();
            println!("{config:#?}");
        },
    }
}
