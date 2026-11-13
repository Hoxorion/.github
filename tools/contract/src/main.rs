// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 NullSablex and the Hoxorion contributors

//! The checks every Hoxorion repository is held to, run by the shared CI workflow and by
//! hand from a repository's root:
//!
//! ```text
//! hoxorion-contract licensing --license MPL-2.0
//! hoxorion-contract dco --base <sha> --head <sha>
//! hoxorion-contract normalize --license MPL-2.0 --terms <folder with LICENSE-*, DCO, CLA.md>
//! hoxorion-contract message --signoff "Name <email>"   (a commit message, stdin to stdout)
//! ```

mod dco;
mod git;
mod licensing;
mod message;
mod normalize;

use std::process::ExitCode;

type Result<T> = std::result::Result<T, String>;

const USAGE: &str = "usage:\n  hoxorion-contract licensing --license <SPDX id>\n  hoxorion-contract dco --base <sha> --head <sha>\n  hoxorion-contract normalize --license <SPDX id> --terms <folder>\n  hoxorion-contract message --signoff <Name <email>>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("licensing") => licensing::run(&args[1..]),
        Some("dco") => dco::run(&args[1..]),
        Some("normalize") => normalize::run(&args[1..]),
        Some("message") => message::run(&args[1..]),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The value after `flag` in `args`, which must hold nothing else but `flags`.
fn flag<'a>(args: &'a [String], flag: &str) -> Result<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|at| args.get(at + 1))
        .map(String::as_str)
        .ok_or_else(|| format!("missing {flag}\n{USAGE}"))
}
