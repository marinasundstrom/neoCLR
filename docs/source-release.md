# Source-preview packaging and audit

The first public deliverable is a source archive or repository snapshot. It includes
neoCLR's tracked source, runtime IL, samples, fixtures, tests and documentation.
Cargo.lock pins the dependencies; Cargo downloads them separately. Prebuilt binaries,
a vendored dependency bundle and a crates.io publication are outside this target.

## Audit baseline

The inventory at 195c68c contained 322 tracked files: 29 src files, 48 runtime files,
86 examples/fixtures, 80 test files, 72 documentation files and seven root/CI files.
The notice inventory and draft release notes are added by this packaging slice.

A tracked-path review found no target directory, generated .neo.json artifacts,
native binaries, private-key files, .env files or symlinks. A content scan found no
private-key headers or the checked GitHub/AWS credential patterns. These are bounded
checks, not a guarantee that every possible secret or provenance issue is detectable.
Fixture files deliberately include invalid/empty text for I/O tests and must remain.

The dependency inventory covers all 20 registry packages in Cargo.lock, including
build dependencies and windows-link. Preserved license texts are byte-hashed in the
[notice manifest](../third-party/manifest.json). Native libffi and its separately
licensed build/test tooling are recorded explicitly. No dependency source is copied
into the preview archive by this notice inventory.

A review of repository headers and attribution markers did not identify an embedded
third-party implementation copy in src/tests/examples. The documentation attributes
CLI and Raven design references. This review does not establish the historical origin
of every line; project maintainers should review provenance before final publication.
The notices cover the inspected dependency versions and must be updated with Cargo.lock.

## Automated candidate validation

The [next-preview validation guide](next-preview-validation.md) describes the automated
source-archive, notice, full-test and source/artifact checks. Run
`python3 scripts/validate-release.py --toolchain stable` or select Rust 1.85.0. The
script records the exact committed tree and checksum without tagging or publishing.
CI applies this check across the six existing platform/toolchain jobs.

## Assemble a review archive

Run from a clean, committed checkout. Use a new output path; review archives are
local artifacts and do not create a tag or publish anything:

```sh
git status --short
git rev-parse HEAD
git ls-files
git archive --format=tar --prefix=neoclr-source/ --output=/tmp/neoclr-source-review.tar HEAD
tar -tf /tmp/neoclr-source-review.tar
```

On Windows, choose an appropriate temporary path. For a final release, record the
exact commit, selected version/tag, archive filename and its SHA-256. Review the
archive contents, extract it into a new directory and follow the README there.
Do not substitute a working-directory zip that could include ignored build outputs.

The archive must contain Cargo.toml, Cargo.lock, build.rs, LICENSE, README.md,
THIRD_PARTY_NOTICES.md, third-party/manifest.json and every referenced notice text,
plus src/, runtime/, examples/, tests/, docs/ and the CI workflow. Inspect the
runtime include manifest and fixture paths as part of the extracted build check.

`cargo package --locked --list` is a useful secondary inventory, but a Cargo package
is not this deliverable: it adds Cargo-generated metadata and a normalized manifest.
Cargo.toml deliberately retains publish=false. A package-list warning about missing
homepage/repository fields is not evidence that a remote URL should be invented.

## Final release checks

Maintain [the changelog](../CHANGELOG.md) in every development commit using the
[changelog workflow](changelog.md). For a release candidate, promote only the selected
Unreleased entries to its actual version/date; keep previously published sections and
release notes unchanged.

- Refresh dependency metadata and notice hashes against the final Cargo.lock.
- Review any source/fixture/license additions since the audited snapshot.
- Confirm archive membership matches the selected tracked tree and preserve its hash.
- Build/test the extracted archive on claimed platforms using minimum and stable Rust.
- Finish provenance review, platform evidence and the draft release notes before
  selecting publication as a separate action.

See [validation evidence](preview-1-validation.md) for the already tested clean-source
workflows and the [Preview 1 checklist](preview-1.md) for gates still open.
