# Next-preview validation

The selected candidate is v0.1.0-preview.2, a source-only prerelease dated 2026-09-08.
See [Preview 2 release notes](preview-2-release-notes.md).
The [changelog](../CHANGELOG.md) records features and migration guidance. Published
Preview 1 notes and evidence remain frozen.

## Implementation scope

- Repeated Neo declarations renew local storage without changing ordinary array
  assignment or invalidating live managed aliases.
- A complete Neo collection example creates ArrayList<Counter&>, uses a List view,
  mutates referenced objects and demonstrates descriptor-copy behavior.
- Neo projects ordinary and output reference contracts, typed uninitialized locals,
  forwarding and conditional library outputs.
- Live managed references support concrete GetType discovery through interface views.
- Preliminary ReferenceEquals compares managed locations independently of value equality.

These are implemented. Automatic block cleanup/destruction, readonly references,
inheritance, reflective invocation, pinning, persistent host roots and broader compiler
features remain outside this candidate. Verification remains optional for IL/artifact
execution and part of Neo compilation; runtime provenance/output checks remain mandatory.

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
5. Verifies and runs nine Neo programs as source and JSON artifacts, checking identical
   output, then builds and runs the native interop sample from the extracted tree.
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
