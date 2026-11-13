// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 NullSablex and the Hoxorion contributors

//! Licensing: the licence text is there, every manifest names the licence and the holders,
//! every source file opens with its SPDX identifier and holders, and the contribution terms
//! (DCO and CLA) are in the repository.
//!
//! Only files git tracks are judged: build output and what `.gitignore` leaves out are not
//! the repository's, and other projects' files kept beside it (an include mirror) are never
//! committed under our notice.

use std::fs;

use crate::{Result, flag, git::git};

pub const HOLDERS: &str = "NullSablex and the Hoxorion contributors";
/// Files written by hand: Rust, WGSL shaders, and Pawn scripts and includes.
const SOURCES: [&str; 4] = ["rs", "wgsl", "pwn", "inc"];
/// What contributing is accepted under, in every repository.
const CONTRIBUTION_TERMS: [&str; 2] = ["DCO", "CLA.md"];

pub fn run(args: &[String]) -> Result<()> {
    let license = flag(args, "--license")?;
    let title = match license {
        "MPL-2.0" => "Mozilla Public License Version 2.0",
        "Apache-2.0" => "Apache License",
        other => return Err(format!("{other} is not a licence Hoxorion uses (MPL-2.0 or Apache-2.0)")),
    };
    let files: Vec<String> = git(&["ls-files", "-z"])?.split('\0').filter(|f| !f.is_empty()).map(str::to_owned).collect();
    let mut problems = Vec::new();

    match fs::read_to_string("LICENSE") {
        Ok(text) if text.contains(title) => {}
        Ok(_) => problems.push(format!("LICENSE is not the {license} text")),
        Err(_) => problems.push("there is no LICENSE".to_owned()),
    }
    for terms in CONTRIBUTION_TERMS {
        if !files.iter().any(|file| file == terms) {
            problems.push(format!("there is no {terms}: contributions are accepted under it"));
        }
    }
    for manifest in files.iter().filter(|file| file.rsplit('/').next() == Some("Cargo.toml")) {
        let text = fs::read_to_string(manifest).map_err(|error| format!("{manifest}: {error}"))?;
        problems.extend(manifest_problems(manifest, &text, license));
    }
    let sources = files.iter().filter(|file| file.rsplit_once('.').is_some_and(|(_, extension)| SOURCES.contains(&extension)));
    let mut checked = 0;
    for source in sources {
        checked += 1;
        let text = fs::read_to_string(source).map_err(|error| format!("{source}: {error}"))?;
        if !has_header(&text, license) {
            problems.push(format!("{source} does not open with the licence header"));
        }
    }
    if !problems.is_empty() {
        return Err(format!(
            "{} licensing problem(s):\n  {}\nA source file opens with:\n  // SPDX-License-Identifier: {license}\n  // Copyright © <year> {HOLDERS}",
            problems.len(),
            problems.join("\n  ")
        ));
    }
    println!("licensing holds: {license}, {checked} source file(s), every manifest, LICENSE, DCO and CLA.md");
    Ok(())
}

/// What is wrong with one `Cargo.toml`.
pub fn manifest_problems(path: &str, text: &str, license: &str) -> Vec<String> {
    let has = |wanted: &str| text.lines().any(|line| line.trim() == wanted);
    let mut problems = Vec::new();
    if !has(&format!("license = \"{license}\"")) {
        problems.push(format!("{path} does not say license = \"{license}\""));
    }
    if !has(&format!("authors = [\"{HOLDERS}\"]")) {
        problems.push(format!("{path} does not say authors = [\"{HOLDERS}\"]"));
    }
    problems
}

/// Whether `text` opens with the SPDX line and the holders. The year is the one the file
/// was written in (or a span of years), so any is accepted.
pub fn has_header(text: &str, license: &str) -> bool {
    let mut lines = text.lines();
    let spdx = lines.next().map(|line| line.trim_start_matches('\u{feff}')) == Some(&format!("// SPDX-License-Identifier: {license}"));
    let holders = lines.next().is_some_and(|line| {
        line.strip_prefix("// Copyright © ").and_then(|rest| rest.split_once(' ')).is_some_and(|(years, who)| {
            !years.is_empty() && years.chars().all(|c| c.is_ascii_digit() || c == '-') && who == HOLDERS
        })
    });
    spdx && holders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_header_needs_both_lines_in_order() {
        let good = "// SPDX-License-Identifier: MPL-2.0\n// Copyright © 2026 NullSablex and the Hoxorion contributors\n\nfn main() {}";
        assert!(has_header(good, "MPL-2.0"));
        assert!(has_header(&good.replace("2026", "2026-2027"), "MPL-2.0"));
        assert!(!has_header(good, "Apache-2.0"), "the other licence");
        assert!(!has_header(&good.replace("NullSablex", "Someone"), "MPL-2.0"), "other holders");
        assert!(!has_header(&good.replace("2026 ", ""), "MPL-2.0"), "no year");
        assert!(!has_header("fn main() {}", "MPL-2.0"));
        assert!(!has_header(&format!("\n{good}"), "MPL-2.0"), "not the first lines");
    }

    #[test]
    fn a_manifest_names_the_licence_and_the_holders() {
        let good = "[package]\nname = \"x\"\nauthors = [\"NullSablex and the Hoxorion contributors\"]\nlicense = \"MPL-2.0\"\n";
        assert!(manifest_problems("Cargo.toml", good, "MPL-2.0").is_empty());
        assert_eq!(manifest_problems("Cargo.toml", good, "Apache-2.0").len(), 1);
        assert_eq!(manifest_problems("Cargo.toml", &good.replace(" and the Hoxorion contributors", ""), "MPL-2.0").len(), 1);
        assert_eq!(manifest_problems("Cargo.toml", "[package]\nname = \"x\"\n", "MPL-2.0").len(), 2);
    }
}
