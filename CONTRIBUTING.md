# Contributing to Hoxorion

This applies to every repository of Hoxorion that has no `CONTRIBUTING.md` of its own.

## Contribution terms

Every repository carries the [Developer Certificate of Origin 1.1](DCO) and the
[Contributor License Agreement](CLA.md). Read both before contributing.

- Sign off every commit with your own name and email: `git commit -s` adds
  `Signed-off-by: Your Name <you@example.com>`. Under the CLA, your sign-off certifies the
  DCO and accepts the CLA for that contribution.
- Each co-author (`Co-authored-by:`) adds their own sign-off. Never sign off for someone else.
- The sign-off is not a GPG signature, and a GPG signature does not replace it.
- Third-party material: say where it comes from and under what licence.

A pull request with a commit missing a sign-off fails the `DCO and CLA` check. To fix it:

```sh
git rebase --signoff master
git push --force-with-lease
```

## Licensing

Every source file opens with its licence and holders:

```text
// SPDX-License-Identifier: MPL-2.0
// Copyright © <year> NullSablex and the Hoxorion contributors
```

The licence is the repository's (see its `LICENSE`); the year is the one the file was
written in. Every `Cargo.toml` says the same licence and
`authors = ["NullSablex and the Hoxorion contributors"]`. The `Licensing` check holds
each repository to this; run it locally from the repository's root with:

```sh
cargo run --manifest-path <path to Hoxorion/.github>/tools/contract/Cargo.toml -- licensing --license MPL-2.0
```

## Commits

Messages follow [Conventional Commits](https://www.conventionalcommits.org/) in English:
`feat(net): …`, `fix: …`, `docs: …`.
