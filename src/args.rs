use std::{
    ffi::OsStr,
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

                    if entry.path().extension() != Some(OsStr::new("kdl")) {
                        continue;
                    }

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

                    println!("generating {}", entry.path().to_string_lossy());

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

    let output_path = output_path
        .map(|p| &p.value)
        .cloned()
        .unwrap_or_else(|| input_path.with_extension("dart"));

    std::fs::write(output_path, output).into_diagnostic()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use clap::Parser;
    use tempdir::TempDir;

    use crate::{
        args::{Args, run},
        test_file,
    };

    #[test]
    fn walks_directories_recursively() {
        let dir = TempDir::new("dart-typegen-test").unwrap();

        std::fs::create_dir(dir.path().join("directory")).unwrap();
        std::fs::write(
            dir.path().join("foo.kdl"),
            include_str!(test_file!(class_docs)),
        )
        .unwrap();
        // std::fs::write(
        //     dir.path().join("ignored.kdl"),
        //     include_str!(test_file!(class_docs)),
        // )
        // .unwrap();
        std::fs::write(
            dir.path().join("directory").join("bar.kdl"),
            include_str!(test_file!(class_docs)),
        )
        .unwrap();

        // std::fs::write(dir.path().join(".gitignore"), "/ignored.kdl\n").unwrap();
        std::fs::write(dir.path().join("not-kdl.foobar"), "something something").unwrap();

        run(&Args::parse_from([
            OsStr::new("dart-typegen"),
            OsStr::new("generate"),
            dir.path().as_os_str(),
        ]))
        .unwrap();

        assert!(dir.path().join("foo.dart").exists());
        // assert!(!dir.path().join("ignored.dart").exists());
        assert!(dir.path().join("directory").join("bar.dart").exists());
    }
}
