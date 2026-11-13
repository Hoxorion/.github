// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 NullSablex and the Hoxorion contributors

//! DCO and CLA: every commit of a pull request carries its author's own
//! `Signed-off-by: Name <email>`, and every co-author's. Under the repository's `CLA.md`, that
//! sign-off is how a contributor certifies the DCO and accepts the CLA for the contribution.
//!
//! Only git's own trailer parsing counts, so prose that mentions a sign-off certifies
//! nothing. The check establishes that the trailers are there, not who wrote them: identity,
//! provenance and an employer's authorization stay with the review.

use crate::{Result, flag, git::git};

const DEPENDABOT_AUTHOR: &str = "dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>";
const DEPENDABOT_SIGNOFF: &str = "dependabot[bot] <support@github.com>";

pub fn run(args: &[String]) -> Result<()> {
    let base = flag(args, "--base")?;
    let head = flag(args, "--head")?;
    for sha in [base, head] {
        if sha.len() != 40 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("{sha} is not a full 40-character commit SHA"));
        }
        git(&["cat-file", "-e", &format!("{sha}^{{commit}}")])?;
    }
    let commits = git(&["rev-list", "--reverse", &format!("{base}..{head}")])?;
    if commits.trim().is_empty() {
        return Err("the range holds no commits".to_owned());
    }
    let mut failed = Vec::new();
    for sha in commits.lines() {
        let field = |format: &str| git(&["show", "-s", &format!("--format={format}"), sha]);
        let author = field("%an <%ae>")?;
        let signoffs = field("%(trailers:key=Signed-off-by,valueonly,unfold=true)")?;
        let coauthors = field("%(trailers:key=Co-authored-by,valueonly,unfold=true)")?;
        if let Err(why) = check(&author, &signoffs, &coauthors) {
            failed.push(format!("{sha}: {why}"));
        }
    }
    if !failed.is_empty() {
        return Err(format!(
            "{} commit(s) without the sign-off the DCO and the CLA ask for:\n  {}\nSign them off with your own name and email and push again:\n  git rebase --signoff <base branch>\n  git push --force-with-lease",
            failed.len(),
            failed.join("\n  ")
        ));
    }
    println!("DCO and CLA sign-offs hold for {} commit(s)", commits.lines().count());
    Ok(())
}

/// Whether `identity` is `Name <email>` with both parts present and nothing stray in the
/// address.
fn valid_identity(identity: &str) -> bool {
    let Some((name, email)) = identity.trim().rsplit_once(" <") else { return false };
    let Some(email) = email.strip_suffix('>') else { return false };
    !name.trim().is_empty() && email.contains('@') && !email.chars().any(|c| c.is_whitespace() || c == '<' || c == '>')
}

/// The author and every co-author signed off, each with their own identity.
fn check(author: &str, signoffs: &str, coauthors: &str) -> Result<()> {
    let signoffs: Vec<&str> = signoffs.lines().map(str::trim).collect();
    let people = std::iter::once(author.trim()).chain(coauthors.lines().map(str::trim).filter(|line| !line.is_empty()));
    for identity in people {
        if !valid_identity(identity) {
            return Err(format!("`{identity}` is not a full `Name <email>`"));
        }
        // Dependabot authors with its noreply address and signs off with support@github.com:
        // the exact known pair is accepted, not bots in general.
        let signed = signoffs.contains(&identity)
            || (identity == DEPENDABOT_AUTHOR && signoffs.contains(&DEPENDABOT_SIGNOFF));
        if !signed {
            return Err(format!("no `Signed-off-by: {identity}`"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHOR: &str = "Contributor <contributor@example.org>";
    const COAUTHOR: &str = "Coauthor <coauthor@example.org>";

    #[test]
    fn the_author_signs_off_as_themselves() {
        assert!(check(AUTHOR, AUTHOR, "").is_ok());
        assert!(check(AUTHOR, &format!("{AUTHOR}\n\n"), "\n").is_ok());
        assert!(check(AUTHOR, "", "").is_err());
        assert!(check(AUTHOR, COAUTHOR, "").is_err(), "somebody else's sign-off");
    }

    #[test]
    fn every_coauthor_signs_off_too() {
        assert!(check(AUTHOR, AUTHOR, COAUTHOR).is_err());
        assert!(check(AUTHOR, &format!("{AUTHOR}\n{COAUTHOR}"), COAUTHOR).is_ok());
    }

    #[test]
    fn an_identity_is_a_name_and_an_email() {
        for identity in ["Contributor", " <user@example.org>", "Name <>", "Name <a b@c>"] {
            assert!(check(identity, identity, "").is_err(), "{identity}");
        }
    }

    #[test]
    fn dependabot_is_its_known_pair_and_nothing_wider() {
        assert!(check(DEPENDABOT_AUTHOR, DEPENDABOT_SIGNOFF, "").is_ok());
        assert!(check(DEPENDABOT_AUTHOR, "", "").is_err());
        assert!(check(DEPENDABOT_AUTHOR, "dependabot[bot] <other@example.org>", "").is_err());
        assert!(check(DEPENDABOT_AUTHOR, DEPENDABOT_SIGNOFF, COAUTHOR).is_err());
        assert!(check(AUTHOR, DEPENDABOT_SIGNOFF, "").is_err());
        assert!(check("Contributor <support@github.com>", DEPENDABOT_SIGNOFF, "").is_err());
    }
}
