// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::{ResultExt, Whatever};
use vergen::{BuildBuilder, CargoBuilder, RustcBuilder};
use vergen_gix::{Emitter, GixBuilder};

fn main() -> Result<(), Whatever> {
    let mut emitter = Emitter::default();
    let builder = &mut emitter
        .add_instructions(
            &CargoBuilder::all_cargo().whatever_context("Failed to build cargo variables")?,
        )
        .whatever_context("Failed to add cargo instructions")?
        .add_instructions(
            &BuildBuilder::all_build().whatever_context("Failed to build builder variables")?,
        )
        .whatever_context("Failed to add builder variables")?
        .add_instructions(
            &RustcBuilder::all_rustc().whatever_context("Failed to build rustc variables")?,
        )
        .whatever_context("Failed to add rustc variables")?;

    if is_contained_in_git()? {
        builder
            .add_instructions(
                &GixBuilder::all_git().whatever_context("Failed to build git variables")?,
            )
            .whatever_context("Failed to add git variables")?;
    }
    builder.emit().whatever_context("Failed to emit")?;

    Ok(())
}

/// Checks wether the current or one of the parent directories contains a `.git` entry.
fn is_contained_in_git() -> Result<bool, Whatever> {
    let current_dir = std::env::current_dir().whatever_context("Failed to get current dir")?;
    let mut parents = vec![];
    let mut path = &*current_dir
        .canonicalize()
        .whatever_context("Failed to canonicalize path")?;
    parents.push(path);
    while let Some(parent) = path.parent() {
        parents.push(parent);
        path = parent;
    }
    for parent in parents.into_iter().rev() {
        if parent.join(".git").exists() {
            return Ok(true);
        }
    }

    println!("cargo::warning=No .git directory found");
    Ok(false)
}
