# Preview 19 publication — 2026-10-31

Preparation began 2026-10-30; publication follows after midnight in Europe/Stockholm.

All [prepublication checks](../release-preparation-20261030/README.md) pass.
The [release manifest](release-manifest.json) and [checksums](SHA256SUMS) identify
all assets. Publication changes only manifest status/date and its checksum from
the prepared candidate; all ten payload asset hashes remain unchanged.
The source archive is the exact validated `0758082e4` archive; release-tag changes
are documentation and evidence only. Publication verification is recorded separately.

## Verified publication and deployment

[Preview 19](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.19)
is published as a prerelease at commit `8f247a6e88c93317fc235ffa3b5760cd859137da`.
[Publication receipt](publication.json) verifies all twelve asset hashes/sizes and
the public checksum download. The timestamp is October 30 UTC / October 31 in
Europe/Stockholm; release metadata uses the author's local publication day.

All 26 website contract tests and four API snapshot tests pass. The
[local build](website-build.log.gz) checks 6,714 pages; existing API description
coverage warnings remain visible. The separate
[manual deployment](https://github.com/marinasundstrom/neoCLR/actions/runs/41141936054)
passes on the release commit. Live homepage, installation, HTTP benchmark and
execution benchmark pages return HTTP 200 and show Preview 19; response hashes and
checked content are recorded in the publication receipt.

The qualified runtime and library bytes, compiler pin and experimental limitations
are unchanged. Python behavior-fixture migration remains post-release work.
