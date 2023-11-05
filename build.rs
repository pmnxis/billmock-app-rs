/*
 * SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use std::process::Command;

use cargo_metadata::{Error, MetadataCommand};
use git2::Repository;
use mp_fingerprint_type::{FirmwareFingerprint, MpFingerprint};

const IGNORE_PATH_DEP_INJ: &str = ".cargo/config.toml";

fn main() -> Result<(), Error> {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Get project name and version
    let metadata = MetadataCommand::new().no_deps().exec()?;

    if let Some(package) = metadata.packages.first() {
        let project_name = &package.name;
        let project_version = package.version.to_string();

        println!("cargo:rustc-env=PROJECT_NAME={}", project_name);
        println!(
            "cargo:rustc-env=PROJECT_VERSION={}",
            hex::encode(project_version),
        );
    } else {
        println!("cargo:rustc-env=PROJECT_NAME=unkown");
        println!("cargo:rustc-env=PROJECT_VERSION={}", hex::encode(b"?.?.?"),);
    }

    // Get the Git commit hash
    let repo = Repository::open(".").expect("Failed to open repository");
    let head = repo.head().expect("Failed to get HEAD");
    let commit = head.peel_to_commit().expect("Failed to peel commit");
    let commit_hash = commit.id().to_string();
    let commit_short_hash = String::from_utf8(
        commit
            .as_object()
            .short_id()
            .expect("Filed to get short_id")
            .to_ascii_lowercase(),
    )
    .expect("Failed to convert short_id to UTF-8");
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
        let p = status.path();

        // ignore config.toml for dependency injection
        (p != Some(IGNORE_PATH_DEP_INJ))
            & !((s == git2::Status::CURRENT) | (s == git2::Status::IGNORED))
    });

    let (dirty_str, short_dirty_str) = if is_dirty {
        ("-dirty".to_owned(), "-d".to_owned())
    } else {
        ("".to_owned(), "  ".to_owned())
    };

    let output = Command::new("git")
        .args(["log", "-1", "--format=%ai", &commit_hash])
        .output()
        .expect("Failed to execute command");
    let commit_datetime = String::from_utf8_lossy(&output.stdout);

    // Output the version and commit hash to a file
    // This is u8 array

    println!(
        "cargo:rustc-env=GIT_COMMIT_HASH={}{}",
        commit_hash, dirty_str
    );

    println!(
        "cargo:rustc-env=GIT_COMMIT_SHORT_HASH={}",
        hex::encode(format!("{}{}", commit_short_hash, short_dirty_str))
    );
    println!("cargo:rustc-env=GIT_COMMIT_DATETIME={}", commit_datetime);

    // Generate elf header fingerprint
    let metadata = MetadataCommand::new().no_deps().exec()?;
    let main_package = metadata
        .packages
        .first()
        .expect("Cargo.toml doesn't have metadata");

    let hw_feature: Vec<(String, String)> = std::env::vars()
        .filter(|(key, value)| key.starts_with("CARGO_FEATURE_HW_") && value == "1")
        .collect();

    if hw_feature.is_empty() {
        panic!("There's no specified hardware target");
    } else if hw_feature.len() > 1 {
        panic!("Cannot specify multiple hardware");
    }

    let feature_based_model_ver = hw_feature[0]
        .0
        .strip_prefix("CARGO_FEATURE_HW_")
        .unwrap()
        .replace('_', "-");

    let fingerprint = MpFingerprint {
        firmware_fingerprint: FirmwareFingerprint {
            model_name: "BillMock-HW".to_owned(), // this is const value
            model_ver: feature_based_model_ver,
            firmware_ver: main_package.version.to_string(),
            firmware_git_hash: format!("{}{}", commit_hash, dirty_str),
        },
    };

    println!(
        "cargo:rustc-env=MP_FINGERPRINT_TOML_HEX={}",
        fingerprint.to_hex_string(),
    );

    Ok(())
}
