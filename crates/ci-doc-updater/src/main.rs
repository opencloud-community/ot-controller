// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use snafu::{ResultExt, Whatever};

use crate::generate::DirectoryFilesProvider;

mod analyze;
mod generate;

#[derive(Subcommand, Debug)]
enum Command {
    Generate {
        /// Directory containing the generated raw files that should be used
        #[arg(long)]
        raw_files_dir: PathBuf,

        #[arg(long)]
        /// Base directory for documentation, will be searched for markdown files recursively
        documentation_dir: PathBuf,
    },
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<(), Whatever> {
    let args = Args::parse();

    match args.command {
        Command::Generate {
            raw_files_dir,
            documentation_dir,
        } => {
            generate_files(&raw_files_dir, &documentation_dir)?;
        }
    };

    Ok(())
}

fn generate_files(raw_files_dir: &Path, documentation_dir: &Path) -> Result<(), Whatever> {
    let files_provider = DirectoryFilesProvider::new(raw_files_dir);

    let pattern = format!("{}/**/*.md", documentation_dir.to_string_lossy());
    let glob_iterator =
        glob::glob(&pattern).with_whatever_context(|err| format!("Invalid glob: {err}"))?;

    for entry in glob_iterator {
        match entry {
            Ok(path) => {
                println!("Updating file {path:?}");
                let contents = std::fs::read_to_string(&path)
                    .with_whatever_context(|err| format!("Couldn't read file {path:?}: {err}"))?;
                let new_contents = generate::generate(&contents, &files_provider)?;
                if contents != new_contents {
                    std::fs::write(&path, new_contents).with_whatever_context(|err| {
                        format!("Couldn't write file {path:?}: {err}")
                    })?;
                }
            }
            Err(e) => {
                eprintln!("glob error: {e:?}");
            }
        }
    }
    Ok(())
}
