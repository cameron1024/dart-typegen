use std::{
    ops::Deref,
    path::{Path, PathBuf},
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;

use crate::context::Context;

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
    /// Parse the config file at the given path and check for errors.
    Validate { path: PathBuf },

    /// Generate the Dart for a given library definition
    Generate {
        /// Path to the KDL config file, or a directory containing KDL config files.
        path: PathBuf,
    },
}

pub fn run(args: &Args) -> miette::Result<()> {
    match &args.cmd {
        Cmd::Validate { path } => {
            let context = Context::from_path(path)?;
            context.validate(args.deny_warnings)?;
        }
        Cmd::Generate { path } => {
            let has_errors = rayon::scope(|scope| {
                let has_errors = Arc::new(AtomicBool::new(false));

                for result in ignore::Walk::new(path) {
                    let entry = match result {
                        Ok(entry) => entry,
                        Err(e) => {
                            eprintln!("failed to read FS entry, skipping: {e}");
                            continue;
                        }
                    };

                    let Ok(meta) = entry.metadata() else {
                        eprintln!(
                            "failed to read metadata for {}, skipping",
                            entry.path().to_string_lossy()
                        );
                        continue;
                    };

                    if meta.is_dir() {
                        continue;
                    }

                    let deny_warnings = args.deny_warnings;

                    let has_errors = Arc::clone(&has_errors);
                    scope.spawn(move |_scope| {
                        if let Err(e) = generate_single(entry.path(), deny_warnings) {
                            eprintln!("{e:?}");
                            has_errors.store(true, Ordering::SeqCst);
                        }
                    });
                }

                has_errors.load(Ordering::SeqCst)
            });

            if has_errors {
                exit(1);
            }
        }
    }

    Ok(())
}

fn generate_single(input_path: &Path, deny_warnings: bool) -> miette::Result<()> {
    let context = Context::from_path(input_path)?;
    context.validate(deny_warnings)?;
    let (output, output_path) = context.codegen()?;

    match output_path {
        Some(path) => std::fs::write(path.deref(), output).into_diagnostic()?,
        None => {
            eprintln!(
                "no `output.path` directive for {} - writing to stdout",
                input_path.to_string_lossy()
            );
            println!("{output}");
        }
    }

    Ok(())
}
