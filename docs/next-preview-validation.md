# Next-preview validation

This guide now targets Unreleased development after v0.1.0-preview.2. No next version,
release date or release commit is selected. [Preview 2 release notes](preview-2-release-notes.md)
and published evidence remain historical records. The [changelog](../CHANGELOG.md)
records implemented features and migration guidance.

## Validation scope

The current library milestone covers readonly managed references, inheritance and
constructor/interface dispatch, generic functions/delegates/closures, reflection,
common interfaces and collections, ordinal text/character helpers, Math, separate
Date/Time values, the host local clock, guest Environment arguments, lexical Path
operations and bounded UTF-8 file I/O. These are preview subsets, not a claim of
complete .NET class-library parity. See [library scope](library-preview.md) and
individual API design documents for the researched .NET comparisons and limits.

Validation should prove those existing contracts, not expand APIs simply to match a
BCL inventory. Parsing/formatting/globalization, broader filesystem services and
asynchronous I/O remain deferred. Verification is mandatory for Neo compilation and
optional for direct IL/artifact execution; runtime provenance checks remain active.

## Reproducible source-archive check

With Python 3.9+, Git, Rust and the platform's existing native build prerequisites:

```sh
python3 scripts/validate-release.py --toolchain stable
python3 scripts/validate-release.py --toolchain 1.85.0
```

On Windows the Python command may be `python`. Use `--revision <commit>` to select a
candidate and `--output <new-directory>` to preserve results at a chosen location.
The default is HEAD and a fresh temporary directory. The script validates committed
content; uncommitted working-tree changes are not included in its archive.

The check:

1. Resolves one exact commit and archives its tracked source with git archive,
   disabling line-ending conversion so notice bytes agree on Windows and Unix.
2. Rejects links/special files and unexpected archive paths; compares archive membership
   with the tracked tree and checks required source/release files.
3. Checks the dependency notice inventory against Cargo.lock and verifies notice hashes.
4. Runs all test targets from the extracted source on the selected Rust toolchain,
   using a fresh target directory, then builds the executable.
5. Verifies and runs 23 deterministic Neo programs as source and JSON artifacts,
   checking identical output. Separately checks Environment arguments and process
   reads, local-clock snapshots against the host instant, and report-file contents
   from source and artifacts. Then builds and runs the native interop sample.
6. Writes report.json with the commit, archive SHA-256, platform, rustc version,
   notice/file counts, smoke programs, full-test status and overall result.

`--smoke-only` skips the full suite for a quicker packaging check. Its report explicitly
records `full_tests: false`; it cannot substitute for the full release gate. A failing
check records failure and returns nonzero. Archive membership and hash checks are bounded
packaging checks, not a complete security or provenance audit.

## Platform matrix and release gates

CI now runs the archive check on Linux, macOS and Windows with both stable and Rust
1.85.0. The stable jobs also run formatting and strict Clippy. Reports and source
archives are retained as per-job CI artifacts, including reports from failed runs.
A green run for a different commit does not certify this candidate.

Before publication, require:

- Successful six-job CI for the exact selected release commit, with passing full-test
  archive reports on every claimed runner/toolchain combination.
- Review of release notes, changelog migration instructions, dependency notices,
  source additions and the actual artifacts/checksums being published.
- Explicit selection of version/date and publication scope. Do not publish development
  smoke reports as evidence of untested platforms or prebuilt-binary support.

The validator does not push, tag, upload a release or change package versions. CI runs
require a pushed commit or pull request. Local macOS evidence is not Linux/Windows
validation. See [source release](source-release.md) for the remaining publication process.

## Current development evidence

See the [library-preview validation record](experiments/library-preview-validation/README.md)
for the exact macOS commits, toolchains and scope of the locally recorded checks.
These records do not select a release or replace the full cross-platform CI gate.
