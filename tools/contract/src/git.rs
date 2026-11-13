// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 NullSablex and the Hoxorion contributors

//! Asking git, in the current directory: the repository being checked.

use std::process::Command;

use crate::Result;

pub fn git(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output().map_err(|error| format!("running git: {error}"))?;
    if !output.status.success() {
        return Err(format!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim()));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("git {} printed something that is not UTF-8", args.join(" ")))
}
