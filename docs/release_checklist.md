# Release Checklist

This is the canonical release procedure for the `range-set-blaze` crate.

## 1. Choose the Release Version

- Pick the next version following SemVer (this crate is pre-1.0: breaking
  changes bump the minor version, e.g. `0.6.1` -> `0.7.0`).
- Confirm its tag does not already exist.
- Start from a clean, current `main` checkout, or a feature branch that is
  even with `main` and ready to become the release PR.

```bash
git status --short --branch
git tag --list 'vX.Y.Z'
```

## 2. Consider Bumping the Pinned Toolchain

- Every release, check whether the pinned stable toolchain
  (`rust-toolchain.toml` `channel`, and the matching `dtolnay/rust-toolchain@`
  pins in `ci.yml`) and the pinned nightly (`nightly :=` in `justfile`, and the
  nightly install/override lines in `ci.yml`) are worth moving forward:

```bash
rustup check
```

- Preview new lints on a newer stable before committing to the bump:

```bash
just clippy-latest
```

- If bumping, update the toolchain pin(s) and fix or explicitly allow any new
  lints (mirroring how `-A deprecated` and, as of 0.7.0,
  `-A clippy::single_range_in_vec_init` are handled for the nightly job) in
  both `ci.yml` and `justfile`.
- It is fine to skip a bump for a given release to keep its scope small, but
  make that a deliberate choice each time, not a default.
- Note any toolchain bump in `docs/CHANGELOG.md` under `### Changed`.

## 3. Update the Version and Changelog

- Update `version` in `Cargo.toml`.
- In `docs/CHANGELOG.md`, rename the `## Unreleased` section to
  `## [X.Y.Z] - YYYY-MM-DD` and confirm it accurately lists `Added`,
  `Changed`, and `Breaking` entries for everything since the last release.
- This command must print no matches once the section is renamed:

```bash
rg -n -i '\bunreleased\b' docs/CHANGELOG.md
```

## 4. Run Release Checks

```bash
just check-all
just ci-full
```

`ci-full` runs Clippy, the stable test suite, doc-link checks, `cargo audit`,
and an all-features publish dry-run.

## 5. Package Preflight

- Review the package inventory so nothing unintended (large artifacts, local
  scratch files) is included:

```bash
cargo package --list
```

- Commit the version bump and changelog edit locally.

## 6. Open the Release PR

- Push the branch (the current feature branch may double as the release
  branch if it is already even with `main`) and open a PR into `main`.
- Require CI to pass before merging.
- Merge into `main`.

## 7. Publish the Crate

Publishing is effectively permanent. Publish only from the clean, merged
`main` commit:

```bash
git checkout main
git pull
git status --short --branch
cargo publish --locked
```

The person performing the release runs the real `cargo publish`; automated
agents must not run it.

- Wait until the exact version resolves from crates.io outside the workspace:

```bash
cd /tmp
cargo info --registry crates-io range-set-blaze@X.Y.Z
```

## 8. Tag and Create the GitHub Release

- Tag the exact published `main` commit and push the annotated tag:

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

- Create a non-draft, non-prerelease GitHub Release from the tag using the
  matching `docs/CHANGELOG.md` section.

## 9. Final Verification

- Verify the exact crate version on crates.io.
- Verify its docs.rs build.
- Confirm the README badges resolve to the new release.
