// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use vergen::EmitBuilder;

/// Use an environment variable instead of letting `vergen` determine the value.
/// This allows reproducible builds by removing differences between subsequent builds.
/// If the environment variable `name` is set, it will be used to print the cargo instructions,
/// otherwise `enable_fn` will be called and `vergen` will determine what to print.
fn add_custom_environment_variable<F: FnMut(&mut EmitBuilder) -> &mut EmitBuilder>(
    name: &str,
    builder: &mut EmitBuilder,
    mut enable_fn: F,
) {
    if let Ok(s) = std::env::var(name) {
        println!("cargo:rustc-env={name}={s}");
    } else {
        enable_fn(builder);
    }
    println!("cargo:rerun-if-env-changed={name}");
}

fn main() -> Result<()> {
    let builder = &mut EmitBuilder::builder();
    builder
        .all_cargo()
        .all_rustc()
        // The following enables all git features except the ones we want to control manually
        // Taken from `EmitBuilder::all_git`'s implementation
        .git_commit_author_email()
        .git_commit_author_name()
        .git_commit_count()
        .git_commit_date()
        .git_commit_message()
        .git_describe(false, false, None);

    add_custom_environment_variable(
        "VERGEN_BUILD_TIMESTAMP",
        builder,
        EmitBuilder::build_timestamp,
    );

    if git_at_cd_or_above()? {
        add_custom_environment_variable("VERGEN_GIT_SHA", builder, |builder| {
            const USE_SHORT: bool = false;
            builder.git_sha(USE_SHORT)
        });

        add_custom_environment_variable(
            "VERGEN_GIT_COMMIT_TIMESTAMP",
            builder,
            EmitBuilder::git_commit_timestamp,
        );

        add_custom_environment_variable("VERGEN_GIT_BRANCH", builder, EmitBuilder::git_branch);
    } else {
        // Disable git for vergen builder
        builder.disable_git();
    }

    builder.emit()?;

    Ok(())
}

fn git_at_cd_or_above() -> Result<bool> {
    let current_dir = std::env::current_dir()?;
    let mut parents = vec![];
    let mut path = &*current_dir.canonicalize()?;
    while let Some(parent) = path.parent() {
        parents.push(parent);
        path = parent;
    }
    for parent in parents.into_iter().rev() {
        if parent.join(".git").exists() {
            return Ok(true);
        }
    }
    Ok(false)
}
