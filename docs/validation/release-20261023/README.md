# Preview 17 qualification — 2026-10-23

Matching macOS ARM64 and Windows x64 SDK/editor `0.1.17-neoclr.20261022.1`.
All required source, package, editor, HTTPS and measurement gates pass.
[Final asset identities](release-assets.json) record exact inputs, archive hashes,
standalone SDK/bundle matching and dependency-notice coverage.

- [Source and package evidence](../release-stabilization-20261022/README.md)
- [macOS final runtime/API and native-kit checks](../release-stabilization-20261022/versioned-macos/summary.json)
- [Windows final runtime/editor/HTTPS checks](../release-stabilization-20261022/versioned-windows/summary.json)
- [macOS measurements](../../../benchmarks/release-20261022/macos-preview17/README.md)
- [Windows measurements](../../../benchmarks/release-20261023/windows-preview17/README.md)
- [Local SDK/extension installation and rollback](../../local-preview-20261022.md)

Each platform passes 24 await executions, three JSON executions, 132 header cases,
26 editor checks, inlays and two ten-cycle configuration recovery runs. HTTPS has
15 macOS and 27 Windows security/recovery/cleartext cases. Each measurement series
validates 55 CPU/JSON processes and 1,152 HTTP responses. These counts describe
bounded tests, not arbitrary application compatibility or peak-throughput claims.
Publication confirmation is recorded separately after release assets are uploaded.
