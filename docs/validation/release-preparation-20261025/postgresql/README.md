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
