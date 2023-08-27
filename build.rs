/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use std::process::Command;

use cargo_metadata::{Error, MetadataCommand};
use git2::Repository;

fn main() -> Result<(), Error> {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Get project name and version
    let metadata = MetadataCommand::new().no_deps().exec()?;

    if let Some(package) = metadata.packages.first() {
        let project_name = &package.name;
        let project_version = &package.version.to_string();

        println!("cargo:rustc-env=PROJECT_NAME={}", project_name);
        println!("cargo:rustc-env=PROJECT_VERSION={}", project_version);
    } else {
        println!("cargo:rustc-env=PROJECT_NAME=unkown");
        println!("cargo:rustc-env=PROJECT_VERSION=unknown");
    }

    // Get the Git commit hash
    let repo = Repository::open(".").expect("Failed to open repository");
    let head = repo.head().expect("Failed to get HEAD");
    let commit = head.peel_to_commit().expect("Failed to peel commit");
    let commit_hash = commit.id().to_string();
    let statuses = match repo.statuses(None) {
        Ok(statuses) => statuses,
        Err(_) => {
            return Err(Error::CargoMetadata {
                stderr: "Failed to open git repository".to_owned(),
            })
        } // Failed to get repository status
    };

    let is_dirty = statuses.iter().any(|status| {
        let s = status.status();
        !((s == git2::Status::CURRENT) & (s == git2::Status::IGNORED))
    });

    let dirty_str = if is_dirty {
        "-dirty".to_owned()
    } else {
        "".to_owned()
    };

    let output = Command::new("git")
        .args(["log", "-1", "--format=%ai", &commit_hash])
        .output()
        .expect("Failed to execute command");
    let commit_datetime = String::from_utf8_lossy(&output.stdout);

    // Output the version and commit hash to a file

    println!(
        "cargo:rustc-env=GIT_COMMIT_HASH={}{}",
        commit_hash, dirty_str
    );
    println!("cargo:rustc-env=GIT_COMMIT_DATETIME={}", commit_datetime);

    Ok(())
}
