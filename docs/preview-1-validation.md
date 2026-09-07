# Preview 1 validation evidence

This is a record of development evidence and outstanding release checks. It does
not designate a release candidate or claim that the publication checklist is complete.
The [Preview 1 plan](preview-1.md) remains the scope and acceptance checklist.

## Stable-toolchain implementation baseline

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
matrix targets Ubuntu, macOS and Windows with stable Rust and a separate 1.85.0 job. A configured job is not
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

## Minimum-toolchain and clean-source validation

Rust 1.85.0 is now the declared and locally tested minimum. Cargo.toml records
rust-version = "1.85". CI retains stable formatting/Clippy/tests and adds a separate
1.85.0 build/test/native-example job on each of the three operating systems. These
jobs have been configured, not observed passing remotely.

The clean-source check used a git archive of 2d2d7ac, extracted into a new temporary
directory with no target directory or generated artifacts. It contained only tracked
source files and used the existing Cargo registry cache. This validates an independent
source build, not first-time dependency downloading or a remote Git clone. The final
Cargo minimum-version declaration was copied into the snapshot and its all-target
build rerun successfully.

| Item | Result |
| --- | --- |
| Environment | macOS 26.6.2 ARM64, rustc 1.85.0 (4d91de4e4, 2025-02-17), Apple clang 17.0.0, GNU Make 3.81 |
| Build | cargo +1.85.0 build --locked --all-targets succeeded from clean source, including vendored libffi |
| Regression suite | One uninterrupted cargo +1.85.0 test --locked run: 521 tests passed; doc tests completed (no doc test cases) |
| Quick start | HelloWorld source execution printed Hello, world! and => Void |
| Artifact workflow | Assemble, verify and run succeeded; a second assembly to the same path was rejected |
| Separate runtime library | System.neo.json assembled from runtime/System.neoil and ran the HelloWorld artifact successfully |
| Console/file input | Input 21 printed 42; file sample printed 42, InvalidFormat and File input handled |
| Expected Fault | Array-bounds sample exited with status 1 and an owned logical stack trace |
| Embedding | invoke example completed with the documented guest invocation results |
| Native calls | build_native built the sample library with the selected toolchain; pinvoke printed 42 and => Void |

The workflow ran 13 command steps and checked their expected exit statuses; output
was also reviewed against the README. Build artifacts remained in the temporary
snapshot. This evidence supports the local minimum-version claim. It does not close
the exact-release-commit, remote-platform, packaging or first-install prerequisite
checks. The earlier stable baseline and this minimum run exercise the same runtime
implementation; later documentation and CI edits do not designate a release candidate.

Local command logs in temporary directories are development aids, not durable
release evidence. Release records should use preserved logs or immutable CI links.
No remote push, tag creation or publication is implied by this document.

## Source packaging review

The follow-up packaging slice inventories all 20 locked registry dependencies and
preserves 42 notice texts with recorded origins and hashes. See
[third-party notices](../THIRD_PARTY_NOTICES.md) and the [source audit](source-release.md).
A local review archive is checked for tracked-tree membership, required sources,
fixtures and notices; it is not a published or versioned release artifact.

Draft release notes are available. Final maintainer provenance review, candidate
selection, remote-platform results and the final archive checksum remain open.
