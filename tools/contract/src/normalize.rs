// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 NullSablex and the Hoxorion contributors

//! Brings a working tree under the contract: the licence text, the DCO and the CLA in place,
//! every `Cargo.toml` naming the licence and the holders, and every source file opening with
//! its SPDX identifier and holders. Run from the tree's root; what is already right is left
//! alone, so it can run over and over (it is what rewrote each repository's history, commit
//! by commit, so the terms hold from the first one).
//!
//! It walks the folder rather than asking git, since a history rewrite hands it a bare
//! checkout. Build output, hidden folders and other projects' files (`legacy/` mirrors) are
//! skipped, and so are test fixtures, whose compiled output was produced from them as they
//! were.

use std::fs;
use std::path::{Path, PathBuf};

use crate::licensing::HOLDERS;
use crate::{Result, flag};

const SOURCES: [&str; 4] = ["rs", "wgsl", "pwn", "inc"];
const SKIPPED: [&str; 3] = ["target", "legacy", "fixtures"];

pub fn run(args: &[String]) -> Result<()> {
    let license = flag(args, "--license")?;
    let terms = PathBuf::from(flag(args, "--terms")?);
    let license_text = terms.join(match license {
        "MPL-2.0" => "LICENSE-MPL-2.0",
        "Apache-2.0" => "LICENSE-Apache-2.0",
        other => return Err(format!("{other} is not a licence Hoxorion uses")),
    });
    let copy = |from: &Path, to: &str| -> Result<()> {
        let text = fs::read(from).map_err(|error| format!("{}: {error}", from.display()))?;
        if fs::read(to).ok().as_deref() != Some(&text[..]) {
            fs::write(to, text).map_err(|error| format!("{to}: {error}"))?;
        }
        Ok(())
    };
    copy(&license_text, "LICENSE")?;
    copy(&terms.join("DCO"), "DCO")?;
    copy(&terms.join("CLA.md"), "CLA.md")?;

    // Test fixtures are left alone unless asked: their compiled output was produced from
    // them as they are, and a fixture compiled with debug lines must be compiled again.
    let skipped: Vec<&str> = SKIPPED.iter().copied().filter(|name| *name != "fixtures" || !args.iter().any(|arg| arg == "--include-fixtures")).collect();
    let mut files = Vec::new();
    walk(Path::new("."), &skipped, &mut files)?;
    for file in files {
        let name = file.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        let extension = file.extension().and_then(|extension| extension.to_str()).unwrap_or_default();
        if name == "Cargo.toml" {
            let text = read(&file)?;
            let fixed = fix_manifest(&text, license);
            if fixed != text {
                write(&file, &fixed)?;
            }
        } else if SOURCES.contains(&extension) {
            let text = read(&file)?;
            if !crate::licensing::has_header(&text, license) {
                write(&file, &with_header(&text, license))?;
            }
        }
    }
    Ok(())
}

fn walk(folder: &Path, skipped: &[&str], found: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(folder).map_err(|error| format!("{}: {error}", folder.display()))?.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if !name.starts_with('.') && !skipped.contains(&name.as_str()) {
                walk(&path, skipped, found)?;
            }
        } else {
            found.push(path);
        }
    }
    Ok(())
}

fn read(file: &Path) -> Result<String> {
    fs::read_to_string(file).map_err(|error| format!("{}: {error}", file.display()))
}

fn write(file: &Path, text: &str) -> Result<()> {
    fs::write(file, text).map_err(|error| format!("{}: {error}", file.display()))
}

/// `text` with the licence header on top. A header in another form (another licence, other
/// holders) is replaced, never stacked under a new one.
pub fn with_header(text: &str, license: &str) -> String {
    let mut body = text.trim_start_matches('\u{feff}');
    loop {
        let first = body.lines().next().unwrap_or_default();
        if first.starts_with("// SPDX-License-Identifier:") || first.starts_with("// Copyright ©") {
            body = body.split_once('\n').map_or("", |(_, rest)| rest);
        } else {
            break;
        }
    }
    let body = body.trim_start_matches('\n');
    format!("// SPDX-License-Identifier: {license}\n// Copyright © 2026 {HOLDERS}\n\n{body}")
}

/// `text` (a `Cargo.toml`) with its package's `license` and `authors` set; other lines are
/// kept as written. A manifest with no `[package]` (a workspace root) is left alone.
pub fn fix_manifest(text: &str, license: &str) -> String {
    let license_line = format!("license = \"{license}\"");
    let authors_line = format!("authors = [\"{HOLDERS}\"]");
    let mut out: Vec<String> = Vec::new();
    let mut section = String::new();
    let (mut has_license, mut has_authors, mut package_at) = (false, false, None);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            section = trimmed.to_owned();
        }
        if section == "[package]" {
            if trimmed == "[package]" {
                package_at = Some(out.len());
            } else if trimmed.starts_with("license ") || trimmed.starts_with("license=") {
                has_license = true;
                out.push(license_line.clone());
                continue;
            } else if trimmed.starts_with("authors ") || trimmed.starts_with("authors=") {
                has_authors = true;
                out.push(authors_line.clone());
                continue;
            }
        }
        out.push(line.to_owned());
    }
    let Some(at) = package_at else { return text.to_owned() };
    // Missing keys go right after the package's name, where a reader looks for them.
    let insert_at = out.iter().skip(at + 1).position(|line| line.trim_start().starts_with("name")).map_or(at + 1, |offset| at + 1 + offset + 1);
    if !has_license {
        out.insert(insert_at, license_line);
    }
    if !has_authors {
        out.insert(insert_at, authors_line);
    }
    let mut fixed = out.join("\n");
    if text.ends_with('\n') {
        fixed.push('\n');
    }
    fixed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_header_is_added_once_and_replaces_a_wrong_one() {
        let fixed = with_header("//! Docs.\nfn main() {}\n", "MPL-2.0");
        assert!(crate::licensing::has_header(&fixed, "MPL-2.0"));
        assert!(fixed.ends_with("//! Docs.\nfn main() {}\n"));
        let relicensed = with_header(&fixed, "Apache-2.0");
        assert!(crate::licensing::has_header(&relicensed, "Apache-2.0"));
        assert_eq!(relicensed.matches("SPDX").count(), 1);
    }

    #[test]
    fn a_manifest_gets_the_licence_and_holders_and_keeps_the_rest() {
        let old = "[package]\nname = \"x\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n\n[dependencies]\nlicense = \"not a key here\"\n";
        let fixed = fix_manifest(old, "MPL-2.0");
        assert!(crate::licensing::manifest_problems("Cargo.toml", &fixed, "MPL-2.0").is_empty(), "{fixed}");
        assert!(fixed.contains("license = \"not a key here\""), "other sections untouched");
        assert!(!fixed.contains("MIT"));
        assert_eq!(fix_manifest(&fixed, "MPL-2.0"), fixed, "running again changes nothing");
        assert_eq!(fix_manifest("[workspace]\nmembers = []\n", "MPL-2.0"), "[workspace]\nmembers = []\n");
    }
}
