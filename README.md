# Hoxorion/.github

The organization's base repository: what every Hoxorion repository shares. It is kept by
the maintainer and does not take outside contributions; propose a change to these files
in an issue of the repository it affects.

What it holds:

- `CONTRIBUTING.md` and `pull_request_template.md`: the defaults GitHub shows in a repository
  that has none of its own.
- `CLA.md` and `DCO`: the contribution terms. Each repository carries its own copy, since a
  contribution is accepted under the terms in the repository it goes to.
- `.github/workflows/contract.yml`: the shared check, called by each repository's
  `Contract` workflow (template in `workflow-templates/`).
- `tools/contract`: the checker it runs (Rust, no dependencies).

Licensed MPL-2.0 (see [`LICENSE`](LICENSE)). Copyright © 2026 NullSablex and the Hoxorion
contributors.
