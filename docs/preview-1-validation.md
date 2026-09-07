# Preview 1 validation evidence

This is a record of development evidence and outstanding release checks. It does
not designate a release candidate or claim that the publication checklist is complete.
The [Preview 1 plan](preview-1.md) remains the scope and acceptance checklist.

## Current local baseline

| Item | Evidence |
| --- | --- |
| Implementation snapshot | 50335a5, generic Equatable contract and implementations |
| Local environment | macOS ARM64; rustc 1.95.0 (59807616e, 2026-04-14) |
| Focused equality/library/interface/sample checks | 43 tests passed |
| Comprehensive regression coverage | 521 tests passed across 82 test binaries, using the initial full run plus corrected native-suite and remaining-suite reruns |
| Additional checks | Doc tests, cargo fmt --check, strict all-target Clippy and git diff --check passed |
| Source/artifact examples | Automated preview walkthrough passed, including console/file behavior, references, union extraction and equality |

The initial full run stopped at an index-dependent native-flags test after the new
interface changed method ordering. The test was corrected to select the intended
native declaration by name; that suite and all subsequent suites passed on rerun.
This is comprehensive local coverage, not evidence of one uninterrupted full run
on a chosen release commit. The work log records the implementation slices.

No Linux or Windows result for this snapshot is recorded here. The configured CI
matrix targets Ubuntu, macOS and Windows with stable Rust. A configured job is not
a passing result, and runner labels do not establish a fixed CPU architecture.

## Evidence to collect for the release candidate

- Identify the exact commit, proposed version/tag and source-package contents.
- Run one uninterrupted cargo test --locked plus formatting and strict Clippy on
  that candidate; retain logs or CI links tied to the commit.
- Record OS, CPU architecture and actual Rust/C toolchain versions for each claimed
  platform. Validate the declared minimum Rust version separately from stable.
- Exercise a clean checkout following the README, including native prerequisites,
  fixture paths, artifact assembly and expected successful/failing exit statuses.
- Run the embedding and native-library examples on claimed platforms, and record
  any platform-specific restrictions in release notes.
- Review license/provenance notices and verify that the source package includes
  runtime sources, fixtures, tests and documentation without local build artifacts.

The README currently claims Rust 1.85 or newer, while Cargo.toml has no rust-version
and CI tests stable only. That minimum is unverified here. Determining, recording
and testing a truthful minimum is the next build-validation slice.

Local command logs in temporary directories are development aids, not durable
release evidence. Release records should use preserved logs or immutable CI links.
No remote push, tag creation or publication is implied by this document.
