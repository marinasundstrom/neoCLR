# macOS PostgreSQL restoration — 2026-10-26

The author explicitly requires preserving Preview 17's optional PostgreSQL provider.
The earlier base-only macOS archive and measurements remain historical candidate
evidence; they are not the final publication assets. The new archive contains all
five libraries, with the same pinned Raven compiler, runtime and codegen binaries.
The JIT host and library outputs are rebuilt and their identities retained.

The public people-query sample passes. The updated SQL harness imports the native
SDK project configuration rather than legacy CLI core flags. SQL contracts, real
PostgreSQL queries in interpreter/JIT/AOT, and all ten protocol-peer cases pass.
Two direct-CLI harness failures are retained separately from the passing project
path. JSON checks pass all three modes. The first extracted JIT check passed its
ordinary sample/server checks but timed out waiting for its second HTTP server
while other validations ran; isolated retry remains required, without assuming
its cause. Final package, editor/native-kit and measurement gates are pending.
The disposable local PostgreSQL 18 container was stopped after SQL acceptance.

The isolated retry passes extracted JIT and interpreter samples/HTTP. All 24 async
executions, 132 header cases and native-kit console/HTTP/C/C++ checks pass with
the provider-inclusive package. The earlier timeout is preserved; a successful
retry does not establish its cause. Editor, HTTPS and final measurements remain
pending. Final assets are staged separately under `target/preview18-release/final-artifacts`.

All remaining functional gates pass: 26 editor checks, inlays, two ten-cycle
configuration recovery runs, and 15 HTTPS/security/recovery/cleartext cases. The
archive audit verifies 2,418 manifest-listed files and all 469 standalone SDK files.
The provider-inclusive archive's final benchmark refresh remains the last
measurement gate before publication. Windows final measurements have passed.
