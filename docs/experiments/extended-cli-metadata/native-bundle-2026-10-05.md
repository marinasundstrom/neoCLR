# Extracted native bundle qualification — 2026-10-05

The local macOS arm64 candidate passes its bounded distribution gate. This is not a
published release or complete native System bootstrap. [Procedure](../../native-poc-bundle.md).

- Final packaging output was extracted under `/tmp/native bundle final1005` and verified
  from outside the checkout. All 698 manifest files match; five consumer projects compile,
  collections/identity, Tasks/await and JSON execute with exact output, and live HTTP/JSON
  server/client rounds pass. Library sources are absent from these consumers.
- The installed VSIX from the preceding candidate passes all 19 real VS Code checks,
  including native documentation, reference refresh and build/run. Final candidate changes
  add source-build validation and provenance fields; SDK, VSIX and runtime bytes are unchanged.
  Client logs select the installed extension server, despite reporting an older global SDK
  inventory. Server logs include the intentional missing-reference recovery test.
- Tampering fails hash validation before compilation; a mismatched seed fails packaging
  before creating its output directory. These negative checks exercise the preceding
  candidate's unchanged validation paths.
- Runtime was rebuilt with `cargo +stable build --release --locked --bin neoclr` from
  b90a1ff19b047c022a31333cdfb61333fd85fa4a. The selected source-library build and explicit
  compiler/bootstrap hashes are preserved in provenance. The packaging checkout was dirty
  with this slice's scripts/docs; manifest and packager hash record that rather than
  presenting a clean tagged release.

[Machine-readable evidence](native-bundle-2026-10-05.json) preserves commands, outputs,
source/artifact hashes, declared revisions, editor assertions and log excerpts. The
archive is a local temporary artifact, not a durable download. Hashes establish consistency,
not signing or proof of origin for externally supplied bootstrap binaries.

Website validation passes all 18 focused tests and builds/checks 1,803 pages.

The .NET backend and runtime implementation are unchanged. Existing .NET/editor controls
are reused. Website status now distinguishes local installation acceptance from published
Preview 11 setup. The legacy snapshot audit remains failing; no bridge repair or suppressed
audit is claimed. Remaining release work: reproducible retained-bootstrap provenance,
release version/download selection and publication. Full System ownership stays a separate
milestone, not an implied prerequisite for this bounded POC.
